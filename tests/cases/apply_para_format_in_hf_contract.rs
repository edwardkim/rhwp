//! [#5109] `edit apply-para-format-in-hf` 계약.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::control::Control;
use rhwp::model::header_footer::HeaderFooterApply;
use rhwp::model::style::Alignment;
use rhwp::wasm_api::HwpDocument;

static TEMP_SERIAL: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/field-01.hwp")
        .to_string_lossy()
        .into_owned()
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-hfpfmt-{tag}-{}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        TEMP_SERIAL.fetch_add(1, Ordering::Relaxed),
    ))
}

fn fixture_with_odd_footer() -> PathBuf {
    let bytes = std::fs::read(sample()).unwrap();
    let mut doc = HwpDocument::from_bytes(&bytes).unwrap();
    let raw = doc
        .create_header_footer_native(0, false, 2)
        .expect("꼬리말 생성");
    assert!(raw.contains(r#""ok":true"#), "{raw}");
    let out = temp("fx");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

fn footer_alignment(path: &Path) -> Alignment {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for para in &doc.document().sections[0].paragraphs {
        for ctrl in &para.controls {
            if let Control::Footer(f) = ctrl {
                if f.apply_to == HeaderFooterApply::Odd {
                    let psid = f.paragraphs[0].para_shape_id;
                    return doc.document().doc_info.para_shapes[psid as usize].alignment;
                }
            }
        }
    }
    panic!("홀수 꼬리말이 없다");
}

#[test]
fn apply_center_is_visible() {
    let src = fixture_with_odd_footer();
    let current = footer_alignment(&src);
    let (target, props) = if current != Alignment::Center {
        (Alignment::Center, r#"{"alignment":"center"}"#)
    } else {
        (Alignment::Right, r#"{"alignment":"right"}"#)
    };
    let out = temp("out");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-hf",
            src.to_str().unwrap(),
            "--footer",
            "--apply-to",
            "2",
            "--props",
            props,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["isHeader"], false);
    assert_eq!(v["applyTo"], 2);
    assert_eq!(
        footer_alignment(&out),
        target,
        "꼬리말 정렬이 저장본에 없다"
    );
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = fixture_with_odd_footer();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-hf",
            src.to_str().unwrap(),
            "--footer",
            "--apply-to",
            "2",
            "--props",
            r#"{"alignment":"center"}"#,
            "-o",
            out.to_str().unwrap(),
            "--dry-run",
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["dryRun"], true);
    assert_eq!(v["isHeader"], false);
    let _ = std::fs::remove_file(&src);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = fixture_with_odd_footer();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-para-format-in-hf",
            src.to_str().unwrap(),
            "--footer",
            "--props",
            r#"{"alignment":"center"}"#,
            "--nope",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let _ = std::fs::remove_file(&src);
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
        .any(|t| t["name"] == "hwp_apply_para_format_in_hf"));
}
