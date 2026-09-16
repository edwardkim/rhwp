//! [Issue #7174] HWP3 각주 번호의 닫는 장식이 각주 모양에 실리지 않고 본문 글자로 굳는다.
//!
//! HWP3 은 각주/미주 번호의 닫는 장식을 **두 곳**에 나눠 갖는다.
//!
//! ```text
//!   doc_info offset 110 (스펙 타입 echar)   문서 옵션 — 번호에 붙일 장식 문자
//!   주석 본문의 첫 글자                     같은 장식이 리터럴로 한 번 더 들어 있다
//! ```
//!
//! HWP5 는 장식을 `FOOTNOTE_SHAPE` 하나로만 표현하고, 그 값으로 **본문 참조 번호와
//! 주석 영역 번호를 모두** 그린다. 그래서 변환은 둘을 함께 해야 한다 — 모양에 장식을
//! 싣고, 본문에 남은 리터럴을 뗀다. 한쪽만 하면 번호가 `1` 이거나 `1))` 가 된다.
//!
//! # 기대값의 출처
//!
//! 정본은 한글 자신의 HWP5 변환본이다. 264쪽 표본
//! `1170000-200500003_D0150004-1-001`(각주 230개)을 한글 2024 로 열어 텍스트를
//! 추출한 값이다.
//!
//! ```text
//!   정본                283,780자   본문 참조 1)  · 주석 영역 1)
//!   수정 전             283,556자   본문 참조 1   · 주석 영역 1)    ')' 230개 부족
//!   각주 모양만 배선    284,016자   본문 참조 1)  · 주석 영역 1))   ')' 230개 과잉
//!   모양 + 리터럴 제거  283,786자   글자 멀티셋 소실 0 (잔여는 빈 줄 3개, 별개 축)
//! ```
//!
//! 정본의 `FOOTNOTE_SHAPE[0]` 은 뒤 장식 `41`(닫는 소괄호)·시작 번호 `1` 이고, 둘 다
//! 원본 `doc_info` 값과 그대로 같다(offset 110 = `41`, offset 100 = `1`).
//! 증적: `mydocs/report/7174-hwp3-footnote-number-suffix/`.
//!
//! # 이 시험이 잠그는 것과 자료의 성격
//!
//! - 각주 모양 배선은 **저장소 표본**(`samples/SO-SUEOP.hwp`, HWP3)으로 잠근다.
//! - 리터럴 제거는 **합성 최소 HWP3** 로 잠근다. 리터럴을 실제로 담은 문서는 코퍼스의
//!   264쪽 표본 하나뿐이고, `.hwp` 는 `samples/` 아래 어디에 넣어도 `ir_field_sweep`
//!   래칫을 깨므로 저장소에 둘 수 없다. 합성 입력의 **계약 시험**이며 한글 출력과의
//!   일치 증거가 아니다 — 그 증거는 위 표와 보고서다.
//! - 반례도 함께 잠근다: 문서가 장식을 끄면 떼지 않고, 장식을 켰어도 주석 첫 글자가
//!   그 장식이 아니면 떼지 않는다.
//!
//! # 잠그지 않는 것
//!
//! 구분선 여백(PR #7181)·구분선 길이(정본 `-1` 대 고정값)·미주 모양이 통째로 0 인 축.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::{AutoNumberType, Control};
use rhwp::model::paragraph::Paragraph;
use std::path::Path;

/// 번호 장식 문자. **구현이 아니라 원본 바이트에서 정한 값**이다 — 표본의 `doc_info`
/// offset 110 이 `41`(닫는 소괄호)이고, 한글 자신의 HWP5 변환본도 `FOOTNOTE_SHAPE`
/// 뒤 장식에 `41` 을 쓴다. 코퍼스 HWP3 38건 전수도 `41` 이다.
const NUMBER_SUFFIX: char = ')';

/// 각주 시작 번호. 같은 근거 — 표본 `doc_info` offset 100 이 `1` 이고 정본도 `1` 이다.
const START_NUMBER: u16 = 1;

// ---------------------------------------------------------------------------
// 저장소 표본 — 각주 모양 배선
// ---------------------------------------------------------------------------

fn load(rel: &str) -> rhwp::model::document::Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    rhwp::parser::parse_document(&std::fs::read(path).expect("표본 읽기")).expect("파싱")
}

/// HWP3 각주 모양이 문서가 지정한 번호 장식과 시작 번호를 싣는다.
#[test]
fn hwp3_footnote_shape_carries_the_document_number_suffix() {
    let doc = load("samples/SO-SUEOP.hwp");
    let shape = &doc.sections[0].section_def.footnote_shape;
    assert_eq!(
        shape.suffix_char, NUMBER_SUFFIX,
        "각주 모양의 뒤 장식이 비면 저장본의 본문 참조 번호가 닫는 장식을 잃는다"
    );
    assert_eq!(
        shape.start_number, START_NUMBER,
        "각주 모양의 시작 번호가 한글 정본(1)과 다르다"
    );
}

// ---------------------------------------------------------------------------
// 합성 최소 HWP3 — 리터럴 제거
//
// 골격은 `issue_4680_hwp3_odd_page_start` 와 동형이다.
// ---------------------------------------------------------------------------

fn u16le(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}

fn u32le(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}

/// `doc_info` 128바이트. offset 100 = 각주 시작 번호, offset 110 = 각주 옵션(echar).
fn doc_info(bracket: u8) -> [u8; 128] {
    let mut d = [0u8; 128];
    d[100..102].copy_from_slice(&u16le(START_NUMBER));
    d[110] = bracket;
    d
}

fn hwp3_doc(bracket: u8, body: &[u8]) -> Vec<u8> {
    let mut d = Vec::new();
    d.extend_from_slice(b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05");
    assert_eq!(d.len(), 30);
    d.extend_from_slice(&doc_info(bracket));
    d.extend_from_slice(&[0u8; 1008]); // 요약
    d.extend_from_slice(body);
    d
}

fn hwp3_body(paragraphs: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    for _ in 0..7 {
        b.extend_from_slice(&u16le(1));
        let mut name = [0u8; 40];
        name[..4].copy_from_slice(&[0xB9, 0xD9, 0xC5, 0xC1]); // 바탕 (EUC-KR)
        b.extend_from_slice(&name);
    }
    b.extend_from_slice(&u16le(0)); // nstyles
    b.extend_from_slice(paragraphs);
    b.extend_from_slice(&paragraph_list_end());
    b
}

fn paragraph_list_end() -> [u8; 43] {
    [0u8; 43]
}

fn char_shape31() -> [u8; 31] {
    let mut cs = [0u8; 31];
    cs[..2].copy_from_slice(&u16le(250));
    for r in &mut cs[9..16] {
        *r = 100;
    }
    cs
}

fn para_shape187() -> [u8; 187] {
    let mut ps = [0u8; 187];
    ps[6..8].copy_from_slice(&u16le(160));
    ps[172] = 1;
    ps
}

fn para_header(char_count: u16, line_count: u16) -> Vec<u8> {
    let mut h = Vec::new();
    h.push(0u8);
    h.extend_from_slice(&u16le(char_count));
    h.extend_from_slice(&u16le(line_count));
    h.push(0u8); // include_char_shape
    h.push(0u8); // flags
    h.extend_from_slice(&u32le(0)); // special_char_flags
    h.push(0u8); // style_index
    h.extend_from_slice(&char_shape31());
    h.extend_from_slice(&para_shape187());
    h
}

fn line_info(pgy: u16) -> Vec<u8> {
    let mut l = Vec::new();
    l.extend_from_slice(&u16le(0));
    l.extend_from_slice(&u16le(0));
    l.extend_from_slice(&u16le(400));
    l.extend_from_slice(&u16le(pgy));
    l.extend_from_slice(&u16le(0));
    l.extend_from_slice(&u16le(0));
    l.extend_from_slice(&u16le(0));
    l
}

/// 각주 본문 문단 하나 — 번호 코드(18, 종류 1 = 각주) 뒤에 `lead` 와 본문 글자.
/// 스펙 §10.12 표 53: `[hchar 18][word 종류][word 번호값][hchar 18]` = 8바이트 = 4 hchar.
fn note_paragraph_list(lead: Option<char>, body: &str) -> Vec<u8> {
    let tail: Vec<u16> = lead
        .into_iter()
        .chain(body.chars())
        .map(|c| c as u16)
        .collect();
    let mut p = Vec::new();
    // 4(번호 코드) + 글자 + 1(문단 끝)
    p.extend_from_slice(&para_header(4 + tail.len() as u16 + 1, 1));
    p.extend_from_slice(&line_info(0));
    p.extend_from_slice(&u16le(18));
    p.extend_from_slice(&u16le(1)); // 종류 1 = 각주
    p.extend_from_slice(&u16le(1)); // 번호값
    p.extend_from_slice(&u16le(18));
    for c in &tail {
        p.extend_from_slice(&u16le(*c));
    }
    p.extend_from_slice(&u16le(13)); // 문단 끝
    p.extend_from_slice(&paragraph_list_end());
    p
}

/// 각주(17) 하나만 담은 본문 문단.
/// 스펙 §10.11 표 51·52: 식별 정보 8바이트(= 4 hchar) → 정보 14바이트 → 문단 리스트.
fn footnote_paragraph(lead: Option<char>, body: &str) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&para_header(5, 1)); // 4(각주) + 1(문단 끝)
    p.extend_from_slice(&line_info(100));
    p.extend_from_slice(&u16le(17));
    p.extend_from_slice(&u32le(0)); // 예약
    p.extend_from_slice(&u16le(17));
    let mut info = [0u8; 14];
    info[8..10].copy_from_slice(&u16le(0)); // 번호
    info[10..12].copy_from_slice(&u16le(0)); // 종류 0 = 각주
    info[12..14].copy_from_slice(&u16le(400)); // 각주 문단 너비
    p.extend_from_slice(&info);
    p.extend_from_slice(&note_paragraph_list(lead, body));
    p.extend_from_slice(&u16le(13)); // 문단 끝
    p
}

fn parse_note(bracket: u8, lead: Option<char>, body: &str) -> rhwp::model::document::Document {
    let bytes = hwp3_doc(bracket, &hwp3_body(&footnote_paragraph(lead, body)));
    rhwp::parser::hwp3::parse_hwp3(&bytes).expect("합성 HWP3 파싱 실패")
}

/// 첫 각주의 첫 본문 문단을 꺼낸다.
fn first_note_paragraph(doc: &rhwp::model::document::Document) -> &Paragraph {
    doc.sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .flat_map(|p| p.controls.iter())
        .find_map(|c| match c {
            Control::Footnote(note) => note.paragraphs.first(),
            Control::Endnote(note) => note.paragraphs.first(),
            _ => None,
        })
        .expect("합성 문서에서 각주 본문 문단을 못 찾았다 — 대상 0건은 통과 증거가 아니다")
}

/// 합성 문서가 실제로 "번호 코드 + 글자" 형상인지 먼저 확인한다.
#[test]
fn synthetic_hwp3_note_has_the_auto_number_placeholder() {
    let doc = parse_note(NUMBER_SUFFIX as u8, Some(NUMBER_SUFFIX), "A");
    let para = first_note_paragraph(&doc);
    assert!(
        matches!(
            para.controls.first(),
            Some(Control::AutoNumber(an)) if an.number_type == AutoNumberType::Footnote
        ),
        "합성 전제가 깨졌다 — 각주 본문 첫 컨트롤이 각주 자동 번호가 아니다"
    );
    // 자리표시자는 글자 하나지만 저장본에서 확장 컨트롤 8 코드유닛을 차지한다(#3504).
    assert_eq!(
        para.char_offsets.first().copied(),
        Some(0),
        "합성 전제가 깨졌다 — 자동 번호 자리표시자가 문단 맨 앞이 아니다"
    );
    assert_eq!(
        para.char_offsets.get(1).copied(),
        Some(8),
        "합성 전제가 깨졌다 — 자동 번호가 8 코드유닛을 차지하지 않는다"
    );
}

/// 문서가 장식을 켰고 주석 첫 글자가 그 장식이면 한 글자를 뗀다.
#[test]
fn hwp3_note_body_drops_the_duplicated_number_suffix() {
    let doc = parse_note(NUMBER_SUFFIX as u8, Some(NUMBER_SUFFIX), "A");
    let para = first_note_paragraph(&doc);
    assert_eq!(
        para.text.chars().nth(1),
        Some('A'),
        "주석 본문에 장식 리터럴이 남았다 — 각주 모양과 겹쳐 번호가 두 번 그려진다"
    );
    assert_eq!(
        para.char_offsets.len(),
        para.text.chars().count(),
        "리터럴을 뗀 뒤 char_offsets 길이가 글자 수와 어긋났다"
    );
    assert!(
        para.char_offsets.windows(2).all(|w| w[0] < w[1]),
        "리터럴을 뗀 뒤 char_offsets 가 단조증가가 아니다"
    );
}

/// 반례 1 — 문서가 장식을 끄면 같은 글자라도 본문 글자로 남긴다.
#[test]
fn hwp3_note_body_keeps_the_char_when_the_document_turns_decoration_off() {
    let doc = parse_note(0, Some(NUMBER_SUFFIX), "A");
    let para = first_note_paragraph(&doc);
    assert_eq!(
        para.text.chars().nth(1),
        Some(NUMBER_SUFFIX),
        "장식을 끈 문서에서 본문 글자를 떼면 안 된다"
    );
}

/// 반례 2 — 장식을 켰어도 주석 첫 글자가 그 장식이 아니면 떼지 않는다.
#[test]
fn hwp3_note_body_keeps_a_different_leading_char() {
    let doc = parse_note(NUMBER_SUFFIX as u8, Some('.'), "A");
    let para = first_note_paragraph(&doc);
    assert_eq!(
        para.text.chars().nth(1),
        Some('.'),
        "장식과 다른 글자를 떼면 안 된다"
    );
}
