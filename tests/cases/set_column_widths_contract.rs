//! `edit set-column-widths` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::document_core::queries::table_extract::extract_tables;
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn first_top_table(path: &str) -> (usize, u16) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    let g = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.container_path.is_empty())
        .expect("본문 최상위 표");
    (g.index, g.cols)
}

fn column_widths(path: &Path, index: usize) -> Vec<u32> {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let g = extract_tables(doc.document())
        .into_iter()
        .find(|g| g.index == index && g.container_path.is_empty())
        .expect("표");
    let Control::Table(t) =
        &doc.document().sections[g.section].paragraphs[g.paragraph].controls[g.control]
    else {
        panic!("표 컨트롤");
    };
    t.get_column_widths()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-colw-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn set_column_widths_applies() {
    let src = sample();
    let (idx, cols) = first_top_table(&src);
    assert!(cols >= 1);
    let want: Vec<u32> = (0..cols).map(|i| 1500 + u32::from(i) * 50).collect();
    let widths_s = want
        .iter()
        .map(|w| w.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let out = temp("out");
    let idx_s = idx.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-widths",
            src.as_str(),
            "--table",
            &idx_s,
            "--widths",
            &widths_s,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(column_widths(&out, idx), want);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (idx, cols) = first_top_table(&src);
    let widths_s = vec!["1500"; cols as usize].join(",");
    let out = temp("dry");
    let idx_s = idx.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-widths",
            src.as_str(),
            "--table",
            &idx_s,
            "--widths",
            &widths_s,
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-column-widths",
            src.as_str(),
            "--table",
            "0",
            "--widths",
            "1500",
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}

#[test]
fn mcp_declared() {
    let output = Command::new(rhwp_bin())
        .args(["capabilities", "--mcp"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(v["tools"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t["name"] == "hwp_set_column_widths"));
}

// 독립 Printで全1쪽の원문・동일 편집을확인한 병합 열 폭 API 경계.
fn merged_resize_fixture() -> HwpDocument {
    HwpDocument::from_bytes(
        &std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("samples/issue6190/center_align_first_line_indent.hwp"),
        )
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn fully_merged_columns_keep_the_requested_total_in_model_reply_and_save() {
    let mut d = merged_resize_fixture();
    let widths = vec![5000u32, 7000];
    let total: u32 = widths.iter().sum();
    let reply = d.set_table_column_widths_native(0, 7, 0, widths).unwrap();
    let reply: serde_json::Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(reply["tableWidth"], total);
    d.insert_text_native(0, 7, 0, "가").unwrap();
    for reopen in [false, true] {
        if reopen {
            let bytes = d.export_hwp_native().unwrap();
            d = HwpDocument::from_bytes(&bytes).unwrap();
        }
        let Control::Table(t) = &d.document().sections[0].paragraphs[7].controls[0] else {
            panic!()
        };
        assert!(t.cells.iter().all(|c| c.col_span == t.col_count));
        assert_eq!(t.common.width, total);
        assert!(t.cells.iter().all(|c| c.width == total));
        assert_eq!(
            u32::from_le_bytes(t.raw_ctrl_data[12..16].try_into().unwrap()),
            total
        );
    }
}
#[test]
fn overflowing_requested_total_is_rejected_atomically() {
    let mut d = merged_resize_fixture();
    let before = d.export_hwp_native().unwrap();
    assert!(d
        .set_table_column_widths_native(0, 7, 0, vec![u32::MAX, u32::MAX])
        .is_err());
    assert_eq!(d.export_hwp_native().unwrap(), before);
}
