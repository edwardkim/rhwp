//! 미주는 원본 구역이 아니라 모아 찍는 구역(문서 끝)의 페이지에 그려진다.
//! 구역이 둘 이상인 문서에서도 앞 구역 미주의 편집 대상과 캐럿을 찾아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;

#[test]
fn endnote_in_first_of_two_sections_resolves_edit_target() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "첫째 구역").unwrap();
    let mut doc = core.document().clone();
    let second = doc.sections[0].clone();
    doc.sections.push(second);
    core.set_document(doc);

    let inserted: Value =
        serde_json::from_str(&core.insert_endnote_native(0, 0, 2).unwrap()).unwrap();
    let control = inserted["controlIdx"].as_u64().unwrap() as usize;
    core.insert_text_in_footnote_native(0, 0, control, 0, 2, "미주 본문")
        .unwrap();

    let info: Value = serde_json::from_str(
        &core
            .get_note_edit_info_native(0, 0, control)
            .expect("앞 구역 미주의 편집 대상을 찾아야 합니다"),
    )
    .unwrap();
    assert_eq!(info["kind"], "endnote");
    // 미주는 문서 끝, 곧 둘째 구역의 마지막 페이지에 그려진다.
    assert_eq!(
        info["pageNum"].as_u64().unwrap() as u32,
        core.page_count() - 1
    );

    let rect: Value = serde_json::from_str(
        &core
            .get_cursor_rect_in_note_native(0, 0, control, 0, 2)
            .expect("앞 구역 미주의 캐럿 위치를 찾아야 합니다"),
    )
    .unwrap();
    assert_eq!(rect["pageIndex"], info["pageNum"]);
}
