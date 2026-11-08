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
// 69 was the pre-fix rhwp pagination, not an independent Hancom oracle.
// The official PDF in pdf/ has 71 pages. This suite guards the export
// contract: source content and page ownership must survive the round trip.
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
fn spec_revision13_original_preserves_every_section_tail() {
    let data = load_bytes();
    let src = DocumentCore::from_bytes(&data).expect("parse source");
    let pages = src.dump_page_items_json(None);
    for (section, source) in src.document().sections.iter().enumerate() {
        assert!(
            pages
                .as_array()
                .unwrap()
                .iter()
                .any(|page| { page["section"].as_u64() == Some(section as u64) }),
            "source section {section} must have a page"
        );
        // The cover section has no direct body text; its table owns the text.
        let Some(tail) = source
            .paragraphs
            .iter()
            .rposition(|p| !p.text.trim().is_empty())
        else {
            continue;
        };
        assert!(
            pages.as_array().unwrap().iter().any(|page| {
                page["section"].as_u64() == Some(section as u64)
                    && page["columns"].as_array().unwrap().iter().any(|column| {
                        column["items"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|item| item["paraIndex"].as_u64() == Some(tail as u64))
                    })
            }),
            "source section {section} tail {tail} must be preserved"
        );
    }
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

    assert_eq!(src.page_count(), rt.page_count(), "round-trip page count");

    // 표 73 이어받기 조각으로 열리는 쪽 다음 쪽은 문단 84 PartialParagraph 로 열린다.
    // 절대 쪽 번호는 고정하지 않는다 — 이 문서는 정본 대비 쪽수가 다르고(#5585), #7696 이
    // 구역 3 의 빈 쪽(정본 9쪽)을 되살린 뒤 이 두 쪽은 정본과 같은 16·17쪽이다
    // (정본 PDF 16쪽 = 표 73 이어받기, 17쪽 첫머리 = 문단 84 둘째 줄. 표 73 을 자르는
    // 행은 정본과 아직 다르다 — 이 검사의 범위 밖).
    let table73_page = (0..src.page_count())
        .find(|&page| first_item(&src, page) == ("partialTable".to_string(), 73))
        .expect("#5128: 표 73 이어받기 조각으로 열리는 쪽이 있어야 한다");
    assert_eq!(
        first_item(&rt, table73_page),
        first_item(&src, table73_page),
        "#5128: 표 73 분할로 열리는 쪽의 첫 항목이 왕복에서 달라졌다"
    );

    let (src_next_kind, src_next_para) = first_item(&src, table73_page + 1);
    assert_eq!(
        (src_next_kind.as_str(), src_next_para),
        ("partialParagraph", 84),
        "#5128: 표 73 분할 다음 쪽은 문단 84 PartialParagraph 로 열려야 한다"
    );
    assert_eq!(
        first_item(&rt, table73_page + 1),
        (src_next_kind, src_next_para),
        "#5128: 표 73 분할 다음 쪽 첫 항목이 왕복에서 달라졌다"
    );

    // Page count alone cannot detect shifted cuts or lost/replayed units.
    // Compare every source owner and cut, without comparing container-specific
    // stored LineSeg metadata generated by HWPX serialization.
    let ownership = |core: &DocumentCore| {
        core.dump_page_items_json(None)
            .as_array()
            .unwrap()
            .iter()
            .map(|page| {
                let columns: Vec<_> = page["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|column| {
                        column["items"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|item| {
                                let mut owner = serde_json::Map::new();
                                for key in [
                                    "kind",
                                    "paraIndex",
                                    "controlIndex",
                                    "startLine",
                                    "endLine",
                                    "startRow",
                                    "endRow",
                                    "startCut",
                                    "endCut",
                                    "isContinuation",
                                ] {
                                    if let Some(value) = item.get(key) {
                                        owner.insert(key.to_owned(), value.clone());
                                    }
                                }
                                serde_json::Value::Object(owner)
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();
                (page["section"].clone(), columns)
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        ownership(&src),
        ownership(&rt),
        "every round-trip page owner and cut"
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
