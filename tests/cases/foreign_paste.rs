//! 서식표가 다른 외부 문서를 커서에 끼워 넣는 경로의 계약.
//! 공개 API 만 쓴다 — 붙여넣은 결과를 HWPX 로 내보내 그 XML 로 확인한다.

use std::io::Read;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::model::style::{
    Alignment, BorderFill, BorderLine, BorderLineType, CharShape, Font, ParaShape,
};
use rhwp::model::table::{Cell, Table};

fn foreign_font_slots() -> Vec<Vec<Font>> {
    (0..7)
        .map(|slot| {
            vec![Font {
                name: format!("외부글꼴{}", slot),
                ..Default::default()
            }]
        })
        .collect()
}

fn text_para(text: &str, char_shape_id: u32, para_shape_id: u16) -> Paragraph {
    let count = text.chars().count() as u32;
    Paragraph {
        text: text.to_string(),
        char_count: count + 1,
        char_offsets: (0..count).collect(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id,
        }],
        para_shape_id,
        has_para_text: true,
        ..Default::default()
    }
}

fn foreign_with_own_shapes() -> Document {
    let mut doc = Document::default();
    doc.doc_info.font_faces = foreign_font_slots();
    doc.doc_info.char_shapes.push(CharShape {
        base_size: 1400,
        bold: true,
        font_ids: [0; 7],
        ..Default::default()
    });
    doc.doc_info.para_shapes.push(ParaShape {
        alignment: Alignment::Center,
        margin_left: 1234,
        ..Default::default()
    });
    doc.sections.push(Section {
        paragraphs: vec![text_para("첫줄", 0, 0), text_para("둘째줄", 0, 0)],
        ..Default::default()
    });
    doc
}

fn foreign_with_table() -> Document {
    let mut doc = Document::default();
    doc.doc_info.font_faces = foreign_font_slots();
    doc.doc_info.char_shapes.push(CharShape {
        base_size: 900,
        font_ids: [0; 7],
        ..Default::default()
    });
    doc.doc_info.para_shapes.push(ParaShape::default());
    doc.doc_info.border_fills.push(BorderFill {
        borders: [BorderLine {
            line_type: BorderLineType::Solid,
            width: 3,
            color: 0x00_FF_00,
        }; 4],
        ..Default::default()
    });

    let mut table = Table {
        row_count: 1,
        col_count: 1,
        row_sizes: vec![1_000],
        border_fill_id: 1,
        ..Default::default()
    };
    table.cells = vec![Cell {
        row: 0,
        col: 0,
        row_span: 1,
        col_span: 1,
        width: 5_000,
        height: 1_000,
        border_fill_id: 1,
        paragraphs: vec![text_para("셀글자", 0, 0)],
        ..Default::default()
    }];
    table.update_ctrl_dimensions();
    table.rebuild_grid();

    let mut host = Paragraph::new_empty();
    host.controls.push(Control::Table(Box::new(table)));
    doc.sections.push(Section {
        paragraphs: vec![host],
        ..Default::default()
    });
    doc
}

/// 붙여넣은 결과를 HWPX 로 내보내 (header.xml, section0.xml) 을 돌려준다.
fn exported_xml(core: &DocumentCore) -> (String, String) {
    let bytes = core.export_hwpx_native().expect("HWPX 내보내기");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("HWPX 는 zip 이다");
    let mut read = |name: &str| {
        let mut s = String::new();
        zip.by_name(name)
            .unwrap_or_else(|_| panic!("{name} 이 없다"))
            .read_to_string(&mut s)
            .expect("UTF-8");
        s
    };
    let header = read("Contents/header.xml");
    let section = read("Contents/section0.xml");
    (header, section)
}

fn pasted(foreign: Document) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let result = core
        .paste_foreign_document_native(0, 0, 0, foreign)
        .expect("외부 문서 붙여넣기");
    assert!(result.contains("\"ok\":true"), "{result}");
    core
}

/// 외부 서식이 그대로 보존되는가 — 외부 id 를 그냥 옮기면 현재 문서의 같은 번호
/// (바탕글·기본 글꼴)로 읽혀 이 시험이 깨진다.
#[test]
fn foreign_paste_preserves_char_and_para_shape() {
    let core = pasted(foreign_with_own_shapes());
    let (header, section) = exported_xml(&core);

    assert!(
        section.contains("둘째줄"),
        "삽입된 문단이 본문에 있어야 한다"
    );
    // 🔴 언어 슬롯마다 목록이 다르다 — 슬롯별로 따로 대응돼야 각 구간이 제 글꼴을 쓴다.
    for slot in 0..7 {
        let name = format!("외부글꼴{slot}");
        assert!(
            header.contains(&name),
            "글꼴 슬롯 {slot} 이 옮겨지지 않았다"
        );
    }
    assert!(
        header.contains("height=\"1400\""),
        "외부 글자모양(14pt)이 보존돼야 한다"
    );
    assert!(
        header.contains("value=\"1234\""),
        "외부 문단모양(왼쪽 여백 1234)이 보존돼야 한다"
    );
}

/// 같은 조각을 두 번 붙여넣어도 서식표가 늘지 않아야 한다(값이 같으면 재사용).
#[test]
fn foreign_paste_reuses_identical_shapes() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.paste_foreign_document_native(0, 0, 0, foreign_with_own_shapes())
        .unwrap();
    let (first, _) = exported_xml(&core);

    core.paste_foreign_document_native(0, 0, 0, foreign_with_own_shapes())
        .unwrap();
    let (second, _) = exported_xml(&core);

    for tag in [
        "hh:charProperties itemCnt=",
        "hh:paraProperties itemCnt=",
        "hh:fontfaces itemCnt=",
    ] {
        let pick = |xml: &String| {
            let i = xml.find(tag).unwrap_or_else(|| panic!("{tag} 가 없다"));
            xml[i..i + tag.len() + 8].to_string()
        };
        assert_eq!(pick(&first), pick(&second), "{tag} 가 늘었다");
    }
}

/// 표가 든 외부 문서 — 표 구조·셀 문단·셀 서식 참조가 모두 살아 있어야 한다.
#[test]
fn foreign_paste_keeps_table_and_cell_paragraphs() {
    let core = pasted(foreign_with_table());
    let (header, section) = exported_xml(&core);

    assert!(
        section.contains("<hp:tbl"),
        "붙여넣은 표가 본문에 있어야 한다"
    );
    assert!(section.contains("셀글자"), "셀 안 문단이 살아 있어야 한다");
    // 셀 안 글자모양도 함께 이식돼야 한다(재귀 순회가 빠지면 여기서 깨진다).
    assert!(header.contains("height=\"900\""), "셀 글자모양(9pt)");
    assert!(header.contains("외부글꼴0"), "셀 글꼴");
}
