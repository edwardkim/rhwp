//! [#8641] 관계형·고정형 회귀 — 글자처럼 취급(TAC) 표가 문단의 둘째 저장 줄에 있고 앞 줄이
//! 따로 있으면, 표는 앞 줄 다음 자기 저장 줄에서 시작한다(앞 줄 위끝 + 저장 줄 차이 + 표
//! 바깥 위 여백).
//!
//! 주석 갱신: 2026-11-05 (KST) — 측정값은 아래 표
//!
//! - 입력 A: `samples/issue8641/field_table_after_page_box.hwpx` — 서울시 결재문서 원본
//!   (36306589, 한컴 2020 저장, 1쪽). 문단 2 = 누름틀 시작 + 13×7 TAC 표(저장 줄 1) + 쪽 기준
//!   비-TAC 1×1 발신명의 상자. 상자가 같은 저장 줄에서 먼저 정렬돼 "첫 배치 표"가 되면
//!   누름틀 줄(줄 0)을 아무 표도 내보내지 않는다(조판 결함).
//! - 입력 B: `samples/issue8641/table_after_tall_prefix_line_trimmed.hwpx` — 고용노동부 보고서
//!   (1490000-200600005, HWP5 53쪽) 17쪽 문단 220 을 문단 0·218~220 만 남겨 한/글 2022 로
//!   다시 저장한 축소본. 줄 0 은 「.」 한 글자(lh 25769 / th 1100), 12×6 TAC 표는 줄 1. 그리기의
//!   표 원점 보정이 앞 줄 흐른 양을 저장 lh 로 세어 표를 앞 줄 위로 되감는다(그리기 결함).
//! - 독립 기준: 같은 입력의 한/글 2022 Print PDF
//!   `pdf/issue8641_field_table_after_page_box-2020.pdf`(1쪽),
//!   `pdf/issue8641_table_after_tall_prefix_line_trimmed-2020.pdf`(1쪽).
//! - 수정 전 rhwp(devel 6e1de40620): A 는 표가 누름틀 줄 높이(7.3px)만큼 위, B 는 표가
//!   y=−90.2px(쪽 위 밖).
//!
//! 측정(2026-11-05, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣, 수정 코드):
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | Native | fresh WASM | 수정 전 Native |
//! |---|---|---|---|---|
//! | field_table_after_page_box | 1 | 99.98 | 99.98 | 71.29 |
//! | table_after_tall_prefix_line_trimmed | 1 | 100.00 | 100.00 | 6.11 |
//!
//! 두 경로 전쪽 최저: 99.98(A Native·WASM — 쪽 아래 발신명의 상자 글자 폭 차이).
//!
//! 증거: `mydocs/pr/assets/issue_8641/{native,wasm}/*_silhouette.tsv`, `review/*.png`.
//! 검사는 표·줄의 쪽·문단 소속과 원본 저장 줄에서 계산한 표 원점 관계(관계형), 독립 PDF
//! 전체 쪽수(고정형)다. 절대 px 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn core(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn find_table(node: &RenderNode, para: usize, control: usize) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && table.control_index == Some(control) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para, control))
}

fn find_line(node: &RenderNode, para: usize, line: u32) -> Option<BoundingBox> {
    if let RenderNodeType::TextLine(text_line) = &node.node_type {
        if text_line.para_index == Some(para) && text_line.line_index == Some(line) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_line(child, para, line))
}

/// 표가 host 줄 0 다음 자기 저장 줄(줄 1)에서 시작하는지 검사한다.
fn assert_table_on_its_stored_line(sample: &str, para: usize, control: usize) {
    let core = core(sample);
    assert_eq!(core.page_count(), 1, "{sample}: 한컴 Print PDF 1쪽");

    let host = &core.document().sections[0].paragraphs[para];
    let (Some(first), Some(owner)) = (host.line_segs.first(), host.line_segs.get(1)) else {
        panic!("{sample}: 문단 {para} 저장 줄 두 개");
    };
    let Some(Control::Table(table)) = host.controls.get(control) else {
        panic!("{sample}: 문단 {para} 컨트롤 {control} 은 표");
    };
    assert!(table.common.treat_as_char, "{sample}: 글자처럼 취급 표");

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|error| panic!("{sample} 1쪽 렌더 트리: {error:?}"));
    let line0 = find_line(&tree.root, para, 0)
        .unwrap_or_else(|| panic!("{sample}: 문단 {para} 줄 0 을 그린다"));
    let frame = find_table(&tree.root, para, control)
        .unwrap_or_else(|| panic!("{sample}: 표 ({para},{control}) 를 그린다"));

    let expected = line0.y
        + px(owner.vertical_pos - first.vertical_pos)
        + px(i32::from(table.outer_margin_top));
    assert!(
        (frame.y - expected).abs() <= 0.5,
        "{sample}: 표 위끝 {:.1} = 줄 0 위끝 {:.1} + 저장 줄 차이 + 바깥 위 여백 = {expected:.1}",
        frame.y,
        line0.y
    );
    assert!(
        frame.y > line0.y && frame.y >= tree.root.bbox.y,
        "{sample}: 표는 앞 줄 아래, 쪽 안에서 시작한다"
    );
}

#[test]
fn tac_table_after_field_line_keeps_line_when_page_box_sorts_first() {
    assert_table_on_its_stored_line("samples/issue8641/field_table_after_page_box.hwpx", 2, 1);
}

#[test]
fn tac_table_after_line_with_object_height_does_not_rewind() {
    assert_table_on_its_stored_line(
        "samples/issue8641/table_after_tall_prefix_line_trimmed.hwpx",
        3,
        0,
    );
}
