//! [#3737] hwp3-sample11 HWPX 내보내기 전후 쪽수가 같아야 한다.
//!
//! 151→152 는 변환 뒤 재파싱이 원본 HWP3 와 다른 레이아웃 계약을 타서 생긴다.
//! 원본 HWP3→HWPX 는 `hwp3-origin` 마커만 있고 `hwp5-origin` 이 없다. 이
//! 산출물을 변환본처럼 `hwp3_layout` 으로 보면 `spacing_before *2` 가 한 번
//! 더 들어가고, XML import 가 빈 `line_segs` 를 합성한다. 직파싱 HWP3 와
//! 같이 native HWP3 계약을 쓴다.
//!
//! [#3915] 쪽수 실패 표본을 이 파일로 옮긴 PR(#5434) 은 이 수정 이후
//! 다른 잔존 표본으로 다시 옮겨야 한다. 정상화된 문서를 실패 표본으로
//! 두지 않는다.

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/hwp3-sample11.hwp";

#[test]
fn hwp3_sample11_export_hwpx_preserves_page_count() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let src = HwpDocument::from_bytes(&data).expect("parse source");
    let src_pages = src.page_count();
    assert_eq!(src_pages, 151, "#3737 전제: hwp3-sample11 원본 151쪽");

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
    assert!(
        rt_ir
            .hwpx_aux_entry(rhwp::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH)
            .is_none(),
        "원본 HWP3 HWPX 에는 hwp5-origin 이 없어야 한다"
    );
    let rp = rt_ir.layout_profile();
    assert!(
        !rp.hwp3_layout() && rp.hwp3_native_layout(),
        "원본 HWP3 HWPX 는 변환본 계약이 아니라 native HWP3 계약을 써야 한다"
    );

    let src_ir = rhwp::parse_document(&data).expect("parse source ir");
    for pi in [3701usize, 3702, 3703, 3704] {
        let src_table = src_ir.sections[0].paragraphs[pi]
            .controls
            .iter()
            .find_map(|c| match c {
                rhwp::model::control::Control::Table(t) => Some(t),
                _ => None,
            })
            .unwrap_or_else(|| panic!("샘플 전제: 문단 {pi} 표"));
        assert!(src_table.common.treat_as_char, "샘플 전제: 문단 {pi} TAC");
        let rt_table = rt_ir.sections[0].paragraphs[pi]
            .controls
            .iter()
            .find_map(|c| match c {
                rhwp::model::control::Control::Table(t) => Some(t),
                _ => None,
            })
            .unwrap_or_else(|| panic!("왕복 문단 {pi} 표"));
        assert_eq!(
            rt_table.attr & 0x01,
            0x01,
            "HWP3-origin HWPX 는 treatAsChar 표의 attr bit0 을 켜야 한다 (문단 {pi})"
        );
    }

    let round = HwpDocument::from_bytes(&bytes).expect("reparse exported hwpx");
    assert_eq!(
        src_pages,
        round.page_count(),
        "#3737: HWP3→HWPX 왕복 후 페이지 수가 달라졌다"
    );
}
