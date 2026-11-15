//! 고정형: 완료된 37쪽의 한컴 Print 기준선과 한 줄 병합 라벨 정렬을 ±2 CSSpx로 보호한다.
//! 관계형: 같은 수용 컷의 한 줄 라벨은 행 번호와 기준선을 공유하고 다음 문단을 중복하지 않는다.
//! 주석 갱신: 2026-11-15T13:19:32+09:00.
//! 보정 전 50쪽 Native/fresh WASM 최저 98.75%, 대상 37쪽도 각각 98.75%다.
//! 근거: issues7727_7732_20261114/final-1123f322의 두 전쪽 TSV와 직접 판독한 37쪽 PNG.
//! 독립 한컴 2020 Print의 `31`과 `정책실현` 기준선은 모두 742.9480590820312pt다.
//! 설치 글꼴은 전제하지 않는다. 실제 셀/문단 소유로 대상을 선택한다.

use rhwp::{
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

fn source() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs/pr/assets/issue7445/issue1937_rowbreak_footnote_overpagination.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    core
}

fn flatten<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        flatten(child, out);
    }
}

fn owned_cell(root: &RenderNode, col: u16) -> &RenderNode {
    let mut nodes = Vec::new();
    flatten(root, &mut nodes);
    let cells: Vec<_> = nodes.into_iter().filter(|node| {
        matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 45 && cell.col == col)
    }).collect();
    assert_eq!(cells.len(), 1, "원본 행의 물리 셀은 한 번만 배치한다");
    cells[0]
}

fn owned_text<'a>(cell: &'a RenderNode, text: &str) -> (&'a RenderNode, f64) {
    let mut nodes = Vec::new();
    flatten(cell, &mut nodes);
    let runs: Vec<_> = nodes
        .into_iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextRun(run) if run.text == text => {
                Some((node, node.bbox.y + run.baseline))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        runs.len(),
        1,
        "가시 조각의 원문은 누락·중복 없이 한 번만 그린다"
    );
    runs[0]
}

/// 고정형: 독립 Print 기준선 ±2 CSSpx.
#[test]
fn first_saved_span_fragment_keeps_the_independent_print_baseline() {
    let core = source();
    let tree = core.build_page_render_tree(36).unwrap();
    let policy = owned_cell(&tree.root, 2);
    let (_, baseline) = owned_text(policy, "정책실현");
    let expected = 742.9480590820312 * 96.0 / 72.0;
    assert!(
        (baseline - expected).abs() <= 2.0,
        "한컴 Print가 확정한 첫 조각 기준선: actual={baseline}, expected={expected}"
    );
}

/// 관계형: 같은 물리 행의 한 줄 Center 라벨과 번호의 기준선·문단 소유.
#[test]
fn split_label_uses_the_same_center_as_the_complete_peer_in_its_physical_row() {
    let core = source();
    let tree = core.build_page_render_tree(36).unwrap();
    let label = owned_cell(&tree.root, 2);
    let number = owned_cell(&tree.root, 0);
    let (_, label_baseline) = owned_text(label, "정책실현");
    let (_, number_baseline) = owned_text(number, "31");
    assert!((label.bbox.y - number.bbox.y).abs() < 0.1);
    assert!((label.bbox.height - number.bbox.height).abs() < 0.1);
    assert!((label_baseline - number_baseline).abs() <= 2.0,
        "같은 한 줄·가운데 정렬인데 다음 쪽 꼬리 유무로 원점이 달라지면 안 된다: {label_baseline} / {number_baseline}");
    let mut nodes = Vec::new();
    flatten(label, &mut nodes);
    assert!(
        !nodes.iter().any(|node| matches!(&node.node_type,
        RenderNodeType::TextRun(run) if run.text.contains("재정조달"))),
        "다음 쪽 문단을 앞 조각에 다시 그리지 않는다"
    );
}
