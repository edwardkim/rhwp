//! #7444: 복사와 삭제가 같은 선택의 경계 개체를 다르게 다루면 오려 붙일 때 개체가 늘어난다.
//! 아직 수정하지 않은 재현이다. 글자 축 경계에서 복사와 삭제 중 어느 쪽을 바꿀지는 정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

fn document_with_table() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "abc").unwrap();
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    core.create_table_native(0, 1, 0, 1, 2).unwrap();
    core.insert_text_native(0, 2, 0, "xyz").unwrap();
    // 목적지는 개체 없는 빈 문단으로 두어 붙이는 위치의 논리 축 문제와 분리한다.
    core.split_paragraph_native(0, 2, 3, None).unwrap();
    assert_eq!(core.document().sections[0].paragraphs.len(), 4);
    assert_eq!(table_count(&core), 1);
    core
}

fn table_count(core: &DocumentCore) -> usize {
    core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .filter(|control| matches!(control, Control::Table(_)))
        .count()
}

fn cut_and_paste_at_end(
    core: &mut DocumentCore,
    start_para: usize,
    start_offset: usize,
    end_para: usize,
    end_offset: usize,
) -> usize {
    core.copy_selection_native(0, start_para, start_offset, end_para, end_offset)
        .unwrap();
    core.delete_range_native(0, start_para, start_offset, end_para, end_offset, None)
        .unwrap();
    let after_delete = table_count(core);
    let paragraphs = &core.document().sections[0].paragraphs;
    let last = paragraphs.len() - 1;
    assert!(paragraphs[last].text.is_empty());
    assert!(paragraphs[last].controls.is_empty());
    core.paste_internal_native(0, last, 0).unwrap();
    after_delete
}

#[test]
fn cut_with_table_in_selection_interior_moves_one_table() {
    let mut core = document_with_table();
    let after_delete = cut_and_paste_at_end(&mut core, 0, 1, 2, 2);
    assert_eq!(
        after_delete, 0,
        "선택 중간의 표는 오려 두면 원문에서 사라진다"
    );
    assert_eq!(table_count(&core), 1, "한 번 오려 붙여도 표는 하나다");
}

#[test]
fn cut_ending_at_table_paragraph_start_does_not_duplicate_table() {
    let mut core = document_with_table();
    let after_delete = cut_and_paste_at_end(&mut core, 0, 1, 1, 0);
    assert_eq!(
        table_count(&core),
        1,
        "같은 선택을 오려 붙이면 표 수가 보존돼야 한다 (삭제 직후: {after_delete})"
    );
}

#[test]
fn cut_starting_at_table_paragraph_start_does_not_duplicate_table() {
    let mut core = document_with_table();
    let after_delete = cut_and_paste_at_end(&mut core, 1, 0, 2, 2);
    assert_eq!(
        table_count(&core),
        1,
        "같은 선택을 오려 붙이면 표 수가 보존돼야 한다 (삭제 직후: {after_delete})"
    );
}
