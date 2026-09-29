//! Wire types for OpenWorker API.

use serde::{Deserialize, Serialize};

/// Specialist coworker descriptor returned by `GET /api/coworkers`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coworker {
    pub id: String,
    pub name: String,
    pub description: String,
    /// High-level tool categories this coworker uses (e.g. "security", "slack").
    pub capabilities: Vec<String>,
}

/// Request body for `POST /api/job`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRequest {
    /// Coworker id from [`Coworker::id`].
    pub coworker: String,
    /// Natural-language goal for the coworker.
    pub goal: String,
    /// Optional caller context injected as conversation context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Response from `POST /api/job`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStarted {
    pub job_id: String,
}

/// Status values for a running or completed job.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    AwaitingApproval,
    Completed,
    Failed,
    Cancelled,
}

/// Full job state returned by `GET /api/job/{id}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobState {
    pub job_id: String,
    pub coworker: String,
    pub goal: String,
    pub status: JobStatus,
    /// Human-readable progress description.
    #[serde(default)]
    pub progress: String,
    /// Finished result / deliverable (populated when status is Completed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Error message (populated when status is Failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Pending approval gate actions (populated when AwaitingApproval).
    #[serde(default)]
    pub approvals_needed: Vec<ApprovalGate>,
}

/// A single approval gate item — an action OpenWorker wants the user to approve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalGate {
    pub action_id: String,
    pub description: String,
    /// JSON-serialized action payload for context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// Response body for `POST /api/job/{id}/approve`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub action_id: String,
    pub approved: bool,
}

/// OpenWorker server health/status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerStatus {
    pub running: bool,
    pub port: Option<u16>,
    pub version: Option<String>,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;

