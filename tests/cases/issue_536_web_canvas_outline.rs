//! 관계형·고정형: 합성 Canvas2D 색상·geometry와 실제 문서 API의 브라우저 계약.
//! 주석 갱신: 2026-11-09T08:33:08+09:00; 직전 파일 변경: e1167418fb7082f8bfa839d97df50b78d9439dcf 2026-11-07T19:56:08+09:00.
//! 측정: 2026-11-09, visual_sweep.py silhouette-only, Print 96 DPI, 색상 경계 보정 전 2px raw.
//! 대조군 bitmap-channel-transfer-independent.hwpx/독립 2024 PDF 전 1쪽: Native 100.00%, fresh WASM 100.00%.
//! 다중 페이지 radial-corner-print-contract.hwpx/독립 2020 PDF 전 6쪽: Native/fresh WASM 각각 최저 100.00%.
//! 두 입력 합계 7쪽씩의 최저값은 각각 100.00%; compat은 앞 입력 2024, 뒤 입력 2022다.
//! 정식 증거: mydocs/pr/assets/seo-rii-20261109/visual/의 bitmap-channel-{native,wasm}.tsv와 radial-{native,wasm}.tsv.
//! 위 수치는 기본 출력 대조군이며 합성 PaintOp·공개 font fixture 계약의 한컴 시각 측정으로 바꾸어 해석하지 않는다.
//! 합성 Canvas 좌표·색상은 입력 primitive의 독립 계약이다. 실제 문서 페이지의 px 위치를 고정하지 않는다.
//! render-p35-font-native-bitmap.hwpx의 문서 API 검사는 로딩·선택·픽셀·기본 경로 보존 smoke이며 최종 한컴 일치 주장이 아니다.

#![cfg(target_arch = "wasm32")]

use std::io::Cursor;

use rhwp::paint::*;
use rhwp::renderer::layer_renderer::{
    analyze_text_variant_selection, TextVariantSelectionOptions, TextVariantSelectionReport,
    VariantRejectReason,
};
use rhwp::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
use rhwp::renderer::web_canvas::WebCanvasRenderer;
use rhwp::renderer::{PathCommand, TextStyle};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

wasm_bindgen_test_configure!(run_in_browser);

const WIDTH: u32 = 180;
const HEIGHT: u32 = 80;

fn canvas() -> (HtmlCanvasElement, CanvasRenderingContext2d) {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();
    canvas.set_width(WIDTH);
    canvas.set_height(HEIGHT);
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();
    (canvas, context)
}

fn pixels(context: &CanvasRenderingContext2d) -> image::RgbaImage {
    let data = context
        .get_image_data(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT))
        .unwrap()
        .data()
        .0;
    image::RgbaImage::from_raw(WIDTH, HEIGHT, data).unwrap()
}

fn render(
    tree: &PageLayerTree,
    strict: bool,
) -> (image::RgbaImage, Vec<TextVariantSelectionReport>) {
    let (canvas, context) = canvas();
    let mut renderer = WebCanvasRenderer::new(&canvas).unwrap();
    renderer.set_strict_glyph_outline_replay(strict);
    renderer.render_layer_tree(tree);
    let reports = renderer.get_text_variant_selection_diagnostics().to_vec();
    (pixels(&context), reports)
}

fn assert_same_pixels(actual: &image::RgbaImage, expected: &image::RgbaImage) {
    assert_eq!(actual.dimensions(), expected.dimensions());
    if let Some(((x, y, actual), expected)) = actual
        .enumerate_pixels()
        .zip(expected.pixels())
        .find(|((_, _, actual), expected)| *actual != *expected)
    {
        panic!("first differing pixel ({x}, {y}): {actual:?}, expected {expected:?}");
    }
}

fn has_ink(image: &image::RgbaImage, left: bool) -> bool {
    image
        .enumerate_pixels()
        .any(|(x, _, pixel)| (x < 100) == left && pixel.0 != [255, 255, 255, 255])
}

fn assert_selected(tree: &PageLayerTree) -> image::RgbaImage {
    let shared = analyze_text_variant_selection(
        tree,
        TextVariantSelectionOptions::canvas2d_strict_outline(),
    );
    assert_eq!(shared.len(), 1);
    assert_eq!(
        shared[0].selected_variant_kind,
        Some(TextVariantKind::GlyphOutline),
        "{shared:?}"
    );
    validate_text_variant_scope(tree).unwrap();
    let (image, reports) = render(tree, true);
    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].selected_variant_kind,
        Some(TextVariantKind::GlyphOutline),
        "{reports:?}"
    );
    assert!(!reports[0].fallback_required);
    assert!(
        has_ink(&image, true),
        "prepared outline paints left-side ink"
    );
    assert!(
        !has_ink(&image, false),
        "source-bound right-side TextRun is suppressed"
    );
    image
}

fn assert_fallback(tree: &PageLayerTree) {
    let (actual, reports) = render(tree, true);
    let expected = render(&fallback_tree(), false).0;
    assert_same_pixels(&actual, &expected);
    assert_eq!(reports.len(), 1);
    assert_eq!(
        reports[0].selected_variant_kind,
        Some(TextVariantKind::TextRun),
        "{reports:?}"
    );
    assert!(reports[0].fallback_required);
    assert!(!has_ink(&actual, true));
    assert!(has_ink(&actual, false));
}

fn identity() -> LayerAffineTransform {
    LayerAffineTransform {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    }
}

fn rectangle(x: f64, y: f64, width: f64, height: f64) -> Vec<PathCommand> {
    vec![
        PathCommand::MoveTo(x, y),
        PathCommand::LineTo(x + width, y),
        PathCommand::LineTo(x + width, y + height),
        PathCommand::LineTo(x, y + height),
        PathCommand::ClosePath,
    ]
}

fn fallback_tree() -> PageLayerTree {
    let run = TextRunNode {
        text: "A".into(),
        style: TextStyle {
            font_family: "sans-serif".into(),
            font_size: 32.0,
            color: 0x00ff0000,
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
        f64::from(WIDTH),
        f64::from(HEIGHT),
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT)),
            None,
            vec![PaintOp::text_run(
                BoundingBox::new(120.0, 5.0, 40.0, 40.0),
                run,
            )],
        ),
    )
}

fn ops_mut(tree: &mut PageLayerTree) -> &mut Vec<PaintOp> {
    let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
        panic!("source-bound leaf fixture")
    };
    ops
}

fn outline_mut(tree: &mut PageLayerTree) -> &mut LayerGlyphOutlinePaint {
    let PaintOp::GlyphOutline { outline, .. } = &mut ops_mut(tree)[1] else {
        panic!("outline fixture")
    };
    outline
}

fn outline_tree() -> PageLayerTree {
    let mut tree = fallback_tree();
    let PaintOp::TextRun {
        source: Some(source),
        ..
    } = &ops_mut(&mut tree)[0]
    else {
        unreachable!()
    };
    let source = source.clone();
    let mut variant = PaintVariantMeta::text_run_default("text-0");
    variant.variant_id = "glyphOutline".into();
    variant.variant_kind = TextVariantKind::GlyphOutline;
    variant.is_default_fallback = false;
    variant.quality = Some(TextVariantQuality::Exact);
    variant.anchor_op_id = Some("text-0".into());
    let outline = LayerGlyphOutlinePaint {
        source,
        variant,
        payload_kind: GlyphOutlinePayloadKind::MonochromeFill,
        color_layers: None,
        bitmap_glyph: None,
        svg_glyph: None,
        paint_style: PaintTextStyle::from(&TextStyle {
            font_size: 32.0,
            ..Default::default()
        }),
        placement: TextRunPlacement {
            run_to_page: identity(),
            baseline_y: 0.0,
        },
        paths: vec![LayerGlyphOutlinePath {
            glyph_id: 1,
            source_range_utf8: TextSourceRange::new(0, 1),
            glyph_range: GlyphRange::new(0, 1),
            commands: rectangle(10.0, 10.0, 30.0, 30.0),
            fill_rule: GlyphOutlineFillRule::NonZero,
        }],
        stroke: None,
        diagnostics: GlyphRunDiagnostics {
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
        },
    };
    ops_mut(&mut tree).push(PaintOp::glyph_outline(
        BoundingBox::new(10.0, 10.0, 30.0, 30.0),
        outline,
    ));
    tree
}

fn source_font(format: ColorGlyphFormat) -> FontColorGlyphRef {
    FontColorGlyphRef {
        face_key: Some("canvas2d-fixture".into()),
        glyph_id: Some(1),
        palette_index: None,
        color_format: Some(format),
    }
}

fn color(rgba: [f32; 4]) -> ResolvedColor {
    ResolvedColor {
        color_space: Some("sRGB".into()),
        rgba,
    }
}

fn graph_node(node_id: u32) -> ColorPaintGraphNode {
    ColorPaintGraphNode {
        node_id,
        kind: ColorPaintGraphNodeKind::SolidPath,
        solid_path: None,
        linear_gradient_path: None,
        radial_gradient_path: None,
        sweep_gradient_path: None,
        transform: None,
        composite: None,
        clip: None,
        source_range_utf8: Some(TextSourceRange::new(0, 1)),
        glyph_range: Some(GlyphRange::new(0, 1)),
        source_font_ref: Some(source_font(ColorGlyphFormat::ColrV1)),
    }
}

fn solid_node(node_id: u32, rgba: [f32; 4]) -> ColorPaintGraphNode {
    let mut node = graph_node(node_id);
    node.solid_path = Some(ColorPaintSolidPathNode {
        commands: rectangle(10.0, 10.0, 30.0, 30.0),
        fill: color(rgba),
        fill_rule: GlyphOutlineFillRule::NonZero,
        source_glyph_id: Some(1),
        palette_index: None,
    });
    node
}

fn graph_tree(graph: ColorPaintGraphPayload) -> PageLayerTree {
    let mut tree = outline_tree();
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::ColorLayers;
    outline.paths.clear();
    outline.color_layers = Some(ColorLayersPayload {
        color_format: ColorGlyphFormat::ColrV1,
        source_font_ref: Some(source_font(ColorGlyphFormat::ColrV1)),
        palette_ref: None,
        layers: Vec::new(),
        paint_graph: Some(graph),
        source_range_utf8: Some(TextSourceRange::new(0, 1)),
        glyph_range: Some(GlyphRange::new(0, 1)),
    });
    tree
}

fn blue_png() -> Vec<u8> {
    let mut encoded = Cursor::new(Vec::new());
    image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 255, 255]))
        .write_to(&mut encoded, image::ImageFormat::Png)
        .unwrap();
    encoded.into_inner()
}

fn bitmap_tree(bytes: &[u8]) -> PageLayerTree {
    let mut tree = outline_tree();
    let image_ref = tree.resources.intern_image_bytes(bytes);
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::BitmapGlyph;
    outline.paths.clear();
    let payload = BitmapGlyphPayload {
        image_ref,
        source_range_utf8: TextSourceRange::new(0, 1),
        glyph_range: GlyphRange::new(0, 1),
        placement: BoundingBox::new(25.0, 15.0, 20.0, 20.0),
        alpha_premultiplied: false,
        scaling_policy: BitmapGlyphScalingPolicy::SourceExact,
        filtering: BitmapGlyphFiltering::Nearest,
        transform_to_run: None,
    };
    assert!(payload.has_strict_visual_contract());
    outline.bitmap_glyph = Some(payload);
    tree
}

fn svg_tree(fragment: &str) -> PageLayerTree {
    let mut tree = outline_tree();
    let svg_ref = tree.resources.intern_svg_fragment(fragment);
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
    outline.paths.clear();
    let payload = SvgGlyphPayload {
        svg_ref,
        source_range_utf8: TextSourceRange::new(0, 1),
        glyph_range: GlyphRange::new(0, 1),
        view_box: BoundingBox::new(10.0, 20.0, 30.0, 30.0),
        font_em_box: Some(BoundingBox::new(25.0, 15.0, 20.0, 20.0)),
        intrinsic_size: Some(LayerVector { dx: 30.0, dy: 30.0 }),
        static_sanitized: true,
        script_allowed: false,
        animation_allowed: false,
        external_resources_allowed: false,
        interactivity_allowed: false,
        transform_to_run: None,
    };
    assert!(payload.has_static_sanitized_contract());
    outline.svg_glyph = Some(payload);
    tree
}

#[wasm_bindgen_test]
fn web_canvas_outline_default_off_and_explicit_preview_choose_distinct_pixels() {
    let tree = outline_tree();
    let (off, reports) = render(&tree, false);
    assert_same_pixels(&off, &render(&fallback_tree(), false).0);
    assert!(reports.is_empty(), "default path does not analyze sidecars");
    assert!(!has_ink(&off, true));
    assert!(has_ink(&off, false));
    let on = assert_selected(&tree);
    assert_eq!(on.get_pixel(20, 20).0, [0, 0, 0, 255]);
}

#[wasm_bindgen_test]
fn web_canvas_outline_preserves_transform_evenodd_and_both_stroke_orders() {
    let mut tree = outline_tree();
    let outline = outline_mut(&mut tree);
    outline.placement.run_to_page.e = 5.0;
    outline.paths[0].fill_rule = GlyphOutlineFillRule::EvenOdd;
    outline.paths[0]
        .commands
        .extend(rectangle(20.0, 15.0, 10.0, 20.0));
    let image = assert_selected(&tree);
    assert_eq!(image.get_pixel(20, 20).0, [0, 0, 0, 255]);
    assert_eq!(image.get_pixel(30, 20).0, [255, 255, 255, 255]);
    assert_eq!(image.get_pixel(12, 20).0, [255, 255, 255, 255]);
    for order in [
        GlyphOutlinePaintOrder::FillThenStroke,
        GlyphOutlinePaintOrder::StrokeThenFill,
    ] {
        let mut tree = outline_tree();
        let outline = outline_mut(&mut tree);
        outline.placement.run_to_page.e = 5.0;
        outline.payload_kind = GlyphOutlinePayloadKind::MonochromeFillStroke;
        let stroke = GlyphOutlineStrokeStyle {
            color: 0x0000ff,
            width: 8.0,
            join: GlyphOutlineStrokeJoin::Miter,
            cap: GlyphOutlineStrokeCap::Butt,
            miter_limit: 4.0,
            paint_order: order,
        };
        assert!(stroke.is_strict_subset());
        outline.stroke = Some(stroke);
        let image = assert_selected(&tree);
        let expected = if order == GlyphOutlinePaintOrder::FillThenStroke {
            [255, 0, 0, 255]
        } else {
            [0, 0, 0, 255]
        };
        assert_eq!(image.get_pixel(17, 20).0, expected);
        assert_eq!(image.get_pixel(13, 20).0, [255, 0, 0, 255]);
    }
}

#[wasm_bindgen_test]
fn web_canvas_outline_replays_colrv0_layer_order_and_opacity() {
    let mut tree = outline_tree();
    let layer = |index, rgba, opacity| ColorLayerNode {
        layer_index: Some(index),
        glyph_id: Some(1),
        glyph_range: Some(GlyphRange::new(0, 1)),
        source_range_utf8: Some(TextSourceRange::new(0, 1)),
        source_font_ref: Some(source_font(ColorGlyphFormat::ColrV0)),
        commands: Some(rectangle(10.0, 10.0, 30.0, 30.0)),
        fill: Some(color(rgba)),
        fill_rule: Some(GlyphOutlineFillRule::NonZero),
        palette_index: None,
        color: None,
        opacity: Some(opacity),
        transform_to_run: None,
    };
    let payload = ColorLayersPayload {
        color_format: ColorGlyphFormat::ColrV0,
        source_font_ref: Some(source_font(ColorGlyphFormat::ColrV0)),
        palette_ref: None,
        layers: vec![
            layer(0, [1.0, 0.0, 0.0, 1.0], 1.0),
            layer(1, [0.0, 0.0, 1.0, 1.0], 0.5),
        ],
        paint_graph: None,
        source_range_utf8: Some(TextSourceRange::new(0, 1)),
        glyph_range: Some(GlyphRange::new(0, 1)),
    };
    assert!(payload.has_colrv0_resolved_layer_contract());
    let outline = outline_mut(&mut tree);
    outline.payload_kind = GlyphOutlinePayloadKind::ColorLayers;
    outline.paths.clear();
    outline.color_layers = Some(payload);
    let image = assert_selected(&tree);
    let pixel = image.get_pixel(20, 20);
    assert!((126..=129).contains(&pixel[0]) && (126..=129).contains(&pixel[2]));
    assert_eq!(pixel[1], 0);
}

#[wasm_bindgen_test]
fn web_canvas_outline_replays_each_colrv1_gradient_in_actual_canvas() {
    let stops = || {
        vec![
            ColorGradientStop {
                offset: 0.0,
                color: color([1.0, 0.0, 0.0, 1.0]),
            },
            ColorGradientStop {
                offset: 1.0,
                color: color([0.0, 0.0, 1.0, 1.0]),
            },
        ]
    };
    let mut images = Vec::new();
    for kind in [
        ColorPaintGraphNodeKind::LinearGradientPath,
        ColorPaintGraphNodeKind::RadialGradientPath,
        ColorPaintGraphNodeKind::SweepGradientPath,
    ] {
        let mut node = graph_node(1);
        node.kind = kind;
        match kind {
            ColorPaintGraphNodeKind::LinearGradientPath => {
                node.linear_gradient_path = Some(ColorPaintLinearGradientPathNode {
                    commands: rectangle(10.0, 10.0, 30.0, 30.0),
                    fill_rule: GlyphOutlineFillRule::NonZero,
                    source_glyph_id: Some(1),
                    palette_index: None,
                    gradient: ColorLinearGradient {
                        x0: 10.0,
                        y0: 10.0,
                        x1: 40.0,
                        y1: 10.0,
                        stops: stops(),
                    },
                })
            }
            ColorPaintGraphNodeKind::RadialGradientPath => {
                node.radial_gradient_path = Some(ColorPaintRadialGradientPathNode {
                    commands: rectangle(10.0, 10.0, 30.0, 30.0),
                    fill_rule: GlyphOutlineFillRule::NonZero,
                    source_glyph_id: Some(1),
                    palette_index: None,
                    gradient: ColorRadialGradient {
                        cx: 25.0,
                        cy: 25.0,
                        radius: 15.0,
                        stops: stops(),
                    },
                })
            }
            ColorPaintGraphNodeKind::SweepGradientPath => {
                node.sweep_gradient_path = Some(ColorPaintSweepGradientPathNode {
                    commands: rectangle(10.0, 10.0, 30.0, 30.0),
                    fill_rule: GlyphOutlineFillRule::NonZero,
                    source_glyph_id: Some(1),
                    palette_index: None,
                    gradient: ColorSweepGradient {
                        cx: 25.0,
                        cy: 25.0,
                        start_angle_degrees: 0.0,
                        end_angle_degrees: 360.0,
                        stops: stops(),
                    },
                })
            }
            _ => unreachable!(),
        }
        let mut tree = graph_tree(ColorPaintGraphPayload {
            root_node_id: 1,
            nodes: vec![node],
        });
        assert!(outline_mut(&mut tree)
            .color_layers
            .as_ref()
            .unwrap()
            .has_colrv1_supported_graph_contract());
        let image = assert_selected(&tree);
        assert_ne!(image.get_pixel(15, 20), image.get_pixel(25, 30), "{kind:?}");
        images.push(image);
    }
    assert!(images[0] != images[1] && images[1] != images[2]);
}

fn clipped_composite_tree() -> PageLayerTree {
    let mut composite = graph_node(3);
    composite.kind = ColorPaintGraphNodeKind::Composite;
    composite.composite = Some(ColorPaintCompositeNode {
        source_node_id: 2,
        backdrop_node_id: 1,
        mode: ColorPaintCompositeMode::SourceOver,
    });
    let mut transform = graph_node(4);
    transform.kind = ColorPaintGraphNodeKind::Transform;
    transform.transform = Some(ColorPaintTransformNode {
        child_node_id: 3,
        transform: LayerAffineTransform {
            e: 5.0,
            ..identity()
        },
    });
    let mut clip = graph_node(5);
    clip.kind = ColorPaintGraphNodeKind::Clip;
    let mut clip_commands = rectangle(10.0, 10.0, 30.0, 30.0);
    clip_commands.extend(rectangle(20.0, 15.0, 10.0, 20.0));
    clip.clip = Some(ColorPaintClipNode {
        child_node_id: 4,
        clip_commands,
        fill_rule: GlyphOutlineFillRule::EvenOdd,
    });
    let mut tree = graph_tree(ColorPaintGraphPayload {
        root_node_id: 5,
        nodes: vec![
            solid_node(1, [1.0, 0.0, 0.0, 1.0]),
            solid_node(2, [0.0, 0.0, 1.0, 0.5]),
            composite,
            transform,
            clip,
        ],
    });
    outline_mut(&mut tree).placement.run_to_page.e = 5.0;
    tree
}

#[wasm_bindgen_test]
fn web_canvas_outline_replays_transformed_evenodd_clip_and_source_over_atomically() {
    let tree = clipped_composite_tree();
    let image = assert_selected(&tree);
    for x in [22, 42] {
        let pixel = image.get_pixel(x, 20);
        assert!(
            (126..=129).contains(&pixel[0]) && (126..=129).contains(&pixel[2]),
            "x={x}: {pixel:?}"
        );
        assert_eq!(pixel[1], 0);
    }
    for x in [17, 30, 47] {
        assert_eq!(image.get_pixel(x, 20).0, [255, 255, 255, 255], "x={x}");
    }
    let mut cyclic = tree;
    outline_mut(&mut cyclic)
        .color_layers
        .as_mut()
        .unwrap()
        .paint_graph
        .as_mut()
        .unwrap()
        .nodes[4]
        .clip
        .as_mut()
        .unwrap()
        .child_node_id = 5;
    assert_fallback(&cyclic);
}

#[wasm_bindgen_test]
fn web_canvas_outline_replays_shared_dag_and_rejects_exponential_expansion() {
    let mut transform = graph_node(2);
    transform.kind = ColorPaintGraphNodeKind::Transform;
    transform.transform = Some(ColorPaintTransformNode {
        child_node_id: 1,
        transform: identity(),
    });
    let mut composite = graph_node(3);
    composite.kind = ColorPaintGraphNodeKind::Composite;
    composite.composite = Some(ColorPaintCompositeNode {
        source_node_id: 2,
        backdrop_node_id: 1,
        mode: ColorPaintCompositeMode::SourceOver,
    });
    let mut graph = ColorPaintGraphPayload {
        root_node_id: 3,
        nodes: vec![solid_node(1, [1.0, 0.0, 0.0, 0.5]), transform, composite],
    };
    assert!(graph.has_colrv1_supported_graph_contract());
    let image = assert_selected(&graph_tree(graph.clone()));
    let pixel = image.get_pixel(20, 20);
    assert_eq!(pixel[0], 255);
    assert!(
        (62..=65).contains(&pixel[1]) && (62..=65).contains(&pixel[2]),
        "{pixel:?}"
    );
    for id in 4..=22 {
        let mut node = graph_node(id);
        node.kind = ColorPaintGraphNodeKind::Composite;
        node.composite = Some(ColorPaintCompositeNode {
            source_node_id: id - 1,
            backdrop_node_id: id - 2,
            mode: ColorPaintCompositeMode::SourceOver,
        });
        graph.nodes.push(node);
        graph.root_node_id = id;
    }
    assert!(!graph.has_colrv1_supported_graph_contract());
    assert_fallback(&graph_tree(graph));
}

#[wasm_bindgen_test]
fn web_canvas_outline_decodes_bitmap_and_preserves_placement_and_transforms() {
    let encoded = blue_png();
    let mut tree = bitmap_tree(&encoded);
    let image = assert_selected(&tree);
    assert_eq!(image.get_pixel(30, 20).0, [0, 0, 255, 255]);
    assert_eq!(image.get_pixel(12, 12).0, [255, 255, 255, 255]);
    let outline = outline_mut(&mut tree);
    outline.placement.run_to_page.e = 5.0;
    // Bitmap placement is already page-space; an outline matrix must not
    // translate it twice. Only the bitmap payload's explicit transform applies.
    outline.bitmap_glyph.as_mut().unwrap().transform_to_run = Some(LayerAffineTransform {
        e: 5.0,
        ..identity()
    });
    let image = assert_selected(&tree);
    assert_eq!(image.get_pixel(27, 20).0, [255, 255, 255, 255]);
    assert_eq!(image.get_pixel(32, 20).0, [0, 0, 255, 255]);
    assert_eq!(image.get_pixel(52, 20).0, [255, 255, 255, 255]);
    assert_fallback(&bitmap_tree(b"invalid bitmap bytes"));
    let mut missing = bitmap_tree(&encoded);
    missing.resources = ResourceArena::default();
    assert_fallback(&missing);
}

#[wasm_bindgen_test]
fn web_canvas_outline_replays_svg_font_em_box_and_fails_closed_on_bad_resources() {
    let fragment = "<svg><path fill=\"#0000ff\" d=\"M10 20H40V50H10Z\"/></svg>";
    let mut tree = svg_tree(fragment);
    let image = assert_selected(&tree);
    assert_eq!(image.get_pixel(30, 20).0, [0, 0, 255, 255]);
    assert_eq!(image.get_pixel(12, 12).0, [255, 255, 255, 255]);
    let outline = outline_mut(&mut tree);
    outline.placement.run_to_page.e = 5.0;
    // font_em_box is page-space, matching the current producer/Native contract.
    outline.svg_glyph.as_mut().unwrap().transform_to_run = Some(LayerAffineTransform {
        e: 5.0,
        ..identity()
    });
    let image = assert_selected(&tree);
    assert_eq!(image.get_pixel(27, 20).0, [255, 255, 255, 255]);
    assert_eq!(image.get_pixel(32, 20).0, [0, 0, 255, 255]);
    assert_eq!(image.get_pixel(52, 20).0, [255, 255, 255, 255]);
    assert_fallback(&svg_tree("<svg><path d=\"Mbroken\"/></svg>"));
    let mut missing = svg_tree(fragment);
    missing.resources = ResourceArena::default();
    assert_fallback(&missing);
}

#[wasm_bindgen_test]
fn web_canvas_outline_late_prepare_failure_keeps_the_whole_textrun_fallback() {
    let mut tree = outline_tree();
    outline_mut(&mut tree).variant.part_count = 2;
    let mut late = outline_mut(&mut tree).clone();
    late.variant.part_index = 1;
    late.paths[0].commands = rectangle(50.0, 10.0, 20.0, 30.0);
    ops_mut(&mut tree).push(PaintOp::glyph_outline(
        BoundingBox::new(50.0, 10.0, 20.0, 30.0),
        late,
    ));
    let image = assert_selected(&tree);
    assert_eq!(image.get_pixel(20, 20).0, [0, 0, 0, 255]);
    assert_eq!(image.get_pixel(60, 20).0, [0, 0, 0, 255]);
    let PaintOp::GlyphOutline { outline, .. } = &mut ops_mut(&mut tree)[2] else {
        unreachable!()
    };
    // Finite producer geometry passes the common selector, but no drawable
    // segment can be prepared. The already-prepared first part must not paint.
    outline.paths[0].commands = vec![PathCommand::MoveTo(50.0, 10.0)];
    let shared = analyze_text_variant_selection(
        &tree,
        TextVariantSelectionOptions::canvas2d_strict_outline(),
    );
    assert_eq!(
        shared[0].selected_variant_kind,
        Some(TextVariantKind::GlyphOutline)
    );
    assert_fallback(&tree);
    let reports = render(&tree, true).1;
    assert!(reports[0].rejected_variants.iter().any(|rejected| rejected
        .reasons
        .contains(&VariantRejectReason::UnsupportedOutlinePayload)));
}

#[wasm_bindgen_test]
fn web_canvas_outline_restores_canvas_state_and_releases_prepared_tree_resources() {
    let (canvas, context) = canvas();
    let mut renderer = WebCanvasRenderer::new(&canvas).unwrap();
    renderer.set_strict_glyph_outline_replay(true);
    context.set_transform(1.0, 0.0, 0.0, 1.0, 3.0, 4.0).unwrap();
    context.set_global_alpha(1.0);
    context.set_global_composite_operation("multiply").unwrap();
    context.set_image_smoothing_enabled(true);
    renderer.render_layer_tree(&clipped_composite_tree());
    let matrix = context.get_transform().unwrap();
    assert_eq!(
        [
            matrix.a(),
            matrix.b(),
            matrix.c(),
            matrix.d(),
            matrix.e(),
            matrix.f()
        ],
        [1.0, 0.0, 0.0, 1.0, 3.0, 4.0]
    );
    assert_eq!(context.global_alpha(), 1.0);
    assert_eq!(context.global_composite_operation().unwrap(), "multiply");
    renderer.render_layer_tree(&bitmap_tree(&blue_png()));
    assert!(context.image_smoothing_enabled());
    // An independent marker outside the former graph clip proves clip restore.
    context.set_fill_style_str("#00ff00");
    context.fill_rect(70.0, 10.0, 8.0, 8.0);
    assert_eq!(pixels(&context).get_pixel(76, 17).0, [0, 255, 0, 255]);
    context.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
    context
        .set_global_composite_operation("source-over")
        .unwrap();
    renderer.render_layer_tree(&fallback_tree());
    assert!(renderer.get_text_variant_selection_diagnostics().is_empty());
    assert_eq!(renderer.text_variant_selection_diagnostics_dropped(), 0);
    assert_same_pixels(&pixels(&context), &render(&fallback_tree(), false).0);
    renderer.set_strict_glyph_outline_replay(false);
    renderer.render_layer_tree(&outline_tree());
    assert!(renderer.get_text_variant_selection_diagnostics().is_empty());
    assert_same_pixels(&pixels(&context), &render(&fallback_tree(), false).0);
}

#[wasm_bindgen_test]
fn web_canvas_outline_diagnostics_are_bounded_and_reset_on_the_next_tree() {
    let mut leaves = Vec::new();
    for index in 0..257 {
        let mut tree = outline_tree();
        let group = format!("text-{index}");
        let ops = ops_mut(&mut tree);
        let PaintOp::TextRun {
            source: Some(source),
            ..
        } = &mut ops[0]
        else {
            unreachable!()
        };
        source.id = TextSourceId(index);
        let PaintOp::GlyphOutline { outline, .. } = &mut ops[1] else {
            unreachable!()
        };
        outline.source.id = TextSourceId(index);
        outline.variant.equivalence_group = group.clone();
        outline.variant.anchor_op_id = Some(group);
        outline.variant.variant_id = format!("outline-{index}");
        leaves.push(tree.root);
    }
    let tree = PageLayerTree::new(
        f64::from(WIDTH),
        f64::from(HEIGHT),
        LayerNode::group(
            BoundingBox::new(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT)),
            None,
            leaves,
            CacheHint::None,
            GroupKind::Generic,
        ),
    );
    validate_text_variant_scope(&tree).unwrap();
    let (canvas, context) = canvas();
    let mut renderer = WebCanvasRenderer::new(&canvas).unwrap();
    renderer.set_strict_glyph_outline_replay(true);
    renderer.render_layer_tree(&tree);
    assert_eq!(renderer.get_text_variant_selection_diagnostics().len(), 256);
    assert_eq!(renderer.text_variant_selection_diagnostics_dropped(), 1);
    let image = pixels(&context);
    assert!(has_ink(&image, true));
    assert!(
        !has_ink(&image, false),
        "truncating diagnostic reports must not change selection for any of the 257 groups"
    );
    renderer.render_layer_tree(&outline_tree());
    assert_eq!(renderer.get_text_variant_selection_diagnostics().len(), 1);
    assert_eq!(renderer.text_variant_selection_diagnostics_dropped(), 0);
    renderer.set_strict_glyph_outline_replay(false);
    renderer.render_layer_tree(&tree);
    assert!(renderer.get_text_variant_selection_diagnostics().is_empty());
    assert_eq!(renderer.text_variant_selection_diagnostics_dropped(), 0);
}

#[wasm_bindgen_test]
fn public_document_canvas2d_outline_preview_renders_real_producer_and_preserves_default_api() {
    use rhwp::wasm_api::HwpDocument;

    // This checked-in one-page HWPX embeds the font's bitmap-only E100 glyph.
    // Its nominal GlyphRun is deliberately unavailable after display projection;
    // the bitmap GlyphOutline must therefore be selected by the actual producer.
    let document = HwpDocument::new(include_bytes!(
        "../../samples/render-p35-font-native-bitmap.hwpx"
    ))
    .expect("parse embedded-font HWPX");
    assert_eq!(document.page_count(), 1);
    let layer: serde_json::Value = serde_json::from_str(
        &document
            .get_page_layer_tree_with_profile(0, "screen", None, None)
            .expect("screen producer export"),
    )
    .unwrap();
    let expected_size = |scale: f64| {
        (
            (layer["pageWidth"].as_f64().unwrap() * scale).ceil() as u32,
            (layer["pageHeight"].as_f64().unwrap() * scale).ceil() as u32,
        )
    };
    let capture = |canvas: &HtmlCanvasElement, context: &CanvasRenderingContext2d| {
        image::RgbaImage::from_raw(
            canvas.width(),
            canvas.height(),
            context
                .get_image_data(
                    0.0,
                    0.0,
                    f64::from(canvas.width()),
                    f64::from(canvas.height()),
                )
                .unwrap()
                .data()
                .0,
        )
        .unwrap()
    };
    let (default_canvas, default_context) = canvas();
    document
        .render_page_to_canvas_filtered_with_profile(0, &default_canvas, 1.0, "all", "screen")
        .expect("existing default Canvas2D render");
    let default_before = capture(&default_canvas, &default_context);
    assert_eq!(default_before.dimensions(), expected_size(1.0));

    let (preview_canvas, preview_context) = canvas();
    let result = document
        .render_page_to_canvas_strict_outline_with_profile(0, &preview_canvas, 1.0, "screen")
        .expect("explicit document-owned preview");
    let report: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(report["backend"], "canvas2d");
    assert_eq!(report["strictGlyphOutlineReplay"], true);
    assert_eq!(report["reportsDropped"], 0);
    let groups = report["reports"].as_array().unwrap();
    assert!(
        !groups.is_empty(),
        "real font producer emits selection diagnostics"
    );
    assert!(groups.iter().all(|group| group["backend"] == "canvas2d"));
    assert!(
        groups.iter().any(|group| {
            group["selectedVariantKind"] == "glyphOutline" && group["fallbackRequired"] == false
        }),
        "actual document producer must select the embedded bitmap outline: {report}"
    );
    let preview = capture(&preview_canvas, &preview_context);
    assert_eq!(preview.dimensions(), expected_size(1.0));
    assert!(
        preview
            .pixels()
            .any(|pixel| { pixel[3] > 0 && pixel.0 != [255, 255, 255, 255] }),
        "document preview paints visible content"
    );

    // The public API intentionally normalizes nonpositive/nonfinite scales to
    // 1.0, matching the existing Canvas API. Success must mean the same image,
    // dimensions and diagnostics, rather than a stale or zero-sized canvas.
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let normalized = document
            .render_page_to_canvas_strict_outline_with_profile(0, &preview_canvas, scale, "screen")
            .expect("legacy invalid-scale normalization");
        let normalized: serde_json::Value = serde_json::from_str(&normalized).unwrap();
        assert_eq!(normalized, report);
        assert_same_pixels(&capture(&preview_canvas, &preview_context), &preview);
    }

    // Invalid page/profile requests must fail before resizing or clearing the
    // previously rendered canvas and cannot return misleading selection success.
    assert!(document
        .render_page_to_canvas_strict_outline_with_profile(
            0,
            &preview_canvas,
            1.0,
            "unknown-profile"
        )
        .is_err());
    assert_same_pixels(&capture(&preview_canvas, &preview_context), &preview);
    assert!(document
        .render_page_to_canvas_strict_outline_with_profile(
            document.page_count(),
            &preview_canvas,
            1.0,
            "screen"
        )
        .is_err());
    assert_same_pixels(&capture(&preview_canvas, &preview_context), &preview);

    document
        .render_page_to_canvas_strict_outline_with_profile(0, &preview_canvas, 0.5, "screen")
        .expect("fractional scale preview");
    assert_eq!(
        (preview_canvas.width(), preview_canvas.height()),
        expected_size(0.5)
    );
    document
        .render_page_to_canvas_filtered_with_profile(0, &default_canvas, 1.0, "all", "screen")
        .expect("existing default API after preview");
    assert_same_pixels(&capture(&default_canvas, &default_context), &default_before);
    assert_eq!(document.page_count(), 1);
}
