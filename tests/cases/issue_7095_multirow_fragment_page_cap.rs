//! [Issue #7095 확장] 조각 상자의 **쪽 상한**을 다행 표에도 적용한다.
//!
//! # 무엇이 깨져 있었나
//!
//! `#7095` 는 «비끝 조각 상자는 `본문아래 − outMargin.bottom − 100HU` 에서 끝난다» 를 세우면서
//! 술어를 `row_count == 1 && col_count == 1` 로 좁혔다. 근거 문서가 1×1 저장본뿐이었기
//! 때문이다. 그래서 **다행 조각**은 상한을 못 받아 내용이 끝나는 자리까지 그려졌고, 그 상자가
//! 본문 아래로 나갔다.
//!
//! # 독립 기대값 — 한/글 정본
//!
//! 정본 세 문서에서 «여러 쪽이 **같은 값**으로 끝나는» 조각만 골라 상한을 풀었다. 내용이 먼저
//! 끝난 조각은 쪽마다 값이 흩어지고, 상한에 걸린 조각은 한 값으로 모인다. 양쪽 모두 표
//! 괘선(`m`/`l` 조각 병합)으로 쟀다 — 칠 영역으로 집으면 중첩 셀을 틀로 오인한다.
//!
//! ```text
//!   문서                          행×열   omB(HU)  본문아래   정본아래   본문−정본  omB(px)  잔차
//!   issue7336 p2·p5               16×1      283   1028.01   1022.99     5.02     3.77   1.25
//!   rowbreak-problem-pages p3·p4  25×7      141   1028.00   1026.03     1.97     1.88   0.09
//!   issue1853 … 6쪽                2×2      141   1020.48   1018.51     1.97     1.88   0.09
//! ```
//!
//! 셋 다 1×1 이 아니다 — `#7095` 의 계약에는 형상 조건이 없다.
//!
//! `pdf/issue7336/stored_frame_page_larger_rowbreak-2020.pdf` (7쪽 = rhwp 7쪽) 네 쪽이
//! **`min(내용 끝, 쪽 상한)`** 하나로 전부 설명된다.
//!
//! ```text
//!   쪽   정본 아래끝   수정 전     수정 후    판정
//!   p2    1022.99     1037.30    1024.00    상한 적용 (수정 전 본문 1028.01 을 9.3 초과)
//!   p3    1020.27     1021.40    1021.40    내용이 먼저 끝남 — 접지 않는다
//!   p4    1007.63     1008.80    1008.80    내용이 먼저 끝남 — 접지 않는다
//!   p5    1022.99     1029.50    1024.10    상한 적용
//! ```
//!
//! # 잠그는 것
//!
//! ① 상한을 넘는 다행 조각은 상한까지 접힌다. ② 상한 안에서 끝나는 조각은 **건드리지 않는다**
//! (상한을 무조건 적용하면 짧은 조각이 늘어난다). 둘을 함께 잠근다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7336/stored_frame_page_larger_rowbreak.hwpx";

/// 본문 최상위 표(칸 안 중첩 표 제외)의 아래끝.
fn top_level_table_bottom(sample: &str, page_index: u32, para_index: usize) -> f64 {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let root = core
        .build_page_render_tree(page_index)
        .expect("render tree")
        .root;

    fn find<'a>(node: &'a RenderNode, para_index: usize, inside: bool) -> Option<&'a RenderNode> {
        let mut inside = inside;
        if let RenderNodeType::Table(t) = &node.node_type {
            if !inside && t.para_index == Some(para_index) {
                return Some(node);
            }
            inside = true;
        }
        node.children
            .iter()
            .find_map(|child| find(child, para_index, inside))
    }
    let table = find(&root, para_index, false).expect("대상 표 — 시험 설정");
    table.bbox.y + table.bbox.height
}

/// ① 상한을 넘는 다행 조각은 쪽 상한까지 접힌다.
#[test]
fn a_multirow_fragment_past_the_page_cap_is_folded_to_it() {
    for (page_index, before) in [(1u32, 1037.30_f64), (4, 1029.50)] {
        let bottom = top_level_table_bottom(SAMPLE, page_index, 3);
        assert!(
            (bottom - 1022.99).abs() <= 2.0,
            "① {}쪽 조각 상자 아래는 한/글 정본(1022.99px) 2px 안이어야 한다 \
             (수정 전 {before:.2}): {bottom:.2}",
            page_index + 1
        );
    }
}

/// ② 상한 안에서 끝나는 조각은 건드리지 않는다.
///
/// 상한을 **무조건** 적용하면 내용이 먼저 끝난 짧은 조각이 상한까지 늘어난다. 이 두 쪽은
/// 정본이 각각 1020.27 · 1007.63 에서 끝나고 rhwp 도 이미 0.05px 안에서 맞는다.
#[test]
fn a_fragment_that_ends_before_the_cap_is_left_alone() {
    for (page_index, oracle) in [(2u32, 1020.27_f64), (3, 1007.63)] {
        let bottom = top_level_table_bottom(SAMPLE, page_index, 3);
        assert!(
            (bottom - oracle).abs() <= 2.0,
            "② {}쪽 조각은 상한(1022.99) 전에 끝나므로 정본({oracle:.2}px) 자리를 지켜야 한다: \
             {bottom:.2}",
            page_index + 1
        );
    }
}
