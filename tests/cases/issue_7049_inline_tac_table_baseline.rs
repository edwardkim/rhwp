#![cfg(not(target_arch = "wasm32"))]

//! [Issue #7049] 한 글줄에 글자처럼 취급되는 중첩 표가 둘 이상이면 rhwp 가 **상단을
//! 맞춰** 놓는다. 한/글은 그것들을 글자처럼 **줄의 기준선에 앉혀** 하단을 높이차의
//! 0.15 배로 벌린다.
//!
//! 원인은 `paragraph_layout.rs` 의 `stored_lh_covers_om` 이 자기 주석(「한글이
//! `lh = h + om` 으로 저장한 **표 전용 줄**」)과 달리 `>=` 한쪽만 봐서, 밴드가 줄에
//! **들어가기만** 하면 발동한 것이다. 그러면 높이가 다른 표들이 전부 `y + om_top`
//! 이 된다. 그리고 그 else 분기(기준선 하단정렬)도 상자 하단을 `기준선 + om_bottom`
//! 에 두어, 기준선 아래 15% 규약을 놓쳤다.
//!
//! 한/글 2020 실측 하단차(px) — `pdf/**` 의 기준 PDF 에서 `PyMuPDF` `get_drawings`
//! 로 뽑았다(PDF pt × 96/72).
//!
//! ```text
//!   표본                        한/글    devel    이 수정
//!   issue2083_hide_fill_page    18.22   121.80    18.20
//!   issue2470/36382471_masked    6.23    41.60     6.20
//!   36384689_…종합보고서         9.43    62.80     9.50
//! ```
//!
//! 그리고 밴드 분기는 그 줄을 **독점한** 표에만 준다. 세로 위치가 바깥여백과
//! 무관하다는 것이 실측이기 때문이다 — `21_언어_기출` 1쪽은 여백 `283/283` 과
//! `0/0` 인 두 표를 한/글이 **같은 y** 에 놓는다(여백이 관여하면 3.8px 벌어져야
//! 한다). 줄을 독점한 표는 종전대로 밴드 분기를 쓴다.

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn page_tables(rel: &str, page: u32) -> Vec<(f64, f64, f64, f64)> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let core = DocumentCore::from_bytes(&std::fs::read(p).expect("표본 읽기")).expect("문서 로드");
    let tree = core.build_page_render_tree(page).expect("render tree");
    let mut out = Vec::new();
    fn walk(n: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
        if matches!(n.node_type, RenderNodeType::Table { .. }) {
            out.push((n.bbox.x, n.bbox.y, n.bbox.width, n.bbox.height));
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    walk(&tree.root, &mut out);
    out
}

/// 폭이 `[lo, hi]` 인 표들을 x 순으로.
fn by_width(rel: &str, page: u32, lo: f64, hi: f64) -> Vec<(f64, f64, f64, f64)> {
    let mut v: Vec<_> = page_tables(rel, page)
        .into_iter()
        .filter(|t| (lo..=hi).contains(&t.2))
        .collect();
    v.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    v
}

fn has_text(node: &RenderNode, needle: &str) -> bool {
    matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
        || node.children.iter().any(|child| has_text(child, needle))
}

fn nested_table_cell<'a>(
    node: &'a RenderNode,
    needle: &str,
) -> Option<(&'a RenderNode, &'a RenderNode)> {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        if let Some(table) = node.children.iter().find(|child| {
            matches!(child.node_type, RenderNodeType::Table { .. }) && has_text(child, needle)
        }) {
            return Some((node, table));
        }
    }
    node.children
        .iter()
        .find_map(|child| nested_table_cell(child, needle))
}

fn parent_of<'a>(node: &'a RenderNode, target: &RenderNode) -> Option<&'a RenderNode> {
    if node
        .children
        .iter()
        .any(|child| std::ptr::eq(child, target))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| parent_of(child, target))
}

fn text_top(node: &RenderNode, needle: &str) -> Option<f64> {
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle)) {
        return Some(node.bbox.y);
    }
    node.children
        .iter()
        .filter_map(|child| text_top(child, needle))
        .reduce(f64::min)
}

/// 같은 글줄의 TAC 표 둘은 하단이 높이차의 0.15 배만큼 벌어져야 한다.
///
/// 수정 전 `62.80px` (한/글 `9.43px`) — 두 표의 **상단**이 붙어 있었다.
#[test]
fn inline_tac_tables_sit_on_the_line_baseline() {
    let boxes = by_width(
        "samples/hwpx/opengov/36384689_결재문서본문_화재발생종합보고서(제2026-298호).hwpx",
        0,
        200.0,
        430.0,
    );
    assert_eq!(boxes.len(), 2, "같은 줄의 TAC 표 2개: {boxes:?}");

    let mut bottoms: Vec<f64> = boxes.iter().map(|t| t.1 + t.3).collect();
    bottoms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let gap = bottoms[1] - bottoms[0];
    assert!(
        (gap - 9.43).abs() <= 2.0,
        "하단 간격이 한/글 9.43px 와 맞아야 한다 — #7049 회귀          \
         (실측 {gap:.2}px, 상자 {boxes:?}; 수정 전 62.80px)"
    );
}

/// 같은 서명의 두 표본도 함께 닫힌다.
#[test]
fn sibling_samples_close_the_same_gap() {
    for (rel, lo, hi, hancom, before) in [
        (
            "samples/issue2083_hide_fill_page.hwpx",
            230.0,
            395.0,
            18.22,
            121.80,
        ),
        (
            "samples/issue2470/36382471_masked.hwpx",
            245.0,
            365.0,
            6.23,
            41.60,
        ),
    ] {
        let boxes = by_width(rel, 0, lo, hi);
        assert_eq!(boxes.len(), 2, "{rel}: TAC 표 2개 — {boxes:?}");
        let mut bottoms: Vec<f64> = boxes.iter().map(|t| t.1 + t.3).collect();
        bottoms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let gap = bottoms[1] - bottoms[0];
        assert!(
            (gap - hancom).abs() <= 2.0,
            "{rel}: 하단 간격이 한/글 {hancom:.2}px 와 맞아야 한다 — #7049 회귀          \
             (실측 {gap:.2}px; 수정 전 {before:.2}px)"
        );
    }
}

/// 여백이 극단적으로 다른 두 표도 같은 y 에 놓여야 한다.
///
/// `21_언어_기출` 1쪽 머리 표의 「성명」(`om 283/283`)·「수험번호」(`om 0/0`) 상자다.
/// 한/글은 둘을 `266.6~299.2` 로 **완전히 같게** 놓는다. 수정 전 rhwp 는 `258.50`
/// / `254.70` 으로 3.8px(= 283 HWPUNIT, 곧 앞 표의 `om_top`) 어긋나 있었다.
///
/// ⚠ 두 상자의 **절대** y 는 아직 한/글보다 약 8px 위다 — 줄이 칸 어디에 앉는지는
/// 별도 축(`#7008` 범위 외)이라 여기서는 **서로 같은지**만 본다.
#[test]
fn tables_with_unequal_outer_margins_share_one_y() {
    let boxes = by_width("samples/21_언어_기출_편집가능본.hwp", 0, 150.0, 300.0);
    assert_eq!(boxes.len(), 2, "머리 표 안 TAC 표 2개: {boxes:?}");
    let dy = (boxes[0].1 - boxes[1].1).abs();
    assert!(
        dy <= 0.5,
        "여백이 다른 두 표가 같은 y 에 놓여야 한다 — #7049 회귀                   (y 차 {dy:.2}px, 상자 {boxes:?}; 수정 전 3.80px)"
    );
}

/// `#3386`의 표 전용 줄은 표를 소유 글줄 안에 두고 뒤 본문을 보존한다.
#[test]
fn table_exclusive_line_keeps_the_band_anchor() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6542/156678235_mid_para_vpos_rewind.hwp");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("원본 읽기")).expect("문서 로드");
    let page = core.build_page_render_tree(4).expect("5쪽 render tree");
    let (cell, table) = nested_table_cell(&page.root, "시점별 사망보험금 유동화")
        .expect("비교 표가 5쪽의 상위 표 셀에 있어야 한다");
    let band = cell.children.iter().find(|child| {
        matches!(child.node_type, RenderNodeType::TextLine(_))
            && child.bbox.y <= table.bbox.y
            && child.bbox.y + child.bbox.height >= table.bbox.y + table.bbox.height
    });
    assert!(
        band.is_some(),
        "중첩 비교 표가 자신을 소유한 글줄 안에 있어야 한다: {:?}",
        table.bbox
    );
    assert!(
        has_text(cell, "예정이율"),
        "표 앞 설명도 같은 셀에 있어야 한다"
    );
    let outer = parent_of(&page.root, cell).expect("상위 표");
    let following = text_top(&page.root, "기존 종신보험").expect("표 뒤 본문");
    assert!(
        outer.bbox.y + outer.bbox.height < following,
        "상위 표 뒤 본문이 표에 겹치거나 앞쪽으로 이동했다"
    );
}
