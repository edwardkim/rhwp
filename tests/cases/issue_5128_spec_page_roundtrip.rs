//! [#5128] 한글문서파일형식_5.0_revision1.3 HWPX 왕복 쪽수가 같아야 한다.
//!
//! 기계 판정: `검증 실패(--verify-pages): 변환 전 69쪽, 재파싱 후 68쪽`.
//! `--verify` IR 은 통과한다. 원인은 레이아웃 프로필이다.
//!
//! - 원본 HWP5: `native_hwp5_layout()==true`
//! - HWP5-origin HWPX: `native_hwp5_layout()==false`, `hwp5_stored_pagination_layout()==true`
//!
//! 수정 전 첫 갈림은 p015/p016. 원본은 문단 84를 PartialParagraph 로 나누고
//! 다음 쪽을 연다. HWPX 재파싱은 TAC 그림 앞 저장 reset 분할과 RowBreak 표
//! 통째 fit 을 native 전용으로 막아 한 쪽이 줄어든다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH;
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/한글문서파일형식_5.0_revision1.3.hwp";
const EXPECTED_PAGES: u32 = 69;
const EXPECTED_SECTIONS: usize = 6;
const EXPECTED_PARAS: usize = 619;

fn sample_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn load_bytes() -> Vec<u8> {
    let path = sample_path();
    fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn first_item(core: &DocumentCore, page: u32) -> (String, u64) {
    let pages = core.dump_page_items_json(Some(page));
    let Some(page_v) = pages.as_array().and_then(|a| a.first()) else {
        return (String::new(), 0);
    };
    let cols = page_v.get("columns").and_then(|v| v.as_array());
    let Some(cols) = cols else {
        return (String::new(), 0);
    };
    for col in cols {
        let Some(items) = col.get("items").and_then(|v| v.as_array()) else {
            continue;
        };
        if let Some(item) = items.first() {
            let kind = item
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let para = item.get("paraIndex").and_then(|v| v.as_u64()).unwrap_or(0);
            return (kind, para);
        }
    }
    (String::new(), 0)
}

#[test]
fn spec_revision13_original_is_69_pages() {
    let data = load_bytes();
    let src = HwpDocument::from_bytes(&data).expect("parse source");
    assert_eq!(
        src.page_count(),
        EXPECTED_PAGES,
        "#5128 전제: 한글 스펙문서 원본 69쪽"
    );
}

#[test]
fn spec_revision13_export_hwpx_keeps_origin_marker_and_profile() {
    let data = load_bytes();
    let src = HwpDocument::from_bytes(&data).expect("parse source");
    let bytes = src.export_hwpx_native().expect("export hwpx");
    assert!(
        bytes.len() > 4 && &bytes[0..4] == b"PK\x03\x04",
        "산출물이 ZIP(HWPX) 매직으로 시작해야 한다"
    );

    let ir = rhwp::parse_document(&bytes).expect("parse exported ir");
    assert!(
        ir.hwpx_aux_entry(HWP5_ORIGIN_HWPX_MARKER_PATH).is_some(),
        "HWP5→HWPX 마커가 있어야 한다"
    );
    let profile = ir.layout_profile();
    assert!(
        !profile.native_hwp5_layout() && profile.hwp5_origin_hwpx(),
        "재파싱은 HWP5-origin HWPX 프로필이어야 한다"
    );
    assert!(
        profile.hwp5_stored_pagination_layout(),
        "HWP5-origin HWPX 는 원본과 같은 저장 pagination 계약을 써야 한다"
    );
    assert_eq!(ir.sections.len(), EXPECTED_SECTIONS);
    assert_eq!(
        ir.sections
            .iter()
            .map(|s| s.paragraphs.len())
            .sum::<usize>(),
        EXPECTED_PARAS
    );
}

#[test]
fn spec_revision13_export_hwpx_preserves_page_count() {
    let data = load_bytes();
    let src = HwpDocument::from_bytes(&data).expect("parse source");
    let src_pages = src.page_count();
    assert_eq!(src_pages, EXPECTED_PAGES);

    let bytes = src.export_hwpx_native().expect("export hwpx");
    let round = HwpDocument::from_bytes(&bytes).expect("reparse exported hwpx");
    assert_eq!(
        src_pages,
        round.page_count(),
        "#5128: HWP5→HWPX 왕복 후 페이지 수가 달라졌다"
    );
}

#[test]
fn spec_revision13_p015_table73_split_and_p016_keeps_para84() {
    let data = load_bytes();
    let src = DocumentCore::from_bytes(&data).expect("src");
    let bytes = src.export_hwpx_native().expect("export");
    let rt = DocumentCore::from_bytes(&bytes).expect("rt");

    assert_eq!(src.page_count(), EXPECTED_PAGES);
    assert_eq!(rt.page_count(), EXPECTED_PAGES);

    let (src15_kind, src15_para) = first_item(&src, 14);
    let (rt15_kind, rt15_para) = first_item(&rt, 14);
    assert_eq!(src15_kind, "partialTable");
    assert_eq!(src15_para, 73);
    assert_eq!(
        (rt15_kind.as_str(), rt15_para),
        (src15_kind.as_str(), src15_para),
        "#5128: p015 첫 항목(표 73 분할)이 왕복에서 달라졌다"
    );

    let (src16_kind, src16_para) = first_item(&src, 15);
    let (rt16_kind, rt16_para) = first_item(&rt, 15);
    assert_eq!(src16_kind, "partialParagraph");
    assert_eq!(src16_para, 84);
    assert_eq!(
        (rt16_kind.as_str(), rt16_para),
        (src16_kind.as_str(), src16_para),
        "#5128: p016 은 문단 84 PartialParagraph 로 열려야 한다"
    );
}

#[test]
fn spec_revision13_ir_para_count_unchanged() {
    let data = load_bytes();
    let src_ir = rhwp::parse_document(&data).expect("src ir");
    let core = DocumentCore::from_bytes(&data).expect("core");
    let hwpx = core.export_hwpx_native().expect("export");
    let rt_ir = rhwp::parse_document(&hwpx).expect("rt ir");
    assert_eq!(src_ir.sections.len(), rt_ir.sections.len());
    for (i, (a, b)) in src_ir
        .sections
        .iter()
        .zip(rt_ir.sections.iter())
        .enumerate()
    {
        assert_eq!(
            a.paragraphs.len(),
            b.paragraphs.len(),
            "section[{i}] paragraph count"
        );
    }
}
