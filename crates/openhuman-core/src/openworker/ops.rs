//! Business operations for the OpenWorker domain.

use anyhow::Result;

use super::client;
use super::types::{Coworker, JobRequest, JobState, WorkerStatus};

/// Check whether the OpenWorker server is reachable.
pub async fn worker_status() -> WorkerStatus {
    client::probe_status().await
}

/// Return the list of available specialist coworkers.
pub async fn list_coworkers() -> Result<Vec<Coworker>> {
    client::list_coworkers().await
}

/// Delegate a task to the named specialist coworker.
///
/// `coworker` must match a [`Coworker::id`] from [`list_coworkers`].
/// Returns the job id that can be polled with [`job_state`].
pub async fn delegate_task(coworker: &str, goal: &str, context: Option<&str>) -> Result<String> {
    let req = JobRequest {
        coworker: coworker.to_string(),
        goal: goal.to_string(),
        context: context.map(String::from),
    };
    client::start_job(req).await
}

/// Get the current state of a previously submitted job.
pub async fn job_state(job_id: &str) -> Result<JobState> {
    client::get_job(job_id).await
}

/// Respond to an approval gate on a running job.
pub async fn approve_action(job_id: &str, action_id: &str, approved: bool) -> Result<()> {
    client::respond_to_approval(job_id, action_id, approved).await
}

/// Cancel a running or queued job.
pub async fn cancel_job(job_id: &str) -> Result<()> {
    client::cancel_job(job_id).await
}
