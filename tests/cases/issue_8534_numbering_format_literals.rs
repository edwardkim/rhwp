//! [#8534] HWPX 저장 시 문단 번호 형식(`hh:paraHead@numFormat`)을 OWPML 스펙 리터럴로 쓴다.
//!
//! ## 무엇이 틀렸나
//!
//! `numbering_format_str` 이 표 41 코드 12·13 을 비스펙 별칭(`HANGUL_NUMBER`·`HANJA_NUMBER`)으로,
//! 6·7·9·10·11·14 를 `DIGIT` 으로 썼다. 한/글은 그 번호를 아라비아 숫자로 그린다. rhwp 파서는
//! 별칭도 받으므로(#2877) HWPX 왕복 시험으로는 드러나지 않았다.
//!
//! ## 기대값의 출처
//!
//! 1. **OWPML 스키마**: `mydocs/manual/OWPML SCHEMA/Core XML schema.xml` 의 `NumberType1`
//!    enumeration — 쓴 리터럴은 모두 이 목록에 있어야 한다.
//! 2. **표 41 코드**: 파서 `parse_numbering_format_code` 가 스펙 리터럴을 읽은 코드가 원래 코드와
//!    같아야 한다(파서는 #2877 에서 스펙 15종을 표 41 순서로 확정).
//! 3. **한/글 출력**: `samples/issue8534/outline_numfmt.hwp`(개요 수준 1·2·3 = 코드 12·13·9)를
//!    한/글로 열면 `일.` `一.` `㉮.` 이다. 수정 전 rhwp 의 HWP5→HWPX 결과는 한/글에서 `1.` `1.` `1.`.
//!    코드 0~14 전부를 수준별로 나눈 HWP5 3벌(`mydocs/pr/assets/issue_8534/codes_*.hwp`)도 수정 후
//!    HWPX 의 한/글 번호가 HWP5 원본과 같다(수정 전엔 6·7·9·10·11·12·13·14 가 `1.`).

#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;

use rhwp::model::document::Document;
use rhwp::parser::parse_document;
use rhwp::serializer::serialize_hwpx;

const FIXTURE: &str = "samples/issue8534/outline_numfmt.hwp";
const SCHEMA: &str = "mydocs/manual/OWPML SCHEMA/Core XML schema.xml";

fn fixture() -> Document {
    let bytes = std::fs::read(FIXTURE).expect("fixture");
    parse_document(&bytes).expect("HWP5 파싱")
}

fn header_xml(hwpx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("HWPX 는 zip 이다");
    let mut s = String::new();
    zip.by_name("Contents/header.xml")
        .expect("header.xml")
        .read_to_string(&mut s)
        .expect("utf-8");
    s
}

/// 첫 번째 `hh:numbering` 의 수준 1~7 `numFormat` 값.
fn para_head_formats(header: &str) -> Vec<String> {
    let start = header.find("<hh:numbering ").expect("hh:numbering");
    let end = start + header[start..].find("</hh:numbering>").expect("닫는 태그");
    header[start..end]
        .split("<hh:paraHead ")
        .skip(1)
        .take(7)
        .map(|tag| {
            let i = tag.find("numFormat=\"").expect("numFormat") + "numFormat=\"".len();
            tag[i..i + tag[i..].find('"').expect("따옴표")].to_string()
        })
        .collect()
}

/// 스키마 `NumberType1` 의 enumeration 값.
fn schema_number_type1() -> Vec<String> {
    let xsd = std::fs::read_to_string(SCHEMA).expect("OWPML 스키마");
    let start = xsd
        .find(r#"<xs:simpleType name="NumberType1">"#)
        .expect("NumberType1");
    let end = start + xsd[start..].find("</xs:simpleType>").expect("닫는 태그");
    xsd[start..end]
        .split(r#"<xs:enumeration value=""#)
        .skip(1)
        .map(|s| s[..s.find('"').expect("따옴표")].to_string())
        .collect()
}

/// 원본: HWP5 개요 수준 1·2·3 = 코드 12·13·9 가 HWPX 에서 스펙 리터럴이 되고 다시 같은 코드로 읽힌다.
#[test]
fn hwp5_outline_codes_are_written_as_spec_literals() {
    let doc = fixture();
    let n = &doc.doc_info.numberings[0];
    assert!(
        n.raw_para_heads.is_none(),
        "HWP5 원본은 HWPX paraHead 원문이 없다"
    );
    let codes: Vec<u8> = n.heads[..3].iter().map(|h| h.number_format).collect();
    assert_eq!(codes, [12, 13, 9], "원본 HWP5 의 수준 1~3 번호 형식");

    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let formats = para_head_formats(&header_xml(&hwpx));
    assert_eq!(
        formats[..3],
        ["HANGUL_PHONETIC", "IDEOGRAPH", "CIRCLED_HANGUL_SYLLABLE"],
        "표 41 코드 12·13·9 의 OWPML 리터럴"
    );

    let back = parse_document(&hwpx).expect("HWPX 재파싱");
    let back_codes: Vec<u8> = back.doc_info.numberings[0].heads[..3]
        .iter()
        .map(|h| h.number_format)
        .collect();
    assert_eq!(back_codes, codes);
}

/// 표 41 의 코드 0~14 모두: 쓴 리터럴이 스키마 `NumberType1` 에 있고, 다시 읽으면 같은 코드다.
#[test]
fn every_table41_code_is_a_schema_literal_and_round_trips() {
    let schema = schema_number_type1();
    assert!(schema.len() >= 15, "NumberType1 enumeration: {schema:?}");
    let all: Vec<u8> = (0..=14).collect();
    let mut seen = Vec::new();
    for chunk in all.chunks(7) {
        let mut doc = fixture();
        let n = &mut doc.doc_info.numberings[0];
        for (head, &code) in n.heads.iter_mut().zip(chunk) {
            head.number_format = code;
        }
        let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
        let formats = para_head_formats(&header_xml(&hwpx));
        let back = parse_document(&hwpx).expect("HWPX 재파싱");
        for (level, &code) in chunk.iter().enumerate() {
            let literal = &formats[level];
            assert!(
                schema.contains(literal),
                "코드 {code} → {literal} 는 NumberType1 리터럴이 아니다"
            );
            assert_eq!(
                back.doc_info.numberings[0].heads[level].number_format, code,
                "코드 {code} → {literal} → 재파싱"
            );
            seen.push(literal.clone());
        }
    }
    let mut distinct = seen.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(distinct.len(), 15, "코드마다 다른 리터럴: {seen:?}");
}
