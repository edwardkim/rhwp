//! 글자처럼 취급 표 뒤에 컨트롤만 남은 저장 줄(쪽 번호 감추기 등)도 줄 상자를 쌓는다.
//! 표지의 표 문단이 이 줄을 버려 아래 날짜·로고가 그 줄 높이만큼 올라갔다.
//! 공개 합성 IR이다.
use rhwp::model::control::{Control, PageHide};
use rhwp::model::document::{Document, Section, SectionDef};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, TextWrap};
use rhwp::model::style::ParaShape;
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const TABLE_H: i32 = 4_000;
const NEXT_VPOS: i32 = 7_000;

fn seg(text_start: u32, vpos: i32, height: i32, spacing: i32) -> LineSeg {
    LineSeg {
        text_start,
        vertical_pos: vpos,
        line_height: height,
        text_height: height,
        baseline_distance: height * 4 / 5,
        line_spacing: spacing,
        segment_width: 40_000,
        tag: 393216,
        ..Default::default()
    }
}

/// `tail` 이면 표 줄 뒤에 컨트롤 전용 줄을 두고, 아니면 같은 높이를 표 줄 간격에 싣는다.
/// 두 사다리 모두 다음 문단을 `NEXT_VPOS` 에 둔다.
fn fixture(tail: bool) -> Document {
    let table = Table {
        row_count: 1,
        col_count: 1,
        common: CommonObjAttr {
            width: 20_000,
            height: TABLE_H as u32,
            treat_as_char: true,
            text_wrap: TextWrap::TopAndBottom,
            ..Default::default()
        },
        cells: vec![Cell {
            row_span: 1,
            col_span: 1,
            width: 20_000,
            height: TABLE_H as u32,
            paragraphs: vec![Paragraph::default()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let line_segs = if tail {
        vec![seg(0, 0, TABLE_H, 400), seg(8, TABLE_H + 400, 2_000, 600)]
    } else {
        vec![seg(0, 0, TABLE_H, NEXT_VPOS - TABLE_H)]
    };
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.sections.push(Section {
        paragraphs: vec![
            Paragraph {
                controls: vec![
                    Control::Table(Box::new(table)),
                    Control::PageHide(PageHide {
                        hide_page_num: true,
                        ..Default::default()
                    }),
                ],
                char_count: 17,
                line_segs,
                ..Default::default()
            },
            Paragraph {
                text: "NEXT".into(),
                char_count: 5,
                char_offsets: vec![0, 1, 2, 3],
                line_segs: vec![seg(0, NEXT_VPOS, 1_000, 0)],
                ..Default::default()
            },
        ],
        section_def: SectionDef {
            page_def: PageDef {
                width: 44_000,
                height: 60_000,
                margin_left: 2_000,
                margin_right: 2_000,
                margin_top: 2_000,
                margin_bottom: 2_000,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
    doc
}

fn text_top(node: &RenderNode, needle: &str) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.display_or_text().contains(needle) {
            return Some(node.bbox.y);
        }
    }
    node.children.iter().find_map(|c| text_top(c, needle))
}

fn next_top(tail: bool) -> f64 {
    let mut core = DocumentCore::new_empty();
    core.set_document(fixture(tail));
    assert_eq!(core.page_count(), 1);
    text_top(&core.build_page_render_tree(0).unwrap().root, "N").expect("NEXT 문단")
}

#[test]
fn control_only_tail_line_keeps_its_line_box() {
    let with_tail = next_top(true);
    let folded = next_top(false);
    assert!(
        (with_tail - folded).abs() < 0.5,
        "컨트롤 전용 줄 높이 누락: {with_tail:.1} vs {folded:.1}"
    );
}
