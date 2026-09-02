//! [#3521] synam-001 HWPX 내보내기 전후 쪽수가 같아야 한다.
//!
//! 35→36 은 문단 237 TAC 표(treatAsChar, flowWithText=0, wrap=Square)가
//! HWPX 재파싱에서 `table.attr` bit0 을 잃어 블록 RowBreak 로 쪼개지기 때문이다.
//! HWP5-origin HWPX 는 원본 HWP5 와 같이 treatAsChar 만으로 bit0 을 켠다.

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/synam-001.hwp";

#[test]
fn synam001_export_hwpx_preserves_page_count() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let src = HwpDocument::from_bytes(&data).expect("parse source");
    let src_pages = src.page_count();
    assert_eq!(src_pages, 35, "#3521 전제: synam-001 원본 35쪽");

    let bytes = src.export_hwpx_native().expect("export hwpx");
    assert!(
        bytes.len() > 4 && &bytes[0..4] == b"PK\x03\x04",
        "산출물이 ZIP(HWPX) 매직으로 시작해야 한다"
    );

    let rt_ir = rhwp::parse_document(&bytes).expect("parse exported ir");
    assert!(
        rt_ir
            .hwpx_aux_entry(rhwp::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH)
            .is_some(),
        "HWP5→HWPX 마커가 있어야 한다"
    );

    let src_ir = rhwp::parse_document(&data).expect("parse source ir");
    let p237s = &src_ir.sections[0].paragraphs[237];
    let p237r = &rt_ir.sections[0].paragraphs[237];
    let src_table = p237s
        .controls
        .iter()
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t),
            _ => None,
        })
        .expect("문단 237 표");
    let rt_table = p237r
        .controls
        .iter()
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t),
            _ => None,
        })
        .expect("왕복 문단 237 표");
    assert!(src_table.common.treat_as_char, "샘플 전제: 문단 237 TAC");
    assert!(
        !src_table.common.flow_with_text,
        "샘플 전제: 문단 237 flowWithText=0"
    );
    assert_eq!(
        rt_table.attr & 0x01,
        0x01,
        "HWP5-origin HWPX 는 treatAsChar 표의 attr bit0 을 켜야 한다"
    );

    let round = HwpDocument::from_bytes(&bytes).expect("reparse exported hwpx");
    assert_eq!(
        src_pages,
        round.page_count(),
        "#3521: HWP5→HWPX 왕복 후 페이지 수가 달라졌다"
    );
}
