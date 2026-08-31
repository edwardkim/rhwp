//! [Issue #5251] `issue_265.hwp` 를 HWPX 로 저장해 다시 열면 paragraph[0]
//! char_shapes 경계가 유지된다.
//!
//! HWP3 원본은 개체 자리 U+FFFC 를 8유닛으로 세고, pageNum/footer 는
//! PARA_TEXT 슬롯으로 세지 않는다. HWPX 재파싱이 FFFC 를 1유닛·pageNum/footer
//! 를 8유닛으로 세면 경계가 (24,33,38,54) → (17,26,31,63) 으로 밀린다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use std::path::Path;

const SAMPLE: &str = "samples/issue_265.hwp";

fn cs(para: &rhwp::model::paragraph::Paragraph) -> Vec<(u32, u32)> {
    para.char_shapes
        .iter()
        .map(|c| (c.start_pos, c.char_shape_id))
        .collect()
}

#[test]
fn issue_5251_hwpx_roundtrip_keeps_para0_char_shapes() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let orig = DocumentCore::from_bytes(&std::fs::read(path).expect("read")).expect("open");
    let expected = cs(&orig.document().sections[0].paragraphs[0]);
    let hwpx = orig.export_hwpx_native().expect("export");
    let back = DocumentCore::from_bytes(&hwpx).expect("reparse");
    let actual = cs(&back.document().sections[0].paragraphs[0]);
    assert_eq!(
        actual, expected,
        "paragraph[0] char_shapes HWPX 왕복이 밀리면 안 된다"
    );
}
