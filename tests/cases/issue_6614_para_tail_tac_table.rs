//! [Issue #6614] 문단 **끝**에 오는 비인라인 `treat_as_char` 표를 문단 **흐름 시작**에
//! 놓아 본문 첫 줄과 겹치던 결함의 가드.
//!
//! 표 폭이 줄 폭과 같아 `is_tac_table_inline_in_para` 가 인라인을 거부하면
//! (`선언폭 < 줄폭 × 0.9` 실패), 그 표는 `table_y_start` 사슬의 마지막
//! `else { y_offset }` 로 떨어진다. `y_offset` 은 **문단 흐름 시작**이라 문단 첫
//! 글자 줄보다 위다.
//!
//! 실측 — `156658611` 1쪽 담당부서 표 (px @96dpi, 오라클 한/글 2020):
//!
//! ```text
//! 저장 사다리  seg0..3 vpos 53839/56359/58879/61399  lh 1400/1400/1400/1500  ← 글자 줄
//!              seg4    vpos 69067                    lh 3696                 ← 표 줄
//! 표           height 3130 + om_top 283 + om_bottom 283 = 3696  ← seg4 와 정확히 일치
//!
//! 표 상단     rhwp 770.8  vs  한/글 약 1005      (−234px)
//! 겹침        overlap 1 · text-overlap 7 → ANOMALY
//! ```
//!
//! 수정 뒤 표 상단 1000.2, 첫 행 글자 1014.4(한/글 1014.3), `status: CLEAN`.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::model::style::{BorderFill, BorderLine, BorderLineType, CharShape, ParaShape};
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::serializer::hwpx::serialize_hwpx;

const TEXT_LH: i32 = 1400;
const TBL_H: u32 = 3130;
const OM: i16 = 283;
/// 표 줄의 저장 높이 = `om_top + 선언높이 + om_bottom`.
const BAND: i32 = OM as i32 + TBL_H as i32 + OM as i32;

fn seg(text_start: u32, vertical_pos: i32, line_height: i32) -> LineSeg {
    LineSeg {
        text_start,
        vertical_pos,
        line_height,
        text_height: line_height,
        baseline_distance: line_height * 4 / 5,
        line_spacing: line_height * 4 / 5,
        segment_width: 42520,
        tag: 0x0016_0000,
        ..Default::default()
    }
}

fn cell(col: u16, text: &str) -> Cell {
    Cell {
        col,
        row: 0,
        col_span: 1,
        row_span: 1,
        width: 8000,
        height: TBL_H,
        border_fill_id: 1,
        paragraphs: vec![Paragraph {
            text: text.to_string(),
            char_count: text.chars().count() as u32,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// 문단 = 글자 4줄 + **마지막 run 이 전폭 TAC 표**. 저장 사다리는 다섯 줄이고
/// 마지막 줄이 표 줄이다.
fn tail_table_document() -> Document {
    let mut table = Table {
        row_count: 1,
        col_count: 6,
        border_fill_id: 1,
        cells: (0..6).map(|c| cell(c, "담당")).collect(),
        outer_margin_top: OM,
        outer_margin_bottom: OM,
        ..Default::default()
    };
    // 줄 폭(42520)보다 넓다 → `선언폭 < 줄폭 × 0.9` 실패 → **인라인이 아니다**.
    table.common.width = 48546;
    table.common.height = TBL_H;
    table.common.treat_as_char = true;
    table.rebuild_grid();

    let text = "□ 김용현 장관과 보렐 고위대표는 유럽·대서양 안보와 인도·태평양 안보가 \
                서로 연결되어 있다는 데 인식을 같이하고 다양한 분야에서 협력을 확대해 \
                나가기로 하였습니다. <끝>";
    let mut owner = Paragraph {
        text: text.to_string(),
        char_count: text.chars().count() as u32,
        ..Default::default()
    };
    // `text_start` 는 실제 글자 축 안이어야 한다 — 벗어나면 직렬화가 사다리를 통째로
    // 버려(`line_segs_within_text_axis`) 이 시험이 재래핑 경로를 재게 된다.
    let len = owner.char_count;
    owner.line_segs = vec![
        seg(0, 0, TEXT_LH),
        seg(len / 5, 2520, TEXT_LH),
        seg(len * 2 / 5, 5040, TEXT_LH),
        seg(len * 3 / 5, 7560, 1500),
        // 표 줄 — 밴드가 `om + 높이 + om` 과 정확히 같다.
        seg(len * 4 / 5, 15228, BAND),
    ];
    owner.controls.push(Control::Table(Box::new(table)));

    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59528,
        height: 84188,
        ..Default::default()
    };
    section.paragraphs.push(owner);

    let solid = BorderLine {
        line_type: BorderLineType::Solid,
        width: 1,
        color: 0,
    };
    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.doc_info.char_shapes = vec![CharShape::default()];
    doc.doc_info.border_fills = vec![BorderFill {
        borders: [solid; 4],
        ..Default::default()
    }];
    doc.doc_properties.section_count = 1;
    doc.sections.push(section);
    doc
}

fn collect(node: &RenderNode, out: &mut Vec<(&'static str, f64, f64)>, inside_table: bool) {
    match &node.node_type {
        RenderNodeType::Table(_) => {
            out.push(("Table", node.bbox.y, node.bbox.height));
            // 표 안의 칸 글자는 배치 축이 아니다.
            for child in &node.children {
                collect(child, out, true);
            }
            return;
        }
        RenderNodeType::TextLine(_) if !inside_table => {
            out.push(("TextLine", node.bbox.y, node.bbox.height));
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, out, inside_table);
    }
}

/// 문단 끝의 TAC 표는 문단 **마지막 글자 줄보다 아래**에 있어야 한다.
///
/// 종전에는 문단 흐름 시작(첫 글자 줄보다 위)에 놓여 본문과 겹쳤다.
#[test]
fn para_tail_tac_table_sits_below_its_own_text() {
    let bytes = serialize_hwpx(&tail_table_document()).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("reload");
    let page = core.build_page_render_tree(0).expect("render tree");

    let mut nodes = Vec::new();
    collect(&page.root, &mut nodes, false);

    let table_y = nodes
        .iter()
        .find(|(kind, _, _)| *kind == "Table")
        .map(|(_, y, _)| *y)
        .expect("표 노드가 있어야 한다");
    let text_ys: Vec<f64> = nodes
        .iter()
        .filter(|(kind, _, _)| *kind == "TextLine")
        .map(|(_, y, _)| *y)
        .collect();
    assert!(
        !text_ys.is_empty(),
        "본문 글자 줄이 있어야 한다 — 시험 설정 오류. nodes={nodes:?}"
    );
    let first_text = text_ys.iter().cloned().fold(f64::MAX, f64::min);

    // ① 표는 자기 문단 **첫 글자 줄보다 위에 있으면 안 된다.**
    //    종전 결함이 정확히 이 형태다(문단 흐름 시작 = 첫 줄보다 위).
    assert!(
        table_y > first_text + 1.0,
        "문단 끝의 TAC 표가 문단 첫 글자 줄 위에 놓였다 — #6614 회귀.          table_y={table_y:.1} first_text={first_text:.1} text_ys={text_ys:?}"
    );

    // ② 표 상단은 **저장 사다리의 표 줄 + om_top** 이어야 한다.
    //    첫 글자 줄이 `vertpos = 0` 이므로 그 줄이 좌표 기준이 된다.
    let px = |hu: i32| f64::from(hu) / 75.0;
    let expected = first_text + px(15228) + px(i32::from(OM));
    assert!(
        (table_y - expected).abs() <= 2.0,
        "표 상단이 저장 표 줄(vertpos=15228)+om_top 이어야 한다 — #6614 회귀.          table_y={table_y:.1} expected={expected:.1} text_ys={text_ys:?}"
    );
}
