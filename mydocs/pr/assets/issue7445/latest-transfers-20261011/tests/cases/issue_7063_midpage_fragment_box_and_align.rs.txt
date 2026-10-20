//! [Issue #7063 레인②] 쪽 중간에서 시작하는 조각의 상자·clip·정렬을 가른다.
//!
//! # 무엇이 깨져 있었나
//!
//! `#7095` 는 «마지막이 아닌 조각의 상자는 내용이 아니라 쪽이 정한다» 를 세우면서
//! `row_heights[0] = pinned_height` **한 줄**로 세 가지를 동시에 정했다.
//!
//! ```text
//!   ① 테두리 상자 높이     ← 쪽이 정해야 한다
//!   ② clip(보이는 줄)·칸 안 중첩 표가 쓰는 높이  ← 예산이 자른 그대로여야 한다
//!   ③ 세로 정렬의 기준 높이 ← ① 안에서 ②를 정렬해야 한다
//! ```
//!
//! 그 주석의 전제(*"예산이 같은 상자로 잘랐으므로 보이는 줄은 상자 안에 있다"*)는 **쪽 상단**
//! 조각에서만 성립한다. 쪽 **중간** 조각은 예산이 그 상자를 모르므로 ①을 늘리면 ②가 따라
//! 늘어나, 칸 안 중첩 표가 더 큰 높이를 보고 다시 쪼개진다 — 이미 **다음 쪽에 배치된 행**이
//! 이 쪽에도 그려졌다(`hwpx_sample2.hwp` 8쪽: 9쪽 첫 행 `구  분`·`조회방법` 이 8쪽
//! 1086~1124px 에도 나온다).
//!
//! 그래서 `#6923` 은 쪽 중간 조각을 통째로 제외했고, 그 결과 조각 상자가 내용 끝에서 끊겨
//! 이슈 본문의 레인 ②(쪼개진 `Center` 칸이 `Top` 으로 강제됨, 3.33px)가 남아 있었다.
//!
//! # 독립 기대값 — 한/글 정본
//!
//! `pdf/hwpx_sample2-hwpx-2020.pdf` (Hancom PDF 1.3.0.550 · Hwp 2022 · 29쪽 = rhwp 29쪽).
//! 쪽 척도(1122.5/1121.33)를 걷은 값이다.
//!
//! ```text
//!   19쪽 pi=182 첫 조각        정본 상자 89.91 .. 1081.39 · 칸 내용 상단 95.06
//!     수정 전                       90.00 ..  1075.40 ·            91.90   ← 레인 ② 잔여
//!     수정 후                       90.00 ..  1081.80 ·            95.10   ← 잔차 0.04
//!
//!   HWP 쌍둥이 8쪽 pi=74       정본 상자 226.55 .. 1083.31
//!     수정 전                       226.60 .. 1018.20   (65px 짧다)
//!     수정 후                       226.60 .. 1083.70   (잔차 0.4)
//! ```
//!
//! # 잠그는 것
//!
//! ①(상자가 쪽까지) · ②(줄이 늘지 않는다 = 다음 쪽 내용 중복 없음) · ③(정렬이 상자 기준)
//! 셋을 같이 잠근다. 셋 중 하나만 되돌려도 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn page_root(sample: &str, page_index: u32) -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    core.build_page_render_tree(page_index)
        .expect("render tree")
        .root
}

/// 본문 최상위 표(칸 안 중첩 표 제외)를 `para_index` 로 찾는다.
fn top_level_table<'a>(
    node: &'a RenderNode,
    para_index: usize,
    inside: bool,
) -> Option<&'a RenderNode> {
    let mut inside = inside;
    if let RenderNodeType::Table(t) = &node.node_type {
        if !inside && t.para_index == Some(para_index) {
            return Some(node);
        }
        inside = true;
    }
    node.children
        .iter()
        .find_map(|child| top_level_table(child, para_index, inside))
}

fn text_lines(node: &RenderNode, out: &mut Vec<f64>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push(node.bbox.y);
    }
    for child in &node.children {
        text_lines(child, out);
    }
}

/// ①·③ — 19쪽 첫 조각의 상자는 쪽이 정하고, `Center` 칸 내용은 그 상자 안에서 정렬된다.
#[test]
fn issue_7063_midpage_fragment_box_reaches_the_page_and_centers_its_slice() {
    let root = page_root("samples/hwpx_sample2.hwpx", 18);
    let table = top_level_table(&root, 182, false).expect("19쪽 표 pi=182 — 시험 설정");
    let bottom = table.bbox.y + table.bbox.height;
    assert!(
        (bottom - 1081.39).abs() <= 1.0,
        "① 조각 상자 아래는 한/글 정본(1081.39px) 1px 안이어야 한다 (수정 전 1075.40): {bottom:.2}"
    );

    let mut lines = Vec::new();
    text_lines(table, &mut lines);
    lines.sort_by(f64::total_cmp);
    let first = *lines.first().expect("칸 첫 줄");
    assert!(
        (first - 95.06).abs() <= 0.5,
        "③ Center 칸 내용은 늘어난 상자 안에서 정렬돼 정본(95.06px) 0.5px 안이어야 한다 \
         (수정 전 91.90 — Top 강제): {first:.2}"
    );
}

/// ② — 상자를 늘려도 **보이는 줄은 늘지 않는다.** 늘면 다음 쪽 내용이 이 쪽에 중복된다.
///
/// 수정 전 이 갈래를 열면 8쪽 줄이 72 → 78 로 늘고, 그 6줄이 9쪽 첫 행(`구  분`·`조회방법`)과
/// 같은 내용이었다. 마지막 줄은 1123.90 으로 용지(1122.5) 밖이었다.
#[test]
fn issue_7063_midpage_fragment_box_does_not_pull_next_page_rows() {
    let root = page_root("samples/hwpx_sample2.hwp", 7);
    let table = top_level_table(&root, 74, false).expect("8쪽 표 pi=74 — 시험 설정");
    let bottom = table.bbox.y + table.bbox.height;
    assert!(
        (bottom - 1083.31).abs() <= 1.0,
        "① 조각 상자 아래는 한/글 정본(1083.31px) 1px 안이어야 한다 (수정 전 1018.20): {bottom:.2}"
    );

    let mut lines = Vec::new();
    text_lines(table, &mut lines);
    lines.sort_by(f64::total_cmp);
    let last = *lines.last().expect("칸 마지막 줄");
    assert!(
        last <= 1094.0,
        "② 늘어난 상자가 다음 쪽 행을 끌어오면 안 된다 — 마지막 줄이 쪽번호(1094.7) 위여야 \
         한다 (분리 없이 열면 1123.90): {last:.2}"
    );
}
