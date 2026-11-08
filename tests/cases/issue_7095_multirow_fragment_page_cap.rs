//! #7095: 다행 표 조각의 물리 프레임과 괘선 잉크를 구분한다.
//!
//! 독립 기준은 `pdf/issue7336/stored_frame_page_larger_rowbreak-2020.pdf`다.
//! 전체 7쪽의 Native/fresh WASM 최소 실루엣 일치율은 각각 96.62447%,
//! 96.08978%이며, 2·5쪽 경계와 기울임 보정 영역을 직접 대조했다.
//! 원본 용지·본문 여백·표 바깥 여백으로 계산한 상한은 셀의 물리 프레임에
//! 적용한다. 표 bbox에는 괘선 반두께가 포함되므로 같은 상한으로 고정하지 않는다.
//! 긴 조각의 포함과 짧은 조각 뒤 빈 공간을 함께 보호한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7336/stored_frame_page_larger_rowbreak.hwpx";

/// 본문 최상위 표(칸 안 중첩 표 제외)의 아래끝.
fn fragment_bounds(sample: &str, page_index: u32, para_index: usize) -> (f64, f64, f64) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let root = core
        .build_page_render_tree(page_index)
        .expect("render tree")
        .root;

    fn find<'a>(node: &'a RenderNode, para_index: usize, inside: bool) -> Option<&'a RenderNode> {
        let mut inside = inside;
        if let RenderNodeType::Table(t) = &node.node_type {
            if !inside && t.para_index == Some(para_index) {
                return Some(node);
            }
            inside = true;
        }
        node.children
            .iter()
            .find_map(|child| find(child, para_index, inside))
    }
    let table = find(&root, para_index, false).expect("대상 표 — 시험 설정");
    let section = &core.document().sections[0];
    let page = &section.section_def.page_def;
    let rhwp::model::control::Control::Table(source) = &section.paragraphs[para_index].controls[0]
    else {
        panic!("원본 표 — 시험 설정");
    };
    // 본문과 표 바깥 아래 여백은 원본 HU로 계산한다. PDF 실측 좌표를 동결하지 않는다.
    let cap_hu = f64::from(page.height)
        - f64::from(page.margin_bottom)
        - f64::from(page.margin_footer)
        - f64::from(source.outer_margin_bottom);
    fn last_visible_line(node: &RenderNode) -> Option<f64> {
        let own = (node.visible && matches!(node.node_type, RenderNodeType::TextLine(_)))
            .then_some(node.bbox.y + node.bbox.height);
        own.into_iter()
            .chain(node.children.iter().filter_map(last_visible_line))
            .max_by(f64::total_cmp)
    }
    // 셀 조각은 물리 프레임이다. 표 bbox는 아래 괘선의 잉크까지 포함하므로
    // 본문 예약 경계와 같은 값으로 비교하지 않는다.
    let frame_bottom = table
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .map(|node| node.bbox.y + node.bbox.height)
        .max_by(f64::total_cmp)
        .expect("직계 셀 조각");
    // 허용 잉크는 원본이 참조한 테두리 굵기에서 구한다. 현재 출력 크기를
    // 기대값으로 재인용하거나 문서별 좌표 공차를 늘리지 않는다.
    let source_half_stroke = std::iter::once(source.border_fill_id)
        .chain(source.cells.iter().map(|cell| cell.border_fill_id))
        .filter_map(|id| id.checked_sub(1))
        .filter_map(|id| core.document().doc_info.border_fills.get(usize::from(id)))
        .flat_map(|fill| &fill.borders)
        .filter(|line| line.line_type != rhwp::model::style::BorderLineType::None)
        .map(|line| {
            rhwp::model::style::BORDER_WIDTHS[usize::from(line.width)].0 * 96.0 / 25.4 / 2.0
        })
        .fold(0.0, f64::max);
    let painted_bottom = table.bbox.y + table.bbox.height;
    assert!(
        painted_bottom + 0.5 >= frame_bottom,
        "표 잉크 상자는 자기 셀 프레임을 포함한다"
    );
    assert!(
        painted_bottom <= frame_bottom + source_half_stroke + 0.5,
        "표 상자가 셀과 원본 괘선의 반두께 밖으로 성장함: frame={frame_bottom}, ink={painted_bottom}, stroke={source_half_stroke}"
    );
    (
        frame_bottom,
        cap_hu * 96.0 / 7200.0,
        last_visible_line(table).expect("조각의 가시 글줄"),
    )
}

/// 넘친 조각은 본문과 바깥 아래 여백의 경계 안에 있고 내용을 자르지 않는다.
#[test]
fn a_multirow_fragment_past_the_page_cap_is_folded_to_it() {
    for page_index in [1, 4] {
        let (bottom, cap, content_bottom) = fragment_bounds(SAMPLE, page_index, 3);
        assert!(
            bottom <= cap + 0.5,
            "{}쪽 조각 아래 {bottom}가 원본 본문/바깥 여백 경계 {cap}를 넘음",
            page_index + 1
        );
        assert!(
            content_bottom <= bottom + 0.5,
            "{}쪽 조각의 글줄 아래 {content_bottom}가 표 아래 {bottom}에서 잘림",
            page_index + 1
        );
    }
}

/// 짧은 조각은 자기 내용을 담고 쪽 하단의 빈 공간을 그대로 남긴다.
#[test]
fn a_fragment_that_ends_before_the_cap_is_left_alone() {
    for page_index in [2, 3] {
        let (bottom, cap, content_bottom) = fragment_bounds(SAMPLE, page_index, 3);
        assert!(
            bottom < cap - 0.5,
            "{}쪽 짧은 조각 아래 {bottom}를 원본 쪽 경계 {cap}까지 늘림",
            page_index + 1
        );
        assert!(
            content_bottom <= bottom + 0.5,
            "{}쪽 짧은 조각의 내용 아래 {content_bottom}가 표 아래 {bottom}에서 잘림",
            page_index + 1
        );
    }
}
