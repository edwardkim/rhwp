//! Issue #4898 ②: HWPX→HWP 어댑터는 셀 LIST_HEADER `width_ref` bit0(=aim, 자기 여백 사용)을
//! 셀 자신의 `apply_inner_margin` 으로만 정한다.
//!
//! 예전에는 `col_count >= 30` micro-grid 휴리스틱이 aim=false 셀에도 이 비트를 세우고
//! 표 기본 여백을 셀 padding 으로 물질화했다. 한글은 그만큼 셀 안 여백을 크게 잡아 행 높이가
//! 늘고, 1쪽에 맞던 서식이 2쪽으로 넘어갔다.
//!
//! 한글 2022 오라클 10k 전수(x2h) 실측: 휴리스틱이 실제로 산출을 바꾸는 문서 1,239건을 전수
//! 측정해 쪽수 결함 58건이 원본 쪽수로 복귀했고, 새로 깨진 문서는 0건이다.

use rhwp::document_core::converters::hwpx_to_hwp::convert_hwpx_to_hwp_ir;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::{Cell, Table};
use rhwp::model::Padding;

const MICRO_GRID_COLS: u16 = 32;

fn pad(v: i16) -> Padding {
    Padding {
        left: v,
        right: v,
        top: v,
        bottom: v,
    }
}

/// 고열 수(micro-grid) 표 한 개를 가진 최소 문서. 첫 셀은 aim=false, 둘째 셀은 aim=true.
fn micro_grid_document() -> Document {
    let mut aim_false = Cell {
        padding: pad(0),
        ..Default::default()
    };
    aim_false.apply_inner_margin = false;
    aim_false.col = 0;
    aim_false.width = 1000;

    let mut aim_true = Cell {
        padding: pad(141),
        ..Default::default()
    };
    aim_true.apply_inner_margin = true;
    aim_true.col = 1;
    aim_true.width = 1000;

    let table = Table {
        col_count: MICRO_GRID_COLS,
        row_count: 1,
        padding: pad(283),
        cells: vec![aim_false, aim_true],
        ..Default::default()
    };

    let mut paragraph = Paragraph::default();
    paragraph.controls.push(Control::Table(Box::new(table)));

    let mut section = Section::default();
    section.paragraphs.push(paragraph);

    let mut document = Document::default();
    document.sections.push(section);
    document
}

fn first_table(document: &Document) -> &Table {
    document
        .sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .find_map(|ctrl| match ctrl {
            Control::Table(t) => Some(t),
            _ => None,
        })
        .expect("표 없음")
}

#[test]
fn issue_4898_micro_grid_does_not_force_width_ref_bit0() {
    let mut document = micro_grid_document();
    convert_hwpx_to_hwp_ir(&mut document);

    let table = first_table(&document);
    let aim_false = &table.cells[0];

    assert_eq!(
        aim_false.list_header_width_ref & 0x0001,
        0,
        "aim=false 셀은 열 수와 무관하게 width_ref bit0 를 켜지 않는다 (#4898 ②)"
    );
    assert_eq!(
        (
            aim_false.padding.left,
            aim_false.padding.right,
            aim_false.padding.top,
            aim_false.padding.bottom
        ),
        (0, 0, 0, 0),
        "aim=false 셀 padding 을 표 기본 여백으로 물질화하지 않는다 — 한글 행 높이가 커진다"
    );
}

#[test]
fn issue_4898_cell_own_inner_margin_still_sets_width_ref_bit0() {
    let mut document = micro_grid_document();
    convert_hwpx_to_hwp_ir(&mut document);

    let table = first_table(&document);
    let aim_true = &table.cells[1];

    assert_eq!(
        aim_true.list_header_width_ref & 0x0001,
        1,
        "셀 자신이 안 여백을 쓰면 width_ref bit0 는 그대로 선다"
    );
    assert_eq!(
        (
            aim_true.padding.left,
            aim_true.padding.right,
            aim_true.padding.top,
            aim_true.padding.bottom
        ),
        (141, 141, 141, 141),
        "aim=true 셀의 고유 여백은 보존한다"
    );
}

#[test]
fn issue_4898_raw_list_extra_still_materialized() {
    let mut document = micro_grid_document();
    convert_hwpx_to_hwp_ir(&mut document);

    let table = first_table(&document);
    for (idx, cell) in table.cells.iter().enumerate() {
        assert_eq!(
            cell.raw_list_extra.len(),
            13,
            "셀 {idx}: raw_list_extra 물질화는 모든 셀에 그대로 유지한다"
        );
        assert_eq!(
            &cell.raw_list_extra[0..4],
            &cell.width.to_le_bytes(),
            "셀 {idx}: raw_list_extra 앞 4바이트는 셀 폭이다"
        );
    }
}
