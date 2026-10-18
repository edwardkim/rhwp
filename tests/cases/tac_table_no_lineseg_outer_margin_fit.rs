//! 저장 줄 없는 문단의 글자처럼 취급 표는 바깥 여백을 쪽 예산에 한 번만 센다.
//! format 이 이미 더한 여백을 쪽 맞춤 전 판정이 다시 더해, 쪽 끝에 들어가는 표가
//! 다음 쪽으로 밀렸다. 공개 합성 IR이다.
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section, SectionDef};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, TextWrap};
use rhwp::model::style::{LineSpacingType, ParaShape};
use rhwp::model::table::{Cell, Table};
use rhwp::model::Padding;
use rhwp::DocumentCore;

const BODY_HEIGHT: u32 = 56_000;
const FIRST_LINE: i32 = 1_000;
const MARGIN: i16 = 283;

fn fixture(table_height: u32) -> Document {
    let first = Paragraph {
        char_count: 1,
        line_segs: vec![LineSeg {
            line_height: FIRST_LINE,
            text_height: FIRST_LINE,
            baseline_distance: FIRST_LINE * 4 / 5,
            segment_width: 40_000,
            tag: 393216,
            ..Default::default()
        }],
        ..Default::default()
    };
    let table = Table {
        // HWP5 CTRL_HEADER 의 글자처럼 취급 비트
        attr: 1,
        row_count: 1,
        col_count: 1,
        outer_margin_top: MARGIN,
        outer_margin_bottom: MARGIN,
        common: CommonObjAttr {
            width: 20_000,
            height: table_height,
            treat_as_char: true,
            text_wrap: TextWrap::TopAndBottom,
            margin: Padding {
                top: MARGIN,
                bottom: MARGIN,
                ..Default::default()
            },
            ..Default::default()
        },
        cells: vec![Cell {
            row_span: 1,
            col_span: 1,
            width: 20_000,
            height: table_height,
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape {
        line_spacing: 160,
        line_spacing_type: LineSpacingType::Percent,
        ..Default::default()
    }];
    doc.sections.push(Section {
        paragraphs: vec![
            first,
            Paragraph {
                controls: vec![Control::Table(Box::new(table))],
                char_count: 9,
                ..Default::default()
            },
        ],
        section_def: SectionDef {
            page_def: PageDef {
                width: 44_000,
                height: BODY_HEIGHT + 4_000,
                margin_left: 2_000,
                margin_right: 2_000,
                margin_top: 2_000,
                margin_bottom: 2_000,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
    doc
}

fn page_count(table_height: u32) -> u32 {
    let mut core = DocumentCore::new_empty();
    core.set_document(fixture(table_height));
    core.page_count()
}

#[test]
fn tac_table_counts_outer_margin_once_at_page_bottom() {
    // 첫 줄 + 표 + 위아래 여백이 본문보다 100HU 작다. 여백을 두 번 세면 466HU 넘친다.
    let fits = BODY_HEIGHT - FIRST_LINE as u32 - 2 * MARGIN as u32 - 100;
    assert_eq!(page_count(fits), 1);
}

#[test]
fn tac_table_overflowing_with_single_margin_moves_to_next_page() {
    let overflows = BODY_HEIGHT - FIRST_LINE as u32 - 2 * MARGIN as u32 + 200;
    assert_eq!(page_count(overflows), 2);
}
