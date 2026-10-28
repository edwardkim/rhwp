//! 반각 강제 구두점은 대체 글리프가 전진폭보다 넓을 때만 전진폭에 맞춰 누른다.
//! 종전에는 항상 0.5배로 눌러, 이미 좁은 대체 따옴표가 실선처럼 가늘어지고
//! 전각으로 측정된 HFT 한점 리더는 전진폭 왼쪽 절반에 몰렸다.

use rhwp::renderer::halfwidth_punct_paint_scale;

#[test]
fn halfwidth_punct_paint_only_squeezes_glyphs_wider_than_advance() {
    // 전각 대체 글리프(20px)는 반각 전진(10px)에 맞춰 누른다.
    assert_eq!(halfwidth_punct_paint_scale(10.0, 20.0), 0.5);
    // 이미 좁은 대체 따옴표(4px)는 누르지 않는다.
    assert_eq!(halfwidth_punct_paint_scale(6.0, 4.0), 1.0);
    // 전각으로 측정된 HFT 한점 리더도 반으로 누르지 않는다.
    assert_eq!(halfwidth_punct_paint_scale(20.0, 20.0), 1.0);
    // 폭을 모르면 종전 반각 배율을 유지한다.
    assert_eq!(halfwidth_punct_paint_scale(0.0, 20.0), 0.5);
}
