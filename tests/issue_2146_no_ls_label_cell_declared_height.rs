//! Issue #2146: 저장 LINE_SEG 부재(NO_LS) 라벨 셀 행높이 팽창 — 선언 셀높이 신뢰.
//!
//! samples/task2146/21761835_jeonjik_exemption_table.hwp (구미시 [별표 7],
//! 78행×5열 RowBreak 표)의 r0 헤더 행:
//! - c0 "|직렬": 사선(대각선) BF=13 셀, 2문단 전부 저장 LINE_SEG 부재.
//! - c1 "계급|직류": 2문단 전부 NO_LS + ParaShape 고정(Fixed) 줄간격 37.76px
//!   — 2문단 합 75.5px 가 선언 내부높이 48.6px 와 모순.
//!
//! composer 재합성이 문단당 37.76px 로 행높이를 79.3px 로 부풀렸다 — 한글
//! 실측 52.4px(= 저장 선언 3928HU 그대로, COM 오라클 대조) 대비 +26.9px.
//!
//! 수정(composer::no_ls_short_label_cell): 전 문단 NO_LS + 모두 1줄(폭 여유)
//! + 선언 내부높이 ≥ em 합인 셀 중, (a) 사선 셀(한글은 코너 라벨로 그려 일반
//!   텍스트 흐름 미적용) 또는 (b) 저장 Fixed 줄간격 합이 선언 내부높이와 모순
//!   하는 셀은 선언 셀높이를 신뢰한다 (#1763/#2097 선언 권위 계열). 사선 없는
//!   일반 라벨 셀은 제외 — 한글이 fresh 레이아웃으로 선언 이상 키우는 문서
//!   (#1891 76076 규제영향분석서)가 존재한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/task2146/21761835_jeonjik_exemption_table.hwp";

/// r0 헤더 행(사선 c0 + Fixed-모순 c1)의 렌더 셀 높이가 선언 3928HU=52.4px
/// 로 유지되어야 한다. 수정 전에는 NO_LS 재합성 팽창으로 79.3px.
#[test]
fn issue_2146_no_ls_label_cell_keeps_declared_row_height() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let data = fs::read(Path::new(repo_root).join(SAMPLE)).unwrap_or_else(|e| panic!("read: {e}"));
    let core = DocumentCore::from_bytes(&data).expect("load");
    let tree = core.build_page_render_tree(0).expect("render p1");

    fn find_r0_cells(n: &RenderNode, out: &mut Vec<(u16, f64)>) {
        if let RenderNodeType::TableCell(c) = &n.node_type {
            if c.row == 0 && c.row_span == 1 {
                out.push((c.col, n.bbox.height));
            }
        }
        for ch in &n.children {
            find_r0_cells(ch, out);
        }
    }
    let mut cells = Vec::new();
    find_r0_cells(&tree.root, &mut cells);
    assert!(!cells.is_empty(), "r0 셀 노드 부재 — 렌더 트리 구조 변경?");

    for (col, h) in &cells {
        assert!(
            (*h - 52.4).abs() <= 2.0,
            "#2146 회귀: r0 c{col} 렌더 높이 {h:.1}px ≠ 선언 52.4px(3928HU) — \
             NO_LS 라벨 셀 재합성 팽창(수정 전 79.3px) 재발. cells={cells:?}"
        );
    }
}
