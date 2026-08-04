use super::authoritative_stored_line_start_px;
use crate::model::page::{ColumnDef, PageDef};
use crate::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use crate::renderer::composer::compose_paragraph;
use crate::renderer::layout::LayoutEngine;
use crate::renderer::page_layout::PageLayoutInfo;
use crate::renderer::pagination::{ColumnContent, PageContent, PageItem};
use crate::renderer::render_tree::{RenderNode, RenderNodeType};
use crate::renderer::style_resolver::{ResolvedCharStyle, ResolvedParaStyle, ResolvedStyleSet};

const DPI: f64 = 96.0;
const BODY_LEFT_PX: f64 = 37.76;
const COLUMN_WIDTH_HU: i32 = 78_518;

fn line_seg(column_start: i32, segment_width: i32) -> LineSeg {
    LineSeg {
        line_height: 1_000,
        text_height: 900,
        baseline_distance: 800,
        column_start,
        segment_width,
        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
        ..Default::default()
    }
}

fn collect_render_nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect_render_nodes(child, out);
    }
}

fn synthetic_lineseg_indent_tree() -> crate::renderer::render_tree::PageRenderTree {
    let engine = LayoutEngine::with_default_dpi();
    let page_def = PageDef {
        width: 59_528,
        height: 84_188,
        margin_left: 2_832,
        margin_right: 2_832,
        margin_top: 2_832,
        margin_bottom: 2_832,
        margin_header: 1_417,
        margin_footer: 1_417,
        landscape: true,
        ..Default::default()
    };
    let layout = PageLayoutInfo::from_page_def_default(&page_def, &ColumnDef::default());

    let heading = "1.  작성요령".to_string();
    let literal_number = "1.의료기기".to_string();
    let paragraphs = vec![
        Paragraph {
            char_count: heading.encode_utf16().count() as u32 + 1,
            char_offsets: (0..heading.encode_utf16().count() as u32).collect(),
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            text: heading,
            para_shape_id: 0,
            line_segs: vec![line_seg(1_900, 76_616)],
            ..Default::default()
        },
        Paragraph {
            char_count: literal_number.encode_utf16().count() as u32 + 1,
            char_offsets: (0..literal_number.encode_utf16().count() as u32).collect(),
            char_shapes: vec![
                CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                },
                CharShapeRef {
                    start_pos: 2,
                    char_shape_id: 1,
                },
            ],
            text: literal_number,
            para_shape_id: 1,
            line_segs: vec![line_seg(10_320, 68_196)],
            ..Default::default()
        },
    ];
    let composed: Vec<_> = paragraphs.iter().map(compose_paragraph).collect();
    let styles = ResolvedStyleSet {
        hwp3_variant: false,
        char_styles: vec![ResolvedCharStyle::default(), ResolvedCharStyle::default()],
        para_styles: vec![
            ResolvedParaStyle {
                margin_left: 1_900.0 * DPI / 7_200.0,
                ..Default::default()
            },
            ResolvedParaStyle {
                margin_left: (4_129.0 / 2.0) * DPI / 7_200.0,
                ..Default::default()
            },
        ],
        border_styles: Vec::new(),
        numberings: Vec::new(),
        bullets: Vec::new(),
    };
    let page_content = PageContent {
        page_index: 0,
        page_number: 0,
        section_index: 0,
        layout,
        column_contents: vec![ColumnContent {
            column_index: 0,
            start_height: 0.0,
            endnote_flow: false,
            items: vec![
                PageItem::FullParagraph { para_index: 0 },
                PageItem::FullParagraph { para_index: 1 },
            ],
            zone_layout: None,
            zone_y_offset: 0.0,
            wrap_around_paras: Vec::new(),
            used_height: 0.0,
            wrap_anchors: std::collections::HashMap::new(),
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
    };

    engine.build_render_tree(
        &page_content,
        &paragraphs,
        &paragraphs,
        &paragraphs,
        &composed,
        &styles,
        &Default::default(),
        &[],
        None,
        &[],
        None,
        0,
        &[],
    )
}

#[test]
fn matching_lineseg_start_keeps_heading_at_para_margin() {
    let seg = line_seg(1_900, 76_616);
    let styled_margin_px = 1_900.0 * DPI / 7_200.0;
    let offset = authoritative_stored_line_start_px(
        styled_margin_px,
        Some(&seg),
        COLUMN_WIDTH_HU,
        DPI,
        true,
    );

    assert!((BODY_LEFT_PX + offset - 63.09).abs() < 0.02);
}

#[test]
fn larger_authoritative_lineseg_start_positions_literal_number() {
    let seg = line_seg(10_320, 68_196);
    let styled_margin_px = (4_129.0 / 2.0) * DPI / 7_200.0;
    let offset = authoritative_stored_line_start_px(
        styled_margin_px,
        Some(&seg),
        COLUMN_WIDTH_HU,
        DPI,
        true,
    );

    assert!((BODY_LEFT_PX + offset - 175.36).abs() < 0.02);
}

#[test]
fn synthetic_or_non_body_geometry_does_not_override_para_margin() {
    let styled_margin_px = 27.5;
    let mut synthetic = line_seg(10_320, 68_196);
    synthetic.tag |= LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    assert_eq!(
        authoritative_stored_line_start_px(
            styled_margin_px,
            Some(&synthetic),
            COLUMN_WIDTH_HU,
            DPI,
            true,
        ),
        styled_margin_px,
    );

    let wrap_or_cell_seg = line_seg(39_123, 3_397);
    assert_eq!(
        authoritative_stored_line_start_px(
            styled_margin_px,
            Some(&wrap_or_cell_seg),
            42_520,
            DPI,
            false,
        ),
        styled_margin_px,
    );
}

#[test]
fn body_textline_and_textruns_follow_authoritative_lineseg_start() {
    let tree = synthetic_lineseg_indent_tree();
    let mut nodes = Vec::new();
    collect_render_nodes(&tree.root, &mut nodes);

    let heading_line = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextLine(line) if line.para_index == Some(0)
            )
        })
        .expect("heading TextLine");
    let literal_line = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextLine(line) if line.para_index == Some(1)
            )
        })
        .expect("literal-number TextLine");
    let number_run = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(1) && run.text == "1."
            )
        })
        .expect("literal number TextRun");
    let body_run = nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextRun(run)
                    if run.para_index == Some(1) && run.text == "의료기기"
            )
        })
        .expect("literal-number body TextRun");

    assert!(
        (heading_line.bbox.x - 63.09).abs() < 0.05,
        "heading line bbox: {:?}",
        heading_line.bbox,
    );
    assert!(
        (literal_line.bbox.x - 175.36).abs() < 0.05,
        "literal line bbox: {:?}",
        literal_line.bbox,
    );
    assert!(
        (number_run.bbox.x - 175.36).abs() < 0.05,
        "number run bbox: {:?}",
        number_run.bbox,
    );
    assert!(
        (body_run.bbox.x - (number_run.bbox.x + number_run.bbox.width)).abs() < 0.05,
        "body run must follow the literal number: number={:?}, body={:?}",
        number_run.bbox,
        body_run.bbox,
    );
    assert!(
        (literal_line.bbox.x + literal_line.bbox.width
            - (BODY_LEFT_PX + COLUMN_WIDTH_HU as f64 * DPI / 7_200.0))
            .abs()
            < 0.2,
        "TextLine right edge must remain at the column right edge: {:?}",
        literal_line.bbox,
    );
}
