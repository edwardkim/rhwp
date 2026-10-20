//! Issue #1937: 원본 표와 각주의 물리 쪽·내용 소유를 독립 한컴 Print50쪽과 대조한다.
//! 기존 과분할 검사 안에서 종료·이어받기와 내용 보존을 함께 보호한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::{BTreeMap, BTreeSet};

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        // 자동 글머리표는 원문 문자열에 없는 paint다. 원문의 실제 기호는 보존한다.
        if run.char_start.is_none() && run.text == "•" {
            return String::new();
        }
        return run.display_text.as_ref().unwrap_or(&run.text).clone();
    }
    node.children.iter().map(text).collect()
}

fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn table(node: &RenderNode, paragraph: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(paragraph)) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table(child, paragraph))
}

fn owned_line<'a>(node: &'a RenderNode, content: &str) -> Option<&'a RenderNode> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && compact(&text(node)) == content {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| owned_line(child, content))
}

fn note_ink_top(node: &RenderNode) -> f64 {
    if matches!(&node.node_type, RenderNodeType::TextRun(_) if !text(node).trim().is_empty()) {
        return node.bbox.y;
    }
    node.children
        .iter()
        .map(note_ink_top)
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn rowbreak_footnote_table_does_not_over_paginate() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1937_rowbreak_footnote_overpagination.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("원본 HWP5")).unwrap();
    assert_eq!(core.page_count(), 50, "독립 한컴 Print의 전체50쪽");
    let mut cells = BTreeMap::<(u16, u16), String>::new();
    let mut headers = BTreeSet::new();
    let mut notes = Vec::new();
    let mut first_table_page = None;
    for page in 0..core.page_count() {
        let root = core.build_page_render_tree(page).unwrap().root;
        let areas = root
            .children
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::FootnoteArea))
            .collect::<Vec<_>>();
        notes.push(compact(&areas.iter().map(|n| text(n)).collect::<String>()));
        if let Some(fragment) = table(&root, 306) {
            first_table_page.get_or_insert(page + 1);
            for area in areas {
                assert!(
                    fragment.bbox.y + fragment.bbox.height <= note_ink_top(area) + 0.5,
                    "{}쪽: 표와 실제 각주 잉크가 겹치지 않음",
                    page + 1
                );
            }
            for cell in &fragment.children {
                let RenderNodeType::TableCell(owner) = &cell.node_type else {
                    continue;
                };
                let key = (owner.row, owner.col);
                if owner.row != 0 || headers.insert(key) {
                    cells
                        .entry(key)
                        .or_default()
                        .push_str(&compact(&text(cell)));
                }
            }
            if page == 37 {
                let cell = fragment.children.iter().find(|n| compact(&text(n)) == "재정조달"
                    && matches!(&n.node_type, RenderNodeType::TableCell(c) if c.row == 45 && c.col == 2))
                    .expect("독립 Print38쪽의 완료 병합 꼬리");
                let line = owned_line(cell, "재정조달").expect("이 조각이 소유하는 한 문단");
                assert!(
                    (line.bbox.y + line.bbox.height / 2.0 - cell.bbox.y - cell.bbox.height / 2.0)
                        .abs()
                        < 0.5,
                    "원본 Center 정렬은 실제 수용한 독립 행 상자를 소비함"
                );
            }
        }
        // 같은 원본·PDF의9/13쪽은 양쪽 raw99%대이며 독립 괘선 좌표를 ±2CSSpx로 보호한다.
        // 192DPI 비교의 raster 좌표가 아니라96DPI render tree 좌표다.
        if matches!(page, 8 | 12) {
            let fragment = table(&root, 99).expect("독립 Print의 부과금 법령표");
            let bottom = if page == 8 {
                985.322_672_526
            } else {
                1_001.145_334_880
            };
            for (actual, expected) in [
                (fragment.bbox.x, 91.150_665_283),
                (fragment.bbox.y, 114.753_336_589),
                (fragment.bbox.x + fragment.bbox.width, 702.182_617_188),
                (fragment.bbox.y + fragment.bbox.height, bottom),
            ] {
                assert!(
                    (actual - expected).abs() <= 2.0,
                    "{}쪽 법령표의 독립 Print 괘선: {actual} vs {expected}",
                    page + 1
                );
            }
        }
    }
    assert_eq!(
        first_table_page,
        Some(35),
        "앞 본문 뒤 같은 Print35쪽에서 표 시작"
    );
    let rhwp::model::control::Control::Table(source) =
        &core.document().sections[0].paragraphs[306].controls[0]
    else {
        panic!("원본 표")
    };
    assert_eq!(cells.len(), source.cells.len(), "원본의 모든 셀 소유 보존");
    for cell in &source.cells {
        let expected = compact(
            &cell
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<String>(),
        );
        assert_eq!(
            cells[&(cell.row, cell.col)],
            expected,
            "원본 셀 {}:{}의 내용 순서·누락·중복",
            cell.row,
            cell.col
        );
    }
    let prefix = "「예금자보호법」제2조(정의)제1호";
    let tail = "상호저축은행중앙회";
    assert!(notes[41].contains(prefix) && !notes[41].contains(tail));
    assert!(!notes[42].contains(prefix) && notes[42].contains(tail));
    assert!(
        notes[42].find(tail).unwrap() < notes[42].find("32)").unwrap(),
        "31번 각주 꼬리는 다음 각주 앞에 한 번 이어짐"
    );
    assert!(
        notes[42].contains("40)") && !notes[43].contains("40)"),
        "마지막 본문 참조의 각주는 독립 Print43쪽에 수용됨"
    );
}
