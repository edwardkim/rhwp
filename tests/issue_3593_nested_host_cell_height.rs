//! [#3593] 중첩 표를 품은 셀의 콘텐츠 높이 — 저장 vpos 사다리가 붕괴한 문서 회귀 가드.
//!
//! 중첩 표 호스트 셀의 높이는 `height_measurer` 에서
//! `last_seg_end.max(text_height).max(nested_bottom)` 으로 합성한다. 이는 문단들이
//! `LINE_SEG.vertical_pos` 로 "사다리"를 이룬다는 가정이다 — 마지막 문단의
//! `vpos + line_height` 가 곧 셀 전체 높이라는 것.
//!
//! 사다리가 붕괴한 문서(둘째 이후 문단의 vpos 가 0)에서는 이 가정이 무너진다.
//! `para_top` 이 전부 0 이 되어 `nested_bottom` 은 "가장 큰 중첩 표 하나의 높이"로
//! 축소되고, `max` 합성이라 텍스트 높이와 중첩 표 높이가 서로를 가린다.
//!
//! 결과: 셀 높이가 과소 측정되고, 페이지 분할 시 조각이 짧아져 잔여 문단·중첩 행이
//! 렌더에서 탈락한다(표 괘선만 남고 안이 비어 보인다).
//!
//! 파일 픽스처 대신 모델을 직접 조립한다 — 재현 조건(사다리 붕괴 + 중첩 표 높이의
//! lineseg 흡수 여부)을 케이스별로 정확히 고정할 수 있다.

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::height_measurer::HeightMeasurer;
use rhwp::renderer::style_resolver::resolve_styles_with_variant;
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

const TEXT_LH: i32 = 900;
/// lineseg 가 흡수하지 못하는 큰 중첩 표
const BIG_NESTED_H: i32 = 30000;
/// lineseg lh 가 이미 담고 있는 작은 중첩 표
const SMALL_NESTED_H: i32 = 1965;
const SMALL_NESTED_LH: i32 = 2535;

/// `vpos` / `line_height` 를 지정한 단일 seg 문단.
fn para(text: &str, vpos: i32, line_height: i32) -> Paragraph {
    Paragraph {
        text: text.to_string(),
        char_count: text.chars().count() as u32,
        line_segs: vec![LineSeg {
            vertical_pos: vpos,
            line_height,
            text_height: line_height,
            baseline_distance: line_height * 3 / 4,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// 지정 높이의 1행 1열 중첩 표.
fn nested_table(height_hu: i32, cell_text: &str) -> Table {
    let mut common = rhwp::model::shape::CommonObjAttr::default();
    common.height = height_hu as u32;
    common.width = 40000;
    Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            row: 0,
            col: 0,
            row_span: 1,
            col_span: 1,
            width: 40000_u32,
            height: height_hu as u32,
            paragraphs: vec![para(cell_text, 0, height_hu.min(TEXT_LH))],
            ..Default::default()
        }],
        cell_grid: vec![Some(0)],
        common,
        ..Default::default()
    }
}

/// 중첩 표를 품은 문단 (텍스트 없는 호스트 문단).
fn nested_host_para(nested: Table, own_lh: i32) -> Paragraph {
    let mut p = para("", 0, own_lh);
    p.controls = vec![Control::Table(Box::new(nested))];
    p
}

/// 사다리가 붕괴한(전 문단 vpos=0) 중첩 표 호스트 셀 하나를 가진 표를 만든다.
fn build_collapsed_ladder_table() -> Table {
    let paragraphs = vec![
        para("첫 문단", 0, TEXT_LH),
        // 둘째 이후 문단의 vpos == 0 = "앵커 없음" 센티널 → 사다리 붕괴
        para("둘째 문단", 0, TEXT_LH),
        // lh 가 중첩 표 높이를 담지 못한다 → 셀 높이에 별도로 가산돼야 한다
        nested_host_para(nested_table(BIG_NESTED_H, "큰 중첩 표"), TEXT_LH),
        // lh 가 이미 중첩 표 높이를 담고 있다 → 가산하면 이중 계상이다
        nested_host_para(
            nested_table(SMALL_NESTED_H, "작은 중첩 표"),
            SMALL_NESTED_LH,
        ),
    ];

    let mut common = rhwp::model::shape::CommonObjAttr::default();
    common.width = 60000;
    common.height = 200000;
    Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            row: 0,
            col: 0,
            row_span: 1,
            col_span: 1,
            width: 60000_u32,
            height: 200000_u32,
            paragraphs,
            ..Default::default()
        }],
        cell_grid: vec![Some(0)],
        common,
        ..Default::default()
    }
}

fn measure_host_cell_height(table: Table) -> f64 {
    let host_para = Paragraph {
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    };
    let paragraphs = vec![host_para];
    let composed: Vec<_> = paragraphs.iter().map(compose_paragraph).collect();

    let doc = Document::default();
    let styles = resolve_styles_with_variant(&doc.doc_info, DEFAULT_DPI, false);
    let measured =
        HeightMeasurer::new(DEFAULT_DPI).measure_section(&paragraphs, &composed, &styles, None);

    let measured_table = measured.tables.first().expect("측정된 표");
    let cell = measured_table.cells.first().expect("측정된 셀");
    assert!(cell.has_nested_table, "대상 셀은 중첩 표 호스트여야 한다");
    cell.total_content_height
}

#[test]
fn collapsed_vpos_ladder_cell_height_adds_unabsorbed_nested_table() {
    let measured = measure_host_cell_height(build_collapsed_ladder_table());

    // 텍스트 3문단(TEXT_LH) + 흡수 문단 1개(SMALL_NESTED_LH) 는 줄높이로 이미 잡힌다.
    let text_stack = hwpunit_to_px(TEXT_LH * 3 + SMALL_NESTED_LH, DEFAULT_DPI);
    // 큰 중첩 표는 어느 lineseg 도 담지 못하므로 별도로 더해져야 한다.
    let big_nested = hwpunit_to_px(BIG_NESTED_H, DEFAULT_DPI);
    let expected_min = text_stack + big_nested;

    assert!(
        measured >= expected_min - 1.0,
        "사다리 붕괴 셀 높이가 과소 측정됐다: 측정={measured:.1}px, \
         최소 기대={expected_min:.1}px (텍스트 스택 {text_stack:.1} + 미흡수 중첩표 {big_nested:.1})"
    );
}

#[test]
fn absorbed_nested_table_is_not_counted_twice() {
    let measured = measure_host_cell_height(build_collapsed_ladder_table());

    // 흡수된 작은 중첩 표(SMALL_NESTED_H)를 한 번 더 더하면 이중 계상이다.
    let text_stack = hwpunit_to_px(TEXT_LH * 3 + SMALL_NESTED_LH, DEFAULT_DPI);
    let big_nested = hwpunit_to_px(BIG_NESTED_H, DEFAULT_DPI);
    let small_nested = hwpunit_to_px(SMALL_NESTED_H, DEFAULT_DPI);
    let double_counted = text_stack + big_nested + small_nested;

    assert!(
        measured < double_counted - 1.0,
        "흡수된 중첩 표가 이중 계상됐다: 측정={measured:.1}px, \
         이중 계상 기준={double_counted:.1}px (흡수분 {small_nested:.1} 이 또 더해졌다)"
    );
}
