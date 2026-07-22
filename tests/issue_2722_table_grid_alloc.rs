//! Issue #2722 회귀 가드 — 표 그리드 재구축의 무한 할당 방지.
//!
//! `Table::rebuild_grid()` 이 파일에서 온 `row_count`/`col_count` 를 검증 없이
//! 곱해 `vec![None; rc * cc]` 를 예약하면, 65535×65535 = 4,294,836,225 칸 ×
//! `Option<usize>` 16바이트 = 68,717,379,600 바이트 예약이 되어 할당 실패 시
//! `handle_alloc_error` → abort 로 프로세스가 죽는다(테스트 실패가 아니라
//! 바이너리 전체 사망). wasm32 에서는 `Layout::array` 가 32비트 `usize` 를
//! 넘겨 capacity overflow 패닉 → 모듈 트랩이 된다.
//!
//! 아래 가드는 (1) 모델 직접 호출, (2) HML 파싱 경로 두 곳에서 abort 없이
//! 유계 그리드로 끝나는지, 그리고 (3) 정상 표에서는 그리드 크기가 종전과
//! 완전히 동일한지를 고정한다.

use rhwp::model::control::Control;
use rhwp::model::table::{Cell, Table, MAX_TABLE_GRID_CELLS};
use rhwp::parser::hml::parse_hml;

/// (1) 모델 직접 호출 — 셀이 하나도 없는 상한 초과 표.
///
/// [메인테이너 보정] 종전에는 `65535 × 65535` 를 썼다. 가드가 살아 있으면 실셀 범위까지만
/// 예약하므로 안전하지만, **가드가 회귀로 사라지면 `Option<usize>` 16B × 42.9억 = 68.7GB**
/// 를 요구해 테스트가 실패하는 대신 러너가 스왑에 빠지거나 OOM 으로 죽는다. CI 에서 회귀를
/// 진단 가능한 형태로 잡아야 하므로, 상한(`MAX_TABLE_GRID_CELLS` = 4,000,000)을 넘기되
/// 회귀 시에도 감당 가능한 크기(2100 × 2100 = 4.41M 칸 ≈ 70MB)로 낮춘다. 상한 초과 여부를
/// 검사한다는 목적은 그대로다.
#[test]
fn rebuild_grid_bounds_hostile_row_col_count() {
    let mut table = Table::default();
    table.row_count = 2100;
    table.col_count = 2100;
    assert!(
        (table.row_count as usize) * (table.col_count as usize) > MAX_TABLE_GRID_CELLS,
        "재현 입력이 상한을 넘어야 의미가 있다"
    );
    table.rebuild_grid();

    assert!(
        table.cell_grid.len() <= MAX_TABLE_GRID_CELLS,
        "그리드가 상한을 넘음: {}",
        table.cell_grid.len()
    );
    // 셀이 없으므로 예약할 칸도 없다.
    assert_eq!(
        table.cell_grid.len(),
        0,
        "셀 0개 표는 그리드도 0칸이어야 함"
    );
    // 모델 필드는 건드리지 않는다 (직렬화·라운드트립 계약 보존).
    assert_eq!(table.row_count, 2100);
    assert_eq!(table.col_count, 2100);
}

/// (1') 셀이 하나 있는 상한 초과 표 — 실제 셀이 가리키는 범위까지만 예약.
///
/// [메인테이너 보정] (1) 과 같은 이유로 `65535 × 65535` 에서 낮췄다.
#[test]
fn rebuild_grid_bounds_hostile_counts_with_one_cell() {
    let mut table = Table::default();
    table.row_count = 2100;
    table.col_count = 2100;
    table.cells = vec![Cell {
        row: 0,
        col: 0,
        row_span: 1,
        col_span: 1,
        ..Default::default()
    }];
    table.rebuild_grid();

    assert_eq!(
        table.cell_grid.len(),
        1,
        "(0,0) 셀 하나만 있으면 그리드 1칸이면 충분"
    );
    assert_eq!(
        table.cell_grid[0],
        Some(0),
        "앵커 셀 인덱스는 유지되어야 함"
    );
}

/// (2) HML 파싱 경로 — 작은 악성 입력이 abort 없이 파싱돼야 한다.
///
/// [메인테이너 보정] (1) 과 같은 이유로 `65535` 에서 낮췄다. 파일이 선언한 카운트를 그대로
/// 믿고 곱하면 안 된다는 계약은 상한(4,000,000)을 넘기기만 하면 검증된다.
#[test]
fn hml_table_with_hostile_counts_parses_without_abort() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><TABLE RowCount="2100" ColCount="2100">
<ROW><CELL ColAddr="0" RowAddr="0"><PARALIST><P><TEXT><CHAR>x</CHAR></TEXT></P></PARALIST></CELL></ROW>
</TABLE></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;

    let parsed = parse_hml(xml).expect("악성 표 카운트도 graceful 하게 파싱되어야 함");
    let table = parsed.document.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .expect("표 컨트롤이 있어야 함");

    assert!(
        table.cell_grid.len() <= MAX_TABLE_GRID_CELLS,
        "그리드가 상한을 넘음: {}",
        table.cell_grid.len()
    );
    // 파일이 선언한 행/열 수는 그대로 보존한다.
    assert_eq!(table.row_count, 2100);
    assert_eq!(table.col_count, 2100);
}

/// (3) 정상 표는 그리드 크기가 종전 식(`rc * cc`)과 완전히 동일해야 한다.
#[test]
fn normal_table_grid_size_is_unchanged() {
    let mut table = Table::default();
    table.row_count = 3;
    table.col_count = 4;
    table.cells = (0..3)
        .flat_map(|r| {
            (0..4).map(move |c| Cell {
                row: r,
                col: c,
                row_span: 1,
                col_span: 1,
                ..Default::default()
            })
        })
        .collect();
    table.rebuild_grid();

    assert_eq!(table.cell_grid.len(), 3 * 4, "정상 표 그리드 크기 불변");
    for r in 0..3u16 {
        for c in 0..4u16 {
            assert!(
                table.cell_index_at(r, c).is_some(),
                "({r},{c}) 셀 조회가 유지되어야 함"
            );
        }
    }
}
