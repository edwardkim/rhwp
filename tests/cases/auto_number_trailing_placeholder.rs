//! 자동 번호는 문단 글에 자리표 공백 한 글자를 남기고, 그 글자가 번호 컨트롤의 8유닛을 함께
//! 차지한다. 문단 끝 번호 뒤에서 글을 넣거나 문단을 나누거나 합쳐도 자리표와 번호가 갈리지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;

/// 첫 문단 / 가나다[쪽 번호] / 끝 문단 — HWPX 로 저장해 다시 연 꼴(파서가 만든 자리표).
fn number_doc() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "첫 문단끝 문단").unwrap();
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.insert_text_native(0, 1, 0, "가나다").unwrap();
    core.insert_auto_number_at_cursor(0, 1, 3, "page").unwrap();
    let core = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(
        paragraphs(&core),
        [
            ("첫 문단".to_string(), vec![]),
            ("가나다 ".to_string(), vec![4]),
            ("끝 문단".to_string(), vec![]),
        ]
    );
    core
}

/// 문단마다 (글, 자동 번호의 글자 위치).
fn paragraphs(core: &DocumentCore) -> Vec<(String, Vec<usize>)> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .map(|para| {
            let numbers = para
                .controls
                .iter()
                .zip(para.control_text_positions())
                .filter(|(control, _)| matches!(control, Control::AutoNumber(_)))
                .map(|(_, at)| at)
                .collect();
            (para.text.clone(), numbers)
        })
        .collect()
}

/// HWP·HWPX 저장본을 다시 열면 문단이 `expected` 다.
fn assert_saved(core: &DocumentCore, expected: &[(&str, &[usize])]) {
    let expected: Vec<(String, Vec<usize>)> = expected
        .iter()
        .map(|(text, numbers)| (text.to_string(), numbers.to_vec()))
        .collect();
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(
            paragraphs(&DocumentCore::from_bytes(&bytes).unwrap()),
            expected
        );
    }
}

#[test]
fn typing_after_a_trailing_auto_number_keeps_the_number_in_place() {
    let mut core = number_doc();
    core.insert_text_native(0, 1, 4, "X").unwrap();
    assert_saved(
        &core,
        &[("첫 문단", &[]), ("가나다 X", &[3]), ("끝 문단", &[])],
    );

    let mut core = number_doc();
    let result = core.paste_html_native(0, 1, 4, "<p>XY</p>").unwrap();
    assert!(result.contains("\"charOffset\":6"), "{result}");
    assert_saved(
        &core,
        &[("첫 문단", &[]), ("가나다 XY", &[3]), ("끝 문단", &[])],
    );
}

#[test]
fn enter_after_a_trailing_auto_number_leaves_the_number_in_its_paragraph() {
    let mut core = number_doc();
    core.split_paragraph_native(0, 1, 4, None).unwrap();
    assert_saved(
        &core,
        &[
            ("첫 문단", &[]),
            ("가나다 ", &[4]),
            ("", &[]),
            ("끝 문단", &[]),
        ],
    );
}

#[test]
fn typing_after_an_auto_number_merged_with_the_next_paragraph_keeps_the_number() {
    let mut core = number_doc();
    core.merge_paragraph_native(0, 2).unwrap();
    core.insert_text_native(0, 1, 4, "X").unwrap();
    assert_saved(&core, &[("첫 문단", &[]), ("가나다 X끝 문단", &[3])]);
}

/// `number_doc` 의 둘째 문단 1번 글자 앞에 빈 누름틀을 둔 꼴. 끝 자동 번호가 있는 문단에는
/// 누름틀 넣기가 원시 슬롯을 세지 못해 거절하므로, 누름틀을 먼저 넣고 번호를 모델에 붙인다.
fn field_and_number_doc() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "첫 문단끝 문단").unwrap();
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.split_paragraph_native(0, 0, 4, None).unwrap();
    core.insert_text_native(0, 1, 0, "가나다").unwrap();
    core.insert_click_here_field_at(0, 1, 1, "안내문", "메모", "빈칸", true)
        .unwrap();
    let para = &mut core.document_mut().sections[0].paragraphs[1];
    let end = para.char_offsets.last().unwrap() + 1;
    para.text.push(' ');
    para.char_offsets.push(end);
    para.controls.push(Control::AutoNumber(Default::default()));
    para.ctrl_data_records.push(None);
    para.char_count += 8;
    let core = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(fields(&core), [(1, 1)]);
    assert_eq!(paragraphs(&core)[1], ("가나다 ".to_string(), vec![4]));
    core
}

/// 둘째 문단 누름틀의 (시작, 끝) 글자 위치.
fn fields(core: &DocumentCore) -> Vec<(usize, usize)> {
    core.document().sections[0].paragraphs[1]
        .field_ranges
        .iter()
        .map(|range| (range.start_char_idx, range.end_char_idx))
        .collect()
}

#[test]
fn typing_after_a_trailing_auto_number_in_a_click_here_paragraph_keeps_both() {
    let mut core = field_and_number_doc();
    core.insert_text_native(0, 1, 4, "X").unwrap();
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(fields(&DocumentCore::from_bytes(&bytes).unwrap()), [(1, 1)]);
    }
    assert_saved(
        &core,
        &[("첫 문단", &[]), ("가나다 X", &[3]), ("끝 문단", &[])],
    );
}
