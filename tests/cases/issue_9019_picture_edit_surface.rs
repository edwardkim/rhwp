//! 관계형(CLI 결과 JSON 의 컨트롤 번호 ↔ 문서 안 그림 컨트롤, 거부한 편집 ↔ 출력 파일 없음; 실물 px 검사 아님): #9019 회귀.
//! 주석 갱신: 2026-11-12T22:22:47+09:00
//! 이 파일은 조판 회귀가 아닌 CLI 결과 JSON·종료 코드 계약 검사다. 조판 규칙과 그림 배치 계산을
//! 바꾸지 않으므로 독립 PDF 시각 측정 대상이 아니며 시각 점수를 추정하지 않는다.
//!
//! [#9019] `edit insert-picture`·`insert-image` 의 `--json` 결과에 넣은 그림의 컨트롤 번호가
//! 없어서 다음 `set-picture --ctrl` 값을 따로 찾아야 했다. `edit set-picture` 는 `--props` 의
//! 모르는 키·틀린 열거 값·JSON 이 아닌 입력을 건너뛰고 exit 0 으로 끝났다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use rhwp::model::control::Control;
use rhwp::model::shape::TextWrap;
use serde_json::Value;

static TEMP_SERIAL: AtomicU64 = AtomicU64::new(0);

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-9019-{tag}-{}-{}.hwpx",
        std::process::id(),
        TEMP_SERIAL.fetch_add(1, Ordering::Relaxed),
    ))
}

fn repo_path(rel: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .into_owned()
}

fn sample() -> String {
    repo_path("samples/paper-square-table-tac-host.hwpx")
}

fn image() -> String {
    repo_path("samples/images/moogung.jpg")
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin()).args(args).output().unwrap()
}

fn envelope(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "결과 JSON: {e} / {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

/// 결과 JSON 이 가리키는 자리의 그림 배치 방식. 그림이 아니면 실패한다.
fn picture_wrap(path: &Path, section: usize, para: usize, ctrl: usize) -> TextWrap {
    let bytes = std::fs::read(path).unwrap();
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap();
    match &doc.document().sections[section].paragraphs[para].controls[ctrl] {
        Control::Picture(picture) => picture.common.text_wrap,
        other => panic!("구역 {section} 문단 {para} 컨트롤 {ctrl} 은 그림이 아니다: {other:?}"),
    }
}

fn index(v: &Value, key: &str) -> usize {
    v[key]
        .as_u64()
        .unwrap_or_else(|| panic!("결과 JSON 에 {key} 가 없다: {v}")) as usize
}

#[test]
fn insert_picture_reports_ctrl_that_set_picture_accepts() {
    let inserted = temp("pic");
    let out = run(&[
        "edit",
        "insert-picture",
        &sample(),
        "--image",
        &image(),
        "--para",
        "1",
        "--width",
        "8000",
        "--json",
        "-o",
        inserted.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let env = envelope(&out);
    let (section, para, ctrl) = (
        index(&env, "section"),
        index(&env, "paragraph"),
        index(&env, "ctrl"),
    );
    assert_ne!(
        picture_wrap(&inserted, section, para, ctrl),
        TextWrap::TopAndBottom
    );

    let changed = temp("set");
    let out = run(&[
        "edit",
        "set-picture",
        inserted.to_str().unwrap(),
        "--section",
        &section.to_string(),
        "--para",
        &para.to_string(),
        "--ctrl",
        &ctrl.to_string(),
        "--props",
        r#"{"textWrap": "TopAndBottom"}"#,
        "-o",
        changed.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    assert_eq!(
        picture_wrap(&changed, section, para, ctrl),
        TextWrap::TopAndBottom
    );

    let dry = run(&[
        "edit",
        "insert-picture",
        &sample(),
        "--image",
        &image(),
        "--para",
        "1",
        "--dry-run",
        "--json",
    ]);
    assert_eq!(dry.status.code(), Some(0), "{dry:?}");
    assert!(
        envelope(&dry)["ctrl"].is_null(),
        "dry-run 은 넣지 않으므로 ctrl 이 null 이다"
    );

    let _ = std::fs::remove_file(&inserted);
    let _ = std::fs::remove_file(&changed);
}

#[test]
fn insert_image_reports_where_the_picture_went() {
    let inserted = temp("image");
    let out = run(&[
        "edit",
        "insert-image",
        &sample(),
        "--image",
        &image(),
        "--x",
        "5000",
        "--y",
        "5000",
        "--json",
        "-o",
        inserted.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let env = envelope(&out);
    let (section, para, ctrl) = (
        index(&env, "section"),
        index(&env, "paragraph"),
        index(&env, "ctrl"),
    );
    // 결과가 가리키는 컨트롤이 그림이어야 한다 (그림이 아니면 picture_wrap 이 실패한다).
    picture_wrap(&inserted, section, para, ctrl);
    let _ = std::fs::remove_file(&inserted);
}

#[test]
fn set_picture_rejects_props_the_setter_would_skip() {
    let inserted = temp("base");
    let out = run(&[
        "edit",
        "insert-picture",
        &sample(),
        "--image",
        &image(),
        "--para",
        "1",
        "--json",
        "-o",
        inserted.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let ctrl = index(&envelope(&out), "ctrl").to_string();

    for (props, needle) in [
        (r#"{"textWrap":"topAndBottom"}"#, "textWrap"),
        (r#"{"wrap":"TopAndBottom"}"#, "wrap"),
        (
            r#"{"textWrap":"TopAndBottom","vertRelto":"Para"}"#,
            "vertRelto",
        ),
        (r#"{"captionDirection":"Top"}"#, "hasCaption"),
        ("abc", "JSON"),
        ("[1,2]", "JSON 객체"),
    ] {
        for dry_run in [false, true] {
            let out_path = temp("rejected");
            let mut args = vec![
                "edit",
                "set-picture",
                inserted.to_str().unwrap(),
                "--section",
                "0",
                "--para",
                "1",
                "--ctrl",
                &ctrl,
                "--props",
                props,
                "-o",
                out_path.to_str().unwrap(),
            ];
            if dry_run {
                args.push("--dry-run");
            }
            let out = run(&args);
            let stderr = String::from_utf8_lossy(&out.stderr);
            assert_eq!(
                out.status.code(),
                Some(2),
                "{props} dry_run={dry_run}: {stderr}"
            );
            assert!(
                stderr.contains(needle),
                "{props}: 오류에 {needle} 가 없다: {stderr}"
            );
            assert!(
                out.stdout.is_empty(),
                "{props}: 거부할 때 stdout 은 비어 있다"
            );
            assert!(
                !out_path.exists(),
                "{props}: 거부한 편집은 파일을 쓰지 않는다"
            );
        }
    }
    let _ = std::fs::remove_file(&inserted);
}

#[test]
fn check_picture_props_accepts_setter_values_and_rejects_the_rest() {
    use rhwp::document_core::check_picture_props;
    for json in [
        r#"{"textWrap":"TopAndBottom","vertRelTo":"Para","horzAlign":"Center"}"#,
        r#"{"treatAsChar":true,"width":20000,"brightness":-20}"#,
        r#"{"hasCaption":true,"captionDirection":"Top","captionSpacing":850}"#,
        r#"{"effect":"RealPic","description":"그림 1"}"#,
    ] {
        assert!(check_picture_props(json).is_ok(), "받아야 함: {json}");
    }
    for json in [
        "abc",
        "[1,2]",
        "{}",
        r#"{"wrap":"TopAndBottom"}"#,
        r#"{"textWrap":"topAndBottom"}"#,
        r#"{"textWrap":"TopAndBottom","vertRelto":"Para"}"#,
        r#"{"horzAlign":"Inside"}"#,
        r#"{"treatAsChar":"true"}"#,
        r#"{"width":-1}"#,
        r#"{"paddingLeft":40000}"#,
        r#"{"vertOffset":1.5}"#,
        r#"{"captionDirection":"Top"}"#,
        r#"{"hasCaption":false,"captionDirection":"Top"}"#,
    ] {
        assert!(check_picture_props(json).is_err(), "거부해야 함: {json}");
    }
}

/// `src` 에서 `fn name(` 으로 시작하는 함수 본문(다음 함수 정의 앞까지)을 잘라 낸다.
fn fn_body<'a>(src: &'a str, name: &str) -> &'a str {
    let start = src
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("함수 {name} 이 없다"));
    let rest = &src[start + 3..];
    let end = ["\n    pub fn ", "\n    pub(crate) fn ", "\n    fn "]
        .iter()
        .filter_map(|marker| rest.find(marker))
        .min()
        .unwrap_or(rest.len());
    &src[start..start + 3 + end]
}

/// setter 본문에서 `props_json, "키"` 로 읽는 키 전부.
fn keys_read_by(bodies: &[&str]) -> std::collections::BTreeSet<String> {
    let mut keys = std::collections::BTreeSet::new();
    for body in bodies {
        for (i, _) in body.match_indices("props_json, \"") {
            let rest = &body[i + "props_json, \"".len()..];
            keys.insert(rest[..rest.find('"').unwrap()].to_string());
        }
    }
    keys
}

/// 검사 오류가 알려 주는 받는 키 목록.
fn keys_listed_by(check: fn(&str) -> Result<(), String>) -> std::collections::BTreeSet<String> {
    let err = check(r#"{"__unknown__":1}"#).unwrap_err();
    let list = &err[err.find("받는 키: ").expect("받는 키 목록") + "받는 키: ".len()..];
    list[..list.rfind(')').unwrap()]
        .split(", ")
        .map(str::to_string)
        .collect()
}

/// 검사가 받는 키와 setter 가 읽는 키가 같아야 한다. 검사에만 있으면 통과한 키가 다시 조용히
/// 건너뛰어지고, setter 에만 있으면 쓸 수 있는 속성을 거부한다.
#[test]
fn picture_prop_list_matches_the_setter() {
    let picture = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/document_core/commands/object_ops/picture.rs"
    ));
    assert_eq!(
        keys_listed_by(rhwp::document_core::check_picture_props),
        keys_read_by(&[fn_body(picture, "apply_picture_props_inner")])
    );
}
