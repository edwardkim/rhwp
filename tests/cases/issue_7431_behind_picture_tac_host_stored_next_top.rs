//! [#7431] 글뒤 그림 뒤의 빈 TAC 표 host 문단이 저장 줄간격을 빠뜨려, 다음 문단이
//! 저장 사다리보다 위에 놓이던 결함의 가드.
//!
//! `samples/exam_eng.hwp` 2쪽 오른쪽 단 18번: 문단 104 는 글자 없이 글뒤 그림 2개와
//! 자리차지 TAC 표 하나를 담고 저장 줄 하나(vpos 2254, lh 22207 = 표 + 바깥 아래 여백,
//! ls 344)를 갖는다. 다음 문단 105(답지 ①)의 저장 vpos 24805 = 2254 + 22207 + 344 다.
//! 수정 전 rhwp 는 ls 344HU(4.59px)를 빠뜨려 ① 이하 답지 다섯 줄이 4.6px 위에 놓였다
//! (한컴 2020·2022 PDF 는 저장 사다리대로 ① 기준선 555.6/555.8px).
//!
//! 기대값은 구현이 아니라 **한/글이 저장한 사다리**에서 온다: 같은 단·같은 흐름의 두
//! 문단 사이 렌더 거리 = 저장 vpos 차이. 절대 좌표는 잠그지 않는다.
//!
//! - 대상: 단 첫 문단 103 → 105 (글뒤 그림 + TAC host 104 를 건넌다)
//! - 대조군: 105 → 106 (평범한 문단 사이 — 수정 전에도 성립)
//!
//! 비해당 경계(자리차지·어울림·글자처럼 개체가 앞에 있는 TAC host, #4622)는 이 판정에서
//! 제외되며 기존 `#4622`·`#7333` 시험이 지킨다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::TextWrap;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIXTURE: &str = "samples/exam_eng.hwp";
/// 2쪽(0 기반 1) 오른쪽 단의 문단들.
const PAGE: u32 = 1;
const COLUMN_FIRST: usize = 103;
const HOST: usize = 104;
const ANSWER_1: usize = 105;
const ANSWER_2: usize = 106;
const TOL_PX: f64 = 0.5;

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    let bytes = std::fs::read(&path).expect("required fixture samples/exam_eng.hwp");
    DocumentCore::from_bytes(&bytes).expect("exam_eng parse")
}

/// 본문 흐름(표 셀 밖)의 문단별 첫 줄 상단.
fn body_line_top(node: &RenderNode, para: usize) -> Option<f64> {
    match &node.node_type {
        RenderNodeType::Table(_) => return None,
        RenderNodeType::TextLine(line) if line.para_index == Some(para) => {
            return Some(node.bbox.y);
        }
        _ => {}
    }
    node.children
        .iter()
        .filter_map(|child| body_line_top(child, para))
        .reduce(f64::min)
}

fn stored_vpos(core: &DocumentCore, para: usize) -> i32 {
    core.document().sections[0].paragraphs[para]
        .line_segs
        .first()
        .unwrap_or_else(|| panic!("pi{para} stored line"))
        .vertical_pos
}

#[test]
fn fixture_shape_is_behind_pictures_then_tac_table_in_an_empty_host() {
    let core = core();
    let host = &core.document().sections[0].paragraphs[HOST];
    assert!(
        host.text.trim().is_empty(),
        "pi{HOST} host must be textless"
    );
    let mut behind_pictures = 0;
    let mut tac_tables = 0;
    for control in &host.controls {
        match control {
            Control::Picture(p)
                if !p.common.treat_as_char
                    && matches!(p.common.text_wrap, TextWrap::BehindText) =>
            {
                behind_pictures += 1
            }
            Control::Table(t) if t.common.treat_as_char => tac_tables += 1,
            _ => {}
        }
    }
    assert!(behind_pictures >= 1, "pi{HOST} has behind-text pictures");
    assert_eq!(tac_tables, 1, "pi{HOST} has one TAC table");
    let seg = &host.line_segs[0];
    assert_eq!(
        seg.vertical_pos + seg.line_height + seg.line_spacing,
        stored_vpos(&core, ANSWER_1),
        "stored ladder: next top = host vpos + lh + ls"
    );
    assert!(
        seg.line_spacing > 0,
        "the host line spacing is what was dropped"
    );
}

#[test]
fn paragraph_after_behind_picture_tac_host_keeps_stored_ladder_distance() {
    let core = core();
    let root = core.build_page_render_tree(PAGE).expect("page 2 tree").root;
    let top = |p: usize| body_line_top(&root, p).unwrap_or_else(|| panic!("pi{p} line on p2"));

    let (y_first, y_a1, y_a2) = (top(COLUMN_FIRST), top(ANSWER_1), top(ANSWER_2));
    let (v_first, v_a1, v_a2) = (
        stored_vpos(&core, COLUMN_FIRST),
        stored_vpos(&core, ANSWER_1),
        stored_vpos(&core, ANSWER_2),
    );

    // 대조군: 평범한 문단 사이는 저장 거리 그대로다.
    let control = (y_a2 - y_a1) - hu_to_px(v_a2 - v_a1);
    assert!(
        control.abs() < TOL_PX,
        "control pi{ANSWER_1}->pi{ANSWER_2}: rendered {:.2} vs stored {:.2}",
        y_a2 - y_a1,
        hu_to_px(v_a2 - v_a1)
    );

    // 대상: 글뒤 그림 + TAC 표 host 를 건너도 저장 거리 그대로여야 한다.
    let diff = (y_a1 - y_first) - hu_to_px(v_a1 - v_first);
    assert!(
        diff.abs() < TOL_PX,
        "pi{COLUMN_FIRST}->pi{ANSWER_1} across behind-picture TAC host: rendered {:.2} vs \
         stored {:.2} (diff {diff:+.2}px; #7431 dropped host ls 344HU = -4.59px)",
        y_a1 - y_first,
        hu_to_px(v_a1 - v_first)
    );
}
