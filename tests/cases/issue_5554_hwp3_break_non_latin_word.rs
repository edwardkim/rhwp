//! [#5554] HWP3 문단 정렬에서 유도한 `breakNonLatinWord`가 HWPX 내보내기에도 남아야 한다.
//!
//! HWP3 ParaShape에는 이 값을 저장하는 원문 필드가 없다. 한글의 HWP3→HWPX 변환 계약은
//! 양쪽 정렬을 `KEEP_WORD`, 나머지 정렬을 `BREAK_WORD`로 내보낸다. 공개 HWP3 표본을
//! 실제 HWPX 내보내기·재파싱 경로로 통과시켜, 파서 IR의 attr1 bit7이 직렬화에서
//! 사라지지 않는지 고정한다.

use std::fs;
use std::path::Path;

use rhwp::model::style::Alignment;
use rhwp::parse_document;
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/hwp3-sample11.hwp";

#[test]
fn hwp3_alignment_derives_break_non_latin_word_through_hwpx_export() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("{} 읽기: {error}", path.display()));
    let source = parse_document(&bytes).expect("HWP3 파싱");
    let exported = HwpDocument::from_bytes(&bytes)
        .expect("HWP3 열기")
        .export_hwpx_native()
        .expect("HWPX 내보내기");
    let reparsed = parse_document(&exported).expect("내보낸 HWPX 재파싱");

    assert_eq!(
        source.doc_info.para_shapes.len(),
        reparsed.doc_info.para_shapes.len(),
        "HWP3→HWPX 내보내기는 ParaShape ID 개수를 보존해야 한다"
    );

    let mut justify_count = 0usize;
    let mut other_count = 0usize;
    for (index, (source_shape, reparsed_shape)) in source
        .doc_info
        .para_shapes
        .iter()
        .zip(&reparsed.doc_info.para_shapes)
        .enumerate()
        .skip(1)
    {
        let expected_keep_word = source_shape.alignment == Alignment::Justify;
        let actual_keep_word = (reparsed_shape.attr1 >> 7) & 1 != 0;
        if expected_keep_word {
            justify_count += 1;
        } else {
            other_count += 1;
        }
        assert_eq!(
            actual_keep_word, expected_keep_word,
            "ParaShape {index}의 HWP3 정렬 {:?}은 HWPX breakNonLatinWord와 일치해야 한다",
            source_shape.alignment
        );
    }

    assert!(
        justify_count > 0,
        "표본 전제: 양쪽 정렬 ParaShape가 하나 이상"
    );
    assert!(
        other_count > 0,
        "표본 전제: 비양쪽 정렬 ParaShape가 하나 이상"
    );
}
