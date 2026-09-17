//! [#7174 잔여] HWP3→HWP5 저장의 각주/미주 모양이 한/글 변환본과 같은 값을 적는다.
//!
//! `#7181`(구분선 여백)·`#7197`(뒤 장식·시작 번호)로 절반이 닫혔고, 이슈에 남아 있던
//! 것은 **구분선 길이**(`14160` vs `-1`)와 **미주 모양**이다.
//!
//! ## 정답지
//!
//! `tests/fixtures/issue7174/SO-SUEOP-hancom2020.{hwp,hwpx}` — 저장소의 공개 HWP3 표본
//! `samples/SO-SUEOP.hwp` 를 **한/글 2020(11.0.0.9136)** 이 직접 변환한 산출이다.
//! HWPX 쪽은 필드 이름이 그대로 드러나 읽기 쉽다.
//!
//! ```xml
//! <hp:footNotePr><hp:noteLine length="-1" .../>
//!   <hp:noteSpacing betweenNotes="284" belowLine="568" aboveLine="852"/>
//! <hp:endNotePr> <hp:noteLine length="0" type="NONE" .../>
//!   <hp:noteSpacing betweenNotes="0"   belowLine="576" aboveLine="864"/>
//! ```
//!
//! 두 가지를 말한다.
//!
//! 1. 구분선 길이는 고정 HWPUNIT 이 아니라 **OWPML sentinel `-1`(5cm)** 이다.
//!    종전의 `14160` 은 같은 5cm 를 고정값으로 적은 것이라 그려지는 길이는 거의 같지만
//!    (실측 188.80px vs 188.98px @96dpi) 저장 계약이 정본과 달랐다.
//! 2. **미주 여백은 각주 값을 물려받지 않는다.** 이 문서의 각주는 852/568 인데 미주는
//!    864/576 이다. HWP3 문서 정보에는 미주 전용 여백 필드가 없어 한/글이 자기 기본값을
//!    쓰기 때문이다. 종전 rhwp 는 각주 값을 미주에 넣었고, 게다가 "구분선 위"를 HWPX
//!    슬롯에 넣어 HWP5 저장본에서는 그 값이 통째로 0 이 됐다(#7181 과 같은 모양).
//!
//! ## 4바이트 구분선 길이
//!
//! HWP5 `FOOTNOTE_SHAPE` 의 구분선 길이는 4바이트이고, 우리 IR 은 그것을
//! `separator_length`(하위 워드)와 `separator_margin_top`(상위 워드)으로 나눠 읽고
//! 그대로 되쓴다. 그래서 음수 sentinel 은 두 슬롯을 함께 `-1` 로 채워야 한/글이 `-1`
//! 로 읽는다 — 한 쪽만 채우면 `FF FF 00 00` = 65535 가 된다. 한/글 자신의 HWP5 도
//! 이 자리가 `-1`/`-1` 이다(저장소 HWP5 표본 전수에서 같은 짝).
//!
//! ## 이 시험이 잠그지 않는 것
//!
//! 구분선 길이 종류 `1`(본문 폭 1/3)·`2`(단 너비)는 정본 표본이 없어 종전 계산값을
//! 유지했다. 미주 `placement` 가 HWPX 에서 `END_OF_DOCUMENT` 인데 우리 HWP5 attr
//! 해독은 `EachColumn` 으로 읽는 축도 이 이슈의 범위가 아니다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

use rhwp::model::footnote::FootnoteShape;
use rhwp::parser::parse_document;
use rhwp::serializer::serialize_document;

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn load(rel: &str) -> rhwp::model::document::Document {
    parse_document(&std::fs::read(sample(rel)).expect("표본 읽기")).expect("파싱")
}

/// 비교 대상 필드만 뽑는다 — 이 이슈가 말하는 축이다.
fn note_fields(shape: &FootnoteShape) -> Vec<(&'static str, i32)> {
    vec![
        ("separator_length", shape.separator_length),
        ("separator_margin_top", shape.separator_margin_top as i32),
        (
            "separator_margin_bottom",
            shape.separator_margin_bottom as i32,
        ),
        ("note_spacing", shape.note_spacing as i32),
        ("between_notes", shape.raw_unknown as i32),
        ("start_number", shape.start_number as i32),
        ("suffix_char", shape.suffix_char as i32),
        ("separator_line_type", shape.separator_line_type as i32),
        ("separator_line_width", shape.separator_line_width as i32),
    ]
}

/// 한/글 2020 변환본과 우리 변환본의 각주·미주 모양이 같은 값을 말한다.
#[test]
fn hwp3_note_shapes_match_the_hancom_conversion() {
    let oracle = load("tests/fixtures/issue7174/SO-SUEOP-hancom2020.hwp");
    let ours = load("samples/SO-SUEOP.hwp");

    for (label, o, m) in [
        (
            "각주",
            &oracle.sections[0].section_def.footnote_shape,
            &ours.sections[0].section_def.footnote_shape,
        ),
        (
            "미주",
            &oracle.sections[0].section_def.endnote_shape,
            &ours.sections[0].section_def.endnote_shape,
        ),
    ] {
        assert_eq!(
            note_fields(m),
            note_fields(o),
            "{label} 모양이 한/글 2020 변환본과 다르다"
        );
    }
}

/// 값 자체도 못 박는다 — 정본 파일이 바뀌면 위 시험이 함께 움직이므로 숫자를 남긴다.
#[test]
fn hwp3_note_shape_values_are_pinned() {
    for rel in ["samples/SO-SUEOP.hwp", "samples/hwp3-sample10.hwp"] {
        let doc = load(rel);
        let fs = &doc.sections[0].section_def.footnote_shape;
        assert_eq!(
            (fs.separator_length, fs.separator_margin_top),
            (-1, -1),
            "{rel}: 구분선 길이 5cm 는 4바이트 −1 이어야 한다"
        );
        assert_eq!(
            (fs.separator_margin_bottom, fs.note_spacing),
            (852, 568),
            "{rel}: 각주 구분선 위/아래 여백은 문서 값(213·142 hunit ×4)이다"
        );

        // 미주 모양은 미주가 없는 문서에도 적힌다 — 한/글 변환본이 그렇다.
        let es = &doc.sections[0].section_def.endnote_shape;
        assert_eq!(
            (
                es.separator_margin_top,
                es.separator_margin_bottom,
                es.note_spacing
            ),
            (0, 864, 576),
            "{rel}: 미주 여백은 한/글 기본값이며 각주 값을 물려받지 않는다"
        );
        assert_eq!(
            (es.start_number, es.suffix_char),
            (1, ')'),
            "{rel}: 미주 시작 번호·뒤 장식"
        );
    }
}

/// HWP5 로 저장한 뒤 되읽어도 같은 값이다(직렬화기까지 관통한다).
#[test]
fn values_survive_the_hwp5_save() {
    let doc = load("samples/SO-SUEOP.hwp");
    let saved = serialize_document(&doc).expect("HWP5 저장");
    let reparsed = parse_document(&saved).expect("재파싱");

    let oracle = load("tests/fixtures/issue7174/SO-SUEOP-hancom2020.hwp");
    assert_eq!(
        note_fields(&reparsed.sections[0].section_def.footnote_shape),
        note_fields(&oracle.sections[0].section_def.footnote_shape),
        "저장본의 각주 모양이 한/글 변환본과 다르다"
    );
    assert_eq!(
        note_fields(&reparsed.sections[0].section_def.endnote_shape),
        note_fields(&oracle.sections[0].section_def.endnote_shape),
        "저장본의 미주 모양이 한/글 변환본과 다르다"
    );
}

/// 정답지 HWPX 가 말하는 이름값 그대로인지 — 사람이 읽는 근거를 시험에도 남긴다.
#[test]
fn hancom_hwpx_oracle_states_the_same_contract() {
    let bytes =
        std::fs::read(sample("tests/fixtures/issue7174/SO-SUEOP-hancom2020.hwpx")).expect("정본");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).expect("zip");
    let mut xml = String::new();
    {
        let mut entry = archive.by_name("Contents/section0.xml").expect("section0");
        std::io::Read::read_to_string(&mut entry, &mut xml).expect("xml");
    }
    let foot = xml.split("<hp:footNotePr>").nth(1).expect("footNotePr");
    let foot = foot
        .split("</hp:footNotePr>")
        .next()
        .expect("footNotePr 끝");
    let end = xml.split("<hp:endNotePr>").nth(1).expect("endNotePr");
    let end = end.split("</hp:endNotePr>").next().expect("endNotePr 끝");

    assert!(
        foot.contains(r#"length="-1""#),
        "각주 구분선 길이가 −1 이어야 한다: {foot}"
    );
    assert!(
        foot.contains(r#"belowLine="568""#) && foot.contains(r#"aboveLine="852""#),
        "각주 여백은 문서 값이다: {foot}"
    );
    assert!(
        end.contains(r#"belowLine="576""#) && end.contains(r#"aboveLine="864""#),
        "미주 여백은 한/글 기본값이다: {end}"
    );
}
