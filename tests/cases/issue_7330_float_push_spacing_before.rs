//! [#7330] 같은 문단의 문단 기준 자리차지 표가 첫 글줄을 표 아래로 밀었으면, 문단 앞 간격을
//! 표 아래에 **다시 더하지 않는다** — 줄 위치는 max(문단 상단 + 앞 간격, 표 아래)다.
//!
//! 공개 합성 입력 `samples/float_push_spacing_before/float_push_spacing_before.hwp`: 생성기
//! (`make_float_push_spacing_before.py`) 출력 HWPX 를 한/글 2020(hwp2024Convert MCP engine 2020)
//! 으로 HWP 저장한 것이라 저장 LineSeg 는 한/글이 쓴 값이다. 정본
//! `pdf/float_push_spacing_before/float_push_spacing_before-2020.pdf`.
//!
//! - pi=2: 앞 간격 2800HU 문단이 PARA 기준 TOP_AND_BOTTOM 표(3행, 6000HU, 바깥 아래 852HU)를
//!   품는다. 한/글 저장 vpos 10192 = 문단 상단 3200 + 바깥 위 140 + 6000 + 852 — 앞 간격이 없다.
//!   수정 전 rhwp 는 이 줄을 표 아래에서 앞 간격(37.3px)만큼 더 내렸고 뒤 본문이 함께 밀렸다.
//! - pi=4(대조): 같은 앞 간격, 표 없음 — 앞 간격이 그대로 붙어야 한다.
//! - pi=6(대조): 앞 간격 없는 host 의 같은 표.
//!
//! 기대값은 구현값이 아니라 한/글 저장 vpos 의 간격이다. 절대 픽셀은 쓰지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/float_push_spacing_before/float_push_spacing_before.hwp";
const HU_PER_PX: f64 = 75.0;
/// 표 바깥 아래 여백(HU) — 생성기 값.
const OUTER_BOTTOM_HU: f64 = 852.0;

#[derive(Default)]
struct Page {
    /// 본문(셀 밖) 표의 (상단, 하단) — 문서 순서.
    tables: Vec<(f64, f64)>,
    /// (문단, y, 저장 vpos) — 본문 줄만.
    lines: Vec<(usize, f64, i32)>,
}

fn collect(node: &RenderNode, in_table: bool, page: &mut Page) {
    let mut in_table = in_table;
    match &node.node_type {
        RenderNodeType::Table(_) => {
            if !in_table {
                page.tables
                    .push((node.bbox.y, node.bbox.y + node.bbox.height));
            }
            in_table = true;
        }
        RenderNodeType::TextLine(tl) if !in_table => {
            if let (Some(para), Some(vpos)) = (tl.para_index, tl.vpos) {
                page.lines.push((para, node.bbox.y, vpos));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, in_table, page);
    }
}

fn page() -> Page {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    assert_eq!(core.page_count(), 1, "정본은 1쪽");
    let tree = core.build_page_render_tree(0).expect("render 1쪽");
    let mut page = Page::default();
    collect(&tree.root, false, &mut page);
    page.tables
        .sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite"));
    page
}

fn line(page: &Page, para: usize) -> (f64, i32) {
    page.lines
        .iter()
        .find(|(p, _, _)| *p == para)
        .map(|(_, y, vpos)| (*y, *vpos))
        .unwrap_or_else(|| panic!("pi={para} 본문 줄이 1쪽에 있어야 한다"))
}

#[test]
fn pushed_first_line_does_not_add_spacing_before_again() {
    let page = page();
    assert_eq!(page.tables.len(), 2, "본문 표 2개");
    let (_, table_bottom) = page.tables[0];
    let (y, _) = line(&page, 2);
    let gap = y - table_bottom;
    let expected = OUTER_BOTTOM_HU / HU_PER_PX;
    assert!(
        (gap - expected).abs() < 1.0,
        "pi=2 첫 글줄은 표 아래 바깥 여백 바로 밑이다(앞 간격 재가산 금지): \
         gap={gap:.1}px, expected={expected:.1}px"
    );
}

#[test]
fn every_body_line_keeps_hancom_stored_spacing() {
    // 한/글 저장 vpos 간격 = 정본 간격. pi=2 뒤 본문 전체가 앞 간격만큼 밀리지 않는다.
    let page = page();
    let (y0, v0) = line(&page, 0);
    for &(para, y, vpos) in &page.lines {
        let expected = y0 + f64::from(vpos - v0) / HU_PER_PX;
        assert!(
            (y - expected).abs() < 1.0,
            "pi={para}: y={y:.1}, 저장 vpos 기준 {expected:.1}"
        );
    }
}

#[test]
fn spacing_before_without_float_and_host_without_spacing_are_unchanged() {
    let page = page();
    // 대조 1: 표 없는 문단의 앞 간격은 그대로 붙는다.
    let (y3, v3) = line(&page, 3);
    let (y4, v4) = line(&page, 4);
    let stored = f64::from(v4 - v3) / HU_PER_PX;
    assert!(
        stored > 2800.0 / HU_PER_PX,
        "대조 문단의 저장 간격은 앞 간격을 포함한다: {stored:.1}px"
    );
    assert!(
        (y4 - y3 - stored).abs() < 1.0,
        "pi=4 앞 간격 유지: {:.1} vs 저장 {stored:.1}",
        y4 - y3
    );
    // 대조 2: 앞 간격 없는 host 도 표 아래 바깥 여백 바로 밑.
    let (_, table_bottom) = page.tables[1];
    let (y6, _) = line(&page, 6);
    let expected = OUTER_BOTTOM_HU / HU_PER_PX;
    assert!(
        (y6 - table_bottom - expected).abs() < 1.0,
        "pi=6: gap={:.1}, expected={expected:.1}",
        y6 - table_bottom
    );
}

// ── 단 맨 위 반례 ─────────────────────────────────────────────────────────────
//
// 공개 합성 입력 `samples/float_push_spacing_before/float_push_spacing_before_column_top.hwp`
// (생성기 `make_float_push_spacing_before_column_top.py` → 한/글 2020 HWP 저장, 정본
// `pdf/float_push_spacing_before/float_push_spacing_before_column_top-2020.pdf`). 대상 문단을
// pageBreak 로 쪽 맨 위에 둔다.
//
// - 2쪽 pi=2: 앞 간격 2800HU host + 자리차지 표. 한/글 저장 vpos 6992 는 앞 간격 없는 4쪽
//   host(pi=6)의 6992 와 같다 — 쪽 맨 위에서도 앞 간격을 표 아래에 더하지 않는다.
// - 3쪽 pi=4(대조): 같은 앞 간격, 표 없음. 저장 vpos 1400 — 쪽 나눔 뒤 맨 위 문단의 앞 간격은 남는다.

const COLUMN_TOP_SAMPLE: &str =
    "samples/float_push_spacing_before/float_push_spacing_before_column_top.hwp";
/// 표 바깥 위 여백(HU) — 생성기 값.
const OUTER_TOP_HU: f64 = 140.0;

fn column_top_pages() -> Vec<Page> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(COLUMN_TOP_SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {COLUMN_TOP_SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse fixture");
    assert_eq!(core.page_count(), 4, "정본은 4쪽");
    (0..4)
        .map(|i| {
            let tree = core
                .build_page_render_tree(i)
                .unwrap_or_else(|e| panic!("render {}쪽: {e:?}", i + 1));
            let mut page = Page::default();
            collect(&tree.root, false, &mut page);
            page
        })
        .collect()
}

#[test]
fn column_top_pushed_first_line_does_not_add_spacing_before_again() {
    let pages = column_top_pages();
    let (p2, p4) = (&pages[1], &pages[3]);
    assert_eq!(p2.tables.len(), 1, "2쪽 본문 표 1개");
    assert_eq!(p4.tables.len(), 1, "4쪽 본문 표 1개");
    let (y2, v2) = line(p2, 2);
    let (y6, v6) = line(p4, 6);
    assert_eq!(
        v2, v6,
        "입력 전제: 한/글은 두 host 첫 줄을 같은 vpos 에 저장한다"
    );
    let expected = OUTER_BOTTOM_HU / HU_PER_PX;
    let gap2 = y2 - p2.tables[0].1;
    assert!(
        (gap2 - expected).abs() < 1.0,
        "2쪽 맨 위 host 첫 글줄은 표 아래 바깥 여백 바로 밑이다(앞 간격 재가산 금지): \
         gap={gap2:.1}px, expected={expected:.1}px"
    );
    assert!(
        ((y2 - p2.tables[0].0) - (y6 - p4.tables[0].0)).abs() < 1.0,
        "앞 간격 유무와 무관하게 표 상단 대비 첫 글줄 위치가 같다: pi=2 {:.1} / pi=6 {:.1}",
        y2 - p2.tables[0].0,
        y6 - p4.tables[0].0
    );
}

#[test]
fn column_top_spacing_before_without_float_is_kept() {
    let pages = column_top_pages();
    let (p3, p4) = (&pages[2], &pages[3]);
    // 같은 쪽 설정이므로 4쪽 표 상단에서 바깥 위 여백을 뺀 값이 본문 상단이다.
    let body_top = p4.tables[0].0 - OUTER_TOP_HU / HU_PER_PX;
    let (y4, v4) = line(p3, 4);
    assert!(
        v4 > 0,
        "입력 전제: 한/글은 쪽 맨 위 대조 문단의 앞 간격을 저장 vpos 에 담는다: {v4}"
    );
    let expected = body_top + f64::from(v4) / HU_PER_PX;
    assert!(
        (y4 - expected).abs() < 1.0,
        "3쪽 맨 위 대조 문단은 앞 간격을 유지한다: y={y4:.1}, 저장 vpos 기준 {expected:.1}"
    );
}
