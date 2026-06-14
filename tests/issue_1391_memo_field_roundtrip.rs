//! Task #1391 통합 — MEMO 필드 parameters + subList(본문) roundtrip 보존.

use rhwp::model::control::{Control, FieldType};
use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::serialize_hwpx;

fn collect_fields(doc: &rhwp::model::document::Document) -> Vec<(bool, usize, String)> {
    // (parameters 보유, memo 문단 수, 첫 memo 텍스트)
    let mut out = Vec::new();
    for s in &doc.sections {
        for p in &s.paragraphs {
            for c in &p.controls {
                if let Control::Field(f) = c {
                    let memo_text = f
                        .memo_paragraphs
                        .first()
                        .map(|q| q.text.clone())
                        .unwrap_or_default();
                    out.push((
                        f.raw_parameters_xml.is_some(),
                        f.memo_paragraphs.len(),
                        memo_text,
                    ));
                }
            }
        }
    }
    out.sort();
    out
}

#[test]
fn aift_memo_parameters_and_body_roundtrips() {
    let bytes = std::fs::read("samples/hwpx/aift.hwpx").expect("샘플 읽기");
    let doc1 = parse_hwpx(&bytes).expect("parse 원본");
    let fields = collect_fields(&doc1);

    // MEMO 2건이 본문 + parameters 보유
    let memo_count = doc1
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter(|c| matches!(c, Control::Field(f) if f.field_type == FieldType::Memo))
        .count();
    assert_eq!(memo_count, 2, "aift MEMO 2건");
    assert!(
        fields
            .iter()
            .any(|(_, n, t)| *n > 0 && t.contains("기업 소개")),
        "원본 MEMO 본문 적재 확인: {fields:?}"
    );

    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");
    assert_eq!(fields, collect_fields(&doc2), "round1 필드 멀티셋 보존");

    let out2 = serialize_hwpx(&doc2).expect("serialize r2");
    let doc3 = parse_hwpx(&out2).expect("reparse r2");
    assert_eq!(collect_fields(&doc2), collect_fields(&doc3), "2-round 안정");
}

#[test]
fn hyperlink_field_parameters_roundtrips() {
    // 비-MEMO(HYPERLINK 등)도 parameters verbatim 보존 — 143E
    let bytes = std::fs::read("samples/hwpx/143E433F503322BD33.hwpx").expect("샘플 읽기");
    let doc1 = parse_hwpx(&bytes).expect("parse 원본");
    let with_params = collect_fields(&doc1)
        .iter()
        .filter(|(has, _, _)| *has)
        .count();
    assert!(with_params >= 1, "parameters 보유 필드 존재");

    let out = serialize_hwpx(&doc1).expect("serialize");
    let doc2 = parse_hwpx(&out).expect("reparse");
    assert_eq!(
        collect_fields(&doc1),
        collect_fields(&doc2),
        "필드 parameters 보존"
    );
}
