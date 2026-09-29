use super::*;

#[test]
fn delegate_tool_metadata_is_correct() {
    let tool = DelegateToOpenworkerTool;
    assert_eq!(tool.name(), "delegate_to_openworker");
    assert!(tool.description().contains("OpenWorker"));
    assert_eq!(tool.permission_level(), PermissionLevel::Write);

    let schema = tool.parameters_schema();
    assert_eq!(schema["type"], "object");
    let req = schema["required"].as_array().expect("required array");
    assert!(req.contains(&json!("coworker")));
    assert!(req.contains(&json!("goal")));
    assert!(schema["properties"]["coworker"].is_object());
    assert!(schema["properties"]["goal"].is_object());
    assert!(schema["properties"]["context"].is_object());
}

#[tokio::test]
async fn delegate_tool_missing_params_fails() {
    let tool = DelegateToOpenworkerTool;
    // Missing both coworker and goal
    let res = tool.execute(json!({})).await;
    assert!(res.is_err());

    // Missing goal
    let res = tool.execute(json!({"coworker": "security"})).await;
    assert!(res.is_err());

    // Missing coworker
    let res = tool.execute(json!({"goal": "scan code"})).await;
    assert!(res.is_err());
}
