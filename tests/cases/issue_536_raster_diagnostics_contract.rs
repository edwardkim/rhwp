use rhwp::paint::{LayerNode, PageLayerTree};
use rhwp::render_backend::{RenderBackend, SkiaBackend};
use rhwp::renderer::layer_renderer::{
    LayerRasterRenderer, LayerRenderDiagnostics, LayerRenderResult, RasterColorSpace,
    RasterOutputFormat, RasterRenderOptions, RasterRenderOutput, TextVariantSelectionReport,
    VariantSelectedReason, VariantSelectionBackend,
};
use rhwp::renderer::render_tree::BoundingBox;

#[test]
fn raster_diagnostics_default_is_an_unmeasured_empty_snapshot() {
    assert_eq!(
        LayerRenderDiagnostics::default(),
        LayerRenderDiagnostics {
            backend: None,
            render_profile: None,
            variant_selections: Vec::new(),
            shaped_measurements: Vec::new(),
            shaped_measurement_lines: Vec::new(),
            line_break_shadows: Vec::new(),
            shaped_measurement_paragraphs: Vec::new(),
            shaped_measurement_pages: Vec::new(),
            layer_nodes_replayed: 0,
            paint_ops_replayed: 0,
            tile_fallback_cap_hits: 0,
            image_effect_preprocess_failures: 0,
            image_effect_fallback_to_filter: 0,
            image_effect_cache_hits: 0,
            image_effect_cache_misses: 0,
            image_effect_cache_evictions: 0,
            image_effect_cache_skipped_oversized: 0,
            static_picture_cache_hits: 0,
            static_picture_cache_misses: 0,
            static_picture_cache_evictions: 0,
            static_picture_cache_skipped_oversized: 0,
            static_picture_cache_fingerprint_mismatches: 0,
            static_picture_cache_recordings: 0,
            raster_setup_time_ns: 0,
            raster_replay_time_ns: 0,
            raster_encode_time_ns: 0,
            raster_total_time_ns: 0,
            image_effect_preprocessed_bytes: 0,
            image_effect_cache_approx_bytes: 0,
            static_picture_cache_approx_bytes: 0,
        },
    );
}

struct DiagnosticsRenderer;

impl LayerRasterRenderer for DiagnosticsRenderer {
    fn render_raster(
        &self,
        _tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRenderOutput> {
        Ok(RasterRenderOutput {
            bytes: vec![1, 2, 3],
            format: options.format,
            width: 1,
            height: 1,
            dpi: options.dpi,
            color_space: options.color_space,
            diagnostics: LayerRenderDiagnostics {
                backend: Some(VariantSelectionBackend::NativeSkia),
                render_profile: Some("screen".to_string()),
                variant_selections: vec![TextVariantSelectionReport {
                    backend: VariantSelectionBackend::NativeSkia,
                    equivalence_group: "source0".to_string(),
                    selected_variant_id: None,
                    selected_variant_kind: None,
                    selected_reason: VariantSelectedReason::DefaultTextRunFallback,
                    fallback_required: true,
                    rejected_variants: Vec::new(),
                }],
                layer_nodes_replayed: 1,
                paint_ops_replayed: 2,
                raster_setup_time_ns: 5,
                raster_replay_time_ns: 7,
                raster_encode_time_ns: 11,
                raster_total_time_ns: 23,
                ..LayerRenderDiagnostics::default()
            },
        })
    }
}

#[test]
fn raster_output_keeps_diagnostics_while_png_convenience_methods_return_only_bytes() {
    let bbox = BoundingBox::new(0.0, 0.0, 1.0, 1.0);
    let tree = PageLayerTree::new(1.0, 1.0, LayerNode::leaf(bbox, None, Vec::new()));
    let renderer = DiagnosticsRenderer;
    let options = RasterRenderOptions {
        dpi: Some(144.0),
        ..RasterRenderOptions::default()
    };
    let output = renderer.render_raster(&tree, options).unwrap();
    assert_eq!(output.format, RasterOutputFormat::Png);
    assert_eq!(output.color_space, RasterColorSpace::Srgb);
    assert_eq!(output.dpi, Some(144.0));
    assert_eq!(output.diagnostics.raster_total_time_ns, 23);
    assert_eq!(output.diagnostics.variant_selections.len(), 1);
    assert_eq!(output, output.clone());
    let mut changed = output.clone();
    changed.diagnostics.paint_ops_replayed = 3;
    assert_ne!(output, changed);
    assert_eq!(renderer.render_png(&tree).unwrap(), output.bytes);
    assert_eq!(
        renderer.render_png_with_options(&tree, options).unwrap(),
        output.bytes,
    );
}

#[test]
fn empty_skia_adapter_does_not_claim_a_measured_native_render() {
    let output = SkiaBackend::new().finish().unwrap();
    assert!(output.bytes.is_empty());
    assert_eq!((output.width, output.height), (0, 0));
    assert_eq!(output.diagnostics, LayerRenderDiagnostics::default());
}
