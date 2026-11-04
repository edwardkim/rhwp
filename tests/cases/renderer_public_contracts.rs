//! 공개 렌더링 API의 백엔드 선택·좌표 변환·첨자 계약.

use rhwp::renderer::{hwpunit_to_px, px_to_hwpunit, RenderBackend, TextStyle};

#[test]
fn test_render_backend_from_str() {
    assert_eq!(
        RenderBackend::from_str("canvas"),
        Some(RenderBackend::Canvas)
    );
    assert_eq!(RenderBackend::from_str("svg"), Some(RenderBackend::Svg));
    assert_eq!(RenderBackend::from_str("html"), Some(RenderBackend::Html));
    assert_eq!(RenderBackend::from_str("unknown"), None);
}

#[test]
fn test_hwpunit_to_px() {
    // 1인치 = 7200 HWPUNIT, 96 DPI → 96px
    let px = hwpunit_to_px(7200, 96.0);
    assert!((px - 96.0).abs() < 0.01);
}

#[test]
fn test_script_draw_metrics_matches_shared_contract() {
    // [#2771] SVG/Canvas/HTML/Skia/paint JSON 이 공유하는 첨자 계약:
    // 글꼴 0.7 배 + baseline 위 0.3em / 아래 0.15em.
    let base = TextStyle {
        font_size: 20.0,
        ..Default::default()
    };

    let sup = TextStyle {
        superscript: true,
        ..base.clone()
    };
    let (sup_size, sup_y) = sup.script_draw_metrics(20.0, 100.0);
    assert!(
        (sup_size - 14.0).abs() < 1e-9,
        "위첨자 글꼴은 0.7 배여야 함: {sup_size}"
    );
    assert!(
        (sup_y - 94.0).abs() < 1e-9,
        "위첨자 baseline 은 0.3em 위여야 함: {sup_y}"
    );

    let sub = TextStyle {
        subscript: true,
        ..base.clone()
    };
    let (sub_size, sub_y) = sub.script_draw_metrics(20.0, 100.0);
    assert!(
        (sub_size - 14.0).abs() < 1e-9,
        "아래첨자 글꼴은 0.7 배여야 함: {sub_size}"
    );
    assert!(
        (sub_y - 103.0).abs() < 1e-9,
        "아래첨자 baseline 은 0.15em 아래여야 함: {sub_y}"
    );

    // 비첨자는 인자를 그대로 돌려준다.
    assert_eq!(base.script_draw_metrics(20.0, 100.0), (20.0, 100.0));
}

#[test]
fn test_px_to_hwpunit() {
    let hu = px_to_hwpunit(96.0, 96.0);
    assert_eq!(hu, 7200);
}

#[test]
fn test_a4_page_size_px() {
    // A4: 210mm × 297mm = 59528 × 84188 HWPUNIT
    let w = hwpunit_to_px(59528, 96.0);
    let h = hwpunit_to_px(84188, 96.0);
    // A4 @ 96DPI ≈ 793.7 × 1122.5 px
    assert!((w - 793.7).abs() < 1.0);
    assert!((h - 1122.5).abs() < 1.0);
}
