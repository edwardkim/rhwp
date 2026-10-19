//! Host coordinate arithmetic only: no document layout or Hancom visual oracle.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::native_host::view::{
    NativePagePoint, NativeView, NativeViewConfig, NativeViewPoint, NativeViewRect, NativeViewSize,
    MAX_VIEW_EXTENT, MAX_VIEW_PAGES,
};

fn size(width: f64, height: f64) -> NativeViewSize {
    NativeViewSize { width, height }
}

fn point(x: f64, y: f64) -> NativeViewPoint {
    NativeViewPoint { x, y }
}

fn config(pages: Vec<NativeViewSize>, viewport: NativeViewSize) -> NativeViewConfig {
    NativeViewConfig {
        pages,
        viewport,
        zoom: 1.0,
        device_scale: 1.0,
        page_gap: 20.0,
    }
}

fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= 1e-9, "{actual} != {expected}");
}

#[test]
fn issue_536_native_view_centers_mixed_pages_with_fixed_inter_page_gaps() {
    let view = NativeView::new(config(
        vec![size(100.0, 200.0), size(200.0, 100.0)],
        size(300.0, 400.0),
    ))
    .unwrap();
    assert_eq!(view.content_size(), size(300.0, 320.0));
    assert_eq!(view.scroll_limit(), point(0.0, 0.0));
    let visible = view.visible_regions();
    assert_eq!(visible.len(), 2);
    assert_eq!(
        visible[0].page_rect,
        NativeViewRect {
            x: 100.0,
            y: 0.0,
            width: 100.0,
            height: 200.0
        }
    );
    assert_eq!(
        visible[1].page_rect,
        NativeViewRect {
            x: 50.0,
            y: 220.0,
            width: 200.0,
            height: 100.0
        }
    );
    let mut double = view.clone();
    double.zoom_at(2.0, point(0.0, 0.0)).unwrap();
    assert_eq!(double.content_size(), size(400.0, 620.0));
    // The page heights double, but the 20 host-unit gap does not.
    let bottom_first = double
        .page_to_viewport(NativePagePoint {
            page: 0,
            point: point(0.0, 200.0),
        })
        .unwrap()
        .y;
    let top_second = double
        .page_to_viewport(NativePagePoint {
            page: 1,
            point: point(0.0, 0.0),
        })
        .unwrap()
        .y;
    near(top_second - bottom_first, 20.0);
}

#[test]
fn issue_536_native_view_clamps_scroll_and_reclamps_after_page_or_viewport_changes() {
    let mut view = NativeView::new(config(vec![size(800.0, 1000.0)], size(300.0, 200.0))).unwrap();
    assert_eq!(view.scroll_limit(), point(500.0, 800.0));
    view.set_scroll(point(-50.0, f64::MAX)).unwrap();
    assert_eq!(view.state().scroll, point(0.0, 800.0));
    view.scroll_by(point(1000.0, -100.0)).unwrap();
    assert_eq!(view.state().scroll, point(500.0, 700.0));
    view.set_config(config(vec![size(100.0, 100.0)], size(300.0, 200.0)))
        .unwrap();
    assert_eq!(view.state().scroll, point(0.0, 0.0));
    assert!(!view.set_scroll(point(-1.0, -1.0)).unwrap());
}

#[test]
fn issue_536_native_view_page_transform_roundtrips_fractional_zoom_and_scroll() {
    for zoom in [0.75, 1.0, 1.25, 1.5, 2.0] {
        let mut cfg = config(
            vec![size(400.0, 500.0), size(600.0, 700.0)],
            size(250.0, 200.0),
        );
        cfg.zoom = zoom;
        cfg.device_scale = 1.25;
        let mut view = NativeView::new(cfg).unwrap();
        let logical = NativePagePoint {
            page: 1,
            point: point(300.0, 200.0),
        };
        let unscrolled = view.page_to_viewport(logical).unwrap();
        view.set_scroll(point(unscrolled.x - 100.0, unscrolled.y - 80.0))
            .unwrap();
        let on_screen = view.page_to_viewport(logical).unwrap();
        near(on_screen.x, 100.0);
        near(on_screen.y, 80.0);
        let back = view.viewport_to_page(on_screen).unwrap().unwrap();
        assert_eq!(back.page, logical.page);
        near(back.point.x, logical.point.x);
        near(back.point.y, logical.point.y);
        near(view.raster_scale(), zoom * 1.25);
    }
}

#[test]
fn issue_536_native_view_hit_testing_excludes_gaps_margins_and_open_edges() {
    let mut cfg = config(vec![size(100.0, 100.0); 2], size(200.0, 250.0));
    cfg.page_gap = 20.0;
    let view = NativeView::new(cfg).unwrap();
    for outside in [
        point(-1.0, 10.0),
        point(200.0, 10.0),
        point(50.0, 250.0),
        point(49.0, 10.0),
        point(150.0, 10.0),
        point(100.0, 100.0),
        point(100.0, 119.0),
    ] {
        assert_eq!(view.viewport_to_page(outside).unwrap(), None, "{outside:?}");
    }
    assert_eq!(
        view.viewport_to_page(point(50.0, 120.0)).unwrap(),
        Some(NativePagePoint {
            page: 1,
            point: point(0.0, 0.0)
        })
    );
    assert!(view
        .page_to_viewport(NativePagePoint {
            page: 0,
            point: point(100.0, 100.0)
        })
        .is_ok());
    assert!(view
        .page_to_viewport(NativePagePoint {
            page: 2,
            point: point(0.0, 0.0)
        })
        .is_err());
}

#[test]
fn issue_536_native_view_visible_regions_partition_only_page_intersections() {
    let mut cfg = config(vec![size(100.0, 100.0); 3], size(80.0, 80.0));
    cfg.zoom = 1.5;
    cfg.device_scale = 2.0;
    let mut view = NativeView::new(cfg).unwrap();
    view.set_scroll(point(20.0, 130.0)).unwrap();
    let visible = view.visible_regions();
    assert_eq!(
        visible.iter().map(|v| v.page).collect::<Vec<_>>(),
        vec![0, 1]
    );
    near(visible[0].viewport_rect.height, 20.0);
    near(visible[1].viewport_rect.y, 40.0);
    near(visible[1].viewport_rect.height, 40.0);
    for visible in &visible {
        near(visible.page_region.width * 1.5, visible.viewport_rect.width);
        near(
            visible.page_region.height * 1.5,
            visible.viewport_rect.height,
        );
        near(
            visible.page_rect.x + visible.page_region.x * 1.5,
            visible.viewport_rect.x,
        );
        near(
            visible.page_rect.y + visible.page_region.y * 1.5,
            visible.viewport_rect.y,
        );
        near(visible.raster_scale, 3.0);
        assert!(visible.page_region.x >= 0.0 && visible.page_region.y >= 0.0);
        assert!(visible.page_region.x + visible.page_region.width <= 100.0 + 1e-9);
        assert!(visible.page_region.y + visible.page_region.height <= 100.0 + 1e-9);
    }
}

#[test]
fn issue_536_native_view_zoom_preserves_page_anchor_until_scroll_bounds_prevent_it() {
    let mut view =
        NativeView::new(config(vec![size(600.0, 800.0); 3], size(200.0, 160.0))).unwrap();
    view.set_scroll(point(150.0, 900.0)).unwrap();
    let pointer = point(70.0, 60.0);
    let before = view.viewport_to_page(pointer).unwrap().unwrap();
    view.zoom_at(1.75, pointer).unwrap();
    let after = view.viewport_to_page(pointer).unwrap().unwrap();
    assert_eq!(after.page, before.page);
    near(after.point.x, before.point.x);
    near(after.point.y, before.point.y);
    assert!(!view.zoom_at(1.75, pointer).unwrap());
    view.zoom_at(0.1, pointer).unwrap();
    assert_eq!(
        view.state().scroll.x,
        0.0,
        "centered document cannot scroll negatively to keep an impossible anchor"
    );
    assert!(view.state().scroll.y >= 0.0 && view.state().scroll.y <= view.scroll_limit().y);
}

#[test]
fn issue_536_native_view_gap_zoom_uses_the_nearest_page_with_stable_virtual_anchor() {
    let mut view =
        NativeView::new(config(vec![size(600.0, 200.0); 4], size(200.0, 100.0))).unwrap();
    view.set_scroll(point(100.0, 170.0)).unwrap();
    let pointer = point(50.0, 35.0); // content y=205: five units below page 0.
    assert!(view.viewport_to_page(pointer).unwrap().is_none());
    view.zoom_at(2.0, pointer).unwrap();
    // Extended page-0 coordinate (150,205) stays under the pointer.
    near(view.state().scroll.x, 250.0);
    near(view.state().scroll.y, 375.0);
}

#[test]
fn issue_536_native_view_caret_reveal_is_minimal_and_handles_zero_width_and_large_targets() {
    let mut view =
        NativeView::new(config(vec![size(600.0, 800.0); 2], size(200.0, 160.0))).unwrap();
    let caret = NativeViewRect {
        x: 300.0,
        y: 100.0,
        width: 0.0,
        height: 20.0,
    };
    assert!(view.ensure_caret_visible(1, caret, 10.0).unwrap());
    assert_eq!(view.state().scroll, point(110.0, 790.0));
    assert!(!view.ensure_caret_visible(1, caret, 10.0).unwrap());
    let top = view
        .page_to_viewport(NativePagePoint {
            page: 1,
            point: point(300.0, 100.0),
        })
        .unwrap();
    assert_eq!(top, point(190.0, 130.0));
    let oversized = NativeViewRect {
        x: 0.0,
        y: 200.0,
        width: 600.0,
        height: 300.0,
    };
    view.ensure_caret_visible(0, oversized, 0.0).unwrap();
    assert_eq!(view.state().scroll, point(200.0, 270.0));
    let edge = NativeViewRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 1.0,
    };
    view.ensure_caret_visible(0, edge, 50.0).unwrap();
    assert_eq!(view.state().scroll, point(0.0, 0.0));
}

#[test]
fn issue_536_native_view_empty_pages_and_dpr_changes_preserve_logical_coordinates() {
    let mut empty = NativeView::new(config(vec![], size(200.0, 100.0))).unwrap();
    assert!(empty.visible_regions().is_empty());
    assert_eq!(empty.content_size(), size(200.0, 0.0));
    assert_eq!(empty.viewport_to_page(point(50.0, 50.0)).unwrap(), None);
    empty.zoom_at(2.0, point(50.0, 50.0)).unwrap();
    assert_eq!(empty.state().scroll, point(0.0, 0.0));
    let mut view = NativeView::new(config(vec![size(300.0, 400.0)], size(200.0, 100.0))).unwrap();
    view.set_scroll(point(20.0, 30.0)).unwrap();
    let before = view.visible_regions();
    let mut cfg = view.state().config.clone();
    cfg.device_scale = 2.0;
    view.set_config(cfg).unwrap();
    let after = view.visible_regions();
    assert_eq!(before[0].page_region, after[0].page_region);
    assert_eq!(before[0].page_rect, after[0].page_rect);
    assert_eq!(after[0].raster_scale, before[0].raster_scale * 2.0);
}

#[test]
fn issue_536_native_view_invalid_and_overflow_inputs_are_rejected_atomically() {
    let base = config(vec![size(100.0, 100.0)], size(50.0, 50.0));
    let mut view = NativeView::new(base.clone()).unwrap();
    view.set_scroll(point(10.0, 20.0)).unwrap();
    let before = view.clone();
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -1.0] {
        let mut cfg = base.clone();
        cfg.pages[0].width = invalid;
        assert!(view.set_config(cfg).is_err());
        let mut cfg = base.clone();
        cfg.viewport.height = invalid;
        assert!(view.set_config(cfg).is_err());
        assert!(view.zoom_at(invalid, point(10.0, 10.0)).is_err());
        let mut cfg = base.clone();
        cfg.device_scale = invalid;
        assert!(view.set_config(cfg).is_err());
        assert_eq!(view, before);
    }
    let mut cfg = base.clone();
    cfg.pages = vec![size(1.0, 1.0); MAX_VIEW_PAGES + 1];
    assert!(view.set_config(cfg).is_err());
    for cfg in [
        config(vec![size(1.0, MAX_VIEW_EXTENT); 2], size(1.0, 1.0)),
        config(
            vec![size(1.0, MAX_VIEW_EXTENT), size(1.0, f64::MIN_POSITIVE)],
            size(1.0, 1.0),
        ),
        config(vec![size(f64::MAX, 1.0)], size(1.0, 1.0)),
        NativeViewConfig {
            zoom: 64.0,
            device_scale: 2.0,
            ..base.clone()
        },
        NativeViewConfig {
            zoom: f64::MIN_POSITIVE,
            device_scale: f64::MIN_POSITIVE,
            ..base.clone()
        },
        NativeViewConfig {
            page_gap: f64::NAN,
            ..base.clone()
        },
    ] {
        assert!(view.set_config(cfg).is_err());
    }
    assert!(view.set_scroll(point(f64::NAN, 0.0)).is_err());
    assert!(view.scroll_by(point(f64::INFINITY, 0.0)).is_err());
    assert!(view.zoom_at(2.0, point(-1.0, 0.0)).is_err());
    assert!(view
        .ensure_caret_visible(
            0,
            NativeViewRect {
                x: 0.0,
                y: 99.0,
                width: 0.0,
                height: 2.0
            },
            1.0
        )
        .is_err());
    assert_eq!(view, before);
}

#[test]
fn issue_536_native_view_json_roundtrip_validates_state_and_recomputes_bounds() {
    let mut view =
        NativeView::new(config(vec![size(300.0, 400.0); 2], size(200.0, 100.0))).unwrap();
    view.set_scroll(point(70.0, 120.0)).unwrap();
    let json = serde_json::to_value(&view).unwrap();
    assert!(json["config"].get("deviceScale").is_some());
    assert!(json["config"].get("pageGap").is_some());
    assert!(json.get("pageBounds").is_none());
    let roundtrip: NativeView = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(roundtrip, view);
    let mut clamped = json.clone();
    clamped["scroll"] = serde_json::json!({ "x": -10, "y": 100000 });
    let clamped: NativeView = serde_json::from_value(clamped).unwrap();
    assert_eq!(clamped.state().scroll, point(0.0, 720.0));
    let mut invalid = json.clone();
    invalid["config"]["zoom"] = serde_json::json!(0);
    assert!(serde_json::from_value::<NativeView>(invalid).is_err());
    let mut excessive = json;
    excessive["config"]["pages"] = serde_json::json!(vec![size(1.0, 1.0); MAX_VIEW_PAGES + 1]);
    assert!(serde_json::from_value::<NativeView>(excessive).is_err());
}

#[test]
fn issue_536_native_view_representability_and_exact_page_count_boundaries_are_bounded() {
    let mut cfg = config(vec![size(1.0, 1.0); MAX_VIEW_PAGES], size(1.0, 1.0));
    cfg.page_gap = 0.0;
    let view = NativeView::new(cfg).unwrap();
    assert_eq!(view.content_size().height, MAX_VIEW_PAGES as f64);
    let mut cfg = config(
        vec![size(1.0, MAX_VIEW_EXTENT), size(1.0, f64::MIN_POSITIVE)],
        size(1.0, 1.0),
    );
    cfg.page_gap = 0.0;
    assert!(
        NativeView::new(cfg).is_err(),
        "a positive height that collapses at its origin must reject"
    );
    let view =
        NativeView::new(config(vec![size(0.0002, 1.0)], size(MAX_VIEW_EXTENT, 2.0))).unwrap();
    let region = view.visible_regions()[0].page_region;
    assert_eq!(region.x, 0.0);
    assert_eq!(
        region.width, 0.0002,
        "rounded host edges cannot enlarge page logical requests"
    );
}
