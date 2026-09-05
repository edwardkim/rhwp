//! Strict Native Skia alternatives must paint exact resources or retain TextRun.

#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use std::io::Cursor;

use rhwp::paint::*;
use rhwp::renderer::layer_renderer::LayerRasterRenderer;
use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
use rhwp::renderer::skia::{
    native_skia_glyph_run_replay_proof, NativeGlyphRunReplayProofReason, SkiaLayerRenderer,
};
use rhwp::renderer::{PathCommand, TextStyle};

const TTF: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
const TTC: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactFaceSmoke.ttc");
const VARIABLE: &[u8] =
    include_bytes!("../../ttfs/redistributable/happiness-sans/HappinessSansVF.ttf");

fn fallback_tree() -> PageLayerTree {
    let run = TextRunNode {
        text: "A".into(),
        style: TextStyle {
            font_family: "sans-serif".into(),
            font_size: 32.0,
            color: 0x0000ff,
            ..Default::default()
        },
        char_shape_id: None,
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
        baseline: 32.0,
        field_marker: FieldMarkerType::None,
        layout_positions: None,
        display_text: None,
    };
    PageLayerTree::new(
        180.0,
        80.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 180.0, 80.0),
            None,
            vec![PaintOp::text_run(
                BoundingBox::new(120.0, 5.0, 40.0, 40.0),
                run,
            )],
        ),
    )
}

fn source(tree: &PageLayerTree) -> TextSourceSpan {
    let LayerNodeKind::Leaf { ops } = &tree.root.kind else {
        unreachable!()
    };
    let PaintOp::TextRun {
        source: Some(source),
        ..
    } = &ops[0]
    else {
        unreachable!()
    };
    source.clone()
}

fn ops_mut(tree: &mut PageLayerTree) -> &mut Vec<PaintOp> {
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        unreachable!()
    };
    ops
}

fn variant(kind: TextVariantKind) -> PaintVariantMeta {
    let mut variant = PaintVariantMeta::text_run_default("text-0");
    variant.variant_id = match kind {
        TextVariantKind::GlyphRun => "glyphRun",
        _ => "glyphOutline",
    }
    .into();
    variant.variant_kind = kind;
    variant.is_default_fallback = false;
    variant.quality = Some(TextVariantQuality::Exact);
    variant.anchor_op_id = Some("text-0".into());
    variant
}

fn diagnostics() -> GlyphRunDiagnostics {
    GlyphRunDiagnostics {
        quality: TextVariantQuality::Exact,
        replay_eligibility: GlyphRunReplayEligibility::Portable,
        strict_visual_eligible: true,
        max_origin_delta_px: 0.0,
        max_advance_delta_px: 0.0,
        max_residual_after_adjustment_px: 0.0,
        cluster_mismatch_count: 0,
        missing_glyph_count: 0,
        used_fallback_font_count: 0,
        reason: None,
    }
}

fn glyph_tree(bytes: &[u8], face_index: u32, character: char) -> PageLayerTree {
    let mut tree = fallback_tree();
    let digest = FontDigest {
        algorithm: "blake3".into(),
        value: resource_digest_hex(bytes),
    };
    let data_ref = BinaryResourceRef {
        kind: BinaryResourceKind::FontBlob,
        id: font_blob_resource_key(bytes.len(), &digest.value),
    };
    tree.resources.intern_font_blob_bytes(bytes);
    tree.resources
        .font_resources_mut()
        .blobs
        .push(FontBlobResource {
            id: FontBlobKey("blob".into()),
            digest: Some(digest.clone()),
            source: FontResourceSource::Embedded,
            data_ref: Some(data_ref.clone()),
            portability: FontPortability::PortableBlob { digest, data_ref },
        });
    tree.resources
        .font_resources_mut()
        .faces
        .push(FontFaceResource {
            id: FontFaceKey("face".into()),
            blob_key: FontBlobKey("blob".into()),
            face_index,
            postscript_name: None,
            family_names: Vec::new(),
            style_names: Vec::new(),
            weight_class: None,
            width_class: None,
            italic: None,
        });
    let glyph = ttf_parser::Face::parse(bytes, face_index)
        .unwrap()
        .glyph_index(character)
        .unwrap()
        .0;
    let run = LayerGlyphRunPaint {
        source: source(&tree),
        variant: variant(TextVariantKind::GlyphRun),
        paint_style: PaintTextStyle::from(&TextStyle {
            font_size: 32.0,
            ..Default::default()
        }),
        shape_key: ShapeKey {
            font_instance: FontInstanceKey {
                face_key: FontFaceKey("face".into()),
                size_px: 32.0,
                variations: Vec::new(),
                synthetic_bold: false,
                synthetic_italic: false,
            },
            direction: TextDirection::Ltr,
            writing_mode: WritingMode::HorizontalTb,
            script: None,
            language: None,
            features: Vec::new(),
            shaping_engine: ShapingEngineId("fixture".into()),
            fallback_policy: FontFallbackPolicyId("none".into()),
        },
        placement: TextRunPlacement {
            run_to_page: LayerAffineTransform {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 10.0,
                f: 42.0,
            },
            baseline_y: 42.0,
        },
        glyph_ids: vec![u32::from(glyph)],
        positions: vec![LayerPoint { x: 0.0, y: 0.0 }],
        advances: None,
        clusters: vec![GlyphCluster {
            source_range_utf8: TextSourceRange::new(0, 1),
            source_range_utf16: Some(TextSourceRange::new(0, 1)),
            text_range_utf8: Some(TextSourceRange::new(0, 1)),
            glyph_range: GlyphRange::new(0, 1),
            flags: Vec::new(),
        }],
        direction: TextDirection::Ltr,
        bidi_level: Some(0),
        writing_mode: WritingMode::HorizontalTb,
        orientation: GlyphRunOrientation::Horizontal,
        glyph_transforms: None,
        diagnostics: diagnostics(),
    };
    ops_mut(&mut tree).push(PaintOp::glyph_run(
        BoundingBox::new(10.0, 10.0, 40.0, 40.0),
        run,
    ));
    tree
}

fn glyph_mut(tree: &mut PageLayerTree) -> &mut LayerGlyphRunPaint {
    let PaintOp::GlyphRun { run, .. } = &mut ops_mut(tree)[1] else {
        unreachable!()
    };
    run
}

fn proof(tree: &PageLayerTree) -> rhwp::renderer::skia::NativeGlyphRunReplayProof {
    let LayerNodeKind::Leaf { ops } = &tree.root.kind else {
        unreachable!()
    };
    let PaintOp::GlyphRun { run, .. } = &ops[1] else {
        unreachable!()
    };
    native_skia_glyph_run_replay_proof(run, &tree.resources)
}

fn render(tree: &PageLayerTree) -> image::RgbaImage {
    let png = SkiaLayerRenderer::new().render_png(tree).unwrap();
    image::load_from_memory(&png).unwrap().to_rgba8()
}

fn assert_selected(tree: &PageLayerTree) -> image::RgbaImage {
    let image = render(tree);
    assert!(
        image
            .enumerate_pixels()
            .any(|(x, _, pixel)| x < 100 && pixel[3] > 64),
        "exact resource must paint visible left-side ink"
    );
    assert!(
        !image
            .enumerate_pixels()
            .any(|(x, _, pixel)| x >= 100 && pixel[3] > 64),
        "anchored right-side fallback must be suppressed"
    );
    image
}

#[test]
fn native_glyph_replay_constructs_ttf_and_exact_nonzero_ttc_face() {
    for tree in [glyph_tree(TTF, 0, 'A'), glyph_tree(TTC, 1, '\u{e104}')] {
        assert!(proof(&tree).typeface_constructible);
        assert_selected(&tree);
    }
}

#[test]
fn native_glyph_replay_changes_ink_for_exact_synthetic_instances() {
    let normal = glyph_tree(TTF, 0, 'A');
    let normal_pixels = assert_selected(&normal);
    for (bold, italic) in [(true, false), (false, true), (true, true)] {
        let mut tree = normal.clone();
        let instance = &mut glyph_mut(&mut tree).shape_key.font_instance;
        instance.synthetic_bold = bold;
        instance.synthetic_italic = italic;
        assert!(proof(&tree).typeface_constructible);
        assert_ne!(normal_pixels, assert_selected(&tree));
    }
}

#[test]
fn native_glyph_replay_preserves_shadow_outline_and_relief_passes() {
    let normal = glyph_tree(TTF, 0, 'A');
    let normal_pixels = assert_selected(&normal);
    let mut images = Vec::new();
    for effect in 0..5 {
        let mut tree = normal.clone();
        let style = &mut glyph_mut(&mut tree).paint_style;
        match effect {
            0 | 4 => {
                style.shadow_type = 1;
                style.shadow_color = 0xff0000;
                style.shadow_offset_x = 8.0;
                style.shadow_offset_y = 5.0;
                if effect == 4 {
                    style.outline_type = 1;
                }
            }
            1 => style.outline_type = 1,
            2 => style.emboss = true,
            _ => style.engrave = true,
        }
        assert!(proof(&tree).typeface_constructible, "effect {effect}");
        let image = assert_selected(&tree);
        assert_ne!(normal_pixels, image, "effect {effect}");
        if effect == 0 || effect == 4 {
            assert!(image.pixels().any(|pixel| pixel[2] > 200 && pixel[0] < 30));
        }
        images.push(image);
    }
    assert_ne!(images[1], images[4], "outline must retain its shadow");
    assert_ne!(images[2], images[3], "relief direction must change");
    let mut relief = normal.clone();
    glyph_mut(&mut relief).paint_style.emboss = true;
    let expected = assert_selected(&relief);
    let style = &mut glyph_mut(&mut relief).paint_style;
    style.engrave = true;
    style.outline_type = 1;
    style.shadow_type = 1;
    style.shadow_offset_x = 8.0;
    style.shadow_offset_y = 5.0;
    assert_eq!(expected, assert_selected(&relief));
    for offset in [f64::NAN, f64::INFINITY, f64::MAX] {
        let mut tree = normal.clone();
        let style = &mut glyph_mut(&mut tree).paint_style;
        style.shadow_type = 1;
        style.shadow_offset_x = offset;
        assert!(proof(&tree)
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::UnsupportedPaintEffect));
        assert_eq!(render(&tree), render(&fallback_tree()));
    }
}

#[test]
fn native_glyph_replay_constructs_requested_variable_axis_and_rejects_invalid_tuples() {
    let normal = glyph_tree(VARIABLE, 0, '가');
    let axis = ttf_parser::Face::parse(VARIABLE, 0)
        .unwrap()
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wght"))
        .unwrap();
    let mut images = Vec::new();
    for value in [axis.min_value, axis.def_value, axis.max_value] {
        let mut tree = normal.clone();
        glyph_mut(&mut tree).shape_key.font_instance.variations = vec![VariationAxisValue {
            tag: "wght".into(),
            value,
        }];
        assert!(proof(&tree).typeface_constructible);
        images.push(assert_selected(&tree));
    }
    assert_ne!(images.first(), images.last());
    for axes in [
        vec![VariationAxisValue {
            tag: "wght".into(),
            value: axis.max_value + 1.0,
        }],
        vec![VariationAxisValue {
            tag: "nope".into(),
            value: 0.0,
        }],
        vec![VariationAxisValue {
            tag: "wght".into(),
            value: f32::NAN,
        }],
        vec![
            VariationAxisValue {
                tag: "wght".into(),
                value: axis.def_value
            };
            2
        ],
    ] {
        let mut tree = normal.clone();
        glyph_mut(&mut tree).shape_key.font_instance.variations = axes;
        assert!(proof(&tree)
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FontVariationUnsupported));
        assert_eq!(render(&tree), render(&fallback_tree()));
    }
}

#[test]
fn native_glyph_replay_rejects_invalid_bytes_digest_face_and_geometry_without_font_lookup() {
    let base = glyph_tree(TTF, 0, 'A');
    let fallback = render(&fallback_tree());
    for case in 0..7 {
        let mut tree = base.clone();
        match case {
            0 => {
                tree.resources.font_resources_mut().blobs[0]
                    .digest
                    .as_mut()
                    .unwrap()
                    .value = "wrong".into()
            }
            1 => tree.resources.font_resources_mut().faces[0].face_index = 99,
            2 => glyph_mut(&mut tree).glyph_ids[0] = u32::from(u16::MAX) + 1,
            3 => glyph_mut(&mut tree).positions[0].x = f64::MAX,
            4 => glyph_mut(&mut tree).shape_key.font_instance.size_px = f64::NAN,
            5 => glyph_mut(&mut tree).direction = TextDirection::Rtl,
            _ => {
                let face = tree.resources.font_resources().faces[0].clone();
                tree.resources.font_resources_mut().faces.push(face);
            }
        }
        assert!(!proof(&tree).typeface_constructible, "case {case}");
        assert_eq!(render(&tree), fallback, "case {case}");
    }
}

fn rectangle_path() -> LayerGlyphOutlinePath {
    LayerGlyphOutlinePath {
        glyph_id: 1,
        source_range_utf8: TextSourceRange::new(0, 1),
        glyph_range: GlyphRange::new(0, 1),
        fill_rule: GlyphOutlineFillRule::NonZero,
        commands: vec![
            PathCommand::MoveTo(10.0, 10.0),
            PathCommand::LineTo(40.0, 10.0),
            PathCommand::LineTo(40.0, 40.0),
            PathCommand::LineTo(10.0, 40.0),
            PathCommand::ClosePath,
        ],
    }
}

fn outline_tree() -> PageLayerTree {
    let mut tree = fallback_tree();
    let outline = LayerGlyphOutlinePaint {
        source: source(&tree),
        variant: variant(TextVariantKind::GlyphOutline),
        payload_kind: GlyphOutlinePayloadKind::MonochromeFill,
        color_layers: None,
        bitmap_glyph: None,
        svg_glyph: None,
        paint_style: PaintTextStyle::from(&TextStyle::default()),
        placement: TextRunPlacement {
            run_to_page: LayerAffineTransform {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 0.0,
                f: 0.0,
            },
            baseline_y: 0.0,
        },
        paths: vec![rectangle_path()],
        stroke: None,
        diagnostics: diagnostics(),
    };
    ops_mut(&mut tree).push(PaintOp::glyph_outline(
        BoundingBox::new(10.0, 10.0, 30.0, 30.0),
        outline,
    ));
    tree
}

fn outline_mut(tree: &mut PageLayerTree) -> &mut LayerGlyphOutlinePaint {
    let PaintOp::GlyphOutline { outline, .. } = &mut ops_mut(tree)[1] else {
        unreachable!()
    };
    outline
}

#[test]
fn native_glyph_replay_renders_outline_fill_rule_transform_and_stroke_order() {
    let mut tree = outline_tree();
    outline_mut(&mut tree).placement.run_to_page.e = 5.0;
    let image = assert_selected(&tree);
    assert_eq!(image.get_pixel(20, 20).0, [0, 0, 0, 255]);
    assert_eq!(image.get_pixel(12, 20)[3], 0);
    for order in [
        GlyphOutlinePaintOrder::FillThenStroke,
        GlyphOutlinePaintOrder::StrokeThenFill,
    ] {
        let outline = outline_mut(&mut tree);
        outline.payload_kind = GlyphOutlinePayloadKind::MonochromeFillStroke;
        outline.stroke = Some(GlyphOutlineStrokeStyle {
            color: 0x0000ff,
            width: 8.0,
            join: GlyphOutlineStrokeJoin::Miter,
            cap: GlyphOutlineStrokeCap::Butt,
            miter_limit: 4.0,
            paint_order: order,
        });
        let image = assert_selected(&tree);
        let inside_edge = image.get_pixel(17, 20);
        assert_eq!(
            inside_edge[0] > 200,
            order == GlyphOutlinePaintOrder::FillThenStroke
        );
    }
}

#[test]
fn native_glyph_replay_requires_every_variant_part_and_every_path() {
    let mut tree = outline_tree();
    let outline = outline_mut(&mut tree);
    outline.variant.part_count = 2;
    let mut second = outline.clone();
    second.variant.part_index = 1;
    second.paths[0].commands[0] = PathCommand::MoveTo(f64::NAN, 0.0);
    ops_mut(&mut tree).push(PaintOp::glyph_outline(
        BoundingBox::new(10.0, 10.0, 30.0, 30.0),
        second,
    ));
    assert_eq!(render(&tree), render(&fallback_tree()));
    ops_mut(&mut tree).pop();
    assert_eq!(render(&tree), render(&fallback_tree()));
}

fn bitmap_tree(bytes: &[u8]) -> PageLayerTree {
    let mut tree = outline_tree();
    let image_ref = tree.resources.intern_image_bytes(bytes);
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::BitmapGlyph;
    outline.paths.clear();
    outline.bitmap_glyph = Some(BitmapGlyphPayload {
        image_ref,
        source_range_utf8: TextSourceRange::new(0, 1),
        glyph_range: GlyphRange::new(0, 1),
        placement: BoundingBox::new(25.0, 15.0, 20.0, 20.0),
        alpha_premultiplied: true,
        scaling_policy: BitmapGlyphScalingPolicy::SourceExact,
        filtering: BitmapGlyphFiltering::Nearest,
        transform_to_run: None,
    });
    tree
}

#[test]
fn native_glyph_replay_decodes_bitmap_before_selection_and_preserves_payload_placement() {
    let mut encoded = Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 255, 255]))
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    let tree = bitmap_tree(encoded.get_ref());
    let image = assert_selected(&tree);
    assert_eq!(image.get_pixel(30, 20).0, [0, 0, 255, 255]);
    assert_eq!(image.get_pixel(12, 12)[3], 0);
    let corrupt = &encoded.get_ref()[..encoded.get_ref().len() / 2];
    assert_eq!(render(&bitmap_tree(corrupt)), render(&fallback_tree()));
}

fn svg_tree(fragment: &str) -> PageLayerTree {
    let mut tree = outline_tree();
    let svg_ref = tree.resources.intern_svg_fragment(fragment);
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
    outline.paths.clear();
    outline.svg_glyph = Some(SvgGlyphPayload {
        svg_ref,
        source_range_utf8: TextSourceRange::new(0, 1),
        glyph_range: GlyphRange::new(0, 1),
        view_box: BoundingBox::new(10.0, 20.0, 30.0, 30.0),
        intrinsic_size: None,
        static_sanitized: true,
        script_allowed: false,
        animation_allowed: false,
        external_resources_allowed: false,
        interactivity_allowed: false,
        transform_to_run: None,
    });
    tree
}

#[test]
fn native_glyph_replay_uses_static_svg_viewbox_and_rejects_unsafe_resources() {
    let tree = svg_tree("<svg xmlns=\"http://www.w3.org/2000/svg\"><path fill=\"#0000ff\" d=\"M10 20H40V50H10Z\"/></svg>");
    let image = assert_selected(&tree);
    assert_eq!(image.get_pixel(20, 20).0, [0, 0, 255, 255]);
    for fragment in [
        "<svg><script>alert(1)</script><path d=\"M10 20H40V50Z\"/></svg>",
        "<svg><image href=\"file:///private/missing.png\"/><path d=\"M10 20H40V50Z\"/></svg>",
        "<svg><path d=\"Mbroken\"/></svg>",
    ] {
        assert_eq!(render(&svg_tree(fragment)), render(&fallback_tree()));
    }
}

#[test]
fn native_glyph_replay_renders_resolved_colrv0_layers_in_order() {
    let mut tree = outline_tree();
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::ColorLayers;
    outline.paths.clear();
    let layer = |rgba| ColorLayerNode {
        layer_index: Some(0),
        glyph_id: Some(1),
        glyph_range: Some(GlyphRange::new(0, 1)),
        source_range_utf8: Some(TextSourceRange::new(0, 1)),
        source_font_ref: None,
        commands: Some(rectangle_path().commands),
        fill: Some(ResolvedColor {
            color_space: Some("sRGB".into()),
            rgba,
        }),
        fill_rule: Some(GlyphOutlineFillRule::NonZero),
        palette_index: None,
        color: None,
        opacity: None,
        transform_to_run: None,
    };
    outline.color_layers = Some(ColorLayersPayload {
        color_format: ColorGlyphFormat::ColrV0,
        source_font_ref: None,
        palette_ref: None,
        layers: vec![layer([1.0, 0.0, 0.0, 1.0]), layer([0.0, 0.0, 1.0, 0.5])],
        paint_graph: None,
        source_range_utf8: None,
        glyph_range: None,
    });
    let image = assert_selected(&tree);
    let pixel = image.get_pixel(20, 20);
    assert!((120..=135).contains(&pixel[0]) && (120..=135).contains(&pixel[2]));
    outline_mut(&mut tree).color_layers.as_mut().unwrap().layers[1]
        .fill
        .as_mut()
        .unwrap()
        .rgba[0] = f32::NAN;
    assert_eq!(render(&tree), render(&fallback_tree()));
}
