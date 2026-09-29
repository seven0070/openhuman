//! Agent tool: `delegate_to_openworker`
//!
//! Surfaced to the JARVIS LLM so it can autonomously delegate specialist
//! tasks to the local OpenWorker sub-process.
//!
//! Re-exported through `crates/openhuman-core/src/tools/mod.rs`.

use async_trait::async_trait;
use serde_json::json;

use tinytools::{PermissionLevel, Tool, ToolResult};

use super::ops;

pub const DELEGATE_TOOL_NAME: &str = "delegate_to_openworker";

/// Delegates a specialist task to an OpenWorker coworker.
///
/// JARVIS calls this when the user requests work that maps to a specialist
/// domain: security audits, cloud posture, document drafting, Slack/calendar
/// management, or standing automation schedules.
pub struct DelegateToOpenworkerTool;

#[async_trait]
impl Tool for DelegateToOpenworkerTool {
    fn name(&self) -> &str {
        DELEGATE_TOOL_NAME
    }

    fn description(&self) -> &str {
        "Delegate a specialist task to the local OpenWorker AI coworker server. \
         Use for: security code audits, dependency scans, cloud posture checks, \
         polished document/spreadsheet production, Slack thread management, \
         calendar scheduling, and standing automation schedules (morning briefs, \
         weekly reports). Provide the coworker id and the goal; the tool returns \
         a job_id you can track with openworker.job_state."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "required": ["coworker", "goal"],
            "properties": {
                "coworker": {
                    "type": "string",
                    "description": "Coworker id. Common values: 'security', 'cloud', \
                                    'document', 'slack', 'calendar', 'automation'. \
                                    Use openworker.list_coworkers to see available specialists."
                },
                "goal": {
                    "type": "string",
                    "description": "Clear natural-language description of the desired outcome."
                },
                "context": {
                    "type": "string",
                    "description": "Optional context from the current conversation to \
                                    inject into the coworker's session."
                }
            }
        })
    }

    fn permission_level(&self) -> PermissionLevel {
        // Initiates a long-running external process — treat as a write-level action.
        PermissionLevel::Write
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let coworker = args
            .get("coworker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: coworker"))?;
        let goal = args
            .get("goal")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing required parameter: goal"))?;
        let context = args.get("context").and_then(|v| v.as_str());

        // Probe liveness before submitting.
        let status = ops::worker_status().await;
        if !status.running {
            return Ok(ToolResult::error(
                "OpenWorker specialist server is not running. \
                 Ask the user to start it from the JARVIS OpenWorker panel, \
                 or call the openworker_start Tauri command.",
            ));
        }

        let job_id = ops::delegate_task(coworker, goal, context)
            .await
            .map_err(|e| anyhow::anyhow!("failed to submit job to OpenWorker: {e}"))?;

        Ok(ToolResult::success(
            json!({
                "job_id": job_id,
                "coworker": coworker,
                "message": format!(
                    "Task submitted to the '{coworker}' specialist. \
                     Job id: {job_id}. \
                     Use openworker.job_state to poll progress."
                )
            })
            .to_string(),
        ))
    }
}

#[cfg(test)]
#[path = "tools_tests.rs"]
mod tests;
