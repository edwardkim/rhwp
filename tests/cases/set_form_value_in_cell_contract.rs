//! `edit set-form-value-in-cell` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

static SEQ: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwpx/form-002.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "rhwp-formcell-{tag}-{}-{}-{}.hwpx",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

/// 문서 트리에서 표 셀 안 첫 Form 좌표를 고른다. (0,0,0,0,0,0) 을 가정하지 않는다.
fn first_cell_form(path: &str) -> (usize, usize, usize, usize, usize, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for (si, sec) in doc.document().sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            for (tci, c) in para.controls.iter().enumerate() {
                if let Control::Table(t) = c {
                    for (cell_idx, cell) in t.cells.iter().enumerate() {
                        for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                            for (fci, fc) in cp.controls.iter().enumerate() {
                                if matches!(fc, Control::Form(_)) {
                                    return (si, pi, tci, cell_idx, cpi, fci);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    panic!("셀 안 양식 컨트롤이 없다");
}

fn cell_form_value(
    path: &Path,
    sec: usize,
    table_para: usize,
    table_ci: usize,
    cell: usize,
    cell_para: usize,
    form_ci: usize,
) -> i64 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let Control::Table(t) = &doc.document().sections[sec].paragraphs[table_para].controls[table_ci]
    else {
        panic!("표가 아니다");
    };
    let Control::Form(f) = &t.cells[cell].paragraphs[cell_para].controls[form_ci] else {
        panic!("양식이 아니다");
    };
    f.value as i64
}

#[test]
fn set_form_value_in_cell_writes() {
    let src = sample();
    let (sec, tpara, tci, cell, cpara, fci) = first_cell_form(&src);
    let before = cell_form_value(Path::new(&src), sec, tpara, tci, cell, cpara, fci);
    let next = if before == 0 { 1 } else { 0 };
    let out = temp("out");
    let payload = format!(r#"{{"value":{next}}}"#);
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-form-value-in-cell",
            src.as_str(),
            "--section",
            &sec.to_string(),
            "--table-para",
            &tpara.to_string(),
            "--table-ci",
            &tci.to_string(),
            "--cell",
            &cell.to_string(),
            "--cell-para",
            &cpara.to_string(),
            "--ctrl",
            &fci.to_string(),
            "--value",
            &payload,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(out.exists());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["section"], sec);
    assert_eq!(v["tablePara"], tpara);
    assert_eq!(v["tableCi"], tci);
    assert_eq!(v["cell"], cell);
    assert_eq!(v["cellPara"], cpara);
    assert_eq!(v["ctrl"], fci);
    assert_eq!(
        cell_form_value(&out, sec, tpara, tci, cell, cpara, fci),
        next
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let (sec, tpara, tci, cell, cpara, fci) = first_cell_form(&src);
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-form-value-in-cell",
            src.as_str(),
            "--section",
            &sec.to_string(),
            "--table-para",
            &tpara.to_string(),
            "--table-ci",
            &tci.to_string(),
            "--cell",
            &cell.to_string(),
            "--cell-para",
            &cpara.to_string(),
            "--ctrl",
            &fci.to_string(),
            "--value",
            r#"{"value":1}"#,
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
            "set-form-value-in-cell",
            src.as_str(),
            "--section",
            "0",
            "--table-para",
            "0",
            "--table-ci",
            "0",
            "--cell",
            "0",
            "--cell-para",
            "0",
            "--ctrl",
            "0",
            "--value",
            r#"{"value":1}"#,
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
}
