//! [Issue #6156] `DOCUMENT_PROPERTIES.section_count` 가 실제 방출 구역 수에서
//! 유도되지 않아 한글이 문서를 손상으로 판정한다.
//!
//! 한글은 이 값을 `BodyText/SectionN` 탐색의 상한으로 읽는다. 선언값이 실제 스트림
//! 수보다 크면 없는 구역을 찾다가 손상 판정을 내고 `forceopen` 으로도 열리지 않는다
//! (#6125 재현물 실측: 선언 2 / 스트림 1 → 한컴 COM `Open` 실패).
//!
//! 종전 직렬화기는 `section_bytes_list.len() != doc.sections.len()` 일 때만 보정해
//! #5142 분할로 스트림이 **늘어난** 경우만 잡았다. 입력 IR 이 이미 어긋난 경우
//! (선언 2 / IR 구역 1)는 두 값이 같아 보정이 발동하지 않고, HWP5 네이티브 경로는
//! `DOCUMENT_PROPERTIES` 를 raw 로 통과시키므로 불일치가 왕복해도 그대로 남았다.
//!
//! 계약: 저장 산출물의 선언 구역 수 == 실제 방출한 `BodyText/SectionN` 스트림 수.
//! 입력 모델이 무엇을 선언했든, 어느 경로(스트림 raw 통과 · 레코드 raw 통과 ·
//! 모델 writer)를 타든 같다.

use rhwp::model::document::Document;
use rhwp::model::paragraph::Paragraph;
use rhwp::parser::parse_document;
use rhwp::serializer::cfb_writer::serialize_hwp;

/// 구역이 둘 이상인 저장소 표본 — 구역을 덜어내 불일치를 만든다.
const MULTI_SECTION_SAMPLE: &str = "samples/2026_oss_rst.hwp";

fn roundtrip(doc: &Document) -> Document {
    let bytes = serialize_hwp(doc).expect("serialize");
    parse_document(&bytes).expect("reparse")
}

/// 선언값이 실제보다 큰 입력(#6125 재현물 형상) — raw 통과 경로.
///
/// 원본을 파싱하면 `DocInfo.raw_stream` 과 `DOCUMENT_PROPERTIES.raw_data` 가 봉인된
/// 채로 실려 온다. 구역만 덜어내면 모델의 선언값(2)은 그대로이고 스트림은 1개가
/// 되므로, 보정이 없으면 원본 선언값이 그대로 통과해 한글이 손상 판정을 낸다.
#[test]
fn issue_6156_declared_count_over_emitted_streams_is_corrected() {
    let bytes = std::fs::read(MULTI_SECTION_SAMPLE).expect("표본 읽기");
    let mut doc = parse_document(&bytes).expect("parse");
    assert!(
        doc.sections.len() >= 2,
        "표본은 구역이 둘 이상이어야 한다: {}",
        MULTI_SECTION_SAMPLE
    );
    let declared_before = doc.doc_properties.section_count;
    assert_eq!(
        declared_before as usize,
        doc.sections.len(),
        "표본 자체는 선언/실제가 일치해야 한다"
    );

    // #6125 재현물과 같은 형상: 구역 하나만 남기고 선언값은 건드리지 않는다.
    doc.sections.truncate(1);
    assert_eq!(doc.doc_properties.section_count, declared_before);

    let saved = roundtrip(&doc);
    assert_eq!(saved.sections.len(), 1, "구역 스트림은 1개여야 한다");
    assert_eq!(
        saved.doc_properties.section_count, 1,
        "선언 구역 수가 실제 방출 스트림 수(1)에서 유도되지 않았다 — 한글 손상 판정 형상"
    );
}

/// 모델 writer 경로 — raw 캐시 없는 합성 IR 도 같은 계약을 지킨다.
#[test]
fn issue_6156_synthetic_ir_declared_count_follows_streams() {
    let mut doc = Document::default();
    doc.sections.push(rhwp::model::document::Section::default());
    doc.sections[0].paragraphs.push(Paragraph::default());
    // 실제 구역은 1개인데 선언만 3.
    doc.doc_properties.section_count = 3;
    doc.doc_properties.raw_data = None;

    let saved = roundtrip(&doc);
    assert_eq!(saved.sections.len(), 1);
    assert_eq!(saved.doc_properties.section_count, 1);
}

/// 정상 문서는 값이 바뀌지 않는다 — 보정이 멀쩡한 왕복을 흔들지 않는지.
#[test]
fn issue_6156_well_formed_document_keeps_its_count() {
    let bytes = std::fs::read(MULTI_SECTION_SAMPLE).expect("표본 읽기");
    let doc = parse_document(&bytes).expect("parse");
    let declared = doc.doc_properties.section_count;

    let saved = roundtrip(&doc);
    assert_eq!(saved.doc_properties.section_count, declared);
    assert_eq!(saved.sections.len(), declared as usize);
}
