//! [#8648] 관계형·고정형 회귀 — 칸 안의 쪽 기준 「글 앞으로」 표는 칸의 흐름 높이에 쌓이지
//! 않는다. 쪽 아래 정렬 발신명의 상자는 저장 높이 그대로 본문 아래끝에 놓인다.
//!
//! 주석 갱신: 2026-11-06 (KST) — 측정값은 아래 표
//!
//! - 입력: `samples/issue8648/sender_box_page_overlay_stamp.hwpx` — 서울시 결재문서
//!   (36355589, 한컴 2020 저장, 1쪽)를 한/글 2022 로 다시 저장한 뒤 최상위 문단 0·5·6 만
//!   남기고(문단 5·6 은 누름틀 하나를 나눠 가짐) 문단 0 의 표 두 개(시행·결재 상자, 이
//!   회귀와 무관한 별도 원인)를 지워 다시 한/글 2022 로 저장한 축소본.
//! - 문단 2 컨트롤 1 = 쪽 기준(vertRelTo=PAGE)·아래 정렬 1×1 발신명의 상자(선언 9513HU).
//!   칸 문단 한 줄에 글자처럼 취급 1×3 표(7593HU)와 쪽 기준 「글 앞으로」 빈 직인 칸
//!   (7226HU)이 함께 있다.
//! - 독립 기준: 같은 입력의 한/글 2022 Print PDF
//!   `pdf/issue8648_sender_box_page_overlay_stamp-2020.pdf`(1쪽).
//! - 수정 전 rhwp(devel ece9c3383b): 직인 칸을 1×3 표 아래로 쌓아 상자를 197.6px 로 키우고,
//!   아래 정렬이 상자를 70.8px 위로 올렸다(본문 「총무부장」 글자가 한컴보다 위).
//!
//! 측정(2026-11-06, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣):
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | Native | fresh WASM | 수정 전 Native |
//! |---|---|---|---|---|
//! | sender_box_page_overlay_stamp | 1 | 99.97 | 100.00 | 93.11 |
//!
//! 두 경로 전쪽 최저: 99.97(Native).
//!
//! 증거: `mydocs/pr/assets/issue_8648/{native,native_base,wasm}/*_silhouette.tsv`,
//! `review/*.png`. 검사는 상자 높이와 저장 선언 높이, 상자 안 표의 가운데 정렬, 상자 아래끝과
//! 쪽 아래 여백선의 관계(관계형)와 독립 PDF 전체 쪽수(고정형)다. 절대 px 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{TextWrap, VertRelTo};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8648/sender_box_page_overlay_stamp.hwpx";
const HOST_PARA: usize = 2;
const BOX_CONTROL: usize = 1;

fn px(hu: i64) -> f64 {
    hu as f64 * 96.0 / 7200.0
}

fn find_table(node: &RenderNode, para: usize, control: usize) -> Option<&RenderNode> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && table.control_index == Some(control) {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para, control))
}

#[test]
fn page_anchored_overlay_in_cell_does_not_grow_bottom_aligned_box() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    assert_eq!(core.page_count(), 1, "한컴 Print PDF 1쪽");

    let section = &core.document().sections[0];
    let Some(Control::Table(sender)) = section.paragraphs[HOST_PARA].controls.get(BOX_CONTROL)
    else {
        panic!("문단 {HOST_PARA} 컨트롤 {BOX_CONTROL} 은 발신명의 상자 표");
    };
    assert!(!sender.common.treat_as_char);
    assert_eq!(sender.common.vert_rel_to, VertRelTo::Page, "쪽 기준 상자");
    let cell_para = &sender.cells[0].paragraphs[0];
    let inner_tac = cell_para
        .controls
        .iter()
        .position(|c| matches!(c, Control::Table(t) if t.common.treat_as_char));
    let stamp = cell_para.controls.iter().position(|c| {
        matches!(c, Control::Table(t) if !t.common.treat_as_char
            && t.common.text_wrap == TextWrap::InFrontOfText
            && t.common.vert_rel_to == VertRelTo::Page)
    });
    let (Some(inner_tac), Some(_stamp)) = (inner_tac, stamp) else {
        panic!("상자 칸 문단에 글자처럼 취급 표와 쪽 기준 글 앞으로 표가 함께 있다");
    };

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("1쪽 렌더 트리: {e:?}"));
    let frame: BoundingBox = find_table(&tree.root, HOST_PARA, BOX_CONTROL)
        .unwrap_or_else(|| panic!("발신명의 상자를 그린다"))
        .bbox
        .clone();
    let inner: BoundingBox = find_table(
        find_table(&tree.root, HOST_PARA, BOX_CONTROL).unwrap(),
        0,
        inner_tac,
    )
    .unwrap_or_else(|| panic!("상자 안 글자처럼 취급 표를 그린다"))
    .bbox
    .clone();

    let declared = px(i64::from(sender.common.height));
    assert!(
        (frame.height - declared).abs() <= 0.5,
        "상자 높이 {:.1} = 저장 선언 {declared:.1} (글 앞으로 직인 칸을 흐름에 쌓지 않는다)",
        frame.height
    );

    let top_gap = inner.y - frame.y;
    let bottom_gap = frame.y + frame.height - (inner.y + inner.height);
    assert!(
        top_gap > 0.0 && (top_gap - bottom_gap).abs() <= 0.5,
        "가운데 정렬 칸: 표 위 여유 {top_gap:.1} = 아래 여유 {bottom_gap:.1}"
    );

    let page = &section.section_def.page_def;
    let bottom_line =
        px(i64::from(page.height) - i64::from(page.margin_bottom) - i64::from(page.margin_footer));
    assert!(
        (frame.y + frame.height - bottom_line).abs() <= 0.5,
        "쪽 기준 아래 정렬 상자 아래끝 {:.1} = 쪽 아래 여백선 {bottom_line:.1}",
        frame.y + frame.height
    );
}
