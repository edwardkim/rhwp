//! Opt-in Text IR v2 export: anchored envelopes, proven strict variants, no loss.

use rhwp::paint::*;
use rhwp::renderer::render_tree::{BoundingBox, PageBackgroundNode, TextRunNode};
use rhwp::renderer::TextStyle;
use serde_json::{json, Value};

const FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");

fn text(text: &str) -> PaintOp {
    PaintOp::text_run(
        BoundingBox::new(10.0, 20.0, 16.0, 16.0),
        TextRunNode {
            text: text.to_string(),
            style: TextStyle {
                font_family: "fixture".to_string(),
                font_size: 16.0,
                ..Default::default()
            },
            char_shape_id: Some(7),
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 12.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        },
    )
}

fn tree(ops: Vec<PaintOp>) -> PageLayerTree {
    PageLayerTree::new(
        100.0,
        100.0,
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 100.0, 100.0), None, ops),
    )
}

fn native_tree(text_value: &str) -> PageLayerTree {
    let mut tree = tree(vec![text(text_value)]);
    lower_font_native_glyph_sidecars(
        &mut tree.root,
        &mut tree.resources,
        &[EmbeddedFontFace {
            char_shape_id: 7,
            language_index: 0,
            family: "fixture",
            alternate_family: None,
            bytes: FONT,
            face_index: 0,
        }],
    );
    tree
}

fn value(json: String) -> Value {
    serde_json::from_str(&json).unwrap()
}

#[test]
fn v2_compat_is_opt_in_and_preserves_the_canonical_text_payload() {
    let tree = tree(vec![text("plain text")]);
    let before = tree.to_json();
    let old = value(before.clone());
    let new = value(tree.to_json_v2_compat().unwrap());
    assert_eq!(tree.to_json(), before);
    assert_eq!(old["schemaVersion"], 1);
    assert_eq!(new["schemaVersion"], 2);
    assert_eq!(new["textV2"]["canonicalOp"], "text");
    assert_eq!(new["textV2"]["fallbackRequired"], true);
    let slot = &new["root"]["ops"][0];
    assert_eq!(slot["id"], "text-0");
    assert_eq!(slot["fallbackPolicy"], "required");
    let mut payload = slot["variants"][0]["parts"][0]["payload"].clone();
    assert_eq!(
        payload.as_object_mut().unwrap().remove("id"),
        Some(json!("op-text-0"))
    );
    assert_eq!(payload, old["root"]["ops"][0]);
    assert_eq!(new["textSources"], old["textSources"]);
    assert_eq!(new["resources"], old["resources"]);
    assert_eq!(new["outputOptions"], old["outputOptions"]);
}

#[test]
fn v2_envelope_keeps_all_compatibility_variants_and_font_resources() {
    let tree = native_tree("\u{E100}");
    let old = value(tree.to_json());
    let new = value(tree.to_json_v2_compat().unwrap());
    let slot = &new["root"]["ops"][0];
    assert_eq!(new["root"]["ops"].as_array().unwrap().len(), 1);
    assert_eq!(slot["variants"].as_array().unwrap().len(), 3);
    assert_eq!(slot["variants"][0]["kind"], "textRun");
    assert_eq!(slot["variants"][1]["kind"], "glyphRun");
    assert_eq!(slot["variants"][2]["kind"], "glyphOutline");
    assert_eq!(new["fontResources"], old["fontResources"]);
    assert_eq!(new["resources"], old["resources"]);
}

#[test]
fn strict_glyph_run_export_uses_only_the_proven_font_variant() {
    let tree = native_tree("\u{E100}");
    let new = value(tree.to_json_v2_strict_glyph_run().unwrap());
    let slot = &new["root"]["ops"][0];
    assert_eq!(slot["fallbackPolicy"], "none");
    assert_eq!(slot["defaultVariantId"], "glyphRun");
    assert_eq!(slot["variants"].as_array().unwrap().len(), 1);
    assert_eq!(
        slot["variants"][0]["parts"][0]["payload"]["type"],
        "glyphRun"
    );
    assert_eq!(new["textV2"]["strictVisualFallbackFree"], true);
    assert_eq!(new["text"]["fallbackRequired"], false);
    assert!(new["requiredFeatures"]
        .as_array()
        .unwrap()
        .contains(&json!("text.strictVisualFallbackFree")));
}

#[test]
fn strict_outline_export_preserves_bitmap_resources() {
    let tree = native_tree("\u{E100}");
    let old = value(tree.to_json());
    let new = value(tree.to_json_v2_strict_glyph_outline().unwrap());
    let slot = &new["root"]["ops"][0];
    assert_eq!(slot["defaultVariantId"], "glyphOutline");
    assert_eq!(slot["variants"].as_array().unwrap().len(), 1);
    let payload = &slot["variants"][0]["parts"][0]["payload"];
    assert_eq!(payload["type"], "glyphOutline");
    assert_eq!(new["resources"], old["resources"]);
}

#[test]
fn strict_export_rejects_uncovered_slots_missing_resources_and_bad_glyph_geometry() {
    let plain = tree(vec![text("plain")]);
    assert!(plain.to_json_v2_strict_glyph_run().is_err());
    assert!(plain.to_json_v2_strict_glyph_outline().is_err());
    let mut missing = native_tree("\u{E100}");
    missing.resources = ResourceArena::default();
    assert!(missing.to_json_v2_strict_glyph_run().is_err());
    assert!(missing.to_json_v2_strict_glyph_outline().is_err());
    let mut malformed = native_tree("\u{E100}");
    let LayerNodeKind::Leaf { ops } = &mut malformed.root.kind else {
        panic!("leaf");
    };
    for op in ops {
        if let PaintOp::GlyphRun { run, .. } = op {
            run.positions.clear();
        }
    }
    assert!(malformed.to_json_v2_strict_glyph_run().is_err());
}

#[test]
fn compat_export_rejects_incomplete_parts_and_invalid_source_bindings() {
    let mut incomplete = native_tree("\u{E100}");
    let LayerNodeKind::Leaf { ops } = &mut incomplete.root.kind else {
        panic!("leaf");
    };
    for op in ops {
        if let PaintOp::GlyphRun { run, .. } = op {
            run.variant.part_count = 2;
        }
    }
    assert!(incomplete.to_json_v2_compat().is_err());
    let mut invalid = native_tree("\u{E100}");
    let LayerNodeKind::Leaf { ops } = &mut invalid.root.kind else {
        panic!("leaf");
    };
    for op in ops {
        if let PaintOp::GlyphOutline { outline, .. } = op {
            outline.source.id = TextSourceId(99);
        }
    }
    assert!(invalid.to_json_v2_compat().is_err());
}

#[test]
fn envelopes_stay_at_the_fallback_paint_order_slot() {
    let mut tree = native_tree("\u{E100}");
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        panic!("leaf");
    };
    let glyph = ops.remove(1);
    let bbox = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
    ops.insert(
        0,
        PaintOp::PageBackground {
            bbox,
            background: Box::new(PageBackgroundNode {
                background_color: Some(0xffffff),
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: None,
            }),
        },
    );
    ops.insert(0, glyph);
    let new = value(tree.to_json_v2_compat().unwrap());
    assert_eq!(new["root"]["ops"][0]["type"], "pageBackground");
    assert_eq!(new["root"]["ops"][1]["type"], "text");
    assert_eq!(new["root"]["ops"].as_array().unwrap().len(), 2);
}

#[test]
fn export_rejects_mislabeled_public_sidecars_before_variant_selection() {
    let mut tree = native_tree("\u{E100}");
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        panic!("leaf");
    };
    for op in ops {
        if let PaintOp::GlyphRun { run, .. } = op {
            run.variant.variant_kind = TextVariantKind::TextRun;
        }
    }
    assert_eq!(
        tree.to_json_v2_strict_glyph_run().unwrap_err()[0].code,
        "variantKindMismatch"
    );
    assert!(tree.to_json_v2_compat().is_err());
}

#[test]
fn resource_backed_advanced_outlines_report_strict_availability_without_enabling_backends() {
    let tree = native_tree("\u{E100}");
    let diagnostics = TextV2Diagnostics::from_layer_tree(&tree);
    let variant = diagnostics.slot_diagnostics[0]
        .variants
        .iter()
        .find(|variant| variant.variant_kind == "glyphOutline")
        .unwrap();
    assert!(variant.strict_visual_eligible);
    let strict = value(tree.to_json_v2_strict_glyph_outline().unwrap());
    assert!(strict["requiredFeatures"]
        .as_array()
        .unwrap()
        .contains(&json!("text.glyphOutline.bitmapGlyph")));
}
