//! Issue #3403: 페이지 분할된 표 안 셀 매치가 표 시작 페이지로 보고되던 문제.
//!
//! `grep()` 은 `(구역, 문단) → 첫 페이지` 인덱스 하나로 모든 매치의 페이지를 채웠다. 표가
//! 쪽을 넘겨 이어지면 뒤쪽 행의 셀은 다음 쪽에 렌더되는데도 호스트 문단의 첫 쪽이 보고돼,
//! "그 쪽만 렌더해 근거 인용"하는 RAG 루프가 한 쪽 앞을 가리켰다.
//!
//! 재현 문서: `samples/2022년 국립국어원 업무계획.hwp` — `Ⅳ. 역점 추진과제` 표(para 586)가
//! 30–31쪽(0기준)에 걸친다.
//!
//! 정답지는 실제 렌더다. `export-svg` 로 뽑은 30쪽에는 "통합사전", 31쪽에는 "내실화"·
//! "수요조사"·"양성기관" 이 있음을 확인하고 이 계약을 고정했다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

const SAMPLE: &str = "samples/2022년 국립국어원 업무계획.hwp";
const HOST_PARAGRAPH: u64 = 586;

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn search_json(query: &str) -> serde_json::Value {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["search", sample().to_str().unwrap(), query, "--json"])
        .output()
        .expect("rhwp 실행 실패");
    assert!(
        out.status.success(),
        "search 실패: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("search JSON 파싱")
}

/// 분할 표 안 매치는 호스트 문단의 첫 쪽이 아니라 **그 행이 렌더되는 쪽**으로 보고돼야 한다.
#[test]
fn split_table_cell_matches_report_rendered_page() {
    let json = search_json("한국어");
    let matches = json["matches"].as_array().expect("matches 배열");
    let host: Vec<&serde_json::Value> = matches
        .iter()
        .filter(|m| m["paragraph"].as_u64() == Some(HOST_PARAGRAPH))
        .collect();
    assert!(
        !host.is_empty(),
        "재현 문서의 분할 표 문단 {HOST_PARAGRAPH} 매치가 없다"
    );

    let pages: std::collections::BTreeSet<u64> =
        host.iter().filter_map(|m| m["page"].as_u64()).collect();
    assert!(
        pages.len() > 1,
        "분할 표인데 매치가 한 쪽으로만 보고됐다: {:?}",
        pages
    );

    // 실제 렌더(export-svg) 기준: 30쪽=통합사전, 31쪽=내실화·수요조사·양성기관.
    for (needle, expected_page) in [
        ("통합사전 시스템", 30u64),
        ("내실화", 31),
        ("수요조사", 31),
        ("양성기관", 31),
    ] {
        let hit = host
            .iter()
            .find(|m| m["text"].as_str().is_some_and(|t| t.contains(needle)))
            .unwrap_or_else(|| panic!("{needle:?} 매치를 찾지 못했다"));
        assert_eq!(
            hit["page"].as_u64(),
            Some(expected_page),
            "{needle:?} 는 {expected_page}쪽에 렌더된다, got={}",
            hit["page"]
        );
    }
}

/// 페이지 보정이 매치 자체를 늘리거나 줄이지 않아야 한다(무회귀).
#[test]
fn page_attribution_does_not_change_match_count() {
    let json = search_json("한국어");
    assert_eq!(
        json["matchCount"].as_u64(),
        Some(123),
        "매치 수가 달라졌다 — 페이지 보정은 개수에 영향을 주지 않아야 한다"
    );
}
