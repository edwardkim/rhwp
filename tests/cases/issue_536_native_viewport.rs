//! Region raster contracts compare against the same full-page replay, not a
//! document visual golden. No Hancom equivalence or subtree culling is claimed.

#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use std::io::Cursor;

use rhwp::model::image::ImageEffect;
use rhwp::paint::{CacheHint, ClipKind, GroupKind, LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::raster_pixels::{RasterPixels, RasterRegion, RasterRegionPixels};
use rhwp::renderer::render_tree::{BoundingBox, EllipseNode, ImageNode, RectangleNode};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::renderer::{GradientFillInfo, PatternFillInfo, ShapeStyle};

fn rectangle(bounds: BoundingBox, color: u32) -> PaintOp {
    PaintOp::rectangle(
        bounds,
        RectangleNode::new(
            0.0,
            ShapeStyle {
                fill_color: Some(color),
                ..Default::default()
            },
            None,
        ),
    )
}

fn solid_page(width: f64, height: f64) -> PageLayerTree {
    let page = BoundingBox::new(0.0, 0.0, width, height);
    // Deliberately extend ink beyond all page edges. A region must retain the
    // full raster's surface clipping instead of revealing this overflow.
    let ink = BoundingBox::new(-20.0, -20.0, width + 40.0, height + 40.0);
    PageLayerTree::new(
        width,
        height,
        LayerNode::leaf(page, None, vec![rectangle(ink, 0xff)]),
    )
}

fn varied_page() -> PageLayerTree {
    let bounds = BoundingBox::new(0.0, 0.0, 47.25, 35.5);
    let gradient = PaintOp::rectangle(
        BoundingBox::new(1.25, 1.5, 42.0, 29.0),
        RectangleNode::new(
            2.0,
            ShapeStyle {
                opacity: 0.75,
                ..Default::default()
            },
            Some(Box::new(GradientFillInfo {
                gradient_type: 1,
                angle: 45,
                center_x: 50,
                center_y: 50,
                colors: vec![0xff, 0xff0000],
                positions: vec![0.0, 1.0],
            })),
        ),
    );
    let mut ellipse = EllipseNode::new(
        ShapeStyle {
            fill_color: Some(0x00ff00),
            opacity: 0.65,
            ..Default::default()
        },
        None,
    );
    ellipse.transform.rotation = 90.0;
    let pattern = PaintOp::rectangle(
        BoundingBox::new(7.5, 18.0, 17.0, 9.0),
        RectangleNode::new(
            0.0,
            ShapeStyle {
                pattern: Some(PatternFillInfo {
                    pattern_type: 5,
                    tile_size: 7.68,
                    pattern_color: 0,
                    background_color: 0xffffff,
                }),
                ..Default::default()
            },
            None,
        ),
    );
    let image = image::RgbaImage::from_fn(3, 3, |x, y| {
        image::Rgba([(x * 80) as u8, (y * 80) as u8, 200, 255])
    });
    let mut encoded = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image)
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    let mut image = ImageNode::new(1, Some(encoded.into_inner()));
    image.effect = ImageEffect::GrayScale;
    image.brightness = 12;
    let leaf = LayerNode::leaf(
        bounds,
        None,
        vec![
            gradient,
            PaintOp::ellipse(BoundingBox::new(10.0, 3.0, 12.0, 8.0), ellipse),
            pattern,
            PaintOp::image(BoundingBox::new(27.0, 14.0, 14.0, 13.0), image, None),
        ],
    );
    let clipped = LayerNode::clip_rect(
        bounds,
        None,
        BoundingBox::new(2.25, 2.5, 39.0, 26.0),
        leaf,
        ClipKind::Generic,
    );
    PageLayerTree::new(
        bounds.width,
        bounds.height,
        LayerNode::group(
            bounds,
            None,
            vec![clipped],
            CacheHint::StaticSubtree,
            GroupKind::Generic,
        ),
    )
}

fn assert_crop_and_padding(full: &RasterPixels, region: &RasterRegionPixels, padding: [u8; 4]) {
    region.pixels.validate().unwrap();
    assert_eq!(region.pixels.format, full.format);
    assert_eq!(region.pixels.color_space, full.color_space);
    assert_eq!(region.pixels.dpi, full.dpi);
    for y in 0..region.pixels.height {
        for x in 0..region.pixels.width {
            let page_x = i64::from(region.origin_x) + i64::from(x);
            let page_y = i64::from(region.origin_y) + i64::from(y);
            let expected = if page_x >= 0
                && page_y >= 0
                && page_x < i64::from(full.width)
                && page_y < i64::from(full.height)
            {
                let offset = page_y as usize * full.stride + page_x as usize * 4;
                &full.bytes[offset..offset + 4]
            } else {
                &padding
            };
            let offset = y as usize * region.pixels.stride + x as usize * 4;
            assert_eq!(
                &region.pixels.bytes[offset..offset + 4],
                expected,
                "region pixel ({x},{y}) at page ({page_x},{page_y})"
            );
        }
    }
}

#[test]
fn fractional_regions_replay_groups_clips_and_effects_on_the_full_page_grid() {
    let renderer = SkiaLayerRenderer::new();
    let page = varied_page();
    for scale in [0.75, 1.0, 1.25, 1.5, 2.0] {
        let options = RasterRenderOptions {
            scale,
            dpi: Some(144.0),
            ..Default::default()
        };
        let full = renderer.render_pixels_with_options(&page, options).unwrap();
        assert!(full.bytes.chunks_exact(4).any(|pixel| pixel[3] > 0));
        for request in [
            RasterRegion {
                x: 1.3,
                y: 2.2,
                width: 21.4,
                height: 18.1,
            },
            RasterRegion {
                x: 20.25,
                y: 11.75,
                width: 24.25,
                height: 20.5,
            },
            RasterRegion {
                x: 1.3,
                y: 2.2,
                width: 21.4,
                height: 18.1,
            },
        ] {
            let region = renderer
                .render_region_pixels_with_options(&page, request, options)
                .unwrap();
            assert_crop_and_padding(&full, &region, [0; 4]);
        }
    }
    assert!(
        renderer.cache_stats().pictures.hits > 0,
        "repeat regions reuse recorded groups"
    );
}

#[test]
fn integer_origin_preserves_antialiased_ellipse_pixels() {
    // Minimal reproducer: no gradient, group, clip, cache, or rotated shape.
    // Translating this AA oval by one integer pixel changes edge coverage on
    // Skia raster surfaces. Compare in page coordinates, without a tolerance.
    let renderer = SkiaLayerRenderer::new().with_cache_limits(0, 0);
    let bounds = BoundingBox::new(0.0, 0.0, 60.0, 45.0);
    let page = PageLayerTree::new(
        60.0,
        45.0,
        LayerNode::leaf(
            bounds,
            None,
            vec![PaintOp::ellipse(
                BoundingBox::new(15.0, 1.25, 10.0, 15.0),
                EllipseNode::new(
                    ShapeStyle {
                        fill_color: Some(0x00ff00),
                        opacity: 166.0 / 255.0,
                        ..Default::default()
                    },
                    None,
                ),
            )],
        ),
    );
    let options = RasterRenderOptions::default();
    let full = renderer.render_pixels_with_options(&page, options).unwrap();
    let region = renderer
        .render_region_pixels_with_options(
            &page,
            RasterRegion {
                x: 1.0,
                y: 0.0,
                width: 59.0,
                height: 45.0,
            },
            options,
        )
        .unwrap();
    assert_crop_and_padding(&full, &region, [0; 4]);
}

#[test]
fn returned_origin_and_size_enclose_fractional_logical_requests() {
    let renderer = SkiaLayerRenderer::new();
    let region = renderer
        .render_region_pixels_with_options(
            &solid_page(40.0, 30.0),
            RasterRegion {
                x: 1.3,
                y: 2.2,
                width: 7.4,
                height: 5.1,
            },
            RasterRenderOptions {
                scale: 1.25,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!((region.origin_x, region.origin_y), (1, 2));
    assert_eq!((region.pixels.width, region.pixels.height), (10, 8));
}

#[test]
fn negative_and_off_page_windows_preserve_full_raster_clipping_and_padding() {
    let renderer = SkiaLayerRenderer::new();
    let page = solid_page(31.2, 19.7);
    for (background_color, transparent, padding) in [
        (None, true, [0; 4]),
        (None, false, [255; 4]),
        (Some(0x00332211), true, [0x11, 0x22, 0x33, 255]),
    ] {
        let options = RasterRenderOptions {
            scale: 1.25,
            background_color,
            transparent,
            ..Default::default()
        };
        let full = renderer.render_pixels_with_options(&page, options).unwrap();
        for request in [
            RasterRegion {
                x: -3.3,
                y: -2.1,
                width: 9.0,
                height: 8.0,
            },
            RasterRegion {
                x: 27.3,
                y: 17.1,
                width: 9.0,
                height: 8.0,
            },
            RasterRegion {
                x: -12.0,
                y: -10.0,
                width: 2.0,
                height: 3.0,
            },
            RasterRegion {
                x: 50.0,
                y: 40.0,
                width: 2.0,
                height: 3.0,
            },
        ] {
            let region = renderer
                .render_region_pixels_with_options(&page, request, options)
                .unwrap();
            assert_crop_and_padding(&full, &region, padding);
        }
    }
}

#[test]
fn small_region_keeps_output_budget_independent_of_the_bounded_working_raster() {
    let renderer = SkiaLayerRenderer::new().with_cache_limits(0, 0);
    let page = solid_page(1_000.0, 2_000.0);
    let options = RasterRenderOptions {
        max_dimension: 32,
        max_pixels: 32 * 24,
        ..Default::default()
    };
    assert!(renderer.render_pixels_with_options(&page, options).is_err());
    let region = renderer
        .render_region_pixels_with_options(
            &page,
            RasterRegion {
                x: 500.0,
                y: 800.0,
                width: 32.0,
                height: 24.0,
            },
            options,
        )
        .unwrap();
    assert_eq!((region.origin_x, region.origin_y), (500, 800));
    assert_eq!((region.pixels.width, region.pixels.height), (32, 24));
    assert_eq!(region.pixels.bytes.len(), 32 * 24 * 4);
    assert!(region
        .pixels
        .bytes
        .chunks_exact(4)
        .all(|pixel| pixel == [255, 0, 0, 255]));
}

#[test]
fn region_working_and_peak_limits_fail_before_allocating_and_map_to_host_limits() {
    use rhwp::native_host::{NativeDocumentSession, NativeHostError};
    use rhwp::HwpError;

    let renderer = SkiaLayerRenderer::new().with_cache_limits(0, 0);
    let small = RasterRegion {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };
    for page in [
        solid_page(10_000.0, 20_000.0),
        solid_page(16_385.0, 1.0),
        solid_page(4097.0, 4096.0),
    ] {
        let error = renderer
            .render_region_pixels_with_options(&page, small, RasterRenderOptions::default())
            .unwrap_err();
        assert!(matches!(&error, HwpError::ResourceLimit(_)));
        let host_error = NativeHostError::from(error);
        assert!(matches!(host_error, NativeHostError::LimitExceeded(_)));
        assert_eq!(host_error.code(), "limitExceeded");
    }
    // Each bitmap fits its own limit, but 64 MiB scratch + this output exceeds
    // the independent 128 MiB peak. Rejection must precede either allocation.
    let error = renderer
        .render_region_pixels_with_options(
            &solid_page(4096.0, 4096.0),
            RasterRegion {
                width: 8193.0,
                height: 2048.0,
                ..small
            },
            RasterRenderOptions::default(),
        )
        .unwrap_err();
    assert!(matches!(error, HwpError::ResourceLimit(_)));

    // No intersection means no scratch; the output itself still obeys peak.
    let error = renderer
        .render_region_pixels_with_options(
            &solid_page(40.0, 30.0),
            RasterRegion {
                x: 100.0,
                width: 8193.0,
                height: 4096.0,
                ..small
            },
            RasterRenderOptions::default(),
        )
        .unwrap_err();
    assert!(matches!(error, HwpError::ResourceLimit(_)));

    // The caller's output guard remains first, even on an oversized page.
    let error = renderer
        .render_region_pixels_with_options(
            &solid_page(10_000.0, 20_000.0),
            RasterRegion {
                width: 2.0,
                ..small
            },
            RasterRenderOptions {
                max_dimension: 1,
                ..Default::default()
            },
        )
        .unwrap_err();
    assert!(matches!(error, HwpError::RenderError(_)));

    let session = NativeDocumentSession::new_blank().unwrap();
    assert!(matches!(
        session.render_region(0, small, 64.0),
        Err(NativeHostError::LimitExceeded(_))
    ));
}

#[test]
fn wholly_off_page_region_needs_no_working_bitmap_and_keeps_clear_color() {
    let renderer = SkiaLayerRenderer::new().with_cache_limits(0, 0);
    let page = solid_page(10_000.0, 20_000.0);
    for (background_color, transparent, expected) in [
        (None, true, [0, 0, 0, 0]),
        (None, false, [255, 255, 255, 255]),
        (Some(0x332211), true, [0x11, 0x22, 0x33, 255]),
    ] {
        let region = renderer
            .render_region_pixels_with_options(
                &page,
                RasterRegion {
                    x: -2.0,
                    y: -3.0,
                    width: 2.0,
                    height: 3.0,
                },
                RasterRenderOptions {
                    max_dimension: 3,
                    max_pixels: 6,
                    background_color,
                    transparent,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!((region.origin_x, region.origin_y), (-2, -3));
        assert_eq!((region.pixels.width, region.pixels.height), (2, 3));
        assert!(region
            .pixels
            .bytes
            .chunks_exact(4)
            .all(|pixel| pixel == expected));
    }
}

#[test]
fn region_rejects_invalid_geometry_scale_and_output_budgets_before_allocating() {
    let renderer = SkiaLayerRenderer::new();
    let page = solid_page(40.0, 30.0);
    let valid = RasterRegion {
        x: 1.0,
        y: 2.0,
        width: 4.0,
        height: 5.0,
    };
    for request in [
        RasterRegion {
            x: f64::NAN,
            ..valid
        },
        RasterRegion {
            y: f64::INFINITY,
            ..valid
        },
        RasterRegion {
            width: 0.0,
            ..valid
        },
        RasterRegion {
            height: -1.0,
            ..valid
        },
        RasterRegion {
            width: f64::INFINITY,
            ..valid
        },
        RasterRegion {
            x: f64::MAX,
            width: f64::MAX,
            ..valid
        },
        RasterRegion {
            x: 1e20,
            width: 1.0,
            ..valid
        },
        RasterRegion {
            x: 16_777_216.0,
            ..valid
        },
        RasterRegion {
            y: -16_777_217.0,
            ..valid
        },
    ] {
        assert!(renderer
            .render_region_pixels_with_options(&page, request, Default::default())
            .is_err());
    }
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e100, 1e-100] {
        assert!(renderer
            .render_region_pixels_with_options(
                &page,
                valid,
                RasterRenderOptions {
                    scale,
                    ..Default::default()
                },
            )
            .is_err());
    }
    for options in [
        RasterRenderOptions {
            max_dimension: 0,
            ..Default::default()
        },
        RasterRenderOptions {
            max_dimension: 4,
            ..Default::default()
        },
        RasterRenderOptions {
            max_pixels: 0,
            ..Default::default()
        },
        RasterRenderOptions {
            max_pixels: 19,
            ..Default::default()
        },
        RasterRenderOptions {
            dpi: Some(f64::NAN),
            ..Default::default()
        },
    ] {
        assert!(renderer
            .render_region_pixels_with_options(&page, valid, options)
            .is_err());
    }
    for width in [0.0, f64::NAN, f64::INFINITY, 16_777_217.0] {
        let invalid_page = PageLayerTree::new(
            width,
            30.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 1.0, 1.0), None, vec![]),
        );
        assert!(renderer
            .render_region_pixels_with_options(&invalid_page, valid, Default::default())
            .is_err());
    }
}
