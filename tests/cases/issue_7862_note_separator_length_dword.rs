//! [#7862] HWP5 각주/미주 모양의 구분선 길이는 4바이트다.
//!
//! # 무엇이 깨져 있었나
//!
//! HWP5 `FOOTNOTE_SHAPE` 는 `u32 구분선 길이 · i16 구분선 위 · i16 구분선 아래 · u16 주석
//! 사이` 순서인데, rhwp 는 길이를 i16 으로 읽고 상위 워드를 HWPX `aboveLine` 슬롯
//! (`separator_margin_top`)에 넣었다. 한/글 기본 미주 모양(`12280 | 224`)은 상위 워드가
//! 0 이 아니라서
//!
//! - 읽기: 길이 12280(43.3mm)·구분선 위 224 로 해석해 구분선이 짧고 미주가 8px 위였다.
//! - HWPX→HWP5 쓰기: `aboveLine` 을 길이 상위 워드 자리에 써서 길이가 부풀고
//!   구분선 위가 0 이 됐다(각주 `-1` 표지는 65535 로 깨졌다).
//! - HWP5→HWPX 쓰기: `length="12280" aboveLine="224"` 를 냈다.
//!
//! # 기대값의 출처 — 한/글 자신의 저장본
//!
//! - 한/글 기본 미주 모양의 HWPX 표기는 `length="14692344" aboveLine="850"
//!   belowLine="567" betweenNotes="0"` 이다(`src/parser/hwpx/section.rs` 시험 입력과 같다).
//!   224 × 65536 + 12280 = 14692344.
//! - `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp` 는 같은 문서를
//!   한/글이 HWP5 로 저장한 것이고, `.hwpx` 는 한/글이 HWPX 로 저장한 것이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::footnote::FootnoteShape;

fn load(path: &str) -> DocumentCore {
    let full = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let bytes = std::fs::read(&full).unwrap_or_else(|e| panic!("{path} 읽기: {e}"));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("{path} 로드: {e:?}"))
}

/// (길이, 구분선 위, 구분선 아래, 주석 사이) — 한/글 UI 의 네 값.
fn ui(shape: &FootnoteShape) -> (i32, i16, i16, u16) {
    (
        shape.separator_length,
        shape.separator_above_margin_hu(),
        shape.separator_below_margin_hu(),
        shape.between_notes_margin_hu(),
    )
}

fn note_shapes(core: &DocumentCore) -> ((i32, i16, i16, u16), (i32, i16, i16, u16)) {
    let def = &core.document().sections[0].section_def;
    (ui(&def.footnote_shape), ui(&def.endnote_shape))
}

const PAIR_HWP: &str = "samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp";
const PAIR_HWPX: &str = "samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwpx";

/// 한/글 기본 미주 모양을 HWP5 에서 읽으면 한/글 HWPX 표기와 같은 값이 나온다.
#[test]
fn hwp5_default_endnote_shape_reads_the_full_dword_length() {
    let core = load("samples/endnote-01.hwp");
    let (footnote, endnote) = note_shapes(&core);
    assert_eq!(
        endnote,
        (14_692_344, 850, 567, 0),
        "한/글 기본 미주 모양(length=14692344 aboveLine=850 belowLine=567 betweenNotes=0)"
    );
    // 각주 `-1`(5cm) 표지는 그대로다.
    assert_eq!(footnote, (-1, 850, 567, 283));
}

/// 같은 문서의 한/글 HWP5·HWPX 저장본은 같은 네 값으로 읽힌다(대조군).
#[test]
fn hancom_hwp5_and_hwpx_of_the_same_document_agree() {
    assert_eq!(note_shapes(&load(PAIR_HWP)), note_shapes(&load(PAIR_HWPX)));
}

/// HWP5 본문 Section0 의 `FOOTNOTE_SHAPE` 레코드 원바이트(각주·미주 순서).
fn footnote_shape_records(hwp5: &[u8]) -> Vec<Vec<u8>> {
    use rhwp::parser::cfb_reader::CfbReader;
    use rhwp::parser::record::Record;
    let mut cfb = CfbReader::open(hwp5).expect("CFB 열기");
    let file_header = cfb.read_file_header().expect("FileHeader 읽기");
    let compressed = file_header.get(36).is_some_and(|b| b & 0x01 != 0);
    let section = cfb
        .read_body_text_section(0, compressed, false)
        .expect("Section0 읽기");
    Record::read_all(&section)
        .expect("Section0 record 파싱")
        .into_iter()
        .filter(|r| r.tag_id == rhwp::parser::tags::HWPTAG_FOOTNOTE_SHAPE)
        .map(|r| r.data)
        .collect()
}

/// HWPX 를 HWP5 로 저장한 각주/미주 모양 레코드가 한/글이 같은 문서를 HWP5 로 저장한
/// 레코드와 바이트까지 같다. (다시 읽어 비교하면 읽기·쓰기의 같은 오해가 상쇄되므로
/// 바이트로 본다.)
#[test]
fn hwpx_to_hwp5_note_shape_records_match_hancom_bytes() {
    let hancom = footnote_shape_records(
        &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(PAIR_HWP))
            .expect("한/글 HWP5 읽기"),
    );
    assert_eq!(hancom.len(), 2, "각주·미주 모양 두 레코드");
    let exported = load(PAIR_HWPX)
        .export_hwp_with_adapter_snapshot()
        .expect("HWPX→HWP5 저장");
    assert_eq!(
        footnote_shape_records(&exported),
        hancom,
        "HWPX→HWP5 저장본의 각주/미주 모양 레코드가 한/글 저장본과 다르다"
    );
}

/// HWP5 원본을 그대로 저장하면 각주/미주 모양 레코드가 바이트 그대로 남는다.
#[test]
fn hwp5_round_trip_keeps_note_shape_record_bytes() {
    for path in ["samples/endnote-01.hwp", PAIR_HWP] {
        let original =
            std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
        let saved = load(path).export_hwp_native().expect("HWP5 저장");
        assert_eq!(
            footnote_shape_records(&saved),
            footnote_shape_records(&original),
            "{path} 저장 왕복에서 각주/미주 모양 레코드가 바뀌었다"
        );
    }
}

/// HWP5 를 HWPX 로 내보내면 한/글 기본 미주 모양의 HWPX 표기가 나온다.
#[test]
fn hwp5_to_hwpx_writes_hancom_default_endnote_line() {
    let exported = load("samples/endnote-01.hwp")
        .export_hwpx_native()
        .expect("HWP5→HWPX 저장");
    let reread = DocumentCore::from_bytes(&exported).expect("저장본 다시 읽기");
    let (_, endnote) = note_shapes(&reread);
    assert_eq!(endnote, (14_692_344, 850, 567, 0));
}
