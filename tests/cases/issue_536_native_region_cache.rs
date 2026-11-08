//! 관계형·고정형: 합성 PageLayerTree와 new_blank 문서의 crop 동등성, 고정 타일 겹침, LRU 용량·무효화 계약.
//! 주석 갱신: 2026-11-07T12:39:55+09:00; 직전 파일 커밋: 2026-11-06T18:07:14+09:00 (0d70e975806ef52eca1506b960f80a23dd0f9aa5).
//! 실물 페이지의 조판 좌표를 고정하는 검사가 아니므로 이 assertion의 raw 시각 측정값은 비해당이다.
//! 합성 알고리즘 계약과 별도의 Native/fresh WASM 실물 검증을 구분한다.
//! 근거: mydocs/pr/assets/seo_rii_20261030/README.md.
//! Region cache checks use the same direct full-page Skia raster as reference.
//! They prove immutable crop equivalence and bounded replay reuse, not Hancom
//! visual parity, OS presentation performance or subtree culling.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use rhwp::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use rhwp::native_host::editor::NativeEditor;
use rhwp::native_host::region_cache::{
    DEFAULT_REGION_CACHE_BYTES, MAX_REGION_CACHE_BYTES, MAX_REGION_CACHE_ENTRIES,
    MAX_TILE_CACHE_ENTRIES,
};
use rhwp::native_host::{NativeDocumentSession, NativeFrame, NativeHostError};
use rhwp::paint::{CacheHint, ClipKind, GroupKind, LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::raster_pixels::{RasterPixels, RasterRegion, RasterRegionPixels};
use rhwp::renderer::render_tree::{BoundingBox, EllipseNode, RectangleNode};
use rhwp::renderer::skia::{SkiaLayerRenderer, SkiaRegionPlan, SkiaTilePlan};
use rhwp::renderer::{GradientFillInfo, ShapeStyle};
use rhwp::{DocumentCore, HwpError};

fn request() -> RasterRegion {
    RasterRegion {
        x: 1.3,
        y: 2.2,
        width: 21.4,
        height: 18.1,
    }
}

fn culling_shape(bounds: BoundingBox, stroke_width: f64) -> LayerNode {
    LayerNode::leaf(
        bounds,
        None,
        vec![PaintOp::rectangle(
            bounds,
            RectangleNode::new(
                0.0,
                ShapeStyle {
                    fill_color: Some(0x22aa44),
                    stroke_color: Some(0x773311),
                    stroke_width,
                    ..Default::default()
                },
                None,
            ),
        )],
    )
}

#[test]
fn proven_off_clip_subtree_is_skipped_without_changing_pixels() {
    let page = BoundingBox::new(0.0, 0.0, 128.0, 96.0);
    let outside = LayerNode::group(
        // Deliberately incorrect layout bounds: the proof uses actual ops.
        BoundingBox::new(0.0, 0.0, 1.0, 1.0),
        None,
        (0..48)
            .map(|i| culling_shape(BoundingBox::new(80.0, f64::from(i), 8.0, 4.0), 0.25))
            .collect(),
        CacheHint::None,
        GroupKind::Generic,
    );
    let tree = PageLayerTree::new(
        page.width,
        page.height,
        LayerNode::clip_rect(
            page,
            None,
            BoundingBox::new(0.0, 0.0, 32.0, 64.0),
            LayerNode::group(
                page,
                None,
                vec![
                    culling_shape(BoundingBox::new(8.0, 8.0, 16.0, 16.0), 0.25),
                    outside,
                ],
                CacheHint::None,
                GroupKind::Generic,
            ),
            ClipKind::Generic,
        ),
    );
    let enabled = SkiaLayerRenderer::new().with_cache_limits(0, 0);
    let disabled = SkiaLayerRenderer::new()
        .with_cache_limits(0, 0)
        .with_viewport_culling(false);
    let options = RasterRenderOptions::default();
    assert_eq!(
        enabled
            .render_pixels_with_options(&tree, options)
            .unwrap()
            .bytes,
        disabled
            .render_pixels_with_options(&tree, options)
            .unwrap()
            .bytes
    );
    let skipped = enabled
        .render_raster_with_options(&tree, options)
        .unwrap()
        .diagnostics;
    let replayed = disabled
        .render_raster_with_options(&tree, options)
        .unwrap()
        .diagnostics;
    assert!(skipped.layer_nodes_culled > 0);
    assert_eq!(replayed.layer_nodes_culled, 0);
    assert!(replayed.layer_nodes_replayed >= skipped.layer_nodes_replayed + 48);
    assert!(replayed.paint_ops_replayed >= skipped.paint_ops_replayed + 48);
}

#[test]
fn stroke_floor_and_antialiasing_near_clip_survive_culling_at_fractional_scales() {
    let bounds = BoundingBox::new(0.0, 0.0, 128.0, 96.0);
    for profile in [
        rhwp::paint::RenderProfile::Screen,
        rhwp::paint::RenderProfile::Print,
    ] {
        for scale in [0.125, 0.75, 1.25, 2.0] {
            for cached in [false, true] {
                let mut tree = PageLayerTree::new(
                    bounds.width,
                    bounds.height,
                    LayerNode::clip_rect(
                        bounds,
                        None,
                        BoundingBox::new(0.0, 0.0, 32.0, 48.0),
                        LayerNode::group(
                            bounds,
                            None,
                            vec![
                                culling_shape(BoundingBox::new(32.1, 8.0, 8.0, 16.0), 0.01),
                                culling_shape(BoundingBox::new(110.0, 8.0, 8.0, 16.0), 0.01),
                            ],
                            CacheHint::StaticSubtree,
                            GroupKind::Generic,
                        ),
                        ClipKind::Generic,
                    ),
                );
                tree.profile = profile;
                let options = RasterRenderOptions {
                    scale,
                    transparent: true,
                    ..Default::default()
                };
                let enabled = if cached {
                    SkiaLayerRenderer::new()
                } else {
                    SkiaLayerRenderer::new().with_cache_limits(0, 0)
                };
                let disabled = SkiaLayerRenderer::new()
                    .with_cache_limits(0, 0)
                    .with_viewport_culling(false);
                let reference = disabled.render_pixels_with_options(&tree, options).unwrap();
                assert!(reference.bytes.chunks_exact(4).any(|pixel| pixel[3] > 0),
                    "the stroke beyond the clip must contribute boundary alpha: {profile:?} {scale}");
                for _ in 0..2 {
                    assert_eq!(
                        enabled
                            .render_pixels_with_options(&tree, options)
                            .unwrap()
                            .bytes,
                        reference.bytes
                    );
                }
                if !cached {
                    assert!(
                        enabled
                            .render_raster_with_options(&tree, options)
                            .unwrap()
                            .diagnostics
                            .layer_nodes_culled
                            > 0
                    );
                }
            }
        }
    }
}

#[test]
fn print_fill_snapping_matches_without_culling_at_clip_edges() {
    let bounds = BoundingBox::new(0.0, 0.0, 128.0, 96.0);
    for scale in [0.125, 0.75, 1.25, 2.0] {
        for x in [30.1, 31.9, 32.1] {
            let shape = |x| {
                let bbox = BoundingBox::new(x, 8.2, 8.1, 16.3);
                LayerNode::leaf(
                    bbox,
                    None,
                    vec![PaintOp::rectangle(
                        bbox,
                        RectangleNode::new(
                            0.0,
                            ShapeStyle {
                                fill_color: Some(0x33bb77),
                                ..Default::default()
                            },
                            None,
                        ),
                    )],
                )
            };
            let mut tree = PageLayerTree::new(
                bounds.width,
                bounds.height,
                LayerNode::clip_rect(
                    bounds,
                    None,
                    BoundingBox::new(0.0, 0.0, 32.0, 48.0),
                    LayerNode::group(
                        bounds,
                        None,
                        vec![shape(x), shape(110.0)],
                        CacheHint::None,
                        GroupKind::Generic,
                    ),
                    ClipKind::Generic,
                ),
            );
            tree.profile = rhwp::paint::RenderProfile::Print;
            let options = RasterRenderOptions {
                scale,
                transparent: true,
                ..Default::default()
            };
            let enabled = SkiaLayerRenderer::new().with_cache_limits(0, 0);
            let disabled = SkiaLayerRenderer::new()
                .with_cache_limits(0, 0)
                .with_viewport_culling(false);
            assert_eq!(
                enabled
                    .render_pixels_with_options(&tree, options)
                    .unwrap()
                    .bytes,
                disabled
                    .render_pixels_with_options(&tree, options)
                    .unwrap()
                    .bytes
            );
            assert!(
                enabled
                    .render_raster_with_options(&tree, options)
                    .unwrap()
                    .diagnostics
                    .layer_nodes_culled
                    > 0
            );
        }
    }
}

#[test]
fn unknown_shadow_bounds_are_replayed_even_outside_the_clip() {
    let bounds = BoundingBox::new(0.0, 0.0, 128.0, 96.0);
    let shape = BoundingBox::new(100.0, 10.0, 8.0, 8.0);
    let tree = PageLayerTree::new(
        bounds.width,
        bounds.height,
        LayerNode::clip_rect(
            bounds,
            None,
            BoundingBox::new(0.0, 0.0, 32.0, 48.0),
            LayerNode::leaf(
                shape,
                None,
                vec![PaintOp::ellipse(
                    shape,
                    EllipseNode::new(
                        ShapeStyle {
                            fill_color: Some(0),
                            shadow: Some(rhwp::renderer::ShadowStyle {
                                shadow_type: 1,
                                color: 0,
                                offset_x: -100.0,
                                offset_y: 0.0,
                                alpha: 0,
                            }),
                            ..Default::default()
                        },
                        None,
                    ),
                )],
            ),
            ClipKind::Generic,
        ),
    );
    let output = SkiaLayerRenderer::new()
        .with_cache_limits(0, 0)
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .unwrap();
    assert_eq!(output.diagnostics.layer_nodes_culled, 0);
    assert!(output.diagnostics.paint_ops_replayed > 0);
}

#[test]
fn cold_fixed_tiles_match_with_culling_enabled_or_disabled() {
    let bounds = BoundingBox::new(0.0, 0.0, 1400.0, 900.0);
    let tree = PageLayerTree::new(
        bounds.width,
        bounds.height,
        LayerNode::group(
            bounds,
            None,
            (0..80)
                .map(|i| {
                    culling_shape(
                        BoundingBox::new(
                            f64::from(i % 10) * 135.0 + 0.3,
                            f64::from(i / 10) * 100.0 + 0.7,
                            30.0,
                            20.0,
                        ),
                        0.1,
                    )
                })
                .collect(),
            CacheHint::None,
            GroupKind::Generic,
        ),
    );
    for scale in [0.75, 1.25, 2.0] {
        let options = RasterRenderOptions {
            scale,
            ..Default::default()
        };
        let region = RasterRegion {
            x: 397.3,
            y: 237.1,
            width: 240.5,
            height: 180.4,
        };
        let enabled = SkiaLayerRenderer::new().with_cache_limits(0, 0);
        let disabled = SkiaLayerRenderer::new()
            .with_cache_limits(0, 0)
            .with_viewport_culling(false);
        let result = enabled
            .render_tiled_region_pixels_with_options(&tree, region, options)
            .unwrap();
        let reference = disabled
            .render_tiled_region_pixels_with_options(&tree, region, options)
            .unwrap();
        assert_eq!(
            (result.origin_x, result.origin_y),
            (reference.origin_x, reference.origin_y)
        );
        assert_eq!(result.pixels.bytes, reference.pixels.bytes);
    }
}

fn page() -> PageLayerTree {
    let bounds = BoundingBox::new(0.0, 0.0, 60.0, 45.0);
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
    let ellipse = PaintOp::ellipse(
        BoundingBox::new(15.0, 1.25, 10.0, 15.0),
        EllipseNode::new(
            ShapeStyle {
                fill_color: Some(0x00ff00),
                opacity: 166.0 / 255.0,
                ..Default::default()
            },
            None,
        ),
    );
    PageLayerTree::new(
        bounds.width,
        bounds.height,
        LayerNode::clip_rect(
            bounds,
            None,
            BoundingBox::new(2.25, 2.5, 39.0, 26.0),
            LayerNode::leaf(bounds, None, vec![gradient, ellipse]),
            ClipKind::Generic,
        ),
    )
}

fn assert_crop(full: &RasterPixels, region: &RasterRegionPixels, padding: [u8; 4]) {
    region.pixels.validate().unwrap();
    for y in 0..region.pixels.height {
        for x in 0..region.pixels.width {
            let px = i64::from(x) + i64::from(region.origin_x);
            let py = i64::from(y) + i64::from(region.origin_y);
            let expected = if px >= 0
                && py >= 0
                && px < i64::from(full.width)
                && py < i64::from(full.height)
            {
                let offset = py as usize * full.stride + px as usize * 4;
                &full.bytes[offset..offset + 4]
            } else {
                &padding
            };
            let offset = y as usize * region.pixels.stride + x as usize * 4;
            assert_eq!(
                &region.pixels.bytes[offset..offset + 4],
                expected,
                "region ({x},{y}) maps to page ({px},{py})"
            );
        }
    }
}

#[test]
fn one_tile_matches_the_existing_full_page_raster_and_padding_policy() {
    let renderer = SkiaLayerRenderer::new().with_cache_limits(0, 0);
    for scale in [0.75, 1.25, 2.0] {
        for (background_color, transparent, padding) in [
            (None, true, [0; 4]),
            (None, false, [255; 4]),
            (Some(0x332211), true, [0x11, 0x22, 0x33, 255]),
        ] {
            let options = RasterRenderOptions {
                scale,
                background_color,
                transparent,
                dpi: Some(144.0),
                ..Default::default()
            };
            let full = renderer
                .render_pixels_with_options(&page(), options)
                .unwrap();
            for region in [
                request(),
                RasterRegion {
                    x: -2.3,
                    y: -1.2,
                    ..request()
                },
                RasterRegion {
                    x: 50.25,
                    y: 39.5,
                    ..request()
                },
            ] {
                let tile = renderer
                    .render_tiled_region_pixels_with_options(&page(), region, options)
                    .unwrap();
                assert_crop(&full, &tile, padding);
                assert_eq!(tile.pixels.color_space, full.color_space);
                assert_eq!(tile.pixels.dpi, full.dpi);
            }
        }
    }
}

fn tile_boundary_page() -> PageLayerTree {
    let bounds = BoundingBox::new(0.0, 0.0, 710.5, 685.25);
    let mut tree = PageLayerTree::new(
        bounds.width,
        bounds.height,
        LayerNode::clip_rect(
            bounds,
            None,
            BoundingBox::new(233.25, 235.5, 460.25, 435.5),
            LayerNode::leaf(
                bounds,
                None,
                vec![
                    PaintOp::rectangle(
                        BoundingBox::new(234.75, 237.25, 465.0, 430.5),
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
                    ),
                    PaintOp::ellipse(
                        BoundingBox::new(240.2, 245.75, 452.5, 420.25),
                        EllipseNode::new(
                            ShapeStyle {
                                fill_color: Some(0x00ff00),
                                opacity: 166.0 / 255.0,
                                ..Default::default()
                            },
                            None,
                        ),
                    ),
                ],
            ),
            ClipKind::Generic,
        ),
    );
    tree.root = LayerNode::group(
        bounds,
        None,
        vec![tree.root],
        CacheHint::StaticSubtree,
        GroupKind::Generic,
    );
    tree
}

#[test]
fn fixed_grid_tiles_keep_overlap_pixels_stable_across_window_and_cache_changes() {
    let renderer = SkiaLayerRenderer::new();
    let uncached = SkiaLayerRenderer::new().with_cache_limits(0, 0);
    let tree = tile_boundary_page();
    for scale in [0.75, 1.25, 1.5] {
        for (background_color, transparent, padding) in [
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
            let full = renderer
                .render_tiled_region_pixels_with_options(
                    &tree,
                    RasterRegion {
                        x: 0.0,
                        y: 0.0,
                        width: tree.page_width,
                        height: tree.page_height,
                    },
                    options,
                )
                .unwrap();
            assert_eq!(full.origin_x, 0);
            assert_eq!(full.origin_y, 0);
            assert!(
                SkiaTilePlan::new(
                    tree.page_width,
                    tree.page_height,
                    RasterRegion {
                        x: 0.0,
                        y: 0.0,
                        width: tree.page_width,
                        height: tree.page_height
                    },
                    options
                )
                .unwrap()
                .tile_count()
                    > 1
            );
            // Cross both grid axes, request overlapping parts, and include the
            // last fractional page column/row and off-page clear padding.
            for region in [
                RasterRegion {
                    x: 220.3,
                    y: 230.2,
                    width: 400.4,
                    height: 380.1,
                },
                RasterRegion {
                    x: 340.5,
                    y: 345.25,
                    width: 235.0,
                    height: 220.75,
                },
                RasterRegion {
                    x: -1.3,
                    y: -2.2,
                    width: 714.6,
                    height: 690.4,
                },
                RasterRegion {
                    x: 650.25,
                    y: 630.75,
                    width: 65.5,
                    height: 60.5,
                },
            ] {
                let output = renderer
                    .render_tiled_region_pixels_with_options(&tree, region, options)
                    .unwrap();
                assert_crop(&full.pixels, &output, padding);
                assert_eq!(
                    output,
                    uncached
                        .render_tiled_region_pixels_with_options(&tree, region, options)
                        .unwrap()
                );
            }
        }
    }
    assert!(renderer.cache_stats().pictures.hits > 0);
    assert_eq!(uncached.cache_stats().pictures.hits, 0);
}

#[test]
fn a_large_page_renders_a_small_tile_without_a_full_page_bitmap() {
    let renderer = SkiaLayerRenderer::new();
    let mut tree = page();
    tree.page_width = 20_000.0;
    tree.page_height = 30_000.0;
    let options = RasterRenderOptions::default();
    assert!(renderer
        .render_region_pixels_with_options(&tree, request(), options)
        .is_err());
    let output = renderer
        .render_tiled_region_pixels_with_options(&tree, request(), options)
        .unwrap();
    let reference = renderer
        .render_tiled_region_pixels_with_options(&page(), request(), options)
        .unwrap();
    assert_eq!(output, reference);
    assert!(output
        .pixels
        .bytes
        .chunks_exact(4)
        .any(|pixel| pixel[3] != 0));
    let before = renderer.cache_stats();
    let offpage = renderer
        .render_tiled_region_pixels_with_options(
            &tree,
            RasterRegion {
                x: -4.0,
                y: -4.0,
                width: 2.0,
                height: 2.0,
            },
            options,
        )
        .unwrap();
    assert_eq!(offpage.pixels.bytes, vec![0; 16]);
    assert_eq!(
        renderer.cache_stats(),
        before,
        "off-page windows do not replay"
    );
    tree.page_width = f64::from(SkiaRegionPlan::MAX_EXACT_PIXEL);
    tree.page_height = tree.page_width;
    let edge = renderer
        .render_tiled_region_pixels_with_options(
            &tree,
            RasterRegion {
                x: tree.page_width - 2.0,
                y: tree.page_height - 2.0,
                width: 2.0,
                height: 2.0,
            },
            options,
        )
        .unwrap();
    assert_eq!(edge.origin_x, SkiaRegionPlan::MAX_EXACT_PIXEL - 2);
    assert_eq!(edge.origin_y, SkiaRegionPlan::MAX_EXACT_PIXEL - 2);
    assert_eq!(edge.pixels.bytes, vec![0; 16]);
}

#[test]
fn native_tiled_frames_allow_high_scale_and_preserve_the_page_cache() {
    let session = session(false);
    session.render_region(0, request(), 0.5).unwrap();
    let before = session.region_cache_stats();
    assert!(session.render_region(0, request(), 16.0).is_err());
    let frame = session.render_tiled_region(0, request(), 16.0).unwrap();
    frame.pixels.validate().unwrap();
    assert_eq!(frame.revision, session.revision());
    assert_eq!(frame.font_generation, session.font_generation());
    assert_eq!(frame.origin_x, (request().x * 16.0).floor() as i32);
    assert_eq!(frame.origin_y, (request().y * 16.0).floor() as i32);
    let after_tile = session.region_cache_stats();
    assert_eq!(after_tile.entries, before.entries);
    assert_eq!(after_tile.replays, before.replays);
    assert_eq!(after_tile.evictions, before.evictions);
    assert_eq!(after_tile.hits, before.hits);
    assert_eq!(after_tile.tile_entries, 1);
    assert_eq!(after_tile.tile_replays, 1);
    assert_eq!(
        after_tile.retained_bytes,
        before.retained_bytes + after_tile.tile_retained_bytes
    );
    assert!(session
        .render_tiled_region(u32::MAX, request(), 1.0)
        .is_err());
    assert!(session.render_tiled_region(0, request(), f64::NAN).is_err());
    assert!(session
        .render_tiled_region(
            0,
            RasterRegion {
                width: 0.0,
                ..request()
            },
            1.0
        )
        .is_err());
    assert_eq!(session.region_cache_stats(), after_tile);
    session.render_region(0, request(), 0.5).unwrap();
    assert_eq!(session.region_cache_stats().hits, before.hits + 1);
}

#[test]
fn prepared_snapshot_reads_fractional_aa_gradient_and_clip_regions_exactly() {
    let renderer = SkiaLayerRenderer::new().with_cache_limits(0, 0);
    let tree = page();
    for scale in [0.75, 1.0, 1.25, 1.5, 2.0] {
        for (background_color, transparent, padding) in [
            (None, true, [0; 4]),
            (None, false, [255; 4]),
            (Some(0x332211), true, [0x11, 0x22, 0x33, 255]),
        ] {
            let options = RasterRenderOptions {
                scale,
                dpi: Some(144.0),
                background_color,
                transparent,
                ..Default::default()
            };
            let full = renderer.render_pixels_with_options(&tree, options).unwrap();
            let (cold, prepared) = renderer
                .prepare_page_for_region(&tree, request(), options)
                .unwrap();
            let prepared = prepared.unwrap();
            assert_eq!(prepared.retained_bytes(), full.bytes.len());
            assert_crop(&full, &cold, padding);
            let before = renderer.cache_stats();
            for region in [
                request(),
                RasterRegion {
                    x: -2.3,
                    y: -1.2,
                    ..request()
                },
                RasterRegion {
                    x: 50.25,
                    y: 39.5,
                    ..request()
                },
                RasterRegion {
                    x: -100.0,
                    ..request()
                },
            ] {
                let warm = prepared.render_region(region).unwrap();
                assert_crop(&full, &warm, padding);
                assert_eq!(warm.pixels.color_space, full.color_space);
                assert_eq!(warm.pixels.dpi, full.dpi);
            }
            assert_eq!(
                renderer.cache_stats(),
                before,
                "snapshot reads do not replay"
            );
        }
    }
}

#[test]
fn prepared_snapshot_owns_pixels_after_renderer_and_tree_are_dropped() {
    let (prepared, expected) = {
        let renderer = SkiaLayerRenderer::new();
        let tree = page();
        let (output, prepared) = renderer
            .prepare_page_for_region(&tree, request(), RasterRenderOptions::default())
            .unwrap();
        (prepared.unwrap(), output)
    };
    assert_eq!(prepared.render_region(request()).unwrap(), expected);
}

#[test]
fn prepared_off_page_reads_count_retained_pixels_in_peak_budget_before_allocating() {
    let tree = page();
    let options = RasterRenderOptions::default();
    let renderer = SkiaLayerRenderer::new();
    let (_, prepared) = renderer
        .prepare_page_for_region(&tree, request(), options)
        .unwrap();
    let off_page = RasterRegion {
        x: 100.0,
        y: 100.0,
        width: 16384.0,
        height: 2048.0,
    };
    // The direct planner allows exactly 128 MiB with no working raster. The
    // prepared image's existing allocation makes that same output too large.
    let direct = SkiaRegionPlan::new(tree.page_width, tree.page_height, off_page, options).unwrap();
    assert_eq!(direct.working_bytes(), 0);
    assert_eq!(direct.output_bytes(), 128 * 1024 * 1024);
    assert!(matches!(
        prepared.unwrap().render_region(off_page),
        Err(HwpError::ResourceLimit(_))
    ));
}

#[test]
fn preparing_a_wholly_off_page_request_avoids_an_oversized_working_bitmap() {
    let mut tree = page();
    tree.page_width = 10000.0;
    tree.page_height = 20000.0;
    let renderer = SkiaLayerRenderer::new();
    let (output, prepared) = renderer
        .prepare_page_for_region(
            &tree,
            RasterRegion {
                x: -4.0,
                y: -4.0,
                width: 2.0,
                height: 2.0,
            },
            RasterRenderOptions::default(),
        )
        .unwrap();
    assert!(prepared.is_none());
    assert_eq!(output.pixels.bytes, vec![0; 16]);
    assert_eq!(renderer.cache_stats().pictures.misses, 0);
}

fn session(two_pages: bool) -> NativeDocumentSession {
    session_with_pages(if two_pages { 2 } else { 1 })
}

fn session_with_pages(pages: usize) -> NativeDocumentSession {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "Skia 가😀").unwrap();
    if pages > 1 {
        let mut document = core.document().clone();
        for _ in 1..pages {
            document.sections.push(document.sections[0].clone());
        }
        core.set_document(document);
        for section in 1..pages {
            core.insert_text_native(section, 0, 0, &format!("Different page {section}: ABC "))
                .unwrap();
        }
        assert_eq!(core.page_count() as usize, pages);
    }
    NativeDocumentSession::from_core(core)
}

fn page_bytes(session: &NativeDocumentSession, page: u32, scale: f64) -> usize {
    let size = session.page_size(page).unwrap();
    (size.width * scale).ceil() as usize * (size.height * scale).ceil() as usize * 4
}

#[test]
fn native_regions_replay_once_then_reuse_one_bounded_page_for_moving_windows() {
    let session = session(false);
    let scale = 0.75;
    let reference = session.render_page(0, scale).unwrap();
    let first = session.render_region(0, request(), scale).unwrap();
    let after_cold = session.region_cache_stats();
    assert_eq!(after_cold.replays, 1);
    assert_eq!(after_cold.misses, 1);
    assert_eq!(after_cold.entries, 1);
    assert!(after_cold.retained_bytes <= DEFAULT_REGION_CACHE_BYTES);
    let renderer_stats = session.cache_stats();
    for (index, x) in [2.5, 10.25, 20.75, 1.3].into_iter().enumerate() {
        let frame = session
            .render_region(0, RasterRegion { x, ..request() }, scale)
            .unwrap();
        assert_crop(
            &reference.pixels,
            &RasterRegionPixels {
                pixels: frame.pixels,
                origin_x: frame.origin_x,
                origin_y: frame.origin_y,
            },
            [255; 4],
        );
        let stats = session.region_cache_stats();
        assert_eq!(stats.replays, 1);
        assert_eq!(stats.hits, index as u64 + 1);
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.retained_bytes, after_cold.retained_bytes);
        assert_eq!(session.cache_stats(), renderer_stats);
    }
    assert_eq!(
        session.render_region(0, request(), scale).unwrap().pixels,
        first.pixels
    );
}

#[test]
fn native_page_round_trips_and_scale_keys_reuse_snapshots_until_explicit_clear() {
    let session = session(true);
    session.render_region(0, request(), 0.5).unwrap();
    session.render_region(1, request(), 0.5).unwrap();
    assert_eq!(session.region_cache_stats().replays, 2);
    assert_eq!(session.region_cache_stats().entries, 2);
    assert_eq!(session.region_cache_stats().evictions, 0);
    session.render_region(0, request(), 0.5).unwrap();
    session.render_region(0, request(), 0.75).unwrap();
    assert_eq!(session.region_cache_stats().replays, 3);
    assert_eq!(session.region_cache_stats().hits, 1);
    assert_eq!(session.region_cache_stats().entries, 3);
    assert_eq!(session.region_cache_stats().evictions, 0);
    assert_eq!(
        session.region_cache_stats().retained_bytes,
        page_bytes(&session, 0, 0.5) + page_bytes(&session, 1, 0.5) + page_bytes(&session, 0, 0.75)
    );
    let before_clear = session.region_cache_stats();
    session.clear_caches();
    let cleared = session.region_cache_stats();
    assert_eq!(cleared.entries, 0);
    assert_eq!(cleared.retained_bytes, 0);
    assert_eq!(cleared.replays, before_clear.replays);
    assert_eq!(cleared.evictions, before_clear.evictions + 3);
    assert_eq!(cleared.invalidations, before_clear.invalidations + 1);
    session.render_region(0, request(), 0.75).unwrap();
    assert_eq!(session.region_cache_stats().replays, 4);
}

#[test]
fn distinct_pages_keep_their_own_exact_full_page_crops_on_warm_round_trips() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    document.sections.push(document.sections[0].clone());
    core.set_document(document);
    for (section, width) in [(0, 9000), (1, 5000)] {
        core.create_shape_control_native(
            section,
            0,
            0,
            width,
            6750,
            0,
            0,
            true,
            "Square",
            "rectangle",
            false,
            false,
            &[],
        )
        .unwrap();
    }
    assert_eq!(core.page_count(), 2);
    let session = NativeDocumentSession::from_core(core);
    let scale = 0.5;
    let references = [
        session.render_page(0, scale).unwrap(),
        session.render_page(1, scale).unwrap(),
    ];
    assert_ne!(references[0].pixels.bytes, references[1].pixels.bytes);
    for page in [0, 1, 0, 1] {
        let size = session.page_size(page).unwrap();
        let frame = session
            .render_region(
                page,
                RasterRegion {
                    x: -1.3,
                    y: -2.2,
                    width: size.width + 2.6,
                    height: size.height + 4.4,
                },
                scale,
            )
            .unwrap();
        assert_crop(
            &references[page as usize].pixels,
            &RasterRegionPixels {
                pixels: frame.pixels,
                origin_x: frame.origin_x,
                origin_y: frame.origin_y,
            },
            [255; 4],
        );
    }
    let stats = session.region_cache_stats();
    assert_eq!(stats.replays, 2);
    assert_eq!(stats.misses, 2);
    assert_eq!(stats.hits, 2);
    assert_eq!(stats.entries, 2);
    assert_eq!(
        stats.retained_bytes,
        references
            .iter()
            .map(|frame| frame.pixels.bytes.len())
            .sum::<usize>()
    );
}

#[test]
fn entry_cap_evicts_the_least_recently_used_scale_after_a_successful_hit() {
    assert_eq!(MAX_REGION_CACHE_ENTRIES, 4);
    let session = session(false);
    let scales = [0.125, 0.25, 0.375, 0.5, 0.625];
    for scale in &scales[..4] {
        session.render_region(0, request(), *scale).unwrap();
    }
    let filled = session.region_cache_stats();
    assert_eq!(filled.entries, MAX_REGION_CACHE_ENTRIES);
    assert_eq!(filled.replays, 4);
    assert_eq!(
        filled.retained_bytes,
        scales[..4]
            .iter()
            .map(|scale| page_bytes(&session, 0, *scale))
            .sum::<usize>()
    );
    session.render_region(0, request(), scales[0]).unwrap();
    session.render_region(0, request(), scales[4]).unwrap();
    let replaced = session.region_cache_stats();
    assert_eq!(replaced.entries, MAX_REGION_CACHE_ENTRIES);
    assert_eq!(replaced.evictions, 1);
    assert_eq!(replaced.replays, 5);
    assert_eq!(
        replaced.retained_bytes,
        [scales[0], scales[2], scales[3], scales[4]]
            .iter()
            .map(|scale| page_bytes(&session, 0, *scale))
            .sum::<usize>()
    );
    session.render_region(0, request(), scales[0]).unwrap();
    assert_eq!(session.region_cache_stats().hits, 2);
    session.render_region(0, request(), scales[1]).unwrap();
    assert_eq!(session.region_cache_stats().replays, 6);
    assert_eq!(session.region_cache_stats().evictions, 2);
}

#[test]
fn total_byte_cap_evicts_the_least_recently_used_page_before_the_entry_cap() {
    let session = session_with_pages(3);
    let scale = 0.5;
    let budget = page_bytes(&session, 0, scale) + page_bytes(&session, 1, scale);
    assert!(budget < DEFAULT_REGION_CACHE_BYTES);
    session.set_region_cache_limit(budget).unwrap();
    session.render_region(0, request(), scale).unwrap();
    session.render_region(1, request(), scale).unwrap();
    assert_eq!(session.region_cache_stats().retained_bytes, budget);
    session.render_region(0, request(), scale).unwrap();
    session.render_region(2, request(), scale).unwrap();
    let replaced = session.region_cache_stats();
    assert_eq!(replaced.entries, 2);
    assert!(replaced.entries < MAX_REGION_CACHE_ENTRIES);
    assert_eq!(replaced.retained_bytes, budget);
    assert_eq!(replaced.replays, 3);
    assert_eq!(replaced.evictions, 1);
    session.render_region(0, request(), scale).unwrap();
    assert_eq!(session.region_cache_stats().hits, 2);
    session.render_region(1, request(), scale).unwrap();
    assert_eq!(session.region_cache_stats().replays, 4);
    assert_eq!(session.region_cache_stats().retained_bytes, budget);
}

#[test]
fn native_cache_budget_bypass_and_disable_preserve_pixels_without_retention() {
    let session = session(false);
    let expected = session.render_region(0, request(), 0.75).unwrap().pixels;
    session.set_region_cache_limit(1).unwrap();
    for expected_bypass in 1..=2 {
        assert_eq!(
            session.render_region(0, request(), 0.75).unwrap().pixels,
            expected
        );
        let stats = session.region_cache_stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.retained_bytes, 0);
        assert_eq!(stats.bypassed_oversized, expected_bypass);
    }
    session.set_region_cache_limit(0).unwrap();
    assert_eq!(
        session.render_region(0, request(), 0.75).unwrap().pixels,
        expected
    );
    assert_eq!(session.region_cache_stats().entries, 0);
    session
        .set_region_cache_limit(DEFAULT_REGION_CACHE_BYTES)
        .unwrap();
    session.render_region(0, request(), 0.75).unwrap();
    let retained = session.region_cache_stats();
    assert_eq!(retained.entries, 1);
    assert!(matches!(
        session.set_region_cache_limit(MAX_REGION_CACHE_BYTES + 1),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert_eq!(session.region_cache_stats(), retained);
}

#[test]
fn a_page_above_the_retention_cap_evicts_the_old_entry_and_replays_on_each_request() {
    let session = session(false);
    session.render_region(0, request(), 0.75).unwrap();
    assert_eq!(session.region_cache_stats().entries, 1);
    let size = session.page_size(0).unwrap();
    let scale = 2.0;
    let page_bytes =
        (size.width * scale).ceil() as usize * (size.height * scale).ceil() as usize * 4;
    assert!(page_bytes > DEFAULT_REGION_CACHE_BYTES);
    let first = session.render_region(0, request(), scale).unwrap();
    assert_eq!(session.region_cache_stats().entries, 0);
    assert_eq!(session.region_cache_stats().retained_bytes, 0);
    assert_eq!(session.region_cache_stats().evictions, 1);
    assert_eq!(session.region_cache_stats().bypassed_oversized, 1);
    assert_eq!(
        session.render_region(0, request(), scale).unwrap().pixels,
        first.pixels
    );
    assert_eq!(session.region_cache_stats().entries, 0);
    assert_eq!(session.region_cache_stats().bypassed_oversized, 2);
    session.render_region(0, request(), 0.75).unwrap();
    assert_eq!(session.region_cache_stats().entries, 1);
    assert_eq!(session.region_cache_stats().replays, 4);
    assert_eq!(session.region_cache_stats().hits, 0);
}

#[test]
fn an_oversized_page_evicts_all_retained_pages_before_bypassing_retention() {
    let session = session(true);
    session.render_region(0, request(), 0.5).unwrap();
    session.render_region(1, request(), 0.5).unwrap();
    assert_eq!(session.region_cache_stats().entries, 2);
    assert!(page_bytes(&session, 0, 2.0) > DEFAULT_REGION_CACHE_BYTES);
    session.render_region(0, request(), 2.0).unwrap();
    let stats = session.region_cache_stats();
    assert_eq!(stats.entries, 0);
    assert_eq!(stats.retained_bytes, 0);
    assert_eq!(stats.evictions, 2);
    assert_eq!(stats.replays, 3);
    assert_eq!(stats.bypassed_oversized, 1);
}

#[test]
fn invalid_requests_and_rejected_updates_preserve_all_entries_and_lru_order() {
    let mut session = session(false);
    let scales = [0.125, 0.25, 0.375, 0.5];
    for scale in scales {
        session.render_region(0, request(), scale).unwrap();
    }
    let before = session.region_cache_stats();
    assert_eq!(before.entries, MAX_REGION_CACHE_ENTRIES);
    assert!(session
        .render_region(
            0,
            RasterRegion {
                width: 0.0,
                ..request()
            },
            scales[0]
        )
        .is_err());
    assert!(session
        .render_region(u32::MAX, request(), scales[0])
        .is_err());
    assert!(session.render_region(0, request(), f64::NAN).is_err());
    assert!(matches!(
        session.set_region_cache_limit(MAX_REGION_CACHE_BYTES + 1),
        Err(NativeHostError::LimitExceeded(_))
    ));
    assert!(session
        .set_font_sources(&["/missing/rhwp-p67-font.ttf".into()], None)
        .is_err());
    assert_eq!(session.region_cache_stats(), before);
    assert!(!session.set_font_sources(&[], None).unwrap());
    assert_eq!(session.region_cache_stats(), before);
    session
        .set_region_cache_limit(DEFAULT_REGION_CACHE_BYTES)
        .unwrap();
    assert_eq!(session.region_cache_stats(), before);
    session.render_region(0, request(), 0.625).unwrap();
    assert_eq!(session.region_cache_stats().replays, 5);
    session.render_region(0, request(), scales[1]).unwrap();
    assert_eq!(session.region_cache_stats().hits, 1);
    // The failed request for scales[0] must not have promoted the oldest key.
    session.render_region(0, request(), scales[0]).unwrap();
    assert_eq!(session.region_cache_stats().replays, 6);
}

#[test]
fn font_generation_change_invalidates_even_when_document_revision_is_unchanged() {
    let mut session = session(true);
    for page in 0..2 {
        session.render_region(page, request(), 0.5).unwrap();
        session.render_tiled_region(page, request(), 0.5).unwrap();
    }
    assert_eq!(session.region_cache_stats().entries, 2);
    assert_eq!(session.region_cache_stats().tile_entries, 2);
    let environment =
        FontEnvironment::from_json(r#"{"id":"p67-font-generation","substitutions":{}}"#).unwrap();
    assert!(session.set_font_sources(&[], Some(environment)).unwrap());
    assert_eq!(session.revision(), 0);
    assert_eq!(session.font_generation(), 1);
    assert_eq!(session.region_cache_stats().entries, 0);
    assert_eq!(session.region_cache_stats().tile_entries, 0);
    assert_eq!(session.region_cache_stats().retained_bytes, 0);
    assert_eq!(session.region_cache_stats().evictions, 2);
    assert_eq!(session.region_cache_stats().tile_evictions, 2);
    for page in 0..2 {
        let frame = session.render_region(page, request(), 0.5).unwrap();
        assert_eq!(frame.font_generation, 1);
        let tile = session.render_tiled_region(page, request(), 0.5).unwrap();
        assert_eq!(tile.font_generation, 1);
    }
    assert_eq!(session.region_cache_stats().replays, 4);
    assert_eq!(session.region_cache_stats().tile_replays, 4);
}

#[test]
fn edits_undo_and_redo_invalidate_region_snapshots_and_stamp_new_revision() {
    let session = session(true);
    for page in 0..2 {
        session.render_region(page, request(), 0.5).unwrap();
        session.render_tiled_region(page, request(), 0.5).unwrap();
    }
    assert_eq!(session.region_cache_stats().entries, 2);
    assert_eq!(session.region_cache_stats().tile_entries, 2);
    let position = NativeTextPosition {
        section: 0,
        paragraph: 0,
        scalar_offset: 0,
        cell_path: Vec::new(),
    };
    let mut editor = NativeEditor::new(
        session,
        NativeTextSelection {
            anchor: position.clone(),
            focus: position,
        },
    )
    .unwrap();
    let edited = editor.replace_selection(0, "한글").unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 0);
    assert_eq!(editor.session().region_cache_stats().tile_entries, 0);
    assert_eq!(editor.session().region_cache_stats().retained_bytes, 0);
    for page in 0..2 {
        let frame = editor
            .session()
            .render_region(page, request(), 0.5)
            .unwrap();
        assert_eq!(frame.revision, edited.revision);
        let tile = editor
            .session()
            .render_tiled_region(page, request(), 0.5)
            .unwrap();
        assert_eq!(tile.revision, edited.revision);
    }
    assert_eq!(editor.session().region_cache_stats().replays, 4);
    assert_eq!(editor.session().region_cache_stats().tile_replays, 4);
    let undone = editor.undo(edited.revision).unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 0);
    assert_eq!(editor.session().region_cache_stats().tile_entries, 0);
    assert_eq!(editor.session().region_cache_stats().retained_bytes, 0);
    for page in 0..2 {
        let frame = editor
            .session()
            .render_region(page, request(), 0.5)
            .unwrap();
        assert_eq!(frame.revision, undone.revision);
        let tile = editor
            .session()
            .render_tiled_region(page, request(), 0.5)
            .unwrap();
        assert_eq!(tile.revision, undone.revision);
    }
    assert_eq!(editor.session().region_cache_stats().replays, 6);
    assert_eq!(editor.session().region_cache_stats().tile_replays, 6);
    let redone = editor.redo(undone.revision).unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 0);
    assert_eq!(editor.session().region_cache_stats().tile_entries, 0);
    assert_eq!(editor.session().region_cache_stats().retained_bytes, 0);
    for page in 0..2 {
        let frame = editor
            .session()
            .render_region(page, request(), 0.5)
            .unwrap();
        assert_eq!(frame.revision, redone.revision);
        let tile = editor
            .session()
            .render_tiled_region(page, request(), 0.5)
            .unwrap();
        assert_eq!(tile.revision, redone.revision);
    }
    assert_eq!(editor.session().region_cache_stats().replays, 8);
    assert_eq!(editor.session().region_cache_stats().tile_replays, 8);
}

#[test]
fn composition_preview_and_cancel_invalidate_without_leaving_a_stale_cached_page() {
    let session = session(true);
    for page in 0..2 {
        session.render_region(page, request(), 0.5).unwrap();
        session.render_tiled_region(page, request(), 0.5).unwrap();
    }
    let position = NativeTextPosition {
        section: 0,
        paragraph: 0,
        scalar_offset: 0,
        cell_path: Vec::new(),
    };
    let mut editor = NativeEditor::new(
        session,
        NativeTextSelection {
            anchor: position.clone(),
            focus: position,
        },
    )
    .unwrap();
    let id = editor.begin_composition(0).unwrap().composition_id.unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 2);
    assert_eq!(editor.session().region_cache_stats().tile_entries, 2);
    let preview = editor.update_composition(0, id, "한글").unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 0);
    assert_eq!(editor.session().region_cache_stats().tile_entries, 0);
    assert_eq!(editor.session().region_cache_stats().retained_bytes, 0);
    for page in 0..2 {
        editor
            .session()
            .render_region(page, request(), 0.5)
            .unwrap();
        editor
            .session()
            .render_tiled_region(page, request(), 0.5)
            .unwrap();
    }
    assert_eq!(editor.session().region_cache_stats().replays, 4);
    assert_eq!(editor.session().region_cache_stats().tile_replays, 4);
    let cancelled = editor.cancel_composition(preview.revision, id).unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 0);
    assert_eq!(editor.session().region_cache_stats().tile_entries, 0);
    assert_eq!(editor.session().region_cache_stats().retained_bytes, 0);
    for page in 0..2 {
        let frame = editor
            .session()
            .render_region(page, request(), 0.5)
            .unwrap();
        assert_eq!(frame.revision, cancelled.revision);
        let tile = editor
            .session()
            .render_tiled_region(page, request(), 0.5)
            .unwrap();
        assert_eq!(tile.revision, cancelled.revision);
    }
    assert_eq!(editor.session().region_cache_stats().replays, 6);
    assert_eq!(editor.session().region_cache_stats().tile_replays, 6);
}

#[test]
fn off_page_requests_do_not_replay_or_replace_small_retained_pages() {
    let session = session(true);
    session.render_region(0, request(), 0.5).unwrap();
    session.render_region(1, request(), 0.5).unwrap();
    let before = session.region_cache_stats();
    let off_page = RasterRegion {
        x: -4.0,
        y: -4.0,
        width: 2.0,
        height: 2.0,
    };
    let frame = session.render_region(1, off_page, 0.75).unwrap();
    assert!(frame
        .pixels
        .bytes
        .chunks_exact(4)
        .all(|pixel| pixel == [255; 4]));
    let after = session.region_cache_stats();
    assert_eq!(after.entries, 2);
    assert_eq!(after.retained_bytes, before.retained_bytes);
    assert_eq!(after.replays, before.replays);
    assert_eq!(after.evictions, before.evictions);
    assert_eq!(after.off_page_requests, 1);
    session.render_region(0, request(), 0.5).unwrap();
    assert_eq!(session.region_cache_stats().hits, 1);
    assert_eq!(session.region_cache_stats().replays, 2);
}

fn assert_native_frame_matches(actual: &NativeFrame, expected: &NativeFrame) {
    assert_eq!(actual.pixels, expected.pixels);
    assert_eq!(actual.origin_x, expected.origin_x);
    assert_eq!(actual.origin_y, expected.origin_y);
    assert_eq!(actual.revision, expected.revision);
    assert_eq!(actual.font_generation, expected.font_generation);
}

#[test]
fn cooperative_tiled_cancellation_discards_partial_frames_and_retains_complete_tiles() {
    let session = NativeDocumentSession::new_blank().unwrap();
    let region = RasterRegion {
        x: 0.0,
        y: 0.0,
        width: 1024.0,
        height: 1024.0,
    };
    let mut boundaries = 0;
    let cancelled = session
        .render_tiled_region_cancellable(0, region, 1.0, || {
            boundaries += 1;
            boundaries < 3
        })
        .unwrap();
    assert!(cancelled.is_none(), "a partial viewport must never escape");
    assert_eq!(boundaries, 3);
    let after_cancel = session.region_cache_stats();
    assert_eq!(
        after_cancel.tile_replays, 1,
        "only the first tile completed"
    );
    assert_eq!(after_cancel.tile_entries, 1);
    let warm_region = RasterRegion {
        width: 8.0,
        height: 8.0,
        ..region
    };
    let warm = session.render_tiled_region(0, warm_region, 1.0).unwrap();
    assert_eq!((warm.pixels.width, warm.pixels.height), (8, 8));
    assert_eq!(
        session.region_cache_stats().tile_replays,
        after_cancel.tile_replays
    );
    assert_eq!(
        session.region_cache_stats().tile_hits,
        after_cancel.tile_hits + 1
    );
    let completed = session.render_tiled_region(0, region, 1.0).unwrap();
    let reference = NativeDocumentSession::new_blank()
        .unwrap()
        .render_tiled_region(0, region, 1.0)
        .unwrap();
    assert_eq!(completed.pixels, reference.pixels);
    assert_eq!(session.region_cache_stats().tile_replays, 4);

    // Cancellation at the final boundary also drops a completely copied buffer.
    // The completed immutable tile stays available, while no frame is returned.
    let mut boundaries = 0;
    assert!(session
        .render_tiled_region_cancellable(0, warm_region, 1.0, || {
            boundaries += 1;
            boundaries < 3
        })
        .unwrap()
        .is_none());
    assert_eq!(boundaries, 3);
    assert_eq!(session.region_cache_stats().tile_replays, 4);
    let mut boundaries = 0;
    assert!(session
        .render_tiled_region_cancellable(
            0,
            RasterRegion {
                x: -100.0,
                y: -100.0,
                ..warm_region
            },
            1.0,
            || {
                boundaries += 1;
                boundaries < 2
            }
        )
        .unwrap()
        .is_none());
    assert_eq!(
        boundaries, 2,
        "off-page windows still poll before returning output"
    );
    assert_eq!(session.region_cache_stats().tile_replays, 4);
}

#[test]
fn native_tiled_warm_frames_keep_pixels_and_skip_replay() {
    let cached = session(false);
    let uncached = session(false);
    uncached.set_region_cache_limit(0).unwrap();
    let region = RasterRegion {
        x: -1.3,
        y: -2.2,
        width: 350.4,
        height: 180.1,
    };
    let reference = uncached.render_tiled_region(0, region, 1.25).unwrap();
    let cold = cached.render_tiled_region(0, region, 1.25).unwrap();
    assert_native_frame_matches(&cold, &reference);
    let after_cold = cached.region_cache_stats();
    assert_eq!(after_cold.entries, 0);
    assert_eq!(after_cold.tile_entries, 1);
    assert_eq!(after_cold.tile_misses, 1);
    assert_eq!(after_cold.tile_replays, 1);
    assert_eq!(after_cold.tile_hits, 0);
    assert_eq!(after_cold.retained_bytes, after_cold.tile_retained_bytes);
    for expected_hits in 1..=3 {
        let warm = cached.render_tiled_region(0, region, 1.25).unwrap();
        assert_native_frame_matches(&warm, &reference);
        let stats = cached.region_cache_stats();
        assert_eq!(stats.tile_entries, 1);
        assert_eq!(stats.tile_hits, expected_hits);
        assert_eq!(stats.tile_replays, after_cold.tile_replays);
        assert_eq!(stats.tile_misses, after_cold.tile_misses);
        assert_eq!(stats.retained_bytes, after_cold.retained_bytes);
        assert_eq!(stats.replays, 0);
    }
    cached.clear_caches();
    assert_eq!(cached.region_cache_stats().tile_entries, 0);
    assert_eq!(cached.region_cache_stats().retained_bytes, 0);
    // The caller's earlier frame owns its bytes after its source tile is gone.
    assert_native_frame_matches(&cold, &reference);
}

#[test]
fn overlapping_native_tiled_windows_replay_only_new_grid_tiles() {
    let cached = session(false);
    let uncached = session(false);
    uncached.set_region_cache_limit(0).unwrap();
    let first_region = RasterRegion {
        x: 250.0,
        y: 10.0,
        width: 60.0,
        height: 30.0,
    };
    let second_region = RasterRegion {
        x: 300.0,
        width: 250.0,
        ..first_region
    };
    // At scale 2 the first window touches columns 0/1; the second touches 1/2.
    let first = cached.render_tiled_region(0, first_region, 2.0).unwrap();
    assert_native_frame_matches(
        &first,
        &uncached.render_tiled_region(0, first_region, 2.0).unwrap(),
    );
    let after_first = cached.region_cache_stats();
    assert_eq!(after_first.tile_replays, 2);
    assert_eq!(after_first.tile_misses, 2);
    let second = cached.render_tiled_region(0, second_region, 2.0).unwrap();
    assert_native_frame_matches(
        &second,
        &uncached.render_tiled_region(0, second_region, 2.0).unwrap(),
    );
    let after_second = cached.region_cache_stats();
    assert_eq!(after_second.tile_entries, 3);
    assert_eq!(after_second.tile_replays, 3);
    assert_eq!(after_second.tile_misses, 3);
    assert_eq!(after_second.tile_hits, 1);
    assert_native_frame_matches(
        &cached.render_tiled_region(0, first_region, 2.0).unwrap(),
        &first,
    );
    let after_return = cached.region_cache_stats();
    assert_eq!(after_return.tile_replays, 3);
    assert_eq!(after_return.tile_hits, 3);
}

#[test]
fn native_page_and_tile_snapshots_share_one_retained_byte_budget() {
    let cached = session(false);
    let scale = 0.5;
    let budget = page_bytes(&cached, 0, scale);
    cached.set_region_cache_limit(budget).unwrap();
    let full = cached.render_region(0, request(), scale).unwrap();
    assert_eq!(cached.region_cache_stats().retained_bytes, budget);
    let tiled = cached.render_tiled_region(0, request(), scale).unwrap();
    let after_tile = cached.region_cache_stats();
    assert_eq!(after_tile.entries, 0);
    assert_eq!(after_tile.tile_entries, 1);
    assert_eq!(after_tile.evictions, 1);
    assert_eq!(after_tile.retained_bytes, after_tile.tile_retained_bytes);
    assert!(after_tile.retained_bytes <= budget);
    assert!(after_tile.retained_bytes > 0);
    assert_native_frame_matches(
        &cached.render_tiled_region(0, request(), scale).unwrap(),
        &tiled,
    );
    assert_eq!(cached.region_cache_stats().tile_hits, 1);
    assert_native_frame_matches(&cached.render_region(0, request(), scale).unwrap(), &full);
    let after_page = cached.region_cache_stats();
    assert_eq!(after_page.entries, 1);
    assert_eq!(after_page.tile_entries, 0);
    assert_eq!(after_page.tile_retained_bytes, 0);
    assert_eq!(after_page.tile_evictions, 1);
    assert_eq!(after_page.retained_bytes, budget);
    assert_eq!(after_page.replays, 2);
    assert_eq!(after_page.tile_replays, 1);
}

#[test]
fn native_tile_byte_limit_evicts_the_least_recently_used_tile() {
    let cached = session(false);
    let budget = 2
        * SkiaTilePlan::MAX_WORKING_DIMENSION as usize
        * SkiaTilePlan::MAX_WORKING_DIMENSION as usize
        * 4;
    cached.set_region_cache_limit(budget).unwrap();
    let regions = [10.0, 265.0, 520.0].map(|x| RasterRegion {
        x,
        y: 10.0,
        width: 5.0,
        height: 5.0,
    });
    for region in &regions[..2] {
        cached.render_tiled_region(0, *region, 2.0).unwrap();
    }
    assert_eq!(cached.region_cache_stats().tile_entries, 2);
    cached.render_tiled_region(0, regions[0], 2.0).unwrap();
    cached.render_tiled_region(0, regions[2], 2.0).unwrap();
    let replaced = cached.region_cache_stats();
    assert_eq!(replaced.tile_entries, 2);
    assert_eq!(replaced.tile_replays, 3);
    assert_eq!(replaced.tile_hits, 1);
    assert_eq!(replaced.tile_evictions, 1);
    assert!(replaced.retained_bytes <= budget);
    cached.render_tiled_region(0, regions[0], 2.0).unwrap();
    assert_eq!(cached.region_cache_stats().tile_hits, 2);
    cached.render_tiled_region(0, regions[1], 2.0).unwrap();
    let returned = cached.region_cache_stats();
    assert_eq!(returned.tile_replays, 4);
    assert_eq!(returned.tile_evictions, 2);
    assert_eq!(returned.tile_entries, 2);
    assert!(returned.retained_bytes <= budget);
}

#[test]
fn native_tile_disable_and_oversized_bypass_keep_uncached_pixels() {
    let cached = session(false);
    let uncached = session(false);
    uncached.set_region_cache_limit(0).unwrap();
    let expected = uncached.render_tiled_region(0, request(), 0.5).unwrap();
    for budget in [0, 1] {
        cached.set_region_cache_limit(budget).unwrap();
        let before = cached.region_cache_stats();
        for _ in 0..2 {
            assert_native_frame_matches(
                &cached.render_tiled_region(0, request(), 0.5).unwrap(),
                &expected,
            );
        }
        let after = cached.region_cache_stats();
        assert_eq!(after.entries, 0);
        assert_eq!(after.tile_entries, 0);
        assert_eq!(after.retained_bytes, 0);
        assert_eq!(after.tile_retained_bytes, 0);
        assert_eq!(after.tile_replays, before.tile_replays + 2);
        assert_eq!(
            after.tile_bypassed_oversized,
            before.tile_bypassed_oversized + 2
        );
        assert_eq!(after.tile_hits, before.tile_hits);
    }
}

#[test]
fn rejected_native_tiled_requests_preserve_snapshots_counters_and_recency() {
    let cached = session(false);
    let budget = 2
        * SkiaTilePlan::MAX_WORKING_DIMENSION as usize
        * SkiaTilePlan::MAX_WORKING_DIMENSION as usize
        * 4;
    cached.set_region_cache_limit(budget).unwrap();
    let regions = [10.0, 265.0, 520.0].map(|x| RasterRegion {
        x,
        y: 10.0,
        width: 5.0,
        height: 5.0,
    });
    for region in &regions[..2] {
        cached.render_tiled_region(0, *region, 2.0).unwrap();
    }
    let before = cached.region_cache_stats();
    for region in [
        RasterRegion {
            width: 0.0,
            ..regions[0]
        },
        RasterRegion {
            x: f64::NAN,
            ..regions[0]
        },
        RasterRegion {
            width: 1_000_000.0,
            ..regions[0]
        },
    ] {
        assert!(cached.render_tiled_region(0, region, 2.0).is_err());
    }
    assert!(cached
        .render_tiled_region(u32::MAX, regions[0], 2.0)
        .is_err());
    assert!(cached.render_tiled_region(0, regions[0], f64::NAN).is_err());
    assert!(cached
        .set_region_cache_limit(MAX_REGION_CACHE_BYTES + 1)
        .is_err());
    assert_eq!(cached.region_cache_stats(), before);
    cached.render_tiled_region(0, regions[2], 2.0).unwrap();
    cached.render_tiled_region(0, regions[1], 2.0).unwrap();
    assert_eq!(cached.region_cache_stats().tile_hits, 1);
    assert_eq!(cached.region_cache_stats().tile_replays, 3);
    // Failed requests mentioning tile 0 must not have promoted it above tile 1.
    cached.render_tiled_region(0, regions[0], 2.0).unwrap();
    assert_eq!(cached.region_cache_stats().tile_replays, 4);
}

#[test]
fn native_tiled_off_page_clear_preserves_retained_snapshots_without_replay() {
    let cached = session(true);
    cached.render_region(0, request(), 0.5).unwrap();
    cached.render_tiled_region(0, request(), 1.25).unwrap();
    let before = cached.region_cache_stats();
    let clear = cached
        .render_tiled_region(
            1,
            RasterRegion {
                x: -4.0,
                y: -4.0,
                width: 2.0,
                height: 2.0,
            },
            1.25,
        )
        .unwrap();
    assert!(clear.pixels.bytes.chunks_exact(4).all(|p| p == [255; 4]));
    let after = cached.region_cache_stats();
    assert_eq!(after.entries, before.entries);
    assert_eq!(after.tile_entries, before.tile_entries);
    assert_eq!(after.retained_bytes, before.retained_bytes);
    assert_eq!(after.tile_replays, before.tile_replays);
    assert_eq!(after.tile_hits, before.tile_hits);
    assert_eq!(after.tile_misses, before.tile_misses);
    assert_eq!(after.evictions, before.evictions);
    assert_eq!(after.tile_evictions, before.tile_evictions);
    cached.render_tiled_region(0, request(), 1.25).unwrap();
    assert_eq!(cached.region_cache_stats().tile_hits, before.tile_hits + 1);
}

#[test]
fn native_tiles_keep_page_and_exact_scale_keys_separate() {
    // This cache-key contract needs visibly different pages, independent of
    // installed fonts. Supply the repository's MIT-licensed synthetic face.
    let fixture_session = || {
        let mut session = session(true);
        let substitutions: std::collections::BTreeMap<_, _> = session
            .document()
            .document()
            .doc_info
            .font_faces
            .iter()
            .flatten()
            .map(|font| (font.name.clone(), "RHWP Host Fixture".to_owned()))
            .collect();
        let environment = FontEnvironment::from_json(
            &serde_json::json!({
                "id": "native-tile-key-fixture",
                "substitutions": substitutions,
            })
            .to_string(),
        )
        .unwrap();
        let font = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/fonts/RHWPHostFixture-Regular.ttf");
        session
            .set_font_sources(&[font], Some(environment))
            .unwrap();
        session
    };
    let cached = fixture_session();
    let uncached = fixture_session();
    uncached.set_region_cache_limit(0).unwrap();
    let region = RasterRegion {
        x: 30.0,
        y: 30.0,
        width: 300.0,
        height: 130.0,
    };
    let page_zero = cached.render_tiled_region(0, region, 1.25).unwrap();
    let page_one = cached.render_tiled_region(1, region, 1.25).unwrap();
    assert_ne!(page_zero.pixels.bytes, page_one.pixels.bytes);
    for (page, scale) in [(0, 1.5), (0, 1.25), (1, 1.25)] {
        assert_native_frame_matches(
            &cached.render_tiled_region(page, region, scale).unwrap(),
            &uncached.render_tiled_region(page, region, scale).unwrap(),
        );
    }
    let stats = cached.region_cache_stats();
    assert_eq!(stats.entries, 0);
    assert_eq!(stats.tile_entries, 3);
    assert_eq!(stats.tile_replays, 3);
    assert_eq!(stats.tile_misses, 3);
    assert_eq!(stats.tile_hits, 2);
}

#[test]
fn native_page_entry_cap_counts_pages_separately_from_retained_tiles() {
    let cached = session(false);
    cached.render_tiled_region(0, request(), 1.0).unwrap();
    let scales = [0.125, 0.25, 0.375, 0.5, 0.625];
    for scale in &scales[..4] {
        cached.render_region(0, request(), *scale).unwrap();
    }
    let filled = cached.region_cache_stats();
    assert_eq!(filled.entries, MAX_REGION_CACHE_ENTRIES);
    assert_eq!(filled.tile_entries, 1);
    assert_eq!(filled.evictions, 0);
    assert_eq!(filled.tile_evictions, 0);
    cached.render_region(0, request(), scales[0]).unwrap();
    cached.render_region(0, request(), scales[4]).unwrap();
    let replaced = cached.region_cache_stats();
    assert_eq!(replaced.entries, MAX_REGION_CACHE_ENTRIES);
    assert_eq!(replaced.tile_entries, 1);
    assert_eq!(replaced.evictions, 1);
    assert_eq!(replaced.tile_evictions, 0);
    assert_eq!(replaced.replays, 5);
    // The globally oldest entry is the tile, but the page-count limit must
    // remove the least recent page rather than evicting that different kind.
    cached.render_tiled_region(0, request(), 1.0).unwrap();
    assert_eq!(cached.region_cache_stats().tile_hits, 1);
    cached.render_region(0, request(), scales[0]).unwrap();
    assert_eq!(cached.region_cache_stats().hits, 2);
    cached.render_region(0, request(), scales[1]).unwrap();
    assert_eq!(cached.region_cache_stats().replays, 6);
    assert_eq!(cached.region_cache_stats().evictions, 2);
    assert_eq!(cached.region_cache_stats().tile_evictions, 0);
}

#[test]
fn native_thin_page_tile_metadata_cap_evicts_even_below_the_byte_budget() {
    assert_eq!(MAX_TILE_CACHE_ENTRIES, 64);
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    // 75 HWPUNIT per CSS pixel at 96 dpi. A one-pixel-wide blank page makes
    // 65 distinct grid tiles tiny, so entry count rather than bytes must evict.
    let height = (MAX_TILE_CACHE_ENTRIES as u32 + 1) * SkiaTilePlan::TILE_SIZE as u32 + 1;
    document.sections[0].section_def.page_def = rhwp::model::page::PageDef {
        width: 75,
        height: height * 75,
        ..Default::default()
    };
    core.set_document(document);
    let cached = NativeDocumentSession::from_core(core);
    let region = |index: usize| RasterRegion {
        x: 0.0,
        y: index as f64 * f64::from(SkiaTilePlan::TILE_SIZE) + 8.0,
        width: 1.0,
        height: 1.0,
    };
    for index in 0..MAX_TILE_CACHE_ENTRIES {
        cached.render_tiled_region(0, region(index), 1.0).unwrap();
    }
    let filled = cached.region_cache_stats();
    assert_eq!(filled.tile_entries, MAX_TILE_CACHE_ENTRIES);
    assert_eq!(filled.tile_replays, MAX_TILE_CACHE_ENTRIES as u64);
    assert_eq!(filled.tile_evictions, 0);
    assert!(filled.retained_bytes < DEFAULT_REGION_CACHE_BYTES);
    cached.render_tiled_region(0, region(0), 1.0).unwrap();
    cached
        .render_tiled_region(0, region(MAX_TILE_CACHE_ENTRIES), 1.0)
        .unwrap();
    let replaced = cached.region_cache_stats();
    assert_eq!(replaced.tile_entries, MAX_TILE_CACHE_ENTRIES);
    assert_eq!(replaced.tile_replays, MAX_TILE_CACHE_ENTRIES as u64 + 1);
    assert_eq!(replaced.tile_evictions, 1);
    assert!(replaced.retained_bytes < DEFAULT_REGION_CACHE_BYTES);
    cached.render_tiled_region(0, region(0), 1.0).unwrap();
    assert_eq!(cached.region_cache_stats().tile_hits, 2);
    cached.render_tiled_region(0, region(1), 1.0).unwrap();
    let returned = cached.region_cache_stats();
    assert_eq!(returned.tile_replays, MAX_TILE_CACHE_ENTRIES as u64 + 2);
    assert_eq!(returned.tile_evictions, 2);
    assert_eq!(returned.tile_entries, MAX_TILE_CACHE_ENTRIES);
}

// This existing Native CI case also exercises the worker transport; keep it
// inline so the test-suite collector does not create another integration target.
mod runtime_contracts {
    use rhwp::native_host::runtime::{
        NativeRuntime, NativeRuntimeCommand as Command, NativeRuntimeCompletion,
        NativeRuntimeConfig, NativeRuntimeError, NativeRuntimeInfo, NativeRuntimeOutput as Output,
        NativeRuntimeRegion, NativeRuntimeTarget,
    };
    use std::thread;
    use std::time::{Duration, Instant};

    fn runtime() -> NativeRuntime {
        NativeRuntime::spawn(NativeRuntimeConfig::default()).unwrap()
    }
    fn next(runtime: &mut NativeRuntime) -> NativeRuntimeCompletion {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(completion) = runtime.try_recv().unwrap() {
                return completion;
            }
            assert!(Instant::now() < deadline, "worker completion deadline");
            thread::sleep(Duration::from_millis(2));
        }
    }
    fn run(runtime: &mut NativeRuntime, command: Command) -> Output {
        let id = runtime.try_submit(command).unwrap();
        let completion = next(runtime);
        assert_eq!(completion.request_id, id);
        completion.result.unwrap()
    }
    fn new(runtime: &mut NativeRuntime) -> NativeRuntimeInfo {
        let Output::Info(info) = run(runtime, Command::New { replace: None }) else {
            panic!("expected document info");
        };
        info
    }
    fn render(target: NativeRuntimeTarget) -> Command {
        Command::RenderRegion {
            target,
            font_generation: 0,
            page: 0,
            region: NativeRuntimeRegion {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 8.0,
            },
            scale: 1.0,
        }
    }
    fn tiled(target: NativeRuntimeTarget) -> Command {
        Command::RenderTiledRegion {
            target,
            font_generation: 0,
            page: 0,
            region: NativeRuntimeRegion {
                x: -0.1,
                y: 1.25,
                width: 1.2,
                height: 2.5,
            },
            scale: 8.0,
        }
    }

    #[test]
    fn tiled_runtime_uses_large_page_tiles_and_exposes_combined_raster_cache() {
        let mut runtime = runtime();
        let target = new(&mut runtime).target();
        run(
            &mut runtime,
            Command::SetRasterCacheLimit {
                target,
                max_bytes: 8 * 1024 * 1024,
            },
        );
        let Output::Frame { frame, .. } = run(&mut runtime, tiled(target)) else {
            panic!("expected tiled frame");
        };
        assert_eq!((frame.origin_x, frame.origin_y), (-1, 10));
        assert_eq!((frame.pixels.width, frame.pixels.height), (10, 20));
        assert_eq!(frame.pixels.stride, 40);
        assert_eq!(frame.pixels.bytes.len(), 800);
        assert_eq!(frame.revision, target.revision);
        let Output::RasterCacheStats { stats: cold, .. } =
            run(&mut runtime, Command::RasterCacheStats { target })
        else {
            panic!("raster stats");
        };
        assert!(cold.tile_replays > 0);
        assert!(cold.tile_entries > 0);
        let Output::Frame { frame: warm, .. } = run(&mut runtime, tiled(target)) else {
            panic!("warm tiled frame");
        };
        assert_eq!(frame.pixels, warm.pixels);
        let Output::RasterCacheStats { stats: warm, .. } =
            run(&mut runtime, Command::RasterCacheStats { target })
        else {
            panic!("raster stats");
        };
        assert_eq!(warm.tile_replays, cold.tile_replays);
        assert!(warm.tile_hits > cold.tile_hits);
        assert!(warm.retained_bytes <= 8 * 1024 * 1024);

        // Exact crop still needs its independently bounded full-page raster.
        let mut exact = render(target);
        if let Command::RenderRegion { scale, .. } = &mut exact {
            *scale = 8.0;
        }
        runtime.try_submit(exact).unwrap();
        assert!(next(&mut runtime).result.is_err());
        run(
            &mut runtime,
            Command::SetRasterCacheLimit {
                target,
                max_bytes: 0,
            },
        );
        let Output::RasterCacheStats { stats, .. } =
            run(&mut runtime, Command::RasterCacheStats { target })
        else {
            panic!("raster stats");
        };
        assert_eq!(stats.retained_bytes, 0);
        assert_eq!(stats.entries + stats.tile_entries, 0);
        runtime
            .try_submit(Command::SetRasterCacheLimit {
                target,
                max_bytes: 64 * 1024 * 1024 + 1,
            })
            .unwrap();
        assert_eq!(next(&mut runtime).result.unwrap_err().code, "limitExceeded");
    }

    #[test]
    fn worker_output_budget_matches_actual_exact_and_tiled_frame_geometry() {
        for use_tiles in [false, true] {
            let mut runtime = NativeRuntime::spawn(NativeRuntimeConfig {
                max_outstanding: 4,
                max_pending_bytes: 80,
            })
            .unwrap();
            let target = new(&mut runtime).target();
            let make_command = || {
                let region = NativeRuntimeRegion {
                    x: -0.3,
                    y: 0.2,
                    width: 3.4,
                    height: 2.9,
                };
                if use_tiles {
                    Command::RenderTiledRegion {
                        target,
                        font_generation: 0,
                        page: 0,
                        region,
                        scale: 1.25,
                    }
                } else {
                    Command::RenderRegion {
                        target,
                        font_generation: 0,
                        page: 0,
                        region,
                        scale: 1.25,
                    }
                }
            };
            let first = runtime.try_submit(make_command()).unwrap();
            assert_eq!(
                runtime.try_submit(make_command()).unwrap_err().error,
                NativeRuntimeError::ByteCapacity
            );
            let mut invalid = make_command();
            match &mut invalid {
                Command::RenderRegion { region, .. }
                | Command::RenderTiledRegion { region, .. } => region.width = -1.0,
                _ => unreachable!(),
            }
            assert_eq!(
                runtime.try_submit(invalid).unwrap_err().error,
                NativeRuntimeError::InvalidArgument
            );
            let completion = next(&mut runtime);
            assert_eq!(completion.request_id, first);
            let Output::Frame { frame, .. } = completion.result.unwrap() else {
                panic!("rejected request superseded a valid frame");
            };
            assert_eq!((frame.origin_x, frame.origin_y), (-1, 0));
            assert_eq!((frame.pixels.width, frame.pixels.height), (5, 4));
            assert_eq!((frame.pixels.stride, frame.pixels.bytes.len()), (20, 80));
            let second = runtime.try_submit(make_command()).unwrap();
            let completion = next(&mut runtime);
            assert_eq!(completion.request_id, second);
            assert!(matches!(completion.result.unwrap(), Output::Frame { .. }));
        }
    }
}
