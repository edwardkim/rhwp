//! 개체의 원시 자리표와 실제 U+FFFC 본문 글자를 구분하는 읽기 계약.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::{Cell, Table};

fn table(label: &str) -> Control {
    Control::Table(Box::new(Table {
        row_count: 1,
        col_count: 1,
        row_sizes: vec![1],
        cells: vec![Cell {
            paragraphs: vec![Paragraph {
                text: label.into(),
                char_count: label.chars().count() as u32 + 1,
                ..Paragraph::new_empty()
            }],
            ..Cell::new_empty(0, 0, 1000, 1000, 0)
        }],
        ..Default::default()
    }))
}

fn source(text: &str, offsets: Vec<u32>, count: u32, controls: Vec<Control>) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서");
    let section = &mut core.document_mut().sections[0];
    section.raw_stream = None;
    section.paragraphs.truncate(1);
    section.paragraphs.push(Paragraph {
        text: text.into(),
        char_offsets: offsets,
        char_count: count,
        controls,
        ..Paragraph::new_empty()
    });
    core
}

fn text(core: &DocumentCore) -> String {
    serde_json::from_str(&core.text_file_unicode_json()).expect("문서 글 JSON")
}

#[test]
fn owned_markers_are_not_reemitted_after_nested_content_or_shared_anchors() {
    let cases = [
        source("\u{fffc}", vec![0], 9, vec![table("첫 표")]),
        source(
            "앞\u{fffc}뒤",
            vec![8, 9, 17],
            19,
            vec![Control::Header(Default::default()), table("첫 표")],
        ),
        source(
            "\u{fffc}",
            vec![16],
            25,
            vec![
                Control::Header(Default::default()),
                Control::Footer(Default::default()),
                table("첫 표"),
            ],
        ),
        source(
            "\u{fffc}\u{fffc}",
            vec![0, 8],
            17,
            vec![table("첫 표"), table("둘째 표")],
        ),
    ];
    for core in cases {
        let before = core.document().sections[0].paragraphs[1].text.clone();
        let extracted = text(&core);
        assert!(!extracted.contains('\u{fffc}'), "{extracted:?}");
        assert_eq!(extracted.matches("첫 표").count(), 1);
        assert_eq!(core.document().sections[0].paragraphs[1].text, before);
        let scan = core.scan_items_json();
        assert!(!scan.contains('\u{fffc}'), "{scan}");
        let reopened = DocumentCore::from_bytes(&core.export_hwp_native().expect("HWP 저장"))
            .expect("HWP 재열기");
        assert_eq!(text(&reopened), extracted);
    }
}

#[test]
fn literal_markers_and_incomplete_or_reversed_slots_remain_text() {
    let cases = [
        source("\u{fffc}", vec![0], 2, vec![]),
        source("\u{fffc}", vec![8], 10, vec![table("표")]),
        source("\u{fffc}", vec![0], 0, vec![table("표")]),
        source("\u{fffc}뒤", vec![8, 1], 3, vec![table("표")]),
        source("\u{fffc}", vec![], 9, vec![table("표")]),
    ];
    for core in cases {
        assert_eq!(text(&core).matches('\u{fffc}').count(), 1);
    }
}
