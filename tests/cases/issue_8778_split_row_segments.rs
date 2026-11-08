//! 관계형·고정형 — [#8778] 나뉜 한 줄의 세그먼트, 단 시작 어울림 띠의 매칭, TAC 표 앞 저장 줄.
//! 주석 갱신: 2026-11-09 00:34 KST (새 파일)
//!
//! 입력 1: 공개 결재문서 36495398 의 축소본(둘째 구역 16번 이후 문단 삭제, 이 환경에 없는
//! HY중고딕·HY신명조 문단 제외). 기준: 같은 축소본의 한컴 engine 2020 Print PDF
//! `pdf/issue8778/36495398-split-row-segments-reduced-2020.pdf`(3쪽).
//! - 고정형: 전체 쪽수 3.
//! - 관계형 2쪽: 첫 두 문단의 두 세그먼트(같은 저장 vpos, tag bit 17·18)는 같은 줄 높이에 놓이고,
//!   사전 검토항목 표(`pi=2`)는 둘째 문단 아래·본문 안에 놓인다.
//! - 관계형 3쪽: 쪽 첫 문단 `pi=4` 의 제목 표는 누름틀 시작만 든 저장 줄 아래(저장 줄 차이 +
//!   바깥 위 여백 범위)에 놓인다. 저장 줄 없는 문단 `pi=12` 는 2쪽 그림의 어울림 띠에 매칭되지
//!   않아 전폭 한 줄이다.
//!
//! 입력 2(반례): 공개 보도자료 156739836 의 축소본(첫째·둘째 구역은 첫 문단만, 셋째 구역은 첫
//! 문단과 138~145번 문단, 그림 축소). 기준: `pdf/issue8778/156739836-float-prefix-row-reduced-2020.pdf`(4쪽).
//! - 관계형 4쪽: 앞 저장 줄을 자리차지 표(`pi=8 ci=0`)가 차지하는 TAC 표(`ci=1`)는 떠 있는 표
//!   아래에 놓이되 저장 표 줄 위치(저장 vpos + 바깥 위 여백)를 넘지 않는다.
//!
//! 측정(2026-11-09, head 745deec173, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, 문서 face 만 공급한 글꼴 디렉터리):
//! - 입력 1 Native: 3쪽 / 3쪽, p1 100.00 · p2 99.95 · p3 99.98, 최저 99.95 (base 는 4쪽으로 쪽수 불일치)
//! - 입력 1 fresh WASM: 3쪽 / 3쪽, p1 100.00 · p2 100.00 · p3 99.98, 최저 99.98
//! - 입력 2 Native: 4쪽 / 4쪽, p1 100.00 · p2 99.77 · p3 95.69 · p4 96.55, 최저 95.69
//! - 입력 2 fresh WASM: 4쪽 / 4쪽, p1 100.00 · p2 99.77 · p3 95.77 · p4 96.85, 최저 95.77
//! - 증거: `mydocs/pr/assets/issue_8778/`
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

const SAMPLE: &str = "samples/issue8778/36495398-split-row-segments-reduced.hwpx";

/// 단(`Column`) 바로 아래 본문 항목의 들여쓰기. 표 칸 안 문단(같은 `pi=` 번호)과 가른다.
const BODY_DEPTH: usize = 6;

#[derive(Debug, Clone)]
struct Extent {
    page: usize,
    depth: usize,
    kind: String,
    top: f64,
    bottom: f64,
    width: f64,
    rest: String,
}

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn number_after(line: &str, key: &str) -> Option<f64> {
    let start = line.find(key)? + key.len();
    line[start..].split_whitespace().next()?.parse().ok()
}

fn parse_extent(page: usize, line: &str) -> Option<Extent> {
    let trimmed = line.trim_start();
    let depth = line.len() - trimmed.len();
    let kind = trimmed.split_whitespace().next()?.to_string();
    let y_start = trimmed.find("y=")? + 2;
    let (top, after_top) = trimmed[y_start..].trim_start().split_once("..")?;
    let bottom = after_top.split_whitespace().next()?;
    Some(Extent {
        page,
        depth,
        kind,
        top: top.trim().parse().ok()?,
        bottom: bottom.trim().parse().ok()?,
        width: number_after(trimmed, "w=")?,
        rest: trimmed.to_string(),
    })
}

fn extents() -> (usize, Vec<Extent>) {
    extents_of(SAMPLE)
}

fn extents_of(sample: &str) -> (usize, Vec<Extent>) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
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

fn body_items<'a>(items: &'a [Extent], page: usize, kind: &str, tag: &str) -> Vec<&'a Extent> {
    items
        .iter()
        .filter(|e| {
            e.page == page && e.kind == kind && e.depth == BODY_DEPTH && e.rest.contains(tag)
        })
        .collect()
}

fn find<'a>(items: &'a [Extent], page: usize, kind: &str, tag: &str) -> &'a Extent {
    body_items(items, page, kind, tag)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{page}쪽 {kind} `{tag}` 없음"))
}

fn body_area(items: &[Extent], page: usize) -> &Extent {
    items
        .iter()
        .find(|e| e.page == page && e.kind == "Body")
        .unwrap_or_else(|| panic!("{page}쪽 Body 없음"))
}

#[test]
fn split_row_segments_keep_the_review_table_on_the_header_page() {
    let (pages, items) = extents();
    assert_eq!(pages, 3, "한컴 Print PDF 는 3쪽");

    for para in ["pi=0 ", "pi=1 "] {
        let left = find(&items, 2, "TextLine", &format!("{para}line=0 "));
        let right = find(&items, 2, "TextLine", &format!("{para}line=1 "));
        assert!(
            (left.top - right.top).abs() < 0.5,
            "같은 줄의 두 세그먼트는 같은 높이: {left:?} {right:?}"
        );
    }
    let second = find(&items, 2, "TextLine", "pi=1 line=1 ");
    let table = find(&items, 2, "Table", "pi=2 ci=0 ");
    let body = body_area(&items, 2);
    assert!(
        table.top >= second.bottom - 0.5,
        "표는 둘째 문단 아래에서 시작한다: {table:?} {second:?}"
    );
    assert!(
        table.bottom <= body.bottom + 0.5,
        "표는 2쪽 본문 안에 들어간다: {table:?} {body:?}"
    );
}

#[test]
fn field_only_stored_row_stays_above_the_title_table() {
    let (_, items) = extents();
    // 원본 저장값: `pi=4` 줄 0(누름틀 시작) vpos 70363, 표 줄 vpos 72523 → 2160HU = 28.8px.
    // 표 바깥 위 여백 283HU = 3.77px. 표는 쪽 첫 문단 원점에서 그만큼 아래다.
    let row_offset = 2160.0 * 96.0 / 7200.0;
    let margin_top = 283.0 * 96.0 / 7200.0;
    let body = body_area(&items, 3);
    let title = find(&items, 3, "Table", "pi=4 ci=1 ");
    let offset = title.top - body.top;
    assert!(
        offset >= row_offset - 0.5 && offset <= row_offset + margin_top + 0.5,
        "제목 표는 누름틀 저장 줄 아래에 놓인다: offset={offset:.1} {title:?} {body:?}"
    );
}

#[test]
fn full_width_paragraph_does_not_join_the_column_start_wrap_band() {
    let (_, items) = extents();
    let body = body_area(&items, 3);
    let lines = body_items(&items, 3, "TextLine", "pi=12 ");
    assert_eq!(lines.len(), 1, "pi=12 는 한 줄이다: {lines:?}");
    assert!(
        lines[0].width >= body.width - 0.5,
        "pi=12 는 전폭 줄이다: {:?} {body:?}",
        lines[0]
    );
}

const FLOAT_PREFIX_SAMPLE: &str = "samples/issue8778/156739836-float-prefix-row-reduced.hwpx";

#[test]
fn tac_table_after_a_float_prefix_row_does_not_pass_its_stored_row() {
    // 반례: 표 줄 앞 저장 줄이 자리차지(떠 있는) 표를 들고 있으면 그 줄의 자리는 떠 있는 표가
    // 정한다. 흐름 원점에 앞 줄 차이(2240HU)를 그대로 더하면 표가 저장 표 줄 위치를 지나친다.
    // 저장 첫 줄(vpos 40534)이 흐름과 맞는 쪽이므로 표 줄 저장 vpos 42774 + 바깥 위 여백 141HU
    // 가 상한이다.
    let (_, items) = extents_of(FLOAT_PREFIX_SAMPLE);
    let body = body_area(&items, 4);
    let float = find(&items, 4, "Table", "pi=8 ci=0 ");
    let table = find(&items, 4, "Table", "pi=8 ci=1 ");
    let stored_top = body.top + (42774.0 + 141.0) * 96.0 / 7200.0;
    assert!(
        table.top <= stored_top + 0.5,
        "표는 저장 표 줄 위치를 넘지 않는다: {table:?} stored_top={stored_top:.1}"
    );
    assert!(
        table.top >= float.bottom,
        "표는 앞 줄의 떠 있는 표 아래에 놓인다: {table:?} {float:?}"
    );
}
