//! Typed Text IR v2 uses the same checked envelopes and original Rust payloads.

use rhwp::paint::*;
use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
use rhwp::renderer::TextStyle;

const FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");

fn leaf(text: &str) -> LayerNode {
    let bbox = BoundingBox::new(10.123_456_789, 20.0, 16.0, 16.0);
    let run = TextRunNode {
        text: text.to_string(),
        style: TextStyle {
            font_family: "fixture".to_string(),
            font_size: 16.0,
            ..Default::default()
        },
        char_shape_id: Some(7),
        para_shape_id: Some(2),
        section_index: Some(0),
        para_index: Some(3),
        char_start: Some(9),
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
    };
    LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)])
}

fn native_tree() -> PageLayerTree {
    let mut tree = PageLayerTree::new(100.0, 100.0, leaf("\u{E100}"));
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

#[test]
fn typed_compatibility_snapshot_borrows_unrounded_canonical_paint() {
    let tree = PageLayerTree::new(100.0, 100.0, leaf("native payload"));
    let slots = tree.text_v2_slots().unwrap();
    assert!(tree.validate_text_v2_slots().is_empty());
    assert_eq!(slots.len(), 1);
    let slot = &slots[0];
    assert_eq!(slot.id(), "text-0");
    assert_eq!(slot.paint_order_slot_id(), slot.id());
    assert_eq!(slot.default_variant_id(), "textRun");
    assert_eq!(slot.fallback_policy(), TextFallbackPolicy::Required);
    assert_eq!(slot.bbox().x, 10.123_456_789);
    let variant = &slot.variants()[0];
    assert_eq!(variant.kind(), TextVariantKind::TextRun);
    assert_eq!(variant.quality(), None);
    assert!(variant.required_features().is_empty());
    let part = &variant.parts()[0];
    assert_eq!(part.part_index(), 0);
    assert_eq!(part.part_count(), 1);
    assert_eq!(part.local_paint_order(), None);
    assert_eq!(part.bbox().x, slot.bbox().x);
    assert_eq!(part.bbox().y, slot.bbox().y);
    assert_eq!(part.bbox().width, slot.bbox().width);
    assert_eq!(part.bbox().height, slot.bbox().height);
    let LayerNodeKind::Leaf { ops } = &tree.root.kind else {
        panic!("leaf");
    };
    assert!(std::ptr::eq(part.payload(), &ops[0]));
    let PaintOp::TextRun { run, source, .. } = part.payload() else {
        panic!("TextRun");
    };
    assert_eq!(run.char_start, Some(9));
    assert!(source.as_ref().unwrap().stable_source_key.is_some());
}

#[test]
fn typed_variants_downgrade_without_changing_payloads_or_resources() {
    let tree = native_tree();
    let before = tree.to_json();
    let slots = tree.text_v2_slots().unwrap();
    assert_eq!(slots[0].variants().len(), 3);
    assert_eq!(slots[0].variants()[1].variant_id(), "glyphRun");
    assert_eq!(slots[0].variants()[2].kind(), TextVariantKind::GlyphOutline);
    let mut restored = tree.clone();
    let LayerNodeKind::Leaf { ops } = &mut restored.root.kind else {
        panic!("leaf");
    };
    *ops = downgrade_text_v2_op_to_v1_compat(&slots[0]).unwrap();
    assert_eq!(restored.to_json(), before);
}

#[test]
fn typed_slots_follow_group_and_clip_paint_order() {
    let bounds = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
    let tree = PageLayerTree::new(
        100.0,
        100.0,
        LayerNode::group(
            bounds,
            None,
            vec![
                leaf("first"),
                LayerNode::clip_rect(bounds, None, bounds, leaf("second"), ClipKind::Generic),
                leaf("third"),
            ],
            CacheHint::None,
            GroupKind::Generic,
        ),
    );
    let slots = tree.text_v2_slots().unwrap();
    assert_eq!(
        slots.iter().map(|slot| slot.id()).collect::<Vec<_>>(),
        ["text-0", "text-1", "text-2"]
    );
}

#[test]
fn typed_slots_reject_the_same_invalid_source_and_part_contracts_as_json() {
    for invalid_source in [false, true] {
        let mut tree = native_tree();
        let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
            panic!("leaf");
        };
        let run = ops
            .iter_mut()
            .find_map(|op| match op {
                PaintOp::GlyphRun { run, .. } => Some(run),
                _ => None,
            })
            .unwrap();
        if invalid_source {
            run.source.id = TextSourceId(123);
        } else {
            run.variant.part_count = 2;
        }
        let expected = tree.to_json_v2_compat().unwrap_err();
        assert_eq!(tree.text_v2_slots().unwrap_err(), expected);
        assert_eq!(tree.validate_text_v2_slots(), expected);
    }
}

#[test]
fn strict_typed_slots_need_resource_proof_and_cannot_downgrade() {
    let mut tree = native_tree();
    for slots in [
        tree.text_v2_slots_strict_glyph_run().unwrap(),
        tree.text_v2_slots_strict_glyph_outline().unwrap(),
    ] {
        assert_eq!(slots[0].fallback_policy(), TextFallbackPolicy::None);
        assert_eq!(slots[0].variants().len(), 1);
        assert_eq!(
            downgrade_text_v2_op_to_v1_compat(&slots[0]).unwrap_err()[0].code,
            "fallbackRequiredTextRunMissing"
        );
    }
    tree.resources = ResourceArena::default();
    assert!(tree.text_v2_slots_strict_glyph_run().is_err());
    assert!(tree.text_v2_slots_strict_glyph_outline().is_err());
}
