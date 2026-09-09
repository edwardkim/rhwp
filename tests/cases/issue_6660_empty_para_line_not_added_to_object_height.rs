//! [Issue #6660] 개체만 든 셀의 행이 한/글보다 커진다.
//!
//! 근인: 셀 높이 회계가 `text_height + non_inline_h` 인데, **글자가 하나도 없는
//! 문단**의 줄 높이까지 `text_height` 에 들어간다. 그 줄은 개체를 담는 자리이지
//! 개체 아래에 따로 놓이는 글줄이 아니므로 두 번 세는 셈이다.
//!
//! 실측(`exam_science.hwp` 4쪽 아래 표, r=4): 그림 57.6px 인 칸에서
//! `text=13.3 + nonInline=57.6 = 70.9` 로 재어 행이 65.9px 이 됐다.
//! 한/글은 61.0px — 그림 57.6 에 위아래 여백 1.88×2 를 더한 값이다.
//!
//! 같은 표 다섯 행 높이 (래스터로 괘선 픽셀 행을 찾아 실측):
//!   수정 전  20.5 / 23.5 / 24.0 / 23.0 / 66.0
//!   수정 후  21.5 / 24.5 / 25.0 / 25.0 / 61.0
//!   한/글    21.5 / 24.5 / 25.0 / 25.0 / 61.0
//!
//! 표가 놓이는 y 자체는 여전히 한/글보다 5.5px 아래다 — 그 축은 #6681 이다.
//! 그래서 이 시험은 절대 위치가 아니라 **행 높이**를 본다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/exam_science.hwp";
/// 4쪽 아래 표가 놓인 x 대역. 위쪽 표·본문과 겹치지 않는다.
const TABLE_X: std::ops::Range<f64> = 250.0..500.0;
/// 그 표가 놓인 y 대역. 표 시작이 5.5px 어긋나 있어(#6681) 넉넉히 잡는다.
const TABLE_Y: std::ops::Range<f64> = 900.0..1120.0;
/// 그림이 든 행의 높이. 한/글 61.0px, 그림 57.6px + 여백 1.88×2 = 61.4px.
/// 빈 문단 줄(13.3px)을 함께 세면 65.9px 로 벗어난다.
const PICTURE_ROW_HEIGHT: std::ops::RangeInclusive<f64> = 59.5..=62.5;

/// 렌더 트리에서 표 칸 노드의 (x, y, 높이) 를 모은다.
fn cell_boxes(node: &RenderNode, out: &mut Vec<(f64, f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        out.push((node.bbox.x, node.bbox.y, node.bbox.height));
    }
    for c in &node.children {
        cell_boxes(c, out);
    }
}

#[test]
fn issue_6660_object_only_cell_row_matches_hancom_height() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let tree = core.build_page_render_tree(3).expect("page 4 render tree");

    let mut boxes = Vec::new();
    cell_boxes(&tree.root, &mut boxes);
    let mut rows: Vec<(f64, f64)> = boxes
        .into_iter()
        .filter(|(x, y, _)| TABLE_X.contains(x) && TABLE_Y.contains(y))
        .map(|(_, y, h)| (y, h))
        .collect();
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite y"));
    assert!(
        !rows.is_empty(),
        "4쪽 {TABLE_X:?} × {TABLE_Y:?} 구간에서 셀을 찾아야 한다"
    );

    let tallest = rows
        .iter()
        .map(|(_, h)| *h)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        PICTURE_ROW_HEIGHT.contains(&tallest),
        "그림이 든 행의 높이가 {tallest:.1}px 이다 — 한/글 61.0px 기준 \
         {PICTURE_ROW_HEIGHT:?} 안이어야 한다. 글자 없는 문단의 줄 높이를 개체 \
         높이에 함께 세면 65.9px 로 커진다 (#6660). 행 목록: {rows:?}"
    );
}
