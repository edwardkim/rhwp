//! Existing run/MCP entry point reuses the native atomic template engine.
use rhwp::document_core::TemplateOperation;
use serde_json::{json, Value};

pub(super) fn is_action(step: &Value) -> bool {
    super::import::is_action(step)
        || TemplateOperation::is_action(step["action"].as_str().unwrap_or(""))
}

pub(super) fn parse(step: &Value) -> Result<TemplateOperation, String> {
    // Outer steps remain extensible and support `if`. The request itself is strict.
    TemplateOperation::from_parts(step["action"].as_str().unwrap_or(""), &step["request"])
        .map_err(|e| e.to_string())
}

pub(super) fn preview(
    doc: &rhwp::wasm_api::HwpDocument,
    step: &Value,
    index: usize,
) -> Result<Value, String> {
    let op = parse(step)?;
    let workload = op.workload().map_err(|e| e.to_string())?;
    let result = doc
        .preview_template_operation_native(&op)
        .map_err(|e| e.to_string())?;
    Ok(json!({"step":index,"action":step["action"],"operationResult":result,"workload":workload}))
}

pub(super) fn execute(
    doc: &mut rhwp::wasm_api::HwpDocument,
    step: &Value,
    index: usize,
) -> Result<Value, String> {
    let op = parse(step)?;
    let workload = op.workload().map_err(|e| e.to_string())?;
    let result = doc
        .execute_template_operation_native(&op)
        .map_err(|e| e.to_string())?;
    Ok(json!({"step":index,"action":step["action"],"operationResult":result,"workload":workload}))
}
