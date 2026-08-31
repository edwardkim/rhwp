//! #6300 강제 줄나눔(0x000A) 뒤에 인라인 개체가 오면 저장 줄 경계를 유지한다.
//!
//! 156464313 보도자료: 저장 LINE_SEG text_start `[0, 45, 83, 119, 152]` 인데
//! rhwp 가 `[0, 45, 83, 83, 152]` 로 119 를 잃고 다음 줄에 두 줄이 몰려 우단을
//! 넘겼다. `\n` 앞 텍스트를 이전 줄에 합치지 않아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, RectangleShape, ShapeObject};
use rhwp::model::table::Table;
use rhwp::renderer::composer::compose_paragraph;

fn issue_paragraph(control: Control) -> Paragraph {
    let mut chars: Vec<char> = "가".repeat(151).chars().collect();
    chars.push('\n');
    let text: String = chars.iter().collect();
    let n = text.chars().count();
    Paragraph {
        text,
        char_offsets: (0..n as u32).collect(),
        char_count: n as u32 + 8,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![
            LineSeg {
                text_start: 0,
                line_height: 1500,
                text_height: 1500,
                baseline_distance: 1200,
                ..Default::default()
            },
            LineSeg {
                text_start: 45,
                line_height: 1500,
                text_height: 1500,
                baseline_distance: 1200,
                ..Default::default()
            },
            LineSeg {
                text_start: 83,
                line_height: 1500,
                text_height: 1500,
                baseline_distance: 1200,
                ..Default::default()
            },
            LineSeg {
                text_start: 119,
                line_height: 1500,
                text_height: 1500,
                baseline_distance: 1200,
                ..Default::default()
            },
            LineSeg {
                text_start: 152,
                line_height: 7730,
                text_height: 7730,
                baseline_distance: 6500,
                ..Default::default()
            },
        ],
        controls: vec![control],
        ..Default::default()
    }
}

fn tac_table() -> Control {
    let mut table = Table::default();
    table.common.treat_as_char = true;
    table.common.width = 20_000;
    table.common.height = 7730;
    Control::Table(Box::new(table))
}

fn tac_shape() -> Control {
    Control::Shape(Box::new(ShapeObject::Rectangle(RectangleShape {
        common: CommonObjAttr {
            treat_as_char: true,
            width: 20_000,
            height: 7730,
            ..Default::default()
        },
        ..Default::default()
    })))
}

fn composed_starts(para: &Paragraph) -> Vec<usize> {
    compose_paragraph(para)
        .lines
        .iter()
        .map(|line| line.char_start)
        .collect()
}

#[test]
fn hard_break_before_tac_table_keeps_stored_line_start_119() {
    let para = issue_paragraph(tac_table());
    let starts = composed_starts(&para);
    assert!(
        starts.contains(&119),
        "저장 text_start 119 가 살아 있어야 한다: {starts:?}"
    );
    let mashed = starts.windows(2).any(|w| w[0] == 83 && w[1] == 83);
    assert!(
        !mashed,
        "83 이 중복되면 한 줄이 비고 다음 줄에 두 줄이 몰린다: {starts:?}"
    );
}

#[test]
fn hard_break_before_tac_shape_keeps_stored_line_start_119() {
    let para = issue_paragraph(tac_shape());
    let starts = composed_starts(&para);
    assert!(
        starts.contains(&119),
        "도형 인라인 개체도 119 경계를 유지해야 한다: {starts:?}"
    );
    let mashed = starts.windows(2).any(|w| w[0] == 83 && w[1] == 83);
    assert!(!mashed, "83 중복 금지: {starts:?}");
}
