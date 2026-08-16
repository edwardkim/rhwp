//! [Issue #3532] HWP3 출처 문단의 본문 끝 zero-width 글자모양 경계(문단 마크
//! 전용 모양)가 HWPX 왕복에서 말미 컨트롤 슬롯 폭만큼 밀리던 계약을 고정한다.
//!
//! mismatch 방출 경로(위치 슬롯 추정 불가 — HWP3 IR)가 말미 컨트롤을 마지막
//! 경계 **앞** run 에 몰아써서, 재파싱 경계가 8유닛×n 만큼 밀렸다
//! (hwp3-sample10 문단 26: (50,80)→(66,80)). 경계를 먼저 끊고 컨트롤을 마지막
//! run 안쪽에 싣는다.

use rhwp::document_core::DocumentCore;
use rhwp::parse_document;

const SAMPLE: &str = "samples/hwp3-sample10.hwp";

#[test]
fn issue3532_trailing_boundary_survives_hwpx_roundtrip() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let original = parse_document(&bytes).expect("parse hwp3");

    // 샘플 전제: 본문 끝 zero-width 경계 + 말미 컨트롤이 공존하는 문단(26).
    let p26 = &original.sections[0].paragraphs[26];
    let text_units = p26.text.chars().count() as u32;
    assert!(
        p26.char_shapes
            .last()
            .is_some_and(|cs| cs.start_pos == text_units),
        "샘플 전제: 마지막 경계가 본문 끝(zero-width)이어야 한다"
    );
    assert!(!p26.controls.is_empty(), "샘플 전제: 말미 컨트롤");

    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let hwpx = core.export_hwpx_native().expect("export");
    let roundtripped = parse_document(&hwpx).expect("reparse");

    for pi in [26usize, 340] {
        let a: Vec<(u32, u32)> = original.sections[0].paragraphs[pi]
            .char_shapes
            .iter()
            .map(|c| (c.start_pos, c.char_shape_id))
            .collect();
        let b: Vec<(u32, u32)> = roundtripped.sections[0].paragraphs[pi]
            .char_shapes
            .iter()
            .map(|c| (c.start_pos, c.char_shape_id))
            .collect();
        assert_eq!(
            a, b,
            "문단 {pi}: 본문 끝 경계가 컨트롤 슬롯만큼 밀리면 안 된다 (#3532)"
        );
    }
}
