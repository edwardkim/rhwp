//! [Issue #4493] 공개 저수준 `parse_document → Document 직접 변경 →
//! serialize_document` 경로에서 DocInfo 변경이 원본 raw 캐시(스트림·레코드)에
//! 가려져 조용히 사라지던 계약을 고정한다.
//!
//! - 무변경 문서는 원본 DocInfo 레코드 스트림 바이트를 정확히 재사용한다.
//! - 공개 모델 직접 변경(char_shape·doc_properties)은 저장·재로드 뒤 유지된다.
//! - 공개 `raw_stream` 을 다른 바이트로 교체해도 기존 봉인으로 승인되지 않는다.
//! - [#4432] &mut 저장 경로는 저장 성공 시 dirty 를 내리고 raw 캐시를 재밀봉한다.

use rhwp::document_core::DocumentCore;
use rhwp::{parse_document, serialize_document};

const SAMPLE: &str = "samples/2026_oss_rst.hwp";

fn sample_bytes() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

#[test]
fn unchanged_parse_serialize_reuses_docinfo_raw_bytes() {
    let doc = parse_document(&sample_bytes()).expect("parse");
    let original_raw = doc
        .doc_info
        .raw_stream
        .clone()
        .expect("HWP5 파싱은 DocInfo raw 캐시를 채운다");
    assert!(
        doc.doc_info.raw_provenance.is_some(),
        "파서가 만든 문서에는 봉인이 있어야 한다 (#4493)"
    );

    let out = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&out).expect("reparse");
    assert_eq!(
        reparsed.doc_info.raw_stream.as_ref(),
        Some(&original_raw),
        "무변경 문서의 DocInfo 레코드 스트림은 바이트 그대로 통과해야 한다"
    );
}

#[test]
fn public_char_shape_mutation_survives_save_reload() {
    let mut doc = parse_document(&sample_bytes()).expect("parse");
    assert!(!doc.doc_info.char_shapes.is_empty());
    let target = 0usize;
    let new_size = doc.doc_info.char_shapes[target].base_size + 700;
    // 공개 모델 직접 변경 — dirty 표식을 세우지 않는다(그게 이 이슈의 재현이다).
    doc.doc_info.char_shapes[target].base_size = new_size;
    assert!(!doc.doc_info.raw_stream_dirty);

    let out = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&out).expect("reparse");
    assert_eq!(
        reparsed.doc_info.char_shapes[target].base_size, new_size,
        "공개 char_shape 직접 변경이 저장·재로드 뒤 유지돼야 한다 (#4493)"
    );
}

#[test]
fn public_doc_properties_mutation_survives_save_reload() {
    let mut doc = parse_document(&sample_bytes()).expect("parse");
    let new_start = doc.doc_properties.page_start_num + 4;
    doc.doc_properties.page_start_num = new_start;
    assert!(!doc.doc_info.raw_stream_dirty);

    let out = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&out).expect("reparse");
    assert_eq!(
        reparsed.doc_properties.page_start_num, new_start,
        "공개 doc_properties 직접 변경이 저장·재로드 뒤 유지돼야 한다 (#4493)"
    );
}

#[test]
fn untouched_records_keep_raw_bytes_when_one_record_changes() {
    // 레코드 봉인의 핵심: 하나가 바뀌어도 나머지 레코드는 원본 바이트를 유지한다
    // (하위 raw 전면 폐기 금지 — 미모델링 바이트 보존).
    let mut doc = parse_document(&sample_bytes()).expect("parse");
    let untouched_raw = doc.doc_info.char_shapes[1].raw_data.clone();
    assert!(
        untouched_raw.is_some(),
        "샘플 전제: 레코드 raw 가 있어야 한다"
    );
    doc.doc_info.char_shapes[0].base_size += 700;

    let out = serialize_document(&doc).expect("serialize");
    let reparsed = parse_document(&out).expect("reparse");
    assert_eq!(
        reparsed.doc_info.char_shapes[1].raw_data, untouched_raw,
        "변경되지 않은 레코드는 원본 바이트가 유지돼야 한다"
    );
}

#[test]
fn swapped_raw_stream_is_not_honored() {
    let mut doc = parse_document(&sample_bytes()).expect("parse");
    let char_shape_count = doc.doc_info.char_shapes.len();
    // 봉인 이후 raw 바이트를 통째로 바꿔치기 — 승인되면 안 된다.
    doc.doc_info.raw_stream = Some(vec![0xDE; 64]);

    let out = serialize_document(&doc).expect("교체 raw 는 무시되고 모델로 재생성돼야 한다");
    let reparsed = parse_document(&out).expect("산출물은 정상 문서여야 한다");
    assert_eq!(
        reparsed.doc_info.char_shapes.len(),
        char_shape_count,
        "산출물은 교체 바이트가 아니라 모델 상태를 담아야 한다 (#4493)"
    );
}

#[test]
fn issue_4432_save_lowers_dirty_and_reseals() {
    let mut core = DocumentCore::from_bytes(&sample_bytes()).expect("open");
    // dirty 를 세우는 통상 편집 경로 — 중앙 무효화 진입점(document_mut) 경유.
    core.document_mut().doc_info.char_shapes[0].base_size += 700;
    core.document_mut().doc_info.raw_stream_dirty = true;

    let first = core.export_hwp_with_adapter().expect("save 1");
    assert!(
        !core.document().doc_info.raw_stream_dirty,
        "저장 성공 지점에서 dirty 가 내려가야 한다 (#4432)"
    );
    let second = core.export_hwp_with_adapter().expect("save 2");
    assert_eq!(first, second, "재밀봉 뒤 무변경 재저장은 같은 바이트");

    // 재밀봉된 캐시가 실제 모델 상태와 정합해야 한다 — 재로드로 검증.
    let reparsed = parse_document(&second).expect("reparse");
    assert_eq!(
        reparsed.doc_info.char_shapes[0].base_size,
        core.document().doc_info.char_shapes[0].base_size
    );
}
