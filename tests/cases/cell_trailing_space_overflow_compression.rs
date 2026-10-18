//! 칸 안 오른쪽 정렬 숫자의 꼬리 공백이 넘침 판정에 들어가 글자가 눌린다.
//!
//! 통계표 숫자 칸은 흔히 `"120,283 "` 처럼 꼬리 공백 하나를 단다. 가시 글자는 칸 안쪽
//! 폭에 들어가는데 꼬리 공백까지 더한 폭이 넘치면, 비정렬(왼쪽/오른쪽/가운데) 줄의 넘침
//! 압축이 음수 자간을 걸어 숫자 전체를 가로로 눌렀다. 정렬은 이미 꼬리 공백을 빼고
//! 하므로(#7081) 압축 판정만 공백을 세고 있었다. 한/글은 꼬리 공백을 줄 밖에 매달고
//! 글자를 누르지 않는다.
//!
//! 불변식: 가시 글자가 들어가는 칸에서는 꼬리 공백의 유무가 글자 자리를 바꾸지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// `(run 시작 x, 글자 경계)` — 공개 text-layout 으로 읽는다.
fn number_run(core: &DocumentCore) -> (f64, Vec<f64>) {
    let raw = core.get_page_text_layout_native(0).expect("text-layout");
    let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON");
    let runs: Vec<_> = layout["runs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|run| {
            run["text"]
                .as_str()
                .is_some_and(|t| t.starts_with("120,283"))
        })
        .collect();
    assert_eq!(runs.len(), 1, "숫자 런은 하나여야 한다");
    let x = runs[0]["x"].as_f64().expect("x");
    let char_x = runs[0]["charX"]
        .as_array()
        .expect("charX")
        .iter()
        .map(|v| v.as_f64().expect("charX 값"))
        .collect();
    (x, char_x)
}

/// 1×1 표의 오른쪽 정렬 칸에 `text` 를 넣고 칸 폭을 `cell_width` 로 맞춘다.
fn right_aligned_cell(text: &str, cell_width: Option<u32>) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    {
        let mut section = rhwp::model::document::Section::default();
        section.section_def.page_def = rhwp::model::page::PageDef::a4_default();
        section
            .paragraphs
            .push(rhwp::model::paragraph::Paragraph::new_empty());
        let mut document = rhwp::model::document::Document::default();
        document.sections.push(section);
        core.set_document(document);
    }
    core.create_table_native(0, 0, 0, 1, 1).expect("1×1 표");
    core.insert_text_in_cell_native(0, 0, 0, 0, 0, 0, text)
        .expect("칸 글 삽입");
    core.apply_para_format_in_cell_native(0, 0, 0, 0, 0, r#"{"alignment":"right"}"#)
        .expect("오른쪽 정렬");
    if let Some(width) = cell_width {
        let mut document = core.document().clone();
        let Some(Control::Table(table)) = document.sections[0].paragraphs[0].controls.first_mut()
        else {
            panic!("표를 찾지 못했다");
        };
        let cell = &mut table.cells[0];
        cell.apply_inner_margin = true;
        cell.padding.left = 0;
        cell.padding.right = 0;
        for seg in &mut cell.paragraphs[0].line_segs {
            seg.segment_width = width as i32;
        }
        table.set_column_widths(&[width]).expect("열 폭");
        core.set_document(document);
    }
    core
}

#[test]
fn trailing_space_does_not_compress_fitting_right_aligned_number() {
    // 넓은 칸에서 자연 폭을 잰다.
    let (_, natural) = number_run(&right_aligned_cell("120,283 ", None));
    let visible = natural[7];
    let space = natural[8] - natural[7];
    assert!(space > 1.0, "꼬리 공백 폭이 있어야 한다: {natural:?}");

    // 가시 글자는 들어가고 꼬리 공백까지는 들어가지 않는 칸 (px → HWPUNIT 75배).
    let width = ((visible + space / 2.0) * 75.0).round() as u32;
    let (x_with, with_space) = number_run(&right_aligned_cell("120,283 ", Some(width)));
    let (x_without, without_space) = number_run(&right_aligned_cell("120,283", Some(width)));

    for i in 0..=7 {
        assert!(
            (x_with + with_space[i] - (x_without + without_space[i])).abs() < 0.15,
            "꼬리 공백 유무가 {i}번째 글자 자리를 바꾸면 안 된다: \
             with={x_with}+{with_space:?} without={x_without}+{without_space:?}"
        );
    }
}
