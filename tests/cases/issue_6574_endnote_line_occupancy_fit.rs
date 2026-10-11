//! [#6574] 미주 단 하단 수용은 줄의 글자 상자로 잰다 — 저장 글자 높이가 그 줄의 인라인
//! 개체까지 담고, 글자 상자는 기준선 위 85% · 아래 15% 다.
//!
//! 두 반례를 실제 저장본과 한/글 정본 PDF 의 단 끝 줄로 잠근다.
//!
//! 1. 수식이 그려진 상자는 점유가 아니다. `3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7`
//!    17쪽 왼쪽 단 끝 줄(`그림 … 에서 중심이 …`)은 글자 상자 하단 1092.0px · 단 하단 1092.3px 인데,
//!    줄 안 수식은 기준선 정렬로 0.7px 아래에 그려져 하단이 1092.7px 이다. 정본(Hwp 2024
//!    13.0.0.3622)은 이 줄을 같은 단 1077.4px 에 둔다. 개체를 그린 상자로 재면 다음 단으로 넘긴다.
//! 2. 저장 기준선이 0 인 줄이 있다(보통 줄 `줄 높이 = 글자 높이 = 900, 기준선 0`). 글자 상자는
//!    줄 위보다 올라가지 않으므로 기준선을 글자 높이의 85% 로 읽는다. 저장 기준선을 그대로 쓰면
//!    이런 줄의 점유가 1.8px 로 줄어 단에 줄이 더 들어간다. `3-09월_교육_통합_2022` 10쪽
//!    정본의 단 끝 줄로 잠근다(왼쪽 `이어야 하므로 양수 …` 다음 줄 1051.8px, 오른쪽
//!    `… 라 하면 삼각형 ACE 에서 코사인법칙`).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 쪽(0-기준)의 단별 `(상단 y, 텍스트)` 줄 목록.
fn column_lines(core: &DocumentCore, page: u32) -> Vec<Vec<(f64, String)>> {
    let tree = core.build_page_render_tree(page).unwrap();
    let mut columns = Vec::new();
    fn find_columns<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(node.node_type, RenderNodeType::Column(_)) {
            out.push(node);
            return;
        }
        for child in &node.children {
            find_columns(child, out);
        }
    }
    fn text_lines(node: &RenderNode, out: &mut Vec<(f64, String)>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let mut text = String::new();
            fn runs(node: &RenderNode, out: &mut String) {
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    out.push_str(&run.text);
                }
                for child in &node.children {
                    runs(child, out);
                }
            }
            runs(node, &mut text);
            out.push((node.bbox.y, text));
            return;
        }
        for child in &node.children {
            text_lines(child, out);
        }
    }
    let mut found = Vec::new();
    find_columns(&tree.root, &mut found);
    for column in found {
        let mut lines = Vec::new();
        text_lines(column, &mut lines);
        columns.push(lines);
    }
    columns
}

#[test]
fn inline_equation_drawn_box_does_not_push_column_end_line() {
    let core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp"
    ))
    .unwrap();
    let columns = column_lines(&core, 16);
    let (y, text) = columns[0].last().expect("17쪽 왼쪽 단 줄");
    assert!(
        text.trim_start().starts_with("그림"),
        "정본처럼 `그림 …` 줄이 왼쪽 단 끝에 남아야 한다: {text:?}"
    );
    assert!(
        (y - 1077.4).abs() < 0.5,
        "정본 줄 위치 1077.4px, 실제 {y:.2}"
    );
}

#[test]
fn zero_stored_baseline_line_occupies_its_char_box() {
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/3-09월_교육_통합_2022.hwp"))
        .unwrap();
    let columns = column_lines(&core, 9);
    let left = &columns[0];
    let (before_last_y, before_last) = &left[left.len() - 2];
    let (last_y, _) = left.last().unwrap();
    assert!(
        before_last.trim_start().starts_with("이어야 하므로 양수"),
        "정본 왼쪽 단 끝 앞 줄: {before_last:?} ({before_last_y:.2})"
    );
    assert!(
        (last_y - 1051.8).abs() < 0.5,
        "정본 왼쪽 단 끝 줄 1051.8px, 실제 {last_y:.2}"
    );
    let (_, right_last) = columns[1].last().expect("10쪽 오른쪽 단 줄");
    assert!(
        right_last.contains("라 하면 삼각형"),
        "정본 오른쪽 단 끝 줄: {right_last:?}"
    );
}
