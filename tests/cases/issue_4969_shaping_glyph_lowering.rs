//! Issue #4969 W10-Q2-D3: page sidecars lower losslessly but remain dormant.

#[path = "../../src/renderer/kerning.rs"]
mod kerning;
#[path = "../../src/renderer/shaping.rs"]
mod shaping;
#[path = "../../src/renderer/shaping_composition.rs"]
mod shaping_composition;
#[path = "../../src/renderer/shaping_context.rs"]
mod shaping_context;
#[path = "../../src/renderer/shaping_paragraph.rs"]
mod shaping_paragraph;
#[path = "../../src/renderer/shaping_publication.rs"]
mod shaping_publication;

// The product implementation stays crate-private. This narrow wrapper lets the
// source integration case compile that exact module without widening rhwp's API.
mod paint {
    pub use rhwp::paint::*;

    pub(crate) const MAX_PORTABLE_FONT_BLOB_BYTES: usize = 32 * 1024 * 1024;
    pub(crate) const MAX_PORTABLE_GLYPHS_PER_RUN: usize = 4096;
}

mod renderer {
    pub(crate) use crate::shaping_publication;
    pub use rhwp::renderer::render_tree;
    pub use rhwp::renderer::style_resolver;
    pub use rhwp::renderer::PathCommand;

    // The product source module calls the crate-private #5821 SSOT. This
    // source integration wrapper mirrors that exact two-branch formula because
    // crate-private library symbols cannot be re-exported to an integration
    // crate.
    pub(crate) fn condensed_ratio_draw_params(font_size: f64, ratio: f64) -> (f64, f64) {
        if ratio > 0.0 && ratio < 0.999 {
            let scale = ratio.sqrt();
            (font_size * scale, scale)
        } else {
            (font_size, if ratio > 0.0 { ratio } else { 1.0 })
        }
    }
}

#[path = "../../src/paint/shaping_glyph.rs"]
mod shaping_glyph;

use std::sync::Arc;

use kerning::{ExactFontSlot, ExactFontSource, ExactFontSourceRegistry};
use rhwp::paint::{
    font_blob_resource_key, parse_font_blob_resource_key, resource_digest_hex, LayerJsonOptions,
    LayerNode, PageLayerTree, PaintOp, ResourceArena, TextVariantKind,
};
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, VariantRejectReason,
    VariantSelectionBackend,
};
use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
use rhwp::renderer::TextStyle;
use shaping::{ShapingFeature, ShapingVariation};
use shaping_composition::{
    attach_horizontal_shaping_mapped_run, certify_horizontal_shaping_mapped_run,
    map_horizontal_shaping_emitted_run, HorizontalShapingEmittedRunCandidate,
};
use shaping_context::{
    HorizontalShapingContext, HorizontalShapingReplaySourceCertificate,
    HorizontalShapingReplaySourceCertificateRejectReason, HorizontalShapingRequest,
};
use shaping_glyph::{
    lower_horizontal_shaping_layer_node_shadow,
    lower_horizontal_shaping_layer_node_shadow_with_prepared_sources,
    lower_horizontal_shaping_page_sidecars, HorizontalShapingGlyphLoweringRejectReason,
    HorizontalShapingPreparedSourceCache,
};
use shaping_paragraph::{
    run_horizontal_shaping_line_transaction, HorizontalShapingFallbackOwner,
    HorizontalShapingLineRequest, HorizontalShapingParagraphRequest,
    HorizontalShapingParagraphScalarStyle,
};
use shaping_publication::{
    HorizontalShapingPageSidecars, HorizontalShapingRunDecision, HorizontalShapingRunRange,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;

const SOURCE_HAN: &[u8] =
    include_bytes!("../../ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf");
const HAPPINESS: &[u8] =
    include_bytes!("../../ttfs/redistributable/happiness-sans/HappinessSansVF.ttf");
const SLOT: ExactFontSlot = ExactFontSlot {
    char_shape_id: 4969,
    language_index: 0,
};
const TEXT: &str = "ᄒᆞᆫ글";
const VARIABLE_TEXT: &str = "가변";
const VARIABLE_SLOT: ExactFontSlot = ExactFontSlot {
    char_shape_id: 4969,
    language_index: 2,
};

fn qualified_context_and_outcome() -> (
    HorizontalShapingContext,
    Arc<shaping_paragraph::HorizontalShapingLineOutcome>,
) {
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            SLOT,
            ExactFontSource {
                bytes: SOURCE_HAN,
                face_index: 0,
            },
        )
        .expect("register exact old-Hangul source");
    let context = HorizontalShapingContext::new(registry);
    let positions = [0.0, 4.0, 8.0, 12.0, 16.0];
    let styles = vec![
        HorizontalShapingParagraphScalarStyle {
            slot: SLOT,
            effective_font_size_px: 10.0,
            width_ratio: 0.8,
            letter_spacing_px: 0.0,
            kerning: true,
            bold: false,
            italic: false,
            superscript: false,
            subscript: false,
        };
        4
    ];
    let outcome = {
        let mut transaction = context.transaction();
        Arc::new(run_horizontal_shaping_line_transaction(
            &mut transaction,
            &HorizontalShapingLineRequest {
                paragraph: HorizontalShapingParagraphRequest {
                    attempt_id_base: 1,
                    text: TEXT,
                    fallback_positions: &positions,
                    scalar_styles: &styles,
                    hard_boundaries: &[false; 5],
                    fallback_owner: HorizontalShapingFallbackOwner::W9K1,
                    model_text_matches_shaping_text: true,
                    horizontal_ltr_bidi0: true,
                    condense_min_space: 0,
                    has_inline_controls: false,
                    has_tabs: false,
                    has_rotation: false,
                    has_char_overlap: false,
                },
                candidate_boundaries: &[0, 4],
                available_widths_px: &[100.0],
            },
        ))
    };
    (context, outcome)
}

fn prepared_sidecar() -> (
    HorizontalShapingPageSidecars,
    Arc<shaping_context::HorizontalShapingMeasurement>,
    Arc<HorizontalShapingReplaySourceCertificate>,
) {
    let (context, outcome) = qualified_context_and_outcome();
    let measurement = Arc::clone(&outcome.lines[0].target_runs[0].measurement);
    let mapped = map_horizontal_shaping_emitted_run(
        &outcome,
        HorizontalShapingEmittedRunCandidate {
            node_id: 17,
            paragraph_text: TEXT,
            emitted_text: TEXT,
            scalar_start: 0,
            origin_x_px: 0.0,
            layout_positions_present: false,
            display_projection_present: false,
            horizontal_ltr_bidi0: true,
            has_field_or_note_split: false,
            has_char_overlap: false,
            has_border_or_background: false,
            has_decoration: false,
        },
    )
    .expect("exact final-run mapping");
    let certified_decision = certify_horizontal_shaping_mapped_run(&context, &mapped)
        .expect("certify exact mapped replay source");
    let certificate = Arc::clone(
        certified_decision
            .replay_source_certificate()
            .expect("mapped decision certificate"),
    );
    let mut sidecars = HorizontalShapingPageSidecars::default();
    sidecars
        .attach(mapped.node_id, mapped.range, certified_decision)
        .expect("attach certified sidecar");
    (sidecars, measurement, certificate)
}

fn text_run() -> TextRunNode {
    let mut style = TextStyle::default();
    style.font_family = "Source Han Serif K Old Hangul".to_string();
    style.font_size = 10.0;
    style.ratio = 0.8;
    style.kerning = true;
    TextRunNode {
        text: TEXT.to_string(),
        style,
        char_shape_id: Some(SLOT.char_shape_id),
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: Some(0),
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 10.0,
        field_marker: FieldMarkerType::None,
        layout_positions: None,
        display_text: None,
    }
}

fn variable_sidecar(
    node_id: u32,
    variations: &[ShapingVariation],
) -> (
    HorizontalShapingPageSidecars,
    Arc<shaping_context::HorizontalShapingMeasurement>,
) {
    let mut registry = ExactFontSourceRegistry::default();
    registry
        .register(
            VARIABLE_SLOT,
            ExactFontSource {
                bytes: HAPPINESS,
                face_index: 0,
            },
        )
        .expect("register exact Happiness Sans variable source");
    let context = HorizontalShapingContext::new(registry);
    let features = [ShapingFeature {
        tag: "kern".into(),
        value: 1,
    }];
    let outcome = {
        let mut transaction = context.transaction();
        transaction.shadow_measure(&HorizontalShapingRequest {
            attempt_id: node_id,
            slot: VARIABLE_SLOT,
            text: VARIABLE_TEXT,
            effective_font_size_px: 10.0,
            width_ratio: 0.8,
            script: Some("Hang"),
            language: Some("ko"),
            features: &features,
            variations,
        })
    };
    let measurement = outcome
        .measurement
        .expect("variable shadow measurement must apply");
    let certificate = context
        .certify_replay_source(&measurement)
        .expect("certify exact variable replay source");
    let range = HorizontalShapingRunRange {
        scalar_start: 0,
        scalar_end: VARIABLE_TEXT.chars().count(),
        utf8_start: 0,
        utf8_end: VARIABLE_TEXT.len(),
        utf16_start: 0,
        utf16_end: VARIABLE_TEXT.encode_utf16().count(),
    };
    let decision = Arc::new(
        HorizontalShapingRunDecision::applied_with_replay_source_certificate(
            range,
            outcome.trace,
            Arc::clone(&measurement),
            certificate,
        ),
    );
    let mut sidecars = HorizontalShapingPageSidecars::default();
    sidecars
        .attach(node_id, range, decision)
        .expect("attach variable replay sidecar");
    (sidecars, measurement)
}

fn variable_text_run() -> TextRunNode {
    let mut style = TextStyle::default();
    style.font_family = "Happiness Sans".to_string();
    style.font_size = 10.0;
    style.ratio = 0.8;
    style.kerning = true;
    TextRunNode {
        text: VARIABLE_TEXT.to_string(),
        style,
        char_shape_id: Some(VARIABLE_SLOT.char_shape_id),
        para_shape_id: None,
        section_index: None,
        para_index: None,
        char_start: Some(0),
        cell_context: None,
        is_para_end: false,
        is_line_break_end: false,
        rotation: 0.0,
        is_vertical: false,
        char_overlap: None,
        border_fill_id: 0,
        baseline: 10.0,
        field_marker: FieldMarkerType::None,
        layout_positions: None,
        display_text: None,
    }
}

fn variable_lowering(
    node_id: u32,
    variations: &[ShapingVariation],
) -> (
    shaping_glyph::HorizontalShapingGlyphLoweringReport,
    Arc<shaping_context::HorizontalShapingMeasurement>,
    ResourceArena,
    BoundingBox,
    TextRunNode,
) {
    let (sidecars, measurement) = variable_sidecar(node_id, variations);
    let run = variable_text_run();
    let bbox = BoundingBox::new(3.0, 5.0, measurement.total_advance_px, 14.0);
    let node = LayerNode::leaf(bbox, Some(node_id), Vec::new());
    let mut resources = ResourceArena::default();
    let report = lower_horizontal_shaping_layer_node_shadow(
        &node,
        bbox,
        &run,
        23,
        &sidecars,
        &mut resources,
    );
    (report, measurement, resources, bbox, run)
}

fn same_f64(left: f64, right: f64) -> bool {
    left.is_finite()
        && right.is_finite()
        && (left - right).abs() <= 1.0e-9 * left.abs().max(right.abs()).max(1.0)
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_c_canonical_instance_reaches_glyph_run_and_variable_outline() {
    let title = [
        ShapingVariation {
            tag: "wght".into(),
            value: 900.0,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
    ];
    let (report, measurement, _, _, _) = variable_lowering(30, &title);

    assert_eq!(report.reject_reason, None);
    let glyph_run = report
        .glyph_run
        .expect("instance-qualified GlyphRun shadow");
    assert_eq!(glyph_run.shape_key.font_instance.variations.len(), 2);
    assert_eq!(glyph_run.shape_key.font_instance.variations[0].tag, "opsz");
    assert_eq!(glyph_run.shape_key.font_instance.variations[0].value, 900.0);
    assert_eq!(glyph_run.shape_key.font_instance.variations[1].tag, "wght");
    assert_eq!(glyph_run.shape_key.font_instance.variations[1].value, 900.0);

    let outline = report
        .glyph_outline
        .expect("exact variable outline shadow candidate");
    let proof = report
        .variable_outline_proof
        .expect("variable outline bbox proof");
    assert_eq!(outline.paths.len(), measurement.glyphs_px.len());
    assert_eq!(proof.glyph_count, measurement.glyphs_px.len());
    assert_eq!(proof.bbox_mismatch_count, 0);
    assert!(proof.command_count > 0);
    assert!(proof.run_local_bbox.width > 0.0);
    assert!(proof.run_local_bbox.height > 0.0);
    assert_eq!(
        proof.settings_sha256,
        measurement.applied.identity.settings_sha256
    );
    assert_eq!(
        outline
            .paths
            .iter()
            .map(|path| path.glyph_id)
            .collect::<Vec<_>>(),
        measurement
            .glyphs_px
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>()
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_c_default_and_title_instances_do_not_share_outline_identity() {
    let title = [
        ShapingVariation {
            tag: "wght".into(),
            value: 900.0,
        },
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
    ];
    let (default_report, default_measurement, _, _, _) = variable_lowering(32, &[]);
    let (title_report, title_measurement, _, _, _) = variable_lowering(33, &title);
    let default_run = default_report.glyph_run.expect("default GlyphRun shadow");
    let title_run = title_report.glyph_run.expect("Title GlyphRun shadow");
    let default_proof = default_report
        .variable_outline_proof
        .expect("default variable outline proof");
    let title_proof = title_report
        .variable_outline_proof
        .expect("Title variable outline proof");

    assert!(default_run.shape_key.font_instance.variations.is_empty());
    assert_eq!(title_run.shape_key.font_instance.variations.len(), 2);
    assert_ne!(
        default_measurement.applied.identity.settings_sha256,
        title_measurement.applied.identity.settings_sha256
    );
    assert!(
        !same_f64(
            default_proof.run_local_bbox.width,
            title_proof.run_local_bbox.width
        ) || !same_f64(
            default_proof.run_local_bbox.height,
            title_proof.run_local_bbox.height
        ),
        "default and Title instances must not collapse to one outline geometry"
    );
    assert_eq!(default_proof.bbox_mismatch_count, 0);
    assert_eq!(title_proof.bbox_mismatch_count, 0);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_c_variable_outline_is_selected_only_on_proven_shadow_backends() {
    let title = [
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
        ShapingVariation {
            tag: "wght".into(),
            value: 900.0,
        },
    ];
    let (mut report, _, resources, bbox, run) = variable_lowering(31, &title);
    let glyph_run = report.glyph_run.take().expect("variable GlyphRun shadow");
    let glyph_outline = report
        .glyph_outline
        .take()
        .expect("variable GlyphOutline shadow");
    let root = LayerNode::leaf(
        bbox,
        Some(31),
        vec![
            PaintOp::text_run(bbox, run),
            PaintOp::GlyphRun {
                bbox,
                run: Box::new(glyph_run),
            },
            PaintOp::GlyphOutline {
                bbox,
                outline: Box::new(glyph_outline),
            },
        ],
    );
    let mut tree = PageLayerTree::new(100.0, 100.0, root);
    tree.resources = resources;

    for backend in [
        VariantSelectionBackend::CanvasKit,
        VariantSelectionBackend::CanvasKitBrowser,
    ] {
        let reports = analyze_text_variant_selection(
            &tree,
            TextVariantSelectionOptions {
                backend,
                ..TextVariantSelectionOptions::canvaskit()
            },
        );
        assert_eq!(reports.len(), 1);
        assert_eq!(
            reports[0].selected_variant_kind,
            Some(TextVariantKind::GlyphOutline),
            "{backend:?} must select the producer-resolved outline"
        );
        assert!(!reports[0].fallback_required);
    }

    for backend in [
        VariantSelectionBackend::NativeSkia,
        VariantSelectionBackend::Svg,
        VariantSelectionBackend::Canvas2D,
    ] {
        let reports = analyze_text_variant_selection(
            &tree,
            TextVariantSelectionOptions {
                backend,
                prefer_strict_outline: true,
                ..TextVariantSelectionOptions::canvaskit()
            },
        );
        assert_eq!(reports.len(), 1);
        assert_eq!(
            reports[0].selected_variant_kind,
            Some(TextVariantKind::TextRun),
            "{backend:?} must retain the non-default instance fallback"
        );
        assert!(reports[0].fallback_required);
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q3_e4_incomplete_or_malformed_outline_falls_back_atomically() {
    let title = [
        ShapingVariation {
            tag: "opsz".into(),
            value: 900.0,
        },
        ShapingVariation {
            tag: "wght".into(),
            value: 900.0,
        },
    ];
    let (mut report, _, resources, bbox, run) = variable_lowering(34, &title);
    let glyph_run = report.glyph_run.take().expect("variable GlyphRun");
    let mut malformed_outline = report.glyph_outline.take().expect("variable GlyphOutline");

    let incomplete_root = LayerNode::leaf(
        bbox,
        Some(34),
        vec![
            PaintOp::text_run(bbox, run.clone()),
            PaintOp::GlyphRun {
                bbox,
                run: Box::new(glyph_run.clone()),
            },
        ],
    );
    let mut incomplete_tree = PageLayerTree::new(100.0, 100.0, incomplete_root);
    incomplete_tree.resources = resources.clone();

    malformed_outline.paths.clear();
    let malformed_root = LayerNode::leaf(
        bbox,
        Some(34),
        vec![
            PaintOp::text_run(bbox, run),
            PaintOp::GlyphRun {
                bbox,
                run: Box::new(glyph_run),
            },
            PaintOp::GlyphOutline {
                bbox,
                outline: Box::new(malformed_outline),
            },
        ],
    );
    let mut malformed_tree = PageLayerTree::new(100.0, 100.0, malformed_root);
    malformed_tree.resources = resources;

    for backend in [
        VariantSelectionBackend::CanvasKit,
        VariantSelectionBackend::CanvasKitBrowser,
    ] {
        let incomplete = analyze_text_variant_selection(
            &incomplete_tree,
            TextVariantSelectionOptions {
                backend,
                prefer_strict_outline: true,
                ..TextVariantSelectionOptions::canvaskit()
            },
        );
        assert_eq!(incomplete.len(), 1);
        assert_eq!(
            incomplete[0].selected_variant_kind,
            Some(TextVariantKind::TextRun),
            "{backend:?} must reject an incomplete atomic pair"
        );
        assert!(incomplete[0].fallback_required);

        let malformed = analyze_text_variant_selection(
            &malformed_tree,
            TextVariantSelectionOptions {
                backend,
                prefer_strict_outline: true,
                ..TextVariantSelectionOptions::canvaskit()
            },
        );
        assert_eq!(malformed.len(), 1);
        assert_eq!(
            malformed[0].selected_variant_kind,
            Some(TextVariantKind::TextRun),
            "{backend:?} must reject a malformed outline"
        );
        assert!(malformed[0].fallback_required);
        assert!(malformed[0].rejected_variants.iter().any(|candidate| {
            candidate.variant_kind == TextVariantKind::GlyphOutline
                && candidate
                    .reasons
                    .contains(&VariantRejectReason::EmptyGlyphOutlinePayload)
        }));
    }
}

#[test]
#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
fn issue_4969_q3_c_native_skia_exact_blob_instance_round_trips_coordinates() {
    use skia_safe::font_arguments::{variation_position::Coordinate, VariationPosition};
    use skia_safe::{Font, FontArguments, FontMgr, FourByteTag};

    let default_typeface = FontMgr::default()
        .new_from_data(HAPPINESS, Some(0))
        .expect("Native Skia must construct the exact official variable TTF");
    let coordinates = [
        Coordinate {
            axis: FourByteTag::from_chars('o', 'p', 's', 'z'),
            value: 900.0,
        },
        Coordinate {
            axis: FourByteTag::from_chars('w', 'g', 'h', 't'),
            value: 900.0,
        },
    ];
    let arguments = FontArguments::new().set_variation_design_position(VariationPosition {
        coordinates: &coordinates,
    });
    let title_typeface = default_typeface
        .clone_with_arguments(&arguments)
        .expect("Native Skia must clone the exact Title instance");
    let round_trip = title_typeface
        .variation_design_position()
        .expect("Native Skia must expose the applied design coordinates");
    for expected in coordinates {
        assert!(round_trip.iter().any(|actual| {
            actual.axis == expected.axis && (actual.value - expected.value).abs() <= f32::EPSILON
        }));
    }

    let mut default_font = Font::new(default_typeface, 1_000.0);
    default_font.set_linear_metrics(true).set_subpixel(true);
    let glyphs = default_font.text_to_glyphs_vec(VARIABLE_TEXT);
    let mut default_widths = vec![0.0; glyphs.len()];
    default_font.get_widths(&glyphs, &mut default_widths);
    let mut title_font = Font::new(title_typeface, 1_000.0);
    title_font.set_linear_metrics(true).set_subpixel(true);
    let mut title_widths = vec![0.0; glyphs.len()];
    title_font.get_widths(&glyphs, &mut title_widths);
    assert!(default_widths
        .iter()
        .zip(&title_widths)
        .any(|(default, title)| (default - title).abs() > f32::EPSILON));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_a_projects_exact_local_geometry_and_clusters() {
    let (sidecars, measurement, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(3.0, 5.0, measurement.total_advance_px, 14.0);
    let node = LayerNode::leaf(bbox, Some(17), vec![PaintOp::text_run(bbox, run.clone())]);
    let mut resources = ResourceArena::default();
    let report = lower_horizontal_shaping_layer_node_shadow(
        &node,
        bbox,
        &run,
        23,
        &sidecars,
        &mut resources,
    );
    let lowered = report.glyph_run.expect("dormant exact glyph run");

    assert_eq!(report.source_node_id, Some(17));
    assert_eq!(report.reject_reason, None);
    assert!(report.claims_glyph_run_slot);
    let expected_ids = [614, 1230, 1497, 2085];
    let expected_page_x = [0.0, 7.728, 15.456, 15.456];
    let expected_page_advance_x = [7.728, 7.728, 0.0, 0.0];
    let expected_local_x = [
        0.0,
        8.640166665059187,
        17.280333330118374,
        17.280333330118374,
    ];
    let expected_local_advance_x = [8.640166665059187, 8.640166665059187, 0.0, 0.0];
    let expected_draw_font_size = 8.94427190999916;
    let expected_draw_x_scale = 0.8944271909999159;
    assert_eq!(lowered.glyph_ids, expected_ids);
    assert_eq!(lowered.positions.len(), expected_local_x.len());
    assert_eq!(
        lowered.advances.as_ref().expect("advances").len(),
        expected_local_advance_x.len()
    );
    for index in 0..expected_ids.len() {
        assert_eq!(measurement.glyphs_px[index].glyph_id, expected_ids[index]);
        assert_eq!(measurement.glyphs_px[index].x, expected_page_x[index]);
        assert_eq!(measurement.glyphs_px[index].y, 0.0);
        assert_eq!(
            measurement.glyphs_px[index].advance_x,
            expected_page_advance_x[index]
        );
        assert_eq!(measurement.glyphs_px[index].advance_y, 0.0);
        assert!(same_f64(
            lowered.positions[index].x,
            expected_local_x[index]
        ));
        assert_eq!(lowered.positions[index].y, 0.0);
        assert!(same_f64(
            lowered.advances.as_ref().unwrap()[index].dx,
            expected_local_advance_x[index]
        ));
        assert_eq!(lowered.advances.as_ref().unwrap()[index].dy, 0.0);
        assert!(same_f64(
            lowered.positions[index].x * lowered.placement.run_to_page.a,
            expected_page_x[index]
        ));
        assert!(same_f64(
            lowered.advances.as_ref().unwrap()[index].dx * lowered.placement.run_to_page.a,
            expected_page_advance_x[index]
        ));
    }
    assert_eq!(lowered.clusters.len(), 2);
    assert_eq!(lowered.clusters[0].source_range_utf8.start, 0);
    assert_eq!(lowered.clusters[0].source_range_utf8.end, 9);
    assert_eq!(lowered.clusters[0].source_range_utf16.unwrap().start, 0);
    assert_eq!(lowered.clusters[0].source_range_utf16.unwrap().end, 3);
    assert_eq!(lowered.clusters[0].glyph_range.start, 0);
    assert_eq!(lowered.clusters[0].glyph_range.end, 1);
    assert_eq!(lowered.clusters[1].source_range_utf8.start, 9);
    assert_eq!(lowered.clusters[1].source_range_utf8.end, 12);
    assert_eq!(lowered.clusters[1].source_range_utf16.unwrap().start, 3);
    assert_eq!(lowered.clusters[1].source_range_utf16.unwrap().end, 4);
    assert_eq!(lowered.clusters[1].glyph_range.start, 1);
    assert_eq!(lowered.clusters[1].glyph_range.end, 4);
    assert!(lowered.glyph_transforms.is_none());
    assert!(same_f64(
        lowered.paint_style.font_size,
        expected_draw_font_size
    ));
    assert_eq!(lowered.paint_style.ratio, 1.0);
    assert!(same_f64(
        lowered.shape_key.font_instance.size_px,
        expected_draw_font_size
    ));
    assert!(same_f64(
        lowered.placement.run_to_page.a,
        expected_draw_x_scale
    ));
    assert_eq!(lowered.placement.run_to_page.d, 1.0);
    assert!(lowered.diagnostics.strict_visual_eligible);
    assert!(lowered.diagnostics.max_origin_delta_px <= 1.0e-9);
    assert!(lowered.diagnostics.max_advance_delta_px <= 1.0e-9);
    assert!(lowered.diagnostics.max_residual_after_adjustment_px <= 1.0e-9);
    assert_eq!(
        lowered.diagnostics.reason.as_deref(),
        Some("q2CommonShapingCondensedDrawProjectionV1")
    );
    assert_eq!(resources.font_blob_count(), 1);
    assert_eq!(resources.font_resources().blobs.len(), 1);
    assert_eq!(resources.font_resources().faces.len(), 1);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_a_fails_closed_without_node_or_source_certificate() {
    let (sidecars, _, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(0.0, 0.0, 20.0, 14.0);
    let mut resources = ResourceArena::default();
    let missing_node = LayerNode::leaf(bbox, None, Vec::new());
    let report = lower_horizontal_shaping_layer_node_shadow(
        &missing_node,
        bbox,
        &run,
        1,
        &sidecars,
        &mut resources,
    );
    assert_eq!(report.source_node_id, None);
    assert_eq!(
        report.reject_reason,
        Some(HorizontalShapingGlyphLoweringRejectReason::MissingSourceNode)
    );
    assert!(!report.claims_glyph_run_slot);

    let (context, outcome) = qualified_context_and_outcome();
    let mapped = map_horizontal_shaping_emitted_run(
        &outcome,
        HorizontalShapingEmittedRunCandidate {
            node_id: 19,
            paragraph_text: TEXT,
            emitted_text: TEXT,
            scalar_start: 0,
            origin_x_px: 0.0,
            layout_positions_present: false,
            display_projection_present: false,
            horizontal_ltr_bidi0: true,
            has_field_or_note_split: false,
            has_char_overlap: false,
            has_border_or_background: false,
            has_decoration: false,
        },
    )
    .expect("uncertified mapping");
    let mut uncertified_sidecars = HorizontalShapingPageSidecars::default();
    attach_horizontal_shaping_mapped_run(&mut uncertified_sidecars, &mapped)
        .expect("D2 sidecar remains valid without replay certificate");
    let node = LayerNode::leaf(bbox, Some(19), Vec::new());
    let report = lower_horizontal_shaping_layer_node_shadow(
        &node,
        bbox,
        &run,
        1,
        &uncertified_sidecars,
        &mut resources,
    );
    assert_eq!(report.source_node_id, Some(19));
    assert_eq!(
        report.reject_reason,
        Some(HorizontalShapingGlyphLoweringRejectReason::MissingReplaySourceCertificate)
    );
    assert!(!report.claims_glyph_run_slot);
    assert_eq!(resources.font_blob_count(), 0);
    assert!(resources.font_resources().blobs.is_empty());
    assert!(resources.font_resources().faces.is_empty());
    drop(context);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_a_certificate_shares_registry_bytes_and_rejects_stale_generation() {
    let (context, outcome) = qualified_context_and_outcome();
    let measurement = Arc::clone(&outcome.lines[0].target_runs[0].measurement);
    let first = context
        .certify_replay_source(&measurement)
        .expect("first exact certificate");
    let second = context
        .certify_replay_source(&measurement)
        .expect("second exact certificate");
    assert!(Arc::ptr_eq(
        first.source_bytes_arc(),
        second.source_bytes_arc()
    ));
    assert_eq!(
        first.source_bytes().as_ptr(),
        second.source_bytes().as_ptr()
    );
    assert!(!format!("{first:?}").contains("OTTO"));

    let mut newer_registry = ExactFontSourceRegistry::default();
    newer_registry
        .register(
            SLOT,
            ExactFontSource {
                bytes: SOURCE_HAN,
                face_index: 0,
            },
        )
        .expect("register first slot");
    newer_registry
        .register(
            ExactFontSlot {
                char_shape_id: SLOT.char_shape_id + 1,
                language_index: SLOT.language_index,
            },
            ExactFontSource {
                bytes: SOURCE_HAN,
                face_index: 0,
            },
        )
        .expect("register alias slot and advance generation");
    let newer_context = HorizontalShapingContext::new(newer_registry);
    assert!(matches!(
        newer_context.certify_replay_source(&measurement),
        Err(HorizontalShapingReplaySourceCertificateRejectReason::StaleRegistryGeneration)
    ));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_a_defers_nonzero_vertical_design_positioning() {
    let (sidecars, measurement, certificate) = prepared_sidecar();
    let mut changed_measurement = (*measurement).clone();
    let mut changed_applied = (*changed_measurement.applied).clone();
    changed_applied.glyphs[0].y_offset = 1;
    changed_measurement.glyphs_px[0].y =
        text_run().style.font_size / f64::from(changed_measurement.units_per_em);
    changed_measurement.applied = Arc::new(changed_applied);
    let changed_measurement = Arc::new(changed_measurement);
    let trace = sidecars
        .get(17)
        .expect("certified decision")
        .trace()
        .clone();
    let decision = Arc::new(
        HorizontalShapingRunDecision::applied_with_replay_source_certificate(
            sidecars.get(17).unwrap().range(),
            trace,
            changed_measurement,
            certificate,
        ),
    );
    let mut changed_sidecars = HorizontalShapingPageSidecars::default();
    changed_sidecars
        .attach(18, decision.range(), decision)
        .expect("attach internally consistent vertical-position fixture");

    let run = text_run();
    let bbox = BoundingBox::new(0.0, 0.0, 20.0, 14.0);
    let node = LayerNode::leaf(bbox, Some(18), Vec::new());
    let mut resources = ResourceArena::default();
    let report = lower_horizontal_shaping_layer_node_shadow(
        &node,
        bbox,
        &run,
        1,
        &changed_sidecars,
        &mut resources,
    );
    assert_eq!(
        report.reject_reason,
        Some(HorizontalShapingGlyphLoweringRejectReason::VerticalPositioningAuthorityPending)
    );
    assert!(!report.claims_glyph_run_slot);
    assert_eq!(resources.font_blob_count(), 0);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_a_shadow_lowerer_does_not_mutate_the_input_leaf() {
    let (sidecars, _, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(0.0, 0.0, 20.0, 14.0);
    let node = LayerNode::leaf(bbox, Some(17), vec![PaintOp::text_run(bbox, run.clone())]);
    let mut resources = ResourceArena::default();
    let report =
        lower_horizontal_shaping_layer_node_shadow(&node, bbox, &run, 5, &sidecars, &mut resources);

    assert!(report.glyph_run.is_some());
    let rhwp::paint::LayerNodeKind::Leaf { ops } = &node.kind else {
        panic!("fixture node must remain a leaf");
    };
    assert_eq!(ops.len(), 1);
    assert!(matches!(ops[0], PaintOp::TextRun { .. }));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_b_common_run_claim_is_unique_and_resource_bounded() {
    let (sidecars, measurement, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(3.0, 5.0, measurement.total_advance_px, 14.0);
    let mut node = LayerNode::leaf(bbox, Some(17), vec![PaintOp::text_run(bbox, run)]);
    let mut resources = ResourceArena::default();

    let claimed = lower_horizontal_shaping_page_sidecars(&mut node, &sidecars, &mut resources);
    assert_eq!(claimed.len(), 1);
    assert!(claimed.contains(&0));
    let rhwp::paint::LayerNodeKind::Leaf { ops } = &node.kind else {
        panic!("fixture node must remain a leaf");
    };
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, PaintOp::TextRun { .. }))
            .count(),
        1
    );
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, PaintOp::GlyphRun { .. }))
            .count(),
        1
    );
    assert_eq!(resources.font_blob_count(), 1);
    assert_eq!(resources.font_resources().blobs.len(), 1);
    assert_eq!(resources.font_resources().faces.len(), 1);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
fn issue_4969_q2_d4_a_reject_reason_names_are_stable_and_non_sensitive() {
    assert_eq!(
        HorizontalShapingGlyphLoweringRejectReason::MissingSidecar.as_str(),
        "missingSidecar"
    );
    assert_eq!(
        HorizontalShapingGlyphLoweringRejectReason::MeasurementGeometryInvalid.as_str(),
        "measurementGeometryInvalid"
    );
    assert_eq!(
        HorizontalShapingGlyphLoweringRejectReason::VerticalPositioningAuthorityPending.as_str(),
        "verticalPositioningAuthorityPending"
    );
    assert_eq!(
        HorizontalShapingReplaySourceCertificateRejectReason::SourceIdentityMismatch.as_str(),
        "sourceIdentityMismatch"
    );
}

#[test]
fn issue_4969_q2_d5_r0_prepares_one_portable_source_once_for_1_2_8_runs() {
    let mut observations = Vec::new();
    for run_count in [1usize, 2, 8] {
        let (sidecars, measurement, _) = prepared_sidecar();
        let run = text_run();
        let bbox = BoundingBox::new(3.0, 5.0, measurement.total_advance_px, 14.0);
        let node = LayerNode::leaf(bbox, Some(17), Vec::new());
        let mut resources = ResourceArena::default();
        let mut prepared_sources = HorizontalShapingPreparedSourceCache::default();
        let mut digest_passes = 0usize;
        let mut face_parses = 0usize;
        let mut arena_intern_attempts = 0usize;

        for text_source_id in 0..run_count {
            let report = lower_horizontal_shaping_layer_node_shadow_with_prepared_sources(
                &node,
                bbox,
                &run,
                u32::try_from(text_source_id).expect("bounded text source id"),
                &sidecars,
                &mut resources,
                &mut prepared_sources,
            );
            assert!(report.glyph_run.is_some());
            digest_passes += report.portable_source_work.explicit_blake3_digest_passes;
            face_parses += report.portable_source_work.face_parse_attempts;
            arena_intern_attempts += report.portable_source_work.arena_intern_attempts;
        }

        observations.push((
            run_count,
            digest_passes,
            face_parses,
            arena_intern_attempts,
            resources.font_blob_count(),
            resources.font_resources().faces.len(),
            prepared_sources.entry_count(),
            prepared_sources.total_source_bytes(),
        ));
    }

    assert_eq!(
        observations,
        vec![
            (1, 1, 1, 1, 1, 1, 1, SOURCE_HAN.len()),
            (2, 1, 1, 1, 1, 1, 1, SOURCE_HAN.len()),
            (8, 1, 1, 1, 1, 1, 1, SOURCE_HAN.len()),
        ],
        "font-wide digest, face preparation, and arena registration must scale with one unique exact source, not emitted run count",
    );
}

#[test]
fn issue_4969_q2_d5_r1_cache_limit_rejects_before_resource_mutation() {
    let (sidecars, measurement, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(3.0, 5.0, measurement.total_advance_px, 14.0);
    let node = LayerNode::leaf(bbox, Some(17), Vec::new());
    let mut resources = ResourceArena::default();
    let mut prepared_sources = HorizontalShapingPreparedSourceCache::with_limits(0, 0);

    let report = lower_horizontal_shaping_layer_node_shadow_with_prepared_sources(
        &node,
        bbox,
        &run,
        0,
        &sidecars,
        &mut resources,
        &mut prepared_sources,
    );

    assert_eq!(
        report.reject_reason,
        Some(HorizontalShapingGlyphLoweringRejectReason::ResourceLimitExceeded)
    );
    assert!(report.glyph_run.is_none());
    assert!(!report.claims_glyph_run_slot);
    assert_eq!(report.portable_source_work, Default::default());
    assert_eq!(prepared_sources.entry_count(), 0);
    assert_eq!(prepared_sources.total_source_bytes(), 0);
    assert_eq!(resources.font_blob_count(), 0);
    assert!(resources.font_resources().blobs.is_empty());
    assert!(resources.font_resources().faces.is_empty());
}

#[test]
fn issue_4969_q2_d5_r1_product_page_lowering_reuses_one_prepared_source() {
    let (sidecars, measurement, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(3.0, 5.0, measurement.total_advance_px, 14.0);
    let ops = (0..8)
        .map(|_| PaintOp::text_run(bbox, run.clone()))
        .collect();
    let mut node = LayerNode::leaf(bbox, Some(17), ops);
    let mut resources = ResourceArena::default();

    let claimed = lower_horizontal_shaping_page_sidecars(&mut node, &sidecars, &mut resources);

    assert_eq!(claimed.len(), 8);
    assert!((0..8).all(|text_source_id| claimed.contains(&text_source_id)));
    let rhwp::paint::LayerNodeKind::Leaf { ops } = &node.kind else {
        panic!("fixture node must remain a leaf");
    };
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, PaintOp::TextRun { .. }))
            .count(),
        8
    );
    assert_eq!(
        ops.iter()
            .filter(|op| matches!(op, PaintOp::GlyphRun { .. }))
            .count(),
        8
    );
    assert_eq!(resources.font_blob_count(), 1);
    assert_eq!(resources.font_resources().blobs.len(), 1);
    assert_eq!(resources.font_resources().faces.len(), 1);
}

#[test]
fn issue_4969_q2_d5_r1_prepared_cache_debug_excludes_font_bytes() {
    let (sidecars, measurement, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(3.0, 5.0, measurement.total_advance_px, 14.0);
    let node = LayerNode::leaf(bbox, Some(17), Vec::new());
    let mut resources = ResourceArena::default();
    let mut prepared_sources = HorizontalShapingPreparedSourceCache::default();
    let report = lower_horizontal_shaping_layer_node_shadow_with_prepared_sources(
        &node,
        bbox,
        &run,
        0,
        &sidecars,
        &mut resources,
        &mut prepared_sources,
    );

    assert!(report.glyph_run.is_some());
    let debug = format!("{prepared_sources:?}");
    assert!(debug.contains("entry_count: 1"));
    assert!(debug.contains(&format!("total_source_bytes: {}", SOURCE_HAN.len())));
    assert!(!debug.contains("OTTO"));
    assert!(!debug.contains("Source Han Serif"));
}

#[test]
fn issue_4969_q2_d5_r2_font_by_key_is_opt_in_and_preserves_exact_metadata() {
    let (sidecars, measurement, _) = prepared_sidecar();
    let run = text_run();
    let bbox = BoundingBox::new(3.0, 5.0, measurement.total_advance_px, 14.0);
    let mut node = LayerNode::leaf(bbox, Some(17), vec![PaintOp::text_run(bbox, run)]);
    let mut resources = ResourceArena::default();
    let claimed = lower_horizontal_shaping_page_sidecars(&mut node, &sidecars, &mut resources);
    assert_eq!(claimed.len(), 1);

    let mut tree = PageLayerTree::new(100.0, 100.0, node);
    tree.resources = resources;
    let default_json = tree.to_json();
    assert_eq!(
        default_json,
        tree.to_json_with_options(LayerJsonOptions::default()),
        "default output must remain byte-identical to the inline contract"
    );

    let by_key_json = tree.to_json_with_options(LayerJsonOptions {
        omit_font_bytes: true,
        ..LayerJsonOptions::default()
    });
    let inline: serde_json::Value = serde_json::from_str(&default_json).expect("inline JSON");
    let by_key: serde_json::Value = serde_json::from_str(&by_key_json).expect("by-key JSON");
    let expected_key = font_blob_resource_key(SOURCE_HAN.len(), &resource_digest_hex(SOURCE_HAN));

    assert_eq!(
        inline["resources"]["fontBlobs"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        by_key["resources"]["fontBlobs"].as_array().unwrap().len(),
        0
    );
    assert_eq!(
        inline["resources"]["fontBlobKeys"],
        by_key["resources"]["fontBlobKeys"]
    );
    assert_eq!(by_key["resources"]["fontBlobKeys"][0], expected_key);
    assert_eq!(inline["fontResources"], by_key["fontResources"]);
    assert_eq!(
        by_key["fontResources"]["blobs"][0]["dataRef"]["id"],
        expected_key
    );
    assert!(
        by_key_json.len() + SOURCE_HAN.len() < default_json.len(),
        "opt-in JSON must actually remove the page-linear font payload"
    );
}

#[test]
fn issue_4969_q2_d5_r2_exact_font_resolver_rejects_unverified_keys() {
    let mut document = rhwp::DocumentCore::new_empty();
    document
        .register_exact_font_source_native(SLOT.char_shape_id, SLOT.language_index, SOURCE_HAN, 0)
        .expect("register exact source");
    let digest = resource_digest_hex(SOURCE_HAN);
    let key = font_blob_resource_key(SOURCE_HAN.len(), &digest);

    assert_eq!(
        parse_font_blob_resource_key(&key),
        Some((SOURCE_HAN.len(), digest.as_str()))
    );
    for invalid in [
        format!("font:blake3:0{}:{digest}", SOURCE_HAN.len()),
        format!("font:blake3:0:{digest}"),
        format!("font:blake3:{}:{}", SOURCE_HAN.len(), digest.to_uppercase()),
        "font:blake3:4:feed".to_string(),
        format!("font:sha256:{}:{digest}", SOURCE_HAN.len()),
        format!("font:blake3:{}:{digest}:extra", SOURCE_HAN.len()),
    ] {
        assert_eq!(parse_font_blob_resource_key(&invalid), None, "{invalid}");
    }

    assert_eq!(
        document.get_source_font_bytes_native(&key).as_deref(),
        Some(SOURCE_HAN)
    );
    assert!(document
        .get_source_font_bytes_native(&format!("font:blake3:{}:{digest}", SOURCE_HAN.len() + 1))
        .is_none());
    assert!(document
        .get_source_font_bytes_native(&format!("font:blake3:0{0}:{digest}", SOURCE_HAN.len()))
        .is_none());
    assert!(document
        .get_source_font_bytes_native(&format!(
            "font:blake3:{}:{}",
            SOURCE_HAN.len(),
            "0".repeat(64)
        ))
        .is_none());
    assert!(document
        .get_source_font_bytes_native(&format!("font:blake3:{}:{digest}", 32 * 1024 * 1024 + 1))
        .is_none());
}
