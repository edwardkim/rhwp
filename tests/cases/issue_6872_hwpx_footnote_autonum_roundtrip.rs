//! [#6872] HWPX→HWPX 왕복에서 각주 번호 서식이 무너진다.
//!
//! **증상.** `x2x` 왕복만으로 각주 참조 표시가 바뀐다(쪽수·글자수는 그대로).
//!
//! ```text
//!   02333  '전자·통신1)1) …'  →  '전자·통신2)2) …'      번호가 밀린다
//!   00931  '(증가** 표시는 …'  →  '(증가*)*) 표시는 …'   표시가 * → *)
//! ```
//!
//! **근인은 넷이고 전부 필드 단위 왕복 누락이다.** 원본 ↔ 왕복 XML 대조 실측:
//!
//! ```text
//!  ① <hp:numbering type="ON_PAGE"/>            → "RESTART_PAGE"
//!  ② <hp:footNote … userChar="42">             → suffixChar="41"
//!  ③ <hp:autoNumFormat type="USER_CHAR" …>     → type="DIGIT"       (인라인 autoNum)
//!  ④ <hp:autoNumFormat … suffixChar="">        → suffixChar=")"     (노트 모양)
//! ```
//!
//! ① 파서는 `ON_PAGE`·`RESTART_PAGE` 를 둘 다 받지만 직렬화기는 `RESTART_*` 만 냈다.
//!    한컴이 쓴 HWPX **3,418건**을 전수로 세면 `CONTINUOUS` 7,766 · `ON_PAGE` 12 이고
//!    `RESTART_*` 는 **하나도 없다**.
//!
//! ② 번호 모양이 사용자 기호일 때 한컴은 `suffixChar` 대신 `userChar` 에 기호를 싣는다.
//!    파서가 그 속성을 읽지 않아 값이 사라지고, 직렬화기가 기본값 `)`(0x29)를 채웠다.
//!
//! ③ 인라인 `<hp:autoNum>` 의 형식 표는 쪽 번호용 `0..7` 만 알아 `USER_CHAR` 가 `_ => 0`
//!    (DIGIT)로 떨어졌다.
//!
//! ④ 사용자 기호 모양은 표시가 기호 하나로 끝나므로 한컴도 `suffixChar` 를 비운다.
//!    코퍼스 노트 모양 7,778개 중 `suffixChar=""` 는 1개이고 그것이 유일한 `USER_CHAR`
//!    다. 숫자 계열(7,651개가 `)`)의 폴백은 건드리지 않는다.
//!
//! 이 시험은 합성 IR 로 **네 지점의 방출 계약**만 잰다.

use rhwp::model::footnote::{Footnote, FootnoteNumbering, FootnoteShape, NumberFormat};

/// 각주 하나를 담은 최소 문서를 HWPX 로 저장하고 `section0.xml` 을 돌려준다.
fn section_xml(build: impl FnOnce(&mut rhwp::model::document::Document)) -> String {
    let mut doc = rhwp::model::document::Document::default();
    // 구역 하나 · 문단 하나짜리 최소 문서.
    let mut section = rhwp::model::document::Section::default();
    section
        .paragraphs
        .push(rhwp::model::paragraph::Paragraph::default());
    doc.sections.push(section);
    build(&mut doc);
    let bytes = rhwp::serializer::hwpx::serialize_hwpx(&doc).expect("HWPX 직렬화는 성공해야 한다");
    let mut zip =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("산출물은 zip 이어야 한다");
    let mut out = String::new();
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).expect("zip 항목");
        if f.name().to_ascii_lowercase().contains("section0") {
            use std::io::Read;
            f.read_to_string(&mut out).expect("section0 읽기");
            break;
        }
    }
    assert!(!out.is_empty(), "section0.xml 을 찾지 못했다");
    out
}

/// 첫 구역의 각주 모양을 고쳐 준다.
fn with_footnote_shape(
    doc: &mut rhwp::model::document::Document,
    edit: impl FnOnce(&mut FootnoteShape),
) {
    let section = doc.sections.first_mut().expect("구역이 하나는 있어야 한다");
    edit(&mut section.section_def.footnote_shape);
}

#[test]
fn issue_6872_note_numbering_uses_hancom_on_page_token() {
    // **뒤집힘 ①** — 한컴이 쓰지 않는 `RESTART_*` 를 내보내지 않는다.
    let xml = section_xml(|doc| {
        with_footnote_shape(doc, |shape| {
            shape.numbering = FootnoteNumbering::RestartPage;
        });
    });
    assert!(
        xml.contains(r#"<hp:numbering type="ON_PAGE""#),
        "각주 번호 매기기는 ON_PAGE 로 나가야 한다:\n{}",
        &xml[..xml.len().min(400)]
    );
    assert!(
        !xml.contains("RESTART_PAGE"),
        "RESTART_PAGE 는 한컴 저장본에 없는 토큰이다"
    );
}

#[test]
fn issue_6872_continuous_numbering_is_unchanged() {
    // **음성 대조 ①** — 코퍼스 7,766개가 쓰는 기본 토큰은 그대로다.
    let xml = section_xml(|doc| {
        with_footnote_shape(doc, |shape| {
            shape.numbering = FootnoteNumbering::Continue;
        });
    });
    assert!(
        xml.contains(r#"<hp:numbering type="CONTINUOUS""#),
        "CONTINUOUS 는 불변이어야 한다"
    );
}

#[test]
fn issue_6872_user_char_decoration_round_trips_as_user_char() {
    // **뒤집힘 ②** — `userChar` 로 들어온 장식 문자는 같은 이름으로 나간다.
    let xml = section_xml(|doc| {
        let mut note = Footnote {
            number: 1,
            after_decoration_letter: 0x2A, // '*'
            decoration_is_user_char: true,
            number_shape: 385,
            instance_id: 2_004_981_886,
            ..Default::default()
        };
        note.paragraphs
            .push(rhwp::model::paragraph::Paragraph::default());
        let section = doc.sections.first_mut().expect("구역");
        let para = section
            .paragraphs
            .first_mut()
            .expect("문단이 하나는 있어야 한다");
        para.controls
            .push(rhwp::model::control::Control::Footnote(Box::new(note)));
    });
    assert!(
        xml.contains(r#"userChar="42""#),
        "사용자 기호는 userChar 로 나가야 한다:\n{}",
        &xml[..xml.len().min(600)]
    );
    assert!(
        !xml.contains(r#"suffixChar="42""#),
        "이름을 suffixChar 로 바꾸면 표시가 *) 가 된다"
    );
}

#[test]
fn issue_6872_plain_decoration_still_uses_suffix_char() {
    // **음성 대조 ②** — 종전 경로(장식 문자 `)`)는 이름이 바뀌지 않는다.
    let xml = section_xml(|doc| {
        let mut note = Footnote {
            number: 1,
            after_decoration_letter: 0x29, // ')'
            instance_id: 1_282_670_874,
            ..Default::default()
        };
        note.paragraphs
            .push(rhwp::model::paragraph::Paragraph::default());
        let section = doc.sections.first_mut().expect("구역");
        let para = section.paragraphs.first_mut().expect("문단");
        para.controls
            .push(rhwp::model::control::Control::Footnote(Box::new(note)));
    });
    assert!(
        xml.contains(r#"suffixChar="41""#),
        "일반 장식 문자는 suffixChar 그대로다:\n{}",
        &xml[..xml.len().min(600)]
    );
    assert!(!xml.contains(r#"userChar="41""#), "이름이 새면 안 된다");
}

#[test]
fn issue_6872_user_char_shape_leaves_the_suffix_empty() {
    // **뒤집힘 ④** — 사용자 기호 모양에는 닫는 장식이 붙지 않는다.
    let xml = section_xml(|doc| {
        with_footnote_shape(doc, |shape| {
            shape.number_format = NumberFormat::UserChar;
            shape.user_char = '*';
            shape.suffix_char = '\0';
        });
    });
    assert!(
        xml.contains(r#"type="USER_CHAR" userChar="*" prefixChar="" suffixChar="""#),
        "USER_CHAR 모양의 suffixChar 는 비어야 한다:\n{}",
        &xml[..xml.len().min(600)]
    );
}

#[test]
fn issue_6872_digit_shape_keeps_the_paren_suffix() {
    // **음성 대조 ④** — 코퍼스 7,651개가 쓰는 숫자 모양 폴백은 불변이다.
    let xml = section_xml(|doc| {
        with_footnote_shape(doc, |shape| {
            shape.number_format = NumberFormat::Digit;
            shape.suffix_char = '\0';
        });
    });
    assert!(
        xml.contains(r#"type="DIGIT" userChar="" prefixChar="" suffixChar=")""#),
        "숫자 모양은 종전대로 ) 를 남긴다:\n{}",
        &xml[..xml.len().min(600)]
    );
}
