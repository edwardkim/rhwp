//! [Issue #6063] HWPX 문단 중간 저장 vpos 되감김(68820→3340)을 흐름 드리프트
//! 필터가 버려 꼬리 줄이 본문 칸 밖에 그려지던 결함.
//!
//! `samples/issue1880_anchor_stack_sb_convert.hwpx` 5쪽 마지막 문단(pi=51, 제10조)
//! 은 저장 LINE_SEG 가 line0 vpos=68820(쪽 하단) → line1 vpos=3340(다음 쪽 머리)로
//! 되감긴다. `internal_vpos_page_break_line` 은 이 경계를 찾지만, HWPX 필터가
//! 흐름 앵커 불일치를 국소 cursor 로 오인해 버려 세 줄을 현재 쪽에 붙였다.
//! 수정 전 실측: line0/1/2 가 col_bottom=1028.0 을 20.3/51.5/82.7px 넘김.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue1880_anchor_stack_sb_convert.hwpx";
const PAGE: u32 = 4; // 5쪽 (0-based)

fn svg_text_concat(svg: &str) -> String {
    let mut out = String::new();
    for cap in svg.split("</text>") {
        if let Some(i) = cap.rfind('>') {
            out.push_str(&cap[i + 1..]);
        }
    }
    out
}

fn collect_body_text_lines(node: &RenderNode, lines: &mut Vec<(usize, usize, f64)>, in_body: bool) {
    let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    let skip = matches!(
        node.node_type,
        RenderNodeType::Header | RenderNodeType::Footer | RenderNodeType::TableCell(_)
    );
    if skip {
        return;
    }
    if in_body {
        if let RenderNodeType::TextLine(tl) = &node.node_type {
            if let (Some(pi), Some(li)) = (tl.para_index, tl.line_index) {
                lines.push((pi, li, node.bbox.y + node.bbox.height));
            }
        }
    }
    for child in &node.children {
        collect_body_text_lines(child, lines, in_body);
    }
}

#[test]
fn hwpx_midpara_vpos_rewind_tail_stays_inside_body() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert!(
        core.page_count() >= 6,
        "구역 본문이 이어지는 6쪽 이상이어야 한다"
    );

    let tree = core
        .build_page_render_tree(PAGE)
        .expect("page 5 render tree");
    fn find_body(n: &RenderNode) -> Option<f64> {
        if matches!(n.node_type, RenderNodeType::Body { .. }) {
            return Some(n.bbox.y + n.bbox.height);
        }
        n.children.iter().find_map(find_body)
    }
    let body_bottom = find_body(&tree.root).expect("Body node");

    let mut lines = Vec::new();
    collect_body_text_lines(&tree.root, &mut lines, false);
    let rewind_tail: Vec<_> = lines
        .iter()
        .filter(|(pi, li, _)| *pi == 51 && *li >= 1)
        .copied()
        .collect();
    assert!(
        rewind_tail.is_empty(),
        "5쪽에 pi=51 되감긴 꼬리(line>=1)가 남아 있다: {rewind_tail:?} body_bottom={body_bottom:.1}"
    );
}

#[test]
fn hwpx_midpara_vpos_rewind_moves_article_10_tail_to_next_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page5 = svg_text_concat(&core.render_page_svg_native(4).expect("page 5 svg"));
    let page6 = svg_text_concat(&core.render_page_svg_native(5).expect("page 6 svg"));

    assert!(
        page5.contains("제10조") || page5.contains("휴대용"),
        "5쪽에 제10조 첫 줄은 남아도 된다"
    );
    assert!(
        page6.contains("보고") || page6.contains("여야한다") || page6.contains("소각"),
        "되감긴 제10조 꼬리(분실·소각 보고)는 6쪽에 있어야 한다. p5={page5:?} p6={page6:?}"
    );
}
