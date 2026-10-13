//! HWPX header.xml `<hh:head secCnt="N">` 가 선언한 구역 수만큼만 구역을 읽는다.
//!
//! `samples/hwpx/hwpx-02.hwpx` 는 #1557 이전 rhwp 가 저장한 파일이다. content.hpf spine 은
//! `section0`·`section1` 둘을 나열하지만 header.xml 은 `secCnt="1"` 이다. 한/글 2022 PDF
//! (`pdf/hwpx/hwpx-02-2022.pdf`)는 5쪽이고 section1 의 글(「거주자」 정의 등)이 없다.
//!
//! 독립 근거 — 한/글 변환 서비스(engine 2020)로 같은 파일의 변형을 PDF 로 출력했다:
//!
//! | 변형 | secCnt | container.rdf 의 section1 | 한/글 쪽수 | section1 글 |
//! |---|---|---|---|---|
//! | 원본 | 1 | 없음 | 5 | 없음 |
//! | a | 2 | 없음 | 6 | 있음 |
//! | b | 1 | 있음 | 5 | 없음 |
//!
//! 따라서 한/글이 읽는 구역 수는 spine 이나 rdf 가 아니라 `secCnt` 가 정한다.
//! 반례: `secCnt` 가 spine 과 같으면(변형 a) 모든 구역을 읽는다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};
use std::path::Path;

const SAMPLE: &str = "samples/hwpx/hwpx-02.hwpx";

fn sample_bytes() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// header.xml 의 `secCnt="1"` 을 `replacement` 로 바꾼 패키지를 만든다.
fn with_header_sec_cnt(replacement: &str) -> Vec<u8> {
    let src = sample_bytes();
    let mut archive = zip::ZipArchive::new(Cursor::new(src)).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut replaced = false;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).expect("entry");
        let name = entry.name().expect("entry name").to_string();
        let method = entry.compression();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        if name == "Contents/header.xml" {
            let text = String::from_utf8(data).expect("utf8 header");
            assert!(
                text.contains(r#"secCnt="1""#),
                "원본 header 가 secCnt=1 이어야 한다"
            );
            data = text.replacen(r#"secCnt="1""#, replacement, 1).into_bytes();
            replaced = true;
        }
        let options = zip::write::SimpleFileOptions::default().compression_method(method);
        out.start_file(name, options).expect("start file");
        out.write_all(&data).expect("write entry");
    }
    assert!(replaced, "header.xml 을 찾지 못했다");
    out.finish().expect("finish").into_inner()
}

fn section_count(bytes: &[u8]) -> usize {
    rhwp::parser::hwpx::parse_hwpx(bytes)
        .expect("parse hwpx")
        .sections
        .len()
}

fn page_count(bytes: &[u8]) -> u32 {
    rhwp::document_core::DocumentCore::from_bytes(bytes)
        .expect("open")
        .page_count()
}

#[test]
fn spine_sections_beyond_declared_sec_cnt_are_not_loaded() {
    let bytes = sample_bytes();
    assert_eq!(
        section_count(&bytes),
        1,
        "secCnt=1 인데 spine 의 section1 까지 읽었다 — 한/글은 앞 1개 구역만 읽는다"
    );
    assert_eq!(page_count(&bytes), 5, "한/글 2022 PDF 는 5쪽이다");
}

#[test]
fn declared_sec_cnt_matching_spine_loads_every_section() {
    // 변형 a: secCnt=2 → 한/글 6쪽.
    let bytes = with_header_sec_cnt(r#"secCnt="2""#);
    assert_eq!(section_count(&bytes), 2);
    assert_eq!(page_count(&bytes), 6, "한/글 변형 a PDF 는 6쪽이다");
}

#[test]
fn missing_or_out_of_range_sec_cnt_falls_back_to_spine() {
    // 선언이 없거나 0, spine 보다 크면 근거가 없으므로 spine 을 그대로 따른다.
    for replacement in [r#"secCntX="1""#, r#"secCnt="0""#, r#"secCnt="9""#] {
        let bytes = with_header_sec_cnt(replacement);
        assert_eq!(
            section_count(&bytes),
            2,
            "{replacement}: spine 2개를 읽어야 한다"
        );
    }
}
