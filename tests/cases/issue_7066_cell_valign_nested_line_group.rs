#![cfg(not(target_arch = "wasm32"))]

//! [Issue #7066] 칸 안 중첩 표가 있으면 칸 **세로 정렬**이 무효가 되어 내용이 상단에 붙던
//! 결함의 가드.
//!
//! ## 계약
//!
//! 세로 정렬이 쓰는 콘텐츠 높이(`calc_nested_controls_bottom_height`)는 한 줄에 나란히
//! 놓인 중첩 표를 **합이 아니라 최댓값**으로 세야 한다. 합하면 콘텐츠가 칸보다 크다고
//! 판정되어 정렬 여유가 `0` 으로 깎이고 `Center`·`Bottom` 이 상단정렬로 무너진다.
//!
//! `#7008` 이 측정(`height_measurer::cell_nested_controls_bottom`)의 같은 합산을 줄 단위로
//! 고쳤다. 이 판은 배치 쪽 형제가 같은 `float_placement::nested_table_groups` 를 쓰게 한다.
//!
//! ## 실측 — 결재 서식 3문서, 한/글 2022 정본
//!
//! ```text
//!                            수정 전    수정 후    한/글    정본
//!   issue2470  문서번호표 상단   157.60    179.10   177.73   pdf/issue2470/36382471_masked-2022.pdf
//!              결재표 하단      272.80    294.30   292.80   (Hwp 2022 12.0.0.4547)
//!   issue2083  문서번호표 상단   208.40    242.20   240.70   pdf/issue2083_hide_fill_page-hwpx-2020.pdf
//!              결재표 하단      361.10    395.00   393.17   (Hwp 2022 0.0.0.0)
//!   21_언어    중첩 상자 상단    254.70    262.90   266.60   pdf/21_언어_기출_편집가능본-2022.pdf
//!                                                           (Hwp 2022 12.0.0.4426)
//! ```
//!
//! 글자로 재면 결재 블록 전체가 균일하게 움직인다 — `issue2470` 은 `Δy +20.07 → −1.41`,
//! `issue2083` 은 `+32.32 → −1.54` (낱말 17·20개, `Δx` 는 둘 다 `0.8px` 안).
//!
//! ⚠ 남는 `1.4~1.8px` 는 기준선 경로의 균일 오프셋이라 이 이슈 밖이다(`#7058` 에서
//! 독립적으로 같은 크기가 관측됐다). `21_언어` 의 `3.7px` 은 상자 외곽선만의 몫이다 —
//! 그 안의 글자는 한/글과 `Δy −0.19` 로 맞는다. 둘 다 이 판에서 재지 않았다.

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{PageRenderTree, RenderNode, RenderNodeType};

/// 정본과의 남은 차 `1.4~1.8px` 를 담되 수정 전(`20~32px`)은 분명히 걸러내는 폭.
const TOL: f64 = 2.5;

fn page0(sample: &str) -> PageRenderTree {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("#7066 공개 fixture 읽기 {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes)
        .expect("문서 로드")
        .build_page_render_tree(0)
        .expect("1쪽 render tree")
}

fn tables(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::Table { .. }) {
        out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
    }
    for child in &node.children {
        tables(child, out);
    }
}

/// 너비로 표를 집는다 — 각 쪽에서 그 폭을 가진 표는 하나뿐이다.
fn table_by_width(tree: &PageRenderTree, want: f64) -> (f64, f64, f64, f64) {
    let mut all = Vec::new();
    tables(&tree.root, &mut all);
    let mut hit: Vec<_> = all
        .into_iter()
        .filter(|&(_, _, w, _)| (w - want).abs() < 1.0)
        .collect();
    assert_eq!(hit.len(), 1, "폭 {want} 인 표가 정확히 하나여야 한다");
    hit.pop().expect("대상 표")
}

/// `valign=Center` 칸 — 결재 블록이 칸 가운데로 내려와야 한다.
#[test]
fn center_aligned_cell_recovers_its_slack_in_36382471() {
    let tree = page0("samples/issue2470/36382471_masked.hwpx");
    let (_, doc_no_y, _, _) = table_by_width(&tree, 252.3);
    let (_, approval_y, _, approval_h) = table_by_width(&tree, 358.2);

    assert!(
        (doc_no_y - 177.73).abs() < TOL,
        "문서번호 표 상단이 {doc_no_y} — 한/글 177.73, 수정 전 157.60"
    );
    let approval_bottom = approval_y + approval_h;
    assert!(
        (approval_bottom - 292.80).abs() < TOL,
        "결재 표 하단이 {approval_bottom} — 한/글 292.80, 수정 전 272.80"
    );
}

/// 같은 결함의 다른 서식 — 여기서는 `32px` 어긋났다.
#[test]
fn center_aligned_cell_recovers_its_slack_in_issue2083() {
    let tree = page0("samples/issue2083_hide_fill_page.hwpx");
    let (_, doc_no_y, _, _) = table_by_width(&tree, 237.2);
    let (_, approval_y, _, approval_h) = table_by_width(&tree, 385.9);

    assert!(
        (doc_no_y - 240.70).abs() < TOL,
        "문서번호 표 상단이 {doc_no_y} — 한/글 240.70, 수정 전 208.40"
    );
    let approval_bottom = approval_y + approval_h;
    assert!(
        (approval_bottom - 393.17).abs() < TOL,
        "결재 표 하단이 {approval_bottom} — 한/글 393.17, 수정 전 361.10"
    );
}

/// `valign=Bottom` 칸 — `#7008` 이 세운 "두 상자는 같은 줄" 계약도 함께 지켜야 한다.
#[test]
fn bottom_aligned_cell_keeps_the_two_boxes_on_one_line_in_21_language() {
    let tree = page0("samples/21_언어_기출_편집가능본.hwp");
    let (_, name_y, _, _) = table_by_width(&tree, 189.0);
    let (_, number_y, _, _) = table_by_width(&tree, 285.1);

    assert!(
        (name_y - number_y).abs() < 0.5,
        "두 상자가 같은 줄이 아니다 — {name_y} vs {number_y}"
    );
    // 상자 외곽선에는 별도 축의 3.7px 이 남아 있다(본문 주석 참조) — 수정 전 11.9px 과는
    // 분명히 갈린다.
    assert!(
        (name_y - 266.60).abs() < 4.5,
        "중첩 상자 상단이 {name_y} — 한/글 266.60, 수정 전 254.70"
    );
}
