//! [Issue #6192] 글 뒤 그림은 셀 상단 대신 빈 호스트 문단에 소속되어 인용문을 감싼다.
//!
//! 현재2쪽 발췌본과 독립 한컴 PDF의 Native/fresh WASM 전쪽 비교를 먼저 확인했다.
//! 원4쪽의 절대 y를 발췌본에 적용했던 검사는 제거하고, 여섯 STEP의 본문→호스트→
//! 인용문 순서와 그림의 인용문 포함관계를 검증한다. 실제 모양은 Visual Sweep 증적으로 남긴다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6192/cell_behind_text_para_anchor.hwpx";

#[test]
fn issue_6192_cell_overlay_picture_anchors_to_host_paragraph() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("문서 읽기")).expect("문서 열기");
    let page = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let mut cells = Vec::new();
    collect_step_cells(&page.root, &mut cells);
    assert_eq!(cells.len(), 6, "여섯 STEP의 인용문 셀이 보존되어야 한다");

    for cell in cells {
        let lines: Vec<_> = cell
            .children
            .iter()
            .filter_map(|node| match &node.node_type {
                RenderNodeType::TextLine(line) => Some((node, line.para_index)),
                _ => None,
            })
            .collect();
        let intro = lines
            .iter()
            .find(|(_, pi)| *pi == Some(0))
            .expect("설명 문단")
            .0;
        let host = lines
            .iter()
            .find(|(_, pi)| *pi == Some(1))
            .expect("빈 호스트 문단")
            .0;
        let quotes: Vec<_> = lines
            .iter()
            .filter(|(_, pi)| *pi == Some(2))
            .map(|(n, _)| *n)
            .collect();
        let image = cell
            .children
            .iter()
            .find(|n| {
                matches!(&n.node_type,
            RenderNodeType::Image(image) if image.para_index == Some(1))
            })
            .expect("호스트 소속 그림");
        let first = quotes.first().expect("인용문 첫 줄");
        let last = quotes.last().expect("인용문 마지막 줄");

        assert!(
            intro.bbox.y + intro.bbox.height <= host.bbox.y,
            "설명 뒤에 호스트가 놓여야 한다"
        );
        assert!(
            host.bbox.y <= image.bbox.y && image.bbox.y <= first.bbox.y,
            "그림은 호스트 뒤에서 시작해 인용문 첫 줄을 감싸야 한다"
        );
        assert!(
            last.bbox.y + last.bbox.height <= image.bbox.y + image.bbox.height,
            "그림은 인용문 마지막 줄까지 감싸야 한다"
        );
        assert!(
            cell.bbox.x <= image.bbox.x
                && image.bbox.x + image.bbox.width <= cell.bbox.x + cell.bbox.width,
            "그림은 같은 셀의 좌우 경계 안에 있어야 한다"
        );
        assert!(
            image.bbox.y + image.bbox.height <= cell.bbox.y + cell.bbox.height,
            "그림은 같은 셀의 아래 경계 안에 있어야 한다"
        );
    }
}

fn collect_step_cells<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TableCell(_))
        && node.children.iter().any(|n| {
            matches!(&n.node_type,
            RenderNodeType::TextLine(line) if line.para_index == Some(2))
        })
        && node.children.iter().any(|n| {
            matches!(&n.node_type,
            RenderNodeType::Image(image) if image.para_index == Some(1))
        })
    {
        out.push(node);
    }
    for child in &node.children {
        collect_step_cells(child, out);
    }
}
