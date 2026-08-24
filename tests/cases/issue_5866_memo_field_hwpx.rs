//! [#5866] HWP5 메모 필드(command `MEMO/…`)를 HWPX 로 저장할 때 CROSSREF 로
//! 굳히지 않고 한글 실측 형상(type="MEMO" + 파라미터 6종 + 빈 subList)으로 낸다.
//!
//! HWP5 는 메모의 정체성을 ctrl_id 가 아니라 command 문자열로 들고 있어 IR 은
//! `FieldType::Unknown` 이다. 종전 대체값 `CROSSREF` 는 필드 범위 숨김을 풀어
//! **메모 대상 텍스트가 본문 문장에 붙었다**(07868·08040·02302). 한글 2024 SaveAs
//! HWPX 실측(07868)을 미러한 형상은 한글이 정상 개방(131쪽)·추출 경계까지 원본과
//! 동형이고, 02302·08040 은 한글 추출이 원본과 **바이트 동일**해진다.

use std::io::Read;

use rhwp::model::control::{Control, Field, FieldType};
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{CharShapeRef, FieldRange, Paragraph};
use rhwp::serializer::hwpx::serialize_hwpx;

const MEMO_COMMAND: &str = r"MEMO/65535/1/3929787472/30641593/shinhokim_2/\;;";

fn section0_xml(hwpx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("HWPX zip 열기");
    let mut entry = zip
        .by_name("Contents/section0.xml")
        .expect("Contents/section0.xml");
    let mut xml = String::new();
    entry.read_to_string(&mut xml).expect("section0.xml 읽기");
    xml
}

fn document_with_memo_field() -> Document {
    let mut doc = Document::default();
    doc.doc_info.char_shapes = vec![Default::default()];
    let para = Paragraph {
        text: "메모 대상".to_string(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        controls: vec![Control::Field(Field {
            field_type: FieldType::Unknown,
            command: MEMO_COMMAND.to_string(),
            field_id: 1681818964,
            ..Default::default()
        })],
        field_ranges: vec![FieldRange {
            start_char_idx: 0,
            end_char_idx: 4,
            control_idx: 0,
            ..Default::default()
        }],
        ..Default::default()
    };
    doc.sections.push(Section {
        paragraphs: vec![para],
        ..Default::default()
    });
    doc
}

#[test]
fn issue_5866_memo_command_emits_memo_field_shape() {
    let xml = section0_xml(&serialize_hwpx(&document_with_memo_field()).expect("HWPX 직렬화"));
    assert!(
        xml.contains(r#"type="MEMO""#),
        "#5866: MEMO/ command 필드는 type=\"MEMO\" 로 나가야 한다\n{xml}"
    );
    assert!(
        !xml.contains(r#"type="CROSSREF""#),
        "#5866: CROSSREF 대체가 남아 있으면 안 된다\n{xml}"
    );
    // 한글 실측 파라미터 — 전부 command 토큰에서 유도된다.
    for needle in [
        r#"<hp:stringParam name="Command">MEMO/65535/1/3929787472/30641593/shinhokim_2/\;;</hp:stringParam>"#,
        r#"<hp:stringParam name="ID">memo1</hp:stringParam>"#,
        r#"<hp:integerParam name="Number">1</hp:integerParam>"#,
        r#"<hp:stringParam name="Author">shinhokim_2</hp:stringParam>"#,
        r#"<hp:stringParam name="MemoShapeIDRef">65535</hp:stringParam>"#,
    ] {
        assert!(
            xml.contains(needle),
            "#5866: {needle} 이 있어야 한다\n{xml}"
        );
    }
    // 빈 subList — 이것이 없으면 한글이 파일을 열지 못한다(이슈 실측).
    let begin = xml.find(r#"type="MEMO""#).unwrap();
    let field_end = xml[begin..]
        .find("</hp:fieldBegin>")
        .map(|i| begin + i)
        .expect("fieldBegin 닫는 태그");
    let inner = &xml[begin..field_end];
    assert!(
        inner.contains("<hp:subList") && inner.contains("</hp:subList>"),
        "#5866: fieldBegin 안에 빈 subList 가 있어야 한다\n{inner}"
    );
}

/// 실물 재현 — 02302 (한글 추출이 원본과 바이트 동일해진 문서). HWP5 파싱 →
/// HWPX 직렬화에서 메모 필드가 MEMO 로 나가고 CROSSREF 대체가 사라진다.
#[test]
fn issue_5866_real_hwp5_memo_document_emits_memo_not_crossref() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5866/memo_field_hwp5.hwp");
    let bytes = std::fs::read(&path).expect("read sample");
    let doc = rhwp::parse_document(&bytes).expect("parse hwp5");
    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    // 메모가 어느 구역에 있든 잡히도록 전 구역을 잇는다.
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&hwpx[..])).expect("HWPX zip 열기");
    let names: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .filter(|n| n.starts_with("Contents/section") && n.ends_with(".xml"))
        .collect();
    let mut xml = String::new();
    for name in names {
        zip.by_name(&name)
            .unwrap()
            .read_to_string(&mut xml)
            .expect("section xml 읽기");
    }
    assert!(
        xml.contains(r#"type="MEMO""#),
        "#5866: 02302 의 메모 필드가 MEMO 로 나가야 한다"
    );
    assert!(
        !xml.contains(r#"type="CROSSREF""#),
        "#5866: 02302 산출에 CROSSREF 가 남으면 안 된다 (원본 HWPX 에는 CROSSREF 0)"
    );
}

#[test]
fn issue_5866_non_memo_unknown_field_keeps_crossref_fallback() {
    let mut doc = document_with_memo_field();
    if let Some(Control::Field(field)) = doc.sections[0].paragraphs[0].controls.get_mut(0) {
        field.command = "SOMETHING/ELSE".to_string();
    }
    let xml = section0_xml(&serialize_hwpx(&doc).expect("HWPX 직렬화"));
    assert!(
        xml.contains(r#"type="CROSSREF""#),
        "#5866 회귀 가드: MEMO 가 아닌 Unknown 필드는 종전 CROSSREF 대체 유지\n{xml}"
    );
}
