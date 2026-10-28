//! Region cache checks use the same direct full-page Skia raster as reference.
//! They prove immutable crop equivalence and bounded replay reuse, not Hancom
//! visual parity, OS presentation performance or subtree culling.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use rhwp::document_core::native_interaction::{NativeTextPosition, NativeTextSelection};
use rhwp::native_host::editor::NativeEditor;
use rhwp::native_host::region_cache::{DEFAULT_REGION_CACHE_BYTES, MAX_REGION_CACHE_BYTES};
use rhwp::native_host::{NativeDocumentSession, NativeHostError};
use rhwp::paint::{ClipKind, LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::renderer::layer_renderer::RasterRenderOptions;
use rhwp::renderer::raster_pixels::{RasterPixels, RasterRegion, RasterRegionPixels};
use rhwp::renderer::render_tree::{BoundingBox, EllipseNode, RectangleNode};
use rhwp::renderer::skia::{SkiaLayerRenderer, SkiaRegionPlan};
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
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "Skia 가😀").unwrap();
    if two_pages {
        let mut document = core.document().clone();
        document.sections.push(document.sections[0].clone());
        core.set_document(document);
        assert!(core.page_count() >= 2);
    }
    NativeDocumentSession::from_core(core)
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
fn native_page_scale_and_explicit_clear_replace_or_invalidate_the_one_entry() {
    let session = session(true);
    session.render_region(0, request(), 0.5).unwrap();
    session.render_region(1, request(), 0.5).unwrap();
    assert_eq!(session.region_cache_stats().replays, 2);
    assert_eq!(session.region_cache_stats().evictions, 1);
    session.render_region(0, request(), 0.5).unwrap();
    session.render_region(0, request(), 0.75).unwrap();
    assert_eq!(session.region_cache_stats().replays, 4);
    assert_eq!(session.region_cache_stats().evictions, 3);
    let before_clear = session.region_cache_stats();
    session.clear_caches();
    let cleared = session.region_cache_stats();
    assert_eq!(cleared.entries, 0);
    assert_eq!(cleared.retained_bytes, 0);
    assert_eq!(cleared.replays, before_clear.replays);
    assert_eq!(cleared.invalidations, before_clear.invalidations + 1);
    session.render_region(0, request(), 0.75).unwrap();
    assert_eq!(session.region_cache_stats().replays, 5);
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
fn invalid_regions_and_rejected_font_updates_leave_the_retained_frame_usable() {
    let mut session = session(false);
    let first = session.render_region(0, request(), 0.75).unwrap().pixels;
    let before = session.region_cache_stats();
    assert!(session
        .render_region(
            0,
            RasterRegion {
                width: 0.0,
                ..request()
            },
            0.75
        )
        .is_err());
    assert!(session.render_region(u32::MAX, request(), 0.75).is_err());
    assert!(session.render_region(0, request(), f64::NAN).is_err());
    assert!(session
        .set_font_sources(&["/missing/rhwp-p67-font.ttf".into()], None)
        .is_err());
    assert_eq!(session.region_cache_stats(), before);
    assert!(!session.set_font_sources(&[], None).unwrap());
    assert_eq!(session.region_cache_stats(), before);
    assert_eq!(
        session.render_region(0, request(), 0.75).unwrap().pixels,
        first
    );
    assert_eq!(session.region_cache_stats().replays, 1);
    assert_eq!(session.region_cache_stats().hits, 1);
}

#[test]
fn font_generation_change_invalidates_even_when_document_revision_is_unchanged() {
    let mut session = session(false);
    session.render_region(0, request(), 0.75).unwrap();
    let environment =
        FontEnvironment::from_json(r#"{"id":"p67-font-generation","substitutions":{}}"#).unwrap();
    assert!(session.set_font_sources(&[], Some(environment)).unwrap());
    assert_eq!(session.revision(), 0);
    assert_eq!(session.font_generation(), 1);
    assert_eq!(session.region_cache_stats().entries, 0);
    let frame = session.render_region(0, request(), 0.75).unwrap();
    assert_eq!(frame.font_generation, 1);
    assert_eq!(session.region_cache_stats().replays, 2);
}

#[test]
fn edits_undo_and_redo_invalidate_region_snapshots_and_stamp_new_revision() {
    let session = session(false);
    session.render_region(0, request(), 0.75).unwrap();
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
    let frame = editor.session().render_region(0, request(), 0.75).unwrap();
    assert_eq!(frame.revision, edited.revision);
    assert_eq!(editor.session().region_cache_stats().replays, 2);
    let undone = editor.undo(edited.revision).unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 0);
    let frame = editor.session().render_region(0, request(), 0.75).unwrap();
    assert_eq!(frame.revision, undone.revision);
    let redone = editor.redo(undone.revision).unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 0);
    let frame = editor.session().render_region(0, request(), 0.75).unwrap();
    assert_eq!(frame.revision, redone.revision);
    assert_eq!(editor.session().region_cache_stats().replays, 4);
}

#[test]
fn composition_preview_and_cancel_invalidate_without_leaving_a_stale_cached_page() {
    let session = session(false);
    session.render_region(0, request(), 0.75).unwrap();
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
    assert_eq!(editor.session().region_cache_stats().entries, 1);
    let preview = editor.update_composition(0, id, "한글").unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 0);
    editor.session().render_region(0, request(), 0.75).unwrap();
    assert_eq!(editor.session().region_cache_stats().replays, 2);
    let cancelled = editor.cancel_composition(preview.revision, id).unwrap();
    assert_eq!(editor.session().region_cache_stats().entries, 0);
    let frame = editor.session().render_region(0, request(), 0.75).unwrap();
    assert_eq!(frame.revision, cancelled.revision);
    assert_eq!(editor.session().region_cache_stats().replays, 3);
}

#[test]
fn off_page_requests_do_not_replay_or_replace_a_small_retained_page() {
    let session = session(true);
    session.render_region(0, request(), 0.5).unwrap();
    let before = session.region_cache_stats();
    let off_page = RasterRegion {
        x: -4.0,
        y: -4.0,
        width: 2.0,
        height: 2.0,
    };
    let frame = session.render_region(1, off_page, 0.5).unwrap();
    assert!(frame
        .pixels
        .bytes
        .chunks_exact(4)
        .all(|pixel| pixel == [255; 4]));
    let after = session.region_cache_stats();
    assert_eq!(after.entries, 1);
    assert_eq!(after.retained_bytes, before.retained_bytes);
    assert_eq!(after.replays, before.replays);
    assert_eq!(after.evictions, before.evictions);
    assert_eq!(after.off_page_requests, 1);
    session.render_region(0, request(), 0.5).unwrap();
    assert_eq!(session.region_cache_stats().hits, 1);
    assert_eq!(session.region_cache_stats().replays, 1);
}
