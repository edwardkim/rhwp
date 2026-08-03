//! [#3407] `info --json`·`batch info` 봉투의 `title` 필드 계약 테스트.
//!
//! 계약: `title` 은 렌더된 페이지 텍스트(`export-text --json` 과 같은 원천)의
//! 첫 의미 줄(trim 후 비어있지 않은 첫 줄)이다. 앞쪽 3쪽까지만 훑으며(표지가
//! 이미지·빈 쪽이면 다음 쪽으로 내려간다), 그래도 없으면 `null` 이다.
//! best-effort 필드로 값 자체는 계약이 아니지만, 필드의 존재·타입·"export-text
//! 첫 의미 줄과 동형" 규칙은 본 테스트가 잡는다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// 표지 첫 줄이 문서 제목인 표본 (제목 있는 경우).
const SAMPLE_COVER_TITLE: &str = "samples/2022년 국립국어원 업무계획.hwp";
/// 표지(0~1쪽)가 이미지라 렌더 텍스트가 비는 표본 — 다음 쪽 fallback 검증.
const SAMPLE_IMAGE_COVER: &str = "samples/2025 행정업무운영 편람(최종).hwp";
/// 앞쪽 페이지에 의미 줄이 전혀 없는 표본 — null fallback 검증.
const SAMPLE_NO_TEXT: &str = "samples/253E164F57A1BC6934-empty.hwp";
/// 단건/배치·export-text 동형 검증용 소형 표본.
const SAMPLE_SMALL: &str = "samples/hwp3-sample.hwp";

/// title 스캔 상한(쪽) — 구현의 TITLE_SCAN_PAGES 와 같은 값이어야 한다.
const TITLE_SCAN_PAGES: usize = 3;

fn sample_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn run_with_stdin(args: &[&str], stdin_body: &str) -> Output {
    let mut child = Command::new(rhwp_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    write_stdin_ignoring_early_exit(&mut child, stdin_body);
    child.wait_with_output().expect("rhwp 종료 대기 실패")
}

/// stdin 에 본문을 쓰되, 자식이 stdin 을 읽기 전에 종료한 경우의 BrokenPipe 는
/// 무시한다. 인자 검증 거부 계열 테스트는 프로세스가 입력을 소비하기 전에
/// 종료하는 것이 정상 경로라, 쓰기 완료 여부는 검증 대상(종료 코드·출력)이
/// 아니다 (#3763 — batch_axes_contract.rs 와 같은 처리).
fn write_stdin_ignoring_early_exit(child: &mut std::process::Child, stdin_body: &str) {
    use std::io::ErrorKind;
    if let Err(err) = child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(stdin_body.as_bytes())
    {
        assert_eq!(
            err.kind(),
            ErrorKind::BrokenPipe,
            "stdin 쓰기 실패: {err:?}"
        );
    }
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn info_json(rel: &str) -> serde_json::Value {
    let sample = sample_path(rel);
    let args = ["info", "--json", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(&args, &output)
        )
    })
}

/// `export-text --json` 봉투에서 계약 규칙대로 첫 의미 줄을 뽑는다 — 종전
/// 2-pass 대장화가 소비자 쪽에서 하던 파싱과 같은 규칙.
fn first_meaningful_line_via_export_text(rel: &str) -> Option<String> {
    let sample = sample_path(rel);
    let args = ["export-text", "--json", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("export-text JSON");
    for page in v["pages"]
        .as_array()
        .expect("pages")
        .iter()
        .take(TITLE_SCAN_PAGES)
    {
        for line in page["text"].as_str().expect("text").lines() {
            let t = line.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

// ── info --json title ──────────────────────────────────────────────────────

#[test]
fn info_json_title_is_cover_first_meaningful_line() {
    let v = info_json(SAMPLE_COVER_TITLE);
    assert_eq!(
        v["title"], "2022년 국립국어원 업무계획",
        "표지 첫 의미 줄이 title 이어야 합니다: {v}"
    );
    // 기존 봉투 필드 무회귀 가드 — 필드 추가만 허용, 변경·삭제 금지.
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["format"], "hwp5", "{v}");
    assert!(v["sizeBytes"].as_u64().is_some(), "{v}");
    assert!(v["pageCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["paraCount"].as_u64().unwrap() >= 1, "{v}");
    assert!(v["fonts"].is_array(), "{v}");
}

#[test]
fn info_json_title_falls_back_past_image_cover() {
    // 표지 0~1쪽이 이미지(렌더 텍스트 "")인 문서 — 이슈 #3407 3안의 fallback.
    let v = info_json(SAMPLE_IMAGE_COVER);
    assert_eq!(
        v["title"], "행정업무운영 편람",
        "표지가 비면 다음 쪽 첫 의미 줄로 내려가야 합니다: {v}"
    );
}

#[test]
fn info_json_title_null_when_front_pages_have_no_text() {
    let v = info_json(SAMPLE_NO_TEXT);
    assert!(
        v.get("title").is_some(),
        "title 필드는 값이 없어도 존재해야 합니다(생략 금지): {v}"
    );
    assert!(
        v["title"].is_null(),
        "의미 줄이 없으면 title 은 null 이어야 합니다: {v}"
    );
}

#[test]
fn info_json_title_matches_export_text_first_line() {
    // 동형 계약: 1-pass(title)와 종전 2-pass(export-text 첫 의미 줄)는 같은 값이다.
    let expected = first_meaningful_line_via_export_text(SAMPLE_SMALL);
    let v = info_json(SAMPLE_SMALL);
    match expected {
        Some(line) => assert_eq!(v["title"], line.as_str(), "{v}"),
        None => assert!(v["title"].is_null(), "{v}"),
    }
}

// ── batch info --json title ────────────────────────────────────────────────

#[test]
fn batch_info_records_carry_title() {
    let with_title = sample_path(SAMPLE_COVER_TITLE);
    let no_text = sample_path(SAMPLE_NO_TEXT);
    let args = ["batch", "info", "--json"];
    let stdin_body = format!(
        "{}\n{}\n",
        with_title.to_str().unwrap(),
        no_text.to_str().unwrap()
    );
    let output = run_with_stdin(&args, &stdin_body);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let records: Vec<serde_json::Value> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("NDJSON 아님 ({e}): {l}")))
        .collect();
    assert_eq!(records.len(), 2, "{}", describe(&args, &output));
    // 단건 info --json 과 같은 스키마 — 1-pass 대장화의 핵심 계약.
    assert_eq!(
        records[0]["title"], "2022년 국립국어원 업무계획",
        "{records:?}"
    );
    assert!(records[1]["title"].is_null(), "{records:?}");
    for v in &records {
        assert_eq!(v["schemaVersion"], "1.0", "{v}");
        assert!(v["fonts"].is_array(), "{v}");
    }
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
