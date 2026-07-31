//! Task #2342 셀 후속: 셀 문단 병합의 undo 가 사라진 문단의 스코프 메타를 되돌린다.
//!
//! `split_at` 은 새 문단을 앞 문단에서 파생시키므로, 병합으로 사라졌던 문단을 되살릴 때
//! 메타를 함께 넣어 주지 않으면 문단 1 의 서식을 뒤집어쓴다. 본문·머리말/꼬리말·각주는
//! PR #3223 에서 닫혔고, 이 테스트는 남아 있던 **셀 경로**(표 셀 · 글상자 · 캡션 ·
//! 중첩 by-path)를 고정한다.
//!
//! 프리미티브(`Paragraph::capture_meta`/`apply_meta`, `ParaMeta` 7 필드)는 이미 있으므로
//! 병합이 메타를 실어 보내고 분할이 그것을 받는 배선만 확인하면 된다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::image::Picture;
use rhwp::model::paragraph::{ColumnBreakType, NumberingRestart, ParaMeta, Paragraph};
use rhwp::model::shape::Caption;
use rhwp::model::table::{Cell, Table};
use serde_json::Value;

/// 2×2 표 하나를 가진 문서. 반환값은 표가 놓인 본문 문단 인덱스.
fn doc_with_table() -> (DocumentCore, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank document");
    core.create_table_ex_native(0, 0, 0, 2, 2, false, None, None)
        .expect("2x2 table");

    let para_idx = core.document().sections[0]
        .paragraphs
        .iter()
        .position(|p| p.controls.iter().any(|c| matches!(c, Control::Table(_))))
        .expect("표가 놓인 문단");
    (core, para_idx)
}

fn table_control_idx(core: &DocumentCore, para_idx: usize) -> usize {
    core.document().sections[0].paragraphs[para_idx]
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(_)))
        .expect("표 컨트롤")
}

fn doc_with_textbox() -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank document");
    let inserted = core
        .create_shape_control_native(
            0,
            0,
            0,
            21_600,
            7_200,
            0,
            0,
            true,
            "TopAndBottom",
            "textbox",
            false,
            false,
            &[],
        )
        .expect("글상자 생성");
    let inserted: Value = serde_json::from_str(&inserted).expect("shape result json");
    let para_idx = inserted["paraIdx"].as_u64().expect("shape paraIdx") as usize;
    let ctrl_idx = inserted["controlIdx"].as_u64().expect("shape controlIdx") as usize;
    (core, para_idx, ctrl_idx)
}

fn doc_with_picture_caption() -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank document");

    let mut picture = Picture::default();
    picture.common.width = 21_600;
    picture.common.height = 7_200;
    picture.caption = Some(Caption {
        width: 10_000,
        max_width: 10_000,
        paragraphs: vec![Paragraph::default()],
        ..Default::default()
    });

    let para_idx = 0;
    let para = &mut core.document_mut().sections[0].paragraphs[para_idx];
    para.controls.push(Control::Picture(Box::new(picture)));
    let ctrl_idx = para.controls.len() - 1;
    (core, para_idx, ctrl_idx)
}

fn control_paragraphs(
    core: &DocumentCore,
    para_idx: usize,
    ctrl_idx: usize,
    cell_idx: usize,
) -> &[Paragraph] {
    match &core.document().sections[0].paragraphs[para_idx].controls[ctrl_idx] {
        Control::Table(table) => &table.cells[cell_idx].paragraphs,
        Control::Shape(shape) => {
            assert_eq!(cell_idx, 0, "글상자의 cell index");
            &shape
                .drawing()
                .and_then(|drawing| drawing.text_box.as_ref())
                .expect("글상자")
                .paragraphs
        }
        Control::Picture(picture) => {
            assert_eq!(cell_idx, 0, "그림 캡션의 cell index");
            &picture.caption.as_ref().expect("그림 캡션").paragraphs
        }
        other => panic!("셀 문단 컨테이너가 아님: {other:?}"),
    }
}

fn control_paragraphs_mut(
    core: &mut DocumentCore,
    para_idx: usize,
    ctrl_idx: usize,
    cell_idx: usize,
) -> &mut Vec<Paragraph> {
    match &mut core.document_mut().sections[0].paragraphs[para_idx].controls[ctrl_idx] {
        Control::Table(table) => &mut table.cells[cell_idx].paragraphs,
        Control::Shape(shape) => {
            assert_eq!(cell_idx, 0, "글상자의 cell index");
            &mut shape
                .drawing_mut()
                .and_then(|drawing| drawing.text_box.as_mut())
                .expect("글상자")
                .paragraphs
        }
        Control::Picture(picture) => {
            assert_eq!(cell_idx, 0, "그림 캡션의 cell index");
            &mut picture.caption.as_mut().expect("그림 캡션").paragraphs
        }
        other => panic!("셀 문단 컨테이너가 아님: {other:?}"),
    }
}

/// 병합 결과 JSON 에서 `removedParaMeta` 를 꺼낸다.
fn removed_meta(result: &str) -> ParaMeta {
    let value: Value = serde_json::from_str(result).expect("merge result json");
    serde_json::from_value(value["removedParaMeta"].clone())
        .unwrap_or_else(|e| panic!("removedParaMeta 가 있어야 한다 ({e}): {result}"))
}

/// 두 번째 문단에 알아볼 수 있는 메타를 심는다.
fn stamp_meta(para: &mut Paragraph) {
    para.para_shape_id = 20;
    para.style_id = 5;
    para.column_type = ColumnBreakType::Page;
    para.raw_break_type = 0x04;
    para.numbering_restart = Some(NumberingRestart::NewStart(7));
    para.raw_header_extra = vec![0, 0, 0, 0, 0, 0, 0xBB, 0xBB, 0xBB, 0xBB];
    para.tab_extended = vec![[100, 0, 0x0200, 0, 0, 0, 9]];
}

fn assert_meta_restored(para: &Paragraph) {
    assert_eq!(para.para_shape_id, 20, "문단 모양");
    assert_eq!(para.style_id, 5, "스타일");
    assert_eq!(para.column_type, ColumnBreakType::Page, "단 나눔");
    assert_eq!(para.raw_break_type, 0x04, "raw 나눔 종류");
    assert_eq!(
        para.numbering_restart,
        Some(NumberingRestart::NewStart(7)),
        "번호 다시 시작"
    );
    assert_eq!(
        para.raw_header_extra,
        vec![0, 0, 0, 0, 0, 0, 0xBB, 0xBB, 0xBB, 0xBB],
        "raw 헤더 여분"
    );
    assert_eq!(
        para.tab_extended,
        vec![[100, 0, 0x0200, 0, 0, 0, 9]],
        "확장 탭"
    );
}

fn assert_flat_merge_undo_restores_meta(
    mut core: DocumentCore,
    para_idx: usize,
    ctrl_idx: usize,
    container: &str,
) {
    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 0, "첫째")
        .unwrap_or_else(|e| panic!("{container} 첫 문단 삽입 실패: {e}"));
    core.split_paragraph_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 2, None)
        .unwrap_or_else(|e| panic!("{container} 문단 분할 실패: {e}"));
    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, 0, 1, 0, "둘째")
        .unwrap_or_else(|e| panic!("{container} 둘째 문단 삽입 실패: {e}"));

    {
        let paragraphs = control_paragraphs_mut(&mut core, para_idx, ctrl_idx, 0);
        paragraphs[0].para_shape_id = 10;
        stamp_meta(&mut paragraphs[1]);
    }

    let merged = core
        .merge_paragraph_in_cell_native(0, para_idx, ctrl_idx, 0, 1)
        .unwrap_or_else(|e| panic!("{container} 문단 병합 실패: {e}"));
    let meta = removed_meta(&merged);

    core.split_paragraph_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 2, Some(meta))
        .unwrap_or_else(|e| panic!("{container} undo 분할 실패: {e}"));

    let paragraphs = control_paragraphs(&core, para_idx, ctrl_idx, 0);
    assert_eq!(paragraphs[1].text, "둘째", "{container} 둘째 문단 텍스트");
    assert_meta_restored(&paragraphs[1]);
    assert_eq!(
        paragraphs[0].para_shape_id, 10,
        "{container} 앞 문단은 그대로"
    );
}

/// 표 셀: 병합 → undo(분할)가 사라진 문단의 메타를 되돌린다.
#[test]
fn cell_merge_undo_restores_removed_paragraph_meta() {
    let (core, para_idx) = doc_with_table();
    let ctrl_idx = table_control_idx(&core, para_idx);
    assert_flat_merge_undo_restores_meta(core, para_idx, ctrl_idx, "표 셀");
}

/// 글상자 arm 도 병합 결과에 사라진 문단의 메타를 싣는다.
#[test]
fn textbox_merge_undo_restores_removed_paragraph_meta() {
    let (core, para_idx, ctrl_idx) = doc_with_textbox();
    assert_flat_merge_undo_restores_meta(core, para_idx, ctrl_idx, "글상자");
}

/// 그림 캡션 arm 도 병합 결과에 사라진 문단의 메타를 싣는다.
#[test]
fn picture_caption_merge_undo_restores_removed_paragraph_meta() {
    let (core, para_idx, ctrl_idx) = doc_with_picture_caption();
    assert_flat_merge_undo_restores_meta(core, para_idx, ctrl_idx, "그림 캡션");
}

fn doc_with_nested_table() -> (DocumentCore, usize, Vec<(usize, usize, usize)>) {
    let (mut core, para_idx) = doc_with_table();
    let outer_ctrl_idx = table_control_idx(&core, para_idx);

    let mut inner_table = Table {
        row_count: 1,
        col_count: 1,
        row_sizes: vec![5_000],
        cells: vec![Cell {
            col_span: 1,
            row_span: 1,
            width: 5_000,
            height: 5_000,
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        }],
        cell_grid: vec![Some(0)],
        ..Default::default()
    };
    inner_table.common.width = 5_000;
    inner_table.common.height = 5_000;

    let outer_table =
        match &mut core.document_mut().sections[0].paragraphs[para_idx].controls[outer_ctrl_idx] {
            Control::Table(table) => table,
            other => panic!("바깥 표가 아님: {other:?}"),
        };
    let outer_cell_para = &mut outer_table.cells[0].paragraphs[0];
    outer_cell_para
        .controls
        .push(Control::Table(Box::new(inner_table)));
    let inner_ctrl_idx = outer_cell_para.controls.len() - 1;

    let path = vec![(outer_ctrl_idx, 0, 0), (inner_ctrl_idx, 0, 0)];
    (core, para_idx, path)
}

fn nested_cell_paragraphs<'a>(
    core: &'a DocumentCore,
    para_idx: usize,
    path: &[(usize, usize, usize)],
) -> &'a [Paragraph] {
    let Control::Table(outer_table) =
        &core.document().sections[0].paragraphs[para_idx].controls[path[0].0]
    else {
        panic!("바깥 표가 아님");
    };
    let outer_cell_para = &outer_table.cells[path[0].1].paragraphs[path[0].2];
    let Control::Table(inner_table) = &outer_cell_para.controls[path[1].0] else {
        panic!("안쪽 표가 아님");
    };
    &inner_table.cells[path[1].1].paragraphs
}

fn nested_cell_paragraphs_mut<'a>(
    core: &'a mut DocumentCore,
    para_idx: usize,
    path: &[(usize, usize, usize)],
) -> &'a mut Vec<Paragraph> {
    let Control::Table(outer_table) =
        &mut core.document_mut().sections[0].paragraphs[para_idx].controls[path[0].0]
    else {
        panic!("바깥 표가 아님");
    };
    let outer_cell_para = &mut outer_table.cells[path[0].1].paragraphs[path[0].2];
    let Control::Table(inner_table) = &mut outer_cell_para.controls[path[1].0] else {
        panic!("안쪽 표가 아님");
    };
    &mut inner_table.cells[path[1].1].paragraphs
}

/// 진짜 깊이 2 by-path 경로도 같은 규약을 따른다.
#[test]
fn nested_cell_merge_undo_restores_removed_paragraph_meta() {
    let (mut core, para_idx, first_path) = doc_with_nested_table();

    core.insert_text_in_cell_by_path(0, para_idx, &first_path, 0, "첫째")
        .expect("깊이 2 첫 문단");
    core.split_paragraph_in_cell_by_path(0, para_idx, &first_path, 2, None)
        .expect("깊이 2 by-path 분할");

    let mut second_path = first_path.clone();
    second_path.last_mut().expect("안쪽 경로").2 = 1;
    core.insert_text_in_cell_by_path(0, para_idx, &second_path, 0, "둘째")
        .expect("깊이 2 둘째 문단");

    {
        let paragraphs = nested_cell_paragraphs_mut(&mut core, para_idx, &first_path);
        paragraphs[0].para_shape_id = 10;
        stamp_meta(&mut paragraphs[1]);
    }

    let merged = core
        .merge_paragraph_in_cell_by_path(0, para_idx, &second_path)
        .expect("깊이 2 by-path 병합");
    let meta = removed_meta(&merged);

    core.split_paragraph_in_cell_by_path(0, para_idx, &first_path, 2, Some(meta))
        .expect("깊이 2 undo 분할");

    let paragraphs = nested_cell_paragraphs(&core, para_idx, &first_path);
    assert_eq!(paragraphs[1].text, "둘째");
    assert_meta_restored(&paragraphs[1]);
    assert_eq!(paragraphs[0].para_shape_id, 10, "앞 문단은 그대로");
}

/// 메타를 넘기지 않으면 기존 Enter 분할 시맨틱 그대로다 — 앞 문단에서 상속받는다.
#[test]
fn cell_split_without_meta_keeps_enter_inheritance() {
    let (mut core, para_idx) = doc_with_table();
    let ctrl_idx = table_control_idx(&core, para_idx);

    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 0, "첫째둘째")
        .expect("문단");
    control_paragraphs_mut(&mut core, para_idx, ctrl_idx, 0)[0].para_shape_id = 10;

    core.split_paragraph_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 2, None)
        .expect("Enter 분할");

    let paragraphs = control_paragraphs(&core, para_idx, ctrl_idx, 0);
    assert_eq!(
        paragraphs[1].para_shape_id, 10,
        "Enter 분할은 앞 문단을 상속한다"
    );
}
