//! [#3592] 행 범위 부분 렌더로 그려진 중첩 표에서도 셀 `vertical_align` 을 존중해야 한다.
//!
//! ## 결함
//! `table_layout.rs` 의 셀 배치는 행 범위 필터가 걸리면 정렬을 통째로 Top 으로 바꾼다.
//!
//! ```text
//! let effective_valign = if row_filter.is_some() { VerticalAlign::Top }
//!                        else { cell.vertical_align };
//! ```
//!
//! 페이지에 걸쳐 이어지는 조각에서는 타당하지만, **그 조각 안에 온전히 들어가는 셀**까지
//! Top 이 된다. 세로로 긴 병합 셀의 라벨이 정중앙이 아니라 맨 위에 붙는다.
//!
//! ## 오라클
//! `pdf/kps-ai-2022.pdf` p65(= rhwp p66) 실측. 한컴은 `1. 기본정보` / `2. 운영계획` /
//! `구분` 같은 병합 라벨 셀을 **정중앙**에 놓는다. rhwp 는 상단(순수 여백 3.8px)에 붙인다.
//!
//! ## 가드
//! 모델이 `VerticalAlign::Center` 로 지정한 중첩 표 셀은 렌더에서도 상하 여유가 대칭이어야
//! 한다. 텍스트로 모델↔렌더를 대응시켜 쪽 번호 변동에 영향받지 않게 한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::table::{Table, VerticalAlign};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::collections::HashMap;

const SAMPLE: &str = "samples/kps-ai.hwp";

/// 오라클에서 정중앙으로 확인한 병합 라벨 셀들 (공백 제거 텍스트).
const CENTERED_LABELS: &[&str] = &["1.기본정보", "2.운영계획", "5.종합의견"];

/// 상하 여유 비대칭 허용치(px). 줄 높이 반올림 잔차만 흡수한다.
const SYMMETRY_TOL: f64 = 2.0;

fn nonspace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 중첩 표(depth≥1) 안, 문단 1개인 셀의 (텍스트 → 세로정렬).
fn collect_nested_cell_valign(
    table: &Table,
    depth: usize,
    out: &mut HashMap<String, VerticalAlign>,
) {
    for cell in &table.cells {
        if depth >= 1 && cell.paragraphs.len() == 1 {
            let t = nonspace(&cell.paragraphs[0].text);
            if t.chars().count() >= 3 {
                out.insert(t, cell.vertical_align);
            }
        }
        for para in &cell.paragraphs {
            for ctrl in &para.controls {
                if let Control::Table(inner) = ctrl {
                    collect_nested_cell_valign(inner, depth + 1, out);
                }
            }
        }
    }
}

fn subtree_text(node: &RenderNode) -> String {
    let mut out = String::new();
    fn walk(n: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(r) = &n.node_type {
            out.extend(r.display_or_text().chars().filter(|c| !c.is_whitespace()));
        }
        for c in &n.children {
            walk(c, out);
        }
    }
    walk(node, &mut out);
    out
}

/// 렌더된 셀의 (상단 여유, 하단 여유) — 텍스트 키.
fn collect_rendered(node: &RenderNode, out: &mut HashMap<String, (f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        let lines: Vec<&RenderNode> = node
            .children
            .iter()
            .filter(|c| matches!(c.node_type, RenderNodeType::TextLine(_)))
            .collect();
        if let (Some(first), Some(last)) = (lines.first(), lines.last()) {
            let text = subtree_text(node);
            if text.chars().count() >= 3 {
                let top = first.bbox.y - node.bbox.y;
                let bot = (node.bbox.y + node.bbox.height) - (last.bbox.y + last.bbox.height);
                out.entry(text).or_insert((top, bot));
            }
        }
    }
    for c in &node.children {
        collect_rendered(c, out);
    }
}

fn load() -> (HashMap<String, VerticalAlign>, HashMap<String, (f64, f64)>) {
    let bytes = std::fs::read(SAMPLE).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"));

    let mut model = HashMap::new();
    for section in &core.document().sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                if let Control::Table(t) = ctrl {
                    collect_nested_cell_valign(t, 0, &mut model);
                }
            }
        }
    }

    let mut rendered = HashMap::new();
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page)
            .unwrap_or_else(|e| panic!("build page {page}: {e:?}"));
        collect_rendered(&tree.root, &mut rendered);
    }
    (model, rendered)
}

#[test]
fn oracle_centered_merged_labels_are_vertically_centered() {
    let (model, rendered) = load();
    let mut bad = Vec::new();
    for label in CENTERED_LABELS {
        let align = model
            .get(*label)
            .unwrap_or_else(|| panic!("모델에서 {label:?} 셀을 찾지 못했다"));
        assert!(
            matches!(align, VerticalAlign::Center),
            "{label:?} 는 모델에서 Center 여야 한다 (실제 {align:?})"
        );
        let (top, bot) = rendered
            .get(*label)
            .copied()
            .unwrap_or_else(|| panic!("렌더에서 {label:?} 셀을 찾지 못했다"));
        if (top - bot).abs() > SYMMETRY_TOL {
            bad.push((label, top, bot));
        }
    }
    assert!(
        bad.is_empty(),
        "Center 지정 병합 라벨이 세로 중앙에 놓이지 않았다 (한컴 pdf/kps-ai-2022.pdf p65 실측 = 정중앙). \
         {:?}",
        bad
    );
}

/// 문서 전체 규모 가드 — Center 지정 중첩 셀은 상단정렬로 그려지지 않는다.
///
/// 행 범위 필터가 잘리지 않은 셀까지 Top 으로 덮던 시점에는 이 문서에서 27건이었다.
#[test]
fn center_aligned_nested_cells_are_not_top_aligned() {
    let (model, rendered) = load();
    let mut top_aligned = Vec::new();
    for (text, align) in &model {
        if !matches!(align, VerticalAlign::Center) {
            continue;
        }
        let Some(&(top, bot)) = rendered.get(text) else {
            continue;
        };
        if bot > top + SYMMETRY_TOL {
            top_aligned.push(text.clone());
        }
    }
    assert!(
        top_aligned.is_empty(),
        "Center 지정 중첩 셀 {}개가 상단정렬로 그려졌다",
        top_aligned.len()
    );
}
