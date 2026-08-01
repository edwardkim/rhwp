//! [#3633 후속] `digest` v2 계약 테스트 — 주소 보존 절 단위 청킹 + 쪽 범위 발췌.
//!
//! 계약 요지:
//! - `digest --sections --json`: 페이지 발췌 대신 **구조 노드별 청크**
//!   `sections:[{title,page,charCount,excerpt}]` 를 낸다. `page` 는 0 기준 글로벌
//!   쪽 번호(주소 보존) — LLM 이 절 단위로 요약하고 쪽 번호로 인용할 수 있다.
//!   구조가 없는 문서는 쪽 단위 폴백(`sectionsMode:"page"`)으로 강등하되
//!   강등 사실을 봉투에 명시한다(판정 가능).
//! - `digest --pages a..b --json`: 범위 지정 발췌(대형 문서 분할 요약용).
//!   남은 범위가 있으면 `nextStep` 이 다음 호출을 그대로 받아 적게 안내한다.
//! - 실패 시 stdout 0바이트, 종료 코드는 [#2707] 계약(0/1/2). v1 기본 봉투는 무회귀.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 구조(조문 패턴 15개 노드)와 여러 쪽(16쪽)을 모두 가진 실제 샘플 — v1 테스트와 동일 원천.
const SAMPLE: &str = "samples/hwp3-sample.hwp";
/// 구조 노드가 하나도 없는 실제 샘플(6쪽) — 쪽 단위 폴백 검증용.
const NO_STRUCTURE_SAMPLE: &str = "samples/2026_oss_rst.hwp";

/// [#3633 후속] sections 모드 nextStep 고정 문자열 계약 — 구현과 문자 그대로 일치.
const SECTIONS_NEXT_STEP: &str = "절 원문은 export-text --json -p <쪽>, 찾으려면 search --json";
/// [#3633 후속] pages 모드에서 남은 범위가 없을 때의 고정 문자열 계약.
const PAGES_DONE_NEXT_STEP: &str = "범위 발췌 완료 — 더 찾으려면 search --json";
/// [#3633] v1 기본 모드 nextStep — 무회귀 확인용 (digest_macro_contract.rs 와 동일).
const V1_NEXT_STEP: &str = "더 읽으려면 export-text --json -p <쪽>, 찾으려면 search --json";

fn sample_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
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

fn parse_stdout_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

// ── ① sections 모드: 주소 보존 절 단위 청킹 ────────────────────────────────

#[test]
fn digest_sections_envelope_and_addresses() {
    let sample = sample_path(SAMPLE);
    let args = ["digest", "--sections", "--json", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    // 봉투는 한 줄 — 초소형 모델이 줄 단위로 그대로 삼킬 수 있어야 한다.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout.lines().filter(|l| !l.trim().is_empty()).count(),
        1,
        "봉투는 한 줄이어야 합니다.\n{}",
        describe(&args, &output)
    );

    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["format"], "hwp3", "{v}");
    let page_count = v["pageCount"].as_u64().expect("pageCount");
    // 구조가 있는 문서는 구조 유래 청크여야 한다(쪽 폴백 금지).
    assert!(
        v["sectionsMode"] == "outline" || v["sectionsMode"] == "clause",
        "구조 있는 문서의 sectionsMode 는 outline|clause 여야 합니다: {v}"
    );
    let sections = v["sections"].as_array().expect("sections 배열");
    assert!(!sections.is_empty(), "{v}");
    // sectionCount 는 절단 전 전체 개수 — 봉투만 보고 누락 여부를 판정할 수 있어야 한다.
    let section_count = v["sectionCount"].as_u64().expect("sectionCount");
    assert!(section_count >= sections.len() as u64, "{v}");

    let mut prev_page = 0u64;
    let mut any_excerpt = false;
    for s in sections {
        assert!(s["title"].is_string(), "title 은 문자열: {v}");
        // 주소 보존 계약: page 는 0 기준 글로벌 쪽 번호이고 pageCount 안에 있어야 한다.
        let page = s["page"]
            .as_u64()
            .unwrap_or_else(|| panic!("page 누락: {s}"));
        assert!(
            page < page_count,
            "page {page} 가 pageCount {page_count} 밖: {v}"
        );
        // 문서 순서 순회이므로 쪽 번호는 단조 비감소여야 한다(주소 신뢰성).
        assert!(page >= prev_page, "쪽 번호 역행: {v}");
        prev_page = page;
        // 판정 가능 계약: charCount(원문 전체) vs excerpt(발췌)로 잔여량을 판정한다.
        let char_count = s["charCount"].as_u64().expect("charCount");
        let excerpt = s["excerpt"].as_str().expect("excerpt");
        assert!(
            excerpt.chars().count() as u64 <= char_count,
            "excerpt 가 charCount 보다 김: {s}"
        );
        if !excerpt.is_empty() {
            any_excerpt = true;
        }
    }
    assert!(any_excerpt, "발췌가 전부 비어 있습니다: {v}");
    // sections 모드 nextStep 고정 문자열 계약.
    assert_eq!(v["nextStep"], SECTIONS_NEXT_STEP, "{v}");
    // v1 의 페이지 발췌(excerpt)·outline 은 sections 봉투에 실리지 않는다(중복 컨텍스트 금지).
    assert!(v.get("excerpt").is_none(), "{v}");
    assert!(v.get("outline").is_none(), "{v}");
}

#[test]
fn digest_sections_max_chars_caps_each_excerpt() {
    let sample = sample_path(SAMPLE);
    let args = [
        "digest",
        "--sections",
        "--max-chars",
        "8",
        "--json",
        sample.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    for s in v["sections"].as_array().expect("sections") {
        assert!(
            s["excerpt"].as_str().expect("excerpt").chars().count() <= 8,
            "sections 모드에서 --max-chars 는 절별 발췌 상한이어야 합니다: {s}"
        );
    }
    assert_eq!(v["truncated"], true, "{v}");
}

#[test]
fn digest_sections_page_fallback_when_no_structure() {
    let sample = sample_path(NO_STRUCTURE_SAMPLE);
    let args = ["digest", "--sections", "--json", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    // 구조 없는 문서: 쪽 단위 폴백으로 강등하되 강등 사실을 명시한다(판정 가능).
    assert_eq!(v["sectionsMode"], "page", "{v}");
    let sections = v["sections"].as_array().expect("sections");
    let page_count = v["pageCount"].as_u64().expect("pageCount");
    assert_eq!(sections.len() as u64, page_count.min(50), "{v}");
    for (i, s) in sections.iter().enumerate() {
        assert_eq!(
            s["page"].as_u64(),
            Some(i as u64),
            "폴백 청크는 쪽 번호가 곧 주소: {v}"
        );
    }
    assert_eq!(v["nextStep"], SECTIONS_NEXT_STEP, "{v}");
}

// ── ② pages 모드: 범위 지정 발췌 + 남은 범위 안내 ──────────────────────────

#[test]
fn digest_pages_range_and_continuation() {
    let sample = sample_path(SAMPLE);
    let args = [
        "digest",
        "--pages",
        "1..2",
        "--json",
        sample.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["pages"]["from"], 1, "{v}");
    assert_eq!(v["pages"]["to"], 2, "{v}");
    assert!(!v["excerpt"].as_str().expect("excerpt").is_empty(), "{v}");
    // 남은 범위 안내: 같은 폭(2쪽)의 다음 창을 그대로 받아 적게 한다.
    assert_eq!(v["nextStep"], "이어서 digest --json --pages 3..4", "{v}");
}

#[test]
fn digest_pages_tail_clamps_and_finishes() {
    let sample = sample_path(SAMPLE); // 16쪽 → 마지막 쪽 15
    let args = [
        "digest",
        "--pages",
        "14..99",
        "--json",
        sample.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    assert_eq!(v["pages"]["from"], 14, "{v}");
    // 끝 범위는 마지막 쪽으로 조여진다 — 존재하지 않는 쪽을 성공으로 보고하지 않는다.
    assert_eq!(v["pages"]["to"], 15, "{v}");
    assert_eq!(v["nextStep"], PAGES_DONE_NEXT_STEP, "{v}");
}

#[test]
fn digest_pages_invalid_syntax_exit_usage() {
    let sample = sample_path(SAMPLE);
    for bad in ["코끼리", "3..", "..5", "5..1", "1-3"] {
        let args = ["digest", "--pages", bad, "--json", sample.to_str().unwrap()];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "--pages {bad} 는 사용법 오류여야 합니다.\n{}",
            describe(&args, &output)
        );
        assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
    }
}

#[test]
fn digest_pages_out_of_range_exit_runtime() {
    let sample = sample_path(SAMPLE);
    let args = [
        "digest",
        "--pages",
        "99..120",
        "--json",
        sample.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

#[test]
fn digest_sections_and_pages_are_mutually_exclusive() {
    let sample = sample_path(SAMPLE);
    let args = [
        "digest",
        "--sections",
        "--pages",
        "0..1",
        "--json",
        sample.to_str().unwrap(),
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

// ── ③ v1 무회귀 + 표면 등재 ────────────────────────────────────────────────

#[test]
fn digest_default_mode_unchanged_v1() {
    let sample = sample_path(SAMPLE);
    let args = ["digest", "--json", sample.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_stdout_json(&args, &output);
    // 1호 봉투 무회귀: outline·excerpt·고정 nextStep 이 그대로 남는다.
    assert!(v["outline"].is_array(), "{v}");
    assert!(!v["excerpt"].as_str().expect("excerpt").is_empty(), "{v}");
    assert_eq!(v["nextStep"], V1_NEXT_STEP, "{v}");
    assert!(v.get("sections").is_none(), "{v}");
}

#[test]
fn digest_v2_options_registered_in_capabilities() {
    let cap = parse_stdout_json(&["capabilities"], &run(&["capabilities"]));
    let digest = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "digest")
        .cloned()
        .unwrap_or_else(|| panic!("capabilities 에 digest 누락: {cap}"));
    let flags = digest["flags"].as_array().expect("flags");
    for expected in ["--sections", "--pages"] {
        assert!(
            flags.iter().any(|f| f == expected),
            "digest flags 에 {expected} 누락: {digest}"
        );
    }
    assert!(
        digest["recordFields"]
            .as_array()
            .expect("recordFields")
            .iter()
            .any(|f| f == "sections"),
        "digest recordFields 에 sections 누락: {digest}"
    );
}

#[test]
fn digest_v2_options_registered_in_mcp_schema() {
    let mcp = parse_stdout_json(&["capabilities", "--mcp"], &run(&["capabilities", "--mcp"]));
    let digest = mcp["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_digest")
        .cloned()
        .unwrap_or_else(|| panic!("MCP 도구 hwp_digest 누락: {mcp}"));
    let props = &digest["inputSchema"]["properties"];
    assert!(props.get("sections").is_some(), "{digest}");
    assert!(props.get("pages").is_some(), "{digest}");
    // required 는 path 하나만 유지 — 옵션이 필수로 승격되면 안 된다.
    let required = digest["inputSchema"]["required"]
        .as_array()
        .expect("required");
    assert_eq!(required.len(), 1, "{digest}");
    assert_eq!(required[0], "path", "{digest}");
    // [#3633] 초소형 모델 컨텍스트 절약 계약(40자 이내 설명)은 v2 에서도 유지.
    let desc = digest["description"].as_str().expect("description");
    assert!(desc.chars().count() <= 40, "{desc}");
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
