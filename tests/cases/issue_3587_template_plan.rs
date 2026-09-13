//! Real CLI/MCP calls, saved bytes and unchanged dry-run inputs. No Gym dependency.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    document_core::{DocumentCore, TemplateOperation},
    model::control::Control,
};
use serde_json::{json, Value};
use std::{
    io::Write,
    process::{Command, Stdio},
};

const SAMPLE: &str = "samples/rnote/labnote-001.hwp";

struct TempDir(std::path::PathBuf);
impl TempDir {
    fn new() -> Self {
        static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rhwp-3587-plan-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn rows() -> Value {
    json!({"action":"repeat_and_fill_table_rows","request":{
        "sectionIndex":0,"paragraphIndex":12,"controlIndex":1,"startRow":5,"endRow":6,"insertBefore":6,
        "bindings":[{"key":"value","target":{"kind":"textRange","path":[{"kind":"paragraph","index":0},{"kind":"control","index":0},{"kind":"cell","index":0},{"kind":"paragraph","index":0}],"start":0,"end":0}}],
        "records":[{"value":"실험 1"},{"value":"실험 2"}]
    }})
}
fn run(plan: &Value) -> (i32, Value) {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["run", "--plan-json", &plan.to_string(), "--json"])
        .output()
        .unwrap();
    let v = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{e}: {} / {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (out.status.code().unwrap(), v)
}
fn input_for_format(dir: &TempDir, ext: &str) -> std::path::PathBuf {
    if ext == "hwp" {
        return SAMPLE.into();
    }
    // Derived format probe, not an independent Hancom oracle. `run` preserves input format.
    let path = dir.path().join("input.hwpx");
    let core = DocumentCore::from_bytes(&std::fs::read(SAMPLE).unwrap()).unwrap();
    std::fs::write(&path, core.export_hwpx_native().unwrap()).unwrap();
    path
}
fn assert_format(path: &std::path::Path, ext: &str) {
    let bytes = std::fs::read(path).unwrap();
    let expected = if ext == "hwpx" {
        rhwp::parser::FileFormat::Hwpx
    } else {
        rhwp::parser::FileFormat::Hwp
    };
    assert_eq!(rhwp::parser::detect_format(&bytes), expected);
}
fn assert_table(path: &std::path::Path) {
    let c = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let Control::Table(t) = &c.document().sections[0].paragraphs[12].controls[1] else {
        panic!("table")
    };
    assert_eq!(t.row_count, 36);
    assert_eq!(
        t.cells.iter().find(|c| c.row == 6).unwrap().paragraphs[0].text,
        "실험 1"
    );
    assert_eq!(
        t.cells.iter().find(|c| c.row == 7).unwrap().paragraphs[0].text,
        "실험 2"
    );
}
#[test]
fn cli_dry_run_and_saved_hwp_hwpx_match_native_row_result() {
    let dir = TempDir::new();
    let original = std::fs::read(SAMPLE).expect("required real sample");
    for ext in ["hwp", "hwpx"] {
        let input = input_for_format(&dir, ext);
        let c = DocumentCore::from_bytes(&std::fs::read(&input).unwrap()).unwrap();
        let expected = serde_json::to_value(
            c.preview_template_operation_native(
                &TemplateOperation::from_json(&rows().to_string()).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let path = dir.path().join(format!("rows.{ext}"));
        let mut plan =
            json!({"planVersion":"1.0","input":input,"output":path,"steps":[rows()],"dryRun":true});
        let (code, preview) = run(&plan);
        assert_eq!(code, 0, "{preview}");
        assert!(!path.exists());
        assert_eq!(preview["preview"][0]["operationResult"], expected);
        assert_eq!(preview["untrustedContent"], true);
        plan["dryRun"] = json!(false);
        let (code, actual) = run(&plan);
        assert_eq!(code, 0, "{actual}");
        assert_eq!(actual["steps"][0]["operationResult"], expected);
        assert_eq!(actual["changedPages"], Value::Null);
        assert_format(&path, ext);
        assert_table(&path);
    }
    assert_eq!(std::fs::read(SAMPLE).unwrap(), original);
}

#[test]
fn mixed_steps_and_invalid_request_never_overwrite_output() {
    let dir = TempDir::new();
    let path = dir.path().join("preserve.hwp");
    std::fs::write(&path, b"sentinel").unwrap();
    let old = json!({"action":"replace_text","find":"x","replace":"y"});
    for steps in [json!([rows(), old]), json!([rows(), rows()])] {
        let plan = json!({"planVersion":"1.0","input":SAMPLE,"output":path,"steps":steps});
        let (code, v) = run(&plan);
        assert_eq!(code, 2, "{v}");
        assert!(v["invalid"][0]["reason"]
            .as_str()
            .unwrap()
            .contains("단독 step"));
        assert_eq!(std::fs::read(&path).unwrap(), b"sentinel");
    }
    let mut bad = rows();
    bad["request"]["records"][1] = json!({"wrong":"x"});
    for dry in [true, false] {
        let (code, v) = run(
            &json!({"planVersion":"1.0","input":SAMPLE,"output":path,"steps":[bad],"dryRun":dry}),
        );
        assert_eq!(code, 2, "{v}");
        assert!(v["invalid"].as_array().is_some_and(|a| !a.is_empty()));
        assert_eq!(std::fs::read(&path).unwrap(), b"sentinel");
    }
}

#[test]
fn mcp_existing_run_tool_executes_new_template_action() {
    let dir = TempDir::new();
    let path = dir.path().join("mcp.hwpx");
    let input = input_for_format(&dir, "hwpx");
    let plan = json!({"planVersion":"1.0","input":input,"output":path,"steps":[rows()]});
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let rpc = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hwp_run_plan","arguments":{"plan":plan}}});
    writeln!(child.stdin.take().unwrap(), "{rpc}").unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let reply: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(reply["result"]["isError"], false, "{reply}");
    let journal: Value =
        serde_json::from_str(reply["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(
        journal["steps"][0]["operationResult"]["result"]["rowCount"],
        36
    );
    assert_table(&path);
    assert_format(&path, "hwpx");
}

#[test]
fn fixed_and_block_actions_match_native_and_wasm_wrapper_then_reopen() {
    let dir = TempDir::new();
    let binding = json!({"key":"value","target":{"kind":"textRange","path":[{"kind":"paragraph","index":0},{"kind":"control","index":1},{"kind":"cell","index":5},{"kind":"paragraph","index":0}],"start":0,"end":0}});
    let form = json!({"action":"fill_template","request":{"scope":{"sectionIndex":0,"start":12,"end":13},"bindings":[binding],"record":{"value":"기록"}}});
    let block = json!({"action":"repeat_and_fill_paragraph_block","request":{"block":{"sectionIndex":0,"sourceStart":12,"sourceEnd":13,"insertBefore":13,"count":2},"bindings":[binding],"records":[{"value":"첫째"},{"value":"둘째"}]}});
    for ext in ["hwp", "hwpx"] {
        let input = input_for_format(&dir, ext);
        let bytes = std::fs::read(&input).unwrap();
        for (i, op) in [form.clone(), block.clone()].into_iter().enumerate() {
            let mut native = DocumentCore::from_bytes(&bytes).unwrap();
            let result = native
                .execute_template_operation_native(
                    &TemplateOperation::from_json(&op.to_string()).unwrap(),
                )
                .unwrap();
            let expected = serde_json::to_value(result).unwrap();
            let mut wasm = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap();
            // Real wrapper on native: success avoids platform-specific JsValue error construction.
            let preview: Value = serde_json::from_str(
                &wasm
                    .apply_template_operation(&json!({"operation":op,"dryRun":true}).to_string())
                    .unwrap(),
            )
            .unwrap();
            let actual: Value = serde_json::from_str(
                &wasm
                    .apply_template_operation(&json!({"operation":op}).to_string())
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(preview["operationResult"], expected);
            assert_eq!(actual["operationResult"], expected);
            assert_eq!(
                format!("{:?}", native.document()),
                format!("{:?}", wasm.document())
            );
            let path = dir.path().join(format!("{i}.{ext}"));
            let plan = json!({"planVersion":"1.0","input":input,"output":path,"steps":[op]});
            let (code, journal) = run(&plan);
            assert_eq!(code, 0, "{journal}");
            assert_eq!(journal["steps"][0]["operationResult"], expected);
            assert_format(&path, ext);
            let reopened = DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
            let native_bytes = if ext == "hwp" {
                native.export_hwp_native()
            } else {
                native.export_hwpx_native()
            }
            .unwrap();
            let native_reopen = DocumentCore::from_bytes(&native_bytes).unwrap();
            assert_eq!(
                format!("{:?}", reopened.document()),
                format!("{:?}", native_reopen.document())
            );
        }
    }
}

#[test]
fn template_condition_skip_and_input_hash_gate_remain_intact() {
    let dir = TempDir::new();
    let path = dir.path().join("out.hwp");
    let mut step = rows();
    step["if"] = json!({"textFound":"missing-text-3587-xyz"});
    step["request"]["paragraphIndex"] = json!(999999);
    let mut plan =
        json!({"planVersion":"1.0","input":SAMPLE,"output":path,"steps":[step],"dryRun":true});
    let (code, journal) = run(&plan);
    assert_eq!(code, 0, "{journal}");
    assert_eq!(journal["preview"][0]["skipped"], true);
    assert!(!path.exists());
    plan["steps"] = json!([rows()]);
    plan["preconditions"] = json!({"inputSha256":"0".repeat(64)});
    let (code, journal) = run(&plan);
    assert_eq!(code, 3, "{journal}");
    assert_eq!(journal["preconditionFailed"]["kind"], "inputSha256");
    assert!(!path.exists());
}
