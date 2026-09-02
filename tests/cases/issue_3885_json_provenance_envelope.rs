//! [#3885] 표지는 항상 실린다 — 빠졌던 JSON 봉투 4건을 실 CLI 로 잠근다.
//!
//! `provenance::marked` 가 단일 출처다. 이 테스트는 그 helper 를 다시 만들지 않고
//! `rhwp --json` stdout 을 파싱해 키 존재와 값을 본다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

/// 검증 숫자(mod 11)를 통과하는 가공 주민등록번호. 실재 인물과 무관하다.
const VALID_SSN: &str = "900101-1234568";
const VALID_CARD: &str = "4111-1111-1111-1111";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp-3885-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("작업 디렉터리");
    dir.join(name)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], out: &Output) -> String {
    format!(
        "args={args:?}\nexit={:?}\nstdout={}\nstderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn json_of(args: &[&str], out: &Output) -> Value {
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout 이 JSON 이 아닙니다({e}): {}", describe(args, out)))
}

fn fields_of(v: &Value) -> Vec<&str> {
    v["untrustedFields"]
        .as_array()
        .unwrap_or_else(|| panic!("untrustedFields 배열 없음: {v}"))
        .iter()
        .filter_map(|x| x.as_str())
        .collect()
}

fn make_pii_document() -> Option<PathBuf> {
    let src = repo("samples/field-01.hwp");
    if !src.exists() {
        eprintln!("샘플 없음({}) — 건너뜀", src.display());
        return None;
    }
    let out = scratch("pii.hwp");
    let _ = std::fs::remove_file(&out);
    let data = format!(
        r#"{{"작성자":"홍길동 {VALID_SSN}","전화번호":"010-1234-5678","이메일":"hong@example.com","회사명":"카드 {VALID_CARD}"}}"#
    );
    let args = [
        "edit",
        "fill-fields",
        src.to_str().unwrap(),
        "--data",
        &data,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let result = run(&args);
    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        describe(&args, &result)
    );
    Some(out)
}

/// 문서를 열지 않는 스키마 명령도 `untrustedContent:false` 를 명시한다.
/// 키 부재는 "안전"이 아니라 "이 빌드는 표지를 모른다"로 읽힌다.
#[test]
fn schema_commands_state_untrusted_content_false() {
    for cmd in ["export-ir-schema", "export-capabilities-schema"] {
        let args = [cmd, "--json"];
        let out = run(&args);
        assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
        let v = json_of(&args, &out);
        assert!(
            v.get("untrustedContent").is_some(),
            "{cmd}: untrustedContent 키 부재: {v}"
        );
        assert_eq!(
            v["untrustedContent"],
            Value::Bool(false),
            "{cmd}: 문서를 열지 않는데 false 가 아닙니다: {v}"
        );
        assert_eq!(
            v["untrustedFields"],
            Value::Array(Vec::new()),
            "{cmd}: untrustedFields 가 빈 배열이 아닙니다: {v}"
        );
    }
}

/// `edit redact --dry-run --json` 은 `findings[].raw`(원문 개인정보)를 싣는다.
/// 그 봉투에 표지가 없으면 S1 계약이 가장 민감한 값에서 무너진다.
#[test]
fn redact_dry_run_marks_findings_raw_untrusted() {
    let Some(doc) = make_pii_document() else {
        return;
    };
    let p = doc.to_str().expect("경로 UTF-8");
    let args = ["edit", "redact", p, "--dry-run", "--json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert!(
        v["findingCount"].as_u64().unwrap_or(0) > 0,
        "탐지 0건이면 이 가드는 공허하다: {v}"
    );
    assert_eq!(
        v["untrustedContent"],
        Value::Bool(true),
        "raw 원문을 싣는 봉투인데 untrustedContent 가 true 가 아닙니다: {v}"
    );
    let fields = fields_of(&v);
    assert!(
        fields.contains(&"findings[].raw"),
        "findings[].raw 가 untrustedFields 에 없습니다: {v}"
    );
    assert!(
        fields.contains(&"findings[].masked"),
        "findings[].masked 가 untrustedFields 에 없습니다: {v}"
    );
}

/// `--no-raw` 면 raw 경로가 봉투에 없으므로 표지에서도 빠진다.
/// masked 는 남아 `untrustedContent:true` 를 유지한다.
#[test]
fn redact_no_raw_drops_raw_from_untrusted_fields() {
    let Some(doc) = make_pii_document() else {
        return;
    };
    let p = doc.to_str().expect("경로 UTF-8");
    let args = ["edit", "redact", p, "--dry-run", "--json", "--no-raw"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(v["untrustedContent"], Value::Bool(true), "{v}");
    let fields = fields_of(&v);
    assert!(
        !fields.contains(&"findings[].raw"),
        "--no-raw 인데 표지가 findings[].raw 를 주장합니다: {v}"
    );
    assert!(fields.contains(&"findings[].masked"), "{v}");
    if let Some(items) = v["findings"].as_array() {
        for item in items {
            assert!(
                item.get("raw").is_none(),
                "--no-raw 인데 findings[].raw 가 남아 있습니다: {v}"
            );
        }
    }
}

/// `edit sanitize --json` 의 `removed[].before` 는 지워진 문서 속성 원문이다.
#[test]
fn sanitize_marks_removed_before_untrusted() {
    let Some(doc) = make_pii_document() else {
        return;
    };
    let p = doc.to_str().expect("경로 UTF-8");
    let outp = scratch("sanitized.hwp");
    let o = outp.to_str().expect("경로 UTF-8");
    let args = ["edit", "sanitize", p, "-o", o, "--json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert!(
        v["removedCount"].as_u64().unwrap_or(0) > 0,
        "제거된 속성이 없으면 이 검사는 공허합니다: {v}"
    );
    assert_eq!(v["untrustedContent"], Value::Bool(true), "{v}");
    let fields = fields_of(&v);
    assert!(
        fields.contains(&"removed[].before"),
        "removed[].before 가 untrustedFields 에 없습니다: {v}"
    );
}
