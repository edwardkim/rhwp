//! #8160: sample16 전용 회귀. 이전 등록: tests/digest_v2_contract.rs
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

/// 여러 쪽(16쪽)인 실제 HWP3 샘플 — pages 범위·v1 기본 봉투 검증 원천.
/// 실제 조문 구조를 가진 HWP3 샘플. 일반 문서의 독립 번호를 조문으로 과검출하지 않도록
/// #3715가 강화됐으므로, 구조 경로는 실제 `제N조` 표지가 있는 이 fixture로 검증한다.
const STRUCTURED_SAMPLE: &str = "samples/hwp3-sample16.hwp";

/// [#3633 후속] sections 모드 nextStep 고정 문자열 계약 — 구현과 문자 그대로 일치.
const SECTIONS_NEXT_STEP: &str = "절 원문은 export-text --json -p <쪽>, 찾으려면 search --json";

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
fn issue_8160_digest_sections_envelope_and_addresses() {
    let sample = sample_path(STRUCTURED_SAMPLE);
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
fn issue_8160_digest_sections_max_chars_caps_each_excerpt() {
    let sample = sample_path(STRUCTURED_SAMPLE);
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

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
