//! [Issue #5933] HWPML(HML) 을 HWP5 로 저장하면 한글이 본문을 통째로 버린다.
//!
//! 한컴 저장본과 rhwp 의 HWP5 왕복본은 예외 없이 구역당 `HWPTAG_PAGE_BORDER_FILL` 이
//! **3개**(양쪽/짝수쪽/홀수쪽)다(#3676). HWPX·HWP3 출처는 저장 어댑터
//! (`convert_if_hwpx_source` → `normalize_page_border_fills_for_hwp`)가 그 개수를 채우는데,
//! HML 출처는 어댑터 자체를 타지 않아 **1개만** 나갔다.
//!
//! 한글 2022 오라클 실측(08462, 행안부 고시 배포본): 저장본이 **1쪽 0자**로 열리고 컨트롤
//! 인구조사가 `cold:1,secd:1` 로 붕괴한다(원본은 4쪽 7,428자). rhwp 자기 조판은 같은
//! 산출을 4쪽으로 읽으므로 `--verify-pages` 로는 보이지 않는다.
//!
//! 돌연변이 검정으로 원인을 갈랐다 — OLE contract 스트림 9개를 채워도, 스트림을 압축하고
//! `FileHeader` flags bit0 을 켜도 1쪽 0자 그대로였고, **PBF 3개**만이 4쪽 7,549자를 되살렸다.
//!
//! 판정은 **방출된 레코드 개수**로 한다. HWP5 파서는 `extra_page_border_fills` 를 채우지
//! 않으므로(HWPX 파서만 채운다) 재파싱 IR 로는 이 계약을 볼 수 없다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::document_core::DocumentCore;
use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags::HWPTAG_PAGE_BORDER_FILL;

const SAMPLE: &str = "samples/hml/aligns.hml";

fn open_sample() -> DocumentCore {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"))
}

/// 저장한 HWP5 의 `BodyText/Section0` 에서 PAGE_BORDER_FILL 레코드를 센다.
fn page_border_fill_records(hwp5: &[u8]) -> usize {
    let mut reader = CfbReader::open(hwp5).expect("CFB 열기");
    let header = reader.read_file_header().expect("FileHeader");
    let compressed = header.len() >= 40 && header[36] & 0x01 != 0;
    let section = reader
        .read_body_text_section(0, compressed, false)
        .expect("BodyText/Section0");
    Record::read_all(&section)
        .expect("레코드 파싱")
        .iter()
        .filter(|r| r.tag_id == HWPTAG_PAGE_BORDER_FILL)
        .count()
}

/// 저장한 HWP5 는 구역마다 PBF 레코드가 3개여야 한다.
#[test]
fn saved_hwp5_emits_three_page_border_fill_records() {
    let mut core = open_sample();
    let bytes = core.export_hwp_with_adapter().expect("HWP5 저장");
    assert_eq!(
        page_border_fill_records(&bytes),
        3,
        "PAGE_BORDER_FILL 이 3개로 방출되지 않았다 — 미달이면 한글이 본문을 버린다 (#5933 회귀)"
    );
}

/// 두 번 저장해도 개수가 늘지 않는다(멱등).
#[test]
fn repeated_saves_stay_at_three_records() {
    let mut core = open_sample();
    let first = core.export_hwp_with_adapter().expect("첫 저장");
    let second = core.export_hwp_with_adapter().expect("둘째 저장");
    assert_eq!(page_border_fill_records(&first), 3, "첫 저장");
    assert_eq!(page_border_fill_records(&second), 3, "둘째 저장");
}

/// 저장은 live IR 을 바꾸지 않는다 — HWPX 출처와 같은 복원 계약이라
/// 이어지는 HWPX 저장이 원본에 없던 EVEN/ODD `pageBorderFill` 을 만들지 않는다.
#[test]
fn saving_does_not_leak_the_padding_into_live_ir() {
    let mut core = open_sample();
    let before: Vec<usize> = core
        .document()
        .sections
        .iter()
        .map(|s| s.section_def.extra_page_border_fills.len())
        .collect();
    let _ = core.export_hwp_with_adapter().expect("HWP5 저장");
    let after: Vec<usize> = core
        .document()
        .sections
        .iter()
        .map(|s| s.section_def.extra_page_border_fills.len())
        .collect();
    assert_eq!(
        before, after,
        "저장 보정이 live IR 에 남았다 — 이어지는 HWPX 저장이 없던 EVEN/ODD 를 만든다"
    );
}
