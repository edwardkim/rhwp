//! 고정형 — [#8834] 쪽 아래 고정 틀 칸 안의 쪽 기준 글 앞으로 표는 칸 높이에 쌓이지 않는다.
//! 주석 갱신: 2026-11-09 14:46 KST (새 파일)
//!
//! 공개 API 로 만든 합성 문서(본문 문단 저장 줄 없음)의 계약 검사다. 기대값은 선언 높이에서 온다.
//! - 고정형(쪽수): 쪽 기준·아래 정렬 고정 틀(선언 높이 = 본문의 48%)의 칸에 글자처럼 취급 표와 쪽
//!   기준 글 앞으로 표가 함께 있으면, 글 앞 표는 칸 높이에 쌓이지 않아 문서가 1쪽이다(쌓으면 틀이
//!   본문의 96%가 되어 앞 글줄과 함께 들어가지 않는다). 칸 문단은 실물처럼 틀 높이 한 줄을 저장해 둔
//!   모양이며, 칸 문단에 저장 줄이 없는 경로는 이 검사의 범위 밖이다.
//!
//! 같은 이슈의 다른 원인(로드 시 합성된 TAC 표 host 줄의 바깥 여백 이중 가산)은 HWPX 로드 재조판
//! 경로에서만 생긴다. 공개 API 로 만든 문서는 그 경로를 타지 않아 여기서 검사하지 않고, 실물 원본과
//! 10k 코퍼스 A/B 로 확인했다(PR 본문).
//!
//! 대응 실물: 공개 결재문서 36387608(rhwp 2쪽 → 1쪽, 한컴 engine 2020 Print PDF 1쪽).
//! 측정(2026-11-09, head MEASURE_HEAD, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`): Native 1쪽 84.80 · fresh WASM 1쪽 85.14(base 2쪽, p1 63.98).
//! 90% 미만은 이 환경에 없는 함초롬돋움·함초롬바탕(본문 대부분) 글꼴 차이라, 실물 대신 이 합성
//! 계약으로 회귀를 건다. 증거: `mydocs/pr/assets/issue_8834/`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::shape::{CommonObjAttr, HorzRelTo, TextWrap, VertAlign, VertRelTo};
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::RenderNodeType;

const PX_PER_HWPUNIT: f64 = 96.0 / 7200.0;

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
