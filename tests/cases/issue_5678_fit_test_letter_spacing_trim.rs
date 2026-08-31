//! [Issue #5678] 문제 3 — fit 판정의 자간 trim 이 어느 시험에도 구속되지 않았다.
//!
//! `fit_test_letter_spacing_trim_hwp` 와 `FitWidthHwp::trimmed` 는 글자마다 값을 할당하면서도
//! 저장소 어느 시험에도 걸려 있지 않았다. 다섯 건으로 계약을 못박는다.
//!
//! 음성 대조: trim 을 제거하면 부호 시험과 판정-뒤집기 시험 2건이 실패한다.
//!
//! 이 시험이 `src/` 의 `#[cfg(test)]` 가 아니라 여기 있는 이유는 source unit tier 정책이
//! 제품 소스의 신규 unit test 를 금지하기 때문이다. 헬퍼는 `renderer::composer::fit_test_internals`
//! 가 `#[doc(hidden)]` 으로 내보낸다 — rhwp 의 API 가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::composer::fit_test_internals::{
    fit_test_letter_spacing_trim_hwp, text_token_fits_line_hwp, to_hwp, FitWidthHwp,
    LINE_BREAK_TOLERANCE,
};

/// 계약: **후보 토큰의 마지막 글자 뒤 자간만** 뺀다. 줄 끝 글자의 뒤 자간은 그려지지
/// 않으므로 "들어가는가"를 따질 때 빼고 재고, 펜은 전체 폭만큼 전진한다.
#[test]
fn trim_takes_only_the_spacing_after_the_candidate_last_char() {
    let spacing = [1.0, 2.0, 4.0];
    // token_end_idx 는 exclusive — 마지막 글자는 idx-1 이다.
    assert_eq!(fit_test_letter_spacing_trim_hwp(&spacing, 1), to_hwp(1.0));
    assert_eq!(fit_test_letter_spacing_trim_hwp(&spacing, 2), to_hwp(2.0));
    assert_eq!(fit_test_letter_spacing_trim_hwp(&spacing, 3), to_hwp(4.0));
}

/// 토큰이 비었거나(`0`) 자간 배열 밖이면 보정하지 않는다.
#[test]
fn trim_is_zero_outside_the_spacing_slice() {
    let spacing = [3.0];
    assert_eq!(fit_test_letter_spacing_trim_hwp(&spacing, 0), 0);
    assert_eq!(fit_test_letter_spacing_trim_hwp(&spacing, 9), 0);
    assert_eq!(fit_test_letter_spacing_trim_hwp(&[], 1), 0);
}

/// **부호가 고정돼 있지 않다.** 양수 자간은 후보를 좁게, 음수 자간은 넓게 만든다.
///
/// 이슈가 지적한 대로 근거 코퍼스(`76076_regulatory_analysis`)는 `-0.16…-1.76px`
/// 로 음수 방향뿐이었다. 양수 방향을 여기서 함께 고정한다.
#[test]
fn trimmed_width_follows_the_spacing_sign() {
    let w = to_hwp(100.0);
    let narrower = FitWidthHwp::trimmed(w, &[2.0], 1);
    let wider = FitWidthHwp::trimmed(w, &[-2.0], 1);
    assert!(
        narrower.hwp() < w,
        "양수 자간은 fit 판정 폭을 좁혀야 한다: {} vs {w}",
        narrower.hwp()
    );
    assert!(
        wider.hwp() > w,
        "음수 자간은 fit 판정 폭을 넓혀야 한다: {} vs {w}",
        wider.hwp()
    );
    assert_eq!(narrower.hwp(), w - to_hwp(2.0));
    assert_eq!(wider.hwp(), w - to_hwp(-2.0));
}

/// **양수 자간에서 trim 이 판정을 뒤집는 지점이 실재한다.**
///
/// 이슈 문제 2 가 지목한 상쇄다 — 자기 뒤 자간을 뺀 덕에만 들어간 토큰이 있고,
/// 펜은 `w_hwp` 전체만큼 전진한다. 이것은 결함이 아니라 **선언된 계약**이다
/// (줄 끝 자간은 그려지지 않는다). 다만 계약이 실제로 발동하는 구간이 있다는
/// 사실 자체를 고정해 두어, 나중에 trim 을 없애도 아무 시험이 안 깨지는 일이
/// 다시 생기지 않게 한다.
#[test]
fn positive_spacing_trim_can_flip_the_fit_verdict() {
    let effective = to_hwp(100.0);
    let current = to_hwp(90.0);
    // 자연 폭은 tolerance 를 넘고, trim 을 빼면 들어간다.
    let token_w = effective + LINE_BREAK_TOLERANCE - current + to_hwp(1.0);
    let untrimmed = FitWidthHwp::untrimmed(token_w);
    let trimmed = FitWidthHwp::trimmed(token_w, &[2.0], 1);
    assert!(
        !text_token_fits_line_hwp(current, untrimmed, 0, effective, 12.0),
        "trim 없이는 들어가지 않아야 한다"
    );
    assert!(
        text_token_fits_line_hwp(current, trimmed, 0, effective, 12.0),
        "trim 을 빼면 들어가야 한다"
    );
}

/// 커닝 보정은 fit 판정 폭에만 더한다 — 펜 전진 폭 축과 섞이지 않는다.
#[test]
fn pair_adjustment_only_moves_the_fit_width() {
    let w = to_hwp(50.0);
    let base = FitWidthHwp::trimmed(w, &[1.0], 1);
    let adjusted = base.with_pair_adjustment(to_hwp(3.0));
    assert_eq!(adjusted.hwp(), base.hwp() + to_hwp(3.0));
}
