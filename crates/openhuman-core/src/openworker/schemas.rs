//! JSON-RPC controllers for the OpenWorker domain.
//!
//! RPC namespace: `openworker.*`
//!
//! Registered in `crates/openhuman-core/src/core/all.rs` under
//! `DomainGroup::Integrations`.

use serde_json::{Map, Value};

use crate::core::all::{ControllerFuture, RegisteredController};
use crate::core::{ControllerSchema, FieldSchema, TypeSchema};

use super::ops;

// ── Schema builder ────────────────────────────────────────────────────────────

fn schema(
    function: &'static str,
    description: &'static str,
    inputs: Vec<FieldSchema>,
    outputs: Vec<FieldSchema>,
) -> ControllerSchema {
    ControllerSchema {
        namespace: "openworker",
        function,
        description,
        inputs,
        outputs,
    }
}

fn job_id_input() -> FieldSchema {
    FieldSchema {
        name: "job_id",
        ty: TypeSchema::String,
        comment: "Job id returned by openworker.delegate.",
        required: true,
    }
}

fn job_id_output() -> FieldSchema {
    FieldSchema {
        name: "job_id",
        ty: TypeSchema::String,
        comment: "Unique identifier for the submitted job.",
        required: true,
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

fn handle_status(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let status = ops::worker_status().await;
        serde_json::to_value(status).map_err(|e| format!("serialization: {e}"))
    })
}

fn handle_list_coworkers(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        ops::list_coworkers()
            .await
            .map_err(|e| e.to_string())
            .and_then(|v| serde_json::to_value(v).map_err(|e| format!("serialization: {e}")))
    })
}

fn handle_delegate(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let coworker = params
            .get("coworker")
            .and_then(|v| v.as_str())
            .ok_or("missing required param: coworker")?
            .to_string();
        let goal = params
            .get("goal")
            .and_then(|v| v.as_str())
            .ok_or("missing required param: goal")?
            .to_string();
        let context = params
            .get("context")
            .and_then(|v| v.as_str())
            .map(String::from);

        let job_id = ops::delegate_task(&coworker, &goal, context.as_deref())
            .await
            .map_err(|e| e.to_string())?;

        Ok(Value::String(job_id))
    })
}

fn handle_job_state(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let job_id = params
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or("missing required param: job_id")?
            .to_string();

        ops::job_state(&job_id)
            .await
            .map_err(|e| e.to_string())
            .and_then(|v| serde_json::to_value(v).map_err(|e| format!("serialization: {e}")))
    })
}

fn handle_approve(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let job_id = params
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or("missing required param: job_id")?
            .to_string();
        let action_id = params
            .get("action_id")
            .and_then(|v| v.as_str())
            .ok_or("missing required param: action_id")?
            .to_string();
        let approved = params
            .get("approved")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        ops::approve_action(&job_id, &action_id, approved)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Value::Bool(true))
    })
}

fn handle_cancel(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let job_id = params
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or("missing required param: job_id")?
            .to_string();

        ops::cancel_job(&job_id).await.map_err(|e| e.to_string())?;
        Ok(Value::Bool(true))
    })
}

// ── Registration ──────────────────────────────────────────────────────────────

/// Returns all OpenWorker controllers for registration in `core::all`.
/// The caller supplies the [`crate::core::all::DomainGroup`] via `push()`.
pub fn all_openworker_registered_controllers() -> Vec<RegisteredController> {
    vec![
        RegisteredController {
            schema: schema(
                "status",
                "Get the current status of the local OpenWorker specialist server.",
                vec![],
                vec![
                    FieldSchema {
                        name: "running",
                        ty: TypeSchema::Bool,
                        comment: "Whether the OpenWorker server is reachable.",
                        required: true,
                    },
                    FieldSchema {
                        name: "port",
                        ty: TypeSchema::U64,
                        comment: "TCP port the server is listening on.",
                        required: false,
                    },
                ],
            ),
            handler: handle_status,
        },
        RegisteredController {
            schema: schema(
                "list_coworkers",
                "List available OpenWorker specialist coworkers.",
                vec![],
                vec![FieldSchema {
                    name: "coworkers",
                    ty: TypeSchema::Json,
                    comment: "Array of coworker descriptors.",
                    required: true,
                }],
            ),
            handler: handle_list_coworkers,
        },
        RegisteredController {
            schema: schema(
                "delegate",
                "Delegate a specialist task to an OpenWorker coworker. Returns a job_id.",
                vec![
                    FieldSchema {
                        name: "coworker",
                        ty: TypeSchema::String,
                        comment: "Coworker id (from openworker.list_coworkers).",
                        required: true,
                    },
                    FieldSchema {
                        name: "goal",
                        ty: TypeSchema::String,
                        comment: "Natural-language description of the desired outcome.",
                        required: true,
                    },
                    FieldSchema {
                        name: "context",
                        ty: TypeSchema::String,
                        comment: "Optional context from the current conversation.",
                        required: false,
                    },
                ],
                vec![job_id_output()],
            ),
            handler: handle_delegate,
        },
        RegisteredController {
            schema: schema(
                "job_state",
                "Get the current state, progress, and result of an OpenWorker job.",
                vec![job_id_input()],
                vec![FieldSchema {
                    name: "state",
                    ty: TypeSchema::Json,
                    comment: "Full JobState object.",
                    required: true,
                }],
            ),
            handler: handle_job_state,
        },
        RegisteredController {
            schema: schema(
                "approve",
                "Approve or deny a pending approval gate on a running OpenWorker job.",
                vec![
                    job_id_input(),
                    FieldSchema {
                        name: "action_id",
                        ty: TypeSchema::String,
                        comment: "Action id from the approval gate.",
                        required: true,
                    },
                    FieldSchema {
                        name: "approved",
                        ty: TypeSchema::Bool,
                        comment: "true to approve, false to deny.",
                        required: false,
                    },
                ],
                vec![],
            ),
            handler: handle_approve,
        },
        RegisteredController {
            schema: schema(
                "cancel",
                "Cancel a running or queued OpenWorker job.",
                vec![job_id_input()],
                vec![],
            ),
            handler: handle_cancel,
        },
    ]
}

/// Returns all OpenWorker controller schemas.
pub fn all_openworker_controller_schemas() -> Vec<ControllerSchema> {
    all_openworker_registered_controllers()
        .into_iter()
        .map(|c| c.schema)
        .collect()
}

#[cfg(test)]
#[path = "schemas_tests.rs"]
mod tests;
