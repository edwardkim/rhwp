//! [#8118] 저장 줄도 글자도 없는 문단의 글자처럼 취급 그림이 그려지지 않던 결함의 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! `compose_lines` 는 저장 LINE_SEG 도 글자도 없는 문단에 조합 줄을 만들지 않는다. 그래서
//! 문단 경로는 높이만 전진하는 빈 줄 폴백을 타고, 줄 안에서 TAC 그림을 그리지 않는다. 개체
//! 경로는 그 문단에 FullParagraph 항목이 있으면 문단 경로가 그렸다고 보고 그림을 건너뛰었다.
//! 두 가정이 맞물려 자리만 비고 그림이 빠졌다.
//!
//! # 기대값의 출처
//!
//! 재현물은 실문서(저장 LINE_SEG 없음)를 글자 치환·그림 교체로 익명화한 것이다. 같은 파일을
//! 한컴 웹한글기안기로 연 인쇄 출력에서 쪽 맨 위 빈 문단 자리에 제목 테두리 그림이 그려지고,
//! 다음 문단은 그 아래에서 시작한다.
//!
//! # 검사하는 것
//!
//! 그림이 한 번만 그려지는지, 그림이 자기 문단의 줄 상자 안에 있는지, 다음 문단이 그림 아래에서
//! 시작하는지.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8118/empty_para_tac_picture.hwp";
/// 저장 줄도 글자도 없이 TAC 그림(컨트롤 3)과 글앞 글상자(컨트롤 2)를 가진 첫 문단.
const HOST_PARA: usize = 0;
const PICTURE_CONTROL: usize = 3;
const NEXT_PARA: usize = 1;

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

fn host_pictures(node: &RenderNode, out: &mut Vec<Rect>) {
    if let RenderNodeType::Image(img) = &node.node_type {
        if img.para_index == Some(HOST_PARA) && img.control_index == Some(PICTURE_CONTROL) {
            out.push(Rect::of(node));
        }
    }
    for child in &node.children {
        host_pictures(child, out);
    }
}

/// 단 직속 글줄(글상자·표 안 글줄 제외) 중 `para` 문단의 것.
fn column_lines(node: &RenderNode, para: usize, out: &mut Vec<Rect>) {
    if matches!(node.node_type, RenderNodeType::Column(_)) {
        for child in &node.children {
            if let RenderNodeType::TextLine(tl) = &child.node_type {
                if tl.para_index == Some(para) {
                    out.push(Rect::of(child));
                }
            }
        }
        return;
    }
    for child in &node.children {
        column_lines(child, para, out);
    }
}

#[test]
fn empty_host_draws_its_tac_picture_once_inside_its_line() {
    let doc = load();
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");

    let mut pictures = Vec::new();
    host_pictures(&tree.root, &mut pictures);
    assert_eq!(
        pictures.len(),
        1,
        "빈 문단의 글자처럼 취급 그림은 정확히 한 번 그려져야 한다 — 문단 경로는 조합 줄이 없어 \
         그리지 않고, 개체 경로가 FullParagraph 를 보고 건너뛰면 빠진다: {pictures:?}"
    );
    let picture = pictures[0];

    let mut host_lines = Vec::new();
    column_lines(&tree.root, HOST_PARA, &mut host_lines);
    let host_line = host_lines.first().copied().expect("빈 문단 줄 상자");
    assert!(
        picture.y + 0.5 >= host_line.y && picture.bottom() <= host_line.bottom() + 0.5,
        "그림은 자기 문단의 줄 상자 안에 있어야 한다: 줄 {host_line:?}, 그림 {picture:?}"
    );

    let mut next_lines = Vec::new();
    column_lines(&tree.root, NEXT_PARA, &mut next_lines);
    let next_line = next_lines.first().copied().expect("다음 문단 첫 줄");
    assert!(
        next_line.y + 0.5 >= picture.bottom(),
        "다음 문단은 그림 아래에서 시작해야 한다: 그림 {picture:?}, 다음 문단 {next_line:?}"
    );
}
