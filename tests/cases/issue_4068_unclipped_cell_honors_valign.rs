//! [Issue #4068] **안 잘린 중첩 칸이 선언된 세로 정렬을 잃는다.**
//!
//! `table_layout.rs` 의 `effective_valign` 은 네 가지 잘림 조건에서 `Top` 으로 수렴한다.
//! 그중 `cell_clipped_by_parent_viewport` 는 호출자가 넘긴 `col_area` 로 잘림을
//! 판정하는데, 그 값은 직전 조각까지 품은 채 올 수 있다(그 자리 주석이 이미 적어 둔
//! 사실이다). 그래서 **페이지 안에 온전히 들어간** 칸까지 "잘렸다"고 오판했다.
//!
//! ```text
//!   hwpx_sample2 19쪽 · 중첩 표 1행×2열 · 두 칸 모두 선언 valign=Center
//!     셀 961.80..1063.40 · page bbox 0.00..1122.50   → 실제로는 안 잘린다
//!     그런데 parentvp=true 로 Top 강제
//! ```
//!
//! 결과는 칸 내용이 정렬 몫만큼 위로 붙는 것이다. 한/글 정본(engine 2020) PDF 와
//! 베이스라인끼리 직접 대면 이렇게 나온다 — 같은 쪽 첫 줄이 Δ+0.03px 로 떨어지므로
//! 이 계기 자체는 검증돼 있다.
//!
//! ```text
//!   글자 칸 베이스라인   수정 전 Δ +9.99px  →  수정 후 Δ +8.11px
//! ```
//!
//! 고친 것은 술어에 **실제 클립(page bbox)** 한 항을 더한 것뿐이다. 안 잘린 칸은
//! 잘림 수렴의 대상이 아니다.
//!
//! ⚠ 이 수정은 `#4068` 의 그림 dy 를 **움직이지 않는다.** 그림 칸은 내용 높이가
//! 안높이보다 커서 정렬 몫이 0 이다(아래 `the_cell_without_alignment_slack_stays_put`
//! 가 그것을 잠근다). 이 시험이 잠그는 것은 "칸 정렬 오판" 하나다.
//!
//! 아래 셋이 **양쪽을 잠근다** — 정렬을 못 받으면 ①이, 여유 없는 칸까지 밀면 ②가,
//! 진짜 잘림 보호가 무너지면 ③이 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::Section;
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table, VerticalAlign};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 중첩 표를 담은 정식 회귀 입력 — 그림 칸과 글자 칸이 한 행에 나란히 있다.
const NESTED_SAMPLE: &str = "samples/hwpx_sample2.hwp";
/// `#2007` 의 **진짜** 쪽-잘림 보호 문서. 이 수정에 영향받지 않아야 한다.
const CLIP_SAMPLE: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";

/// `walk` 은 자기 자신을 세므로 최상위 표의 칸이 1, 그 안의 중첩 표 칸이 2 다.
/// 중첩 단계는 문서마다 다르다 — `hwpx_sample2` 는 2, `42065` 는 3 이다.
const NESTED_DEPTH_SAMPLE2: usize = 2;
const NESTED_DEPTH_42065: usize = 3;

fn sample(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

// 선언 높이가 부모 셀의 남은 영역보다 큰 중첩 표의 제한 계약.
// 문단 기준 자리차지 표는 vertOffset을 따른다. 후속 문단이 있으면
// 그 오프셋을 부모의 흐름 높이에 중복 가산하지 않는 #6697 계약을 사용한다.
fn parent_clip_fixture(align: VerticalAlign, offset: u32) -> RenderNode {
    let line = LineSeg {
        line_height: 800,
        text_height: 800,
        baseline_distance: 680,
        segment_width: 18000,
        ..Default::default()
    };
    let mut nested = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            col_span: 1,
            row_span: 1,
            width: 18000,
            height: 10000,
            vertical_align: align,
            paragraphs: vec![Paragraph {
                text: "VISIBLE".to_owned(),
                char_count: 7,
                char_offsets: (0..=7).collect(),
                line_segs: vec![line.clone()],
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    };
    nested.common.width = 18000;
    nested.common.height = 10000;
    nested.common.text_wrap = TextWrap::TopAndBottom;
    nested.common.vert_rel_to = VertRelTo::Para;
    nested.common.vertical_offset = offset;
    nested.rebuild_grid();
    let mut outer = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            col_span: 1,
            row_span: 1,
            width: 30000,
            height: 12000,
            paragraphs: vec![
                Paragraph {
                    text: "ANCHOR".to_owned(),
                    char_count: 6,
                    char_offsets: (0..=6).collect(),
                    line_segs: vec![line],
                    controls: vec![Control::Table(Box::new(nested))],
                    ..Default::default()
                },
                Paragraph {
                    text: "TAIL".to_owned(),
                    char_count: 4,
                    char_offsets: (0..=4).collect(),
                    line_segs: vec![LineSeg {
                        vertical_pos: 10800,
                        line_height: 800,
                        text_height: 800,
                        baseline_distance: 680,
                        segment_width: 30000,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    outer.common.width = 30000;
    outer.common.height = 12000;
    outer.common.treat_as_char = true;
    outer.rebuild_grid();
    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59529,
        height: 84189,
        margin_left: 8504,
        margin_right: 8504,
        margin_top: 5668,
        margin_bottom: 4252,
        ..Default::default()
    };
    section.paragraphs.push(Paragraph {
        controls: vec![Control::Table(Box::new(outer))],
        ..Default::default()
    });
    // 기본 글자 모양/글꼴도 갖춘 문서에서 출발해야 TextLine이 생성된다.
    // 스타일이 비어 있는 Document::default()는 정렬 반례 입력이 아니다.
    let mut core = DocumentCore::new_empty();
    let mut doc = core.document().clone();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.sections = vec![section];
    core.set_document(doc);
    core.build_page_render_tree(0)
        .expect("부모 clip 반례 렌더")
        .root
}

fn outer_clip(node: &RenderNode) -> Option<(f64, f64)> {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        assert!(cell.clip, "paint가 실제 적용하는 부모 clip이어야 한다");
        return Some((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    node.children.iter().find_map(outer_clip)
}

#[test]
fn parent_viewport_trims_nested_cell_before_alignment() {
    let top_root = parent_clip_fixture(VerticalAlign::Top, 10000);
    let top = nested_cell_contents(&top_root, 2);
    assert_eq!(top.len(), 1);
    for align in [VerticalAlign::Center, VerticalAlign::Bottom] {
        let root = parent_clip_fixture(align, 10000);
        let (parent_top, parent_bottom) = outer_clip(&root).expect("부모 셀");
        let cells = nested_cell_contents(&root, 2);
        assert_eq!(cells.len(), 1);
        let cell = cells[0];
        assert!(cell.cell_y >= root.bbox.y);
        assert!(cell.cell_y + cell.cell_h <= root.bbox.y + root.bbox.height);
        assert!(cell.cell_y >= parent_top && cell.cell_y < parent_bottom);
        // 선언된 10000HU 전체가 아닌 실제 가시 높이를 정렬 전에 사용한다.
        // 부모 밖까지 펼쳐진 셀에 정렬 몫을 계산한다는 정적 반례는 성립하지 않는다.
        assert!(cell.cell_h < 10000.0 / 75.0 - 0.5);
        assert!(
            (cell.cell_y + cell.cell_h - parent_bottom).abs() < 0.01,
            "{align:?}: 셀 하단이 부모 viewport에 맞아야 한다: {cell:?}, parent={parent_top}..{parent_bottom}"
        );
        let line_height = 800.0 / 75.0;
        assert!(cell.first_line_y >= parent_top);
        assert!(
            cell.first_line_y + line_height <= parent_bottom + 0.01,
            "{align:?}: 첫 줄 전체가 부모 clip 안에 남아야 한다: {cell:?}"
        );
        let expected = (cell.cell_h - line_height)
            / if align == VerticalAlign::Center {
                2.0
            } else {
                1.0
            };
        assert!(
            (cell.offset() - top[0].offset() - expected).abs() < 0.01,
            "{align:?}: 선언 높이가 아닌 가시 높이로 정렬해야 한다: {cell:?}, expected={expected}"
        );
    }
}

#[test]
fn a_fully_parent_contained_cell_keeps_center_and_bottom_alignment() {
    let top_root = parent_clip_fixture(VerticalAlign::Top, 0);
    let top = nested_cell_contents(&top_root, 2);
    assert_eq!(top.len(), 1);
    let mut offsets = Vec::new();
    for align in [VerticalAlign::Center, VerticalAlign::Bottom] {
        let root = parent_clip_fixture(align, 0);
        let (parent_top, parent_bottom) = outer_clip(&root).expect("부모 셀");
        let cells = nested_cell_contents(&root, 2);
        assert_eq!(cells.len(), 1);
        let cell = cells[0];
        assert!(cell.cell_y >= parent_top && cell.cell_y + cell.cell_h <= parent_bottom);
        offsets.push(cell.offset() - top[0].offset());
    }
    assert!(
        offsets[0] > 1.0,
        "Center 정렬 여유가 있어야 한다: {offsets:?}"
    );
    assert!(
        (offsets[1] - 2.0 * offsets[0]).abs() < 0.01,
        "Bottom은 Center의 두 배 여유: {offsets:?}"
    );
}

/// 한 칸이 그린 첫 글줄의 상단과 그 칸의 상단.
#[derive(Debug, Clone, Copy)]
struct CellContent {
    cell_x: f64,
    cell_y: f64,
    cell_h: f64,
    first_line_y: f64,
}

impl CellContent {
    /// 칸 상단부터 첫 글줄까지 — 여백 + 세로 정렬 몫.
    fn offset(&self) -> f64 {
        self.first_line_y - self.cell_y
    }
}

/// `depth` 단계 이상 중첩된 칸들의 (칸 상자, 첫 글줄) 을 모은다.
fn nested_cell_contents(node: &RenderNode, min_depth: usize) -> Vec<CellContent> {
    fn first_line_y(node: &RenderNode) -> Option<f64> {
        let mut best: Option<f64> = None;
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            best = Some(node.bbox.y);
        }
        for child in &node.children {
            if let Some(y) = first_line_y(child) {
                best = Some(best.map_or(y, |b: f64| b.min(y)));
            }
        }
        best
    }

    fn walk(node: &RenderNode, depth: usize, min_depth: usize, out: &mut Vec<CellContent>) {
        let is_cell = matches!(node.node_type, RenderNodeType::TableCell(_));
        let depth = depth + usize::from(is_cell);
        if is_cell && depth >= min_depth {
            if let Some(y) = first_line_y(node) {
                out.push(CellContent {
                    cell_x: node.bbox.x,
                    cell_y: node.bbox.y,
                    cell_h: node.bbox.height,
                    first_line_y: y,
                });
            }
        }
        for child in &node.children {
            walk(child, depth, min_depth, out);
        }
    }

    let mut out = Vec::new();
    walk(node, 0, min_depth, &mut out);
    out.sort_by(|a, b| {
        (a.cell_y, a.cell_x)
            .partial_cmp(&(b.cell_y, b.cell_x))
            .unwrap()
    });
    out.dedup_by(|a, b| (a.cell_y, a.cell_x) == (b.cell_y, b.cell_x));
    out
}

fn page_tree(rel: &str, page_num: u32) -> (RenderNode, f64) {
    let bytes = std::fs::read(sample(rel)).expect("정식 회귀 sample 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core
        .build_page_render_tree(page_num)
        .expect("페이지 render tree");
    let page_bottom = tree.root.bbox.y + tree.root.bbox.height;
    (tree.root, page_bottom)
}

/// `hwpx_sample2` 19쪽(0-based 18)의 중첩 표 한 행 — 왼쪽 그림 칸, 오른쪽 글자 칸.
fn nested_row() -> (CellContent, CellContent, f64) {
    let (root, page_bottom) = page_tree(NESTED_SAMPLE, 18);
    let cells: Vec<CellContent> = nested_cell_contents(&root, NESTED_DEPTH_SAMPLE2)
        .into_iter()
        .filter(|c| (955.0..975.0).contains(&c.cell_y))
        .collect();
    assert_eq!(
        cells.len(),
        2,
        "이 시험의 전제는 중첩 표 한 행의 두 칸이다 — 형상이 바뀌면 전제가 깨진다: {cells:?}"
    );
    (cells[0], cells[1], page_bottom)
}

/// ① 안 잘린 칸은 선언된 `Center` 를 받는다.
///
/// 수정 전에는 두 칸이 **같은** 오프셋(0.90px = 여백만)이었다. 오른쪽 글자 칸은
/// 내용(96.00px)이 안높이(99.76px)보다 짧아 정렬 몫 1.90px 을 받아야 한다.
#[test]
fn an_unclipped_nested_cell_keeps_its_declared_center_alignment() {
    let (picture_cell, text_cell, page_bottom) = nested_row();

    // 전제: 두 칸 모두 페이지 안에 **온전히** 들어간다 — 이 수정이 보는 바로 그 조건.
    for cell in [picture_cell, text_cell] {
        assert!(
            cell.cell_y >= -0.5 && cell.cell_y + cell.cell_h <= page_bottom + 0.5,
            "전제 붕괴: 칸이 페이지 밖으로 나갔다 {cell:?} (page_bottom={page_bottom})"
        );
    }

    let slack = text_cell.offset() - picture_cell.offset();
    assert!(
        (1.0..=3.0).contains(&slack),
        "여유 있는 Center 칸이 정렬 몫을 받아야 한다 — 수정 전에는 0.00 이었다. \
         실측 {slack:.2}px (그림칸 {:.2} · 글자칸 {:.2})",
        picture_cell.offset(),
        text_cell.offset()
    );
}

/// ② 정렬 여유가 없는 칸은 **움직이지 않는다**.
///
/// 왼쪽 그림 칸은 내용(103.95px)이 안높이(99.76px)보다 커서 정렬 몫이 0 이다.
/// 이 수정이 모든 중첩 칸을 무조건 밀어내는 게 아님을 잠근다.
#[test]
fn the_cell_without_alignment_slack_stays_put() {
    let (picture_cell, _text_cell, _) = nested_row();
    assert!(
        picture_cell.offset() < 1.5,
        "여유 없는 칸은 여백만큼만 내려가야 한다 — 실측 {:.2}px",
        picture_cell.offset()
    );
}

/// ③ **반례** — `#2007` 의 쪽-잘림 보호 문서는 이 수정에 흔들리지 않는다.
///
/// `42065` 11쪽(0-based 10)의 중첩 칸 두 개는 수정 전후 **같은 자리**다(전/후 SVG
/// 비교에서 차이는 1e-13px 부동소수 잡음뿐이었다). 이 수정이 잘림 보호까지 걷어냈다면
/// 여기 오프셋이 움직인다.
#[test]
fn the_page_clip_protected_document_is_unaffected() {
    let (root, _) = page_tree(CLIP_SAMPLE, 10);
    let cells: Vec<CellContent> = nested_cell_contents(&root, NESTED_DEPTH_42065)
        .into_iter()
        .filter(|c| c.cell_h > 100.0)
        .collect();
    assert_eq!(
        cells.len(),
        2,
        "이 시험의 전제는 큰 중첩 칸 두 개다: {cells:?}"
    );

    let offsets: Vec<f64> = cells.iter().map(|c| c.offset()).collect();
    for (got, want) in offsets.iter().zip([1.88_f64, 3.76_f64]) {
        assert!(
            (got - want).abs() <= 0.5,
            "#2007 보호 문서의 칸 내용이 움직였다 — 기대 {want:.2}px, 실측 {got:.2}px \
             (전체 {offsets:?})"
        );
    }
}
