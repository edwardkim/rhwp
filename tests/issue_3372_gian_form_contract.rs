//! [#3372] 일반기안문 표준 서식 자산 계약 테스트.
//!
//! 계약: `tools/forms/일반기안문_서식.hwpx` 는 ① rhwp 가 파싱·렌더할 수 있는 유효한
//! HWPX 이고 ② 별지 제1호서식의 기입 위치 23곳이 누름틀로 실재하며 이름이 고정이다
//! (에이전트가 `fields --json` 자기서술만으로 채움 데이터를 구성한다 — 이름이 바뀌면
//! 소비자가 깨진다). 값 채움 계약은 `edit fill-fields`(#3345) 쪽 테스트가 담당한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FORM: &str = "tools/forms/일반기안문_서식.hwpx";
const FORM_GANI: &str = "tools/forms/간이기안문_서식.hwpx";

/// 별지 제2호서식(간이기안문) 기입 위치 — 결재란 표 포함.
const REQUIRED_FIELDS_GANI: &[&str] = &[
    "생산등록번호",
    "등록일",
    "결재일",
    "공개구분",
    "결재직위1",
    "결재직위2",
    "결재직위3",
    "결재직위4",
    "협조자",
    "제목",
    "요약설명",
    "작성일",
    "작성기관",
];

/// 별지 제1호서식 기입 위치 — 이름 집합이 곧 서식의 공개 계약이다.
const REQUIRED_FIELDS: &[&str] = &[
    "행정기관명",
    "수신자",
    "경유",
    "제목",
    "본문",
    "붙임",
    "발신명의",
    "수신자명단",
    "기안자",
    "검토자",
    "결재권자",
    "협조자",
    "시행번호",
    "시행일",
    "접수번호",
    "접수일",
    "우편번호",
    "주소",
    "홈페이지",
    "전화번호",
    "팩스번호",
    "전자우편",
    "공개구분",
];

fn form_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FORM)
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// 서식이 유효한 HWPX 로 파싱된다.
#[test]
fn form_parses_as_valid_hwpx() {
    let p = form_path();
    let args = ["info", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("순수 JSON");
    assert_eq!(v["format"], "hwpx", "{v}");
    assert!(v["paraCount"].as_u64().unwrap_or(0) >= 15, "{v}");
}

/// 기입 위치 23곳이 정확한 이름의 누름틀로 실재한다.
#[test]
fn form_exposes_all_required_fields() {
    let p = form_path();
    let args = ["fields", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("순수 JSON");
    let names: Vec<&str> = v["fields"]
        .as_array()
        .expect("fields 배열")
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    for required in REQUIRED_FIELDS {
        assert!(
            names.contains(required),
            "누름틀 '{required}' 이 서식에 없습니다. 실재 목록: {names:?}"
        );
    }
    assert_eq!(
        names.len(),
        REQUIRED_FIELDS.len(),
        "서식의 누름틀 수가 계약과 다릅니다: {names:?}"
    );
}

/// 서식이 렌더 가능하다 (SVG smoke) — 배치 회귀의 최소 게이트.
#[test]
fn form_renders_to_svg() {
    let p = form_path();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let out = std::env::temp_dir().join(format!("rhwp-3372-{}-{nonce}", std::process::id()));
    let args = [
        "export-svg",
        p.to_str().unwrap(),
        "-p",
        "0",
        "-o",
        out.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let produced = std::fs::read_dir(&out)
        .expect("출력 폴더")
        .filter_map(Result::ok)
        .any(|e| e.path().extension().is_some_and(|x| x == "svg"));
    assert!(produced, "SVG 산출물이 있어야 합니다");
    let _ = std::fs::remove_dir_all(&out);
}

/// 간이기안문 — 결재란 표 안 누름틀까지 이름 집합이 고정이다.
#[test]
fn gani_form_exposes_all_required_fields() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(FORM_GANI);
    let args = ["fields", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("순수 JSON");
    let names: Vec<&str> = v["fields"]
        .as_array()
        .expect("fields 배열")
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    for required in REQUIRED_FIELDS_GANI {
        assert!(
            names.contains(required),
            "누름틀 '{required}' 이 서식에 없습니다. 실재 목록: {names:?}"
        );
    }
    assert_eq!(names.len(), REQUIRED_FIELDS_GANI.len(), "{names:?}");
}

/// 간이기안문 렌더 smoke — 표 2개(등록표·결재란 병합)가 렌더 가능해야 한다.
#[test]
fn gani_form_renders_to_svg() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(FORM_GANI);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let out = std::env::temp_dir().join(format!("rhwp-3372g-{}-{nonce}", std::process::id()));
    let args = [
        "export-svg",
        p.to_str().unwrap(),
        "-p",
        "0",
        "-o",
        out.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let _ = std::fs::remove_dir_all(&out);
}
