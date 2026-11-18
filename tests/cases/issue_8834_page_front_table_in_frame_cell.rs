//! [#8834] 합성 TAC 글줄과 쪽 기준 글 앞 표의 실제 소유 프레임.
//! 주석 갱신: 2026-11-10 21:51 KST
//!
//! 검사 유형은 함수별로 구분한다.
//! - `page_anchored_front_table_in_a_bottom_frame_cell_does_not_stack`: 고정형(합성 문서 1쪽).
//! - `original_hwpx_keeps_independent_print_table_origins`: 고정형(실물 1쪽·독립 PDF 원점 ±1px).
//! - `single_tac_host_shares_the_first_line_indent_with_its_prefix`: 관계형(원본 속성에서 환산한
//!   들여쓰기만큼 공백과 표가 함께 이동). 절대 px 위치는 고정하지 않는다.
//!
//! 공개 API 로 만든 합성 문서(본문 문단 저장 줄 없음)의 계약 검사다. 기대값은 선언 높이에서 온다.
//! - 고정형(쪽수): 쪽 기준·아래 정렬 고정 틀(선언 높이 = 본문의 48%)의 칸에 글자처럼 취급 표와 쪽
//!   기준 글 앞으로 표가 함께 있으면, 글 앞 표는 칸 높이에 쌓이지 않아 문서가 1쪽이다(쌓으면 틀이
//!   본문의 96%가 되어 앞 글줄과 함께 들어가지 않는다). 칸 문단은 실물처럼 틀 높이 한 줄을 저장해 둔
//!   모양이며, 칸 문단에 저장 줄이 없는 경로는 이 검사의 범위 밖이다.
//!
//! 실물 검사는 HWPX 로드 시 합성된 TAC 줄의 바깥 여백 이중 가산도 포함한다. 별도 인라인 painter가
//! 합성 단일 줄의 들여쓰기·기준선을 다시 추정하던 잔여는 확정된 공통 글줄 배치를 소비해 보정했다.
//!
//! 대응 실물: 공개 결재문서 36387608(rhwp 2쪽 → 1쪽, 한컴 engine 2020 Print PDF 1쪽).
//! 원 PR의 Native 84.80·WASM 85.14%는 과거 환경의 결과다. 메인터너 보정 전 Mac 원본은
//! 양쪽 82.08%였으며 글꼴 누락으로 단정하지 않고 표 원점을 추적했다. 보정 후(2026-11-10,
//! parent 4e74b39e5b + validation.json의 source hash) Native/fresh WASM 1쪽 모두 100.00%다.
//! 독립 한컴 2020 Print PDF의 같은 쪽 review·overlay를 확대 판독했으므로 원점은 ±1px로 고정한다.
//! 원본·PDF SHA와 실행 출처: `mydocs/pr/assets/planet6897_clean19_20261110/pr8839/maintainer/validation.json`.
//! 개인 글꼴 파일을 읽지 않는 Mac 최소 글꼴 환경에서도 같은 프레임 계약을 검증한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, HorzRelTo, TextWrap, VertAlign, VertRelTo};
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::RenderNodeType;

const PX_PER_HWPUNIT: f64 = 96.0 / 7200.0;

fn original_hwpx() -> DocumentCore {
    DocumentCore::from_bytes(include_bytes!(
        "../fixtures/issue8834/36387608-approval-original.hwpx"
    ))
    .expect("독립 Print와 동일한 원본 HWPX")
}

fn original_table_box(
    node: &rhwp::renderer::render_tree::RenderNode,
    paragraph: usize,
) -> Option<rhwp::renderer::render_tree::BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(paragraph) && table.cell_context.is_none() {
            return Some(node.bbox);
        }
    }
    node.children
        .iter()
        .find_map(|child| original_table_box(child, paragraph))
}

fn original_prefix_x(
    node: &rhwp::renderer::render_tree::RenderNode,
    paragraph: usize,
) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(paragraph) && run.cell_context.is_none() && run.text == "    " {
            return Some(node.bbox.x);
        }
    }
    node.children
        .iter()
        .find_map(|child| original_prefix_x(child, paragraph))
}

#[test]
fn original_hwpx_keeps_independent_print_table_origins() {
    let core = original_hwpx();
    assert_eq!(core.page_count(), 1, "독립 한컴 Print와 같은 1쪽");
    let tree = core.build_page_render_tree(0).expect("실물 첫 쪽 배치");
    // PDF path CTM을 96/72로 환산한 각 표의 좌상단이다. bbox를 찾을 때에는
    // 고정 영역이 아니라 원본 문단 주소를 사용하므로 잘못 놓인 표도 검출한다.
    for (paragraph, expected_x, expected_y) in [(4, 114.9787, 440.1587), (6, 113.6987, 605.2573)] {
        let bbox = original_table_box(&tree.root, paragraph).expect("원본 TAC 표");
        assert!(
            (bbox.x - expected_x).abs() <= 1.0 && (bbox.y - expected_y).abs() <= 1.0,
            "문단 {paragraph} 표는 독립 PDF 원점 ±1px 안이어야 한다: {bbox:?}, PDF=({expected_x},{expected_y})"
        );
        assert!(
            original_prefix_x(&tree.root, paragraph).expect("표 앞 원본 공백") < bbox.x,
            "공백은 같은 줄의 표 앞에 보존되어야 한다"
        );
    }
}

#[test]
fn single_tac_host_shares_the_first_line_indent_with_its_prefix() {
    let positions = |indent: i32| {
        let mut core = original_hwpx();
        let mut document = core.document().clone();
        for paragraph in [4, 6] {
            let host = &mut document.sections[0].paragraphs[paragraph];
            document.doc_info.para_shapes[host.para_shape_id as usize].indent = indent;
            // 확정된 저장 단일 줄은 별도 인라인 경로를 사용한다. 같은 첫 줄
            // 들여쓰기 플래그를 지정해 현재 재조판 경로와 함께 보호한다.
            for line in &mut host.line_segs {
                line.tag =
                    (line.tag & !LineSeg::TAG_IMPLEMENTATION_PROPERTY) | LineSeg::TAG_INDENTATION;
            }
        }
        core.set_document(document);
        let tree = core.build_page_render_tree(0).expect("들여쓰기 대조 배치");
        [4, 6].map(|paragraph| {
            (
                original_prefix_x(&tree.root, paragraph).expect("같은 줄 공백"),
                original_table_box(&tree.root, paragraph)
                    .expect("같은 줄 표")
                    .x,
            )
        })
    };
    let original = positions(0);
    for indent in [660, -660] {
        let after = positions(indent);
        // HWP 문단 들여쓰기는 1/2 HWPUNIT다. 음수 내어쓰기는 첫 줄을 옮기지 않는다.
        let expected = f64::from(indent.max(0)) / 2.0 * PX_PER_HWPUNIT;
        for ((prefix_before, table_before), (prefix_after, table_after)) in
            original.into_iter().zip(after)
        {
            assert!(
                ((prefix_after - prefix_before) - expected).abs() < 1e-6,
                "같은 첫 줄 공백이 선언 들여쓰기를 소비해야 한다"
            );
            assert!(
                ((table_after - table_before) - expected).abs() < 1e-6,
                "표도 앞 공백과 같은 첫 줄의 들여쓰기를 소비해야 한다"
            );
        }
    }
}

fn empty_paragraph() -> Paragraph {
    Paragraph {
        char_count: 1,
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        ..Default::default()
    }
}

fn one_cell_table(common: CommonObjAttr, cell_paragraph: Paragraph) -> Table {
    let mut table = Table {
        row_count: 1,
        col_count: 1,
        cells: vec![Cell {
            row: 0,
            col: 0,
            row_span: 1,
            col_span: 1,
            width: common.width,
            height: common.height,
            paragraphs: vec![cell_paragraph],
            ..Default::default()
        }],
        common,
        ..Default::default()
    };
    table.rebuild_grid();
    table
}

fn tac_table(width: u32, height: u32, margin_top: i16, margin_bottom: i16) -> Table {
    let mut common = CommonObjAttr {
        treat_as_char: true,
        text_wrap: TextWrap::TopAndBottom,
        vert_rel_to: VertRelTo::Para,
        horz_rel_to: HorzRelTo::Para,
        width,
        height,
        ..Default::default()
    };
    common.margin.top = margin_top;
    common.margin.bottom = margin_bottom;
    let mut table = one_cell_table(common, empty_paragraph());
    // 파서는 글자처럼 취급 개체의 컨트롤 속성 비트 0 을 세운다.
    table.attr |= 0x01;
    table.outer_margin_top = margin_top;
    table.outer_margin_bottom = margin_bottom;
    table
}

fn text_line(text: &str) -> Paragraph {
    let count = u32::try_from(text.chars().count()).expect("글자 수");
    Paragraph {
        text: text.to_string(),
        char_count: count + 1,
        char_offsets: (0..count).collect(),
        ..empty_paragraph()
    }
}

struct Blank {
    core: DocumentCore,
    column_width: u32,
}

fn blank() -> Blank {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서 생성");
    let page = &core.document().sections[0].section_def.page_def;
    let column_width = page.width - page.margin_left - page.margin_right;
    Blank { core, column_width }
}

fn with_paragraphs(mut blank: Blank, paragraphs: Vec<Paragraph>) -> DocumentCore {
    let mut document = blank.core.document().clone();
    document.sections[0].paragraphs = paragraphs;
    blank.core.set_document(document);
    blank.core
}

#[test]
fn page_anchored_front_table_in_a_bottom_frame_cell_does_not_stack() {
    let base = blank();
    let column_width = base.column_width;
    let probe = with_paragraphs(blank(), vec![text_line("측정")]);
    let root = probe.build_page_render_tree(0).expect("쪽 렌더").root;
    let body_height_px = root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .expect("본문 노드")
        .bbox
        .height;
    let body_height = (body_height_px / PX_PER_HWPUNIT) as u32;
    let frame_height = body_height * 48 / 100;

    // 칸 안: 틀 높이를 채우는 글자처럼 취급 표 + 쪽 기준 글 앞으로 표(같은 높이).
    let front = {
        let common = CommonObjAttr {
            treat_as_char: false,
            text_wrap: TextWrap::InFrontOfText,
            vert_rel_to: VertRelTo::Page,
            horz_rel_to: HorzRelTo::Page,
            vert_align: VertAlign::Bottom,
            width: column_width / 3,
            height: frame_height,
            ..Default::default()
        };
        one_cell_table(common, empty_paragraph())
    };
    // 실물(36387608 틀 칸)처럼 칸 문단은 틀 높이 한 줄을 저장해 둔다.
    let cell_paragraph = Paragraph {
        line_segs: vec![LineSeg {
            line_height: frame_height as i32,
            text_height: frame_height as i32,
            baseline_distance: frame_height as i32 * 85 / 100,
            line_spacing: 600,
            segment_width: column_width as i32,
            tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
            ..Default::default()
        }],
        char_count: 8 + 8 + 1,
        controls: vec![
            Control::Table(Box::new(tac_table(column_width - 400, frame_height, 0, 0))),
            Control::Table(Box::new(front)),
        ],
        ..empty_paragraph()
    };
    let frame = one_cell_table(
        CommonObjAttr {
            treat_as_char: false,
            text_wrap: TextWrap::TopAndBottom,
            vert_rel_to: VertRelTo::Page,
            horz_rel_to: HorzRelTo::Column,
            vert_align: VertAlign::Bottom,
            width: column_width,
            height: frame_height,
            ..Default::default()
        },
        cell_paragraph,
    );
    let mut host = text_line("붙임 끝.");
    host.char_count += 8;
    host.controls = vec![Control::Table(Box::new(frame))];

    let mut paragraphs: Vec<Paragraph> = (0..3).map(|i| text_line(&format!("본문 {i}"))).collect();
    paragraphs.push(host);
    let core = with_paragraphs(base, paragraphs);
    assert_eq!(
        core.page_count(),
        1,
        "글줄 넷 + 틀(본문의 48%)은 한 쪽에 들어간다 — 글 앞 표를 칸 높이에 쌓으면 2쪽"
    );
}
