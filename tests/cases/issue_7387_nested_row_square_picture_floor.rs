//! 쪼개질 수 있는 1행 중첩 표의 조각 유닛이 옆 칸 Square 그림의 높이를 담는다.
//!
//! # 무엇이 깨져 있었나
//!
//! native HWP5 에서 1행 중첩 표를 품은 호스트 문단은 칸 글줄 단위 조각
//! (`nested_table_mixed_fragment_heights`)으로 나뉜다. 그 조각 합은 글 칸의 줄만 세어,
//! 옆 칸의 Square 그림(저장 줄이 담지 않는 쪼갤 수 없는 개체)이 정하는 행 높이보다
//! 작았다. 바깥 조각 상자가 그 합으로 정해져 중첩 표가 잘린 높이로 그려졌다.
//! 같은 문서의 HWPX 쌍둥이는 그 표를 한 유닛(행 높이 그대로)으로 소비해 맞다.
//!
//! # 독립 기대값 — 한/글 정본
//!
//! `pdf/hwpx_sample2-2020.pdf`(Hancom PDF 1.3.0.534) 19쪽, pi=182 칸의 p[30] 1×2 표:
//! 세로 괘선 965.8~1073.4 → **107.6px**. 그림 칸 = 그림 7689HU + 세로 오프셋 107HU +
//! 칸 상하 안 여백 141+141HU = 8078HU = 107.7px 와 맞다(표 선언 높이도 8078HU).
//!
//! ```text
//!   수정 전 101.6px (글 칸 6줄 + 안 여백 — 그림 칸이 표 밖 3.2px 로 삐져나옴)
//!   수정 후 107.7px
//! ```
//!
//! 이 쪽의 Visual Sweep(2px 관용 실루엣)은 94.69% → 97.29% 다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/hwpx_sample2.hwp";
/// 정본 19쪽 중첩 1×2 표의 세로 괘선 길이(96dpi px).
const ORACLE_NESTED_TABLE_PX: f64 = 107.6;

fn collect<'a>(node: &'a RenderNode, depth: usize, out: &mut Vec<(usize, &'a RenderNode)>) {
    out.push((depth, node));
    for child in &node.children {
        collect(child, depth + 1, out);
    }
}

#[test]
fn split_nested_row_keeps_sibling_square_picture_height() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc =
        DocumentCore::from_bytes(&std::fs::read(path).expect("공개 회귀 문서")).expect("문서 파싱");
    let tree = doc.build_page_render_tree(18).expect("19쪽");
    let mut nodes = Vec::new();
    collect(&tree.root, 0, &mut nodes);

    // 그림을 품은 표 중 가장 안쪽(가장 깊은) 표가 대상 중첩 표다.
    let image = nodes
        .iter()
        .find(|(_, n)| matches!(n.node_type, RenderNodeType::Image(_)))
        .map(|(_, n)| n.bbox)
        .expect("19쪽 그림");
    let nested = nodes
        .iter()
        .filter(|(_, n)| matches!(n.node_type, RenderNodeType::Table(_)))
        .filter(|(_, n)| n.bbox.y <= image.y + 0.5 && image.y <= n.bbox.y + n.bbox.height)
        .max_by_key(|(depth, _)| *depth)
        .map(|(_, n)| n.bbox)
        .expect("그림을 품은 중첩 표");

    assert!(
        (nested.height - ORACLE_NESTED_TABLE_PX).abs() <= 1.5,
        "19쪽 중첩 1×2 표 높이 {:.2}px 이 정본 {ORACLE_NESTED_TABLE_PX}px 에서 벗어났다. \
         글줄 조각 합만으로 조각 상자를 정하면 옆 칸 Square 그림이 정하는 행 높이를 잃는다.",
        nested.height
    );
    assert!(
        image.y + image.height <= nested.y + nested.height + 0.5,
        "그림 바닥 {:.2} 이 중첩 표 바닥 {:.2} 밖이다.",
        image.y + image.height,
        nested.y + nested.height
    );
}
