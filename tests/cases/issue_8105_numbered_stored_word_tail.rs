//! [#8105] 문단 머리(번호) 문단의 저장 줄 낱말 꼬리.
//!
//! 한/글 2014 로 저장한 1342000 교육과정 지도(HWP5)의 칸 문단 `…갈등 현상 탐색하` / `기` 는 저장 줄이
//! 두 줄이지만, 정본 한/글 2020 Print
//! (`mydocs/pr/assets/issue7445/education-curriculum-map/existing-hwp-hancom2020-print.pdf`)와 한/글
//! 2022·2024 Print 는 모두 한 줄로 다시 나눈다. 수정 전 rhwp 는 저장 줄을 따라 250쪽 행 49 가
//! 69.1px(정본·선언 51.9px)였고, 그 17.2px 가 250·251쪽 뒤 행을 모두 밀었다.
//!
//! 한/글 2024 합성(`mydocs/tech/investigations/issue-8105/probes/make_numbered_stale_tail.py`, HWP5 로
//! 변환한 입력): «-»·«^1.» 번호를 그리는 문단은 폭이 맞는 저장 줄도 쓰지 않고 한 줄로 다시 나누고,
//! 번호 없는 문단은 폭이 맞으면 저장 줄을 따른다.
//!
//! 반례: 같은 문서 59쪽의 번호 없는 칸 문단 `…2017.11.13., 제` / `정]` 은 정본도 두 줄이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE_1342000: &str =
    "mydocs/pr/assets/issue7445/education-curriculum-map/1342000_edu_curriculum_map.hwp";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_1342000);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("parse 1342000")
}

struct Run {
    x: f64,
    y: f64,
    right: f64,
    text: String,
}

fn collect_runs(node: &RenderNode, out: &mut Vec<Run>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push(Run {
            x: node.bbox.x,
            y: node.bbox.y,
            right: node.bbox.x + node.bbox.width,
            text: run.text.clone(),
        });
    }
    for child in &node.children {
        collect_runs(child, out);
    }
}

/// 같은 윗변에 놓인 글자 조각을 x 순으로 이어 한 글줄로 만든다. 20px 넘게 떨어진 조각은
/// 다른 칸의 글줄이다. 공백은 지운다(양쪽 정렬 공백 폭은 비교 대상이 아니다).
fn lines_of(node: &RenderNode) -> Vec<String> {
    let mut runs = Vec::new();
    collect_runs(node, &mut runs);
    runs.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut last: Option<(f64, f64)> = None;
    for run in &runs {
        let same_line =
            last.is_some_and(|(y, right)| (run.y - y).abs() < 0.5 && run.x - right < 20.0);
        if !same_line && !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        current.extend(run.text.chars().filter(|c| !c.is_whitespace()));
        last = Some((run.y, run.right));
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// `needle` 글줄을 품은 표 칸 노드 중 가장 안쪽 것.
fn cell_with<'a>(node: &'a RenderNode, needle: &str) -> Option<&'a RenderNode> {
    let inner = node
        .children
        .iter()
        .find_map(|child| cell_with(child, needle));
    if inner.is_some() {
        return inner;
    }
    (matches!(node.node_type, RenderNodeType::TableCell(_))
        && lines_of(node).iter().any(|line| line.contains(needle)))
    .then_some(node)
}

#[test]
fn numbered_cell_paragraph_rejoins_its_stored_word_tail() {
    let core = core();
    let tree = core.build_page_render_tree(249).expect("250쪽 렌더 트리");
    let cell =
        cell_with(&tree.root, "갈등현상탐색").expect("250쪽에 `…갈등 현상 탐색` 칸이 있어야 한다");
    let lines = lines_of(cell);
    assert!(
        lines.iter().any(|line| line.ends_with("갈등현상탐색하기")),
        "번호 문단은 한 줄 `…갈등 현상 탐색하기` (정본 한/글 2020): {lines:?}"
    );
    assert!(
        !lines.iter().any(|line| line == "기"),
        "꼬리 `기` 가 따로 남으면 안 된다: {lines:?}"
    );
    // 행 49 의 선언 높이 3882 HU — 정본 51.9px.
    let declared = 3882.0 / 75.0;
    assert!(
        (cell.bbox.height - declared).abs() <= 0.5,
        "250쪽 행 49 칸: 기대 {declared:.2}px, 실제 {:.2}px",
        cell.bbox.height
    );
}

#[test]
fn unnumbered_cell_paragraph_keeps_its_stored_word_tail() {
    let core = core();
    let tree = core.build_page_render_tree(58).expect("59쪽 렌더 트리");
    let lines = lines_of(&tree.root);
    let split = lines
        .iter()
        .position(|line| line.ends_with("2017.11.13.,제"))
        .unwrap_or_else(|| panic!("59쪽은 `…2017.11.13., 제` 로 끝나는 줄을 둔다: {lines:?}"));
    assert!(
        lines
            .get(split + 1)
            .is_some_and(|line| line.starts_with("정]")),
        "번호 없는 문단은 저장 줄 그대로 `정]` 이 다음 줄 (정본 한/글 2020): {lines:?}"
    );
}
