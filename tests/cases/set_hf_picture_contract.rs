//! 관계형: `edit set-hf-picture`의 요청 값과 재열기 후 속성·오류 시 파일 미생성 계약.
//! 주석 갱신: 2026-11-13T23:25:55+09:00
//! 실물 px 고정 검사가 아니므로 PDF 일치율을 작성 근거로 사용하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

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
        "rhwp-hfpic-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn header_picture_addr(path: &Path) -> (usize, usize, usize, usize, usize) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    for (si, section) in doc.document().sections.iter().enumerate() {
        for (pi, paragraph) in section.paragraphs.iter().enumerate() {
            for (ci, control) in paragraph.controls.iter().enumerate() {
                let Control::Header(header) = control else {
                    continue;
                };
                for (hi, inner_para) in header.paragraphs.iter().enumerate() {
                    for (hci, inner_control) in inner_para.controls.iter().enumerate() {
                        if matches!(inner_control, Control::Picture(_)) {
                            return (si, pi, ci, hi, hci);
                        }
                    }
                }
            }
        }
    }
    panic!("머리말 그림이 없다");
}

fn picture_width(path: &Path, addr: (usize, usize, usize, usize, usize)) -> u64 {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let raw = doc
        .get_header_footer_picture_properties_native(addr.0, addr.1, addr.2, addr.3, addr.4)
        .expect("머리말 그림 속성");
    let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
    value["width"].as_u64().expect("width")
}

fn fixture_with_header_picture() -> PathBuf {
    let mut doc = HwpDocument::create_empty();
    doc.create_header_footer_native(0, true, 0)
        .expect("머리말 생성");
    let png = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/logo/logo-16.png"))
        .expect("tiny png");
    doc.insert_picture_native(
        0,
        0,
        0,
        &[],
        &png,
        1200,
        1200,
        16,
        16,
        "png",
        "header logo",
        None,
        None,
    )
    .expect("그림 삽입");

    let body = &mut doc.document_mut().sections[0].paragraphs[0];
    let picture_idx = body
        .controls
        .iter()
        .position(|control| matches!(control, Control::Picture(_)))
        .expect("본문 그림");
    let picture = body.controls.remove(picture_idx);
    body.align_ctrl_data_records();
    let header = body
        .controls
        .iter_mut()
        .find_map(|control| match control {
            Control::Header(header) => Some(header),
            _ => None,
        })
        .expect("머리말");
    header.paragraphs[0].controls.push(picture);
    header.paragraphs[0].align_ctrl_data_records();

    let out = temp("fixture");
    std::fs::write(&out, doc.export_hwp().expect("export")).unwrap();
    out
}

#[test]
fn hf_picture_uses_decoded_json_and_rejects_duplicate_members() {
    let source = fixture_with_header_picture();
    let addr = header_picture_addr(&source);
    let values = [addr.0, addr.1, addr.2, addr.3, addr.4].map(|v| v.to_string());
    let target = picture_width(&source, addr) + 333;
    let props = format!(r#"{{"\u0077idth":{target}}}"#);
    let changed = temp("decoded");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-hf-picture",
            source.to_str().unwrap(),
            "--section",
            &values[0],
            "--para",
            &values[1],
            "--ctrl",
            &values[2],
            "--inner-para",
            &values[3],
            "--inner-ctrl",
            &values[4],
            "--props",
            &props,
            "-o",
            changed.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        picture_width(&changed, header_picture_addr(&changed)),
        target
    );
    for dry in [false, true] {
        let rejected = temp("duplicate");
        let mut args = vec![
            "edit",
            "set-hf-picture",
            source.to_str().unwrap(),
            "--section",
            &values[0],
            "--para",
            &values[1],
            "--ctrl",
            &values[2],
            "--inner-para",
            &values[3],
            "--inner-ctrl",
            &values[4],
            "--props",
            r#"{"width":3000,"width":4000}"#,
            "-o",
            rejected.to_str().unwrap(),
        ];
        if dry {
            args.push("--dry-run");
        }
        let output = Command::new(rhwp_bin()).args(args).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("중복"));
        assert!(!rejected.exists());
    }
    let _ = std::fs::remove_file(source);
    let _ = std::fs::remove_file(changed);
}

#[test]
fn saved_picture_width_is_visible() {
    let src = fixture_with_header_picture();
    let addr = header_picture_addr(&src);
    let target = picture_width(&src, addr) + 333;
    let out = temp("out");
    let props = format!(r#"{{"width":{target}}}"#);
    let values = [
        addr.0.to_string(),
        addr.1.to_string(),
        addr.2.to_string(),
        addr.3.to_string(),
        addr.4.to_string(),
    ];
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-hf-picture",
            src.to_str().unwrap(),
            "--section",
            &values[0],
            "--para",
            &values[1],
            "--ctrl",
            &values[2],
            "--inner-para",
            &values[3],
            "--inner-ctrl",
            &values[4],
            "--props",
            &props,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["section"], addr.0);
    assert_eq!(value["paragraph"], addr.1);
    assert_eq!(value["ctrl"], addr.2);
    assert_eq!(value["innerPara"], addr.3);
    assert_eq!(value["innerCtrl"], addr.4);
    assert_eq!(picture_width(&out, addr), target);
    let _ = std::fs::remove_file(&src);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_no_file() {
    let src = sample();
    let out = temp("dry");
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-hf-picture",
            src.as_str(),
            "--section",
            "0",
            "--para",
            "0",
            "--ctrl",
            "0",
            "--inner-para",
            "0",
            "--inner-ctrl",
            "0",
            "--props",
            r#"{"width":1000}"#,
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
    assert_eq!(v["innerPara"], 0);
    assert_eq!(v["innerCtrl"], 0);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "set-hf-picture",
            src.as_str(),
            "--section",
            "0",
            "--para",
            "0",
            "--ctrl",
            "0",
            "--inner-para",
            "0",
            "--inner-ctrl",
            "0",
            "--props",
            "{}",
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
        .any(|t| t["name"] == "hwp_set_hf_picture"));
}
