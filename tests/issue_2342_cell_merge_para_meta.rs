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
use rhwp::model::paragraph::{ColumnBreakType, NumberingRestart, ParaMeta};
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

fn cell_paragraphs<'a>(
    core: &'a DocumentCore,
    para_idx: usize,
    ctrl_idx: usize,
    cell_idx: usize,
) -> &'a [rhwp::model::paragraph::Paragraph] {
    match &core.document().sections[0].paragraphs[para_idx].controls[ctrl_idx] {
        Control::Table(table) => &table.cells[cell_idx].paragraphs,
        other => panic!("표가 아님: {other:?}"),
    }
}

/// 병합 결과 JSON 에서 `removedParaMeta` 를 꺼낸다.
fn removed_meta(result: &str) -> ParaMeta {
    let value: Value = serde_json::from_str(result).expect("merge result json");
    serde_json::from_value(value["removedParaMeta"].clone())
        .unwrap_or_else(|e| panic!("removedParaMeta 가 있어야 한다 ({e}): {result}"))
}

/// 두 번째 문단에 알아볼 수 있는 메타를 심는다.
fn stamp_meta(para: &mut rhwp::model::paragraph::Paragraph) {
    para.para_shape_id = 20;
    para.style_id = 5;
    para.column_type = ColumnBreakType::Page;
    para.raw_break_type = 0x04;
    para.numbering_restart = Some(NumberingRestart::NewStart(7));
    para.raw_header_extra = vec![0, 0, 0, 0, 0, 0, 0xBB, 0xBB, 0xBB, 0xBB];
    para.tab_extended = vec![[100, 0, 0x0200, 0, 0, 0, 9]];
}

fn assert_meta_restored(para: &rhwp::model::paragraph::Paragraph) {
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

/// 표 셀: 병합 → undo(분할)가 사라진 문단의 메타를 되돌린다.
#[test]
fn cell_merge_undo_restores_removed_paragraph_meta() {
    let (mut core, para_idx) = doc_with_table();
    let ctrl_idx = table_control_idx(&core, para_idx);

    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 0, "첫째")
        .expect("첫 문단");
    core.split_paragraph_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 2, None)
        .expect("셀 분할");
    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, 0, 1, 0, "둘째")
        .expect("둘째 문단");

    {
        let Control::Table(table) =
            &mut core.document_mut().sections[0].paragraphs[para_idx].controls[ctrl_idx]
        else {
            panic!("표가 아님");
        };
        table.cells[0].paragraphs[0].para_shape_id = 10;
        stamp_meta(&mut table.cells[0].paragraphs[1]);
    }

    let merged = core
        .merge_paragraph_in_cell_native(0, para_idx, ctrl_idx, 0, 1)
        .expect("셀 병합");
    let meta = removed_meta(&merged);

    core.split_paragraph_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 2, Some(meta))
        .expect("undo 분할");

    let paras = cell_paragraphs(&core, para_idx, ctrl_idx, 0);
    assert_eq!(paras[1].text, "둘째");
    assert_meta_restored(&paras[1]);
    assert_eq!(paras[0].para_shape_id, 10, "앞 문단은 그대로");
}

/// 중첩 by-path 경로도 같은 규약을 따른다.
#[test]
fn nested_cell_merge_undo_restores_removed_paragraph_meta() {
    let (mut core, para_idx) = doc_with_table();
    let ctrl_idx = table_control_idx(&core, para_idx);
    let path = [(ctrl_idx, 0usize, 0usize)];

    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 0, "첫째")
        .expect("첫 문단");
    core.split_paragraph_in_cell_by_path(0, para_idx, &path, 2, None)
        .expect("by-path 분할");
    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, 0, 1, 0, "둘째")
        .expect("둘째 문단");

    {
        let Control::Table(table) =
            &mut core.document_mut().sections[0].paragraphs[para_idx].controls[ctrl_idx]
        else {
            panic!("표가 아님");
        };
        stamp_meta(&mut table.cells[0].paragraphs[1]);
    }

    let merge_path = [(ctrl_idx, 0usize, 1usize)];
    let merged = core
        .merge_paragraph_in_cell_by_path(0, para_idx, &merge_path)
        .expect("by-path 병합");
    let meta = removed_meta(&merged);

    core.split_paragraph_in_cell_by_path(0, para_idx, &path, 2, Some(meta))
        .expect("undo 분할");

    let paras = cell_paragraphs(&core, para_idx, ctrl_idx, 0);
    assert_eq!(paras[1].text, "둘째");
    assert_meta_restored(&paras[1]);
}

/// 메타를 넘기지 않으면 기존 Enter 분할 시맨틱 그대로다 — 앞 문단에서 상속받는다.
#[test]
fn cell_split_without_meta_keeps_enter_inheritance() {
    let (mut core, para_idx) = doc_with_table();
    let ctrl_idx = table_control_idx(&core, para_idx);

    core.insert_text_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 0, "첫째둘째")
        .expect("문단");
    {
        let Control::Table(table) =
            &mut core.document_mut().sections[0].paragraphs[para_idx].controls[ctrl_idx]
        else {
            panic!("표가 아님");
        };
        table.cells[0].paragraphs[0].para_shape_id = 10;
    }

    core.split_paragraph_in_cell_native(0, para_idx, ctrl_idx, 0, 0, 2, None)
        .expect("Enter 분할");

    let paras = cell_paragraphs(&core, para_idx, ctrl_idx, 0);
    assert_eq!(
        paras[1].para_shape_id, 10,
        "Enter 분할은 앞 문단을 상속한다"
    );
}
