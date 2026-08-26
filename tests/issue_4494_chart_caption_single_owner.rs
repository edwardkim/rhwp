//! [Issue #4494] HWP5 Chart 캡션 레코드의 직렬화 소유자는 하나여야 한다.
//!
//! `serialize_shape_control` 은 이미 #2715 에서 추가한 공통 헬퍼로 Chart 캡션을
//! `SHAPE_COMPONENT` 앞에 방출한다. 나중에 추가된 Chart 전용 호출이 중첩 컨트롤을
//! 포함한 동일 캡션 하위 트리를 한 번 더 방출했다.

use rhwp::model::control::{Control, Field, FieldType};
use rhwp::model::document::Section;
use rhwp::model::paragraph::{CharShapeRef, FieldRange, LineSeg, Paragraph};
use rhwp::model::shape::{Caption, ChartShape, ShapeObject};
use rhwp::parser::body_text::parse_body_text_section;
use rhwp::parser::{record::Record, tags};
use rhwp::serializer::body_text::serialize_section;
use rhwp::serializer::control::serialize_control;

const CAPTION_TEXT: &str = "차트 캡션 4494";

fn char_offsets_after(prefix: u32, text: &str) -> Vec<u32> {
    let mut offset = prefix;
    text.chars()
        .map(|ch| {
            let current = offset;
            offset += ch.len_utf16() as u32;
            current
        })
        .collect()
}

fn caption_with_nested_field() -> Caption {
    // 빈 Field 는 첫 캡션 글자 앞에서 FIELD_BEGIN/FIELD_END 각 8 WCHAR,
    // 합계 16 WCHAR 슬롯을 차지한다.
    let field_slots = 16;
    Caption {
        paragraphs: vec![Paragraph {
            char_count: field_slots + CAPTION_TEXT.encode_utf16().count() as u32 + 1,
            text: CAPTION_TEXT.to_string(),
            char_offsets: char_offsets_after(field_slots, CAPTION_TEXT),
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                text_start: 0,
                line_height: 1000,
                text_height: 700,
                ..Default::default()
            }],
            controls: vec![Control::Field(Field {
                field_type: FieldType::Hyperlink,
                command: "https://example.invalid/issue/4494".to_string(),
                field_id: 4494,
                ctrl_id: tags::FIELD_HYPERLINK,
                ..Default::default()
            })],
            field_ranges: vec![FieldRange {
                start_char_idx: 0,
                end_char_idx: 0,
                control_idx: 0,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn chart_control(caption: Option<Caption>) -> Control {
    Control::Shape(Box::new(ShapeObject::Chart(Box::new(ChartShape {
        caption,
        raw_chart_data: vec![0x44, 0x94],
        ..Default::default()
    }))))
}

fn control_id(record: &Record) -> Option<u32> {
    let bytes: [u8; 4] = record.data.get(..4)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

fn serialized_control(caption: Option<Caption>) -> Vec<Record> {
    let mut records = Vec::new();
    serialize_control(&chart_control(caption), 1, None, &mut records);
    records
}

#[test]
fn issue_4494_chart_emits_one_caption_record_subtree() {
    let records = serialized_control(Some(caption_with_nested_field()));
    let shape_component = records
        .iter()
        .position(|record| record.tag_id == tags::HWPTAG_SHAPE_COMPONENT)
        .expect("Chart SHAPE_COMPONENT");
    let caption_records = &records[1..shape_component];

    assert_eq!(
        caption_records
            .iter()
            .filter(|record| record.tag_id == tags::HWPTAG_LIST_HEADER)
            .count(),
        1,
        "Chart CTRL_HEADER와 SHAPE_COMPONENT 사이에는 캡션 LIST_HEADER가 하나여야 한다"
    );
    assert_eq!(
        caption_records
            .iter()
            .filter(|record| {
                record.tag_id == tags::HWPTAG_CTRL_HEADER
                    && control_id(record) == Some(tags::FIELD_HYPERLINK)
            })
            .count(),
        1,
        "캡션과 함께 중첩 Field CTRL_HEADER도 한 번만 방출되어야 한다"
    );
}

#[test]
fn issue_4494_chart_caption_roundtrip_preserves_text_and_one_nested_field() {
    let section = Section {
        paragraphs: vec![Paragraph {
            char_count: 9,
            controls: vec![chart_control(Some(caption_with_nested_field()))],
            ..Default::default()
        }],
        raw_stream: None,
        ..Default::default()
    };

    let bytes = serialize_section(&section);
    let parsed = parse_body_text_section(&bytes).expect("serialized Chart section parses");
    let chart = match parsed.paragraphs[0].controls.first() {
        Some(Control::Shape(shape)) => match shape.as_ref() {
            ShapeObject::Chart(chart) => chart,
            other => panic!("Chart expected, got {other:?}"),
        },
        other => panic!("Shape control expected, got {other:?}"),
    };
    let caption = chart.caption.as_ref().expect("Chart caption roundtrips");

    assert_eq!(caption.paragraphs.len(), 1, "caption paragraph count");
    assert_eq!(caption.paragraphs[0].text, CAPTION_TEXT, "caption text");
    assert_eq!(
        caption
            .paragraphs
            .iter()
            .flat_map(|paragraph| &paragraph.controls)
            .filter(|control| matches!(control, Control::Field(_)))
            .count(),
        1,
        "nested Field control count"
    );
}

#[test]
fn issue_4494_captionless_chart_record_order_is_unchanged() {
    let signature: Vec<(u16, u16)> = serialized_control(None)
        .iter()
        .map(|record| (record.tag_id, record.level))
        .collect();

    assert_eq!(
        signature,
        vec![
            (tags::HWPTAG_CTRL_HEADER, 1),
            (tags::HWPTAG_SHAPE_COMPONENT, 2),
            (tags::HWPTAG_CHART_DATA, 3),
        ],
        "captionless Chart의 기존 레코드 순서에는 변화가 없어야 한다"
    );
}
