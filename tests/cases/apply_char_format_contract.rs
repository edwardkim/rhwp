//! 관계형·고정형: 글꼴 이름 편집 API의 언어 슬롯·저장 보존과 CLI 응답 계약을 검사한다.
//! 주석 갱신: 2026-11-07 07:54:14 KST.
//! 직전 파일 변경: ed272bf6d8e38bdd22e9f78ef72ea9bfc0b8692a (2026-11-06 22:49:11 +09:00).
//! 합성 편집 상태·기계 응답 검사이며 실물 조판의 px 좌표나 Print 일치율을 고정하지 않는다.
//! field-01 입력은 API 상태 보존의 기존 대조군이며 설치 글꼴을 통과 조건으로 사용하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

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

fn first_body_para(path: &str) -> (usize, usize) {
    let bytes = std::fs::read(path).expect("sample");
    let doc = HwpDocument::from_bytes(&bytes).expect("parse");
    for (si, sec) in doc.document().sections.iter().enumerate() {
        for (pi, p) in sec.paragraphs.iter().enumerate() {
            if p.text.chars().count() >= 2 {
                return (si, pi);
            }
        }
    }
    panic!("글자 2개 이상인 본문 문단이 없다");
}

fn para_has_superscript(path: &Path, section: usize, para: usize) -> bool {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let shapes = &doc.document().doc_info.char_shapes;
    doc.document().sections[section].paragraphs[para]
        .char_shapes
        .iter()
        .any(|cs| {
            shapes
                .get(cs.char_shape_id as usize)
                .is_some_and(|s| s.superscript)
        })
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-chfmt-{tag}-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

#[test]
fn apply_superscript_is_visible() {
    let src = sample();
    let (section, para) = first_body_para(&src);
    assert!(
        !para_has_superscript(Path::new(&src), section, para),
        "샘플이 이미 위첨자라 판별이 안 된다"
    );
    let out = temp("out");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-char-format",
            src.as_str(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--offset",
            "0",
            "--count",
            "2",
            "--props",
            r#"{"superscript":true}"#,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["section"], section);
    assert_eq!(v["paragraph"], para);
    assert_eq!(v["offset"], 0);
    assert_eq!(v["count"], 2);
    assert!(
        para_has_superscript(&out, section, para),
        "위첨자가 저장본에 없다"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn dry_run_json_has_fields_and_no_file() {
    let src = sample();
    let (section, para) = first_body_para(&src);
    let out = temp("dry");
    let sec_s = section.to_string();
    let para_s = para.to_string();
    let output = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-char-format",
            src.as_str(),
            "--section",
            &sec_s,
            "--para",
            &para_s,
            "--offset",
            "0",
            "--count",
            "2",
            "--props",
            r#"{"superscript":true}"#,
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
    assert_eq!(v["count"], 2);
}

#[test]
fn unknown_flag_empty_stdout() {
    let src = sample();
    let out = Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-char-format",
            src.as_str(),
            "--props",
            r#"{"superscript":true}"#,
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
        .any(|t| t["name"] == "hwp_apply_char_format"));
}

/// 저장본에서 문단 첫 글자의 한글·영문 슬롯 글꼴 이름을 읽는다.
fn para_font_names(path: &Path, section: usize, para: usize) -> (String, String) {
    let bytes = std::fs::read(path).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();
    let info = &doc.document().doc_info;
    let id = doc.document().sections[section].paragraphs[para]
        .char_shape_id_at(0)
        .expect("글자 모양");
    let cs = &info.char_shapes[id as usize];
    let name = |lang: usize| {
        info.font_faces[lang][cs.font_ids[lang] as usize]
            .name
            .clone()
    };
    (name(0), name(1))
}

fn apply_font_name(
    src: &str,
    out: &Path,
    section: usize,
    para: usize,
    props: &str,
) -> std::process::Output {
    Command::new(rhwp_bin())
        .args([
            "edit",
            "apply-char-format",
            src,
            "--section",
            &section.to_string(),
            "--para",
            &para.to_string(),
            "--props",
            props,
            "-o",
            out.to_str().unwrap(),
            "--json",
        ])
        .output()
        .unwrap()
}

/// `fontName` 은 문서에 없는 글꼴을 슬롯마다 등록하고, 저장본(HWP·HWPX) 글자 모양이 그 이름을 가리킨다.
#[test]
fn font_name_registers_and_applies_in_hwp_and_hwpx() {
    let src = sample();
    let (section, para) = first_body_para(&src);
    let face = "rhwp테스트글꼴";
    let (before, _) = para_font_names(Path::new(&src), section, para);
    assert_ne!(before, face, "샘플이 이미 같은 글꼴이라 판별이 안 된다");
    for ext in ["hwp", "hwpx"] {
        let out = temp("font").with_extension(ext);
        let props = format!(r#"{{"fontName":"{face}"}}"#);
        let output = apply_font_name(&src, &out, section, para, &props);
        assert_eq!(output.status.code(), Some(0), "{ext}: {:?}", output);
        assert_eq!(
            para_font_names(&out, section, para),
            (face.to_string(), face.to_string()),
            "{ext}: 저장본 글꼴이 fontName 과 다르다"
        );
        let _ = std::fs::remove_file(&out);
    }
}

/// 이미 있는 글꼴 이름은 새로 등록하지 않고 그 ID 를 쓴다.
#[test]
fn font_name_reuses_existing_face() {
    let src = sample();
    let (section, para) = first_body_para(&src);
    let (existing, _) = para_font_names(Path::new(&src), section, para);
    let bytes = std::fs::read(&src).unwrap();
    let count = HwpDocument::from_bytes(&bytes)
        .unwrap()
        .document()
        .doc_info
        .font_faces[0]
        .len();
    let out = temp("font-reuse");
    let props = format!(r#"{{"fontName":"{existing}"}}"#);
    let output = apply_font_name(&src, &out, section, para, &props);
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    let bytes = std::fs::read(&out).unwrap();
    let after = HwpDocument::from_bytes(&bytes)
        .unwrap()
        .document()
        .doc_info
        .font_faces[0]
        .len();
    assert_eq!(after, count, "있는 글꼴을 다시 등록했다");
    assert_eq!(para_font_names(&out, section, para).0, existing);
    let _ = std::fs::remove_file(&out);
}

/// `fontName` 과 `fontIds` 를 함께 주면 조용히 한쪽을 버리지 않고 실패한다.
#[test]
fn font_name_with_font_ids_is_rejected() {
    let src = sample();
    let (section, para) = first_body_para(&src);
    let out = temp("font-conflict");
    let output = apply_font_name(
        &src,
        &out,
        section,
        para,
        r#"{"fontName":"바탕","fontIds":[0,0,0,0,0,0,0]}"#,
    );
    assert_ne!(output.status.code(), Some(0), "{:?}", output);
    assert!(!out.exists());
}
