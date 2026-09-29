use super::*;
use serde_json::json;

#[test]
fn job_status_serialization_roundtrips() {
    for (status, expected_str) in [
        (JobStatus::Queued, "\"queued\""),
        (JobStatus::Running, "\"running\""),
        (JobStatus::AwaitingApproval, "\"awaiting_approval\""),
        (JobStatus::Completed, "\"completed\""),
        (JobStatus::Failed, "\"failed\""),
        (JobStatus::Cancelled, "\"cancelled\""),
    ] {
        let serialized = serde_json::to_string(&status).expect("must serialize");
        assert_eq!(serialized, expected_str);
        let deserialized: JobStatus = serde_json::from_str(&serialized).expect("must deserialize");
        assert_eq!(deserialized, status);
    }
}

#[test]
fn coworker_serde_roundtrips() {
    let coworker = Coworker {
        id: "security".to_string(),
        name: "Security Auditor".to_string(),
        description: "Audits repository dependencies and configurations".to_string(),
        capabilities: vec![
            "sast".to_string(),
            "cve".to_string(),
            "container".to_string(),
        ],
    };

    let serialized = serde_json::to_string(&coworker).expect("serialize coworker");
    let deserialized: Coworker = serde_json::from_str(&serialized).expect("deserialize coworker");

    assert_eq!(deserialized.id, "security");
    assert_eq!(deserialized.name, "Security Auditor");
    assert_eq!(deserialized.capabilities.len(), 3);
}

#[test]
fn worker_status_serde_roundtrips() {
    let status = WorkerStatus {
        running: true,
        port: Some(8899),
        version: Some("0.1.0".to_string()),
    };

    let serialized = serde_json::to_string(&status).expect("serialize WorkerStatus");
    let deserialized: WorkerStatus =
        serde_json::from_str(&serialized).expect("deserialize WorkerStatus");

    assert!(deserialized.running);
    assert_eq!(deserialized.port, Some(8899));
    assert_eq!(deserialized.version.as_deref(), Some("0.1.0"));
}

#[test]
fn job_request_and_state_serde() {
    let req = JobRequest {
        coworker: "cloud".to_string(),
        goal: "Deploy terraform stack to us-east-1".to_string(),
        context: Some("us-east-1 region".to_string()),
    };

    let req_json = serde_json::to_string(&req).expect("serialize JobRequest");
    let parsed_req: JobRequest = serde_json::from_str(&req_json).expect("deserialize JobRequest");
    assert_eq!(parsed_req.coworker, "cloud");
    assert_eq!(parsed_req.goal, "Deploy terraform stack to us-east-1");

    let state = JobState {
        job_id: "job-123".to_string(),
        coworker: "cloud".to_string(),
        goal: "Deploy terraform stack to us-east-1".to_string(),
        status: JobStatus::AwaitingApproval,
        progress: "Terraform plan generated, awaiting approval to apply".to_string(),
        result: None,
        error: None,
        approvals_needed: vec![ApprovalGate {
            action_id: "act-456".to_string(),
            description: "Approve terraform apply in us-east-1".to_string(),
            payload: Some(json!({"resources_to_add": 3})),
        }],
    };

    let state_json = serde_json::to_string(&state).expect("serialize JobState");
    let parsed_state: JobState = serde_json::from_str(&state_json).expect("deserialize JobState");
    assert_eq!(parsed_state.job_id, "job-123");
    assert_eq!(parsed_state.status, JobStatus::AwaitingApproval);
    assert_eq!(parsed_state.approvals_needed.len(), 1);
    assert_eq!(parsed_state.approvals_needed[0].action_id, "act-456");
}
