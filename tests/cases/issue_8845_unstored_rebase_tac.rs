//! 관계형·고정형 — [#8845] 저장 줄 없는 문단 뒤 다시 세운 흐름 기준으로 TAC 표를 놓는다.
//! 주석 갱신: 2026-11-09 17:40 KST (새 파일)
//!
//! 입력: 공개 결재문서 36355707 원본. 기준: 한컴 engine 2020 Print PDF
//! `pdf/issue8845/36355707-unstored-rebase-tac-2020.pdf`(2쪽).
//! 가린 글 문단과 2×8 TAC 표 host(`pi=10`)에는 `linesegarray` 가 없고, 앞뒤 문단의 저장 vpos 는
//! 가리기 전 글로 조판한 값이다. 흐름 커서는 계산 줄 뒤에서 지연 기준을 다시 세우는데, 종전에는
//! TAC 표의 저장 줄 원점이 단 첫 표의 옛 기준을 먼저 골라 표만 25.6px 아래에 그렸다.
//! - 고정형: 전체 쪽수 2.
//! - 관계형 1쪽: 표(`pi=10 ci=0`) 위끝 = `(단위 : 천원)` 줄(`pi=9`) 위끝 + 저장 줄 높이·줄간격
//!   (1000 + 600 HU) + 표 바깥 위 여백(141 HU).
//! - 관계형 1쪽: 다음 문단 `pi=11` 위끝 = 표 아래끝 + 바깥 아래 여백(141 HU) + host 줄간격(720 HU).
//!
//! 측정(2026-11-09, 이 파일과 같은 커밋의 코드, base = upstream/devel 024d610633,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource`):
//! - Native: 2쪽 / 2쪽, p1 99.95 · p2 99.81, 최저 99.81 (base: p1 61.41 · p2 99.81)
//! - fresh WASM: 2쪽 / 2쪽, p1 99.93 · p2 99.97, 최저 99.93
//! - 증거: `mydocs/pr/assets/issue_8845/`(native·wasm·native_base TSV, review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8845/36355707-unstored-rebase-tac.hwpx";

/// HWPUNIT → px(96 DPI).
fn px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

/// 원본 저장값: `pi=9` 저장 줄 높이·줄간격, 표 바깥 위·아래 여백, 표 host 줄간격.
const UNIT_LINE_ADVANCE_HU: i32 = 1000 + 600;
const TABLE_OUTER_MARGIN_HU: i32 = 141;
const TABLE_HOST_LINE_SPACING_HU: i32 = 720;

#[derive(Debug, Clone)]
struct Extent {
    page: usize,
    kind: String,
    top: f64,
    bottom: f64,
    rest: String,
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn parse_extent(page: usize, line: &str) -> Option<Extent> {
    let trimmed = line.trim_start();
    let kind = trimmed.split_whitespace().next()?.to_string();
    let y_start = trimmed.find("y=")? + 2;
    let (top, after_top) = trimmed[y_start..].trim_start().split_once("..")?;
    let bottom = after_top.split_whitespace().next()?;
    Some(Extent {
        page,
        kind,
        top: top.trim().parse().ok()?,
        bottom: bottom.trim().parse().ok()?,
        rest: trimmed.to_string(),
    })
}

fn extents() -> (usize, Vec<Extent>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    assert!(path.is_file(), "fixture 누락: {}", path.display());
    let out = Command::new(rhwp_bin())
        .arg("dump-extents")
        .arg(&path)
        .output()
        .expect("dump-extents 실행");
    assert!(out.status.success(), "dump-extents 실패");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut page = 0usize;
    let mut pages = 0usize;
    let mut items = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("=== ") {
            page = rest
                .split('쪽')
                .next()
                .and_then(|n| n.trim().parse().ok())
                .expect("쪽 머리");
            pages = pages.max(page);
            continue;
        }
        if page > 0 {
            if let Some(extent) = parse_extent(page, line) {
                items.push(extent);
            }
        }
    }
    (pages, items)
}

fn find<'a>(items: &'a [Extent], page: usize, kind: &str, tag: &str) -> &'a Extent {
    items
        .iter()
        .find(|e| e.page == page && e.kind == kind && e.rest.contains(tag))
        .unwrap_or_else(|| panic!("{page}쪽 {kind} `{tag}` 없음"))
}

#[test]
fn tac_table_after_unstored_paragraph_follows_the_rebased_flow() {
    let (pages, items) = extents();
    assert_eq!(pages, 2, "한컴 Print PDF 는 2쪽");

    let unit = find(&items, 1, "TextLine", "pi=9 line=0 ");
    let table = find(&items, 1, "Table", "pi=10 ci=0 ");
    let expected = unit.top + px(UNIT_LINE_ADVANCE_HU) + px(TABLE_OUTER_MARGIN_HU);
    assert!(
        (table.top - expected).abs() <= 0.5,
        "표 위끝 {:.1} 은 단위 줄 위끝 {:.1} + 줄 전진 + 바깥 위 여백 = {:.1}",
        table.top,
        unit.top,
        expected
    );

    let next = find(&items, 1, "TextLine", "pi=11 line=0 ");
    let expected_next = table.bottom + px(TABLE_OUTER_MARGIN_HU) + px(TABLE_HOST_LINE_SPACING_HU);
    assert!(
        (next.top - expected_next).abs() <= 0.5,
        "pi=11 위끝 {:.1} 은 표 아래끝 {:.1} + 바깥 아래 여백 + 줄간격 = {:.1}",
        next.top,
        table.bottom,
        expected_next
    );
}
