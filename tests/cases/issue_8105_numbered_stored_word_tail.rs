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

/// 속성 배열 순서를 바로잡은 수동 입력을 한/글 2024로 직접 Print한 계약이다.
/// appVersion 문자열은 실제 2014 저장 출처로 취급하지 않는다. 양쪽 raw 100%와
/// 독립 PDF 좌표·직접 판독 근거는 mydocs/pr/assets/issue_8105/ordered_stored_tail/README.md에 있다.
#[test]
fn ordered_probe_preserves_exact_width_cache_and_reflows_different_widths() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue8105/numbered_stale_tail_ordered.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("커밋된 대조 입력"))
        .expect("대조 입력 파싱");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 1, "독립 Print 전체 쪽수");
    let tree = core.build_page_render_tree(0).expect("대조군 한 쪽");
    let mut nodes = Vec::new();
    fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
        nodes.push(node);
        for child in &node.children {
            walk(child, nodes);
        }
    }
    walk(&tree.root, &mut nodes);
    let tables: Vec<_> = nodes
        .iter()
        .copied()
        .filter(|node| matches!(&node.node_type, RenderNodeType::Table(_)))
        .collect();
    assert_eq!(tables.len(), 15, "모든 번호·폭 반례의 표 누락·중복");
    // 독립 Print의 순서, 줄 분할, 세로 괘선 중심(pt → 96dpi px).
    // P2/L2/P2f는 정확히 맞는 저장 폭을 유지하고, P2w/L2w/P2W는 다시 나눈다.
    let cases = [
        ("D2", 1, 178.525309, 195.625326),
        ("H2", 1, 224.554688, 241.496012),
        ("H1", 1, 270.423991, 287.525309),
        ("P2", 2, 316.293294, 354.652018),
        ("L2", 2, 383.579997, 421.938639),
        ("P0", 1, 450.866659, 467.807983),
        ("D2f", 1, 496.736003, 513.837321),
        ("H2f", 1, 542.606649, 559.708008),
        ("P2f", 2, 588.635986, 626.994670),
        ("H2a", 1, 655.762655, 672.864014),
        ("H2b", 1, 701.791992, 718.733317),
        ("H2w", 1, 747.662679, 764.763997),
        ("P2w", 1, 793.531982, 810.633301),
        ("L2w", 1, 839.561361, 856.502686),
        ("P2W", 1, 885.431966, 902.531982),
    ];
    let mut previous_bottom = f64::NEG_INFINITY;
    for (index, (label, expected_lines, top, bottom)) in cases.into_iter().enumerate() {
        let owners: Vec<_> = tables
            .iter()
            .copied()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::Table(table)
                    if table.para_index == Some(2 * (index + 1)))
            })
            .collect();
        assert_eq!(owners.len(), 1, "{label}: 원본 문단에 속한 유일한 표");
        let table = owners[0];
        let mut descendants = Vec::new();
        walk(table, &mut descendants);
        let lines: Vec<String> = descendants
            .into_iter()
            .filter_map(|node| {
                if !matches!(node.node_type, RenderNodeType::TextLine(_)) {
                    return None;
                }
                let mut runs = Vec::new();
                collect_runs(node, &mut runs);
                let line: String = runs
                    .iter()
                    .flat_map(|run| run.text.chars())
                    .filter(|ch| !ch.is_whitespace())
                    .collect();
                (!line.is_empty()).then_some(line)
            })
            .collect();
        assert_eq!(
            lines.len(),
            expected_lines,
            "{label}: 독립 Print의 글줄 {lines:?}"
        );
        let joined = lines.join("");
        let prefix = match label {
            "D2" => "1.",
            "D2f" => "2.",
            "H2" | "H1" | "L2" | "H2f" | "H2a" | "H2b" | "H2w" | "L2w" => "-",
            _ => "",
        };
        assert_eq!(
            joined,
            format!("{prefix}여러가지차별현상탐색하기"),
            "{label}: 자동 번호·원문 순서·누락·중복"
        );
        assert!(
            table.bbox.y >= previous_bottom,
            "{label}: 앞 표와 겹치지 않음"
        );
        previous_bottom = table.bbox.y + table.bbox.height;
        for (actual, expected) in [
            (table.bbox.x, 117.217336),
            (table.bbox.x + table.bbox.width, 517.001343),
            (table.bbox.y, top),
            (table.bbox.y + table.bbox.height, bottom),
        ] {
            assert!(
                (actual - expected).abs() <= 2.0,
                "{label}: 독립 PDF {expected:.6}px, 실제 {actual:.6}px, 허용 ±2px"
            );
        }
    }
}
