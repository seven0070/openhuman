use super::*;

#[test]
fn all_openworker_controller_schemas_are_valid() {
    let schemas = all_openworker_controller_schemas();
    assert_eq!(schemas.len(), 6, "Expected 6 OpenWorker controller schemas");

    let expected_functions = [
        "status",
        "list_coworkers",
        "delegate",
        "job_state",
        "approve",
        "cancel",
    ];

    for func in expected_functions {
        let matching = schemas.iter().find(|s| s.function == func);
        assert!(
            matching.is_some(),
            "Expected to find schema for function '{func}'"
        );
        let s = matching.unwrap();
        assert_eq!(s.namespace, "openworker");
        assert!(!s.description.is_empty());
    }
}

#[test]
fn delegate_schema_has_required_inputs() {
    let schemas = all_openworker_controller_schemas();
    let delegate_schema = schemas
        .iter()
        .find(|s| s.function == "delegate")
        .expect("delegate schema exists");

    let coworker_field = delegate_schema
        .inputs
        .iter()
        .find(|f| f.name == "coworker")
        .expect("has coworker field");
    assert!(coworker_field.required);

    let goal_field = delegate_schema
        .inputs
        .iter()
        .find(|f| f.name == "goal")
        .expect("has goal field");
    assert!(goal_field.required);

    assert_eq!(delegate_schema.outputs.len(), 1);
    assert_eq!(delegate_schema.outputs[0].name, "job_id");
}

#[test]
fn all_openworker_registered_controllers_match_schemas() {
    let controllers = all_openworker_registered_controllers();
    assert_eq!(controllers.len(), 6);

    let schemas = all_openworker_controller_schemas();
    for (controller, schema) in controllers.iter().zip(schemas.iter()) {
        assert_eq!(controller.schema.function, schema.function);
        assert_eq!(controller.schema.namespace, schema.namespace);
    }
}
