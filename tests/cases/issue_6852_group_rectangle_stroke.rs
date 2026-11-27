//! 관계형·고정형 회귀 — #7770·#7775 양돈 두 저장 형식의 제목 도형, 글줄 순서와 쪽수.
//! 주석 작성 2026-11-23 20:46 KST.
//! 2026-11-23 독립 한컴 2020 Print, 96 DPI, 2px raw 실루엣.
//! HWP와 HWPX 각각 전11쪽 Native/fresh WASM 최저 99.48%/99.49%, 5쪽은 양쪽 100.00%.
//! 근거는 `mydocs/pr/assets/issue7770/`의 전쪽 TSV와 5쪽 비교 PNG다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const INPUTS: [&str; 2] = [
    "samples/issue6797/156160455-social-pig-farm-income.hwp",
    "samples/hwpx/156160455-social-pig-farm-income.hwpx",
];

fn line_contains(node: &RenderNode, paragraph: usize, needle: &str) -> bool {
    if !matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(paragraph))
    {
        return false;
    }
    let text: String = node
        .children
        .iter()
        .filter_map(|item| match &item.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect();
    text.contains(needle)
}

#[test]
fn pig_farm_heading_keeps_ordered_stroked_rectangles_and_next_body() {
    for source in INPUTS {
        let document = DocumentCore::from_bytes(&std::fs::read(source).expect("원본 fixture"))
            .expect("양돈 문서");
        assert_eq!(document.page_count(), 11, "{source} 독립 Print의 쪽수");
        let page = document.build_page_render_tree(4).expect("5쪽 제목");
        let column = page
            .root
            .children
            .iter()
            .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
            .expect("본문")
            .children
            .iter()
            .find(|node| matches!(node.node_type, RenderNodeType::Column(_)))
            .expect("본문 단");
        let mut markers = column.children.iter().filter(|node| {
            matches!(&node.node_type, RenderNodeType::Group(group)
                if group.para_index == Some(48) && group.control_index == Some(0))
        });
        let marker = markers.next().expect("제목 문단의 묶음");
        assert!(
            markers.next().is_none(),
            "{source} 제목 도형을 한 번 그린다"
        );
        let [shadow, front] = marker.children.as_slice() else {
            panic!("{source} 앞뒤 두 사각형을 원본 순서로 그린다");
        };
        let (RenderNodeType::Rectangle(dark), RenderNodeType::Rectangle(light)) =
            (&shadow.node_type, &front.node_type)
        else {
            panic!("{source} 두 개체는 사각형이다");
        };
        assert_eq!(dark.style.fill_color, Some(0x282828));
        assert_eq!(light.style.fill_color, Some(0xffffff));
        for style in [&dark.style, &light.style] {
            assert_eq!(style.stroke_color, Some(0x000000));
            assert!(style.stroke_width > 0.0, "{source} 원본 실선이 남는다");
        }
        assert!(front.bbox.x < shadow.bbox.x && front.bbox.y < shadow.bbox.y);
        assert!(shadow.bbox.x < front.bbox.x + front.bbox.width);
        assert!(shadow.bbox.y < front.bbox.y + front.bbox.height);
        let heading = column
            .children
            .iter()
            .find(|node| line_contains(node, 48, "기존 선발 방식의 문제점"))
            .expect("도형과 같은 제목 문단");
        let next = column
            .children
            .iter()
            .find(|node| line_contains(node, 49, "돼지의 경우 아비 품종 개량"))
            .expect("제목의 뒤 본문");
        assert!(shadow.bbox.y + shadow.bbox.height <= next.bbox.y);
        assert!(heading.bbox.y + heading.bbox.height <= next.bbox.y);
    }
}
