//! 누름틀 끝 표지는 컨트롤이 아닌 8유닛 슬롯이다. 그 뒤 컨트롤의 글자 위치를 앞당기지 않는다.

use rhwp::document_core::DocumentCore;

/// "앞뒤끝"의 1번 글자에 "가나"를 채운 누름틀을 넣고(앞가나뒤끝, 범위 1..3),
/// 뒤 다음(4번 글자 앞)에 각주를 넣는다.
fn fixture() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞뒤끝").unwrap();
    core.insert_click_here_field_at(0, 0, 1, "안내문", "메모", "채운 필드", true)
        .unwrap();
    core.set_field_value_by_name("채운 필드", "가나").unwrap();
    core.insert_footnote_native(0, 0, 4).unwrap();
    core
}

fn footnote_position(core: &DocumentCore) -> usize {
    let p = &core.document().sections[0].paragraphs[0];
    assert_eq!(p.text, "앞가나뒤끝");
    let footnote = p
        .controls
        .iter()
        .position(|c| matches!(c, rhwp::model::control::Control::Footnote(_)))
        .unwrap();
    p.control_text_positions()[footnote]
}

#[test]
fn control_after_filled_field_end_keeps_its_text_position() {
    let core = fixture();
    assert_eq!(footnote_position(&core), 4);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(
            footnote_position(&DocumentCore::from_bytes(&bytes).unwrap()),
            4
        );
    }
}
