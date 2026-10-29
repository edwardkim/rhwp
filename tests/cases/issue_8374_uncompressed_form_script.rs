//! [#8374] 압축하지 않은 HWP5 의 `Scripts/DefaultJScript` 를 원문 그대로 읽어 HWPX 로 옮긴다.
//!
//! HWP5 는 FileHeader 압축 비트가 0 이면 스크립트 스트림도 압축하지 않고 저장한다. #8360 이
//! HWPX 저장 경로에 연결한 스크립트 변환은 이 스트림을 늘 deflate 로 풀어,
//! `samples/issue5699/37787_regulatory_impact.hwp` 와 `samples/hwpers_test4_complex_table.hwp`
//! 의 HWPX 저장이 `FORM_ACTION_INVALID` 로 실패했다.
//!
//! 기대값은 원본 스트림의 바이트 배치(HWP5 4.2.13: 글자 수 u32 + UTF-16LE 본문, 머리 스크립트와
//! 본문 스크립트 순)에서 직접 디코딩한 원문이다. 압축 문서(`samples/form-01.hwp`)는 종전대로
//! deflate 를 풀어 같은 방식으로 옮긴다.
//!
//! HWP 저장도 같은 계약을 따른다. HWPX 를 다시 열어 HWP 로 저장하면 압축 비트 0 헤더가
//! 나가므로, deflate 로 합성해 둔 스크립트도 풀어서 써야 다시 열 때 원문이 된다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::parser::parse_document;
use std::io::{Cursor, Read};

fn read(rel: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)).expect("표본 읽기")
}

/// 글자 수(u32 LE) + UTF-16LE 본문 두 개를 읽는다.
fn counted_utf16_pair(raw: &[u8]) -> (String, String) {
    let mut offset = 0;
    let mut part = || {
        let count = u32::from_le_bytes(raw[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let units: Vec<u16> = raw[offset..offset + count * 2]
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        offset += count * 2;
        String::from_utf16(&units).expect("UTF-16 스크립트")
    };
    (part(), part())
}

/// 원본 HWP 의 스크립트 스트림을 압축 비트에 따라 디코딩한 (머리, 본문) 스크립트.
fn source_scripts(bytes: &[u8]) -> (bool, (String, String)) {
    let doc = parse_document(bytes).expect("파싱");
    let stream = doc
        .extra_streams
        .iter()
        .find(|(path, _)| path.trim_start_matches('/') == "Scripts/DefaultJScript")
        .map(|(_, bytes)| bytes.clone())
        .expect("Scripts/DefaultJScript");
    let raw = if doc.header.compressed {
        let mut raw = Vec::new();
        flate2::read::DeflateDecoder::new(stream.as_slice())
            .read_to_end(&mut raw)
            .expect("deflate 해제");
        raw
    } else {
        stream
    };
    (doc.header.compressed, counted_utf16_pair(&raw))
}

fn hwpx_entry_utf16(hwpx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(Cursor::new(hwpx)).expect("HWPX zip");
    let mut bytes = Vec::new();
    zip.by_name(name)
        .unwrap_or_else(|_| panic!("{name}"))
        .read_to_end(&mut bytes)
        .expect("엔트리 읽기");
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    String::from_utf16(&units).expect("UTF-16 엔트리")
}

#[test]
fn hwp5_form_scripts_follow_the_file_header_compression_bit() {
    for (rel, compressed) in [
        ("samples/issue5699/37787_regulatory_impact.hwp", false),
        ("samples/hwpers_test4_complex_table.hwp", false),
        ("samples/form-01.hwp", true),
    ] {
        let source = read(rel);
        let (is_compressed, (header, body)) = source_scripts(&source);
        assert_eq!(is_compressed, compressed, "{rel}: FileHeader 압축 비트");
        assert!(
            !header.is_empty() && !body.is_empty(),
            "{rel}: 원본 스크립트"
        );

        let core = DocumentCore::from_bytes(&source).expect("문서 열기");
        let hwpx = core
            .export_hwpx_native()
            .unwrap_or_else(|error| panic!("{rel}: HWPX 저장 실패 — {error}"));
        assert_eq!(
            hwpx_entry_utf16(&hwpx, "Scripts/headerScripts"),
            header,
            "{rel}: 머리 스크립트"
        );
        assert_eq!(
            hwpx_entry_utf16(&hwpx, "Scripts/sourceScripts"),
            body,
            "{rel}: 본문 스크립트"
        );

        // HWPX 를 다시 열어 HWP 로 저장해도 스크립트가 그 HWP 의 압축 비트를 따른다.
        // HWPX 출처 IR 은 압축 비트 0 헤더에 deflate 로 합성한 스크립트를 들고 있다.
        let reopened = DocumentCore::from_bytes(&hwpx).expect("HWPX 다시 열기");
        let hwp = reopened.export_hwp_native().expect("HWP 저장");
        let (_, scripts) = source_scripts(&hwp);
        assert_eq!(scripts, (header, body), "{rel}: HWPX→HWP 스크립트");
    }
}
