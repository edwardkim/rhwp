//! [Issue #7063 레인①] 원본 HWPX 자리차지 RowBreak 표의 조각이 위쪽 바깥여백을
//! 내지 않아 통째로 `outMargin.top` 만큼 위에 앉는다.
//!
//! [#7095](https://github.com/edwardkim/rhwp/issues/7095) 는 한/글이 이 형상의
//! **조각마다** 표 위쪽 바깥여백을 다시 연다는 것을 native HWP5 저장본(156060125 ·
//! 30269)에서 세웠고, 근거 문서가 native 뿐이라 술어에 `hwp5_stored_pagination_layout`
//! 을 요구했다. 그래서 원본 HWPX 표는 첫 조각도 이어받은 조각도 이 여백을 아무도 내지
//! 않았다 — 같은 표의 **가로**는 `topbottom_float_outer_margin_left_hu` 가 이미 내고 있어
//! 한 표의 좌·상이 갈려 있었다(이슈 본문의 19쪽 표2: 좌단 39.70 / 윗변 88.10).
//!
//! 정본 `pdf/hwpx_sample2-hwpx-2020.pdf` (Producer `Hancom PDF 1.3.0.550` ·
//! Creator `Hwp 2022 0.0.0.0` · 29쪽 = rhwp 29쪽). 쪽 척도(1122.5/1121.33)를 걷고
//! 표 조각 윗변을 대조하면 **선언 `outMargin.top` 이 통째로 빠져 있다.**
//!
//! ```text
//!   쪽                     선언 omT      rhwp(수정 전)     정본      차
//!   19쪽 pi=182 첫 조각     141HU(1.88)      88.10        89.91    +1.81
//!   20쪽 pi=182 이어받음    141HU(1.88)      37.80        39.64    +1.84
//!   23~26쪽 pi=201 이어받음 141HU(1.88)      37.80        39.64    +1.84
//!    9쪽 pi= 74 이어받음      0HU(0.00)      37.80        37.72    -0.08   ← 0 대조군
//! ```
//!
//! 같은 값이 정본이 따로 있는 원본 HWPX 두 문서에서 더 나온다 —
//! `rowbreak-problem-pages`(283HU · 조각 3건 · 0 대조군 1건)와
//! `issue2004_cell_image_stack`(283HU · 5쪽). 뒤엣것은 같은 문서의 **HWP 쌍둥이**가
//! `#7095` 로 이미 정본과 맞고 HWPX 만 여백만큼 위였던 자리다.
//!
//! 수정은 `#7095` 술어의 **계보 게이트만** 연다(`table_partial.rs` 의
//! `single_cell_rowbreak_page_fragment`). 형상 조건(1×1 · 비-TAC · RowBreak)은 그대로이므로
//! 다행·다열 조각은 이 갈래 밖이고, 예산은 종전 게이트를 유지한다(그쪽까지 열면
//! `issue3236_split_table` 의 쪽수 정답지 2쪽이 3쪽으로 깨진다 — 실측).
//!
//! 이 검사는 정본 좌표와 함께 **0 대조군**과 **다행·다열 비적용**을 같이 잠가, 규칙이
//! 모든 조각으로 번지는 것을 막는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpx_sample2.hwpx";

fn load_page(page_index: u32) -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    core.build_page_render_tree(page_index)
        .expect("render tree")
        .root
}

/// 본문 최상위 표(칸 안 중첩 표 제외)의 윗변.
fn find_table_top(node: &RenderNode, para_index: usize) -> Option<f64> {
    if let RenderNodeType::Table(t) = &node.node_type {
        if t.para_index == Some(para_index) && t.cell_context.is_none() {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table_top(child, para_index))
}

/// 정본 좌표와 0.3px 안에서 맞는지. 잔차 실측이 ≤0.09px 이므로 이 공차는
/// 반올림만 흡수하고 1.88px 이탈은 잡는다.
fn assert_oracle_top(page_index: u32, para_index: usize, oracle: f64, before: f64) {
    let root = load_page(page_index);
    let top = find_table_top(&root, para_index)
        .unwrap_or_else(|| panic!("{}쪽 표 pi={para_index} — 시험 설정", page_index + 1));
    assert!(
        (top - oracle).abs() <= 0.3,
        "{}쪽 표 pi={para_index} 윗변이 한/글 정본({oracle:.2}px) 0.3px 안이어야 한다 \
         (수정 전 {before:.2}): {top:.2}",
        page_index + 1
    );
}

/// 이슈 본문의 19쪽 표2 — 쪽 중간에서 시작하는 **첫 조각**.
#[test]
fn issue_7063_hwpx_first_fragment_opens_outer_top_margin() {
    assert_oracle_top(18, 182, 89.91, 88.10);
}

/// 같은 표의 20쪽 **이어받은 조각** — 쪽 상단에서도 같은 여백을 연다.
#[test]
fn issue_7063_hwpx_continuation_fragment_opens_outer_top_margin() {
    assert_oracle_top(19, 182, 39.64, 37.80);
}

/// 다행·다열 표는 `#7095` 형상 조건(1×1) 밖이라 이 갈래에 오지 않는다. 열면 같은 문서의
/// HWP 쌍둥이와 간격이 갈린다(`issue_1133_hwpx_preserves_gap_between_consecutive_block_tables`:
/// hwp 120.6 vs hwpx 122.5). 28쪽 `pi=207`(33행×5열 · omT 283HU)은 정본 41.55 에 대해
/// 수정 전후 모두 37.80 이고, 이 축의 **미해결 잔여**로 남는다.
#[test]
fn issue_7063_hwpx_multi_column_fragment_stays_outside_this_lane() {
    let root = load_page(27);
    let top = find_table_top(&root, 207).expect("28쪽 표 pi=207 — 시험 설정");
    assert!(
        (top - 37.80).abs() <= 0.3,
        "다행·다열 조각은 이 갈래 밖이라 종전 좌표(37.80px)를 유지해야 한다: {top:.2}"
    );
}

/// 0 대조군 — `outMargin.top` 이 0 인 표는 수정 전후 모두 정본과 맞는다.
/// 규칙이 조각 전부로 번지면 이 검사가 깨진다.
#[test]
fn issue_7063_hwpx_zero_outer_top_fragment_is_untouched() {
    let root = load_page(8);
    let top = find_table_top(&root, 74).expect("9쪽 표 pi=74 — 시험 설정");
    assert!(
        (top - 37.76).abs() <= 0.3,
        "outMargin.top=0 인 표는 종전 좌표(37.76px)를 유지해야 한다: {top:.2}"
    );
}
