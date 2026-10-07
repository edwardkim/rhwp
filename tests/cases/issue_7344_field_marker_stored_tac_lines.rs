#![cfg(not(target_arch = "wasm32"))]

//! [Issue #7344] 결재문서 `본문` 누름틀의 시작 마커가 글자처럼 취급하는 표 문단의 첫머리에
//! 놓이면, 그 문단은 저장 줄 계약(`composer::stored_tac_lines`)을 잃고 표를 저장 줄 원점이
//! 아닌 **앞 표 바로 아래**에 쌓는다.
//!
//! `36494836` 1쪽 `pi=4` 는 `[누름틀 시작, 1×1 표, 3×3 표]` 세 컨트롤과 저장 줄 둘을 가진다.
//!
//! ```text
//!   ls[0] vpos=19359 lh=1562 ls=660   ← 1×1 표(1282 + 바깥여백 140·140)
//!   ls[1] vpos=21581 lh=8733          ← 3×3 표(8453 + 바깥여백 140·140)
//! ```
//!
//! 두 표의 상단 간격은 저장 줄 원점 차 `2222HU = 29.6px` 이다. 수정 전 rhwp 는 1×1 표의
//! 높이(`17.1px`)만큼만 내려 3×3 표와 그 뒤 본문 전체를 12.5px 위로 올렸다. 한/글 2024 PDF
//! (MCP 변환)에서 3×3 표 첫 줄은 devel 보다 9.32pt 아래이고 이 수정 뒤 +0.08pt 차이다.
//!
//! 누름틀 시작 마커는 원시 스트림의 8유닛 칸만 차지하고 줄 상자를 만들지 않으므로, 줄
//! 소유 판정에서 구역·단 정의, 머리말/꼬리말과 같이 건너뛴다. 같은 문단에서 닫히는 필드는
//! `empty_control_stream_position` 가 계속 거른다(`field_ranges` 비어 있음 조건).

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7344/36494836_field_marker_tac_lines.hwpx";
const HU_PER_PX: f64 = 75.0;

fn load() -> DocumentCore {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&p).expect("표본 읽기")).expect("문서 로드")
}

/// 본문 최상위 표의 `(문단, 컨트롤) → 상단 y`.
fn table_tops(core: &DocumentCore, page: u32) -> Vec<(usize, usize, f64)> {
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut out = Vec::new();
    fn walk(n: &RenderNode, out: &mut Vec<(usize, usize, f64)>) {
        if let RenderNodeType::Table(t) = &n.node_type {
            if t.cell_context.is_none() {
                if let (Some(pi), Some(ci)) = (t.para_index, t.control_index) {
                    out.push((pi, ci, n.bbox.y));
                }
            }
            return;
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    walk(&tree.root, &mut out);
    out
}

fn top_of(tops: &[(usize, usize, f64)], pi: usize, ci: usize) -> f64 {
    tops.iter()
        .find(|t| t.0 == pi && t.1 == ci)
        .unwrap_or_else(|| panic!("pi={pi} ci={ci} 표가 1쪽에 없다: {tops:?}"))
        .2
}

#[test]
fn field_begin_marker_keeps_the_stored_tac_line_origins() {
    let core = load();
    assert_eq!(core.page_count(), 1, "한/글 2024 출력과 같은 1쪽");

    let para = &core.document().sections[0].paragraphs[4];
    assert!(
        matches!(
            para.controls.first(),
            Some(rhwp::model::control::Control::Field(_))
        ),
        "표본 전제: pi=4 첫 컨트롤이 누름틀 시작이다"
    );
    let segs = &para.line_segs;
    assert_eq!(segs.len(), 2, "표본 전제: 표마다 저장 줄 하나");
    let stored_gap = f64::from(segs[1].vertical_pos - segs[0].vertical_pos) / HU_PER_PX;

    let tops = table_tops(&core, 0);
    let small = top_of(&tops, 4, 1);
    let large = top_of(&tops, 4, 2);
    assert!(
        (large - small - stored_gap).abs() <= 0.5,
        "두 표의 상단 간격 {:.2}px 가 저장 줄 원점 차 {stored_gap:.2}px 와 다르다",
        large - small
    );

    // 다음 문단(pi=5)의 표도 저장 사다리의 간격을 유지해야 한다 — 표 문단의 전진량이
    // 마지막 줄의 저장 원점에서 이어진다는 뜻이다.
    let next = &core.document().sections[0].paragraphs[5];
    let stored_next = f64::from(next.line_segs[0].vertical_pos - segs[1].vertical_pos) / HU_PER_PX;
    let next_top = top_of(&tops, 5, 0);
    assert!(
        (next_top - large - stored_next).abs() <= 0.5,
        "pi=5 표 상단 간격 {:.2}px 가 저장 사다리 {stored_next:.2}px 와 다르다",
        next_top - large
    );
}

// ---------------------------------------------------------------------------
// 바뀐 쪽 Visual Sweep 에서 드러난 같은 결재문서 계열의 나머지 세 결함.
// 기대값은 저장값(탭 폭·표 선언 폭·문단 위 간격·셀 안 여백)과 한/글 2024 PDF 실측으로 정한다.
// ---------------------------------------------------------------------------

fn load_rel(rel: &str) -> DocumentCore {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    DocumentCore::from_bytes(&std::fs::read(&p).expect("표본 읽기")).expect("문서 로드")
}

fn collect<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(n);
    for c in &n.children {
        collect(c, out);
    }
}

fn hu(v: f64) -> f64 {
    v / HU_PER_PX
}

/// [#7344] 가운데 정렬 문단 `탭 4개 · 인라인 표 · 공백 6개` (36494836 1쪽 pi=2 결재란).
///
/// 탭은 원시 스트림 8유닛을 차지하는 가시 글자다. 종전에는 그 stride 를 컨트롤 갭으로 읽어
/// 탭마다 세그먼트가 갈렸고, 표가 **첫 탭 바로 뒤**에 놓여 한/글보다 186px 왼쪽에 그려졌다.
/// 한/글은 저장 탭 폭(`<hp:tab width=4000>`)으로 전진하고, 마지막 표 뒤 말미 공백은 정렬 폭에서
/// 뺀다(#6173 일반 문단 계약). 한/글 2024 PDF 의 표 왼쪽 ≈ 327.4px 와 이 식의 값이 맞는다.
#[test]
fn centered_inline_table_after_tabs_uses_stored_tab_widths() {
    let core = load();
    let para = &core.document().sections[0].paragraphs[2];
    let tabs = para.text.chars().take_while(|c| *c == '\t').count();
    assert_eq!(tabs, 4, "표본 전제: 탭 4개가 표 앞에 있다");
    let tab_px: f64 = para.tab_extended[..tabs]
        .iter()
        .map(|e| f64::from(e[0]) * 96.0 / 7200.0)
        .sum();
    let rhwp::model::control::Control::Table(t) = &para.controls[0] else {
        panic!("표본 전제: pi=2 첫 컨트롤이 표");
    };
    let outer = hu(f64::from(t.common.width)
        + f64::from(t.outer_margin_left)
        + f64::from(t.outer_margin_right));
    let om_left = hu(f64::from(t.outer_margin_left));

    let tree = core.build_page_render_tree(0).expect("render tree");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let col = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Column(_)))
        .expect("단");
    let table_x = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::Table(tn) if tn.para_index == Some(2) && tn.cell_context.is_none() => {
                Some(n.bbox.x)
            }
            _ => None,
        })
        .expect("pi=2 표");
    let expected = col.bbox.x + (col.bbox.width - tab_px - outer).max(0.0) / 2.0 + tab_px + om_left;
    assert!(
        (table_x - expected).abs() <= 1.0,
        "결재란 표 왼쪽 {table_x:.2}px 가 저장 탭 폭·말미 공백 제외 가운데 정렬값 {expected:.2}px 와 다르다"
    );
}

/// [#7344] 문단 앵커가 끊긴 셀 사다리(36312980 1쪽 `내용` 칸: p1·p2 vpos=0, p3 vpos=7040).
///
/// 배치는 끊긴 사다리를 순차 적층으로 그린다. 정렬 높이만 마지막 vpos(108.5px)를 써서 4줄
/// (85.1px)이 가운데보다 11.7px 위로 쏠렸다. 한/글 2024 PDF 는 위·아래 여백이 같다.
#[test]
fn broken_cell_ladder_centers_the_stacked_lines() {
    let core = load_rel("samples/issue7344/36312980_broken_cell_ladder_center.hwpx");
    let tree = core.build_page_render_tree(0).expect("render tree");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let line_children = |n: &RenderNode| -> usize {
        n.children
            .iter()
            .filter(|c| matches!(c.node_type, RenderNodeType::TextLine(_)))
            .count()
    };
    let cell = nodes
        .iter()
        .find(|n| {
            matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == 6 && c.col == 1)
                && line_children(n) == 4
        })
        .expect("4줄짜리 `내용` 칸 (행 6, 열 1)");
    let lines: Vec<_> = cell
        .children
        .iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::TextLine(_)))
        .collect();
    let top_gap = lines[0].bbox.y - cell.bbox.y;
    let last = lines.last().unwrap();
    let bottom_gap = cell.bbox.y + cell.bbox.height - (last.bbox.y + last.bbox.height);
    assert!(
        (top_gap - bottom_gap).abs() <= 1.0,
        "가운데 정렬 칸의 위 여백 {top_gap:.2}px 와 아래 여백 {bottom_gap:.2}px 가 다르다"
    );
}

/// [#7344] 저장 줄이 없는 셀 첫 문단의 위 간격(36455985 1쪽 `내용` 칸, 위 정렬).
///
/// 칸의 유일한 문단은 중첩 표만 들고 `linesegarray` 가 없다. 한/글은 다시 조판하며 문단 위
/// 간격(paraPr `prev=1200` HwpUnitChar = 16px)을 그대로 두어, 한/글 2024 PDF 에서 중첩 표 머리
/// 글자가 종전 rhwp 보다 13pt 아래에 있다. 저장 줄이 있는 칸은 그 값이 첫 줄 vpos 로 남는다(#6630).
#[test]
fn cell_first_paragraph_without_stored_lines_keeps_spacing_before() {
    let core = load_rel("samples/issue7344/36455985_no_lineseg_cell_spacing_before.hwpx");
    let tree = core.build_page_render_tree(0).expect("render tree");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let (cell_y, nested_y) = nodes
        .iter()
        .find_map(|n| {
            let RenderNodeType::TableCell(c) = &n.node_type else {
                return None;
            };
            if !(c.row == 2 && c.col == 1) {
                return None;
            }
            n.children
                .iter()
                .find(|ch| matches!(ch.node_type, RenderNodeType::Table(_)))
                .map(|t| (n.bbox.y, t.bbox.y))
        })
        .expect("중첩 표를 든 `내용` 칸 (행 2, 열 1)");
    let host = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => t.cells.iter().find(|cl| {
                cl.row == 2
                    && cl.col == 1
                    && cl.paragraphs.len() == 1
                    && cl.paragraphs[0]
                        .controls
                        .iter()
                        .any(|c| matches!(c, rhwp::model::control::Control::Table(_)))
            }),
            _ => None,
        })
        .expect("행 2 열 1 칸");
    let first = &host.paragraphs[0];
    assert!(
        first.line_segs.iter().all(|s| s.tag & 0x8000_0000 != 0),
        "표본 전제: 첫 문단에 저장 줄이 없다"
    );
    let pad_top = hu(f64::from(host.padding.top));
    let lead = nested_y - (cell_y + pad_top);
    let spacing_before = hu(1200.0);
    assert!(
        (lead - spacing_before).abs() <= 0.5,
        "중첩 표가 칸 안 여백 아래 {lead:.2}px 에 놓인다 — 문단 위 간격 {spacing_before:.2}px 여야 한다"
    );
}
