//! 관계형·고정형 — [#8926] TAC 표 줄 뒤 글줄의 저장 쪽 리셋은 그 줄을 다음 쪽 맨 위로 보낸다.
//! 주석 갱신: 2026-11-11 08:40 KST (새 파일)
//!
//! 입력: 공개 정책연구 활용결과 보고서 서식 `samples/issue8926/6540000-202500049-post-table-line-stored-page-reset.hwp`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue8926/6540000-202500049-post-table-line-stored-page-reset-2020.pdf`(2쪽).
//! 문단 0 하나: 줄 0 「<서식8>」(vpos 0) → 줄 1 TAC 표(9×4, vpos 988, lh 68438) → 줄 2 표 뒤
//! 글줄(vpos **0**). 줄 2 의 저장 vpos 가 쪽 리셋이라 한/글은 그 줄을 2쪽에 둔다. 수정 전 rhwp
//! 는 표가 문단 첫 줄이 아니라는 이유로 표 뒤 글줄의 쪽 넘김을 보지 않아, 그 줄을 1쪽 본문
//! 하단(933.6px) 밖 938.9px 에 남기고 1쪽으로 끝냈다.
//! - 고정형: 전체 쪽수 2(= PDF).
//! - 관계형: 「<서식8>」 줄과 표는 1쪽, 표 뒤 글줄은 그 다음 쪽에 한 번만 있다(순서·누락·중복).
//!
//! 측정(2026-11-11, 이 파일과 같은 커밋의 코드, base = upstream/devel e118de8aba,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 2쪽 / 2쪽, p1 100.00 · p2 100.00, 최저 100.00 (base: 1쪽, 쪽수 불일치로 측정 불가)
//! - fresh WASM: 2쪽 / 2쪽, p1 100.00 · p2 100.00, 최저 100.00
//! - 증거: `mydocs/pr/assets/issue_8926/`(native·wasm TSV, review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8926/6540000-202500049-post-table-line-stored-page-reset.hwp";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: usize = 2;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

/// (쪽 인덱스, 항목 종류, 제어 인덱스, 시작 줄, 끝 줄) — 문단 0 의 항목만.
fn para0_items() -> (
    usize,
    Vec<(usize, String, Option<u64>, Option<u64>, Option<u64>)>,
) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    assert!(path.exists(), "fixture 누락: {SAMPLE}");
    let output = Command::new(rhwp_bin())
        .arg("dump-pages")
        .arg(&path)
        .arg("--json")
        .output()
        .expect("rhwp dump-pages 실행");
    assert!(
        output.status.success(),
        "dump-pages 실패: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let doc: Value = serde_json::from_slice(&output.stdout).expect("dump-pages JSON");
    let pages = doc["pages"].as_array().expect("pages");
    let mut items = Vec::new();
    for page in pages {
        let page_index = page["pageIndex"].as_u64().expect("pageIndex") as usize;
        for column in page["columns"].as_array().expect("columns") {
            for item in column["items"].as_array().expect("items") {
                if item["paraIndex"].as_u64() != Some(0) {
                    continue;
                }
                items.push((
                    page_index,
                    item["kind"].as_str().expect("kind").to_string(),
                    item["controlIndex"].as_u64(),
                    item["startLine"].as_u64(),
                    item["endLine"].as_u64(),
                ));
            }
        }
    }
    (pages.len(), items)
}

#[test]
fn post_table_line_after_stored_page_reset_starts_the_next_page() {
    let (page_count, items) = para0_items();
    assert_eq!(
        page_count, PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다: {items:?}"
    );

    let table_pages: Vec<usize> = items
        .iter()
        .filter(|(_, kind, control, _, _)| kind == "table" && *control == Some(2))
        .map(|(page, ..)| *page)
        .collect();
    assert_eq!(table_pages, vec![0], "TAC 표는 1쪽에 한 번: {items:?}");

    let line_pages = |start: u64| -> Vec<usize> {
        items
            .iter()
            .filter(|(_, kind, _, s, e)| {
                kind == "partialParagraph"
                    && s.is_some_and(|s| s <= start)
                    && e.is_some_and(|e| e > start)
            })
            .map(|(page, ..)| *page)
            .collect()
    };
    assert_eq!(
        line_pages(0),
        vec![0],
        "표 앞 「<서식8>」 줄은 1쪽에 한 번: {items:?}"
    );
    assert_eq!(
        line_pages(2),
        vec![table_pages[0] + 1],
        "표 뒤 글줄(저장 vpos 0)은 표 다음 쪽에 한 번: {items:?}"
    );
}
