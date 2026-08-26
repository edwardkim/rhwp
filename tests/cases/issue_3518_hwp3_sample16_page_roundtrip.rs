//! [#3518] hwp3-sample16 HWPX 내보내기 전후 쪽수가 같아야 한다.
//!
//! 64→65 는 변환 뒤 재파싱이 원본 HWP3 와 다른 레이아웃 계약을 타서 생긴다.
//! - 원본 HWP3→HWPX 는 native HWP3 계약을 쓴다 (변환본 hwp3_layout/*2 금지).
//! - HWP3 U+FFFC 자리 개체에 HWPX 8유닛 슬롯을 겹쳐 쌓지 않는다.
//! - treatAsChar 표는 `table.attr` bit0 을 켜 레거시 TAC 판정이 블록 표로
//!   빠지지 않게 한다 (문단 394).
//! - mismatch 경로라도 본문 좌표가 연속이면 저장 line_segs 를 유지한다.

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/hwp3-sample16.hwp";

#[test]
fn hwp3_sample16_export_hwpx_preserves_page_count() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let src = HwpDocument::from_bytes(&data).expect("parse source");
    let src_pages = src.page_count();
    assert_eq!(src_pages, 64, "#3518 전제: hwp3-sample16 원본 64쪽");

    let bytes = src.export_hwpx_native().expect("export hwpx");
    assert!(
        bytes.len() > 4 && &bytes[0..4] == b"PK\x03\x04",
        "산출물이 ZIP(HWPX) 매직으로 시작해야 한다"
    );

    let rt_ir = rhwp::parse_document(&bytes).expect("parse exported ir");
    assert!(
        rt_ir
            .hwpx_aux_entry(rhwp::model::document::HWP3_ORIGIN_HWPX_MARKER_PATH)
            .is_some(),
        "HWP3→HWPX 마커가 있어야 한다"
    );
    let rp = rt_ir.layout_profile();
    assert!(
        !rp.hwp3_layout() && rp.hwp3_native_layout(),
        "원본 HWP3 HWPX 는 변환본 계약이 아니라 native HWP3 계약을 써야 한다"
    );

    let src_ir = rhwp::parse_document(&data).expect("parse source ir");
    let p394s = &src_ir.sections[0].paragraphs[394];
    let p394r = &rt_ir.sections[0].paragraphs[394];
    assert_eq!(
        p394s.char_count, p394r.char_count,
        "문단 394 char_count 가 8유닛 슬롯만큼 부풀면 TAC 표가 블록으로 빠진다"
    );
    for c in &p394r.controls {
        if let rhwp::model::control::Control::Table(t) = c {
            assert_eq!(
                t.attr & 0x01,
                0x01,
                "treatAsChar 표는 legacy attr bit0 이 켜져야 한다"
            );
        }
    }

    let round = HwpDocument::from_bytes(&bytes).expect("reparse exported hwpx");
    assert_eq!(
        src_pages,
        round.page_count(),
        "#3518: HWP3→HWPX 왕복 후 페이지 수가 달라졌다"
    );
}

#[test]
fn hwp3_sample16_heading_keeps_stored_lineseg_on_hwpx_roundtrip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let original = rhwp::parse_document(&data).expect("parse hwp3");
    let p70 = &original.sections[0].paragraphs[70];
    assert_eq!(p70.text, "1.추진목적", "샘플 전제: 문단 70 제목");
    assert_eq!(p70.line_segs.len(), 1, "샘플 전제: 저장 line_seg 1개");

    let core = rhwp::document_core::DocumentCore::from_bytes(&data).expect("core");
    let hwpx = core.export_hwpx_native().expect("export");
    let roundtripped = rhwp::parse_document(&hwpx).expect("reparse");
    let r70 = &roundtripped.sections[0].paragraphs[70];
    assert_eq!(
        r70.line_segs.len(),
        1,
        "#3518: 문단 70 저장 line_seg 가 HWPX 왕복에서 빠지지 않아야 한다"
    );
    assert_eq!(
        r70.line_segs[0].line_height, p70.line_segs[0].line_height,
        "문단 70 line_height 보존"
    );
}
