//! [Issue #6756] `RowBreak` 표 조각이 컷 지점을 지나친 행까지 그려, 그 행이 다음 쪽에
//! **중복**으로 나오고 본문 하한·용지를 넘었다.
//!
//! `start_cut`/`end_cut` 부기는 `advance_row_cut` 이 **컷 행의 `row_span==1` 셀만**
//! 담는다(= `single_row_cut_index` 순서). 그런데 `cell_cut_window` 는 블록 분할이면
//! 무조건 `block_cut_index` 로 찾아, 컷 행이 **큰 rowspan 블록 안에** 있으면 두 서수
//! 공간이 어긋난다.
//!
//! `17253153` 실측 — 셀 `(11,0)` 이 `row_span=17` 이라 블록이 `(11, 28)` 로 잡힌다:
//!
//! ```text
//!   컷 행 14 의 셀 (14,1)   블록 서수 7   end_cut = [2, 2]  →  get(7) == None
//!                           → eu = usize::MAX  → **컷이 사라져 행 전체가 그려진다**
//! ```
//!
//! 그 행을 다음 조각이 `start_cut` 대로 다시 그려 중복이 된다.
//!
//! ```text
//!            전체 글자   용지 밖   어절 멀티셋
//!   수정 전     3637      10.6px    rhwp 에만 `2.`·`3.`·`4.` 등이 더 있다
//!   수정 후     3559       0.0      **양쪽 모두 0** (한/글과 정확히 같다)
//!   한/글       3559
//! ```
//!
//! `#1748` 이 남긴 "컷 부기는 컷 행의 `row_span==1` 셀만 담는다"는 관찰 그대로다 —
//! 거기서는 **걸친 rowspan 셀**만 높이 기반 경로로 구제했고, 블록 안의 보통 셀은
//! 그대로 남아 있었다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 공개 회귀 입력이 없으면 검사 없이 통과하지 않고 즉시 실패한다.
fn sample() -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue6756/17253153-traffic-safety-designated-routes.hwp"),
    )
    .expect("#6756 정식 회귀 sample 읽기")
}

fn page_text(core: &DocumentCore, page: u32) -> String {
    let tree = core
        .build_page_render_tree(page)
        .expect("#6756 페이지 렌더 트리 생성");
    fn collect(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            collect(child, out);
        }
    }
    let mut out = String::new();
    collect(&tree.root, &mut out);
    out.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 조각 경계의 행이 두 쪽에 **중복**으로 나오면 안 된다.
///
/// 2쪽 끝 두 줄(`3.`·`4.` 좌표)이 3쪽 머리에 다시 나오던 결함.
#[test]
fn fragment_boundary_row_is_not_painted_twice() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(core.page_count(), 5, "한/글 2024 와 같은 5쪽이어야 한다");

    // 문서 전체 글자 수 — 한/글 2024 실측 3559.
    let total: usize = (0..core.page_count())
        .map(|p| page_text(&core, p).chars().count())
        .sum();

    assert!(
        (3520..=3600).contains(&total),
        "조각 경계 행이 중복되면 글자가 늘어난다 — #6756 회귀 \
         (실측 {total}자; 수정 전 3637, 한/글 3559)"
    );
}

/// 조각이 용지 밖으로 나가면 안 된다.
#[test]
fn fragment_stays_on_the_paper() {
    let bytes = sample();
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(1).expect("2쪽 render tree");
    let paper_bottom = tree.root.bbox.y + tree.root.bbox.height;

    fn lowest(node: &RenderNode) -> f64 {
        let own = if matches!(node.node_type, RenderNodeType::TextRun(_)) {
            node.bbox.y + node.bbox.height
        } else {
            f64::MIN
        };
        node.children.iter().map(lowest).fold(own, f64::max)
    }

    let bottom = lowest(&tree.root);
    assert!(
        bottom <= paper_bottom + 0.5,
        "2쪽 글자가 용지 밖으로 나가면 안 된다 — #6756 회귀 \
         (최하단 {bottom:.1} > 용지 {paper_bottom:.1}; 수정 전 10.6px 초과)"
    );
}
