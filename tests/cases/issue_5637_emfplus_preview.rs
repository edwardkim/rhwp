//! [Issue #5637] EMF+ 전용 OLE 미리보기를 그리지 못한다.
//!
//! 일부 생산자의 OlePres000 EMF 는 GDI+ 레코드를 `EMR_COMMENT`("EMF+" 시그니처)에
//! 내장하는 이중 스트림인데, 코멘트 레코드의 Size 필드가 실데이터보다 작게 적혀 있어
//! 코멘트 뒤에서 레코드 프레이밍이 임의 바이트 위에 얹힌다. 종전 파서는 그 지점에서
//! 통째로 실패해 미리보기가 placeholder 로 떨어졌다.
//!
//! 수정: EMF+ 코멘트를 본 스트림에 한해 구조 파단 지점부터 다음 그럴듯한 레코드
//! 연쇄로 재동기한다. 그런 스트림 뒤쪽에는 온전한 GDI 폴백(EMR_STRETCHDIBITS)이
//! 이어지므로 실제 미리보기 비트맵이 표준 경로로 그려진다.
//!
//! 계약: `samples/issue5637/2817919_emfplus_ole_preview.hwpx`(관세청, HWPX 코퍼스
//! 실물)의 BinData OLE 미리보기가 EMR_STRETCHDIBITS 를 포함해 파싱되고, standalone
//! SVG 로 `<image>` 가 방출되어야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;
use std::path::Path;

const SAMPLE: &str = "samples/issue5637/2817919_emfplus_ole_preview.hwpx";

fn load_preview_emf() -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let file = std::fs::File::open(Path::new(repo_root).join(SAMPLE))
        .unwrap_or_else(|e| panic!("open {SAMPLE}: {e}"));
    let mut zip = zip::ZipArchive::new(file).expect("zip archive");
    let mut bytes = Vec::new();
    zip.by_name("BinData/ole1.ole")
        .expect("BinData/ole1.ole")
        .read_to_end(&mut bytes)
        .expect("read ole stream");
    // HWPX BinData .ole 는 u32 LE 길이 접두사 + CFB (#2263)
    assert_eq!(&bytes[4..8], &[0xD0, 0xCF, 0x11, 0xE0], "CFB magic");
    let container =
        rhwp::parser::ole_container::parse_ole_container(&bytes[4..]).expect("ole container");
    container.preview_emf.expect("preview_emf")
}

fn header_prefix() -> Vec<u8> {
    let mut bytes = Vec::with_capacity(88);
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&88u32.to_le_bytes());
    for value in [0i32, 0, 1000, 500, 0, 0, 10000, 5000] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(&0x464D4520u32.to_le_bytes());
    bytes.extend_from_slice(&0x00010000u32.to_le_bytes());
    bytes.extend_from_slice(&108u32.to_le_bytes());
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    for _ in 0..3 {
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }
    for value in [1920i32, 1080, 508, 286] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    assert_eq!(bytes.len(), 88);
    bytes
}

fn push_record(bytes: &mut Vec<u8>, record_type: u32, payload: &[u8]) {
    let size = 8u32 + payload.len() as u32;
    assert_eq!(size % 4, 0, "record size must be 4-aligned");
    bytes.extend_from_slice(&record_type.to_le_bytes());
    bytes.extend_from_slice(&size.to_le_bytes());
    bytes.extend_from_slice(payload);
}

fn push_eof(bytes: &mut Vec<u8>) {
    push_record(bytes, 14, &[0u8; 12]);
}

fn emfplus_comment_payload(filler: usize) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&((4 + filler) as u32).to_le_bytes());
    payload.extend_from_slice(b"EMF+");
    payload.resize(payload.len() + filler, 0u8);
    payload
}

fn stretch_dibits_payload() -> Vec<u8> {
    let mut bmi = Vec::new();
    bmi.extend_from_slice(&40u32.to_le_bytes());
    bmi.extend_from_slice(&2i32.to_le_bytes());
    bmi.extend_from_slice(&2i32.to_le_bytes());
    bmi.extend_from_slice(&1u16.to_le_bytes());
    bmi.extend_from_slice(&32u16.to_le_bytes());
    bmi.extend_from_slice(&[0u8; 24]);

    let mut payload = Vec::new();
    for value in [0i32, 0, 2, 2] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload.extend_from_slice(&0i32.to_le_bytes());
    payload.extend_from_slice(&0i32.to_le_bytes());
    payload.extend_from_slice(&[0u8; 16]);
    for value in [80u32, 40, 120, 16, 0, 0x00CC_0020] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload.extend_from_slice(&2i32.to_le_bytes());
    payload.extend_from_slice(&2i32.to_le_bytes());
    payload.extend_from_slice(&bmi);
    payload.extend_from_slice(&[0xAAu8; 16]);
    payload
}

#[test]
fn issue_5637_resyncs_after_lying_emfplus_comment() {
    let mut bytes = header_prefix();
    push_record(&mut bytes, 0x46, &emfplus_comment_payload(8));
    for _ in 0..6 {
        bytes.extend_from_slice(&0xBF00_0000u32.to_le_bytes());
    }
    push_record(&mut bytes, 0x21, &[]);
    push_eof(&mut bytes);

    let records = rhwp::emf::parse_emf(&bytes).expect("resync parse");
    assert!(
        records
            .iter()
            .any(|record| matches!(record, rhwp::emf::Record::SaveDC)),
        "재동기로 SaveDC 를 살려야 한다: {records:?}"
    );
    assert!(
        records
            .iter()
            .any(|record| matches!(record, rhwp::emf::Record::Eof)),
        "EOF 까지 도달해야 한다"
    );
}

#[test]
fn issue_5637_does_not_resync_without_emfplus_comment() {
    let mut bytes = header_prefix();
    let mut plain_comment = Vec::new();
    plain_comment.extend_from_slice(&8u32.to_le_bytes());
    plain_comment.extend_from_slice(b"GDIC");
    plain_comment.extend_from_slice(&[0u8; 4]);
    push_record(&mut bytes, 0x46, &plain_comment);
    let mut rectangle = Vec::new();
    for value in [1i32, 2, 30, 40] {
        rectangle.extend_from_slice(&value.to_le_bytes());
    }
    push_record(&mut bytes, 0x2B, &rectangle);
    for _ in 0..6 {
        bytes.extend_from_slice(&0xBF00_0000u32.to_le_bytes());
    }

    assert!(
        rhwp::emf::parse_emf(&bytes).is_err(),
        "EMF+ 없는 손상 스트림은 그릴 프리픽스가 있어도 오류를 유지해야 한다"
    );
}

#[test]
fn issue_5637_recovers_a_self_consistent_fallback_bitmap() {
    let mut bytes = header_prefix();
    push_record(&mut bytes, 0x46, &emfplus_comment_payload(8));
    for _ in 0..6 {
        bytes.extend_from_slice(&0xBF00_0000u32.to_le_bytes());
    }
    push_record(&mut bytes, 0x51, &stretch_dibits_payload());
    for _ in 0..6 {
        bytes.extend_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    }

    let records = rhwp::emf::parse_emf(&bytes).expect("salvage parse");
    assert!(
        records
            .iter()
            .any(|record| matches!(record, rhwp::emf::Record::StretchDIBits(_))),
        "자기-일관 EMR_STRETCHDIBITS 를 살려야 한다: {records:?}"
    );
}

#[test]
fn issue_5637_keeps_wellformed_emfplus_stream_unchanged() {
    let mut bytes = header_prefix();
    push_record(&mut bytes, 0x46, &emfplus_comment_payload(8));
    push_record(&mut bytes, 0x21, &[]);
    push_eof(&mut bytes);

    let records = rhwp::emf::parse_emf(&bytes).expect("parse");
    assert_eq!(records.len(), 4, "Header+Comment+SaveDC+EOF: {records:?}");
    assert!(matches!(records[2], rhwp::emf::Record::SaveDC));
    assert!(matches!(records[3], rhwp::emf::Record::Eof));
}

#[test]
fn issue_5637_keeps_a_paintable_emfplus_prefix() {
    let mut bytes = header_prefix();
    push_record(&mut bytes, 0x46, &emfplus_comment_payload(8));
    let mut rectangle = Vec::new();
    for value in [1i32, 2, 30, 40] {
        rectangle.extend_from_slice(&value.to_le_bytes());
    }
    push_record(&mut bytes, 0x2B, &rectangle);
    for _ in 0..8 {
        bytes.extend_from_slice(&0xBF00_0000u32.to_le_bytes());
    }

    let records = rhwp::emf::parse_emf(&bytes).expect("salvage parse");
    assert!(records
        .iter()
        .any(|record| matches!(record, rhwp::emf::Record::Rectangle(_))));
    assert!(
        !records
            .iter()
            .any(|record| matches!(record, rhwp::emf::Record::Eof)),
        "EOF 는 없어야 한다"
    );
}

#[test]
fn issue_5637_rejects_an_unpaintable_emfplus_prefix() {
    let mut bytes = header_prefix();
    push_record(&mut bytes, 0x46, &emfplus_comment_payload(8));
    push_record(&mut bytes, 0x21, &[]);
    for _ in 0..8 {
        bytes.extend_from_slice(&0xBF00_0000u32.to_le_bytes());
    }

    assert!(
        rhwp::emf::parse_emf(&bytes).is_err(),
        "그릴 내용 없는 손상 EMF+ 스트림은 오류여야 한다"
    );
}

#[test]
fn emfplus_dual_stream_preview_parses_with_bitmap() {
    let emf = load_preview_emf();
    let records = rhwp::emf::parse_emf(&emf)
        .unwrap_or_else(|e| panic!("EMF+ 이중 스트림 재동기 파싱이 실패했다 (#5637 회귀): {e}"));
    assert!(
        records
            .iter()
            .any(|r| matches!(r, rhwp::emf::Record::StretchDIBits(_))),
        "GDI 폴백 비트맵(EMR_STRETCHDIBITS)을 회수해야 한다: {}개 레코드",
        records.len()
    );
    assert!(
        records.iter().any(|r| matches!(r, rhwp::emf::Record::Eof)),
        "재동기 후 EOF 레코드까지 도달해야 한다"
    );
}

#[test]
fn emfplus_dual_stream_preview_converts_to_svg_image() {
    let emf = load_preview_emf();
    let svg = rhwp::emf::convert_to_standalone_svg(&emf)
        .expect("standalone SVG 변환이 성공해야 한다 (#5637)");
    let svg = String::from_utf8(svg).expect("svg utf-8");
    assert!(
        svg.contains("<image "),
        "미리보기 비트맵 <image> 가 방출되어야 한다: {}",
        &svg[..svg.len().min(200)]
    );
}
