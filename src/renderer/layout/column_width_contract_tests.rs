//! 지배적인 행의 열 격자와 다른 총폭의 이상 행을 구분한다.
//!
//! #2512의 기존 100HU 합성 사례는 독립 Print로 확인한 최소 내용 폭 1mm보다
//! 작아 이상 행의 차이가 물리 폭에서 사라진다. 최소 폭보다 큰 원시 단위로
//! 같은 계약을 보호하며 실제 페이지의 좌표를 고정하지 않는다.

use super::{hwpunit_to_px, Cell, CommonObjAttr, LayoutEngine, Table, DEFAULT_DPI};

#[test]
fn row_width_outlier_preserves_the_nonminimum_authored_column_grid() {
    let nominal_hu = 1_000;
    let cells = (0..3)
        .flat_map(|row| {
            (0..2).map(move |col| Cell {
                row,
                col,
                row_span: 1,
                col_span: 1,
                width: nominal_hu,
                ..Default::default()
            })
        })
        .collect();
    let regular = Table {
        row_count: 3,
        col_count: 2,
        cells,
        common: CommonObjAttr {
            width: nominal_hu * 2,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut with_outlier = regular.clone();
    with_outlier.cells[0].width = nominal_hu * 3 / 2;
    let source_widths: Vec<_> = with_outlier.cells.iter().map(|cell| cell.width).collect();
    let engine = LayoutEngine::with_default_dpi();
    let regular_widths = engine.resolve_column_widths(&regular, 2);
    let resolved = engine.resolve_column_widths(&with_outlier, 2);

    assert_eq!(
        resolved, regular_widths,
        "한 이상 행이 지배적인 두 행의 열 경계를 확대하지 않음"
    );
    assert_eq!(resolved[0], resolved[1], "같은 원본 폭인 두 열의 관계 유지");
    let authored_total = hwpunit_to_px(regular.common.width as i32, DEFAULT_DPI);
    assert!(
        (resolved.iter().sum::<f64>() - authored_total).abs() < 1e-6,
        "실제 열 합계는 원본의 지배적인 행 합계와 같음"
    );
    assert_eq!(
        with_outlier
            .cells
            .iter()
            .map(|cell| cell.width)
            .collect::<Vec<_>>(),
        source_widths,
        "격자 계산이 저장 원시 폭을 변경하지 않음"
    );
}
