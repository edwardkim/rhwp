//! 템플릿 생성 문서의 PrvText 보정 회귀 테스트 (#2387).
//!
//! `serialize_hwp` 은 `doc.preview` 를 파싱 원본 그대로 실어 낸다(원본 보존 우선).
//! 내장 템플릿(blank2010)으로 만든 새 문서는 템플릿 placeholder 미리보기("\r\n")를
//! 그대로 물고 나가, 내용을 채워도 탐색기·한컴 미리보기에 빈 문서로 보인다.
//!
//! 수정: 원본 PrvText 가 없거나 공백/placeholder 일 때만 본문에서 새로 만든다.
//! 실문서의 원본 미리보기는 건드리지 않아 라운드트립 보존을 깨지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::serializer::cfb_writer::serialize_hwp;

fn preview_text(core: &DocumentCore) -> String {
    core.document()
        .preview
        .as_ref()
        .and_then(|p| p.text.as_ref())
        .cloned()
        .unwrap_or_default()
}

/// 템플릿으로 만든 문서를 직렬화하면 PrvText 가 본문을 담아야 한다.
///
/// 보정 전에는 placeholder "\r\n"(2B)만 남아 미리보기가 빈 문서로 보였다.
#[test]
fn generated_document_preview_carries_body_text() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank 템플릿");
    core.insert_text_native(0, 0, 0, "2026년 상반기 사업 보고서")
        .expect("제목");

    let bytes = serialize_hwp(core.document()).expect("직렬화");
    let reparsed = DocumentCore::from_bytes(&bytes).expect("재파싱");

    let prv = preview_text(&reparsed);
    assert!(
        prv.contains("2026년 상반기 사업 보고서"),
        "PrvText 가 본문 제목을 담아야 한다 — 실측: {prv:?}"
    );
}

/// 이미 실재하는 PrvText 는 재직렬화해도 보존된다(원본 보존, 중복 생성 안 함).
#[test]
fn existing_preview_is_preserved_on_reserialize() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank 템플릿");
    core.insert_text_native(0, 0, 0, "제목 텍스트")
        .expect("제목");

    // 1차 직렬화 → PrvText 가 본문으로 보정됨
    let bytes1 = serialize_hwp(core.document()).expect("직렬화1");
    let doc1 = DocumentCore::from_bytes(&bytes1).expect("재파싱1");
    let prv1 = preview_text(&doc1);
    assert!(prv1.contains("제목 텍스트"), "1차 보정 실패: {prv1:?}");

    // 2차 직렬화 → 실재하는 PrvText 를 건드리지 않고 그대로 유지
    let bytes2 = serialize_hwp(doc1.document()).expect("직렬화2");
    let doc2 = DocumentCore::from_bytes(&bytes2).expect("재파싱2");
    let prv2 = preview_text(&doc2);

    assert_eq!(prv1, prv2, "실재 PrvText 는 재직렬화 시 보존되어야 한다");
}
