//! [#6044] 셀 안 중첩 박스 뒤 문단 간격이 부풀어 유의사항 상자 마지막 줄이
//! 하단 괘선에 잘린다.
//!
//! `156513948` 보도자료 2쪽 "자료 이용시 유의사항" 상자는 1×1 흐름의 바깥 표
//! 칸 안에 중첩 표(조사 개요 박스)를 두고, 그 뒤에 `□ 2022년 상반기 결과 공표
//! 시…` 문단이 이어진다. 한글 저장 LINE_SEG 는 다음 문단 top(vpos=41359HU)에
//! 이미 `spacing_before`(1000HU)를 넣어 두는데, 중첩 표 배치 뒤 `para_y` 를
//! 그 vpos 로 끌어올린 다음 `layout_composed_paragraph` 가 앞 간격을 한 번 더
//! 더해 +10pt 가 부푼다. 상자 높이는 고정이라 초과분이 마지막 줄
//! `총계와 일치하지 않을 수도 있음` 을 하단 괘선에 가로로 자른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6044/156513948.hwpx";

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn load_page2() -> rhwp::renderer::render_tree::PageRenderTree {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(path).expect("read #6044 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #6044 fixture");
    assert_eq!(core.page_count(), 32, "156513948 은 32쪽 문서다");
    core.build_page_render_tree(1).expect("render p2")
}

#[test]
fn issue_6044_last_line_stays_inside_notice_box() {
    let page = load_page2();
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);

    let last = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(r) if r.text.contains("총계와 일치하지") => Some(n.bbox),
            _ => None,
        })
        .expect("마지막 줄 '총계와 일치하지' 가 그려져야 한다");
    let last_bottom = last.y + last.height;

    let notice_cell = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TableCell(_) => Some(n.bbox),
            _ => None,
        })
        .filter(|b| {
            last.x >= b.x - 1.0
                && last.x <= b.x + b.width + 1.0
                && last.y >= b.y - 1.0
                && last.y <= b.y + b.height + 40.0
                && b.height > 400.0
        })
        .min_by(|a, b| a.height.partial_cmp(&b.height).unwrap())
        .expect("유의사항 본문 칸");
    let cell_bottom = notice_cell.y + notice_cell.height;

    assert!(
        last_bottom <= cell_bottom - 1.0,
        "마지막 줄이 상자 하단 괘선 안에 남아야 한다 (수정 전 글자 절반이 괘선에 잘림): \
         last_bottom={last_bottom:.1} cell_bottom={cell_bottom:.1}"
    );
}

#[test]
fn issue_6044_gap_after_nested_overview_box_is_not_inflated() {
    let page = load_page2();
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);

    // 런이 글자 모양으로 쪼개지므로 같은 y 의 텍스트를 이어 붙인다.
    let mut lines: Vec<(f64, String)> = Vec::new();
    for n in &nodes {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            if let Some((_, buf)) = lines.iter_mut().find(|(y, _)| (*y - n.bbox.y).abs() < 0.6) {
                buf.push_str(&r.text);
            } else {
                lines.push((n.bbox.y, r.text.clone()));
            }
        }
    }
    let follow_y = lines
        .iter()
        .find(|(_, s)| s.contains("상반기") && s.contains("공표"))
        .map(|(y, _)| *y)
        .expect("'□ 2022년 상반기 결과 공표' 문단");

    let nested_bottom = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table(_) => Some(n.bbox.y + n.bbox.height),
            _ => None,
        })
        .filter(|bottom| *bottom < follow_y && follow_y - *bottom < 80.0)
        .fold(f64::MIN, f64::max);
    assert!(
        nested_bottom.is_finite() && nested_bottom > 0.0,
        "개요 중첩 박스를 찾아야 한다"
    );

    let gap = follow_y - nested_bottom;
    // 저장 사다리: 표 끝 39835HU → 다음 줄 41359HU = 15.24pt ≈ 20.3px (96dpi).
    // 결함 시 spacing_before 1000HU(10pt≈13.3px)가 한 번 더 더해져 줄이 내려간다.
    // 표 bbox 는 바깥 여백을 빼기도 해서 픽셀 갭은 20px 보다 클 수 있다. 이중
    // 가산(+13px)만 막으면 된다.
    assert!(
        gap < 48.0,
        "중첩 박스 뒤 간격이 이중 spacing_before 없이 닫혀야 한다 (수정 전 +10pt): gap={gap:.1}px"
    );
    assert!(
        gap > 8.0,
        "중첩 박스와 다음 문단이 붙으면 안 된다: gap={gap:.1}px"
    );
}
