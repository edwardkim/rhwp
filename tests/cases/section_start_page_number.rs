//! 구역 정의의 시작 쪽 번호로 쪽 번호를 매긴다.
//!
//! 시험 문서는 한 구역 네 쪽이고, 구역 정의의 시작 쪽 번호가 10(`page_num` 10,
//! `page_num_type` 0)이며 꼬리말 가운데에 쪽 번호가 있다. 한컴 한글 12.30(Mac)
//! 인쇄 PDF(`pdf/section_start_page_number/`)는 꼬리말에 10, 11, 12, 13을 찍고 상태
//! 표시줄도 10/4쪽이다. 시작 번호를 읽지 않으면 쪽 정보와 꼬리말이 1부터 매겨진다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/section_start_page_number/start-number-10.hwp";

fn footer_text(node: &RenderNode, in_footer: bool, out: &mut String) {
    let in_footer = in_footer || matches!(node.node_type, RenderNodeType::Footer);
    if let (true, RenderNodeType::TextRun(run)) = (in_footer, &node.node_type) {
        // 쪽 번호 필드는 모델 한 글자를 표시 문자열로 바꿔 그린다.
        out.push_str(run.display_or_text());
    }
    for child in &node.children {
        footer_text(child, in_footer, out);
    }
}

fn page_number(json: &str) -> u32 {
    let start = json.find("\"pageNumber\":").expect("pageNumber 필드") + "\"pageNumber\":".len();
    json[start..]
        .split([',', '}'])
        .next()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or_else(|| panic!("pageNumber 파싱 실패: {json}"))
}

#[test]
fn section_start_page_number_numbers_pages_and_footer() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 4);

    for page in 0..4u32 {
        let expected = 10 + page;
        let info = core.get_page_info_native(page).expect("page info");
        assert_eq!(page_number(&info), expected, "{page}쪽 쪽 정보: {info}");

        let tree = core.build_page_render_tree(page).expect("render tree");
        let mut text = String::new();
        footer_text(&tree.root, false, &mut text);
        assert_eq!(text.trim(), expected.to_string(), "{page}쪽 꼬리말");
    }
}
