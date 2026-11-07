//! 관계형: 합성 TextRun·공개 fixture 글꼴의 glyph coverage·replay와 typed 실패를 검사한다.
//! 주석 갱신: 2026-11-07T19:17:16+09:00; e4ffabea 리베이스에서 양쪽 회귀 범위를 함께 보존했다.
//! 입력은 파일 내부 합성 문서·PageLayerTree 또는 저장소의 허가된 글꼴 fixture다.
//! API·소유·자원/합성 paint 계약이며 실물 문서 페이지의 조판 위치를 고정하지 않는다.
//! 이 assertion의 독립 한컴 PDF 대비 Native/fresh WASM raw 측정값은 비해당이다.
//! 별도 실물 시각·플랫폼 실행 근거: mydocs/pr/assets/seo-rii-20261107/validation-scope.md.
//! 엄격한 Native Skia 대안은 정확한 자원을 그리거나 원래 TextRun을 보존한다.
//! #7640: 시스템 face의 조회·paint 프로그램 일치와 명시 공급 스타일의 윤곽 소유도 검사한다.

#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use std::io::Cursor;

use rhwp::paint::*;
use rhwp::renderer::layer_renderer::LayerRasterRenderer;
use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
use rhwp::renderer::skia::{
    native_skia_glyph_run_replay_proof, NativeGlyphPreparationFailure,
    NativeGlyphRunReplayProofReason, SkiaLayerRenderer,
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

fn assert_same_pixels(actual: image::RgbaImage, expected: image::RgbaImage) {
    assert_eq!(actual.dimensions(), expected.dimensions());
    if let Some(((x, y, actual), expected)) = actual
        .enumerate_pixels()
        .zip(expected.pixels())
        .find(|((_, _, actual), expected)| *actual != *expected)
    {
        panic!("first differing pixel ({x}, {y}): {actual:?}, expected {expected:?}");
    }
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
        assert!(normal_pixels != assert_selected(&tree));
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
        assert!(normal_pixels != image, "effect {effect}");
        if effect == 0 || effect == 4 {
            assert!(image.pixels().any(|pixel| pixel[2] > 200 && pixel[0] < 30));
        }
        images.push(image);
    }
    assert!(images[1] != images[4], "outline must retain its shadow");
    assert!(images[2] != images[3], "relief direction must change");
    let mut relief = normal.clone();
    glyph_mut(&mut relief).paint_style.emboss = true;
    let expected = assert_selected(&relief);
    let style = &mut glyph_mut(&mut relief).paint_style;
    style.engrave = true;
    style.outline_type = 1;
    style.shadow_type = 1;
    style.shadow_offset_x = 8.0;
    style.shadow_offset_y = 5.0;
    assert_same_pixels(assert_selected(&relief), expected);
    for offset in [f64::NAN, f64::INFINITY, f64::MAX] {
        let mut tree = normal.clone();
        let style = &mut glyph_mut(&mut tree).paint_style;
        style.shadow_type = 1;
        style.shadow_offset_x = offset;
        assert!(proof(&tree)
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::UnsupportedPaintEffect));
        assert_same_pixels(render(&tree), render(&fallback_tree()));
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
    assert!(images.first() != images.last());
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
        assert_same_pixels(render(&tree), render(&fallback_tree()));
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
        assert_same_pixels(render(&tree), fallback.clone());
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
        paint_style: PaintTextStyle::from(&TextStyle {
            font_size: 32.0,
            ..Default::default()
        }),
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
    let mut invalid_style = outline_tree();
    outline_mut(&mut invalid_style).paint_style.font_size = 0.0;
    assert_same_pixels(render(&invalid_style), render(&fallback_tree()));

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
    assert_same_pixels(render(&tree), render(&fallback_tree()));
    ops_mut(&mut tree).pop();
    let error = SkiaLayerRenderer::new().render_png(&tree).unwrap_err();
    assert!(error.to_string().contains("has 1 parts, expected 2"));
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
    assert_same_pixels(render(&bitmap_tree(corrupt)), render(&fallback_tree()));
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
        font_em_box: None,
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
    assert_selected(&svg_tree(
        "<svg><script>alert(1)</script><path d=\"M10 20H40V50Z\"/></svg>",
    ));
    // SVG currentColor inherits the run's foreground. Independent Chrome and
    // fresh WASM artwork evidence: issue8174/v2-s03, all seven cases raw 100%.
    let mut foreground = svg_tree(
        "<svg><rect x=\"10\" y=\"20\" width=\"30\" height=\"30\" fill=\"currentColor\"/></svg>",
    );
    outline_mut(&mut foreground).paint_style.color = 0x008000;
    assert!(assert_selected(&foreground)
        .pixels()
        .any(|pixel| pixel.0 == [0, 128, 0, 255]));
    for fragment in [
        "<svg><image href=\"file:///private/missing.png\"/><path d=\"M10 20H40V50Z\"/></svg>",
        "<svg><path d=\"Mbroken\"/></svg>",
        "<svg><defs><g id=\"a\"><use href=\"#b\"/></g><g id=\"b\"><use href=\"#a\"/></g></defs><use href=\"#a\"/></svg>",
    ] {
        assert_same_pixels(render(&svg_tree(fragment)), render(&fallback_tree()));
    }
    // Same valid artwork after an XML declaration and ignored executable subtree.
    // These are the two former static_svg source-unit contracts, through actual paint.
    let declared = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><svg><script><g id=\"ignored\"/></script><path fill=\"#0000ff\" d=\"M10 20H40V50H10Z\"/></svg>";
    assert_same_pixels(render(&svg_tree(declared)), render(&tree));
    let gradient_artwork = "<defs><linearGradient id=\"g\"><stop stop-color=\"#ff0000\"/></linearGradient></defs><path d=\"M10 20H40V50H10Z\" fill=\"url(#g)\"/>";
    let plain = svg_tree(&format!("<svg>{gradient_artwork}</svg>"));
    let declared = svg_tree(&format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?><svg><script><g id=\"ignored\"/></script>{gradient_artwork}</svg>"));
    assert_same_pixels(render(&declared), render(&plain));
    assert!(assert_selected(&declared)
        .pixels()
        .any(|pixel| pixel.0 == [255, 0, 0, 255]));

    let mut expansion = String::from("<svg><defs><path id=\"a0\" d=\"M10 20H40V50Z\"/>");
    for i in 1..20 {
        expansion.push_str(&format!(
            "<g id=\"a{i}\"><use href=\"#a{}\"/><use href=\"#a{}\"/></g>",
            i - 1,
            i - 1
        ));
    }
    expansion.push_str("</defs><use href=\"#a19\"/></svg>");
    assert_same_pixels(render(&svg_tree(&expansion)), render(&fallback_tree()));
    assert_selected(&tree);
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
    assert_same_pixels(render(&tree), render(&fallback_tree()));
}

#[test]
fn native_glyph_preparation_reports_exact_proof_with_and_without_picture_cache() {
    use rhwp::renderer::layer_renderer::{RasterRenderOptions, VariantRejectReason};
    for case in 0..4 {
        let mut tree = glyph_tree(TTF, 0, 'A');
        let expected_reason = match case {
            0 => {
                tree.resources.font_resources_mut().blobs[0]
                    .digest
                    .as_mut()
                    .unwrap()
                    .value = "wrong".into();
                NativeGlyphRunReplayProofReason::FontBlobDigestMismatch
            }
            1 => {
                tree.resources.font_resources_mut().faces[0].face_index = 99;
                NativeGlyphRunReplayProofReason::ExactFaceUnavailable
            }
            2 => {
                glyph_mut(&mut tree).diagnostics.cluster_mismatch_count = 1;
                NativeGlyphRunReplayProofReason::ClusterMismatch
            }
            _ => {
                glyph_mut(&mut tree).shape_key.font_instance.face_key.0 = "missing".into();
                NativeGlyphRunReplayProofReason::FontFaceMissing
            }
        };
        let expected_proof = proof(&tree);
        assert!(expected_proof.reasons.contains(&expected_reason));
        tree.root = LayerNode::group(
            tree.root.bounds,
            None,
            vec![tree.root.clone()],
            CacheHint::StaticSubtree,
            GroupKind::Generic,
        );
        let cached = SkiaLayerRenderer::new();
        let cold = cached
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .unwrap();
        let warm = cached
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .unwrap();
        let uncached = SkiaLayerRenderer::new()
            .with_cache_limits(0, 0)
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .unwrap();
        assert!(warm.diagnostics.static_picture_cache_hits > 0);
        for output in [&cold, &warm, &uncached] {
            let rejected = &output.diagnostics.native_glyph_rejections;
            assert_eq!(rejected.len(), 1, "case {case}");
            assert_eq!(
                rejected[0].failure,
                NativeGlyphPreparationFailure::GlyphRunRejected(expected_proof.clone())
            );
            assert_eq!(output.diagnostics.native_glyph_rejections_dropped, 0);
            assert!(output.diagnostics.variant_selections[0].fallback_required);
            if case == 0 {
                assert_eq!(
                    output.diagnostics.variant_selections[0].rejected_variants[0].reasons,
                    vec![VariantRejectReason::FontBlobDigestMismatch]
                );
            }
            assert_same_pixels(
                image::load_from_memory(&output.bytes).unwrap().to_rgba8(),
                render(&fallback_tree()),
            );
        }
        assert_eq!(
            cold.diagnostics.variant_selections,
            warm.diagnostics.variant_selections
        );
        assert_eq!(
            cold.diagnostics.variant_selections,
            uncached.diagnostics.variant_selections
        );
    }
}

#[test]
fn native_outline_preparation_reports_payload_failure_and_retains_fallback() {
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    let mut tree = outline_tree();
    outline_mut(&mut tree).paint_style.font_size = 0.0;
    let output = SkiaLayerRenderer::new()
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .unwrap();
    assert_eq!(
        output.diagnostics.native_glyph_rejections[0].failure,
        NativeGlyphPreparationFailure::UnsupportedOutlinePayload
    );
    assert_same_pixels(
        image::load_from_memory(&output.bytes).unwrap().to_rgba8(),
        render(&fallback_tree()),
    );
}

#[test]
fn native_preparation_reports_bound_count_and_unicode_identifiers_on_public_replay() {
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    let mut tree = glyph_tree(TTF, 0, 'A');
    tree.resources.font_resources_mut().blobs[0]
        .digest
        .as_mut()
        .unwrap()
        .value = "wrong".into();
    let candidate = ops_mut(&mut tree).pop().unwrap();
    let identifier = "한".repeat(256);
    for index in 0..259 {
        let mut candidate = candidate.clone();
        let PaintOp::GlyphRun { run, .. } = &mut candidate else {
            unreachable!()
        };
        run.variant.equivalence_group = identifier.clone();
        run.variant.anchor_op_id = Some(identifier.clone());
        run.variant.variant_id = format!("{identifier}-{index}");
        ops_mut(&mut tree).push(candidate);
    }
    tree.root = LayerNode::group(
        tree.root.bounds,
        None,
        vec![tree.root.clone()],
        CacheHint::StaticSubtree,
        GroupKind::Generic,
    );
    let renderer = SkiaLayerRenderer::new();
    let cold = renderer
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .unwrap();
    let warm = renderer
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .unwrap();
    let uncached = SkiaLayerRenderer::new()
        .with_cache_limits(0, 0)
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .unwrap();
    assert!(warm.diagnostics.static_picture_cache_hits > 0);
    let expected = "한".repeat(85);
    for output in [&cold, &warm, &uncached] {
        assert_eq!(output.diagnostics.native_glyph_rejections.len(), 256);
        assert_eq!(output.diagnostics.native_glyph_rejections_dropped, 3);
        for report in &output.diagnostics.native_glyph_rejections {
            assert_eq!(report.equivalence_group, expected);
            assert_eq!(report.variant_id, expected);
            assert!(report.equivalence_group.len() <= 256 && report.variant_id.len() <= 256);
            assert!(matches!(&report.failure,
                NativeGlyphPreparationFailure::GlyphRunRejected(proof)
                if proof.reasons == vec![NativeGlyphRunReplayProofReason::FontBlobDigestMismatch]));
        }
        assert_same_pixels(
            image::load_from_memory(&output.bytes).unwrap().to_rgba8(),
            render(&fallback_tree()),
        );
    }
    assert_eq!(
        cold.diagnostics.native_glyph_rejections,
        warm.diagnostics.native_glyph_rejections
    );
    assert_eq!(
        cold.diagnostics.native_glyph_rejections,
        uncached.diagnostics.native_glyph_rejections
    );
}

#[test]
fn native_picture_diagnostics_do_not_retain_a_previous_siblings_report_cap() {
    use rhwp::renderer::layer_renderer::RasterRenderOptions;

    let mut first = glyph_tree(TTF, 0, 'A');
    first.resources.font_resources_mut().blobs[0]
        .digest
        .as_mut()
        .unwrap()
        .value = "wrong".into();
    let mut second = first.root.clone();
    let LayerNodeKind::Leaf { ops: second_ops } = &mut second.kind else {
        unreachable!()
    };
    let PaintOp::TextRun {
        source: Some(source),
        ..
    } = &mut second_ops[0]
    else {
        unreachable!()
    };
    source.id = TextSourceId(1);
    let PaintOp::GlyphRun { run, .. } = &mut second_ops[1] else {
        unreachable!()
    };
    run.source.id = TextSourceId(1);
    run.variant.equivalence_group = "text-1".into();
    run.variant.anchor_op_id = Some("text-1".into());
    let candidate = ops_mut(&mut first).pop().unwrap();
    for index in 0..256 {
        let mut candidate = candidate.clone();
        let PaintOp::GlyphRun { run, .. } = &mut candidate else {
            unreachable!()
        };
        run.variant.variant_id = format!("prefix-{index}");
        ops_mut(&mut first).push(candidate);
    }
    let bounds = first.root.bounds;
    let mut tree = PageLayerTree::new(
        first.page_width,
        first.page_height,
        LayerNode::group(
            bounds,
            None,
            vec![
                first.root,
                LayerNode::group(
                    bounds,
                    None,
                    vec![second],
                    CacheHint::StaticSubtree,
                    GroupKind::Generic,
                ),
            ],
            CacheHint::None,
            GroupKind::Generic,
        ),
    );
    tree.resources = first.resources;
    let renderer = SkiaLayerRenderer::new();
    let initial = renderer
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .unwrap();
    assert_eq!(initial.diagnostics.native_glyph_rejections.len(), 256);
    assert_eq!(initial.diagnostics.native_glyph_rejections_dropped, 1);

    // Keep the same fallback/source count and the identical cached group, but
    // remove the earlier sibling's rejected alternatives. Its diagnostics must
    // not remain charged against the unchanged group's next replay.
    let LayerNodeKind::Group { children, .. } = &mut tree.root.kind else {
        unreachable!()
    };
    let LayerNodeKind::Leaf { ops } = &mut children[0].kind else {
        unreachable!()
    };
    ops.retain(|op| matches!(op, PaintOp::TextRun { .. }));
    let expected = SkiaLayerRenderer::new()
        .with_cache_limits(0, 0)
        .render_raster_with_options(&tree, RasterRenderOptions::default())
        .unwrap();
    assert_eq!(expected.diagnostics.native_glyph_rejections.len(), 1);
    assert_eq!(expected.diagnostics.native_glyph_rejections_dropped, 0);
    for _ in 0..2 {
        let actual = renderer
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .unwrap();
        assert_eq!(
            actual.diagnostics.native_glyph_rejections,
            expected.diagnostics.native_glyph_rejections
        );
        assert_eq!(actual.diagnostics.native_glyph_rejections_dropped, 0);
        assert_same_pixels(
            image::load_from_memory(&actual.bytes).unwrap().to_rgba8(),
            image::load_from_memory(&expected.bytes).unwrap().to_rgba8(),
        );
    }
    assert!(renderer.cache_stats().pictures.hits > 0);
}

fn bounded_vertical_glyph_tree() -> PageLayerTree {
    bounded_vertical_glyph_tree_with_font(TTF, 'A')
}

fn bounded_vertical_glyph_tree_with_font(bytes: &[u8], character: char) -> PageLayerTree {
    let mut tree = glyph_tree(bytes, 0, character);
    let run = glyph_mut(&mut tree);
    run.variant.variant_id = "verticalGlyphRun".into();
    run.variant.requires = vec![
        "fontResources".into(),
        "text.glyphRun".into(),
        "text.glyphRun.verticalUpright".into(),
    ];
    run.shape_key.writing_mode = WritingMode::VerticalRl;
    run.shape_key.shaping_engine.0 = "rustybuzz-q4-vertical-v1".into();
    run.writing_mode = WritingMode::VerticalRl;
    run.orientation = GlyphRunOrientation::VerticalUpright;
    run.placement.run_to_page = LayerAffineTransform {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };
    run.placement.baseline_y = 0.0;
    run.positions = vec![LayerPoint { x: 10.0, y: 42.0 }];
    run.advances = Some(vec![LayerVector { dx: 0.0, dy: 32.0 }]);
    run.clusters[0].flags = vec![GlyphClusterFlag::FallbackBoundary];
    run.diagnostics.reason = Some("boundedVerticalHwp5TableCellV1".into());
    tree
}

#[test]
fn bounded_vertical_replay_checks_source_scalars_before_a_cached_glyph_is_selected() {
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    const SUPPLEMENTARY: &[u8] = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");

    let rebind = |tree: &mut PageLayerTree, text: &str| {
        let PaintOp::TextRun { run, source, .. } = &mut ops_mut(tree)[0] else {
            unreachable!()
        };
        run.text = text.into();
        let rebound = TextSourceSpan::for_text_run(0, run);
        *source = Some(rebound.clone());
        let run = glyph_mut(tree);
        run.source = rebound.clone();
        run.clusters[0].source_range_utf8 = rebound.utf8_range;
        run.clusters[0].source_range_utf16 = Some(rebound.utf16_range);
        run.clusters[0].text_range_utf8 = Some(rebound.utf8_range);
        tree.text_sources = TextSourceTable::from_layer_node(&tree.root);
    };
    let mut supplementary = bounded_vertical_glyph_tree_with_font(SUPPLEMENTARY, '😀');
    rebind(&mut supplementary, "😀");
    assert!(proof(&supplementary).typeface_constructible);
    assert_selected(&supplementary);
    let renderer = SkiaLayerRenderer::new();
    let positive = renderer
        .render_raster_with_options(&supplementary, RasterRenderOptions::default())
        .unwrap();
    assert_eq!(
        positive.diagnostics.variant_selections[0]
            .selected_variant_id
            .as_deref(),
        Some("verticalGlyphRun")
    );
    assert!(positive.diagnostics.native_glyph_rejections.is_empty());

    let mut multiple = supplementary.clone();
    rebind(&mut multiple, "éé");
    // The resource/run declaration is unchanged: both strings occupy four
    // UTF-8 bytes and two UTF-16 units. Only the actual source distinguishes
    // this malformed bounded claim from the supplementary-scalar control.
    assert!(proof(&multiple).typeface_constructible);
    let mut fallback = multiple.clone();
    ops_mut(&mut fallback).retain(|op| matches!(op, PaintOp::TextRun { .. }));
    let expected = render(&fallback);
    let uncached = SkiaLayerRenderer::new().with_cache_limits(0, 0);
    for renderer in [&renderer, &uncached] {
        for _ in 0..2 {
            let output = renderer
                .render_raster_with_options(&multiple, RasterRenderOptions::default())
                .unwrap();
            assert_eq!(
                output.diagnostics.variant_selections[0]
                    .selected_variant_id
                    .as_deref(),
                Some("textRun")
            );
            assert!(matches!(
                &output.diagnostics.native_glyph_rejections[0].failure,
                NativeGlyphPreparationFailure::GlyphRunRejected(proof)
                    if proof.reasons == vec![NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported]
            ));
            assert_same_pixels(
                image::load_from_memory(&output.bytes).unwrap().to_rgba8(),
                expected.clone(),
            );
        }
    }
}

#[test]
fn native_bounded_vertical_replay_uses_exact_upright_glyph_at_producer_origin() {
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    let tree = bounded_vertical_glyph_tree();
    assert!(proof(&tree).typeface_constructible);
    // The same unrotated glyph at the same origin is an independent control
    // for consuming page-space vertical placement without applying a 90° turn.
    assert_same_pixels(
        assert_selected(&tree),
        assert_selected(&glyph_tree(TTF, 0, 'A')),
    );
    let renderer = SkiaLayerRenderer::new();
    for _ in 0..2 {
        let output = renderer
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .unwrap();
        assert_eq!(
            output.diagnostics.variant_selections[0]
                .selected_variant_id
                .as_deref(),
            Some("verticalGlyphRun")
        );
        assert!(!output.diagnostics.variant_selections[0].fallback_required);
        assert!(output.diagnostics.native_glyph_rejections.is_empty());
    }
    assert!(renderer.cache_stats().glyphs.hits > 0);
}

#[test]
fn native_vertical_replay_keeps_fallback_for_unproven_modes_and_malformed_bounded_claims() {
    let base = bounded_vertical_glyph_tree();
    let fallback = render(&fallback_tree());
    for case in 0..18 {
        let mut tree = base.clone();
        let run = glyph_mut(&mut tree);
        let expected = match case {
            0 => {
                run.diagnostics.reason = None;
                NativeGlyphRunReplayProofReason::DirectionUnsupported
            }
            1 => {
                run.diagnostics.reason = Some("boundedVerticalHwpxTableCellV1".into());
                NativeGlyphRunReplayProofReason::VerticalOrientationUnsupported
            }
            2 => {
                run.orientation = GlyphRunOrientation::VerticalSideways;
                NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported
            }
            3 => {
                run.orientation = GlyphRunOrientation::MixedPerGlyph;
                NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported
            }
            4 => {
                run.shape_key.shaping_engine.0 = "unproven".into();
                NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported
            }
            5 => {
                run.variant.requires.pop();
                NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported
            }
            6 => {
                run.clusters[0].source_range_utf8.end = 2;
                NativeGlyphRunReplayProofReason::ClusterMismatch
            }
            7 => {
                run.clusters[0].flags.clear();
                NativeGlyphRunReplayProofReason::ClusterMismatch
            }
            8 => {
                run.advances = None;
                NativeGlyphRunReplayProofReason::AdvanceCountMismatch
            }
            9 => {
                run.placement.run_to_page.e = 1.0;
                NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported
            }
            10 => {
                run.positions[0].y = f64::NAN;
                NativeGlyphRunReplayProofReason::PositionNotFinite
            }
            11 => {
                run.shape_key.font_instance.synthetic_bold = true;
                NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported
            }
            12 => {
                run.diagnostics.quality = TextVariantQuality::PositionAdjusted;
                NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported
            }
            13 => {
                run.paint_style.shadow_type = 1;
                NativeGlyphRunReplayProofReason::UnsupportedPaintEffect
            }
            14 => {
                run.writing_mode = WritingMode::HorizontalTb;
                run.shape_key.writing_mode = WritingMode::HorizontalTb;
                run.orientation = GlyphRunOrientation::Horizontal;
                NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported
            }
            15 => {
                run.paint_style.font_size += 1.0;
                NativeGlyphRunReplayProofReason::VerticalAuthorityUnsupported
            }
            16 => {
                tree.resources.font_resources_mut().blobs[0]
                    .digest
                    .as_mut()
                    .unwrap()
                    .value = "wrong".into();
                NativeGlyphRunReplayProofReason::FontBlobDigestMismatch
            }
            _ => {
                tree.resources.font_resources_mut().faces[0].face_index = 99;
                NativeGlyphRunReplayProofReason::ExactFaceUnavailable
            }
        };
        let rejected = proof(&tree);
        assert!(!rejected.typeface_constructible, "case {case}");
        assert!(
            rejected.reasons.contains(&expected),
            "case {case}: {rejected:?}"
        );
        assert_same_pixels(render(&tree), fallback.clone());
    }
}

// 기존 Native CI 진입점에서 함께 실행한다. 개인 글꼴은 필수가 아니며,
// 읽을 수 있는 OS face를 대조하고 별도 OFL 명시 공급 검사는 항상 실행한다.
mod issue_7640_system_program {
    use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
    use rhwp::renderer::layer_renderer::LayerRasterRenderer;
    use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
    use rhwp::renderer::skia::SkiaLayerRenderer;
    use rhwp::renderer::TextStyle;
    use skia_safe::{FontMgr, FontStyle};

    fn tree(family: &str, text: &str, bold: bool) -> PageLayerTree {
        let bbox = BoundingBox::new(10.0, 10.0, 450.0, 60.0);
        let run = TextRunNode {
            text: text.into(),
            style: TextStyle {
                font_family: family.into(),
                font_size: 32.0,
                bold,
                source_font_is_ttf: true,
                shade_color: 0x00ff_ffff,
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
            baseline: 40.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        };
        PageLayerTree::with_profile(
            480.0,
            90.0,
            LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]),
            RenderProfile::Print,
        )
    }

    fn pixels(renderer: &SkiaLayerRenderer, tree: &PageLayerTree) -> image::RgbaImage {
        image::load_from_memory(&renderer.render_png(tree).unwrap())
            .unwrap()
            .to_rgba8()
    }

    #[test]
    fn available_system_faces_match_their_actual_program_snapshot() {
        let manager = FontMgr::default();
        let names: std::collections::HashSet<_> = manager.family_names().collect();
        let mut checked = 0;
        for family in [
            "Helvetica",
            "Arial",
            "DejaVu Sans",
            "Liberation Sans",
            "한컴돋움",
        ] {
            if !names.contains(family) {
                continue;
            }
            for bold in [false, true] {
                let style = if bold {
                    FontStyle::bold()
                } else {
                    FontStyle::normal()
                };
                let Some(face) = manager.match_family_style(family, style) else {
                    continue;
                };
                let Some((bytes, index)) = face.to_font_bytes() else {
                    continue;
                };
                // 이 대조는 정적 단일 프로그램 범위다. 컬렉션·가변 글꼴은 기존 별도 계약에서 검사한다.
                if index != 0
                    || ttf_parser::fonts_in_collection(&bytes).is_some()
                    || bytes.len() > 32 * 1024 * 1024
                    || face
                        .variation_design_position()
                        .is_some_and(|v| !v.is_empty())
                {
                    continue;
                }
                let parsed = ttf_parser::Face::parse(&bytes, 0).unwrap();
                let text = if "좌표공간의점글"
                    .chars()
                    .all(|c| parsed.glyph_index(c).is_some())
                {
                    "좌표공간의 점 글꼴"
                } else {
                    "AWM glyph program"
                };
                let page = tree(family, text, bold);
                let explicit = SkiaLayerRenderer::new()
                    .try_with_font_data(&[bytes])
                    .unwrap();
                assert!(
                    pixels(&SkiaLayerRenderer::new(), &page) == pixels(&explicit, &page),
                    "시스템 조회와 실제 프로그램의 glyph 소유가 같다: {family}, bold={bold}"
                );
                checked += 1;
            }
        }
        eprintln!("#7640 읽을 수 있는 정적 OS 프로그램 대조: {checked}");
    }

    #[test]
    fn explicit_regular_and_bold_programs_keep_their_own_outlines() {
        const REGULAR: &[u8] =
            include_bytes!("../fixtures/native_font_styles/RHWPSuppliedStyle-Regular.ttf");
        const BOLD: &[u8] =
            include_bytes!("../fixtures/native_font_styles/RHWPSuppliedStyle-Bold.ttf");
        let regular = SkiaLayerRenderer::new()
            .try_with_font_data(&[REGULAR.to_vec()])
            .unwrap();
        let bold = SkiaLayerRenderer::new()
            .try_with_font_data(&[BOLD.to_vec()])
            .unwrap();
        let mixed = SkiaLayerRenderer::new()
            .try_with_font_data(&[REGULAR.to_vec(), BOLD.to_vec()])
            .unwrap();
        let normal_page = tree("RHWP Supplied Style Contract", "AW 총투자", false);
        let bold_page = tree("RHWP Supplied Style Contract", "AW 총투자", true);
        assert_ne!(pixels(&regular, &normal_page), pixels(&bold, &bold_page));
        assert_eq!(pixels(&regular, &normal_page), pixels(&mixed, &normal_page));
        assert_eq!(pixels(&bold, &bold_page), pixels(&mixed, &bold_page));
    }
}
