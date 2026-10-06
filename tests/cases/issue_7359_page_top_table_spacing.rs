//! #7359: 저장 줄 정보를 쓰는 쪽 상단의 제목·캡션·표 소속과 순서를 보존한다.
//! 독립 한컴 PDF 14쪽과 Native/fresh WASM 비교에서 확인한 내용 관계를 검사한다.
//! 절대 화면 좌표와 허용 픽셀 간격을 기대값으로 고정하지 않는다.
//! 정확한 간격·그림·괘선 차이는 별도의 Visual Sweep에서 판독한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp";

fn find_para(node: &RenderNode, para_index: usize, table: bool) -> Option<&RenderNode> {
    let matches = match &node.node_type {
        RenderNodeType::Table(meta) if table => meta.para_index == Some(para_index),
        RenderNodeType::TextLine(meta) if !table => meta.para_index == Some(para_index),
        _ => false,
    };
    // 셀 내부의 구역 안 문단 번호를 본문 호스트 문단으로 오인하지 않는다.
    if !table && matches!(node.node_type, RenderNodeType::Table(_)) {
        return None;
    }
    matches.then_some(node).or_else(|| {
        node.children
            .iter()
            .find_map(|child| find_para(child, para_index, table))
    })
}

#[test]
fn page_top_stored_spacing_aligns_caption_and_table_with_pdf() {
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)).expect("정식 HWP5 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 문서 로드");
    let page = core.build_page_render_tree(13).expect("정본의 14쪽");
    let heading = find_para(&page.root, 133, false).expect("14쪽 제목 문단");
    let caption = find_para(&page.root, 134, false).expect("14쪽 표 호스트 캡션");
    let table = find_para(&page.root, 134, true).expect("캡션과 같은 문단이 소유한 표");
    let text: String = core
        .extract_page_text_native(13)
        .expect("14쪽 본문 추출")
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    for content in ["최종안제시및보고자료", "최종제안마크", "마크의미"] {
        assert_eq!(
            text.matches(content).count(),
            1,
            "14쪽 내용 보존: {content}"
        );
    }
    assert!(
        heading.bbox.y + heading.bbox.height <= caption.bbox.y,
        "제목 다음에 캡션이 오며 서로 겹치지 않아야 한다"
    );
    assert!(
        caption.bbox.y + caption.bbox.height <= table.bbox.y,
        "캡션이 자신이 설명하는 표 위에 있으며 서로 겹치지 않아야 한다"
    );
    let RenderNodeType::Table(meta) = &table.node_type else {
        unreachable!("표 노드로 선택했다");
    };
    assert_eq!((meta.row_count, meta.col_count), (3, 3), "정본 표의 행·열");
}
