//! Issue #1770: rhwp HWPX→HWP 변환본의 pagination 자기정합 — HWPX-origin 마커.
//!
//! rhwp 의 변환은 LINE_SEG 를 verbatim 직렬화하므로 산출 HWP5 의 IR 은 HWPX 시멘틱
//! 그대로다(2953495: ir-diff 0). 그러나 재파스 시 `is_hwpx_source=false` 가 되어
//! RowBreak 분할 tolerance(2.0 vs 64.0px) 등 소스 분기가 갈려 같은 IR 이 다른
//! 쪽수(4→5쪽)로 벌어졌다. 변환 시 `/RhwpHwpxOrigin` 마커 스트림을 심고 파서가
//! `Document::is_hwpx_variant` 로 감지, pagination/렌더를 HWPX 시멘틱으로 해석한다.
//!
//! fixture: `samples/issue1770_rowsplit_tolerance.hwpx` (정부 행정규칙 2953495,
//! 근로계약서 — 2×1 RowBreak 표 row1≈947px 가 분할 tolerance 경계에 걸리는 문서).
//! 수정 전: HWPX=4쪽 / convert-HWP=5쪽, render-diff p3 STRUCT(628px).
//! 한컴 2022 오라클: 마커 포함 convert-HWP 열림 정상 + 4쪽
//! (`output/poc/task1770_oracle`, 열림 계약 게이트로 확인).

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue1770_rowsplit_tolerance.hwpx";

fn load(path: &str) -> rhwp::wasm_api::HwpDocument {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = fs::read(&p).unwrap_or_else(|e| panic!("read {path}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {path}: {e:?}"))
}

/// 변환-HWP 재파스가 HWPX 원본과 같은 쪽수로 pagination 된다 (자기정합).
#[test]
fn convert_hwp_pagination_matches_hwpx_source() {
    let mut hwpx = load(SAMPLE);
    let hwpx_pages = hwpx.page_count();
    assert_eq!(hwpx_pages, 4, "#1770 전제: HWPX 원본 4쪽");

    let hwp_bytes = hwpx.export_hwp_with_adapter().expect("convert");
    let mut conv = rhwp::wasm_api::HwpDocument::from_bytes(&hwp_bytes).expect("reparse");
    assert_eq!(
        conv.page_count(),
        hwpx_pages,
        "#1770: convert-HWP 쪽수가 HWPX 원본과 다름 (RowBreak tolerance 소스 분기)"
    );
}

/// 변환 산출물에 origin 마커가 있고, 재파스가 이를 감지하며, 2-round 에도
/// 중복 없이 유지된다.
#[test]
fn convert_hwp_carries_origin_marker_idempotently() {
    use rhwp::document_core::converters::hwpx_to_hwp::HWPX_ORIGIN_STREAM_PATH;

    let mut hwpx = load(SAMPLE);
    let round1 = hwpx.export_hwp_with_adapter().expect("convert r1");
    let doc1 = rhwp::parser::parse_hwp(&round1).expect("reparse r1");
    assert!(doc1.is_hwpx_variant, "#1770: 마커 미감지");
    let markers1 = doc1
        .extra_streams
        .iter()
        .filter(|(p, _)| p == HWPX_ORIGIN_STREAM_PATH)
        .count();
    assert_eq!(markers1, 1, "#1770: round1 마커 수");

    // 2-round: 변환본을 다시 직렬화해도 마커 1개 유지 (extra_streams 보존 + idempotent)
    let round2 = rhwp::serializer::serialize_document(&doc1).expect("serialize r2");
    let doc2 = rhwp::parser::parse_hwp(&round2).expect("reparse r2");
    assert!(doc2.is_hwpx_variant, "#1770: round2 마커 미감지");
    let markers2 = doc2
        .extra_streams
        .iter()
        .filter(|(p, _)| p == HWPX_ORIGIN_STREAM_PATH)
        .count();
    assert_eq!(markers2, 1, "#1770: round2 마커 중복/소실");
}

/// native HWP5 는 마커가 없어 불변 (is_hwpx_variant=false).
#[test]
fn native_hwp_is_not_marked() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/rowbreak-problem-pages.hwp");
    let bytes = fs::read(&p).expect("read native");
    let doc = rhwp::parser::parse_hwp(&bytes).expect("parse native");
    assert!(
        !doc.is_hwpx_variant,
        "#1770: native HWP 가 변환본으로 오인됨"
    );
}
