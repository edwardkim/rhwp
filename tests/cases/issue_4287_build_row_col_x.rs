//! [#4287] 선언 행/열 수가 `MAX_TABLE_GRID_CELLS` 를 넘는 표는 파싱 후
//! 렌더가 abort 없이 끝나야 한다. `build_row_col_x` 가 상한 초과를 오류로
//! 돌리면 `layout_table` 이 표를 건너뛴다.
//!
//! 가드가 사라지면 2100×2100 `Option<f64>` ≈ 70MB 를 예약한다. 65535×65535
//! 는 회귀 시 CI 러너가 OOM 으로 죽으므로 쓰지 않는다 (#2722 보정과 동일).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::table::MAX_TABLE_GRID_CELLS;
use rhwp::parser::hml::parse_hml;
use rhwp::DocumentCore;

const HOSTILE_HML: &[u8] = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><TABLE RowCount="2100" ColCount="2100">
<ROW><CELL ColAddr="0" RowAddr="0"><PARALIST><P><TEXT><CHAR>x</CHAR></TEXT></P></PARALIST></CELL></ROW>
</TABLE></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;

#[test]
fn hostile_declared_grid_parses_and_renders_without_abort() {
    assert!(
        2100usize.saturating_mul(2100) > MAX_TABLE_GRID_CELLS,
        "재현 입력이 상한을 넘어야 의미가 있다"
    );

    let parsed = parse_hml(HOSTILE_HML).expect("악성 표 카운트도 파싱되어야 함");
    let table = parsed.document.sections[0].paragraphs[0]
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .expect("표 컨트롤이 있어야 함");
    assert_eq!(table.row_count, 2100);
    assert_eq!(table.col_count, 2100);
    assert!(table.cell_grid.len() <= MAX_TABLE_GRID_CELLS);

    let core = DocumentCore::from_bytes(HOSTILE_HML).expect("HML 로드");
    let svg = core
        .render_page_svg_native(0)
        .expect("상한 초과 표는 렌더 abort 없이 끝나야 함");
    assert!(svg.contains("<svg"), "SVG 문서가 나와야 함: {svg:.80}");
}
