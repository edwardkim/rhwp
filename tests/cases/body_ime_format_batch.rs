//! 본문 조합의 글 교체·서식 적용도 명시한 배치 안에서는 중간 쪽 나눔을 확정하지 않는다.
//! 글 교체의 기존 서식 상속은 유지하며, 조합 서식은 호출자가 명시적으로 다시 적용한다.
use rhwp::document_core::DocumentCore;
use serde_json::Value;

fn document() -> DocumentCore {
    let mut doc = DocumentCore::new_empty();
    doc.create_blank_document_native().unwrap();
    for para in 0..30 {
        if para > 0 {
            doc.split_paragraph_native(0, para - 1, 3, None).unwrap();
        }
        doc.insert_text_native(0, para, 0, "앞ㅎ뒤").unwrap();
    }
    doc
}

fn format(doc: &mut DocumentCore, para: usize) {
    doc.apply_char_format_native(0, para, 1, 2, r#"{"fontSize":3000}"#)
        .unwrap();
}

fn replace(doc: &mut DocumentCore, para: usize, text: &str) {
    doc.replace_body_text_local_native(0, para, 1, 1, text)
        .unwrap();
}

fn size(doc: &DocumentCore, para: usize, offset: usize) -> u64 {
    let props: Value =
        serde_json::from_str(&doc.get_char_properties_at_native(0, para, offset).unwrap()).unwrap();
    props["fontSize"].as_u64().unwrap()
}

fn layout(doc: &DocumentCore) -> (Vec<String>, Vec<String>) {
    (
        (0..doc.page_count())
            .map(|page| doc.get_page_info_native(page).unwrap())
            .collect(),
        (0..30)
            .map(|para| doc.get_cursor_rect_native(0, para, 2).unwrap())
            .collect(),
    )
}

#[test]
fn body_char_format_defers_pagination_until_end_batch() {
    let mut eager = document();
    let mut batched = document();
    let before = batched.page_count();
    batched.begin_batch_native().unwrap();
    for para in 0..30 {
        format(&mut eager, para);
        format(&mut batched, para);
    }
    assert!(
        eager.page_count() > before,
        "서식으로 실제 쪽 나눔이 바뀌는 대조군"
    );
    assert_eq!(
        batched.page_count(),
        before,
        "applyCharFormat은 endBatch 전에 쪽 나눔을 확정하면 안 된다"
    );
    batched.end_batch_native().unwrap();
    assert_eq!(layout(&batched), layout(&eager));
}

#[test]
fn local_body_replace_defers_pagination_until_end_batch() {
    let mut eager = document();
    let mut batched = document();
    for para in 0..30 {
        format(&mut eager, para);
        format(&mut batched, para);
    }
    let before = batched.page_count();
    batched.begin_batch_native().unwrap();
    for para in 0..30 {
        replace(&mut eager, para, "하");
        replace(&mut batched, para, "하");
    }
    // 삭제한 런 대신 원문 서식을 상속하는 것은 기존 API의 계약이다.
    assert_eq!(size(&batched, 0, 1), 1000);
    assert!(
        eager.page_count() < before,
        "교체로 실제 쪽 나눔이 바뀌는 대조군"
    );
    assert_eq!(
        batched.page_count(),
        before,
        "replaceBodyTextLocal은 endBatch 전에 쪽 나눔을 확정하면 안 된다"
    );
    batched.end_batch_native().unwrap();
    assert_eq!(layout(&batched), layout(&eager));
}

#[test]
fn explicit_composition_format_preserves_text_caret_and_roundtrip_after_batch() {
    let mut eager = document();
    let mut batched = document();
    let before = batched.save_snapshot_native();
    for text in ["ㅎ", "하", "한"] {
        replace(&mut eager, 15, text);
        format(&mut eager, 15);
        batched.begin_batch_native().unwrap();
        replace(&mut batched, 15, text);
        format(&mut batched, 15);
        batched.end_batch_native().unwrap();
        // 각 조합을 마친 즉시 조회하는 캐럿과 쪽 나눔은 비배치 결과와 같아야 한다.
        assert_eq!(layout(&batched), layout(&eager));
        for doc in [&eager, &batched] {
            assert_eq!(
                doc.document().sections[0].paragraphs[15].text,
                format!("앞{text}뒤")
            );
            assert_eq!(
                [size(doc, 15, 0), size(doc, 15, 1), size(doc, 15, 2)],
                [1000, 3000, 1000]
            );
            assert_eq!(doc.document().sections[0].paragraphs[14].text, "앞ㅎ뒤");
            assert_eq!(doc.document().sections[0].paragraphs[16].text, "앞ㅎ뒤");
        }
    }
    let after = batched.save_snapshot_native();
    for bytes in [
        batched.export_hwp_native().unwrap(),
        batched.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            reopened.document().sections[0].paragraphs[15].text,
            "앞한뒤"
        );
        assert_eq!(
            [
                size(&reopened, 15, 0),
                size(&reopened, 15, 1),
                size(&reopened, 15, 2)
            ],
            [1000, 3000, 1000]
        );
    }
    batched.restore_snapshot_native(before).unwrap();
    assert_eq!(batched.document().sections[0].paragraphs[15].text, "앞ㅎ뒤");
    assert_eq!(size(&batched, 15, 1), 1000);
    batched.restore_snapshot_native(after).unwrap();
    assert_eq!(layout(&batched), layout(&eager));
    batched.discard_snapshot_native(before);
    batched.discard_snapshot_native(after);
}
