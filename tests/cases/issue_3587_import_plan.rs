//! 관계형: 문단 가져오기의 CLI/MCP 전달, 내용 보존 및 HWP 표 헤더 왕복 계약.
//! 주석 갱신: 2026-11-27 14:16:46 KST. 직전 파일 변경: 2026-09-13 07:44:09 UTC
//! (`9acd8d54fd04477ee819238ee63abd4eb7676380`).
//! #9225는 기존 표 샘플에 제보의 헤더 바이트를 추가한 합성 HWP를 다시 읽어 검증한다.
//! 렌더링 좌표나 시각 일치율을 검사하지 않는다. 생성된 HWPX는 한컴 기준 출력이 아니다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::{DocumentCore, ImportParagraphBlockRequest};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

fn bin() -> std::ffi::OsString {
    std::env::var_os("CARGO_BIN_EXE_rhwp").unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into())
}
fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rhwp-3587-import-{}-{}-{}",
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
struct Fixture {
    dir: TempDir,
    source: PathBuf,
    input: PathBuf,
    output: PathBuf,
    step: Value,
}
impl Fixture {
    fn new(ext: &str) -> Self {
        let dir = TempDir::new();
        let original = std::fs::read("samples/rnote/labnote-001.hwp").unwrap();
        let bytes = if ext == "hwp" {
            original
        } else {
            DocumentCore::from_bytes(&original)
                .unwrap()
                .export_hwpx_native()
                .unwrap()
        };
        let source = dir.path().join(format!("source.{ext}"));
        let input = dir.path().join(format!("input.{ext}"));
        let output = dir.path().join(format!("imported.{ext}"));
        std::fs::write(&source, &bytes).unwrap();
        std::fs::write(&input, &bytes).unwrap();
        let step = json!({"action":"import_paragraph_block","source":{"path":source,"sha256":hash(&bytes)},
            "request":{"sourceSection":0,"sourceStart":12,"sourceEnd":13,"targetSection":0,"insertBefore":13,"count":1}});
        Self {
            dir,
            source,
            input,
            output,
            step,
        }
    }
    fn plan(&self) -> Value {
        json!({"planVersion":"1.0","input":self.input,"output":self.output,"steps":[self.step]})
    }
}
fn run(plan: &Value) -> (i32, Value) {
    run_with_flags(plan, &[])
}
fn run_with_flags(plan: &Value, flags: &[&str]) -> (i32, Value) {
    let out = Command::new(bin())
        .args(["run", "--plan-json", &plan.to_string(), "--json"])
        .args(flags)
        .output()
        .unwrap();
    let value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{e}: {} / {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (out.status.code().unwrap(), value)
}

#[test]
fn import_preview_apply_and_fill_use_same_native_result_for_both_formats() {
    for ext in ["hwp", "hwpx"] {
        let f = Fixture::new(ext);
        let bytes = std::fs::read(&f.source).unwrap();
        let source = DocumentCore::from_bytes(&bytes).unwrap();
        let target = DocumentCore::from_bytes(&bytes).unwrap();
        let request: ImportParagraphBlockRequest =
            serde_json::from_value(f.step["request"].clone()).unwrap();
        let expected = serde_json::to_value(
            target
                .preview_paragraph_block_import_native(source.document(), &request)
                .unwrap(),
        )
        .unwrap();
        let mut plan = f.plan();
        plan["preconditions"] = json!({"inputSha256":hash(&bytes)});
        plan["dryRun"] = json!(true);
        let (code, dry) = run(&plan);
        assert_eq!(code, 0, "{dry}");
        assert!(!f.output.exists());
        assert_eq!(dry["preview"][0]["operationResult"]["result"], expected);
        assert_eq!(dry["preview"][0]["source"], f.step["source"]);
        assert_eq!(dry["untrustedContent"], true);
        plan["dryRun"] = json!(false);
        let (code, actual) = run(&plan);
        assert_eq!(code, 0, "{actual}");
        assert_eq!(actual["steps"][0]["operationResult"]["result"], expected);
        assert_eq!(actual["steps"][0]["source"], f.step["source"]);
        assert_eq!(actual["inputSha256"], hash(&bytes));
        assert_eq!(
            actual["outputSha256"],
            hash(&std::fs::read(&f.output).unwrap())
        );
        assert!(actual["changedPages"].is_null());
        let copied = &expected["copies"][0];
        let mapping = copied["mappings"].as_array().unwrap().iter().find(|m| {
            m["source"] == json!([{"kind":"paragraph","index":0},{"kind":"control","index":1},{"kind":"cell","index":5},{"kind":"paragraph","index":0}])
        }).expect("copied cell paragraph mapping");
        let mut relative = mapping["destination"].clone();
        let start = expected["inserted"]["start"].as_u64().unwrap();
        relative[0]["index"] = json!(relative[0]["index"].as_u64().unwrap() - start);
        let filled = f.dir.path().join(format!("filled.{ext}"));
        let next = json!({"planVersion":"1.0","input":f.output,"output":filled,
            "preconditions":{"inputSha256":actual["outputSha256"]},"steps":[{"action":"fill_template","request":{
                "scope":{"sectionIndex":0,"start":start,"end":expected["inserted"]["end"]},
                "bindings":[{"key":"body","target":{"kind":"textRange","path":relative,"start":0,"end":0}}],
                "record":{"body":"가져온 연구노트"}}}]});
        let (code, journal) = run(&next);
        assert_eq!(code, 0, "{journal}");
        let core = DocumentCore::from_bytes(&std::fs::read(&filled).unwrap()).unwrap();
        assert_eq!(
            core.document().sections[0].paragraphs.len(),
            target.document().sections[0].paragraphs.len() + 1
        );
        let rhwp::model::control::Control::Table(table) =
            &core.document().sections[0].paragraphs[start as usize].controls[1]
        else {
            panic!("table");
        };
        assert_eq!(table.cells[5].paragraphs[0].text, "가져온 연구노트");
        assert_eq!(std::fs::read(&f.source).unwrap(), bytes);
        assert_eq!(std::fs::read(&f.input).unwrap(), bytes);
    }
}

#[test]
fn import_failures_preserve_output_and_conditions_skip_source_io() {
    let f = Fixture::new("hwp");
    std::fs::write(&f.output, b"sentinel").unwrap();
    for (key, value, code) in [
        ("sha256", json!("0".repeat(64)), 3),
        ("sha256", json!("bad"), 2),
        ("path", json!(f.dir.path().join("missing.hwp")), 1),
        ("extra", json!(true), 2),
    ] {
        let mut plan = f.plan();
        plan["steps"][0]["source"][key] = value;
        for dry in [true, false] {
            plan["dryRun"] = json!(dry);
            let (got, result) = run(&plan);
            assert_eq!(got, code, "{result}");
            if code == 3 {
                assert_eq!(result["preconditionFailed"]["kind"], "sourceSha256");
            }
            assert_eq!(std::fs::read(&f.output).unwrap(), b"sentinel");
        }
    }
    let mut plan = f.plan();
    plan["steps"] = json!([f.step, f.step]);
    assert_eq!(run(&plan).0, 2);
    plan = f.plan();
    plan["preconditions"] = json!({"inputSha256":"0".repeat(64)});
    assert_eq!(run(&plan).1["preconditionFailed"]["kind"], "inputSha256");
    plan = f.plan();
    plan["steps"][0]["request"]["sourceEnd"] = json!(999999);
    assert_eq!(run(&plan).0, 2);
    plan["steps"][0]["source"]["path"] = json!("not-present.hwp");
    plan["steps"][0]["if"] = json!({"textFound":"nonexistent-3587-import-condition"});
    plan["dryRun"] = json!(true);
    let (code, result) = run(&plan);
    assert_eq!(code, 0, "{result}");
    assert_eq!(result["preview"][0]["skipped"], true);
    assert_eq!(std::fs::read(&f.output).unwrap(), b"sentinel");
}

#[test]
fn import_rejects_source_alias_and_supports_zero_count() {
    let f = Fixture::new("hwp");
    let before = std::fs::read(&f.source).unwrap();
    for key in ["input", "output"] {
        let mut plan = f.plan();
        plan[key] = json!(f.source);
        assert_eq!(run(&plan).0, 2);
    }
    let alias = f.dir.path().join("hardlink.hwp");
    std::fs::hard_link(&f.source, &alias).unwrap();
    let mut plan = f.plan();
    plan["output"] = json!(alias);
    #[cfg(unix)]
    assert_eq!(run(&plan).0, 2);
    plan = f.plan();
    plan["steps"][0]["request"]["count"] = json!(0);
    let (code, result) = run(&plan);
    assert_eq!(code, 0, "{result}");
    assert!(result["steps"][0]["operationResult"]["result"]["copies"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(std::fs::read(&f.source).unwrap(), before);
}

fn mcp(plan: &Value) -> Value {
    let mut child = Command::new(bin())
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hwp_run_plan","arguments":{"plan":plan}}})).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let reply: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(reply["result"]["isError"], false, "{reply}");
    let result: Value =
        serde_json::from_str(reply["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    result
}

#[test]
fn mcp_import_uses_existing_run_tool_and_matches_cli_preview() {
    for ext in ["hwp", "hwpx"] {
        let f = Fixture::new(ext);
        let mut plan = f.plan();
        plan["dryRun"] = json!(true);
        let (code, cli) = run(&plan);
        assert_eq!(code, 0, "{cli}");
        assert_eq!(mcp(&plan), cli);
        assert!(!f.output.exists());
        plan["dryRun"] = json!(false);
        let applied = mcp(&plan);
        assert_eq!(
            applied["steps"][0]["operationResult"],
            cli["preview"][0]["operationResult"]
        );
        DocumentCore::from_bytes(&std::fs::read(&f.output).unwrap()).unwrap();
    }
}

#[test]
fn import_rejects_oversized_and_malformed_files_without_replacing_output() {
    let f = Fixture::new("hwp");
    std::fs::write(&f.output, b"sentinel").unwrap();
    let huge = f.dir.path().join("oversized.hwp");
    std::fs::File::create(&huge)
        .unwrap()
        .set_len(64 * 1024 * 1024 + 1)
        .unwrap();
    let mut plan = f.plan();
    plan["steps"][0]["source"]["path"] = json!(huge);
    let (code, result) = run(&plan);
    assert_eq!(code, 2, "{result}");
    let malformed = f.dir.path().join("malformed.hwp");
    std::fs::write(&malformed, b"not a document").unwrap();
    plan["steps"][0]["source"] = json!({"path":malformed,"sha256":hash(b"not a document")});
    let (code, result) = run(&plan);
    assert_eq!(code, 1, "{result}");
    assert_eq!(std::fs::read(&f.output).unwrap(), b"sentinel");
}

const TABLE_SAMPLE: &str = "samples/hwp_table_test.hwp";
const TABLE_PARAGRAPH: usize = 3;

fn table_at(core: &DocumentCore, paragraph: usize) -> &rhwp::model::table::Table {
    let mut tables = core.document().sections[0].paragraphs[paragraph]
        .controls
        .iter()
        .filter_map(|control| match control {
            rhwp::model::control::Control::Table(table) => Some(table.as_ref()),
            _ => None,
        });
    let table = tables
        .next()
        .unwrap_or_else(|| panic!("expected a table in paragraph {paragraph}"));
    assert!(
        tables.next().is_none(),
        "multiple tables in paragraph {paragraph}"
    );
    table
}

fn table_source_with_common_tail(tail: &[u8]) -> Vec<u8> {
    let mut source = DocumentCore::from_bytes(&std::fs::read(TABLE_SAMPLE).unwrap()).unwrap();
    let table = table_at(&source, TABLE_PARAGRAPH);
    assert_eq!(table.raw_ctrl_data.len(), 42);
    assert!(table.common.description.is_empty());
    assert!(table.common.raw_extra.is_empty());
    let rhwp::model::control::Control::Table(table) =
        &mut source.document_mut().sections[0].paragraphs[TABLE_PARAGRAPH].controls[0]
    else {
        unreachable!();
    };
    table.common.raw_extra.extend_from_slice(tail);
    table.raw_ctrl_data.extend_from_slice(tail);
    let bytes = source.export_hwp_native().unwrap();
    let reopened = DocumentCore::from_bytes(&bytes).unwrap();
    let table = table_at(&reopened, TABLE_PARAGRAPH);
    assert_eq!(&table.raw_ctrl_data[40..42], &[0, 0]);
    assert_eq!(&table.raw_ctrl_data[42..], tail);
    assert_eq!(table.common.raw_extra, tail);
    bytes
}

fn table_import_request() -> ImportParagraphBlockRequest {
    ImportParagraphBlockRequest {
        source_section: 0,
        source_start: TABLE_PARAGRAPH,
        source_end: TABLE_PARAGRAPH + 1,
        target_section: 0,
        insert_before: 0,
        count: 1,
        limits: Default::default(),
    }
}

fn assert_imported_table(core: &DocumentCore, original: &rhwp::model::table::Table, tail: &[u8]) {
    let table = table_at(core, 0);
    assert_eq!(table.common.raw_extra, tail);
    assert_eq!(table.row_count, original.row_count);
    assert_eq!(table.col_count, original.col_count);
    assert_eq!(table.cells.len(), original.cells.len());
    for (cell, original_cell) in table.cells.iter().zip(&original.cells) {
        assert!(cell
            .paragraphs
            .iter()
            .map(|paragraph| &paragraph.text)
            .eq(original_cell
                .paragraphs
                .iter()
                .map(|paragraph| &paragraph.text)));
    }
}

#[test]
fn issue_9225_imports_table_and_preserves_zero_common_tail_in_hwp() {
    let source_bytes = table_source_with_common_tail(&[0, 0]);
    let source = DocumentCore::from_bytes(&source_bytes).unwrap();
    let original = table_at(&source, TABLE_PARAGRAPH);
    let request = table_import_request();
    let mut target = DocumentCore::new_empty();
    target.create_blank_document_native().unwrap();
    let preview = target
        .preview_paragraph_block_import_native(source.document(), &request)
        .expect("HWP table with a zero word after its empty description can be imported");
    let applied = target
        .import_paragraph_block_native(source.document(), &request)
        .unwrap();
    assert_eq!(preview.inserted, applied.inserted);
    assert_eq!(applied.inserted, 0..1);
    assert_imported_table(&target, original, &[0, 0]);

    let hwp = DocumentCore::from_bytes(&target.export_hwp_native().unwrap()).unwrap();
    assert_imported_table(&hwp, original, &[0, 0]);
    assert_eq!(&table_at(&hwp, 0).raw_ctrl_data[42..], &[0, 0]);
    let hwp = DocumentCore::from_bytes(&hwp.export_hwp_native().unwrap()).unwrap();
    assert_imported_table(&hwp, original, &[0, 0]);
    assert_eq!(&table_at(&hwp, 0).raw_ctrl_data[42..], &[0, 0]);

    let mut hwpx = DocumentCore::from_bytes(&target.export_hwpx_native().unwrap()).unwrap();
    assert_imported_table(&hwpx, original, &[]);
    hwpx = DocumentCore::from_bytes(&hwpx.export_hwpx_native().unwrap()).unwrap();
    assert_imported_table(&hwpx, original, &[]);
}

#[test]
fn issue_9225_cli_previews_and_imports_table_common_tail() {
    let source_bytes = table_source_with_common_tail(&[0, 0]);
    let source = DocumentCore::from_bytes(&source_bytes).unwrap();
    let original = table_at(&source, TABLE_PARAGRAPH);
    let mut target = DocumentCore::new_empty();
    target.create_blank_document_native().unwrap();
    let before = target.export_hwp_native().unwrap();
    let dir = TempDir::new();
    let source_path = dir.path().join("source.hwp");
    let input = dir.path().join("input.hwp");
    let output = dir.path().join("imported.hwp");
    std::fs::write(&source_path, &source_bytes).unwrap();
    std::fs::write(&input, &before).unwrap();
    let plan = json!({
        "planVersion": "1.0",
        "input": input,
        "output": output,
        "steps": [{
            "action": "import_paragraph_block",
            "source": { "path": source_path, "sha256": hash(&source_bytes) },
            "request": table_import_request()
        }]
    });
    let (dry_code, dry) = run_with_flags(&plan, &["--dry-run"]);
    assert!(!output.exists());

    let (apply_code, actual) = run(&plan);
    assert_eq!([dry_code, apply_code], [0, 0], "{dry} / {actual}");
    assert_eq!(
        dry["preview"][0]["operationResult"]["result"],
        actual["steps"][0]["operationResult"]["result"]
    );
    let reopened = DocumentCore::from_bytes(&std::fs::read(output).unwrap()).unwrap();
    assert_imported_table(&reopened, original, &[0, 0]);
    assert_eq!(std::fs::read(source_path).unwrap(), source_bytes);
    assert_eq!(std::fs::read(input).unwrap(), before);
}

#[test]
fn issue_9225_only_accepts_the_observed_table_common_tail() {
    let mut target = DocumentCore::new_empty();
    target.create_blank_document_native().unwrap();
    let request = table_import_request();
    let normal = DocumentCore::from_bytes(&std::fs::read(TABLE_SAMPLE).unwrap()).unwrap();
    target
        .preview_paragraph_block_import_native(normal.document(), &request)
        .unwrap();
    let before = format!("{:?}", target.document());

    for tail in [&[0, 1][..], &[0][..], &[0, 0, 0, 0][..]] {
        let bytes = table_source_with_common_tail(tail);
        let source = DocumentCore::from_bytes(&bytes).unwrap();
        for error in [
            target
                .preview_paragraph_block_import_native(source.document(), &request)
                .unwrap_err(),
            target
                .import_paragraph_block_native(source.document(), &request)
                .unwrap_err(),
        ] {
            let message = error.to_string();
            assert!(message.contains("uninterpreted common tail"), "{message}");
            assert!(message.contains("[Paragraph(0), Control(0)]"), "{message}");
        }
        assert_eq!(format!("{:?}", target.document()), before);
    }
}
