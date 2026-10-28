#![cfg(not(target_arch = "wasm32"))]

//! [#6271] RowBreak 자리차지 표의 꼬리 줄 vpos snap 이 표 배치를 깨뜨리던 결함의
//! 회귀 시험.
//!
//! 표본(`issue-6271-rowbreak-float-tail-line.hwp`, 실문서에서 텍스트·이미지를 전부
//! 더미로 치환한 합성본)은 문단 하나에 자리차지(TopAndBottom)·vert=Para·RowBreak
//! 3행 표(선언 1041.6px, 본문 가용 1099.9px 이내)와 표 아래 꼬리 줄·글자취급 그림이
//! 함께 앵커돼 있고, host 문단의 단일 lineseg vpos 는 표 **아래** 꼬리 줄 위치다.
//!
//! 종전에는 그 vpos 로 표 배치 **전에** 흐름을 snap 해 페이지가 소진됐고
//! (whole-fit 실패 → 머리 행 조각만 1쪽, 본체는 2쪽), 꼬리 줄에 형제 예약 높이가
//! 이중 가산돼 tac 그림이 쪽 밖(y≈2113px > 단 하단 1115px)에 그려져 소실됐다.
//! 한글 기대: 1쪽 + 그림은 쪽 하단(y≈1064px).

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue-6271-rowbreak-float-tail-line.hwp";

fn image_ys(node: &RenderNode, out: &mut Vec<f64>) {
    if let RenderNodeType::Image(_) = &node.node_type {
        out.push(node.bbox.y);
    }
    for child in &node.children {
        image_ys(child, out);
    }
}

#[test]
fn rowbreak_float_with_tail_line_stays_on_one_page() {
    let bytes = std::fs::read(SAMPLE).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    assert_eq!(
        core.page_count(),
        1,
        "선언높이가 본문에 들어가는 RowBreak 자리차지 표는 1쪽에 통째로 배치돼야 한다"
    );
}

#[test]
fn tail_tac_picture_stays_inside_page_body() {
    let bytes = std::fs::read(SAMPLE).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let tree = core.build_page_render_tree(0).expect("render page 1");
    let mut ys = Vec::new();
    image_ys(&tree.root, &mut ys);
    assert!(!ys.is_empty(), "꼬리 tac 그림이 렌더 트리에 있어야 한다");
    for y in ys {
        assert!(
            y < 1115.0,
            "tac 그림은 본문 하단(1115px) 안에 있어야 한다 — 실측 y={y:.1}"
        );
    }
}
