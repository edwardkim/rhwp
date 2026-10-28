//! [#7216] scaffold 표는 쪽 경계에서 나뉘고, IR·HWPX·HWP5 의 쪽나눔 표기가 한 값이다.
//!
//! 종전 scaffold 는 IR `page_break: None`(나누지 않음)·`repeat_header: false` 로 표를 만들면서
//! HWP5 raw TABLE attr 은 `0x06`(= 나눔 행 단위 0x02 + 제목 반복 0x04)으로 두었다. HWPX 는
//! IR 을 따라 `pageBreak="NONE" repeatHeader="0"` 을 내어, 한글이 긴 표를 통째로 두고 본문
//! 아래로 넘친 행을 잃었다(한글 2020 PDF: 60행 표가 `설명 53` 에서 끊김).
//!
//! 기대값 근거 — 한글 새 표의 사실상 기본값: 코퍼스 HWPX 표 20,405개 중 13,299개가
//! `pageBreak="CELL" repeatHeader="1"`. HWPX `CELL` 은 IR `RowBreak`, HWP5 attr bits 0-1 = 2.
//! 수정 후 한글 2020 PDF: 60행 전부 표시(47행부터 2쪽), 짧은 표는 수정 전과 화소 동일.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::table::{Table, TablePageBreak};
use rhwp::scaffold::{build_scaffold, parse_scaffold_str};
use rhwp::serializer::{serialize_document, serialize_hwpx};

fn spec(rows: usize) -> String {
    let mut table_rows = vec![r#"["No","항목","설명"]"#.to_string()];
    table_rows.extend((1..=rows).map(|i| format!(r#"["{i}","항목 {i}","설명 {i}"]"#)));
    format!(
        r#"{{"version":"1","title":"표 쪽 넘김","blocks":[
            {{"type":"paragraph","text":"{rows}행 표"}},
            {{"type":"table","rows":[{}]}},
            {{"type":"paragraph","text":"표 다음 문단"}}
        ]}}"#,
        table_rows.join(",")
    )
}

fn only_table(doc: &Document) -> &Table {
    let tables: Vec<&Table> = doc.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(tables.len(), 1, "표는 하나여야 한다");
    tables[0]
}

/// HWP5 TABLE attr 을 파서 규칙(`bits 0-1`: 1=셀 단위, 2=나눔 행 단위 · `bit 2`: 제목 반복)으로 읽는다.
fn attr_page_break(attr: u32) -> (TablePageBreak, bool) {
    let page_break = match attr & 0x03 {
        1 => TablePageBreak::CellBreak,
        2 => TablePageBreak::RowBreak,
        _ => TablePageBreak::None,
    };
    (page_break, attr & 0x04 != 0)
}

#[test]
fn issue_7216_scaffold_table_breaks_at_page_boundary_with_consistent_attr() {
    let spec = parse_scaffold_str(&spec(60)).expect("scaffold 명세");
    let doc = build_scaffold(&spec);
    let table = only_table(&doc);

    assert_eq!(
        table.page_break,
        TablePageBreak::RowBreak,
        "scaffold 표는 쪽 경계에서 나뉘어야 한다(HWPX CELL)"
    );
    assert!(
        table.repeat_header,
        "한글 새 표 기본값처럼 제목 반복을 켠다"
    );
    assert_eq!(
        attr_page_break(table.raw_table_record_attr),
        (table.page_break, table.repeat_header),
        "IR 과 HWP5 raw TABLE attr 이 같은 쪽나눔을 말해야 한다 (attr={:#x})",
        table.raw_table_record_attr
    );
}

#[test]
fn issue_7216_hwpx_and_hwp5_round_trips_keep_the_same_page_break() {
    let spec = parse_scaffold_str(&spec(60)).expect("scaffold 명세");
    let doc = build_scaffold(&spec);

    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let section_xml = {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&hwpx)).expect("zip");
        let mut entry = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut entry, &mut xml).expect("xml");
        xml
    };
    let tbl_tag = section_xml
        .split("<hp:tbl ")
        .nth(1)
        .and_then(|rest| rest.split('>').next())
        .expect("hp:tbl");
    assert!(
        tbl_tag.contains(r#"pageBreak="CELL""#) && tbl_tag.contains(r#"repeatHeader="1""#),
        "HWPX 표 속성이 CELL/1 이어야 한다: <hp:tbl {tbl_tag}>"
    );

    let from_hwpx = rhwp::parser::parse_document(&hwpx).expect("HWPX 재파싱");
    let table = only_table(&from_hwpx);
    assert_eq!(
        (table.page_break, table.repeat_header),
        (TablePageBreak::RowBreak, true)
    );

    let hwp5 = serialize_document(&from_hwpx).expect("HWP5 직렬화");
    let from_hwp5 = rhwp::parser::parse_document(&hwp5).expect("HWP5 재파싱");
    let table = only_table(&from_hwp5);
    assert_eq!(
        (table.page_break, table.repeat_header),
        (TablePageBreak::RowBreak, true),
        "HWP5 변환 뒤에도 같은 쪽나눔이어야 한다"
    );
    assert_eq!(
        attr_page_break(table.raw_table_record_attr),
        (TablePageBreak::RowBreak, true)
    );
}

#[test]
fn issue_7216_short_table_still_fits_on_one_page() {
    let spec = parse_scaffold_str(&spec(3)).expect("scaffold 명세");
    let doc = build_scaffold(&spec);
    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let core = rhwp::document_core::DocumentCore::from_bytes(&hwpx).expect("열기");
    assert_eq!(
        core.page_count(),
        1,
        "한 쪽에 들어가는 짧은 표는 쪽이 늘지 않는다"
    );
}
