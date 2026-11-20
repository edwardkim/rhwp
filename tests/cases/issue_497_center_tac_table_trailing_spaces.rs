//! [planet6897/rhwp-issue-docs#497] 관계형·고정형 회귀 — 가운데 정렬 문단 끝 줄의 TAC 표 뒤 말미 공백은 정렬 폭에 넣지 않는다.
//!
//! 주석 갱신: 2026-11-21 02:32 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue497/18561241_allowance_table.hwp` — 공개 조례 별표
//!   「[별표 8] 직급보조비의 가산금 지급 구분표(제6조 관련)(인천광역시 공무원 수당 지급 조례)」(HWP5,
//!   한컴오피스 2020 저장, 1쪽, 10k 코퍼스 #5892). 문단 4 는 `[TAC 표 2행×2열][공백][공백]` 이고
//!   가운데 정렬이며 PARA_LINE_SEG 가 없다(재조판 문단). 표 폭 44370HU, 바깥 여백 좌우 140HU.
//! - 독립 기준: `pdf/issue497/18561241_allowance_table-2022.pdf`(한컴오피스 2022 12.0.0.4547 Print →
//!   Hancom PDF, 1쪽). 표 왼쪽 괘선 x≈100.7~101.1px(96DPI) = 단 왼쪽 + (단 폭 − 표 − 바깥 여백)/2 +
//!   바깥 여백 왼쪽 — 말미 공백 둘을 뺀 가운데 정렬이다.
//! - 수정 전 rhwp(devel `411a9f944c`)는 공통 줄 구성(`inline_flow`)이 첫 공백(8px)을 줄 폭에 넣고 둘째
//!   공백만 걸어 표를 97.05px 에 두었다(4px 왼쪽).
//!
//! 측정: 2026-11-20~21(KST), `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 글꼴 `RHWP_FONT_PATH` 지원 웹폰트.
//! 전체 1쪽 = 한컴 1쪽. 1쪽 Native: 수정 전 85.16%, 수정 후 99.26%. fresh WASM 수정 후 100.00%.
//! 1쪽 문서이므로 두 경로의 전쪽 최저값은 각각 위 1쪽 값이다(Native 99.26%, fresh WASM 100.00%).
//! 증거: `mydocs/pr/assets/issue_497/{native,wasm,base-native}/`.
//!
//! 검사: 원본 계약(저장 줄 없음, 표 뒤 공백 둘, 가운데 정렬), 쪽수 1 = 한컴(고정형), 표+바깥 여백 상자의
//! 왼쪽 여유 = 오른쪽 여유(관계형), 표 뒤 문단이 표 아래(관계형).
//!
//! 반례(두 번째 검사): 같은 원본에서 말미 공백에 밑줄 글자 모양을 준 변형. 밑줄 친 공백은 보이는
//! 내용이라 본문 글줄 경로(`center_alignment_excludes_trailing_space`, #7081 ④)와 같이 정렬 폭에
//! 남아야 한다 — 표가 가운데보다 왼쪽에 놓인다(관계형). 합성 변형이며 한컴 출력 일치 증거가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::style::{Alignment, UnderlineType};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue497/18561241_allowance_table.hwp";
/// `[TAC 표][공백][공백]` 가운데 정렬 문단.
const TAC_PARA: usize = 4;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn find_node<'a>(
    node: &'a RenderNode,
    pred: &dyn Fn(&RenderNode) -> bool,
) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_node(child, pred))
}

/// (단 왼쪽 여유, 단 오른쪽 여유, 표 상자) — 여유는 표 + 바깥 여백 상자 기준.
fn table_gaps(core: &DocumentCore) -> (f64, f64, RenderNode) {
    let para = &core.document().sections[0].paragraphs[TAC_PARA];
    let table = para
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.common.treat_as_char => Some(table),
            _ => None,
        })
        .expect("TAC 표");
    let om_left = hu_to_px(i32::from(table.outer_margin_left));
    let om_right = hu_to_px(i32::from(table.outer_margin_right));

    let page = core.build_page_render_tree(0).expect("1쪽");
    let column = find_node(&page.root, &|node| {
        matches!(node.node_type, RenderNodeType::Column(0))
    })
    .expect("단 상자");
    let table_node = find_node(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(TAC_PARA) && t.cell_context.is_none())
    })
    .expect("문단 4 의 표 상자")
    .clone();
    let left_gap = table_node.bbox.x - om_left - column.bbox.x;
    let right_gap =
        column.bbox.x + column.bbox.width - (table_node.bbox.x + table_node.bbox.width + om_right);
    (left_gap, right_gap, table_node)
}

#[test]
fn centered_tac_table_excludes_trailing_spaces_from_line_width() {
    let core = load();
    assert_eq!(core.page_count(), 1, "한컴 Print 와 같은 1쪽");

    // 원본 계약: 저장 줄 없음(파서 합성 줄만), 표 뒤 공백 둘, 가운데 정렬.
    let doc = core.document();
    let para = &doc.sections[0].paragraphs[TAC_PARA];
    assert!(
        para.line_segs.iter().all(|seg| seg.tag & 0x8000_0000 != 0),
        "문단 {TAC_PARA} 은 저장 줄 없는 재조판 문단"
    );
    assert_eq!(para.text, "  ", "표 뒤 글자는 공백 둘");
    let positions = para.control_text_positions();
    let table_index = para
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
        .expect("TAC 표");
    assert_eq!(
        positions[table_index], 0,
        "표가 공백 앞(글자 위치 0)에 있다"
    );
    assert_eq!(
        doc.doc_info.para_shapes[para.para_shape_id as usize].alignment,
        Alignment::Center,
        "가운데 정렬 문단"
    );

    let (left_gap, right_gap, table_node) = table_gaps(&core);
    assert!(
        left_gap > 1.0 && right_gap > 1.0,
        "표 상자가 단 안에 여유를 두고 놓인다: 왼쪽 {left_gap:.2} 오른쪽 {right_gap:.2}"
    );
    assert!(
        (left_gap - right_gap).abs() <= 0.5,
        "말미 공백을 뺀 가운데 정렬 — 왼쪽 여유 {left_gap:.2} = 오른쪽 여유 {right_gap:.2}"
    );

    // 표 뒤 문단(비고)은 표 아래에서 시작한다.
    let page = core.build_page_render_tree(0).expect("1쪽");
    let next_line = find_node(&page.root, &|node| {
        matches!(&node.node_type, RenderNodeType::TextLine(line)
            if line.para_index == Some(TAC_PARA + 1) && line.line_index == Some(0))
    })
    .expect("문단 5 첫 줄");
    assert!(
        next_line.bbox.y >= table_node.bbox.y + table_node.bbox.height,
        "비고 줄 위 {:.1} 은 표 아래끝 {:.1} 아래",
        next_line.bbox.y,
        table_node.bbox.y + table_node.bbox.height
    );
}

/// 반례: 말미 공백이 밑줄 글자 모양이면 보이는 내용이므로 정렬 폭에 남는다.
#[test]
fn underlined_trailing_spaces_stay_in_centered_line_width() {
    let mut core = load();
    let mut doc = core.document().clone();
    let para = &doc.sections[0].paragraphs[TAC_PARA];
    let base_shape = para
        .char_shapes
        .first()
        .map(|r| r.char_shape_id as usize)
        .expect("글자 모양");
    let mut underlined = doc.doc_info.char_shapes[base_shape].clone();
    underlined.underline_type = UnderlineType::Bottom;
    underlined.raw_data = None;
    doc.doc_info.char_shapes.push(underlined);
    let underlined_id = (doc.doc_info.char_shapes.len() - 1) as u32;
    for shape in &mut doc.sections[0].paragraphs[TAC_PARA].char_shapes {
        shape.char_shape_id = underlined_id;
    }
    core.set_document(doc);

    let (left_gap, right_gap, _) = table_gaps(&core);
    assert!(
        left_gap + 1.0 < right_gap,
        "밑줄 친 말미 공백은 줄 폭에 남아 표가 가운데보다 왼쪽: 왼쪽 {left_gap:.2} 오른쪽 {right_gap:.2}"
    );
}
