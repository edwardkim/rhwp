//! [#7190] HWPX 구역 첫 문단의 저장 `textpos` 가 이미 HWP5 축이면 올려 보지 않는다.
//!
//! #5961 은 HWPX 구역 첫 문단의 `textpos` 가 `hp:secPr` 한 자리만큼 짧은 축이라 보고,
//! 읽는 쪽에서 `hwpx_axis_shift`(8)만큼 올린다. 그런데 같은 한/글 2020 이 쓴 파일 중에는
//! 그 자리까지 이미 센 `textpos` 도 있다. 종전에는 올린 값이 **문단 끝을 넘을 때만** 그
//! 증거로 받아들였고, 줄마다 따로 판정했다.
//!
//! 줄은 글자나 컨트롤 슬롯의 **경계**에서만 시작한다. 올린 값이 경계가 아니면(예: 그림
//! 컨트롤 8유닛의 한가운데) 그 문단의 `textpos` 는 이미 HWP5 축이다. 축은 문단 하나에
//! 하나이므로 판정도 문단 단위로 한다 — 줄마다 판정하면 한 문단에 두 축이 섞여 줄 시작이
//! 거꾸로 가기도 한다(36294034: `[0, 95, 93]`).
//!
//! 정답지 — 한/글 2020 MCP PDF(`samples/issue7190/*-2020.pdf`)의 줄 나눔:
//! - 3011411: `…자세한 내용은` / `상단 메뉴 "[그림]"버튼을 이용하십시오.`
//!   종전 rhwp 는 `상단 메뉴 "` 와 그림을 첫 줄에 붙여 그림이 용지 밖 834.7px 까지 나갔다.
//! - 36473713: `…급수를 받지` / `않고 있어 … 처리하고자 합` / `니다.`
//!   종전 rhwp 는 `…받지 않고 있어 수도` / `조례 … 합니다.` / (빈 줄).
//!
//! 코퍼스 HWPX 구역 첫 문단(보정폭>0, 2줄 이상) 537개 중 판정이 바뀌는 문단은 17개이고,
//! 그중 6개(위 둘 + 36446194·36294034·36376147·1790387 3구역)를 한/글 PDF 로 확인해 모두
//! 저장값 그대로의 줄 나눔과 일치했다. 올린 값만 경계인 문단은 0개다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const PICTURE: &str = "samples/issue7190/3011411_tac_picture_second_line.hwpx";
const LEADING_CONTROL: &str = "samples/issue7190/36473713_leading_control_lines.hwpx";

fn page(rel: &str) -> RenderNode {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {rel}: {e:?}"));
    core.build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render {rel}: {e:?}"))
        .root
}

/// 렌더 트리의 `TextLine` 마다 (bbox, 줄 글자, 줄에 든 그림 bbox 들).
fn text_lines(node: &RenderNode, out: &mut Vec<(BoundingBox, String, Vec<BoundingBox>)>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        let mut images = Vec::new();
        for child in &node.children {
            match &child.node_type {
                RenderNodeType::TextRun(run) => text.push_str(&run.text),
                RenderNodeType::Image(_) => images.push(child.bbox),
                _ => {}
            }
        }
        out.push((node.bbox, text, images));
    }
    for child in &node.children {
        text_lines(child, out);
    }
}

fn body_bbox(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(body_bbox)
}

#[test]
fn issue_7190_tac_picture_is_placed_on_the_stored_second_line() {
    let root = page(PICTURE);
    let body = body_bbox(&root).expect("Body 노드");
    let mut lines = Vec::new();
    text_lines(&root, &mut lines);

    let first = lines
        .iter()
        .find(|(_, t, _)| t.starts_with("[별표"))
        .expect("첫 줄");
    assert!(
        first.1.trim_end().ends_with("자세한 내용은") && first.2.is_empty(),
        "첫 줄은 `…자세한 내용은` 에서 끝나고 그림이 없어야 한다(한/글 PDF), got {:?} images={:?}",
        first.1,
        first.2
    );

    let second = lines
        .iter()
        .find(|(_, t, _)| t.starts_with("상단 메뉴"))
        .unwrap_or_else(|| panic!("`상단 메뉴` 로 시작하는 둘째 줄이 없다: {lines:?}"));
    assert_eq!(
        second.2.len(),
        1,
        "그림은 둘째 줄에 있어야 한다: {second:?}"
    );
    let image = second.2[0];
    assert!(
        image.y >= second.0.y - 0.5,
        "그림은 둘째 줄 안에 놓여야 한다: line={:?} image={image:?}",
        second.0
    );
    assert!(
        image.x + image.width <= body.x + body.width + 0.5,
        "그림 우변 {:.1} 이 본문 우변 {:.1} 을 넘었다",
        image.x + image.width,
        body.x + body.width
    );
}

#[test]
fn issue_7190_leading_control_paragraph_breaks_where_hancom_breaks() {
    let root = page(LEADING_CONTROL);
    let mut lines = Vec::new();
    text_lines(&root, &mut lines);
    let texts: Vec<&str> = lines.iter().map(|(_, t, _)| t.as_str()).collect();

    let at = texts
        .iter()
        .position(|t| t.starts_with("1. 관내"))
        .unwrap_or_else(|| panic!("`1. 관내` 줄이 없다: {texts:?}"));
    assert!(
        texts[at].trim_end().ends_with("급수를 받지"),
        "첫 줄은 `…급수를 받지` 에서 끝나야 한다(한/글 PDF), got {:?}",
        texts[at]
    );
    assert!(
        texts[at + 1].starts_with("않고 있어")
            && texts[at + 1].trim_end().ends_with("처리하고자 합"),
        "둘째 줄은 `않고 있어 … 처리하고자 합` 이어야 한다, got {:?}",
        texts[at + 1]
    );
    assert_eq!(
        texts[at + 2].trim_end(),
        "니다.",
        "셋째 줄은 `니다.` 여야 한다"
    );
}
