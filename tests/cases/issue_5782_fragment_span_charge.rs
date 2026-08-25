//! [#5782] 쪽 분할 표 조각이 셀 안 중첩 표보다 짧아 마지막 글줄과 아래 괘선이
//! clip 에 잘리던 회귀 가드.
//!
//! 2181727(방호장치 안전인증 고시, 별표 1의2) 7쪽 실측 — 수정 전: 조각 셀 clip
//! 하단 979.0 인데 셀 안 `<표 4> 서지시험 Ⅰ` 중첩 표 하단은 998.8 로 19.8px 가
//! 잘렸다(마지막 행 글줄 `크) 차동모드` + 아래 괘선 소실). 근인: 중첩 표 호스트
//! 유닛은 표 행 높이 분해값이라 호스트 문단 뒤 간격(특히 lh 미흡수 표: 표가 줄
//! 아래로 흐르고 다음 문단 vpos 가 그 공간을 증언)을 담지 않아, 쪽나눔 회계
//! (유닛 합 867.5)와 페인트(저장 vpos, 891.1)가 어긋난다. 수정: 리셋(쪽 경계)
//! 구간별로 표-호스트 구간의 유닛 합을 저장 스팬에 맞춘다. 수정 후 셀 clip
//! 하단 1002.6 이 중첩 표 하단(998.8)을 포섭하고 문서 쪽수(12)는 한글과 동일.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5782/2181727_press_guard_test_methods.hwp";

/// depth-1(바깥 표) 셀들과 depth-2(중첩 표) bbox 하단을 수집한다.
fn collect(node: &RenderNode, depth: usize, cells: &mut Vec<(f64, f64)>, nested: &mut Vec<f64>) {
    let next = if matches!(node.node_type, RenderNodeType::Table(_)) {
        depth + 1
    } else {
        depth
    };
    match &node.node_type {
        RenderNodeType::TableCell(_) if depth == 1 => {
            cells.push((node.bbox.y, node.bbox.y + node.bbox.height));
        }
        RenderNodeType::Table(_) if depth == 1 => {
            nested.push(node.bbox.y + node.bbox.height);
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, next, cells, nested);
    }
}

#[test]
fn issue_5782_fragment_clip_covers_nested_table_bottom() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5782 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5782 fixture");

    assert_eq!(
        core.page_count(),
        12,
        "2181727 은 12쪽 문서다 (한글과 동일)"
    );

    let page = core.build_page_render_tree(6).expect("render p7");
    let mut cells = Vec::new();
    let mut nested = Vec::new();
    collect(&page.root, 0, &mut cells, &mut nested);

    // p7 조각 셀(위쪽에서 시작하는 큰 셀)의 clip 하단.
    let cell_bottom = cells
        .iter()
        .filter(|(y, _)| *y < 200.0)
        .map(|(_, b)| *b)
        .fold(f64::MIN, f64::max);
    // 그 셀 안 마지막 중첩 표(`<표 4> 서지시험 Ⅰ`)의 하단.
    let nested_bottom = nested.iter().cloned().fold(f64::MIN, f64::max);
    assert!(
        nested_bottom > 900.0,
        "p7 마지막 중첩 표를 찾아야 함; got {nested_bottom}"
    );
    assert!(
        cell_bottom + 0.5 >= nested_bottom,
        "조각 셀 clip 하단({cell_bottom:.1})이 중첩 표 하단({nested_bottom:.1})을 \
         포섭해야 함 — 수정 전엔 19.8px 가 잘려 마지막 행 글줄과 아래 괘선이 소실됐다"
    );
}
