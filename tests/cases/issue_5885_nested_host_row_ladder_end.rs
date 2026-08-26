//! [#5885] 중첩 표 호스트 문단이 셀 마지막일 때 그 문단 뒤 간격이 유닛 흐름에서
//! 유실돼, 행이 짧은 칸 기준으로 전진하고 바깥 행 구분선이 중첩 표 마지막 행
//! 한가운데를 가로지르던 회귀 가드.
//!
//! 3171199(설계업자 사업수행능력 세부평가기준, 별표 서식) 2쪽 실측 — 수정 전:
//! `⑵재정상태건실도` 행의 셀 두 개는 bottom 870.1, 중첩 표를 품은 칸만 876.4 로
//! 6.3px 더 길었고 다음 행이 870.1 에서 겹쳐 그려졌다. 한글 2022 실측은 행 바닥을
//! 882.4 하나로 닫고 중첩 표 하단(878.7)을 그 안에 둔다. 수정 후 rhwp 는 행 바닥
//! 879.7 로 세 칸을 동일하게 닫고 중첩 표 하단(876.0)이 행 안에 들어온다.
//!
//! 근인: 저장 사다리에서 문단 뒤 간격(9.6px)은 다음 문단 유닛의 corrected line
//! height 가 흡수하는데, 중첩 표 호스트 유닛은 표 행 높이 분해값이라 그 간격이
//! 없고, 호스트가 셀 마지막 문단이면 흡수할 다음 유닛도 없어 유닛 합이 저장
//! 종점보다 짧아진다 (521.7 vs 531.4).

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5885/3171199_design_capability_criteria.hwp";

/// (테이블 깊이, row, y, bottom) 목록 수집 — 깊이 1 = 바깥 표 셀, 2 = 중첩 표 셀.
fn collect_cells(node: &RenderNode, table_depth: usize, acc: &mut Vec<(usize, u16, f64, f64)>) {
    let next_depth = if matches!(node.node_type, RenderNodeType::Table(_)) {
        table_depth + 1
    } else {
        table_depth
    };
    if let RenderNodeType::TableCell(tc) = &node.node_type {
        acc.push((
            table_depth,
            tc.row,
            node.bbox.y,
            node.bbox.y + node.bbox.height,
        ));
    }
    for child in &node.children {
        collect_cells(child, next_depth, acc);
    }
}

#[test]
fn issue_5885_outer_row_closes_below_nested_table_bottom() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5885 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5885 fixture");

    assert_eq!(core.page_count(), 7, "3171199 은 7쪽 문서다 (한글과 동일)");

    let page = core.build_page_render_tree(1).expect("render p2");
    let mut cells = Vec::new();
    collect_cells(&page.root, 0, &mut cells);

    // 바깥 표 `⑵재정상태건실도` 행: y≈344.6 에서 시작하는 depth1 셀들.
    let row_cells: Vec<_> = cells
        .iter()
        .filter(|(d, _, y, _)| *d == 1 && (*y - 344.6).abs() < 3.0)
        .collect();
    assert!(
        row_cells.len() >= 3,
        "p2 재정상태건실도 행 셀 3개를 찾아야 함; got {}",
        row_cells.len()
    );
    let bottoms: Vec<f64> = row_cells.iter().map(|(_, _, _, b)| *b).collect();
    let min_b = bottoms.iter().cloned().fold(f64::MAX, f64::min);
    let max_b = bottoms.iter().cloned().fold(f64::MIN, f64::max);
    assert!(
        max_b - min_b <= 0.5,
        "같은 행의 셀 바닥은 하나로 닫혀야 함 (수정 전 870.1 vs 876.4); got {bottoms:?}"
    );

    // 그 행 안의 중첩 표(`구 분`/`점 수`) 마지막 행 하단은 바깥 행 바닥 안에 있어야 한다.
    let nested_bottom = cells
        .iter()
        .filter(|(d, _, y, _)| *d == 2 && *y > 800.0 && *y < 900.0)
        .map(|(_, _, _, b)| *b)
        .fold(f64::MIN, f64::max);
    assert!(
        nested_bottom > 800.0,
        "p2 중첩 표 셀을 찾아야 함; got {nested_bottom}"
    );
    assert!(
        nested_bottom <= min_b + 0.5,
        "중첩 표 하단({nested_bottom:.1})은 바깥 행 바닥({min_b:.1}) 안에 있어야 함 — \
         수정 전엔 행 구분선이 중첩 표 마지막 행을 가로질렀다"
    );

    // 다음 행(`라. 기술개발 및`)은 행 바닥에서 시작한다 — 겹침 없음.
    let next_row_top = cells
        .iter()
        .filter(|(d, _, y, _)| *d == 1 && *y > min_b - 1.0 && *y < min_b + 5.0)
        .map(|(_, _, y, _)| *y)
        .fold(f64::MAX, f64::min);
    assert!(
        (next_row_top - max_b).abs() <= 0.5,
        "다음 행 시작({next_row_top:.1})은 이전 행 바닥({max_b:.1})과 일치해야 함"
    );
}
