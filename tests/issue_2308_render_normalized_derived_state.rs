//! Issue #2308 functional regression for nested-table derived geometry.
//!
//! Page-count pins do not catch a nested 1×1 table whose width normalization
//! drifts only the split height. The two continuation fragments are pinned after
//! direct comparison with the HWP 2024/Hancom PDF fixture: the second fragment
//! begins at the page's content top while retaining the stored table width.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

fn nested_one_by_one_tables(node: &RenderNode, table_depth: usize, out: &mut Vec<(f64, f64)>) {
    let next_depth = if let RenderNodeType::Table(table) = &node.node_type {
        if table_depth >= 1 && table.row_count == 1 && table.col_count == 1 {
            out.push((node.bbox.y, node.bbox.height));
        }
        table_depth + 1
    } else {
        table_depth
    };
    for child in &node.children {
        nested_one_by_one_tables(child, next_depth, out);
    }
}

fn find_table_with_owner_para(node: &RenderNode, para_index: usize) -> Option<&RenderNode> {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table) if table.para_index == Some(para_index)
    ) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_table_with_owner_para(child, para_index))
}

fn find_nested_single_cell_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table) if table.row_count == 1 && table.col_count == 1
    ) {
        return Some(node);
    }
    node.children.iter().find_map(find_nested_single_cell_table)
}

fn collect_visible_text_line_rights(node: &RenderNode, rights: &mut Vec<f64>) {
    if !node.visible {
        return;
    }
    if matches!(&node.node_type, RenderNodeType::TextLine(_)) {
        rights.push(node.bbox.x + node.bbox.width);
    }
    for child in &node.children {
        collect_visible_text_line_rights(child, rights);
    }
}

fn contains_text(node: &RenderNode, needle: &str) -> bool {
    matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
        || node
            .children
            .iter()
            .any(|child| contains_text(child, needle))
}

#[derive(Clone, Copy)]
struct ClipRect {
    x: f64,
    y: f64,
    right: f64,
    bottom: f64,
}

impl ClipRect {
    fn from_node(node: &RenderNode) -> Self {
        Self {
            x: node.bbox.x,
            y: node.bbox.y,
            right: node.bbox.x + node.bbox.width,
            bottom: node.bbox.y + node.bbox.height,
        }
    }

    fn intersect(self, other: Self) -> Option<Self> {
        let clipped = Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        };
        (clipped.right > clipped.x && clipped.bottom > clipped.y).then_some(clipped)
    }

    fn contains_node(self, node: &RenderNode) -> bool {
        node.bbox.x >= self.x - 0.01
            && node.bbox.y >= self.y - 0.01
            && node.bbox.x + node.bbox.width <= self.right + 0.01
            && node.bbox.y + node.bbox.height <= self.bottom + 0.01
    }

    fn intersects_node(self, node: &RenderNode) -> bool {
        self.intersect(Self::from_node(node)).is_some()
    }
}

fn text_run_is_fully_painted(node: &RenderNode, needle: &str, clip: Option<ClipRect>) -> bool {
    if !node.visible {
        return false;
    }
    let clip = match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.clip => {
            clip.and_then(|active| active.intersect(ClipRect::from_node(node)))
        }
        _ => clip,
    };
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
        && clip.is_some_and(|active| active.contains_node(node))
    {
        return true;
    }
    node.children
        .iter()
        .any(|child| text_run_is_fully_painted(child, needle, clip))
}

fn text_run_is_partially_painted(node: &RenderNode, needle: &str, clip: Option<ClipRect>) -> bool {
    if !node.visible {
        return false;
    }
    let clip = match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.clip => {
            clip.and_then(|active| active.intersect(ClipRect::from_node(node)))
        }
        _ => clip,
    };
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
        && clip.is_some_and(|active| active.intersects_node(node) && !active.contains_node(node))
    {
        return true;
    }
    node.children
        .iter()
        .any(|child| text_run_is_partially_painted(child, needle, clip))
}

#[test]
fn issue_2308_saved_nested_width_keeps_fragment_geometry() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");

    // p33's first fragment begins at the row-6 boundary in the HWP 2024 PDF;
    // its old 351.1px pin predated the empty RowBreak host flow correction and
    // incorrectly described a point inside the preceding row. p34's 1×1
    // rationale fragment retains the stored 426.9px continuation geometry.
    let expected = [(32, 400.4, 649.3), (33, 77.1, 426.9)];
    for (page, expected_y, expected_height) in expected {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|error| panic!("render page {}: {error}", page + 1));
        let mut fragments = Vec::new();
        nested_one_by_one_tables(&tree.root, 0, &mut fragments);
        assert!(
            fragments.iter().any(|(y, height)| {
                (y - expected_y).abs() <= 0.2 && (height - expected_height).abs() <= 0.2
            }),
            "page {} nested fragment must preserve PDF-aligned geometry \
             y={expected_y:.1} h={expected_height:.1}; got {fragments:?}",
            page + 1
        );
    }
}

/// 한컴 PDF p33의 마지막 "현황 추이(p.270)" 줄은 p33의 셀 안에 온전히 남고,
/// p34는 다음 문단에서 시작한다. 중첩 표 조각 유닛을 기본 inMargin 폭으로
/// 재조판하면 한 줄을 덜 측정해 이 경계가 각각 하단/상단 clip에 반쯤 걸린다.
#[test]
fn issue_2308_nested_fragment_cut_does_not_half_paint_boundary_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");

    let p33 = core.build_page_render_tree(32).expect("render HWP PDF p33");
    let p33_clip = Some(ClipRect::from_node(&p33.root));
    assert!(
        text_run_is_fully_painted(&p33.root, "현황 추이", p33_clip),
        "p33 must keep the final source line fully inside the nested-cell clip"
    );

    let p34 = core.build_page_render_tree(33).expect("render HWP PDF p34");
    let p34_clip = Some(ClipRect::from_node(&p34.root));
    assert!(
        !text_run_is_partially_painted(&p34.root, "현황 추이", p34_clip),
        "p34 must not retain a half-painted residue of the p33-owned source line"
    );
    assert!(
        !contains_text(&p34.root, "현황 추이"),
        "p34 must not fully repaint the p33-owned final source line"
    );
    assert!(
        text_run_is_fully_painted(&p34.root, "자율안전확인신고한", p34_clip),
        "p34 must begin with the next fully painted source paragraph"
    );
}

/// HWP 2024 PDF p34의 1×1 중첩 표는 `inMargin=(0,0,141,141)`이더라도
/// 저장된 셀 좌우 여백(510HU)을 유지한다. 이 예외를 놓치면 문단의 paint
/// viewport가 우측 테두리까지 확장되어 한컴 출력과 달리 글자가 선을 침범한다.
#[test]
fn issue_2308_nested_non_tac_table_keeps_saved_horizontal_cell_margin() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");
    let tree = core.build_page_render_tree(33).expect("render HWP PDF p34");

    let outer = find_table_with_owner_para(&tree.root, 325)
        .expect("p34 outer activity-cost table (pi=325)");
    let nested =
        find_nested_single_cell_table(outer).expect("p34 nested single-cell rationale table");
    assert!(
        (nested.bbox.width - 487.6).abs() <= 0.2,
        "p34 nested-table width={:.1}; HWP 2024 PDF retains saved 36,572HU (487.6px)",
        nested.bbox.width
    );
    let mut rights = Vec::new();
    collect_visible_text_line_rights(nested, &mut rights);
    let rightmost = rights.into_iter().fold(f64::NEG_INFINITY, f64::max);
    let border_right = nested.bbox.x + nested.bbox.width;
    assert!(
        rightmost <= border_right - 6.0,
        "p34 nested-table text paint reaches the right border: text_right={rightmost:.1}, \
         border_right={border_right:.1}; HWP PDF retains the saved 510HU cell margin"
    );
}

/// HWP 2024 PDF p34의 직접편익 표는 빈 host 문단 안에 1×1 블록 표를 둔다.
/// 일반 표에는 unit cut이 없는데도 빈 composed line을 이유로 host를 건너뛰면,
/// 표 테두리만 남고 `근거설명` 본문 전체가 사라진다.
#[test]
fn issue_2308_empty_host_paragraph_keeps_block_nested_table_content() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");
    let p34 = core.build_page_render_tree(33).expect("render HWP PDF p34");
    let p34_clip = Some(ClipRect::from_node(&p34.root));

    let outer = find_table_with_owner_para(&p34.root, 336)
        .expect("p34 direct-benefit outer table (pi=336)");
    let nested =
        find_nested_single_cell_table(outer).expect("p34 direct-benefit rationale nested table");
    assert!(
        text_run_is_fully_painted(nested, "분쇄기 등 회전기계", p34_clip),
        "p34 direct-benefit rationale must retain the block nested-table body"
    );
}

/// Native HWP5의 마지막 short RowBreak child는 p34처럼 일반 저장 cell margin을
/// 보존하는 구조가 아니다. 한컴 2024 PDF는 parent owner content box에서 첫 줄을
/// `… 등의 사고`까지 그리고, p82는 동일 문장을 재paint하지 않고 `를 예방…`으로
/// 이어 간다. p34의 우측 border 보호와 이 p81/p82 owner 계약을 함께 고정한다.
#[test]
fn issue_2308_short_rowbreak_child_uses_owner_content_box_only() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let bytes = fs::read(path).expect("read #2195 authority fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #2195 authority fixture");

    let p81 = core.build_page_render_tree(80).expect("render HWP PDF p81");
    assert!(
        contains_text(
            &p81.root,
            "구내운반차 안전조치를 통해 근로자와 부딪히는 등의 사고"
        ),
        "p81 must keep the PDF-owned short-child first line through `사고`"
    );

    let p82 = core.build_page_render_tree(81).expect("render HWP PDF p82");
    assert!(
        contains_text(&p82.root, "를 예방함으로써 산업재해 감소"),
        "p82 must begin the continuation after the p81-owned `사고`"
    );
    assert!(
        !contains_text(&p82.root, "고를 예방함으로써 산업재해 감소"),
        "p82 must not split the PDF-owned word `사고` across pages"
    );
}
