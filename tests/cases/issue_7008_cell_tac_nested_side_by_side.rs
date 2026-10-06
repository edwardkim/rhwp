#![cfg(not(target_arch = "wasm32"))]

//! [Issue #7008] 칸 안 **글자처럼 취급되는** 중첩 표 둘을 측정만 세로로 쌓아 세서,
//! 칸이 부풀고 바깥 표가 17.0px 내려와 본문 첫 글줄 위에 괘선이 겹쳤다.
//!
//! `21_언어_기출_편집가능본.hwp` 1쪽 머리 표(s0 p0 c2)의 칸 `r2c1` 은 저장 사다리가
//! **줄 하나**(`vpos 0, lh 3015`)뿐인데 그 줄에 TAC 중첩 표 둘이 들어 있다.
//!
//! ```text
//!   A  14174 x 2449  tac=true  horzOffset=0  outMargin 283/283/283/283  (밴드 3015 = 줄 높이)
//!   B  21384 x 2448  tac=true  horzOffset=0  outMargin 0
//!   칸 선언 47274 x 3768 (630.3 x 50.2px)
//! ```
//!
//! ⭐ **배치는 이미 나란히 놓는다** — 칸 TAC 분기는 `inline_x` 만 전진시키고 문단
//! 커서는 `max(para_y_before_compose + table_h)` 로만 올린다. 측정만
//! `nested_heights.iter().sum()` 으로 2449+2448 = 65.3px 를 요구했다(최댓값이면
//! 32.7px). `#6787` 이 고친 측정·배치 비대칭의 TAC 판이다.
//!
//! 한/글 2022 기준(`pdf/21_언어_기출_편집가능본-2022.pdf` 1쪽): 머리 표 하단 괘선
//! `314.7`, 표 높이 `183.1px`, 본문 오른쪽 단 첫 글줄 상단 `326.1`.

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/21_언어_기출_편집가능본.hwp";

fn page0() -> rhwp::renderer::render_tree::PageRenderTree {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(p).expect("표본 읽기")).expect("문서 로드");
    core.build_page_render_tree(0).expect("1쪽 render tree")
}

fn tables(node: &RenderNode, out: &mut Vec<(f64, f64, f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::Table { .. }) {
        out.push((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
    }
    for child in &node.children {
        tables(child, out);
    }
}

/// 머리 표 = 1쪽에서 가장 넓은 표.
fn head_table(tree: &rhwp::renderer::render_tree::PageRenderTree) -> (f64, f64, f64, f64) {
    let mut all = Vec::new();
    tables(&tree.root, &mut all);
    all.into_iter()
        .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap())
        .expect("머리 표")
}

/// 머리 표 안의 TAC 중첩 표 둘(「성명」·「수험번호」 상자).
fn nested_boxes(tree: &rhwp::renderer::render_tree::PageRenderTree) -> Vec<(f64, f64, f64, f64)> {
    let head = head_table(tree);
    let mut all = Vec::new();
    tables(&tree.root, &mut all);
    let mut inner: Vec<_> = all
        .into_iter()
        .filter(|t| t.2 < head.2 && t.1 >= head.1 && t.1 <= head.1 + head.3)
        .collect();
    inner.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    inner
}

/// 전제 — 두 상자는 세로로 쌓인 적이 없다. 가로로 떨어져 **같은 띠**를 나눠 갖는다.
///
/// 이 시험이 무너지면 아래 두 시험의 "합이 아니라 최댓값" 논거가 통째로 사라진다.
#[test]
fn the_two_tac_boxes_share_one_band() {
    let tree = page0();
    let boxes = nested_boxes(&tree);
    assert_eq!(boxes.len(), 2, "머리 표 안 중첩 표 2개: {boxes:?}");

    let (ax, ay, aw, ah) = boxes[0];
    let (bx, by, _bw, bh) = boxes[1];
    assert!(
        bx >= ax + aw,
        "두 상자는 가로로 떨어져 있다: A x {ax:.1}+{aw:.1}, B x {bx:.1}"
    );
    let overlap = (ay + ah).min(by + bh) - ay.max(by);
    assert!(
        overlap > ah * 0.8,
        "두 상자는 같은 띠를 나눠 가져야 한다 (겹침 {overlap:.1}px / 높이 {ah:.1}px): {boxes:?}"
    );
}

/// 칸 높이는 두 상자의 **합**이 아니라 **최댓값**이라, 머리 표가 선언 높이를 지킨다.
///
/// 수정 전 `200.1px` (한/글 `183.1px` 대비 **+17.0**).
#[test]
fn head_table_keeps_its_declared_height() {
    let tree = page0();
    let (_x, _y, _w, h) = head_table(&tree);
    assert!(
        (h - 183.1).abs() <= 2.0,
        "머리 표 높이가 한/글 183.1px 와 맞아야 한다 — #7008 회귀          \
         (실측 {h:.1}px; 수정 전 200.1px = 중첩 표 2449+2448 합산)"
    );
}

/// 머리 표 하단 괘선이 본문 오른쪽 단 첫 글줄 **위**에 있어야 한다.
///
/// 수정 전 표 하단 `331.8` vs 첫 글줄 `329.9` — 1.9px 덮었다(그려진 괘선은 `331.0`).
/// 한/글은 괘선 `314.7` vs 글자 상자 `326.1` 로 11.4px 띄운다.
#[test]
fn head_table_does_not_overlap_the_first_body_line() {
    let tree = page0();
    let (_x, y, _w, h) = head_table(&tree);
    let table_bottom = y + h;

    // 오른쪽 단의 첫 **본문** 글줄 — 표 안 글줄(「홀수형」 등)은 세지 않도록
    // 표 노드에서 가지를 끊는다.
    let mut first_line = f64::INFINITY;
    fn scan(node: &RenderNode, x_min: f64, out: &mut f64) {
        if matches!(node.node_type, RenderNodeType::Table { .. }) {
            return;
        }
        if matches!(node.node_type, RenderNodeType::TextLine { .. })
            && node.bbox.x > x_min
            && node.bbox.height > 0.0
        {
            *out = out.min(node.bbox.y);
        }
        for child in &node.children {
            scan(child, x_min, out);
        }
    }
    scan(&tree.root, 560.0, &mut first_line);
    assert!(
        first_line.is_finite(),
        "오른쪽 단 첫 글줄을 찾아야 한다 (표 하단 {table_bottom:.1})"
    );
    assert!(
        table_bottom <= first_line,
        "머리 표 하단 괘선이 본문 첫 글줄을 덮는다 — #7008 회귀          \
         (표 하단 {table_bottom:.1} vs 글줄 {first_line:.1}; 수정 전 331.8 vs 329.9)"
    );
}
