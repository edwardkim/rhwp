//! 관계형·고정형(JSON 값·CLI 종료 코드 계약; 실물 px 검사 아님): PR #8442의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:36:45+09:00 (`3d492e9ad10e76e935cf2e8a26a83605548dca92`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 이 파일은 조판 회귀가 아닌 JSON 값·CLI 적용·거부 계약 검사다. 시각 점수를 추정하지 않는다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-02T06:10:00+09:00
//!
//! [#8439] `edit apply-para-format --props '{"alignment": "center"}'` 처럼 콜론 뒤 공백이 있는
//! 표준 JSON 을 주면 `json_str`/`json_i32` 등이 `"키":값` 고정 문자열을 찾지 못해 아무 서식도
//! 바꾸지 않고 exit 0 으로 끝났다.
//!
//! 검사 범위: 문서 코어의 서식 JSON 읽기(공백·줄바꿈 유무와 무관하게 같은 결과)와 CLI 의
//! `--props` 판정(읽은 키가 하나도 없으면 exit 2). 조판 규칙을 바꾸지 않으며, 공백 없는 JSON
//! 의 적용 결과는 수정 전과 같다. 그래서 독립 PDF 시각 측정 대상이 아니다 — 기대값은 같은
//! 의미의 공백 없는 JSON 으로 적용한 결과다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::document_core::{
    char_format_props_have_effect, para_format_props_have_effect, DocumentCore,
};
use serde_json::Value;

static TEMP_SERIAL: AtomicU64 = AtomicU64::new(0);

fn core_with_text() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank document");
    core.insert_text_native(0, 0, 0, "서식 시험 문단")
        .expect("insert text");
    core
}

fn para_props_after(props: &str) -> Value {
    let mut core = core_with_text();
    core.apply_para_format_native(0, 0, props)
        .unwrap_or_else(|e| panic!("apply_para_format {props}: {e:?}"));
    let mut v: Value =
        serde_json::from_str(&core.get_para_properties_at_native(0, 0).expect("props")).unwrap();
    // 문단 모양 번호는 만들어진 순서에 따른 값이라 비교에서 뺀다.
    v.as_object_mut().unwrap().remove("paraShapeId");
    v
}

fn char_props_after(props: &str) -> Value {
    let mut core = core_with_text();
    core.apply_char_format_native(0, 0, 0, 3, props)
        .unwrap_or_else(|e| panic!("apply_char_format {props}: {e:?}"));
    let mut v: Value =
        serde_json::from_str(&core.get_char_properties_at_native(0, 0, 1).expect("props")).unwrap();
    v.as_object_mut().unwrap().remove("charShapeId");
    v
}

#[test]
fn spaced_para_format_json_applies_like_compact_json() {
    let base = para_props_after(r#"{"spacingAfter":0}"#);
    let compact = r#"{"alignment":"center","lineSpacing":200,"marginLeft":1000,"headType":"Outline","paraLevel":1,"tabStops":[{"position":4000,"type":1,"fill":0}],"borderSpacing":[10,20,30,40]}"#;
    let spaced = "{\n  \"alignment\" : \"center\",\n  \"lineSpacing\": 200,\n  \"marginLeft\":\t1000,\n  \"headType\": \"Outline\", \"paraLevel\": 1,\n  \"tabStops\": [ { \"position\": 4000, \"type\": 1, \"fill\": 0 } ],\n  \"borderSpacing\": [ 10, 20, 30, 40 ]\n}";
    let want = para_props_after(compact);
    assert_ne!(want, base, "공백 없는 JSON 은 서식을 바꿔야 한다");
    assert_eq!(want["alignment"], "center");
    assert_eq!(want["headType"], "Outline");
    assert_eq!(para_props_after(spaced), want);
}

#[test]
fn spaced_char_format_json_applies_like_compact_json() {
    let base = char_props_after(r#"{"italic":false}"#);
    let compact = r##"{"bold":true,"fontSize":1500,"textColor":"#FF0000","underline":true}"##;
    let spaced = "{ \"bold\" : true,\n \"fontSize\": 1500, \"textColor\" : \"#FF0000\", \"underline\": true }";
    let want = char_props_after(compact);
    assert_ne!(want, base);
    assert_eq!(want["bold"], true);
    assert_eq!(char_props_after(spaced), want);
}

#[test]
fn effect_predicates_follow_the_keys_each_apply_reads() {
    for props in [
        r#"{"alignment": "center"}"#,
        "{\n\"lineSpacing\":\t180\n}",
        r#"{"tabStops": []}"#,
        r#"{"borderSpacing": [1, 2, 3, 4]}"#,
    ] {
        assert!(para_format_props_have_effect(props), "{props}");
    }
    for props in [r#"{}"#, r#"{"aligment": "center"}"#, r#"{"bold": true}"#] {
        assert!(!para_format_props_have_effect(props), "{props}");
    }
    assert!(char_format_props_have_effect(r#"{ "bold" : true }"#));
    assert!(!char_format_props_have_effect(r#"{"bolt": true}"#));
    assert!(!char_format_props_have_effect(r#"{"alignment":"center"}"#));
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-8439-{tag}-{}-{}.hwpx",
        std::process::id(),
        TEMP_SERIAL.fetch_add(1, Ordering::Relaxed),
    ))
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/paper-square-table-tac-host.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn first_para_alignment(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap();
    let psid = doc.document().sections[0].paragraphs[0].para_shape_id;
    format!(
        "{:?}",
        doc.document().doc_info.para_shapes[psid as usize].alignment
    )
}

#[test]
fn cli_applies_spaced_props_and_rejects_props_without_known_keys() {
    let before = first_para_alignment(Path::new(&sample()));
    assert_ne!(
        before, "Center",
        "시험 문서의 첫 문단은 가운데 정렬이 아니어야 한다"
    );

    let out = temp("spaced");
    let run = Command::new(rhwp_bin())
        .args(["edit", "apply-para-format", &sample(), "--para", "0"])
        .args(["--props", r#"{"alignment": "center"}"#, "-o"])
        .arg(&out)
        .output()
        .unwrap();
    assert_eq!(run.status.code(), Some(0), "{run:?}");
    assert_eq!(first_para_alignment(&out), "Center");
    let _ = std::fs::remove_file(&out);

    for (cmd, props) in [
        ("apply-para-format", r#"{"aligment": "center"}"#),
        ("apply-char-format", r#"{"bolt": true}"#),
    ] {
        let out = temp("unknown");
        let run = Command::new(rhwp_bin())
            .args([
                "edit",
                cmd,
                &sample(),
                "--para",
                "0",
                "--props",
                props,
                "-o",
            ])
            .arg(&out)
            .output()
            .unwrap();
        assert_eq!(run.status.code(), Some(2), "{cmd} {props}: {run:?}");
        assert!(
            String::from_utf8_lossy(&run.stderr).contains("서식 키를 찾지 못했습니다"),
            "{cmd}: {}",
            String::from_utf8_lossy(&run.stderr)
        );
        assert!(!out.exists(), "{cmd}: 거부한 편집은 파일을 쓰지 않는다");
    }
}
