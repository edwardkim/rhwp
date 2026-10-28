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
//! 기존 세 검사는 정본 위여백과 0 대조군을 유지한다. 다행·다열의 잘못된 종전
//! 좌표 37.80px를 동결하던 검사는 제거했다(정본 41.55px, 현재 41.56px).
//! 리베이스 전 전29쪽 Native/fresh WASM 최저92.22927% 증적은 별도로 보존한다.
//! 현재 검사는 원본 HU 관계를 검증하며 그 통과만으로 최신 시각 검증·PR 승인을 선언하지 않는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpx_sample2.hwpx";

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

/// 표 윗변은 본문 원점과 원본 쪽 소유·바깥 위 여백의 합이다.
/// PDF 실측 좌표를 동결하지 않고 저장 HU와 실제 배치의 관계를 검사한다.
fn assert_source_outer_top(page_index: u32, para_index: usize, continuation: bool) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let section = &core.document().sections[0];
    let host = &section.paragraphs[para_index];
    let rhwp::model::control::Control::Table(source) = &host.controls[0] else {
        panic!("대상 문단의 원본 표가 없음");
    };
    let page = &section.section_def.page_def;
    let body_top_hu = f64::from(page.margin_top) + f64::from(page.margin_header);
    let host_top_hu = if continuation {
        0.0
    } else {
        f64::from(host.line_segs.first().expect("저장 host 줄").vertical_pos)
    };
    let expected_offset_hu = host_top_hu + f64::from(source.outer_margin_top);
    let tree = core.build_page_render_tree(page_index).expect("대상 쪽");
    let top = find_table_top(&tree.root, para_index)
        .unwrap_or_else(|| panic!("{}쪽 표 pi={para_index} — 시험 설정", page_index + 1));
    // 기본 96 DPI의 HU 환산만 사용한다. 반올림 허용은 좌표 정답이 아니다.
    let actual_offset_hu = top * 7200.0 / 96.0 - body_top_hu;
    assert!(
        (actual_offset_hu - expected_offset_hu).abs() <= 7200.0 / 96.0 * 0.3,
        "{}쪽 표 pi={para_index}: 본문 기준 원본 host+위 여백 {}HU, 실제 {}HU",
        page_index + 1,
        expected_offset_hu,
        actual_offset_hu
    );
}

/// 첫 조각은 해당 쪽의 원본 host 원점과 위 여백을 한 번 소유한다.
#[test]
fn issue_7063_hwpx_first_fragment_opens_outer_top_margin() {
    assert_source_outer_top(18, 182, false);
}

/// 이어받은 조각은 이전 쪽 host 거리를 반복하지 않고 위 여백만 다시 연다.
#[test]
fn issue_7063_hwpx_continuation_fragment_opens_outer_top_margin() {
    assert_source_outer_top(19, 182, true);
}

/// 위 여백이 없는 이어받기 조각은 본문 원점과 같은 위치에 선다.
#[test]
fn issue_7063_hwpx_zero_outer_top_fragment_is_untouched() {
    assert_source_outer_top(8, 74, true);
}
