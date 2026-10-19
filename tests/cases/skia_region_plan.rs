//! Public preflight must agree with actual output without changing pixel replay.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use rhwp::paint::{LayerNode, PageLayerTree};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::raster_pixels::RasterRegion;
use rhwp::renderer::render_tree::BoundingBox;
use rhwp::renderer::skia::{SkiaLayerRenderer, SkiaRegionPlan};
use rhwp::HwpError;

fn region(x: f64, y: f64, width: f64, height: f64) -> RasterRegion {
    RasterRegion {
        x,
        y,
        width,
        height,
    }
}

#[test]
fn fractional_plan_has_explicit_origin_intersection_and_simultaneous_bytes() {
    let plan = SkiaRegionPlan::new(
        40.0,
        30.0,
        region(1.3, 2.2, 7.4, 5.1),
        RasterRenderOptions {
            scale: 1.25,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((plan.origin_x(), plan.origin_y()), (1, 2));
    assert_eq!((plan.width(), plan.height()), (10, 8));
    assert_eq!((plan.page_width(), plan.page_height()), (50, 38));
    assert_eq!(plan.intersection(), Some((1, 2, 11, 10)));
    assert_eq!((plan.stride(), plan.output_bytes()), (40, 320));
    assert_eq!(plan.working_bytes(), 50 * 38 * 4);
    assert_eq!(plan.peak_bitmap_bytes(), 50 * 38 * 4 + 320);
}

#[test]
fn negative_window_clips_copy_bounds_without_moving_the_output_origin() {
    let plan = SkiaRegionPlan::new(
        10.0,
        8.0,
        region(-2.2, -1.4, 8.0, 5.0),
        RasterRenderOptions::default(),
    )
    .unwrap();
    assert_eq!((plan.origin_x(), plan.origin_y()), (-3, -2));
    assert_eq!((plan.width(), plan.height()), (9, 6));
    assert_eq!(plan.intersection(), Some((0, 0, 6, 4)));
    assert_eq!(plan.peak_bitmap_bytes(), 10 * 8 * 4 + 9 * 6 * 4);
}

#[test]
fn offpage_preflight_does_not_need_a_working_bitmap_even_for_an_oversize_page() {
    let plan = SkiaRegionPlan::new(
        10_000.0,
        20_000.0,
        region(-2.0, -3.0, 2.0, 3.0),
        RasterRenderOptions {
            max_dimension: 3,
            max_pixels: 6,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(plan.intersection(), None);
    assert_eq!(plan.working_bytes(), 0);
    assert_eq!(plan.peak_bitmap_bytes(), 24);
}

#[test]
fn output_limits_are_independent_and_checked_before_working_limits() {
    let error = SkiaRegionPlan::new(
        10_000.0,
        20_000.0,
        region(0.0, 0.0, 2.0, 1.0),
        RasterRenderOptions {
            max_dimension: 1,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(error, HwpError::RenderError(_)));
    let error = SkiaRegionPlan::new(
        1000.0,
        2000.0,
        region(0.0, 0.0, 32.0, 24.0),
        RasterRenderOptions {
            max_dimension: 32,
            max_pixels: 32 * 24 - 1,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(error, HwpError::RenderError(_)));
    let plan = SkiaRegionPlan::new(
        1000.0,
        2000.0,
        region(0.0, 0.0, 32.0, 24.0),
        RasterRenderOptions {
            max_dimension: 32,
            max_pixels: 32 * 24,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(plan.output_bytes(), 32 * 24 * 4);
    assert_eq!(plan.working_bytes(), 1000 * 2000 * 4);
}

#[test]
fn working_dimension_pixels_and_peak_fail_without_allocating_bitmaps() {
    for (width, height) in [(16_385.0, 1.0), (4097.0, 4096.0)] {
        assert!(matches!(
            SkiaRegionPlan::new(
                width,
                height,
                region(0.0, 0.0, 1.0, 1.0),
                RasterRenderOptions::default()
            ),
            Err(HwpError::ResourceLimit(_))
        ));
    }
    let limit = SkiaRegionPlan::new(
        4096.0,
        4096.0,
        region(0.0, 0.0, 4096.0, 4096.0),
        RasterRenderOptions::default(),
    )
    .unwrap();
    assert_eq!(limit.peak_bitmap_bytes(), 128 * 1024 * 1024);
    assert!(matches!(
        SkiaRegionPlan::new(
            4096.0,
            4096.0,
            region(0.0, 0.0, 8193.0, 2048.0),
            RasterRenderOptions::default()
        ),
        Err(HwpError::ResourceLimit(_))
    ));
    assert!(matches!(
        SkiaRegionPlan::new(
            40.0,
            30.0,
            region(100.0, 0.0, 8193.0, 4096.0),
            RasterRenderOptions::default()
        ),
        Err(HwpError::ResourceLimit(_))
    ));
}

#[test]
fn nonfinite_degenerate_and_unrepresentable_geometry_is_rejected() {
    for request in [
        region(f64::NAN, 0.0, 1.0, 1.0),
        region(0.0, f64::INFINITY, 1.0, 1.0),
        region(0.0, 0.0, 0.0, 1.0),
        region(0.0, 0.0, -1.0, 1.0),
        region(0.0, 0.0, 1.0, f64::NAN),
        region(f64::MAX, 0.0, f64::MAX, 1.0),
        region(16_777_216.0, 0.0, 1.0, 1.0),
    ] {
        assert!(SkiaRegionPlan::new(20.0, 20.0, request, RasterRenderOptions::default()).is_err());
    }
    for (width, height) in [
        (0.0, 1.0),
        (1.0, f64::NAN),
        (f64::INFINITY, 1.0),
        (16_777_217.0, 1.0),
    ] {
        assert!(SkiaRegionPlan::new(
            width,
            height,
            region(-1.0, -1.0, 1.0, 1.0),
            RasterRenderOptions::default()
        )
        .is_err());
    }
}

#[test]
fn invalid_options_fail_in_preflight_and_in_the_render_entrypoint() {
    let page = PageLayerTree::new(
        10.0,
        10.0,
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, vec![]),
    );
    let renderer = SkiaLayerRenderer::new();
    let request = region(0.0, 0.0, 1.0, 1.0);
    for options in [
        RasterRenderOptions {
            scale: f64::NAN,
            ..Default::default()
        },
        RasterRenderOptions {
            scale: f64::INFINITY,
            ..Default::default()
        },
        RasterRenderOptions {
            scale: 0.0,
            ..Default::default()
        },
        RasterRenderOptions {
            scale: f64::MIN_POSITIVE,
            ..Default::default()
        },
        RasterRenderOptions {
            dpi: Some(0.0),
            ..Default::default()
        },
        RasterRenderOptions {
            max_pixels: 0,
            ..Default::default()
        },
        RasterRenderOptions {
            max_dimension: 0,
            ..Default::default()
        },
    ] {
        assert!(SkiaRegionPlan::new(10.0, 10.0, request, options).is_err());
        assert!(renderer
            .render_region_pixels_with_options(&page, request, options)
            .is_err());
    }
}

#[test]
fn preflight_matches_real_region_buffers_and_preserves_clear_policy() {
    let page = PageLayerTree::new(
        19.7,
        12.3,
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 19.7, 12.3), None, vec![]),
    );
    let renderer = SkiaLayerRenderer::new();
    for scale in [0.75, 1.0, 1.25, 1.5, 2.0] {
        for request in [
            region(1.3, 2.2, 7.4, 5.1),
            region(-3.3, -2.1, 9.0, 8.0),
            region(30.0, 20.0, 2.0, 3.0),
        ] {
            for (background_color, transparent, clear) in [
                (None, true, [0; 4]),
                (None, false, [255; 4]),
                (Some(0x332211), true, [0x11, 0x22, 0x33, 255]),
            ] {
                let options = RasterRenderOptions {
                    scale,
                    background_color,
                    transparent,
                    ..Default::default()
                };
                let plan = SkiaRegionPlan::new(page.page_width, page.page_height, request, options)
                    .unwrap();
                let actual = renderer
                    .render_region_pixels_with_options(&page, request, options)
                    .unwrap();
                assert_eq!(
                    (actual.origin_x, actual.origin_y),
                    (plan.origin_x(), plan.origin_y())
                );
                assert_eq!(
                    (actual.pixels.width, actual.pixels.height),
                    (plan.width(), plan.height())
                );
                assert_eq!(
                    (actual.pixels.stride, actual.pixels.bytes.len()),
                    (plan.stride(), plan.output_bytes())
                );
                assert!(actual
                    .pixels
                    .bytes
                    .chunks_exact(4)
                    .all(|pixel| pixel == clear));
            }
        }
    }
}
