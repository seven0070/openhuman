//! OpenWorker integration domain.
//!
//! JARVIS delegates specialist tasks (security audits, cloud posture, document
//! drafting, Slack/calendar automations, standing schedules) to a locally-running
//! OpenWorker Python sub-process. This domain owns:
//!
//! - [`client`]   — async HTTP client for the OpenWorker FastAPI server
//! - [`types`]    — wire types for job requests, results, and worker status
//! - [`ops`]      — business operations: delegate, status, list coworkers
//! - [`schemas`]  — JSON-RPC controllers registered in `core::all`
//! - [`tools`]    — agent tool callable by the LLM: `delegate_to_openworker`

pub mod client;
pub mod ops;
pub mod schemas;
pub mod tools;
pub mod types;
