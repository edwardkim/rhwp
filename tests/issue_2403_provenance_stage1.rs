//! [#2403 Stage 1] SourceProvenance / LayoutCompatibilityProfile 등가성 가드.
//!
//! 파서가 확정한 provenance 가 shim(legacy boolean)과 동기이고, layout_profile()
//! 질의가 기존 파생식과 정확히 같은 값을 내는지 포맷별 실샘플로 고정한다.
//! 이 가드가 있어야 2단계(치환 이관)가 behavior 불변임을 등가로 환원할 수 있다.

use rhwp::model::document::{Document, HWP5_ORIGIN_HWPX_MARKER_PATH};
use rhwp::model::provenance::SourceFormat;
use rhwp::parser::parse_document;

fn load(path: &str) -> Document {
    let data = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    parse_document(&data).unwrap_or_else(|e| panic!("{path}: {e:?}"))
}

/// 기존 파생식 (document_core::queries::rendering::paginate_pass 의 원본 식).
fn legacy_hwpx_stored_layout(doc: &Document, container_is_hwpx: bool) -> bool {
    let hwp5_origin = doc.hwpx_aux_entry(HWP5_ORIGIN_HWPX_MARKER_PATH).is_some();
    (container_is_hwpx && !hwp5_origin) || doc.is_hwpx_variant
}

#[test]
fn hwp5_native_provenance() {
    let doc = load("samples/biz_plan.hwp");
    assert_eq!(doc.provenance.format, SourceFormat::Hwp5);
    assert_eq!(doc.provenance.hwp3_lineage, doc.is_hwp3_variant);
    assert_eq!(doc.provenance.hwpx_lineage, doc.is_hwpx_variant);
    assert!(!doc.is_hwp3_variant && !doc.is_hwpx_variant);
    let p = doc.layout_profile();
    assert!(!p.hwp3_layout());
    assert_eq!(
        p.hwpx_stored_layout(),
        legacy_hwpx_stored_layout(&doc, false)
    );
}

#[test]
fn hwpx_native_provenance() {
    let doc = load("samples/issue_2148_degenerate_cell_vpos.hwpx");
    assert_eq!(doc.provenance.format, SourceFormat::Hwpx);
    let p = doc.layout_profile();
    assert_eq!(
        p.hwpx_stored_layout(),
        legacy_hwpx_stored_layout(&doc, true)
    );
    assert!(p.hwpx_stored_layout(), "native HWPX 는 HWPX 저장 시멘틱");
}

#[test]
fn hwp3_native_provenance() {
    let doc = load("samples/hwp3-sample.hwp");
    assert_eq!(doc.provenance.format, SourceFormat::Hwp3);
    assert_eq!(doc.provenance.hwp3_lineage, doc.is_hwp3_variant);
    let p = doc.layout_profile();
    assert_eq!(p.hwp3_layout(), doc.is_hwp3_variant);
    assert!(!p.hwpx_stored_layout());
}

#[test]
fn hml_provenance() {
    let doc = load("samples/hml/aligns.hml");
    assert_eq!(doc.provenance.format, SourceFormat::Hml);
    let p = doc.layout_profile();
    assert!(!p.hwp3_layout() && !p.hwpx_stored_layout());
}

#[test]
fn hwp3_to_hwp5_variant_lineage_sync() {
    // HWP3→HWP5 변환본 휴리스틱이 발동하는 샘플 — shim 과 provenance 동기 검증.
    let doc = load("samples/hwp3-sample-hwp5.hwp");
    assert_eq!(doc.provenance.format, SourceFormat::Hwp5);
    assert_eq!(
        doc.provenance.hwp3_lineage, doc.is_hwp3_variant,
        "hwp3_lineage 는 is_hwp3_variant 쓰기 지점에서 동기되어야 한다"
    );
    assert_eq!(doc.layout_profile().hwp3_layout(), doc.is_hwp3_variant);
}

#[test]
fn hwp5_origin_hwpx_marker_excludes_hwpx_layout() {
    // 마커가 부착된 HWPX 는 HWP5 시멘틱 유지 — 파생식 등가를 합성 케이스로 고정.
    let mut doc = load("samples/issue_2148_degenerate_cell_vpos.hwpx");
    assert!(doc.layout_profile().hwpx_stored_layout());
    doc.hwpx_aux_entries
        .push((HWP5_ORIGIN_HWPX_MARKER_PATH.to_string(), Vec::new()));
    assert!(
        !doc.layout_profile().hwpx_stored_layout(),
        "HWP5→HWPX 산출물 마커는 HWPX 전용 분기에서 제외 (세션 중 부착 반영)"
    );
    assert_eq!(
        doc.layout_profile().hwpx_stored_layout(),
        legacy_hwpx_stored_layout(&doc, true)
    );
}
