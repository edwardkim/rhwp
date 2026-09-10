//! [Issue #4068] **안 잘린 중첩 칸이 선언된 세로 정렬을 잃는다.**
//!
//! `table_layout.rs` 의 `effective_valign` 은 네 가지 잘림 조건에서 `Top` 으로 수렴한다.
//! 그중 `cell_clipped_by_parent_viewport` 는 호출자가 넘긴 `col_area` 로 잘림을
//! 판정하는데, 그 값은 직전 조각까지 품은 채 올 수 있다(그 자리 주석이 이미 적어 둔
//! 사실이다). 그래서 **페이지 안에 온전히 들어간** 칸까지 "잘렸다"고 오판했다.
//!
//! ```text
//!   hwpx_sample2 19쪽 · 중첩 표 1행×2열 · 두 칸 모두 선언 valign=Center
//!     셀 961.80..1063.40 · page bbox 0.00..1122.50   → 실제로는 안 잘린다
//!     그런데 parentvp=true 로 Top 강제
//! ```
//!
//! 결과는 칸 내용이 정렬 몫만큼 위로 붙는 것이다. 한/글 정본(engine 2020) PDF 와
//! 베이스라인끼리 직접 대면 이렇게 나온다 — 같은 쪽 첫 줄이 Δ+0.03px 로 떨어지므로
//! 이 계기 자체는 검증돼 있다.
//!
//! ```text
//!   글자 칸 베이스라인   수정 전 Δ +9.99px  →  수정 후 Δ +8.11px
//! ```
//!
//! 고친 것은 술어에 **실제 클립(page bbox)** 한 항을 더한 것뿐이다. 안 잘린 칸은
//! 잘림 수렴의 대상이 아니다.
//!
//! ⚠ 이 수정은 `#4068` 의 그림 dy 를 **움직이지 않는다.** 그림 칸은 내용 높이가
//! 안높이보다 커서 정렬 몫이 0 이다(아래 `the_cell_without_alignment_slack_stays_put`
//! 가 그것을 잠근다). 이 시험이 잠그는 것은 "칸 정렬 오판" 하나다.
//!
//! 아래 셋이 **양쪽을 잠근다** — 정렬을 못 받으면 ①이, 여유 없는 칸까지 밀면 ②가,
//! 진짜 잘림 보호가 무너지면 ③이 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 중첩 표를 담은 정식 회귀 입력 — 그림 칸과 글자 칸이 한 행에 나란히 있다.
const NESTED_SAMPLE: &str = "samples/hwpx_sample2.hwp";
/// `#2007` 의 **진짜** 쪽-잘림 보호 문서. 이 수정에 영향받지 않아야 한다.
const CLIP_SAMPLE: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";

/// `walk` 은 자기 자신을 세므로 최상위 표의 칸이 1, 그 안의 중첩 표 칸이 2 다.
/// 중첩 단계는 문서마다 다르다 — `hwpx_sample2` 는 2, `42065` 는 3 이다.
const NESTED_DEPTH_SAMPLE2: usize = 2;
const NESTED_DEPTH_42065: usize = 3;

fn sample(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// 한 칸이 그린 첫 글줄의 상단과 그 칸의 상단.
#[derive(Debug, Clone, Copy)]
struct CellContent {
    cell_x: f64,
    cell_y: f64,
    cell_h: f64,
    first_line_y: f64,
}

impl CellContent {
    /// 칸 상단부터 첫 글줄까지 — 여백 + 세로 정렬 몫.
    fn offset(&self) -> f64 {
        self.first_line_y - self.cell_y
    }
}

/// `depth` 단계 이상 중첩된 칸들의 (칸 상자, 첫 글줄) 을 모은다.
fn nested_cell_contents(node: &RenderNode, min_depth: usize) -> Vec<CellContent> {
    fn first_line_y(node: &RenderNode) -> Option<f64> {
        let mut best: Option<f64> = None;
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            best = Some(node.bbox.y);
        }
        for child in &node.children {
            if let Some(y) = first_line_y(child) {
                best = Some(best.map_or(y, |b: f64| b.min(y)));
            }
        }
        best
    }

    fn walk(node: &RenderNode, depth: usize, min_depth: usize, out: &mut Vec<CellContent>) {
        let is_cell = matches!(node.node_type, RenderNodeType::TableCell(_));
        let depth = depth + usize::from(is_cell);
        if is_cell && depth >= min_depth {
            if let Some(y) = first_line_y(node) {
                out.push(CellContent {
                    cell_x: node.bbox.x,
                    cell_y: node.bbox.y,
                    cell_h: node.bbox.height,
                    first_line_y: y,
                });
            }
        }
        for child in &node.children {
            walk(child, depth, min_depth, out);
        }
    }

    let mut out = Vec::new();
    walk(node, 0, min_depth, &mut out);
    out.sort_by(|a, b| {
        (a.cell_y, a.cell_x)
            .partial_cmp(&(b.cell_y, b.cell_x))
            .unwrap()
    });
    out.dedup_by(|a, b| (a.cell_y, a.cell_x) == (b.cell_y, b.cell_x));
    out
}

fn page_tree(rel: &str, page_num: u32) -> (RenderNode, f64) {
    let bytes = std::fs::read(sample(rel)).expect("정식 회귀 sample 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core
        .build_page_render_tree(page_num)
        .expect("페이지 render tree");
    let page_bottom = tree.root.bbox.y + tree.root.bbox.height;
    (tree.root, page_bottom)
}

/// `hwpx_sample2` 19쪽(0-based 18)의 중첩 표 한 행 — 왼쪽 그림 칸, 오른쪽 글자 칸.
fn nested_row() -> (CellContent, CellContent, f64) {
    let (root, page_bottom) = page_tree(NESTED_SAMPLE, 18);
    let cells: Vec<CellContent> = nested_cell_contents(&root, NESTED_DEPTH_SAMPLE2)
        .into_iter()
        .filter(|c| (955.0..975.0).contains(&c.cell_y))
        .collect();
    assert_eq!(
        cells.len(),
        2,
        "이 시험의 전제는 중첩 표 한 행의 두 칸이다 — 형상이 바뀌면 전제가 깨진다: {cells:?}"
    );
    (cells[0], cells[1], page_bottom)
}

/// ① 안 잘린 칸은 선언된 `Center` 를 받는다.
///
/// 수정 전에는 두 칸이 **같은** 오프셋(0.90px = 여백만)이었다. 오른쪽 글자 칸은
/// 내용(96.00px)이 안높이(99.76px)보다 짧아 정렬 몫 1.90px 을 받아야 한다.
#[test]
fn an_unclipped_nested_cell_keeps_its_declared_center_alignment() {
    let (picture_cell, text_cell, page_bottom) = nested_row();

    // 전제: 두 칸 모두 페이지 안에 **온전히** 들어간다 — 이 수정이 보는 바로 그 조건.
    for cell in [picture_cell, text_cell] {
        assert!(
            cell.cell_y >= -0.5 && cell.cell_y + cell.cell_h <= page_bottom + 0.5,
            "전제 붕괴: 칸이 페이지 밖으로 나갔다 {cell:?} (page_bottom={page_bottom})"
        );
    }

    let slack = text_cell.offset() - picture_cell.offset();
    assert!(
        (1.0..=3.0).contains(&slack),
        "여유 있는 Center 칸이 정렬 몫을 받아야 한다 — 수정 전에는 0.00 이었다. \
         실측 {slack:.2}px (그림칸 {:.2} · 글자칸 {:.2})",
        picture_cell.offset(),
        text_cell.offset()
    );
}

/// ② 정렬 여유가 없는 칸은 **움직이지 않는다**.
///
/// 왼쪽 그림 칸은 내용(103.95px)이 안높이(99.76px)보다 커서 정렬 몫이 0 이다.
/// 이 수정이 모든 중첩 칸을 무조건 밀어내는 게 아님을 잠근다.
#[test]
fn the_cell_without_alignment_slack_stays_put() {
    let (picture_cell, _text_cell, _) = nested_row();
    assert!(
        picture_cell.offset() < 1.5,
        "여유 없는 칸은 여백만큼만 내려가야 한다 — 실측 {:.2}px",
        picture_cell.offset()
    );
}

/// ③ **반례** — `#2007` 의 쪽-잘림 보호 문서는 이 수정에 흔들리지 않는다.
///
/// `42065` 11쪽(0-based 10)의 중첩 칸 두 개는 수정 전후 **같은 자리**다(전/후 SVG
/// 비교에서 차이는 1e-13px 부동소수 잡음뿐이었다). 이 수정이 잘림 보호까지 걷어냈다면
/// 여기 오프셋이 움직인다.
#[test]
fn the_page_clip_protected_document_is_unaffected() {
    let (root, _) = page_tree(CLIP_SAMPLE, 10);
    let cells: Vec<CellContent> = nested_cell_contents(&root, NESTED_DEPTH_42065)
        .into_iter()
        .filter(|c| c.cell_h > 100.0)
        .collect();
    assert_eq!(
        cells.len(),
        2,
        "이 시험의 전제는 큰 중첩 칸 두 개다: {cells:?}"
    );

    let offsets: Vec<f64> = cells.iter().map(|c| c.offset()).collect();
    for (got, want) in offsets.iter().zip([1.88_f64, 3.76_f64]) {
        assert!(
            (got - want).abs() <= 0.5,
            "#2007 보호 문서의 칸 내용이 움직였다 — 기대 {want:.2}px, 실측 {got:.2}px \
             (전체 {offsets:?})"
        );
    }
}
