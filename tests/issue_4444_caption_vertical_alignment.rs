//! Issue #4444: HWPX 캡션 `hp:subList@vertAlign` 왕복 회귀.
//!
//! 표와 사각형 캡션이 공유하는 공개 HWPX 저장→파싱 경로에서 세 정렬값을 검증한다.
//! 정렬만 얕게 비교하지 않도록 캡션 기하 전체와 비기본 문단 sentinel도 함께 고정한다.

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::shape::{
    Caption, CaptionDirection, CaptionVertAlign, RectangleShape, ShapeObject,
};
use rhwp::model::style::{CharShape, ParaShape, Style};
use rhwp::model::table::{Cell, Table};
use rhwp::parser::hwpx::{parse_hwpx, section::parse_hwpx_section};
use rhwp::serializer::hwpx::serialize_hwpx;

const ALIGNMENTS: [CaptionVertAlign; 3] = [
    CaptionVertAlign::Top,
    CaptionVertAlign::Center,
    CaptionVertAlign::Bottom,
];

#[derive(Debug, Clone, PartialEq)]
struct ParagraphSentinel {
    text: String,
    char_count: u32,
    char_offsets: Vec<u32>,
    char_shapes: Vec<(u32, u32)>,
    line_segs: Vec<(u32, i32, i32, i32, i32, i32, i32, i32, u32)>,
}

#[derive(Debug, Clone, PartialEq)]
struct CaptionSentinel {
    owner: &'static str,
    direction: CaptionDirection,
    vert_align: CaptionVertAlign,
    width: u32,
    spacing: i16,
    max_width: u32,
    include_margin: bool,
    paragraphs: Vec<ParagraphSentinel>,
}

fn paragraph_sentinel(paragraph: &Paragraph) -> ParagraphSentinel {
    ParagraphSentinel {
        text: paragraph.text.clone(),
        char_count: paragraph.char_count,
        char_offsets: paragraph.char_offsets.clone(),
        char_shapes: paragraph
            .char_shapes
            .iter()
            .map(|shape| (shape.start_pos, shape.char_shape_id))
            .collect(),
        line_segs: paragraph
            .line_segs
            .iter()
            .map(|line| {
                (
                    line.text_start,
                    line.vertical_pos,
                    line.line_height,
                    line.text_height,
                    line.baseline_distance,
                    line.line_spacing,
                    line.column_start,
                    line.segment_width,
                    line.tag,
                )
            })
            .collect(),
    }
}

fn owner_captions(document: &Document) -> Vec<(&'static str, &Caption)> {
    let mut captions = Vec::new();
    for section in &document.sections {
        for paragraph in &section.paragraphs {
            for control in &paragraph.controls {
                match control {
                    Control::Table(table) => {
                        if let Some(caption) = &table.caption {
                            captions.push(("table", caption));
                        }
                    }
                    Control::Shape(shape) => {
                        if let ShapeObject::Rectangle(rectangle) = shape.as_ref() {
                            if let Some(caption) = &rectangle.drawing.caption {
                                captions.push(("shape", caption));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    assert_eq!(
        captions.iter().map(|(owner, _)| *owner).collect::<Vec<_>>(),
        ["table", "shape"],
        "합성 fixture의 두 caption owner가 바뀌었다"
    );
    captions
}

fn caption_sentinels(document: &Document) -> Vec<CaptionSentinel> {
    owner_captions(document)
        .into_iter()
        .map(|(owner, caption)| CaptionSentinel {
            owner,
            direction: caption.direction,
            vert_align: caption.vert_align,
            width: caption.width,
            spacing: caption.spacing,
            max_width: caption.max_width,
            include_margin: caption.include_margin,
            paragraphs: caption.paragraphs.iter().map(paragraph_sentinel).collect(),
        })
        .collect()
}

fn line_seg(seed: i32) -> LineSeg {
    LineSeg {
        text_start: 0,
        vertical_pos: seed + 10,
        line_height: seed + 700,
        text_height: seed + 500,
        baseline_distance: seed + 420,
        line_spacing: seed + 200,
        column_start: seed + 30,
        segment_width: seed + 4_000,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE | LineSeg::TAG_FIRST_LINE_OF_COLUMN,
    }
}

fn caption_paragraph(text: &str, seed: i32) -> Paragraph {
    let mut char_offsets = Vec::new();
    let mut utf16_pos = 0_u32;
    for character in text.chars() {
        char_offsets.push(utf16_pos);
        utf16_pos += character.len_utf16() as u32;
    }
    Paragraph {
        char_count: utf16_pos + 1,
        text: text.to_string(),
        char_offsets,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 1,
        }],
        line_segs: vec![line_seg(seed)],
        has_para_text: true,
        ..Default::default()
    }
}

fn caption(owner: &'static str, vert_align: CaptionVertAlign) -> Caption {
    let (direction, width, spacing, max_width, include_margin, seed) = match owner {
        "table" => (CaptionDirection::Left, 8_123, 234, 45_678, true, 100),
        "shape" => (CaptionDirection::Right, 9_234, 345, 56_789, false, 300),
        _ => unreachable!(),
    };
    Caption {
        direction,
        vert_align,
        width,
        spacing,
        max_width,
        include_margin,
        paragraphs: vec![
            caption_paragraph(&format!("{owner} 첫 문단"), seed),
            caption_paragraph(&format!("{owner} 둘째 & <문단>"), seed + 50),
        ],
    }
}

fn fixture(vert_align: CaptionVertAlign) -> Document {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        row_sizes: vec![1],
        caption: Some(caption("table", vert_align)),
        ..Default::default()
    };
    table.common.instance_id = 101;
    table.common.width = 12_000;
    table.common.height = 3_000;
    table.common.treat_as_char = true;
    table.cells.push(Cell {
        col_span: 1,
        row_span: 1,
        width: 12_000,
        height: 3_000,
        paragraphs: vec![caption_paragraph("셀 본문", 20)],
        ..Default::default()
    });
    table.rebuild_grid();

    let mut rectangle = RectangleShape::default();
    rectangle.common.instance_id = 202;
    rectangle.common.width = 6_400;
    rectangle.common.height = 3_200;
    rectangle.common.treat_as_char = true;
    rectangle.drawing.inst_id = 202;
    rectangle.drawing.shape_attr.original_width = 6_400;
    rectangle.drawing.shape_attr.original_height = 3_200;
    rectangle.drawing.shape_attr.current_width = 6_400;
    rectangle.drawing.shape_attr.current_height = 3_200;
    rectangle.drawing.caption = Some(caption("shape", vert_align));
    rectangle.x_coords = [0, 6_400, 6_400, 0];
    rectangle.y_coords = [0, 0, 3_200, 3_200];

    let host = Paragraph {
        char_count: 17,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        controls: vec![
            Control::Table(Box::new(table)),
            Control::Shape(Box::new(ShapeObject::Rectangle(rectangle))),
        ],
        has_para_text: true,
        ..Default::default()
    };

    let mut document = Document::default();
    document.doc_info.char_shapes = vec![CharShape::default(), CharShape::default()];
    document.doc_info.para_shapes.push(ParaShape::default());
    document.doc_info.styles.push(Style::default());
    document.sections.push(Section {
        paragraphs: vec![host],
        ..Default::default()
    });
    document.doc_properties.section_count = 1;
    document
}

fn caption_alignments(document: &Document) -> Vec<CaptionVertAlign> {
    owner_captions(document)
        .into_iter()
        .map(|(_, caption)| caption.vert_align)
        .collect()
}

fn parse_minimal_table_caption(sub_list: &str) -> Caption {
    let xml = format!(
        r#"<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
  xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0">
    <hp:tbl rowCnt="0" colCnt="0">
      <hp:caption side="LEFT" width="8123" gap="234" lastWidth="45678" fullSz="1">
        {sub_list}
      </hp:caption>
    </hp:tbl>
  </hp:run></hp:p>
</hs:sec>"#
    );
    let section = parse_hwpx_section(&xml).expect("최소 caption section 파싱");
    let table = section
        .paragraphs
        .iter()
        .flat_map(|paragraph| &paragraph.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("최소 section의 표");
    table.caption.clone().expect("최소 section의 표 캡션")
}

#[test]
fn hwpx_reload_preserves_all_alignments_for_table_and_shape_captions() {
    for alignment in ALIGNMENTS {
        let source = fixture(alignment);
        let expected = caption_sentinels(&source);

        let bytes = serialize_hwpx(&source).expect("HWPX 직렬화");
        assert_eq!(
            caption_sentinels(&source),
            expected,
            "직렬화가 source IR을 변경하면 안 된다: {alignment:?}"
        );
        let reloaded = parse_hwpx(&bytes).expect("HWPX 재파싱");
        assert_eq!(
            caption_sentinels(&reloaded),
            expected,
            "표/사각형 캡션과 문단 sentinel 보존: {alignment:?}"
        );
    }
}

#[test]
fn self_closing_caption_sub_list_reads_center_alignment() {
    let caption = parse_minimal_table_caption(r#"<hp:subList vertAlign="CENTER"/>"#);
    assert_eq!(caption.vert_align, CaptionVertAlign::Center);
}

#[test]
fn unknown_caption_alignment_uses_top_fallback_without_touching_geometry() {
    let caption =
        parse_minimal_table_caption(r#"<hp:subList vertAlign="FUTURE_VALUE"></hp:subList>"#);
    assert_eq!(caption.vert_align, CaptionVertAlign::Top);
    assert_eq!(caption.direction, CaptionDirection::Left);
    assert_eq!(caption.width, 8_123);
    assert_eq!(caption.spacing, 234);
    assert_eq!(caption.max_width, 45_678);
    assert!(caption.include_margin);
}

#[test]
fn public_hwp_conversion_dispatch_keeps_center_caption_alignment() {
    let expected = vec![CaptionVertAlign::Center, CaptionVertAlign::Center];

    let hwp_bytes =
        rhwp::serialize_document(&fixture(CaptionVertAlign::Center)).expect("합성 HWP5 직렬화");
    let hwp = rhwp::parse_document(&hwp_bytes).expect("합성 HWP5 파싱");
    let hwpx_bytes = serialize_hwpx(&hwp).expect("HWP5 IR -> HWPX");
    let hwp_to_hwpx = parse_hwpx(&hwpx_bytes).expect("HWPX 재파싱");

    let hwpx_bytes = serialize_hwpx(&fixture(CaptionVertAlign::Center)).expect("합성 HWPX");
    let hwpx = parse_hwpx(&hwpx_bytes).expect("합성 HWPX 파싱");
    let hwp_bytes = rhwp::serialize_document(&hwpx).expect("HWPX IR -> HWP5");
    let hwpx_to_hwp = rhwp::parse_document(&hwp_bytes).expect("HWP5 재파싱");

    assert_eq!(caption_alignments(&hwp_to_hwpx), expected);
    assert_eq!(caption_alignments(&hwpx_to_hwp), expected);
}
