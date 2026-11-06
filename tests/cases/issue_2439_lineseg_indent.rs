//! 관계형·고정형: 합성 저장 LineSeg의 본문 원점과 run 소속을 검사한다.
//! 주석 갱신: 2026-11-06T21:01:51+09:00.
//! 이동 직전 원본 변경: 2026-11-06T20:11:29+09:00 6844f6ec0854b1c8150244d65cc1a099031287ac.
//! 기존 내부 검사 5건을 공개 Document·LayoutEngine 경로로 이동했다.
//! 입력은 이 파일에서 구성한 합성 Paragraph이며 실물 HWP/HWPX·Print PDF는 없다.
//! 원본 단위에서 정한 기대 좌표와 run 연결 관계를 보존한다. 설치 글꼴에 의존하지 않는다.
//! Native/fresh WASM raw·전쪽 최저값·TSV·PNG는 이 합성 단위 계약의 실측 근거가 아니며
//! 해당 수치를 추정하지 않는다. 실제 문서의 조판 일치율을 검증하는 검사로 취급하지 않는다.

use rhwp::model::document::{Document, HWP5_ORIGIN_HWPX_MARKER_PATH};
use rhwp::model::page::{ColumnDef, PageDef};
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::provenance::SourceFormat;
use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::layout::LayoutEngine;
use rhwp::renderer::page_layout::PageLayoutInfo;
use rhwp::renderer::pagination::{ColumnContent, PageContent, PageItem};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::style_resolver::{ResolvedCharStyle, ResolvedParaStyle, ResolvedStyleSet};

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

fn synthetic_lineseg_indent_tree() -> rhwp::renderer::render_tree::PageRenderTree {
    lineseg_indent_tree(
        &Document::default(),
        [line_seg(1_900, 76_616), line_seg(10_320, 68_196)],
        false,
    )
}

fn lineseg_indent_tree(
    document: &Document,
    segments: [LineSeg; 2],
    non_body_frame: bool,
) -> rhwp::renderer::render_tree::PageRenderTree {
    let engine = LayoutEngine::with_default_dpi();
    engine.set_layout_profile(document.layout_profile());
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
    let mut layout = PageLayoutInfo::from_page_def_default(&page_def, &ColumnDef::default());
    if non_body_frame {
        // 본문 전체 높이를 갖지 않는 별도 프레임은 저장 본문 원점을 빌리지 않는다.
        layout.column_areas[0].height -= 24.0;
        layout.column_areas[0].width = 42_520.0 * DPI / 7_200.0;
    }
    let [heading_segment, literal_segment] = segments;

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
            line_segs: vec![heading_segment],
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
            line_segs: vec![literal_segment],
            ..Default::default()
        },
    ];
    let composed: Vec<_> = paragraphs.iter().map(compose_paragraph).collect();
    let mut styles = ResolvedStyleSet::default();
    styles.char_styles = vec![ResolvedCharStyle::default(), ResolvedCharStyle::default()];
    styles.para_styles = vec![
        ResolvedParaStyle {
            margin_left: 1_900.0 * DPI / 7_200.0,
            ..Default::default()
        },
        ResolvedParaStyle {
            margin_left: (4_129.0 / 2.0) * DPI / 7_200.0,
            ..Default::default()
        },
    ];
    let page_content = PageContent {
        page_index: 0,
        page_number: 0,
        page_number_restarted: false,
        section_index: 0,
        layout,
        column_contents: vec![ColumnContent {
            paragraph_origins: Default::default(),
            computed_paragraphs: Default::default(),
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
            overlay_continuations: Vec::new(),
            overlay_cuts: Vec::new(),
            inline_placements: Default::default(),
            inline_flow_plans: Default::default(),
            stored_paragraph_flow_ends: Default::default(),
            host_compositions: Default::default(),
            stored_column_top_spacings: Default::default(),
            paragraph_float_placements: Default::default(),
            float_pushed_line_spacings: Default::default(),
            page_band_flow_starts: Default::default(),
        }],
        active_header: None,
        active_footer: None,
        page_number_pos: None,
        page_hide: None,
        footnotes: Vec::new(),
        active_master_page: None,
        extra_master_pages: Vec::new(),
        ladder_band_tables: Vec::new(),
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

fn line_x(tree: &rhwp::renderer::render_tree::PageRenderTree, paragraph: usize) -> f64 {
    let mut nodes = Vec::new();
    collect_render_nodes(&tree.root, &mut nodes);
    nodes
        .iter()
        .find(|node| {
            matches!(
                &node.node_type,
                RenderNodeType::TextLine(line) if line.para_index == Some(paragraph)
            )
        })
        .expect("합성 문단의 최종 TextLine")
        .bbox
        .x
}

#[test]
fn matching_lineseg_start_keeps_heading_at_para_margin() {
    let tree = synthetic_lineseg_indent_tree();
    assert!((line_x(&tree, 0) - 63.09).abs() < 0.02);
}

#[test]
fn larger_authoritative_lineseg_start_positions_literal_number() {
    let tree = synthetic_lineseg_indent_tree();
    assert!((line_x(&tree, 1) - 175.36).abs() < 0.02);
}

#[test]
fn synthetic_or_non_body_geometry_does_not_override_para_margin() {
    let mut synthetic = line_seg(10_320, 68_196);
    synthetic.tag |= LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    let synthetic_tree = lineseg_indent_tree(
        &Document::default(),
        [line_seg(1_900, 76_616), synthetic],
        false,
    );
    let styled_origin = BODY_LEFT_PX + (4_129.0 / 2.0) * DPI / 7_200.0;
    assert!((line_x(&synthetic_tree, 1) - styled_origin).abs() < 0.02);

    let non_body_tree = lineseg_indent_tree(
        &Document::default(),
        [line_seg(1_900, 76_616), line_seg(39_123, 3_397)],
        true,
    );
    assert!((line_x(&non_body_tree, 1) - styled_origin).abs() < 0.02);
}

#[test]
fn hwp5_origin_hwpx_keeps_the_hwp5_stored_line_start_contract() {
    let native_hwp5 = Document::default();
    let mut hwp5_origin_hwpx = Document::default();
    hwp5_origin_hwpx.provenance.format = SourceFormat::Hwpx;
    hwp5_origin_hwpx
        .hwpx_aux_entries
        .push((HWP5_ORIGIN_HWPX_MARKER_PATH.into(), b"1".to_vec()));
    let mut original_hwpx = Document::default();
    original_hwpx.provenance.format = SourceFormat::Hwpx;

    for document in [&native_hwp5, &hwp5_origin_hwpx] {
        let tree = lineseg_indent_tree(
            document,
            [line_seg(1_900, 76_616), line_seg(10_320, 68_196)],
            false,
        );
        assert!((line_x(&tree, 1) - 175.36).abs() < 0.02);
    }
    let original_tree = lineseg_indent_tree(
        &original_hwpx,
        [line_seg(1_900, 76_616), line_seg(10_320, 68_196)],
        false,
    );
    let styled_origin = BODY_LEFT_PX + (4_129.0 / 2.0) * DPI / 7_200.0;
    assert!(
        (line_x(&original_tree, 1) - styled_origin).abs() < 0.02,
        "원본 HWPX의 별도 저장 LineSeg 계약까지 HWP5로 넓히면 안 된다"
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
