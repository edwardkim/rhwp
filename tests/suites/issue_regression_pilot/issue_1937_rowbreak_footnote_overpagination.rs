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

// 원문 내용과 표시용 각주 번호를 따로 검사한다. 각주 번호의 TextRun은
// 원문 문자 오프셋을 소유하지 않으며 독립 Print에는 별도로 표시된다.
fn source_text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return if run.char_start.is_some() {
            run.text.clone()
        } else {
            String::new()
        };
    }
    node.children.iter().map(source_text).collect()
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
    let mut annotated_cell = String::new();
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
                        .push_str(&compact(&source_text(cell)));
                    if key == (44, 2) {
                        annotated_cell.push_str(&compact(&text(cell)));
                    }
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
    assert_eq!(
        annotated_cell, "정책실현24)25)재정조달",
        "독립 Print37쪽의 원문과 두 각주 참조 번호 순서·누락·중복"
    );
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

// #7445 보정67: 같은 원본 전50쪽 Native/fresh WASM raw90% 이상 및38쪽98% 이상,
// 독립 Print의 review·overlay 직접 판독 뒤 추가한다. PDF의72dpi 괘선 좌표를96dpi로
// 변환했다. 쪽·원본 표·셀 소속으로 선택하며 절대 영역을 이용해 대상을 찾지 않는다.
struct NativeSpanPrintOutput(std::path::PathBuf);
impl NativeSpanPrintOutput {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("output/pr-review/regression-temp");
        std::fs::create_dir_all(&root).expect("ignored 회귀 출력 폴더");
        let path = root.join(format!(
            "rhwp-1937-print-span-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("이 검사 소유의 Native 출력 폴더");
        Self(path)
    }
}
impl Drop for NativeSpanPrintOutput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn print_source_node<'a>(
    node: &'a serde_json::Value,
    predicate: &impl Fn(&serde_json::Value) -> bool,
) -> Option<&'a serde_json::Value> {
    if predicate(node) {
        return Some(node);
    }
    node["children"]
        .as_array()?
        .iter()
        .find_map(|child| print_source_node(child, predicate))
}

#[test]
fn native_completed_span_tail_keeps_independent_print_borders() {
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let output = NativeSpanPrintOutput::new();
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1937_rowbreak_footnote_overpagination.hwp");
    let result = std::process::Command::new(binary)
        .arg("export-render-tree")
        .arg(source)
        .arg("-o")
        .arg(&output.0)
        .args(["-p", "37", "--compat", "2022"])
        .output()
        .expect("Native CLI의 최종 출력 실행");
    assert!(result.status.success(), "Native 내보내기 실패: {result:?}");
    let root: serde_json::Value = serde_json::from_slice(
        &std::fs::read(output.0.join("render_tree_038.json")).expect("독립 Print38쪽 대응 출력"),
    )
    .expect("Native RenderTree JSON");
    let table = print_source_node(&root, &|node| node["type"] == "Table" && node["pi"] == 306)
        .expect("원본 문단306의 표 소유");
    let cell = print_source_node(table, &|node| {
        node["type"] == "Cell" && node["row"] == 45 && node["col"] == 2
    })
    .expect("원본 병합 칸45:2의 이어받은 꼬리");
    let b = &cell["bbox"];
    let x = b["x"].as_f64().unwrap();
    let y = b["y"].as_f64().unwrap();
    let edges = [
        x,
        y,
        x + b["w"].as_f64().unwrap(),
        y + b["h"].as_f64().unwrap(),
    ];
    // 독립 issue1937_rowbreak_footnote_overpagination-2020.pdf38쪽의 괘선 교차점.
    let expected = [323.825_338, 148.317_301, 395.786_662, 209.530_680];
    for (actual, expected) in edges.into_iter().zip(expected) {
        assert!(
            (actual - expected).abs() <= 2.0,
            "완결 병합 꼬리의 독립 Print 괘선: {actual} vs {expected} CSS px"
        );
    }
}
