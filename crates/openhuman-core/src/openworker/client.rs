//! Async HTTP client for the OpenWorker FastAPI server.
//!
//! The client connects to `http://127.0.0.1:<port>` where the OpenWorker
//! Python subprocess is listening. The port is configurable; the default is
//! 8899 and can be overridden via the `OPENWORKER_PORT` environment variable.

use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use super::types::{ApprovalRequest, Coworker, JobRequest, JobStarted, JobState, WorkerStatus};

/// Default port the OpenWorker FastAPI server listens on.
pub const DEFAULT_PORT: u16 = 8899;

/// Lazily-initialized HTTP client (shared across calls).
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build OpenWorker HTTP client")
    })
}

fn base_url() -> String {
    let port = std::env::var("OPENWORKER_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);
    format!("http://127.0.0.1:{port}")
}

// ── Health ────────────────────────────────────────────────────────────────────

/// Probe the OpenWorker server. Returns `WorkerStatus` regardless of whether
/// the server is reachable (sets `running = false` on connection error).
pub async fn probe_status() -> WorkerStatus {
    let url = format!("{}/api/health", base_url());
    match client().get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let version = resp
                .json::<serde_json::Value>()
                .await
                .ok()
                .and_then(|v| v.get("version").and_then(|s| s.as_str()).map(String::from));
            WorkerStatus {
                running: true,
                port: Some(
                    std::env::var("OPENWORKER_PORT")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(DEFAULT_PORT),
                ),
                version,
            }
        }
        _ => WorkerStatus {
            running: false,
            port: None,
            version: None,
        },
    }
}

// ── Coworkers ─────────────────────────────────────────────────────────────────

/// List available specialist coworkers.
pub async fn list_coworkers() -> Result<Vec<Coworker>> {
    let url = format!("{}/api/coworkers", base_url());
    let resp = client()
        .get(&url)
        .send()
        .await
        .context("connecting to OpenWorker server")?;

    if !resp.status().is_success() {
        bail!("OpenWorker /api/coworkers returned HTTP {}", resp.status());
    }

    resp.json::<Vec<Coworker>>()
        .await
        .context("parsing coworker list")
}

// ── Jobs ──────────────────────────────────────────────────────────────────────

/// Submit a new job to OpenWorker. Returns the job id.
pub async fn start_job(req: JobRequest) -> Result<String> {
    let url = format!("{}/api/job", base_url());
    let resp = client()
        .post(&url)
        .json(&req)
        .send()
        .await
        .context("submitting job to OpenWorker")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("OpenWorker /api/job returned HTTP {status}: {body}");
    }

    let started: JobStarted = resp.json().await.context("parsing job start response")?;
    Ok(started.job_id)
}

/// Get the current state of a job.
pub async fn get_job(job_id: &str) -> Result<JobState> {
    let url = format!("{}/api/job/{job_id}", base_url());
    let resp = client()
        .get(&url)
        .send()
        .await
        .context("fetching job state from OpenWorker")?;

    if !resp.status().is_success() {
        bail!(
            "OpenWorker /api/job/{job_id} returned HTTP {}",
            resp.status()
        );
    }

    resp.json::<JobState>().await.context("parsing job state")
}

/// Approve or deny a pending action gate on a running job.
pub async fn respond_to_approval(job_id: &str, action_id: &str, approved: bool) -> Result<()> {
    let url = format!("{}/api/job/{job_id}/approve", base_url());
    let body = ApprovalRequest {
        action_id: action_id.to_string(),
        approved,
    };
    let resp = client()
        .post(&url)
        .json(&body)
        .send()
        .await
        .context("sending approval to OpenWorker")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("OpenWorker /api/job/{job_id}/approve returned HTTP {status}: {body}");
    }
    Ok(())
}

/// Cancel a running or queued job.
pub async fn cancel_job(job_id: &str) -> Result<()> {
    let url = format!("{}/api/job/{job_id}/cancel", base_url());
    let resp = client()
        .post(&url)
        .send()
        .await
        .context("cancelling OpenWorker job")?;

    if !resp.status().is_success() {
        bail!(
            "OpenWorker /api/job/{job_id}/cancel returned HTTP {}",
            resp.status()
        );
    }
    Ok(())
}
