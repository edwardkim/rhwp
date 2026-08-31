//! [Issue #6114] 쪽 분할 셀 안 글자처럼 취급(TAC) 그림이 페인트 높이(312px)가
//! 아니라 글줄 1줄(26px)만 흐름을 전진시켜 아래 표·본문이 그림 위에 겹친다.
//!
//! 저장 LINE_SEG 가 글줄 높이(1950HU≈26px)만 담고, 그림 본체는 23430HU≈312px
//! 인 칸 — 한글은 그림 높이만큼 칸을 밀고, 칸이 제자리에서 쪽을 나눈다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataContent;
use rhwp::model::control::Control;
use rhwp::model::image::Picture;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, TextWrap, VertRelTo};
use rhwp::model::table::{Cell, Table, TablePageBreak};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 96dpi 에서 26px 글줄. 7200 HU/inch → px = HU / 75.
const TEXT_LINE_HU: i32 = 1950;
/// 이슈 원본 ARMA 차트 129.0×82.7mm.
const CHART_W_HU: u32 = 36_572;
const CHART_H_HU: u32 = 23_430;
const CHART_H_PX: f64 = 312.4;
/// 이슈 원본 별지 신고서 129.0×191.9mm.
const FORM_H_HU: u32 = 54_402;
const FORM_H_PX: f64 = 725.36;

const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xFF, 0xFF, 0x3F,
    0x00, 0x05, 0xFE, 0x02, 0xFE, 0xDC, 0xCC, 0x59, 0xE7, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
    0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn line_seg() -> LineSeg {
    LineSeg {
        line_height: TEXT_LINE_HU,
        baseline_distance: 1_658,
        text_height: TEXT_LINE_HU,
        ..Default::default()
    }
}

fn tac_picture(bin_id: u16, width: u32, height: u32) -> Picture {
    let mut pic = Picture::default();
    pic.common = CommonObjAttr {
        treat_as_char: true,
        text_wrap: TextWrap::TopAndBottom,
        vert_rel_to: VertRelTo::Para,
        width,
        height,
        ..Default::default()
    };
    pic.image_attr.bin_data_id = bin_id;
    pic
}

fn picture_para(pic: Picture) -> Paragraph {
    Paragraph {
        char_count: 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![line_seg()],
        controls: vec![Control::Picture(Box::new(pic))],
        ..Default::default()
    }
}

fn text_para(text: &str) -> Paragraph {
    let n = text.chars().count() as u32;
    Paragraph {
        text: text.to_string(),
        char_count: n + 1,
        char_offsets: (0..n).collect(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![line_seg()],
        ..Default::default()
    }
}

fn build_core() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("blank document");

    let mut table = Table {
        row_count: 1,
        col_count: 1,
        row_sizes: vec![1],
        page_break: TablePageBreak::RowBreak,
        cells: vec![Cell {
            col: 0,
            row: 0,
            col_span: 1,
            row_span: 1,
            width: 40_000,
            height: 90_000,
            paragraphs: vec![
                picture_para(tac_picture(1, CHART_W_HU, CHART_H_HU)),
                text_para("전망치표"),
                picture_para(tac_picture(2, CHART_W_HU, FORM_H_HU)),
            ],
            ..Default::default()
        }],
        common: CommonObjAttr {
            treat_as_char: false,
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Para,
            width: 40_000,
            height: 90_000,
            ..Default::default()
        },
        ..Default::default()
    };
    table.rebuild_grid();

    let mut doc = core.document().clone();
    let para_shape_id = doc.sections[0].paragraphs[0].para_shape_id;
    doc.sections[0].paragraphs = vec![Paragraph {
        para_shape_id,
        char_count: 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            line_height: 400,
            baseline_distance: 320,
            ..Default::default()
        }],
        controls: vec![Control::Table(Box::new(table))],
        ..Default::default()
    }];
    doc.bin_data_content = vec![
        BinDataContent {
            id: 1,
            data: PNG_1X1.to_vec().into(),
            extension: "png".to_string(),
        },
        BinDataContent {
            id: 2,
            data: PNG_1X1.to_vec().into(),
            extension: "png".to_string(),
        },
    ];
    core.set_document(doc);
    core
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn issue_6114_tac_image_in_split_cell_advances_painted_height() {
    let core = build_core();
    let page0 = core.build_page_render_tree(0).expect("page 1");
    let mut nodes = Vec::new();
    walk(&page0.root, &mut nodes);

    let mut images: Vec<(f64, f64, f64)> = nodes
        .iter()
        .filter_map(|n| match n.node_type {
            RenderNodeType::Image(_) => Some((n.bbox.y, n.bbox.height, n.bbox.y + n.bbox.height)),
            _ => None,
        })
        .collect();
    images.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    assert!(
        !images.is_empty(),
        "차트 TAC 그림이 1쪽에 그려져야 한다: {images:?}"
    );

    let (chart_y, chart_h, chart_bottom) = images
        .iter()
        .copied()
        .find(|(_, h, _)| (*h - CHART_H_PX).abs() < 2.0)
        .expect("312px 차트 그림");
    assert!(
        (chart_h - CHART_H_PX).abs() < 2.0,
        "차트 높이 {chart_h:.1} (기대 {CHART_H_PX:.1})"
    );

    let text_y = nodes
        .iter()
        .find_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) if run.text.contains("전망") => Some(n.bbox.y),
            _ => None,
        })
        .expect("'전망치표' 본문");

    assert!(
        text_y >= chart_bottom - 1.0,
        "본문이 차트 아래로 흘러야 한다 (결함 시 차트 상단에서 26px): \
         차트 y={chart_y:.1} h={chart_h:.1} bottom={chart_bottom:.1}, 본문 y={text_y:.1}"
    );
    assert!(
        text_y - chart_y > 200.0,
        "흐름이 글줄(26px)이 아니라 그림 높이만큼 전진해야 한다: Δ={:.1}",
        text_y - chart_y
    );

    let mut form = None;
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("page {} tree: {e}", page + 1));
        let mut page_nodes = Vec::new();
        walk(&tree.root, &mut page_nodes);
        for n in page_nodes {
            if matches!(n.node_type, RenderNodeType::Image(_))
                && (n.bbox.height - FORM_H_PX).abs() < 2.0
            {
                form = Some((page, n.bbox.y, n.bbox.height));
            }
        }
    }
    let (form_page, form_y, form_h) = form.expect("725px 별지 그림");
    assert!(
        (form_h - FORM_H_PX).abs() < 2.0,
        "별지 그림 높이 {form_h:.1}"
    );
    if form_page == 0 {
        assert!(
            form_y >= chart_bottom - 1.0,
            "별지 그림이 차트와 겹치면 안 된다: form y={form_y:.1} chart bottom={chart_bottom:.1}"
        );
    }
}
