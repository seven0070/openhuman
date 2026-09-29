//! OpenWorker Python sub-process lifecycle manager.
//!
//! Manages the OpenWorker FastAPI server as a sidecar process alongside the
//! embedded Rust core. On launch, JARVIS optionally auto-starts OpenWorker
//! (configurable via `jarvis.config.toml`). The renderer can start/stop/query
//! the server via the Tauri commands exported here.
//!
//! Python environment strategy
//! ----------------------------
//! The vendored OpenWorker lives at `<app_resource_dir>/openworker/` (or
//! `vendor/openworker/` in dev mode). We install it into a `.venv` sub-dir
//! using the system Python (>=3.10). On first start we run `python -m venv`
//! followed by `pip install -e .`. Subsequent starts skip install if
//! `.venv/pyvenv.cfg` exists and the `openworker` package is importable.

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use tauri::{AppHandle, State};
use tokio::process::{Child, Command};

/// The default TCP port OpenWorker listens on.
const OPENWORKER_PORT: u16 = 8899;

/// Shared handle to the running OpenWorker subprocess.
pub struct OpenWorkerHandle {
    child: Arc<Mutex<Option<Child>>>,
    port: u16,
}

impl OpenWorkerHandle {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            port: OPENWORKER_PORT,
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().is_some()
    }
}

impl Default for OpenWorkerHandle {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Resolve the OpenWorker source directory.
/// In dev mode: `<workspace>/vendor/openworker/`
/// In release: `<resource_dir>/openworker/`
fn openworker_dir(_app: &AppHandle) -> PathBuf {
    // Dev: the workspace root is two levels above the crate manifest dir.
    #[cfg(debug_assertions)]
    {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest
            .parent() // crates/
            .and_then(|p| p.parent()) // workspace root
            .map(|root| root.join("vendor").join("openworker"))
            .unwrap_or_else(|| manifest.join("vendor").join("openworker"))
    }

    #[cfg(not(debug_assertions))]
    {
        use tauri::Manager;
        _app.path()
            .resource_dir()
            .expect("resource_dir unavailable")
            .join("openworker")
    }
}

/// Path to the venv Python interpreter.
fn venv_python(dir: &PathBuf) -> PathBuf {
    let venv = dir.join(".venv");
    #[cfg(target_os = "windows")]
    return venv.join("Scripts").join("python.exe");
    #[cfg(not(target_os = "windows"))]
    return venv.join("bin").join("python");
}

/// Ensure the Python venv exists and OpenWorker is installed into it.
async fn ensure_venv(dir: &PathBuf) -> Result<(), String> {
    let python = venv_python(dir);
    if python.exists() {
        // venv already present — check importability.
        let check = tokio::process::Command::new(&python)
            .args(["-c", "import openworker"])
            .current_dir(dir)
            .output()
            .await
            .map_err(|e| format!("venv python check failed: {e}"))?;
        if check.status.success() {
            log::debug!("[openworker] venv OK, openworker importable");
            return Ok(());
        }
        log::warn!("[openworker] venv exists but openworker not importable; reinstalling");
    }

    // Create venv.
    log::info!("[openworker] creating Python venv at {:?}", dir.join(".venv"));
    let out = tokio::process::Command::new("python")
        .args(["-m", "venv", ".venv"])
        .current_dir(dir)
        .output()
        .await
        .map_err(|e| format!("python -m venv failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "venv creation failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    // Install OpenWorker (editable).
    log::info!("[openworker] installing openworker into venv");
    let pip = dir.join(".venv");
    #[cfg(target_os = "windows")]
    let pip = pip.join("Scripts").join("pip.exe");
    #[cfg(not(target_os = "windows"))]
    let pip = pip.join("bin").join("pip");

    let out = tokio::process::Command::new(&pip)
        .args(["install", "-e", "."])
        .current_dir(dir)
        .output()
        .await
        .map_err(|e| format!("pip install failed: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "openworker install failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    log::info!("[openworker] venv ready");
    Ok(())
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Start the OpenWorker specialist server.
///
/// Idempotent: if a child is already tracked as running, returns Ok immediately.
#[tauri::command]
pub async fn openworker_start(
    app: AppHandle,
    handle: State<'_, OpenWorkerHandle>,
) -> Result<(), String> {
    if handle.is_running() {
        log::debug!("[openworker] already running");
        return Ok(());
    }

    let dir = openworker_dir(&app);
    if !dir.exists() {
        return Err(format!(
            "OpenWorker source not found at {:?}. Ensure the vendor/openworker submodule is initialized.",
            dir
        ));
    }

    ensure_venv(&dir).await?;

    let python = venv_python(&dir);
    let port = handle.port();

    log::info!("[openworker] spawning server on port {port}");
    let child = Command::new(&python)
        .args(["-m", "openworker", "serve", "--port", &port.to_string()])
        .current_dir(&dir)
        .env("OPENWORKER_PORT", port.to_string())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("failed to spawn openworker server: {e}"))?;

    *handle.child.lock() = Some(child);

    // Give the server a moment to bind.
    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
    log::info!("[openworker] server started on port {port}");

    Ok(())
}

/// Stop the OpenWorker specialist server.
#[tauri::command]
pub async fn openworker_stop(handle: State<'_, OpenWorkerHandle>) -> Result<(), String> {
    let maybe_child = handle.child.lock().take();
    if let Some(mut child) = maybe_child {
        child
            .kill()
            .await
            .map_err(|e| format!("failed to kill openworker: {e}"))?;
        log::info!("[openworker] server stopped");
    }
    Ok(())
}

/// Query whether the OpenWorker server is running.
#[tauri::command]
pub async fn openworker_status_cmd(
    handle: State<'_, OpenWorkerHandle>,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "running": handle.is_running(),
        "port": handle.port(),
    }))
}
