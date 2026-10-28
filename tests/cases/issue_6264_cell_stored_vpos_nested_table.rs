//! [Issue #6264] 셀 문단의 저장 `vertical_pos` 를 절대 앵커로 신뢰해 중첩 표가 셀 바닥
//! 13px 에 눌려 붕괴한다 — 세로 괘선 전부·본문 71줄 소실 (1977964 1쪽).
//!
//! 바깥 표 셀[3](`h=63435HU=845.8px`, `valign=Center`, 문단 24개)의 저장 사다리:
//!
//! ```text
//! p[22] vpos=17776HU(237.0px) lh=900     <- 마지막 텍스트
//! p[23] vpos=61756HU(823.4px) lh=900     <- 2x3 중첩 표 호스트 (표 선언 높이 합 569px)
//! ```
//!
//! 사다리는 호스트 문단의 **줄 높이만** 적고 그 문단이 품은 표를 기술하지 않는다.
//! `vpos` 를 앵커로 쓰면 표 상단이 976px 이 되어 셀 안쪽 바닥까지 13.4px 만 남고,
//! 569px 짜리 표가 14.8px 로 눌리며 1행이 통째로 빠진다.
//!
//! 한글은 이 `vpos` 를 쓰지 않고 앞 문단 뒤로 흘린다(실측 첫 가로 괘선 405.2px).
//! rhwp 의 자연 흐름 값도 이미 그 자리다(`DIAG5601B cp=23 para_y_in=403.8`) — 앵커만
//! 쓰지 않으면 제자리로 돌아온다.
//!
//! 잠금은 좌표 상수 대신 **불변식**을 건다 — 중첩 표는 자기가 놓인 칸 안에 담긴다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6264/1977964_env_satellite_report_form.hwp";
const PAGE: u32 = 0;
/// 중첩 표의 선언 높이 합(2행 x 3열) — 붕괴 전 최소 높이.
const NESTED_MIN_H: f64 = 500.0;

#[test]
fn issue_6264_nested_table_keeps_its_declared_height() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core
        .build_page_render_tree(PAGE)
        .expect("page 1 render tree");

    let mut tables = Vec::new();
    collect_tables(&page.root, 0, &mut tables);
    // 바깥 표 1 + 중첩 표 1 (+ 손자 표) — 최소 둘은 있어야 한다.
    assert!(
        tables.len() >= 2,
        "1쪽에 표가 둘 이상 있어야 한다: {}",
        tables.len()
    );

    // 바깥 표(depth 최소) 안에 놓인 중첩 표를 찾는다.
    let outer = tables.iter().min_by_key(|t| t.0).copied().expect("바깥 표");
    let nested = tables
        .iter()
        .filter(|t| t.0 > outer.0)
        .max_by(|a, b| a.3.partial_cmp(&b.3).unwrap())
        .copied()
        .expect("중첩 표");

    let (_, nested_y, _, nested_h) = nested;
    let (_, outer_y, _, outer_h) = outer;

    // 붕괴하지 않는다 — 종전에는 14.8px 로 눌렸다.
    assert!(
        nested_h >= NESTED_MIN_H,
        "중첩 표가 눌렸다: h={nested_h:.1}px (선언 합 약 569px)"
    );
    // 바깥 표 안에 담긴다 — 종전에는 y=981.3 으로 바닥에 붙었다.
    assert!(
        nested_y + nested_h <= outer_y + outer_h + 0.5,
        "중첩 표({nested_y:.1}..{:.1})가 바깥 표({outer_y:.1}..{:.1}) 밖으로 나간다",
        nested_y + nested_h,
        outer_y + outer_h,
    );
}

/// (depth, y, x, height) 목록.
fn collect_tables(node: &RenderNode, depth: usize, out: &mut Vec<(usize, f64, f64, f64)>) {
    let next = if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push((depth, node.bbox.y, node.bbox.x, node.bbox.height));
        depth + 1
    } else {
        depth
    };
    for child in &node.children {
        collect_tables(child, next, out);
    }
}
