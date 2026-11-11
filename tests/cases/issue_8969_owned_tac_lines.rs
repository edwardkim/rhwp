//! 관계형·고정형 — [#8969] 글자 취급 표가 각자 소유한 저장 줄의 높이·간격과 바깥 여백을 흐름이 그대로 진행한다.
//! 주석 갱신: 2026-11-12 06:52 KST (새 파일)
//!
//! 입력: 공개 결재문서 36400485(한컴오피스 2020 저장 HWPX) — `samples/issue8969/36400485-owned-tac-lines.hwpx`.
//! 기준: 한컴 engine 2020 Print PDF(PrintMethod 0) `pdf/issue8969/36400485-2020.pdf`, 11쪽.
//! - 고정형: 전체 11쪽.
//! - 관계형(4쪽): pi=27 은 TAC 표 두 개가 줄 0(3쪽)·줄 1(vpos 를 되감아 4쪽 맨 위)을 소유한다.
//!   둘째 표(`flowWithText="0"`)가 저장 줄 전체를 진행해야 같은 쪽 pi=45 TAC 표가 저장 vpos 차만큼
//!   아래에 놓인다. 수정 전에는 typeset 이 블록 표로 표 높이만 진행해 23.25px 위였다.
//! - 관계형(2쪽): pi=2 는 글줄 하나 뒤 TAC 표 두 개가 줄 1·2 를 소유한다. 둘째 표는 첫 표와의 저장
//!   vpos 차와 위 여백 차만큼 아래에 놓인다. 수정 전에는 첫 표 아래 여백·줄간격과 둘째 표 위 여백을
//!   잃어 15.7px 위였다. 뒤 pi=5 표도 같은 저장 사다리에 남는다.
//!
//! 측정(2026-11-12, 코드 head e1647446f5, base eb84afc036, `scripts/visual_sweep.py --silhouette-only --dpi 96`,
//! raw = `silhouette_raw_match_percent`, `RHWP_FONT_PATH` 글꼴 공급, WASM 은 같은 폴더를 `--font-path` 로):
//! - 36400485 Native 11쪽 / 11쪽: p2 99.80 (base eb84afc036 86.72), p4 99.49 (base 65.58). 그 밖의 쪽은 base 와 같다.
//! - 36400485 fresh WASM 11쪽 / 11쪽: p2 99.80, p4 99.47.
//! - 두 경로 전쪽 최저: Native 80.97, WASM 80.80. 둘 다 p10 이고 base 와 같은 미달이다. 별개 원인이라
//!   #8973 으로 분리했으며, 이 파일은 p2·p4 만 검사한다.
//! - 증거: `mydocs/pr/assets/issue_8969/`
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::hwpunit_to_px;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8969/36400485-owned-tac-lines.hwpx";
const DPI: f64 = 96.0;

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("open")
}

fn find_table(node: &RenderNode, para: usize, control: usize) -> Option<f64> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para)
            && table.control_index == Some(control)
            && table.cell_context.is_none()
        {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para, control))
}

fn table_top(core: &DocumentCore, page: u32, para: usize, control: usize) -> f64 {
    let tree = core.build_page_render_tree(page).expect("page");
    find_table(&tree.root, para, control)
        .unwrap_or_else(|| panic!("{}쪽 pi={para} ci={control} 표", page + 1))
}

/// (저장 줄의 vpos, 표 바깥 위 여백) — 표가 소유한 저장 줄과 위 여백은 원본 값이다.
fn stored_line(core: &DocumentCore, para: usize, control: usize, line: usize) -> (i32, f64) {
    let paragraph = &core.document().sections[0].paragraphs[para];
    let Control::Table(table) = &paragraph.controls[control] else {
        panic!("pi={para} ci={control} 는 표");
    };
    assert!(
        table.common.treat_as_char,
        "pi={para} ci={control} 글자 취급"
    );
    let seg = &paragraph.line_segs[line];
    assert_eq!(
        i64::from(seg.line_height),
        i64::from(table.common.height)
            + i64::from(table.outer_margin_top)
            + i64::from(table.outer_margin_bottom),
        "pi={para} 줄 {line} 은 표 혼자의 줄"
    );
    (
        seg.vertical_pos,
        hwpunit_to_px(i32::from(table.outer_margin_top), DPI),
    )
}

fn assert_ladder(
    core: &DocumentCore,
    page: u32,
    upper: (usize, usize, usize),
    lower: (usize, usize, usize),
) {
    let (upper_vpos, upper_margin) = stored_line(core, upper.0, upper.1, upper.2);
    let (lower_vpos, lower_margin) = stored_line(core, lower.0, lower.1, lower.2);
    let expected = hwpunit_to_px(lower_vpos - upper_vpos, DPI) + lower_margin - upper_margin;
    let actual = table_top(core, page, lower.0, lower.1) - table_top(core, page, upper.0, upper.1);
    assert!(
        (actual - expected).abs() <= 1.0,
        "{}쪽 pi={} ci={} 는 pi={} ci={} 아래 저장 사다리 {expected:.2}px, 실제 {actual:.2}px",
        page + 1,
        lower.0,
        lower.1,
        upper.0,
        upper.1,
    );
}

#[test]
fn page_count_matches_hancom_print() {
    assert_eq!(core().page_count(), 11, "한컴 Print 11쪽");
}

#[test]
fn second_owned_line_table_advances_its_stored_line() {
    let core = core();
    // 4쪽: pi=27 줄 1(vpos 0 되감김)의 둘째 표와 같은 쪽 pi=45 단일 TAC 표.
    let (reset_vpos, _) = stored_line(&core, 27, 1, 1);
    assert_eq!(reset_vpos, 0, "둘째 표 줄은 새 쪽에서 vpos 를 되감는다");
    assert_ladder(&core, 3, (27, 1, 1), (45, 0, 0));
}

#[test]
fn consecutive_owned_line_tables_keep_outer_margins_and_spacing() {
    let core = core();
    // 2쪽: pi=2 글줄 0 뒤 줄 1·2 를 소유한 두 표, 그리고 뒤 문단 pi=5 표.
    assert_ladder(&core, 1, (2, 2, 1), (2, 3, 2));
    assert_ladder(&core, 1, (2, 2, 1), (5, 0, 0));
}
