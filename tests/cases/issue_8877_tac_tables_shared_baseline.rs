//! 관계형+고정형: 칸 안 글자 없는 줄의 TAC 표 둘이 공유 기준선에 앉는다(#8877).
//! 주석 갱신: 2026-11-10T09:07:16+09:00.
//! 2026-11-10 raw(96DPI, scripts/visual_sweep.py --silhouette-only, 2px): 1쪽 Native 100.00 /
//! fresh WASM 100.00, 전체 4쪽 두 경로 전쪽 최저 100.00(devel b5064fae1e Native 1쪽 84.96).
//! 근거: mydocs/pr/assets/issue_8877/{native,wasm,base-native}/36317264_silhouette.tsv.
//! 독립 기준: 한/글 2022(12.0.0.4547) Print PDF 1쪽 괘선, 96DPI.
//!   4×2 표 위 괘선 139.2px, 5×4 결재 표 위 괘선 156.5px(devel 은 둘 다 137.6px).
//! 설치 글꼴 없이 검사한다. 위치는 표 노드 상자이며 글꼴 메트릭에 의존하지 않는다.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const INPUT: &str = "samples/issue8877/36317264_approval.hwpx";

/// (행, 열, 상자) — 깊이 우선 순서.
fn tables(node: &RenderNode, out: &mut Vec<(u16, u16, f64, f64)>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        out.push((
            table.row_count,
            table.col_count,
            node.bbox.y,
            node.bbox.height,
        ));
    }
    for child in &node.children {
        tables(child, out);
    }
}

fn header_tables() -> ((f64, f64), (f64, f64)) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = std::fs::read(root.join(INPUT)).expect("#8877 표본");
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    assert_eq!(core.page_count(), 4, "한/글 Print PDF 와 같은 4쪽");
    let page = core.build_page_render_tree(0).unwrap();
    let mut found = Vec::new();
    tables(&page.root, &mut found);
    let pick = |rows: u16, cols: u16| {
        let hits: Vec<_> = found
            .iter()
            .filter(|t| t.0 == rows && t.1 == cols)
            .map(|t| (t.2, t.3))
            .collect();
        assert_eq!(hits.len(), 1, "1쪽의 {rows}×{cols} 표는 하나다");
        hits[0]
    };
    (pick(4, 2), pick(5, 4))
}

#[test]
fn shorter_tac_table_sits_on_shared_baseline_not_line_top() {
    let ((left_top, left_h), (right_top, right_h)) = header_tables();
    // 원본 선언: 4×2 표가 줄(lh = om 140 + 9856 + om 140)을 소유하고 5×4 표가 더 낮다.
    assert!(left_h > right_h + 10.0, "줄 소유자는 더 높은 4×2 표");
    // 위끝이 붙지 않는다 — 낮은 표는 기준선 쪽으로 내려간다.
    assert!(
        right_top > left_top + 10.0,
        "5×4 표 위끝 {right_top} 이 4×2 표 위끝 {left_top} 에 붙었다"
    );
    // 기준선에 앉은 두 상자: 낮은 표의 아래끝은 높은 표의 아래끝보다 위, 그러나 가깝다.
    let (left_bottom, right_bottom) = (left_top + left_h, right_top + right_h);
    assert!(
        right_bottom < left_bottom,
        "낮은 표가 높은 표 아래로 내려가지 않는다"
    );
    assert!(
        left_bottom - right_bottom < (left_h - right_h) * 0.5,
        "아래끝 차 {} 는 높이 차 {} 의 절반보다 작다(위끝 정렬이 아니다)",
        left_bottom - right_bottom,
        left_h - right_h
    );
}

#[test]
fn header_tac_tables_match_hancom_print_rules() {
    let ((left_top, _), (right_top, _)) = header_tables();
    assert!((left_top - 139.2).abs() <= 1.0, "4×2 표 위끝 {left_top}");
    assert!((right_top - 156.5).abs() <= 1.0, "5×4 표 위끝 {right_top}");
}
