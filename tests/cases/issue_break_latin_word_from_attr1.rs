//! HWP5 → HWPX 저장에서 라틴 문자 줄나눔 단위(breakLatinWord)가 통째로 사라지던 문제.
//!
//! HWP5 para_shape 는 breakLatinWord(단어/하이픈/글자)를 attr1 bits5-6 에 담는다. 파서는 이
//! 렉시컬 필드(`break_latin_word`)를 채우지 않고 attr1 로만 보존하는데, HWPX 직렬화기는
//! 종전에 그 렉시컬 값이 없으면 무조건 "KEEP_WORD" 로 강등했다 — h2x 저장에서 하이픈·글자
//! 단위 줄나눔이 통째로 사라졌다(10k 코퍼스 실측: 500 문서 중 222 문서가 비-KEEP 설정).
//!
//! 수정: breakNonLatinWord(bit7)와 같은 축으로 attr1 bits5-6 에서 역매핑한다
//! (0=KEEP_WORD·1=HYPHENATION·2=BREAK_WORD). 렉시컬 값이 있으면(HWPX 소스) 그것을 우선한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;

use rhwp::model::document::Document;
use rhwp::model::style::ParaShape;
use rhwp::serializer::hwpx::serialize_hwpx;

fn break_latin_words(hwpx: &[u8]) -> Vec<String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("zip");
    let mut xml = String::new();
    zip.by_name("Contents/header.xml")
        .expect("header.xml")
        .read_to_string(&mut xml)
        .expect("read");
    let needle = "breakLatinWord=\"";
    let mut out = Vec::new();
    let mut rest = xml.as_str();
    while let Some(i) = rest.find(needle) {
        rest = &rest[i + needle.len()..];
        let end = rest.find('"').expect("닫는 따옴표");
        out.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    out
}

fn ps(attr1: u32, lexical: Option<&str>) -> ParaShape {
    ParaShape {
        attr1,
        break_latin_word: lexical.map(str::to_string),
        ..Default::default()
    }
}

#[test]
fn hwp5_break_latin_word_derived_from_attr1_bits() {
    let mut doc = Document::default();
    // HWP5 소스(렉시컬 None): attr1 bits5-6 이 설정을 담는다.
    doc.doc_info.para_shapes.push(ps(2 << 5, None)); // BREAK_WORD
    doc.doc_info.para_shapes.push(ps(1 << 5, None)); // HYPHENATION
    doc.doc_info.para_shapes.push(ps(0, None)); // KEEP_WORD
                                                // HWPX 소스(렉시컬 Some): attr1 을 무시하고 렉시컬 값을 그대로 쓴다.
    doc.doc_info.para_shapes.push(ps(2 << 5, Some("KEEP_WORD")));

    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let vals = break_latin_words(&hwpx);

    assert_eq!(
        vals,
        vec!["BREAK_WORD", "HYPHENATION", "KEEP_WORD", "KEEP_WORD"],
        "attr1 bits5-6 역매핑(2→BREAK_WORD·1→HYPHENATION·0→KEEP_WORD) + 렉시컬 우선이 지켜져야 한다"
    );
}
