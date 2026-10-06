//! [Issue #6845] 그러데이션 축을 **상자 정규화 공간**에서 다뤄 가로세로비만큼 축이 눕고
//! `cos` 부호가 뒤집혀 세로 방향이 반대이던 결함의 가드.
//!
//! ## 계약 — 축 방향은 `(sin a, −cos a)`
//!
//! 한/글 2024 정본(`pdf/113424_evaluation_guideline-2024.pdf`)에서 **세 각도가 독립적으로
//! 같은 식**을 가리켰다. `samples/issue6551/113424_evaluation_guideline.hwpx` 하나가 셋을
//! 모두 갖는다.
//!
//! ```text
//!              정본                                 (sin a, −cos a)   종전
//!   angle=0    5쪽 목차 막대 `#C8EDFF → #FFFFFF`
//!              아래가 `#C8EDFF`, 위가 흰색 → 위쪽      (0, −1)         아래쪽 ✗
//!   angle=90   29쪽 구분 막대 → 오른쪽                 (1, 0)          오른쪽 ✓
//!   angle=110  7쪽 제목 막대, 등색선 dx/dy = −0.365
//!              → 축 (0.939, 0.343)                     같음            (+3.72) ✗
//! ```
//!
//! `angle=0` 은 `header.xml` 의 색 순서가 `#C8EDFF → #FFFFFF` 임을 확인해 **축 부호가
//! 틀린 것이지 색이 뒤집힌 게 아님**을 가렸다.
//!
//! ## 왜 사용자 좌표계여야 하나
//!
//! `objectBoundingBox` 는 상자의 가로·세로를 각각 0~1 로 정규화하므로 **정사각형에서만**
//! 각도가 보존된다. 113424 7쪽 막대는 623.6×37.8px(16.5:1)라 축이 거의 수평으로 눕었다.
//! canvas 의 `(sin·w/2, cos·h/2)` 도 같은 왜곡이다.
//!
//! 이 시험은 순수 함수 `linear_gradient_axis` 를 직접 집는다 — 렌더 산출물을 거치지 않고
//! 계약을 잠그므로, SVG·canvas 어느 쪽 표현이 바뀌어도 축 규약은 남는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::linear_gradient_axis;

/// 축의 **단위 방향 벡터**를 돌려준다.
fn direction(angle: i16, x: f64, y: f64, w: f64, h: f64) -> (f64, f64) {
    let (x1, y1, x2, y2) = linear_gradient_axis(angle, x, y, w, h);
    let (dx, dy) = (x2 - x1, y2 - y1);
    let len = (dx * dx + dy * dy).sqrt();
    assert!(len > 0.0, "angle={angle} 축 길이가 0 이다");
    (dx / len, dy / len)
}

fn assert_dir(angle: i16, w: f64, h: f64, want: (f64, f64)) {
    let got = direction(angle, 0.0, 0.0, w, h);
    assert!(
        (got.0 - want.0).abs() < 1e-3 && (got.1 - want.1).abs() < 1e-3,
        "angle={angle} 상자 {w}x{h}: 축 방향 {got:?}, 기대 {want:?}"
    );
}

/// 정본이 확정한 세 각도. **상자 모양과 무관하게** 같은 방향이어야 한다.
#[test]
fn axis_direction_matches_the_hangul_oracle_for_three_angles() {
    // 113424 7쪽 제목 막대의 실제 치수(16.5:1)와 정사각형에서 모두 같아야 한다.
    for (w, h) in [(623.6, 37.8), (100.0, 100.0), (37.8, 623.6)] {
        // 5쪽 목차 막대 — 정본은 위쪽(색[0] 이 아래).
        assert_dir(0, w, h, (0.0, -1.0));
        // 29쪽 구분 막대 — 오른쪽.
        assert_dir(90, w, h, (1.0, 0.0));
        // 7쪽 제목 막대 — 등색선 dx/dy = −0.365 에 수직.
        let rad = (110.0f64).to_radians();
        assert_dir(110, w, h, (rad.sin(), -rad.cos()));
    }
}

/// 회귀 계약 — 종전 두 판을 그대로 배제한다.
#[test]
fn the_two_old_conventions_are_rejected() {
    // ① `angle=0` 이 아래쪽이던 판(SVG `objectBoundingBox`, canvas 둘 다).
    let (_, dy) = direction(0, 0.0, 0.0, 200.0, 50.0);
    assert!(
        dy < 0.0,
        "angle=0 축이 다시 아래쪽을 향한다 — cos 부호가 뒤집혔다"
    );

    // ② 가로세로비만큼 눕던 판. 16.5:1 상자에서 종전 SVG 는 등색선 기울기가 +3.72,
    //    canvas 는 방향이 (sin·w, cos·h) 였다. 참 방향과 5% 넘게 어긋난다.
    let (dx, dy) = direction(110, 0.0, 0.0, 623.6, 37.8);
    let rad = (110.0f64).to_radians();
    let squashed = {
        let (sx, sy) = (rad.sin() * 623.6, rad.cos() * 37.8);
        let len = (sx * sx + sy * sy).sqrt();
        (sx / len, sy / len)
    };
    assert!(
        (dx - squashed.0).abs() > 0.05 || (dy - squashed.1).abs() > 0.05,
        "축이 다시 상자 비율에 눌렸다 — got ({dx:.3},{dy:.3}), 종전 판 {squashed:?}"
    );
}

/// 끝점은 어떤 각도에서도 **상자를 덮어야** 한다.
///
/// 덮지 못하면 램프가 상자 안에서 끝나 가장자리가 단색으로 잘린다.
#[test]
fn the_axis_spans_the_whole_box_at_every_angle() {
    let (w, h) = (623.6, 37.8);
    for angle in (0..360).step_by(15) {
        let (x1, y1, x2, y2) = linear_gradient_axis(angle as i16, 0.0, 0.0, w, h);
        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = (dx * dx + dy * dy).sqrt();
        let (ux, uy) = (dx / len, dy / len);
        // 상자 네 꼭짓점을 축에 정사영했을 때 모두 [0, len] 안이어야 한다.
        for (px, py) in [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)] {
            let t = (px - x1) * ux + (py - y1) * uy;
            assert!(
                t >= -1e-6 && t <= len + 1e-6,
                "angle={angle}: 꼭짓점 ({px},{py}) 이 축 밖 t={t:.3} (len={len:.3})"
            );
        }
    }
}

/// 축이 상자 중심을 지나야 한다 — 어느 각도에서도 램프가 한쪽으로 쏠리지 않는다.
#[test]
fn the_axis_is_centred_on_the_box() {
    let (x, y, w, h) = (12.5, 34.0, 200.0, 50.0);
    for angle in (0..360).step_by(30) {
        let (x1, y1, x2, y2) = linear_gradient_axis(angle as i16, x, y, w, h);
        assert!(
            ((x1 + x2) / 2.0 - (x + w / 2.0)).abs() < 1e-9
                && ((y1 + y2) / 2.0 - (y + h / 2.0)).abs() < 1e-9,
            "angle={angle}: 축 중점이 상자 중심과 다르다"
        );
    }
}
