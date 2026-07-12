//! Issue #2164: 표 셀에서 Enter로 만든 문단의 vpos가 앞 문단과 겹치는 회귀.
//!
//! 셀 문단 분할 뒤 `LINE_SEG.vertical_pos` 축을 다시 연결하지 않으면 새 문단이
//! 앞 문단과 같은 셀 상단에 배치된다. 실제 제보 원본에서 모델 vpos와 캐럿 y가
//! 모두 문단 순서대로 증가하는지 검증한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

const SAMPLE: &str = "samples/issue2164/의견제출서(양식).hwp";

fn load_sample() -> DocumentCore {
    let bytes = std::fs::read(SAMPLE).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"))
}

fn find_target_cell(core: &DocumentCore) -> (usize, usize, usize) {
    for (parent_para_idx, para) in core.document().sections[0].paragraphs.iter().enumerate() {
        for (control_idx, control) in para.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            let Some(heading) = table.cells.iter().find(|cell| {
                cell.paragraphs
                    .iter()
                    .any(|para| para.text.contains("의견제출 요지"))
            }) else {
                continue;
            };
            let target_row = heading.row + heading.row_span;
            let target_idx = table
                .cells
                .iter()
                .position(|cell| cell.row == target_row && cell.col == heading.col)
                .expect("의견제출 요지 다음 입력 셀");
            return (parent_para_idx, control_idx, target_idx);
        }
    }
    panic!("의견제출 요지 표를 찾지 못함");
}

fn cell_paragraphs(
    core: &DocumentCore,
    parent: usize,
    control: usize,
    cell: usize,
) -> &[rhwp::model::paragraph::Paragraph] {
    match &core.document().sections[0].paragraphs[parent].controls[control] {
        Control::Table(table) => &table.cells[cell].paragraphs,
        other => panic!("대상 컨트롤이 표가 아님: {other:?}"),
    }
}

fn cursor_y(
    core: &DocumentCore,
    parent: usize,
    control: usize,
    cell: usize,
    cell_para: usize,
    offset: usize,
) -> f64 {
    let json = core
        .get_cursor_rect_in_cell_native(0, parent, control, cell, cell_para, offset)
        .unwrap_or_else(|e| panic!("cell paragraph {cell_para} cursor rect: {e}"));
    serde_json::from_str::<Value>(&json).expect("cursor rect JSON")["y"]
        .as_f64()
        .expect("cursor rect y")
}

fn first_vpos(paragraphs: &[rhwp::model::paragraph::Paragraph], index: usize) -> i32 {
    paragraphs[index]
        .line_segs
        .first()
        .unwrap_or_else(|| panic!("cell paragraph {index} LINE_SEG"))
        .vertical_pos
}

#[test]
fn enter_in_table_cell_keeps_following_paragraphs_below_previous_paragraph() {
    let mut core = load_sample();
    let (parent, control, cell) = find_target_cell(&core);
    assert_eq!(cell_paragraphs(&core, parent, control, cell).len(), 2);

    let text = "1212121212121212121";
    core.insert_text_in_cell_native(0, parent, control, cell, 0, 0, text)
        .expect("셀 텍스트 입력");
    core.split_paragraph_in_cell_native(0, parent, control, cell, 0, text.chars().count())
        .expect("셀 문단 분할");

    let paragraphs = cell_paragraphs(&core, parent, control, cell);
    assert_eq!(paragraphs.len(), 3, "Enter로 셀 문단이 하나 늘어야 함");
    let vpos = [
        first_vpos(paragraphs, 0),
        first_vpos(paragraphs, 1),
        first_vpos(paragraphs, 2),
    ];
    assert!(
        vpos[0] < vpos[1] && vpos[1] < vpos[2],
        "Enter 뒤 셀 문단 vpos가 순서대로 증가해야 함: {vpos:?}"
    );

    let y = [
        cursor_y(&core, parent, control, cell, 0, text.chars().count()),
        cursor_y(&core, parent, control, cell, 1, 0),
        cursor_y(&core, parent, control, cell, 2, 0),
    ];
    assert!(
        y[0] < y[1] && y[1] < y[2],
        "Enter 뒤 캐럿 y가 문단 순서대로 증가해야 함: {y:?}"
    );

    let saved = core.export_hwp_native().expect("편집 HWP 저장");
    let reopened = DocumentCore::from_bytes(&saved).expect("편집 HWP 재로드");
    let (saved_parent, saved_control, saved_cell) = find_target_cell(&reopened);
    let saved_paragraphs = cell_paragraphs(&reopened, saved_parent, saved_control, saved_cell);
    assert_eq!(saved_paragraphs.len(), 3, "저장 후 셀 문단 수 보존");
    let saved_vpos = [
        first_vpos(saved_paragraphs, 0),
        first_vpos(saved_paragraphs, 1),
        first_vpos(saved_paragraphs, 2),
    ];
    assert!(
        saved_vpos[0] < saved_vpos[1] && saved_vpos[1] < saved_vpos[2],
        "저장 후에도 셀 문단 vpos가 순서대로 증가해야 함: {saved_vpos:?}"
    );
    let saved_y = [
        cursor_y(
            &reopened,
            saved_parent,
            saved_control,
            saved_cell,
            0,
            text.chars().count(),
        ),
        cursor_y(&reopened, saved_parent, saved_control, saved_cell, 1, 0),
        cursor_y(&reopened, saved_parent, saved_control, saved_cell, 2, 0),
    ];
    assert!(
        saved_y[0] < saved_y[1] && saved_y[1] < saved_y[2],
        "저장 후 캐럿 y가 문단 순서대로 증가해야 함: {saved_y:?}"
    );
}

#[test]
fn backspace_merge_then_enter_reuses_the_same_cell_paragraph_flow() {
    let mut core = load_sample();
    let (parent, control, cell) = find_target_cell(&core);
    let text = "1212121212121212121";

    core.insert_text_in_cell_native(0, parent, control, cell, 0, 0, text)
        .expect("셀 텍스트 입력");
    core.split_paragraph_in_cell_native(0, parent, control, cell, 0, text.chars().count())
        .expect("첫 Enter");
    core.merge_paragraph_in_cell_native(0, parent, control, cell, 1)
        .expect("Backspace 문단 병합");
    assert_eq!(cell_paragraphs(&core, parent, control, cell).len(), 2);

    core.split_paragraph_in_cell_native(0, parent, control, cell, 0, text.chars().count())
        .expect("두 번째 Enter");
    let y = [
        cursor_y(&core, parent, control, cell, 0, text.chars().count()),
        cursor_y(&core, parent, control, cell, 1, 0),
        cursor_y(&core, parent, control, cell, 2, 0),
    ];
    assert!(
        y[0] < y[1] && y[1] < y[2],
        "Backspace 뒤 다시 Enter해도 같은 문단 흐름이어야 함: {y:?}"
    );
}

#[test]
fn repeated_enter_in_table_cell_advances_to_the_new_third_paragraph() {
    let mut core = load_sample();
    let (parent, control, cell) = find_target_cell(&core);
    let first_text = "1111";
    let second_text = "2222";

    core.insert_text_in_cell_native(0, parent, control, cell, 0, 0, first_text)
        .expect("첫 셀 텍스트 입력");
    core.split_paragraph_in_cell_native(0, parent, control, cell, 0, first_text.chars().count())
        .expect("첫 Enter");
    core.insert_text_in_cell_native(0, parent, control, cell, 1, 0, second_text)
        .expect("두 번째 셀 텍스트 입력");
    core.split_paragraph_in_cell_native(0, parent, control, cell, 1, second_text.chars().count())
        .expect("두 번째 Enter");

    let paragraphs = cell_paragraphs(&core, parent, control, cell);
    assert_eq!(
        paragraphs.len(),
        4,
        "연속 Enter로 셀 문단이 두 개 늘어야 함"
    );
    let vpos: Vec<_> = (0..paragraphs.len())
        .map(|index| first_vpos(paragraphs, index))
        .collect();
    assert!(
        vpos.windows(2).all(|pair| pair[0] < pair[1]),
        "두 번째 Enter 뒤에도 셀 문단 vpos가 순서대로 증가해야 함: {vpos:?}"
    );

    let y = [
        cursor_y(&core, parent, control, cell, 0, first_text.chars().count()),
        cursor_y(&core, parent, control, cell, 1, second_text.chars().count()),
        cursor_y(&core, parent, control, cell, 2, 0),
        cursor_y(&core, parent, control, cell, 3, 0),
    ];
    assert!(
        y.windows(2).all(|pair| pair[0] < pair[1]),
        "두 번째 Enter 뒤 캐럿 y가 새 세 번째 문단까지 증가해야 함: {y:?}"
    );
}
