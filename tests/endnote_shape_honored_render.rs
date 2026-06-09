//! Regression: the renderer HONORS the section endnote_shape for a plain (non-exam)
//! endnote — both PLACEMENT (document end on its own new last page) and NUMBER FORMAT
//! (endnote_shape.number_format drives the note marker, e.g. LowerRoman → "i").
//!
//! [FIX 3] A canonical (default) endnote is rendered with the NUMBER ONLY — no
//! footnote-style horizontal separator above it (Hancom does not draw the standard
//! default endnote separator; a deliberately customized one is honored, see
//! issue_1139_inline_picture_duplicate for the exam green-rule case).
//!
//! Guards against reverting to the previous behavior where the endnote flowed inline
//! right after the body and the number was a hardcoded "1)" digit regardless of shape.
//!
//!   cargo test --release --test endnote_shape_honored_render -- --nocapture

use rhwp::wasm_api::HwpDocument;

/// Build a one-paragraph body with real text and a single plain endnote, so the
/// document-end placement applies (body has substantive content; endnote has no
/// decoration letter → not an exam "문N)" label).
fn doc_with_body_and_plain_endnote() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native()
        .expect("create blank document");
    // Real body text so body_has_content gate fires (a blank-only doc keeps the
    // endnote on the same page; with content it moves to a new last page).
    doc.insert_text_native(0, 0, 0, "본문 내용이 한 줄 있는 문서입니다.")
        .expect("insert body text");
    // Plain endnote (before_decoration_letter == 0) at the end of the body.
    doc.insert_endnote_native(0, 0, 5).expect("insert endnote");
    doc
}

#[test]
fn plain_endnote_goes_to_new_last_page_and_honors_number_format() {
    // --- Digit (default) shape: endnote on its own last page, marker "1)" ---
    let mut doc = doc_with_body_and_plain_endnote();
    let pages_default = doc.page_count();
    assert!(
        pages_default >= 2,
        "본문이 있는 문서의 일반 미주는 새 마지막 쪽으로 가야 함(쪽 수 >= 2), got {}",
        pages_default
    );
    let last = pages_default - 1;
    let last_dump = doc.dump_page_items(Some(last));
    let page0_dump = doc.dump_page_items(Some(0));
    // [FIX 3] canonical(기본) 미주는 번호("1)")만 그리고 그 위에 가로 구분선을
    // 그리지 않는다 (각주와 다른 점). blank2010 템플릿의 기본 미주 모양은 각주와
    // 똑같은 가는 검정 실선 구분선 필드를 갖지만, 한컴은 미주에 그것을 렌더하지
    // 않으므로 우리도 숨긴다. (의도적으로 커스터마이즈한 구분선 — 시험지 초록 굵은
    // 줄 등 — 은 issue_1139 가 별도로 honor 보장.)
    assert!(
        last_dump.contains("[미주]"),
        "미주 문단은 마지막 쪽에 있어야 함.\n{last_dump}"
    );
    assert!(
        !last_dump.contains("EndnoteSeparator"),
        "canonical 기본 미주는 구분선을 그리면 안 됨(번호만).\n{last_dump}"
    );
    assert!(
        !page0_dump.contains("[미주]"),
        "미주는 본문 쪽(0)에 인라인으로 남으면 안 됨.\n{page0_dump}"
    );
    assert!(
        last_dump.contains("1)"),
        "기본(digit) 미주 모양은 '1)'로 렌더돼야 함.\n{last_dump}"
    );

    // --- LowerRoman shape: same placement, marker now "i)" ---
    doc.apply_endnote_shape_native(0, "{\"numberFormat\":\"lowerRoman\"}")
        .expect("apply lowerRoman endnote shape");
    let pages_roman = doc.page_count();
    assert!(
        pages_roman >= 2,
        "LowerRoman 적용 후에도 미주는 새 마지막 쪽"
    );
    let last_roman = doc.dump_page_items(Some(pages_roman - 1));
    assert!(
        last_roman.contains("i)"),
        "LowerRoman 미주 모양은 'i)'로 렌더돼야 함(서식 권위 = endnote_shape).\n{last_roman}"
    );
    assert!(
        !last_roman.contains("1)"),
        "LowerRoman 적용 시 하드코딩 '1)'이 남으면 안 됨.\n{last_roman}"
    );
}
