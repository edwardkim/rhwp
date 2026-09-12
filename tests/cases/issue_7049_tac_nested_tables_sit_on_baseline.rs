//! [Issue #7049] 한 글줄의 **글자처럼 취급 중첩 표**들이 상단을 맞춰 놓인다 —
//! 한/글은 줄의 기준선에 앉혀 하단을 높이차의 `0.15` 배로 벌린다.
//!
//! 결재 서식의 「문서번호/시행일자」 상자와 「결재란」 상자처럼 한 줄에 높이가 다른
//! 표 둘이 들어가면 두 상자가 통째로 어긋났다.
//!
//! ```text
//!   36384689 1쪽
//!     rhwp   짧은 표 y 102.00 .. 207.90   큰 표 y 102.00 .. 270.70   ← 상단이 붙는다
//!     한/글  짧은 표 y 155.03 .. 260.83   큰 표 y 101.81 .. 270.26
//! ```
//!
//! 걸림돌이 둘이었다.
//!
//! 1. `stored_lh_covers_om` 이 `raw_lh >= table_h + om` 이라, 밴드가 줄에 **들어가기만**
//!    하면 발동했다. 주석이 규정하는 "표 전용 줄" 은 `lh == h + om` 이다. 한 줄에 표가
//!    둘이면 **둘 다** 이 분기로 빠져 전부 `y + om_top` 이 됐다.
//! 2. else 분기가 상자 하단을 `기준선 + om_bottom` 에 뒀다. 한/글은
//!    `기준선 + 0.15 × 높이` 라, 여백이 같은 두 표의 하단 간격이 0 이 됐다.
//!
//! `0.15` 는 새 상수가 아니다 — 저장 `baseline_distance / line_height` 가 세 표본 모두
//! 정확히 `0.8500` 이고, `composer::line_breaking` 도 글꼴 기준 baseline 을
//! `line_height * 0.85` 로 복원한다. 정본에서 같은 `C` 로 두 표가 풀린다.
//!
//! ```text
//!   큰 표   270.26 = C + 0.15 × 168.7  → C = 244.96
//!   짧은 표 260.83 = C + 0.15 × 105.9  → C = 244.93
//! ```
//!
//! 정답지: `pdf/hwpx/opengov/36384689_…-hwpx-2020.pdf` (한/글 2020).
//!
//! ⚠ 잔차 `−1.2px` 는 남는다. 큰 표가 아직 `stored_lh_covers_om` 분기를 타기 때문인데,
//! 둘 다 기준선 분기로 태우면 `#3386` 이 고친 156678235 p5 를 되돌린다. 이 핀은
//! **결함 폭이 53.4px → 1.2px 로 닫혔음**을 잠그고 잔차는 잠그지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str =
    "samples/hwpx/opengov/36384689_결재문서본문_화재발생종합보고서(제2026-298호).hwpx";

/// 한/글 2020 정본의 두 상자 하단 간격.
const HANGUL_BOTTOM_GAP_PX: f64 = 9.43;

#[test]
fn issue_7049_tac_nested_tables_sit_on_the_line_baseline() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let tree = core.build_page_render_tree(0).expect("page 1 render tree");

    // 선언 높이(HWPUNIT 7943 · 12651)로 두 TAC 중첩 표를 동정한다.
    let short = table_with_height(&tree.root, 105.9).expect("짧은 표(7943 HU)");
    let big = table_with_height(&tree.root, 168.7).expect("큰 표(12651 HU)");

    assert!(
        (short.0 - big.0).abs() > 10.0,
        "두 표의 상단이 붙으면 안 된다 (결함 시 둘 다 102.00): 짧은={:.2} 큰={:.2}",
        short.0,
        big.0
    );

    let gap = big.1 - short.1;
    assert!(
        (gap - HANGUL_BOTTOM_GAP_PX).abs() < 2.0,
        "하단 간격이 한/글 정본 {HANGUL_BOTTOM_GAP_PX}px 근처여야 한다 \
         (결함 시 62.80): {gap:.2}"
    );
}

/// 주어진 높이(px, 오차 1.5px)를 가진 첫 `Table` 의 `(top, bottom)`.
fn table_with_height(node: &RenderNode, height_px: f64) -> Option<(f64, f64)> {
    if matches!(node.node_type, RenderNodeType::Table(_))
        && (node.bbox.height - height_px).abs() < 1.5
    {
        return Some((node.bbox.y, node.bbox.y + node.bbox.height));
    }
    node.children
        .iter()
        .find_map(|child| table_with_height(child, height_px))
}
