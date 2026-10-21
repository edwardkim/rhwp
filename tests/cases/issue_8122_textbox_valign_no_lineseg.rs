//! [#8122] 저장 줄 없는 글상자의 세로 가운데 정렬이 내용 높이를 0으로 보던 결함의 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! `layout_textbox_content` 는 세로 정렬에 쓸 내용 높이를 문단의 저장 LINE_SEG
//! (`vertical_pos + line_height`)로 셌다. 저장 줄이 없는 글상자 문단은 이 값이 없어 내용 높이가
//! 0 이 되고, 가운데 정렬이 안쪽 높이의 절반을 위 여백으로 써서 글이 상자 아래로 넘쳤다.
//!
//! # 기대값의 출처
//!
//! 재현물은 실문서(저장 LINE_SEG 없음)를 글자 치환·그림 교체·문단 삭제로 익명화한 것이다. 같은
//! 파일을 한컴 웹한글기안기로 인쇄한 PDF 에서 글상자 안 두 줄은 모두 상자 안에 있고, 내용이
//! 안쪽 높이를 채워 위아래 남는 공간이 같다.
//!
//! # 검사하는 것
//!
//! 글상자 안 줄이 모두 상자 안에 있는지, 세로 가운데 정렬로 위아래 남는 공간이 같은지.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8122/textbox_valign_no_lineseg.hwp";
/// 저장 줄 없는 두 문단을 담은 세로 가운데 정렬 글앞 글상자(0번 문단 컨트롤 2).
const HOST_PARA: usize = 0;
const TEXTBOX_CONTROL: usize = 2;

#[derive(Clone, Copy, Debug)]
struct Rect {
    y: f64,
    h: f64,
}

impl Rect {
    fn of(node: &RenderNode) -> Self {
        Self {
            y: node.bbox.y,
            h: node.bbox.height,
        }
    }
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn find_textbox_shape(node: &RenderNode) -> Option<&RenderNode> {
    if let RenderNodeType::Rectangle(rect) = &node.node_type {
        if rect.para_index == Some(HOST_PARA) && rect.control_index == Some(TEXTBOX_CONTROL) {
            return Some(node);
        }
    }
    node.children.iter().find_map(find_textbox_shape)
}

#[test]
fn no_lineseg_textbox_centers_its_lines_inside_the_box() {
    let doc = load();
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");

    let shape = find_textbox_shape(&tree.root).expect("글상자 사각형");
    let outer = Rect::of(shape);
    let inner_node = shape
        .children
        .iter()
        .find(|c| matches!(c.node_type, RenderNodeType::TextBox))
        .expect("글상자 안쪽 영역");
    let inner = Rect::of(inner_node);
    let lines: Vec<Rect> = inner_node
        .children
        .iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::TextLine(_)))
        .map(Rect::of)
        .collect();
    assert_eq!(lines.len(), 2, "글상자 문단 두 개가 한 줄씩: {lines:?}");

    for line in &lines {
        assert!(
            line.y + 0.5 >= outer.y && line.bottom() <= outer.bottom() + 0.5,
            "글상자 줄은 상자 안에 있어야 한다 — 내용 높이를 0으로 보면 가운데 정렬이 \
             안쪽 높이의 절반만큼 내려 상자 아래로 넘친다: 상자 {outer:?}, 줄 {line:?}"
        );
    }

    let top_gap = lines[0].y - inner.y;
    let bottom_gap = inner.bottom() - lines[lines.len() - 1].bottom();
    assert!(
        (top_gap - bottom_gap).abs() <= 1.0,
        "세로 가운데 정렬은 위아래 남는 공간이 같아야 한다: 위 {top_gap:.1}, 아래 {bottom_gap:.1}"
    );
}
