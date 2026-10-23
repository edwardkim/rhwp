//! 한 문단이 세 쪽에 걸칠 때 둘째 쪽(이어지는 조각)의 저장 쪽 reset — 자기 각주가 있는 경우.
//!
//! 입력은 `samples/continuation_reset/gen_continuation_reset.py` 로 만든 HWPX 를 한/글 2020 이
//! 다시 저장한 HWP 다(저장 줄·쪽 reset 은 한/글이 쓴 값). 긴 문단(67줄)의 23줄째와 59줄째(N4)·
//! 56줄째(N6)에 `vpos=0` reset 이 있고, 둘째 쪽 줄에 그 문단 자기 각주가 4·6개 있다. 한/글 2020
//! Print(`pdf/continuation_reset/*-2020.pdf`)는 셋째 쪽을 그 reset 줄부터 시작한다.
//!
//! 수정 전 rhwp 는 첫 reset(23줄)만 쪽 경계로 쓰고, 이어지는 쪽은 각주를 빼지 않은 본문 전체를
//! 예산으로 23–66줄을 한 쪽에 넣어 각주 영역과 겹쳤다. 각주 없는 같은 문단(N0)은 둘째 reset 이
//! 없어 바뀌지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse")
}

/// 긴 문단(pi=21) 조각들: (쪽, 시작 줄, 끝 줄).
fn fragments(core: &DocumentCore) -> Vec<(usize, u64, u64)> {
    let pages = core.dump_page_items_json(None);
    let mut out = Vec::new();
    for (page, value) in pages.as_array().unwrap().iter().enumerate() {
        for column in value["columns"].as_array().unwrap() {
            for item in column["items"].as_array().unwrap() {
                if item["paraIndex"].as_u64() == Some(21) && item["kind"] == "partialParagraph" {
                    out.push((
                        page,
                        item["startLine"].as_u64().unwrap(),
                        item["endLine"].as_u64().unwrap(),
                    ));
                }
            }
        }
    }
    out
}

fn footnote_area(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        return Some(node);
    }
    node.children.iter().find_map(footnote_area)
}

/// 각주 영역 밖 본문 글줄의 아래 끝 중 가장 큰 값.
fn lowest_body_line(node: &RenderNode) -> f64 {
    if matches!(node.node_type, RenderNodeType::FootnoteArea) {
        return f64::MIN;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        return node.bbox.y + node.bbox.height;
    }
    node.children
        .iter()
        .map(lowest_body_line)
        .fold(f64::MIN, f64::max)
}

#[test]
fn continuation_page_honors_next_stored_reset_with_own_notes() {
    for (sample, second_reset) in [
        ("samples/continuation_reset/cont_reset_F20_L70_N4.hwp", 59),
        ("samples/continuation_reset/cont_reset_F20_L70_N6.hwp", 56),
    ] {
        let core = core(sample);
        assert_eq!(
            fragments(&core),
            vec![(0, 0, 23), (1, 23, second_reset), (2, second_reset, 67)],
            "{sample}: 둘째 쪽은 둘째 저장 reset 앞에서 끝난다"
        );
        let tree = core.build_page_render_tree(1).expect("page 2");
        let notes = footnote_area(&tree.root).expect("둘째 쪽 각주 영역");
        assert!(
            lowest_body_line(&tree.root) <= notes.bbox.y + 0.5,
            "{sample}: 둘째 쪽 본문 글줄이 각주 영역(위 {:.1})과 겹치지 않는다",
            notes.bbox.y
        );
    }
}

#[test]
fn continuation_without_second_reset_is_unchanged() {
    let core = core("samples/continuation_reset/cont_reset_F20_L70_N0.hwp");
    assert_eq!(fragments(&core), vec![(0, 0, 23), (1, 23, 67)]);
}
