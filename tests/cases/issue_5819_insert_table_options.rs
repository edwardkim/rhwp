//! #5819: CLI/MCP 표 생성 옵션의 실제 실행과 저장·재파싱 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rhwp::model::{control::Control, style::Alignment, table::TablePageBreak};
use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};

const HWP: &str = "samples/field-01.hwp";
const HWPX: &str = "samples/issue5162_field_wraps_table.hwpx";
const HWPX_ANCHOR: &str =
    "./ApplicationArea/cr:ApplicationPoliceArea/cr:Writer/cr:Organization/cr:Organization.Name";

fn source(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

struct OutputFile(PathBuf);
impl OutputFile {
    fn new(extension: &str) -> Self {
        Self(std::env::temp_dir().join(
            format!("rhwp-5819-{}-{}.{extension}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
        ))
    }
    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}
impl Drop for OutputFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn run(input: &str, args: &[&str], output: &OutputFile) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["edit", "insert-table"])
        .arg(source(input))
        .args(["--rows", "2", "--cols", "3"])
        .args(args)
        .args(["-o", output.path(), "--json"])
        .output()
        .unwrap()
}

fn load(path: &Path) -> HwpDocument {
    HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn fields(doc: &HwpDocument) -> Vec<(u32, Option<String>, String, String)> {
    doc.collect_all_fields()
        .into_iter()
        .map(|f| {
            (
                f.field.field_id,
                f.field.field_name().map(str::to_owned),
                f.field.command,
                f.value,
            )
        })
        .collect()
}

fn assert_saved(doc: &HwpDocument, result: &Value, expected_widths: &[u32], repeat: bool) {
    let section = result["section"].as_u64().unwrap() as usize;
    let para = result["tableParagraph"].as_u64().unwrap() as usize;
    let control = result["control"].as_u64().unwrap() as usize;
    let Control::Table(table) =
        &doc.document().sections[section].paragraphs[para].controls[control]
    else {
        panic!("table")
    };
    assert_eq!((table.row_count, table.col_count), (2, 3));
    assert_eq!(table.get_column_widths(), expected_widths);
    assert_eq!(table.common.width, expected_widths.iter().sum::<u32>());
    assert_eq!(table.repeat_header, repeat);
    assert_eq!(table.page_break, TablePageBreak::CellBreak);
    assert_eq!(
        table.leading_header_rows(),
        if repeat { vec![0] } else { vec![] }
    );
    for cell in &table.cells {
        let shape = &doc.document().doc_info.para_shapes[cell.paragraphs[0].para_shape_id as usize];
        assert_eq!(
            shape.alignment,
            [Alignment::Left, Alignment::Center, Alignment::Right][cell.col as usize]
        );
        assert_eq!(cell.is_header, cell.row == 0 && repeat);
    }
}

#[test]
fn absolute_widths_alignment_header_and_anchor_survive_both_formats() {
    for (input, anchor, extension) in [(HWP, "회사명", "hwp"), (HWPX, HWPX_ANCHOR, "hwpx")] {
        let original = load(&source(input));
        let original_fields = fields(&original);
        let field = original
            .collect_all_fields()
            .into_iter()
            .find(|f| f.field.field_name() == Some(anchor))
            .unwrap();
        let anchor_para = original.document().sections[field.location.section_index].paragraphs
            [field.location.para_index]
            .clone();
        let out = OutputFile::new(extension);
        let output = run(
            input,
            &[
                "--widths",
                "3000,6000,9000",
                "--alignments",
                "left,center,right",
                "--at-field",
                anchor,
                "--verify",
            ],
            &out,
        );
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["verify"]["diffCount"], 0);
        assert_eq!(result["tableParagraph"], field.location.para_index + 1);
        let saved = load(&out.0);
        assert_saved(&saved, &result, &[3000, 6000, 9000], true);
        assert_eq!(
            fields(&saved),
            original_fields,
            "all field IDs, names, commands and values must survive"
        );
        assert_eq!(
            saved.document().sections[field.location.section_index].paragraphs
                [field.location.para_index]
                .text,
            anchor_para.text
        );
    }
}

#[test]
fn percent_widths_keep_total_and_explicit_false_disables_headers() {
    let original = load(&source(HWP));
    let pd = &original.document().sections[0].section_def.page_def;
    let width = pd.width - pd.margin_left - pd.margin_right - 566;
    let expected = [width / 4, width / 2 - width / 4, width - width / 2];
    let out = OutputFile::new("hwp");
    let output = run(
        HWP,
        &[
            "--widths",
            "25%,25%,50%",
            "--alignments",
            "left,center,right",
            "--repeat-header",
            "false",
            "--at-field",
            "회사명",
        ],
        &out,
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    // Exact quarter boundaries in this fixture; assertions inspect serialized cells, not the envelope alone.
    assert_eq!(width % 4, 0);
    assert_saved(&load(&out.0), &result, &expected, false);
}

#[test]
fn invalid_options_and_dry_run_never_write() {
    let cases: &[&[&str]] = &[
        &["--widths", "1000,2000"],
        &["--widths", "0,2000,3000"],
        &["--widths", "4294967295,4294967295,1"],
        &["--widths", "20%,30%,40%"],
        &["--widths", "NaN%,50%,50%"],
        &["--widths", "0.000000001%,49.999999999%,50%"],
        &["--widths", "20%,3000,50%"],
        &["--widths", "20%%,30%,50%"],
        &["--alignments", "left,right"],
        &["--alignments", "left,invalid,right"],
        &["--at-field", "missing"],
        &["--at-field", "목차1"],
        &["--at-field", "회사명", "--para", "0"],
        &["--para", "99999"],
        &["--offset", "99999"],
        &["--repeat-header", "invalid"],
    ];
    let before = std::fs::read(source(HWP)).unwrap();
    for args in cases {
        for dry in [false, true] {
            let out = OutputFile::new("hwp");
            // Failed requests also preserve an existing destination.
            std::fs::write(&out.0, b"keep existing destination").unwrap();
            let mut args = args.to_vec();
            if dry {
                args.push("--dry-run");
            }
            let output = run(HWP, &args, &out);
            assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
            assert!(output.stdout.is_empty());
            assert_eq!(std::fs::read(&out.0).unwrap(), b"keep existing destination");
        }
    }
    let out = OutputFile::new("hwp");
    let output = run(
        HWP,
        &[
            "--widths",
            "3000,6000,9000",
            "--at-field",
            "회사명",
            "--dry-run",
        ],
        &out,
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert!(!out.0.exists());
    assert_eq!(std::fs::read(source(HWP)).unwrap(), before);
}

#[test]
fn mcp_call_passes_options_and_false_values_to_the_cli() {
    let out = OutputFile::new("hwp");
    let dry = OutputFile::new("hwp");
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("mcp-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let init = json!({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{
        "protocolVersion":"2025-06-18", "capabilities":{}, "clientInfo":{"name":"issue5819", "version":"1"}}});
    writeln!(stdin, "{init}").unwrap();
    for (id, output, dry_run) in [(2, &out, false), (3, &dry, true)] {
        let message = json!({"jsonrpc":"2.0", "id":id, "method":"tools/call", "params":{
            "name":"hwp_insert_table", "arguments":{
                "path":source(HWP), "rows":2, "cols":3, "atField":"회사명",
                "widths":"3000,6000,9000", "alignments":"left,center,right",
                "repeatHeader":false, "dryRun":dry_run, "output":output.0}}});
        writeln!(stdin, "{message}").unwrap();
    }
    drop(stdin);
    let response = child.wait_with_output().unwrap();
    assert!(response.status.success(), "{response:?}");
    let messages: Vec<Value> = String::from_utf8(response.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let call = messages.iter().find(|m| m["id"] == 2).unwrap();
    assert_eq!(call["result"]["isError"], false, "{call}");
    let result: Value =
        serde_json::from_str(call["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(result["dryRun"], false);
    assert_eq!(result["repeatHeader"], false);
    assert_saved(&load(&out.0), &result, &[3000, 6000, 9000], false);
    let dry_call = messages.iter().find(|m| m["id"] == 3).unwrap();
    assert_eq!(dry_call["result"]["isError"], false, "{dry_call}");
    assert!(!dry.0.exists());
}
