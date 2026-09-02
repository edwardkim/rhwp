//! Issue #6520: production SVG consumes the canonical `PageLayerTree` paint contract.

use rhwp::model::shape::TextWrap;
use rhwp::paint::{
    text_visual_replay_role, CacheHint, FontFaceKey, FontFallbackPolicyId, FontInstanceKey,
    FontPortabilityKind, FontRequest, FontResolver, GlyphCluster, GlyphRange, GlyphRunDiagnostics,
    GlyphRunReplayEligibility, GroupKind, LayerBuilder, LayerNode, LayerNodeKind,
    LayerOutputOptions, LayerPoint, LayerVector, PageLayerTree, PaintOp, RenderProfile,
    ResolvedFontFace, ResolvedGlyphRun, ScriptTag, ShapeKey, ShapingEngineId, TextDecorationKind,
    TextDirection, TextShapeLowerer, TextSourceId, TextSourceRange, TextSourceSpan,
    TextVariantQuality, TextVisualReplayRole, WritingMode,
};
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::layer_renderer::LayerRenderer;
use rhwp::renderer::render_tree::{
    BoundingBox, EquationNode, ImageNode, PageBackgroundNode, PageNode, PageRenderTree,
    PlaceholderNode, RawSvgNode, RectangleNode, RenderLayerInfo, RenderNode, RenderNodeType,
    TableNode, TextRunNode,
};
use rhwp::renderer::svg::FontEmbedMode;
use rhwp::renderer::svg_layer::SvgLayerRenderer;
use rhwp::renderer::{ShapeStyle, TextStyle};
use rhwp::wasm_api::HwpDocument;
use std::path::PathBuf;

fn text_run(text: &str) -> TextRunNode {
    TextRunNode {
        text: text.to_string(),
        style: TextStyle {
            font_family: "Issue 6520 Test".to_string(),
            font_size: 12.0,
            shade_color: 0x00FF_FFFF,
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
        baseline: 12.0,
        field_marker: Default::default(),
        layout_positions: None,
        display_text: None,
    }
}

fn svg_text(svg: &str) -> String {
    svg.split("<text")
        .skip(1)
        .filter_map(|element| element.split_once('>'))
        .filter_map(|(_, content)| content.split_once("</text>"))
        .map(|(text, _)| text)
        .collect()
}

fn collect_text_visual_roles(node: &LayerNode, roles: &mut Vec<TextVisualReplayRole>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect_text_visual_roles(child, roles);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => collect_text_visual_roles(child, roles),
        LayerNodeKind::Leaf { ops } => roles.extend(ops.iter().map(text_visual_replay_role)),
    }
}

fn collect_control_labels<'a>(node: &'a LayerNode, labels: &mut Vec<&'a str>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect_control_labels(child, labels);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => collect_control_labels(child, labels),
        LayerNodeKind::Leaf { ops } => labels.extend(ops.iter().filter_map(|op| match op {
            PaintOp::ControlLabel { label, .. } => Some(label.as_str()),
            _ => None,
        })),
    }
}

fn has_missing_picture(node: &LayerNode) -> bool {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => children.iter().any(has_missing_picture),
        LayerNodeKind::ClipRect { child, .. } => has_missing_picture(child),
        LayerNodeKind::Leaf { ops } => ops.iter().any(|op| {
            matches!(
                op,
                PaintOp::Placeholder { placeholder, .. }
                    if placeholder.kind
                        == rhwp::renderer::render_tree::PlaceholderKind::MissingPicture
            )
        }),
    }
}

fn body_clip(node: &LayerNode) -> Option<BoundingBox> {
    match &node.kind {
        LayerNodeKind::ClipRect {
            clip,
            clip_kind: rhwp::paint::ClipKind::Body,
            ..
        } => Some(*clip),
        LayerNodeKind::ClipRect { child, .. } => body_clip(child),
        LayerNodeKind::Group { children, .. } => children.iter().find_map(body_clip),
        LayerNodeKind::Leaf { .. } => None,
    }
}

#[cfg(feature = "native-skia")]
fn remove_underline_visual(node: &mut LayerNode) {
    match &mut node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                remove_underline_visual(child);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => remove_underline_visual(child),
        LayerNodeKind::Leaf { ops } => {
            ops.retain(|op| {
                !matches!(
                    op,
                    PaintOp::TextDecoration {
                        kind: TextDecorationKind::Underline,
                        ..
                    }
                )
            });
            for op in ops {
                match op {
                    PaintOp::TextRun { run, .. }
                    | PaintOp::CharOverlap { run, .. }
                    | PaintOp::TextControlMark { run, .. }
                    | PaintOp::TabLeader { run, .. }
                    | PaintOp::TextDecoration { run, .. } => {
                        run.style.underline = rhwp::model::style::UnderlineType::None;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn solid_rect(bounds: BoundingBox, color: u32) -> PaintOp {
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

#[test]
fn production_svg_routes_share_the_screen_layer_contract() {
    for sample in ["samples/para-001.hwp", "samples/lseg-05-tab.hwp"] {
        let bytes = std::fs::read(sample).expect("sample");
        let document = HwpDocument::from_bytes(&bytes).expect("parse sample");

        let production = document.render_page_svg_native(0).expect("production SVG");
        let explicit = document
            .render_page_svg_layer_with_profile_native(0, RenderProfile::Screen)
            .expect("explicit screen SVG");
        let font_route = document
            .render_page_svg_with_fonts(0, FontEmbedMode::None, &[])
            .expect("font-option SVG route");

        assert_eq!(production, explicit, "{sample}: default route diverged");
        assert_eq!(production, font_route, "{sample}: font route diverged");
        assert!(production.starts_with("<svg"));
        assert!(production.contains("body-clip-"));
    }
}

#[test]
fn production_svg_sorts_multiple_document_embedded_font_rules() {
    let bytes = std::fs::read("samples/render-p35-font-native-bitmap.hwpx").expect("sample");
    let mut document = HwpDocument::from_bytes(&bytes).expect("parse sample");
    let mut model = document.document().clone();
    let first_name = model.doc_info.font_faces[0][1].name.clone();
    let mut second_font = model.doc_info.font_faces[0][1].clone();
    second_font.name = "ZZZ Issue 6520 Embedded".to_string();
    model.doc_info.font_faces[1][1] = second_font;
    let paragraph = &mut model.sections[0].paragraphs[0];
    paragraph.text = "가A".to_string();
    paragraph.char_count = 2;
    paragraph.char_offsets = vec![0, 1];
    document.set_document(model);

    let first = document.render_page_svg_native(0).expect("first SVG");
    let second = document.render_page_svg_native(0).expect("second SVG");
    assert_eq!(first, second);
    let first_rule = first
        .find(&format!("@font-face {{ font-family: \"{first_name}\""))
        .expect("first embedded face rule");
    let second_rule = first
        .find("@font-face { font-family: \"ZZZ Issue 6520 Embedded\"")
        .expect("second embedded face rule");
    assert!(
        first_rule < second_rule,
        "embedded font rules must be name-sorted"
    );
}

#[test]
fn subset_font_bytes_are_deterministic_for_multiple_codepoints() {
    let bytes = std::fs::read("samples/para-001.hwp").expect("sample");
    let mut document = HwpDocument::from_bytes(&bytes).expect("parse sample");
    let mut model = document.document().clone();
    for font in model.doc_info.font_faces.iter_mut().flatten() {
        font.name = "한컴돋움".to_string();
        font.is_embedded = false;
        font.resolved_bin_data_id = None;
    }
    let paragraph = &mut model.sections[0].paragraphs[0];
    paragraph.text = "가나다ABC123".to_string();
    paragraph.char_count = 9;
    paragraph.char_offsets = (0..9).collect();
    document.set_document(model);
    let font_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ttfs/opensource");

    let outputs = (0..8)
        .map(|_| {
            document
                .render_page_svg_with_fonts(
                    0,
                    FontEmbedMode::Subset,
                    std::slice::from_ref(&font_path),
                )
                .expect("subset SVG")
        })
        .collect::<Vec<_>>();
    assert!(outputs[0].contains("data:font/opentype;base64,"));
    assert!(outputs.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn layer_profile_owns_visibility_and_text_marks() {
    let mut tree = PageRenderTree::new(0, 100.0, 80.0);
    tree.root.node_type = RenderNodeType::Page(PageNode {
        page_index: 0,
        width: 100.0,
        height: 80.0,
        section_index: 0,
    });
    tree.root.children.push(
        RenderNode::new(
            41,
            RenderNodeType::Rectangle(RectangleNode::new(
                0.0,
                ShapeStyle {
                    fill_color: Some(0x00445566),
                    ..Default::default()
                },
                None,
            )),
            BoundingBox::new(40.0, 0.0, 20.0, 20.0),
        )
        .with_editor_only(),
    );
    let mut marked = text_run("a b");
    marked.is_para_end = true;
    tree.root.children.push(RenderNode::new(
        42,
        RenderNodeType::TextRun(marked),
        BoundingBox::new(10.0, 20.0, 40.0, 20.0),
    ));
    tree.root.children.push(RenderNode::new(
        43,
        RenderNodeType::Placeholder(PlaceholderNode::missing_picture(None, None, None, None)),
        BoundingBox::new(60.0, 40.0, 20.0, 20.0),
    ));

    for profile in [
        RenderProfile::FastPreview,
        RenderProfile::Screen,
        RenderProfile::Print,
        RenderProfile::HighQuality,
    ] {
        let options = LayerOutputOptions {
            show_paragraph_marks: true,
            show_control_codes: true,
            ..Default::default()
        };
        let layer_tree = LayerBuilder::new(profile)
            .with_output_options(options)
            .build(&tree);
        assert_eq!(
            has_missing_picture(&layer_tree.root),
            profile.shows_editor_visuals(),
            "missing-picture visibility must be fixed by LayerBuilder"
        );
        let mut renderer = SvgLayerRenderer::new();
        renderer.render_page(&layer_tree).unwrap();
        let output = renderer.output();

        assert_eq!(
            output.contains("665544"),
            profile.shows_editor_visuals(),
            "profile={profile:?} must be decided by LayerBuilder"
        );
        assert!(output.contains('∨'));
        assert!(output.contains('↵'));
        assert_eq!(svg_text(output).matches("a").count(), 1);
        assert_eq!(
            output.contains("stroke-dasharray=\"2 2\""),
            profile.shows_editor_visuals(),
            "SVG must translate tree presence without another profile decision"
        );
    }

    #[cfg(feature = "native-skia")]
    {
        use rhwp::renderer::layer_renderer::LayerRasterRenderer;
        use rhwp::renderer::skia::SkiaLayerRenderer;

        let mut missing_only = PageRenderTree::new(0, 100.0, 80.0);
        missing_only.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 100.0,
            height: 80.0,
            section_index: 0,
        });
        missing_only.root.children.push(RenderNode::new(
            43,
            RenderNodeType::Placeholder(PlaceholderNode::missing_picture(None, None, None, None)),
            BoundingBox::new(60.0, 40.0, 20.0, 20.0),
        ));
        let screen = LayerBuilder::new(RenderProfile::Screen).build(&missing_only);
        let print = LayerBuilder::new(RenderProfile::Print).build(&missing_only);
        let screen = SkiaLayerRenderer::new().render_png(&screen).unwrap();
        let print = SkiaLayerRenderer::new().render_png(&print).unwrap();
        assert_ne!(
            screen, print,
            "Skia must translate profile-owned tree presence"
        );
    }
}

#[test]
fn layer_builder_char_overlap_selects_only_the_explicit_visual() {
    let mut tree = PageRenderTree::new(0, 80.0, 60.0);
    tree.root.node_type = RenderNodeType::Page(PageNode {
        page_index: 0,
        width: 80.0,
        height: 60.0,
        section_index: 0,
    });
    let mut overlap = text_run("X");
    overlap.rotation = 25.0;
    overlap.char_overlap = Some(rhwp::renderer::composer::CharOverlapInfo {
        border_type: 1,
        inner_char_size: 100,
    });
    tree.root.children.push(RenderNode::new(
        44,
        RenderNodeType::TextRun(overlap),
        BoundingBox::new(10.0, 10.0, 20.0, 20.0),
    ));

    let layer_tree = LayerBuilder::new(RenderProfile::Screen).build(&tree);
    let mut roles = Vec::new();
    collect_text_visual_roles(&layer_tree.root, &mut roles);
    assert!(roles.contains(&TextVisualReplayRole::SuppressedFallback));
    assert!(roles.contains(&TextVisualReplayRole::CharOverlap));
    let mut renderer = SvgLayerRenderer::new();
    renderer.render_page(&layer_tree).unwrap();
    let output = renderer.output();

    assert_eq!(svg_text(output).matches('X').count(), 1, "{output}");
    assert!(output.contains("<ellipse"), "{output}");
    assert!(output.contains("transform=\"rotate(25"), "{output}");
}

#[test]
fn rotated_text_replays_glyph_once_and_rotates_explicit_visuals() {
    let mut tree = PageRenderTree::new(0, 100.0, 80.0);
    tree.root.node_type = RenderNodeType::Page(PageNode {
        page_index: 0,
        width: 100.0,
        height: 80.0,
        section_index: 0,
    });
    let mut run = text_run("X ");
    run.rotation = 30.0;
    run.is_para_end = true;
    run.style.underline = rhwp::model::style::UnderlineType::Bottom;
    tree.root.children.push(RenderNode::new(
        46,
        RenderNodeType::TextRun(run),
        BoundingBox::new(20.0, 20.0, 40.0, 20.0),
    ));
    let layer_tree = LayerBuilder::new(RenderProfile::Screen)
        .with_output_options(LayerOutputOptions {
            show_paragraph_marks: true,
            ..Default::default()
        })
        .build(&tree);
    let mut roles = Vec::new();
    collect_text_visual_roles(&layer_tree.root, &mut roles);
    assert!(roles.contains(&TextVisualReplayRole::BaseText));
    assert!(roles.contains(&TextVisualReplayRole::ControlMark));
    assert!(roles.contains(&TextVisualReplayRole::Decoration(
        TextDecorationKind::Underline
    )));
    let mut renderer = SvgLayerRenderer::new();
    renderer.render_page(&layer_tree).unwrap();
    let output = renderer.output();

    assert_eq!(svg_text(output).matches('X').count(), 1, "{output}");
    assert!(output.contains("<line "), "{output}");
    assert!(output.contains('↵'), "{output}");
    assert!(
        output.matches("transform=\"rotate(30").count() >= 3,
        "glyph and both explicit visuals must share the rotation: {output}"
    );

    #[cfg(feature = "native-skia")]
    {
        use rhwp::renderer::layer_renderer::LayerRasterRenderer;
        use rhwp::renderer::skia::SkiaLayerRenderer;

        let decorated = SkiaLayerRenderer::new().render_png(&layer_tree).unwrap();
        let mut undecorated_tree = layer_tree.clone();
        remove_underline_visual(&mut undecorated_tree.root);
        undecorated_tree.text_sources =
            rhwp::paint::TextSourceTable::from_layer_node(&undecorated_tree.root);
        let undecorated = SkiaLayerRenderer::new()
            .render_png(&undecorated_tree)
            .unwrap();
        let decorated = image::load_from_memory(&decorated).unwrap().to_rgba8();
        let undecorated = image::load_from_memory(&undecorated).unwrap().to_rgba8();
        assert!(
            decorated
                .pixels()
                .zip(undecorated.pixels())
                .any(|(left, right)| left != right),
            "native Skia must replay the rotated underline sidecar"
        );
    }
}

#[test]
fn same_bounds_visual_op_cannot_cover_a_different_text_run() {
    let bounds = BoundingBox::new(10.0, 10.0, 20.0, 20.0);
    let mut fallback = text_run("X");
    fallback.char_overlap = Some(rhwp::renderer::composer::CharOverlapInfo {
        border_type: 1,
        inner_char_size: 100,
    });
    let mut different = fallback.clone();
    different.char_overlap = Some(rhwp::renderer::composer::CharOverlapInfo {
        border_type: 3,
        inner_char_size: 100,
    });
    let tree = PageLayerTree::new(
        80.0,
        60.0,
        LayerNode::leaf(
            bounds,
            Some(45),
            vec![
                PaintOp::text_run(bounds, fallback),
                PaintOp::char_overlap(bounds, different),
            ],
        ),
    );

    let error = SvgLayerRenderer::new()
        .render_page(&tree)
        .expect_err("same bounds must not substitute for exact TextRun identity");
    assert!(error
        .to_string()
        .contains("requires exactly one explicit canonical CharOverlap paint"));
}

#[test]
fn duplicate_explicit_text_visual_is_rejected() {
    let bounds = BoundingBox::new(10.0, 10.0, 40.0, 20.0);
    let mut run = text_run("DUPLICATE");
    run.style.underline = rhwp::model::style::UnderlineType::Bottom;
    let tree = PageLayerTree::new(
        80.0,
        60.0,
        LayerNode::leaf(
            bounds,
            Some(81),
            vec![
                PaintOp::text_run(bounds, run.clone()),
                PaintOp::text_decoration(bounds, run.clone(), TextDecorationKind::Underline),
                PaintOp::text_decoration(bounds, run, TextDecorationKind::Underline),
            ],
        ),
    );

    let error = SvgLayerRenderer::new()
        .render_page(&tree)
        .expect_err("one canonical visual role cannot replay twice");
    assert!(error.to_string().contains("underline paint, found 2"));
}

#[test]
fn identical_runs_at_identical_bounds_keep_distinct_visual_sources() {
    let bounds = BoundingBox::new(10.0, 10.0, 40.0, 20.0);
    let mut run = text_run("SAME");
    run.style.underline = rhwp::model::style::UnderlineType::Bottom;
    let tree = PageLayerTree::new(
        80.0,
        60.0,
        LayerNode::leaf(
            bounds,
            Some(83),
            vec![
                PaintOp::text_run(bounds, run.clone()),
                PaintOp::text_decoration(bounds, run.clone(), TextDecorationKind::Underline),
                PaintOp::text_run(bounds, run.clone()),
                PaintOp::text_decoration(bounds, run, TextDecorationKind::Underline),
            ],
        ),
    );
    let LayerNodeKind::Leaf { ops } = &tree.root.kind else {
        panic!("expected leaf");
    };
    let visual_sources = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::TextDecoration {
                source: Some(source),
                ..
            } => Some(source.id.0),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(visual_sources, vec![0, 1]);

    let mut renderer = SvgLayerRenderer::new();
    renderer
        .render_page(&tree)
        .expect("source identity disambiguates identical paint payloads");
    assert_eq!(renderer.output().matches("<line ").count(), 2);
}

#[test]
fn orphan_explicit_text_visual_is_rejected() {
    let bounds = BoundingBox::new(10.0, 10.0, 40.0, 20.0);
    let mut run = text_run("ORPHAN");
    run.style.underline = rhwp::model::style::UnderlineType::Bottom;
    let tree = PageLayerTree::new(
        80.0,
        60.0,
        LayerNode::leaf(
            bounds,
            Some(82),
            vec![PaintOp::text_decoration(
                bounds,
                run,
                TextDecorationKind::Underline,
            )],
        ),
    );

    let error = SvgLayerRenderer::new()
        .render_page(&tree)
        .expect_err("an explicit visual must belong to one fallback slot");
    assert!(error
        .to_string()
        .contains("explicit canonical underline paint"));
    assert!(error.to_string().contains("belongs to 0 TextRuns"));
}

#[test]
fn raw_svg_placeholder_and_explicit_text_visual_ops_are_serialized() {
    let bounds = BoundingBox::new(10.0, 10.0, 80.0, 20.0);
    let mut overlap = text_run("AB");
    overlap.char_overlap = Some(rhwp::renderer::composer::CharOverlapInfo {
        border_type: 1,
        inner_char_size: 100,
    });
    let mut control_mark = text_run(" ");
    control_mark.is_para_end = true;
    let mut leader = text_run("\titem");
    leader
        .style
        .tab_leaders
        .push(rhwp::renderer::TabLeaderInfo {
            start_x: 0.0,
            end_x: 40.0,
            fill_type: 2,
        });
    let mut underline = text_run("decorated");
    underline.style.underline = rhwp::model::style::UnderlineType::Bottom;
    let root = LayerNode::leaf(
        bounds,
        Some(71),
        vec![
            PaintOp::raw_svg(
                bounds,
                RawSvgNode::new(
                    "<g><circle cx=\"20\" cy=\"20\" r=\"8\" fill=\"#ff0000\"/></g>\n".to_string(),
                ),
            ),
            PaintOp::placeholder(
                BoundingBox::new(50.0, 30.0, 40.0, 20.0),
                PlaceholderNode::new(0x00F8F8F8, 0x00000000, "OLE".to_string()),
            ),
            PaintOp::text_run(bounds, overlap.clone()),
            PaintOp::char_overlap(bounds, overlap),
            PaintOp::text_run(bounds, control_mark.clone()),
            PaintOp::text_control_mark(bounds, control_mark),
            PaintOp::text_run(bounds, leader.clone()),
            PaintOp::text_control_mark(bounds, leader.clone()),
            PaintOp::tab_leader(bounds, leader),
            PaintOp::text_run(bounds, underline.clone()),
            PaintOp::text_decoration(bounds, underline, TextDecorationKind::Underline),
        ],
    );
    let tree = PageLayerTree::new(120.0, 80.0, root).with_output_options(LayerOutputOptions {
        show_paragraph_marks: true,
        ..Default::default()
    });
    let mut renderer = SvgLayerRenderer::new();
    renderer.render_page(&tree).unwrap();
    let svg = renderer.output();

    assert!(svg.contains("<circle cx=\"20\""), "{svg}");
    assert!(svg.contains(">OLE</text>"), "{svg}");
    assert!(svg.contains('∨'), "{svg}");
    assert!(svg.contains("stroke-dasharray=\"3 3\""), "{svg}");
    assert!(svg.contains("<ellipse"), "{svg}");
    assert!(svg.contains("<line"), "{svg}");
}

struct EmittingGlyphResolver;

impl FontResolver for EmittingGlyphResolver {
    fn resolve_font(&self, _request: &FontRequest) -> ResolvedFontFace {
        ResolvedFontFace {
            portability: FontPortabilityKind::PortableBlob,
        }
    }

    fn shape_glyph_run(
        &self,
        _request: &FontRequest,
        run: &TextRunNode,
        _resolved: &ResolvedFontFace,
    ) -> Option<ResolvedGlyphRun> {
        Some(ResolvedGlyphRun {
            shape_key: ShapeKey {
                font_instance: FontInstanceKey {
                    face_key: FontFaceKey("issue-6520-face".to_string()),
                    size_px: run.style.font_size,
                    variations: Vec::new(),
                    synthetic_bold: false,
                    synthetic_italic: false,
                },
                direction: TextDirection::Ltr,
                writing_mode: WritingMode::HorizontalTb,
                script: Some(ScriptTag("DFLT".to_string())),
                language: None,
                features: Vec::new(),
                shaping_engine: ShapingEngineId("issue-6520-test".to_string()),
                fallback_policy: FontFallbackPolicyId("none".to_string()),
            },
            glyph_ids: vec![42],
            positions: vec![LayerPoint { x: 0.0, y: 0.0 }],
            advances: Some(vec![LayerVector { dx: 12.0, dy: 0.0 }]),
            clusters: vec![GlyphCluster {
                source_range_utf8: TextSourceRange::new(0, run.text.len() as u32),
                source_range_utf16: Some(TextSourceRange::new(
                    0,
                    run.text.encode_utf16().count() as u32,
                )),
                text_range_utf8: Some(TextSourceRange::new(0, run.text.len() as u32)),
                glyph_range: GlyphRange::new(0, 1),
                flags: Vec::new(),
            }],
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
        })
    }
}

#[test]
fn image_and_equation_control_labels_replay_as_canonical_ops() {
    let bounds = BoundingBox::new(10.0, 10.0, 30.0, 20.0);
    let mut render_tree = PageRenderTree::new(0, 100.0, 100.0);
    render_tree.root.node_type = RenderNodeType::Page(PageNode {
        page_index: 0,
        width: 100.0,
        height: 100.0,
        section_index: 0,
    });
    render_tree.root.children = vec![
        RenderNode::new(201, RenderNodeType::Image(ImageNode::new(1, None)), bounds),
        RenderNode::new(
            202,
            RenderNodeType::Equation(EquationNode {
                svg_content: "<text>x</text>".to_string(),
                layout_box: LayoutBox {
                    x: 0.0,
                    y: 0.0,
                    width: 8.0,
                    height: 12.0,
                    baseline: 10.0,
                    kind: LayoutKind::Text("x".to_string()),
                },
                color_str: "#000000".to_string(),
                color: 0,
                font_size: 12.0,
                script: String::new(),
                section_index: None,
                para_index: None,
                control_index: None,
                cell_index: None,
                cell_para_index: None,
                note_ref: None,
            }),
            bounds,
        ),
        RenderNode::new(
            203,
            RenderNodeType::Table(TableNode {
                row_count: 1,
                col_count: 1,
                border_fill_id: 0,
                section_index: None,
                para_index: None,
                control_index: None,
                cell_context: None,
            }),
            bounds,
        ),
        RenderNode::new(204, RenderNodeType::TextBox, bounds),
        RenderNode::new(205, RenderNodeType::Header, bounds),
        RenderNode::new(206, RenderNodeType::Footer, bounds),
        RenderNode::new(207, RenderNodeType::FootnoteArea, bounds),
    ];
    let tree = LayerBuilder::new(RenderProfile::Screen)
        .with_output_options(LayerOutputOptions {
            show_control_codes: true,
            ..Default::default()
        })
        .build(&render_tree);
    let mut labels = Vec::new();
    collect_control_labels(&tree.root, &mut labels);
    assert_eq!(
        labels,
        vec![
            "[그림]",
            "[수식]",
            "[표]",
            "[글상자]",
            "[머리말]",
            "[꼬리말]",
            "[각주]"
        ]
    );
    let plain = LayerBuilder::new(RenderProfile::Screen).build(&render_tree);
    let mut plain_labels = Vec::new();
    collect_control_labels(&plain.root, &mut plain_labels);
    assert!(plain_labels.is_empty());

    let mut svg = SvgLayerRenderer::new();
    svg.render_page(&tree).unwrap();
    assert!(svg.output().contains("[그림]"));
    assert!(svg.output().contains("[수식]"));
    let json = tree.to_json();
    assert_eq!(json.matches("\"type\":\"controlLabel\"").count(), 7);
    let json: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(json["schemaMinorVersion"], 23);
    assert!(json["usedFeatures"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "paint.controlLabel"));
    assert!(json["knownFeatures"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "paint.controlLabel"));

    #[cfg(feature = "native-skia")]
    {
        use rhwp::renderer::layer_renderer::LayerRasterRenderer;
        use rhwp::renderer::skia::SkiaLayerRenderer;

        let png = SkiaLayerRenderer::new().render_png(&tree).unwrap();
        let image = image::load_from_memory(&png).unwrap().to_rgba8();
        assert!(image.pixels().any(|pixel| pixel[3] > 0));
    }

    let escaped = PageLayerTree::new(
        80.0,
        30.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 80.0, 30.0),
            Some(208),
            vec![PaintOp::control_label(bounds, "A&B <x>")],
        ),
    );
    let mut escaped_svg = SvgLayerRenderer::new();
    escaped_svg.render_page(&escaped).unwrap();
    assert!(escaped_svg.output().contains("A&amp;B &lt;x&gt;"));
    assert!(!escaped_svg.output().contains("A&B <x>"));
}

#[test]
fn glyph_native_variant_selects_validated_text_fallback_or_fails_closed() {
    let bounds = BoundingBox::new(10.0, 10.0, 120.0, 20.0);
    let mut root = LayerNode::leaf(
        bounds,
        Some(70),
        vec![PaintOp::text_run(
            bounds,
            text_run("FALLBACK_SENTINEL_6520"),
        )],
    );
    let report = TextShapeLowerer::new(&EmittingGlyphResolver).lower_root(&mut root);
    assert_eq!(report.public_glyph_run_count(), 1);

    let mut renderer = SvgLayerRenderer::new();
    renderer
        .render_page(&PageLayerTree::new(160.0, 80.0, root.clone()))
        .unwrap();
    assert!(svg_text(renderer.output()).contains("FALLBACK_SENTINEL_6520"));

    let LayerNodeKind::Leaf { ops } = &mut root.kind else {
        panic!("expected leaf");
    };
    ops.retain(|op| !matches!(op, PaintOp::TextRun { .. }));
    let error = SvgLayerRenderer::new()
        .render_page(&PageLayerTree::new(160.0, 80.0, root))
        .expect_err("glyph-only variants must not disappear into an empty SVG");
    assert!(error.to_string().contains("no default fallback"), "{error}");
}

#[test]
fn unrelated_text_run_cannot_cover_a_glyph_variant_source() {
    let bounds = BoundingBox::new(10.0, 10.0, 120.0, 20.0);
    let mut root = LayerNode::leaf(
        bounds,
        Some(74),
        vec![
            PaintOp::text_run(bounds, text_run("FIRST")),
            PaintOp::text_run(bounds, text_run("SECOND")),
        ],
    );
    let report = TextShapeLowerer::new(&EmittingGlyphResolver).lower_root(&mut root);
    assert_eq!(report.public_glyph_run_count(), 2);
    let mut valid = SvgLayerRenderer::new();
    valid
        .render_page(&PageLayerTree::new(160.0, 80.0, root.clone()))
        .expect("multiple independently paired text slots are valid");
    assert!(svg_text(valid.output()).contains("FIRSTSECOND"));
    let LayerNodeKind::Leaf { ops } = &mut root.kind else {
        panic!("expected leaf");
    };
    let mut kept_first_text = false;
    ops.retain(|op| match op {
        PaintOp::TextRun { .. } if !kept_first_text => {
            kept_first_text = true;
            true
        }
        PaintOp::GlyphRun { run, .. } => run.source.id.0 == 1,
        _ => false,
    });

    let error = SvgLayerRenderer::new()
        .render_page(&PageLayerTree::new(160.0, 80.0, root))
        .expect_err("an unrelated fallback must not hide a missing glyph source");
    assert!(
        error
            .to_string()
            .contains("does not match canonical text source 1"),
        "{error}"
    );
}

#[test]
fn identical_text_slots_keep_distinct_fallback_source_ids() {
    let first_bounds = BoundingBox::new(10.0, 10.0, 120.0, 20.0);
    let second_bounds = BoundingBox::new(10.0, 40.0, 120.0, 20.0);
    let mut root = LayerNode::leaf(
        BoundingBox::new(10.0, 10.0, 120.0, 50.0),
        Some(76),
        vec![
            PaintOp::text_run(first_bounds, text_run("SAME")),
            PaintOp::text_run(second_bounds, text_run("SAME")),
        ],
    );
    let report = TextShapeLowerer::new(&EmittingGlyphResolver).lower_root(&mut root);
    assert_eq!(report.public_glyph_run_count(), 2);
    let tree = PageLayerTree::new(160.0, 90.0, root);
    let LayerNodeKind::Leaf { ops } = &tree.root.kind else {
        panic!("expected leaf");
    };
    let source_ids = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::TextRun {
                source: Some(source),
                ..
            } => Some(source.id.0),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(source_ids, vec![0, 1]);

    let mut renderer = SvgLayerRenderer::new();
    renderer.render_page(&tree).unwrap();
    assert!(svg_text(renderer.output()).contains("SAMESAME"));
}

#[test]
fn duplicate_prebound_source_ids_are_rejected_across_leaves() {
    let first_bounds = BoundingBox::new(10.0, 10.0, 120.0, 20.0);
    let second_bounds = BoundingBox::new(10.0, 40.0, 120.0, 20.0);
    let run = text_run("SAME");
    let duplicate_source = TextSourceSpan::for_text_run(7, &run);
    let mut text_op = |bounds| {
        let mut op = PaintOp::text_run(bounds, run.clone());
        let PaintOp::TextRun { source, .. } = &mut op else {
            unreachable!();
        };
        *source = Some(duplicate_source.clone());
        op
    };
    let mut root = LayerNode::group(
        BoundingBox::new(10.0, 10.0, 120.0, 50.0),
        Some(78),
        vec![
            LayerNode::leaf(first_bounds, Some(79), vec![text_op(first_bounds)]),
            LayerNode::leaf(second_bounds, Some(80), vec![text_op(second_bounds)]),
        ],
        rhwp::paint::CacheHint::None,
        rhwp::paint::GroupKind::Generic,
    );
    let report = TextShapeLowerer::new(&EmittingGlyphResolver).lower_root(&mut root);
    assert_eq!(report.public_glyph_run_count(), 2);

    let error = SvgLayerRenderer::new()
        .render_page(&PageLayerTree::new(160.0, 90.0, root))
        .expect_err("duplicate source identities must fail before backend selection");
    assert!(
        error
            .to_string()
            .contains("canonical text source 7 is bound in both"),
        "{error}"
    );
}

#[test]
fn text_shape_lowerer_preserves_sparse_prebound_source_ids() {
    let first_bounds = BoundingBox::new(10.0, 10.0, 120.0, 20.0);
    let second_bounds = BoundingBox::new(10.0, 40.0, 120.0, 20.0);
    let mut first_run = text_run("BOUND");
    first_run.section_index = Some(2);
    first_run.para_index = Some(3);
    first_run.char_start = Some(5);
    let mut first = PaintOp::text_run(first_bounds, first_run.clone());
    let PaintOp::TextRun { source, .. } = &mut first else {
        unreachable!();
    };
    *source = Some(TextSourceSpan::for_text_run(7, &first_run));
    let mut root = LayerNode::leaf(
        BoundingBox::new(10.0, 10.0, 120.0, 50.0),
        Some(77),
        vec![first, PaintOp::text_run(second_bounds, text_run("NEXT"))],
    );
    let report = TextShapeLowerer::new(&EmittingGlyphResolver).lower_root(&mut root);
    assert_eq!(report.public_glyph_run_count(), 2);
    let LayerNodeKind::Leaf { ops } = &root.kind else {
        panic!("expected leaf");
    };
    let text_ids = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::TextRun {
                source: Some(source),
                ..
            } => Some(source.id.0),
            _ => None,
        })
        .collect::<Vec<_>>();
    let glyph_ids = ops
        .iter()
        .filter_map(|op| match op {
            PaintOp::GlyphRun { run, .. } => Some(run.source.id.0),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(text_ids, vec![7, 8]);
    assert_eq!(glyph_ids, vec![7, 8]);
    let source_pairs = ops
        .chunks_exact(2)
        .map(|pair| match pair {
            [PaintOp::TextRun {
                source: Some(fallback),
                ..
            }, PaintOp::GlyphRun { run: glyph, .. }] => (fallback, &glyph.source),
            _ => panic!("expected one fallback and glyph alternative per source"),
        })
        .collect::<Vec<_>>();
    assert!(source_pairs
        .iter()
        .all(|(fallback, glyph)| fallback == glyph));
}

#[test]
fn missing_middle_fallback_cannot_renumber_a_later_text_run_into_place() {
    let bounds = BoundingBox::new(10.0, 10.0, 120.0, 20.0);
    let mut root = LayerNode::leaf(
        bounds,
        Some(75),
        vec![
            PaintOp::text_run(bounds, text_run("FIRST")),
            PaintOp::text_run(bounds, text_run("MIDDLE")),
            PaintOp::text_run(bounds, text_run("THIRD")),
        ],
    );
    let report = TextShapeLowerer::new(&EmittingGlyphResolver).lower_root(&mut root);
    assert_eq!(report.public_glyph_run_count(), 3);
    let LayerNodeKind::Leaf { ops } = &mut root.kind else {
        panic!("expected leaf");
    };
    let mut text_index = 0;
    ops.retain(|op| match op {
        PaintOp::TextRun { .. } => {
            let keep = text_index != 1;
            text_index += 1;
            keep
        }
        PaintOp::GlyphRun { run, .. } => run.source.id.0 == 1,
        _ => false,
    });

    let error = SvgLayerRenderer::new()
        .render_page(&PageLayerTree::new(160.0, 80.0, root))
        .expect_err("later fallback renumbering must not satisfy a missing middle slot");
    assert!(
        error
            .to_string()
            .contains("does not match canonical text source 1"),
        "{error}"
    );
}

#[test]
fn direct_replay_preserves_plane_order_nested_clips_and_debug_projection() {
    let page = BoundingBox::new(0.0, 0.0, 120.0, 80.0);
    let content = BoundingBox::new(10.0, 10.0, 80.0, 40.0);
    let front =
        LayerNode::leaf(content, Some(41), vec![solid_rect(content, 0x00AA00AA)]).with_layer(Some(
            RenderLayerInfo::new(Some(TextWrap::InFrontOfText), 3, 3),
        ));
    let flow = LayerNode::clip_rect(
        content,
        Some(33),
        content,
        LayerNode::leaf(content, Some(32), vec![solid_rect(content, 0x000000AA)]),
        rhwp::paint::ClipKind::Generic,
    );
    let background = LayerNode::leaf(
        page,
        Some(11),
        vec![PaintOp::page_background(
            page,
            PageBackgroundNode {
                background_color: Some(0x00AA0000),
                border_color: None,
                border_width: 0.0,
                gradient: None,
                image: None,
            },
        )],
    );
    let behind = LayerNode::leaf(content, Some(21), vec![solid_rect(content, 0x0000AA00)])
        .with_layer(Some(RenderLayerInfo::new(Some(TextWrap::BehindText), 1, 1)));
    let flow_line = LayerNode::group(
        content,
        Some(34),
        vec![flow],
        CacheHint::None,
        GroupKind::TextLine(rhwp::renderer::render_tree::TextLineNode::with_para(
            20.0, 15.0, 0, 7,
        )),
    );
    let clipped = LayerNode::clip_rect(
        content,
        None,
        content,
        LayerNode::group(
            content,
            Some(30),
            vec![flow_line, behind],
            CacheHint::None,
            GroupKind::Body,
        ),
        rhwp::paint::ClipKind::Body,
    );
    // Deliberately shuffled: the shared replay planes, not tree insertion order, own stacking.
    let root = LayerNode::group(
        page,
        Some(1),
        vec![front, clipped, background],
        CacheHint::None,
        GroupKind::Generic,
    );
    let tree = PageLayerTree::with_profile(120.0, 80.0, root, RenderProfile::Screen)
        .with_output_options(LayerOutputOptions {
            debug_overlay: true,
            ..Default::default()
        });
    let mut renderer = SvgLayerRenderer::new();
    renderer.render_page(&tree).unwrap();
    let svg = renderer.output();

    let background_at = svg.find("#0000aa").expect("page background");
    let behind_at = svg.find("#00aa00").expect("behind-text rect");
    let flow_at = svg.find("#aa0000").expect("flow rect");
    let front_at = svg.find("#aa00aa").expect("front rect");
    assert!(background_at < behind_at && behind_at < flow_at && flow_at < front_at);
    assert!(svg.contains("id=\"body-clip-1000000-behindText\""), "{svg}");
    assert!(svg.contains("id=\"body-clip-1000000-flow\""), "{svg}");
    assert!(svg.contains("id=\"generic-clip-33\""), "{svg}");
    assert!(svg.contains("<g id=\"debug-overlay\""), "{svg}");
    assert!(svg.contains("s0:pi=7"), "{svg}");
}

#[test]
fn bare_master_group_does_not_create_an_svg_only_replay_plane() {
    let bounds = BoundingBox::new(0.0, 0.0, 40.0, 40.0);
    let bare_master = LayerNode::group(
        bounds,
        Some(80),
        vec![LayerNode::leaf(
            bounds,
            Some(81),
            vec![solid_rect(bounds, 0x00112233)],
        )],
        CacheHint::None,
        GroupKind::MasterPage,
    );
    let behind = LayerNode::leaf(bounds, Some(82), vec![solid_rect(bounds, 0x00445566)])
        .with_layer(Some(RenderLayerInfo::new(Some(TextWrap::BehindText), 0, 0)));
    let tree = PageLayerTree::new(
        40.0,
        40.0,
        LayerNode::group(
            bounds,
            Some(79),
            vec![bare_master, behind],
            CacheHint::None,
            GroupKind::Generic,
        ),
    );
    let mut renderer = SvgLayerRenderer::new();
    renderer.render_page(&tree).unwrap();
    let output = renderer.output();

    assert!(output.find("#665544").unwrap() < output.find("#332211").unwrap());
}

#[test]
fn soft_wrap_decoration_trim_does_not_depend_on_source_node_id() {
    fn render(source_node_id: u32) -> (PageLayerTree, String) {
        let bounds = BoundingBox::new(10.0, 10.0, 60.0, 20.0);
        let mut run = text_run("X  ");
        run.style.underline = rhwp::model::style::UnderlineType::Bottom;
        let mut page = PageRenderTree::new(0, 80.0, 60.0);
        page.root.node_type = RenderNodeType::Page(PageNode {
            page_index: 0,
            width: 80.0,
            height: 60.0,
            section_index: 0,
        });
        let mut line = RenderNode::new(
            90,
            RenderNodeType::TextLine(rhwp::renderer::render_tree::TextLineNode::new(20.0, 15.0)),
            bounds,
        );
        line.children.push(RenderNode::new(
            source_node_id,
            RenderNodeType::TextRun(run),
            bounds,
        ));
        page.root.children.push(line);
        let tree = LayerBuilder::new(RenderProfile::Screen).build(&page);
        let mut renderer = SvgLayerRenderer::new();
        renderer.render_page(&tree).unwrap();
        (tree, renderer.output().to_string())
    }

    fn underline_x2(svg: &str) -> f64 {
        let line = svg
            .split("<line ")
            .nth(1)
            .and_then(|element| element.split_once("/>"))
            .map(|(element, _)| element)
            .expect("underline line");
        let value = line
            .split("x2=\"")
            .nth(1)
            .and_then(|suffix| suffix.split_once('"'))
            .map(|(value, _)| value)
            .expect("underline x2");
        value.parse().expect("numeric underline x2")
    }

    let (tree, first) = render(91);
    assert_eq!(first, render(191).1);
    assert!(
        underline_x2(&first) < 70.0,
        "soft-wrap spaces must be trimmed before the bbox end: {first}"
    );
    let json: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
    for feature in [
        "text.decorationOp.producerExtent",
        "text.visualSourceBinding",
    ] {
        assert!(json["usedFeatures"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == feature));
    }
    fn decoration_end(value: &serde_json::Value) -> Option<f64> {
        if value.get("type").and_then(serde_json::Value::as_str) == Some("textDecoration") {
            return value["decoration"]["positions"]
                .as_array()
                .and_then(|positions| positions.last())
                .and_then(serde_json::Value::as_f64);
        }
        match value {
            serde_json::Value::Array(values) => values.iter().find_map(decoration_end),
            serde_json::Value::Object(values) => values.values().find_map(decoration_end),
            _ => None,
        }
    }
    assert_eq!(
        underline_x2(&first),
        10.0 + decoration_end(&json).expect("serialized decoration endpoint")
    );

    #[cfg(feature = "native-skia")]
    {
        use rhwp::renderer::layer_renderer::LayerRasterRenderer;
        use rhwp::renderer::skia::SkiaLayerRenderer;

        let trimmed = SkiaLayerRenderer::new().render_png(&tree).unwrap();
        let mut untrimmed = tree.clone();
        fn clear_trim(node: &mut LayerNode) {
            match &mut node.kind {
                LayerNodeKind::Group { children, .. } => {
                    for child in children {
                        clear_trim(child);
                    }
                }
                LayerNodeKind::ClipRect { child, .. } => clear_trim(child),
                LayerNodeKind::Leaf { ops } => {
                    for op in ops {
                        if let PaintOp::TextDecoration {
                            trim_trailing_spaces,
                            ..
                        } = op
                        {
                            *trim_trailing_spaces = 0;
                        }
                    }
                }
            }
        }
        clear_trim(&mut untrimmed.root);
        let untrimmed = SkiaLayerRenderer::new().render_png(&untrimmed).unwrap();
        assert_ne!(
            trimmed, untrimmed,
            "native Skia must consume the published trim"
        );
    }
}

#[test]
fn clip_enabled_is_consumed_from_layer_output_options() {
    let bounds = BoundingBox::new(0.0, 0.0, 40.0, 40.0);
    let clip = BoundingBox::new(0.0, 0.0, 20.0, 40.0);
    let root = LayerNode::clip_rect(
        bounds,
        Some(9),
        clip,
        LayerNode::leaf(bounds, Some(10), vec![solid_rect(bounds, 0x00112233)]),
        rhwp::paint::ClipKind::Generic,
    );
    let tree = PageLayerTree::new(40.0, 40.0, root).with_output_options(LayerOutputOptions {
        clip_enabled: false,
        ..Default::default()
    });
    let mut renderer = SvgLayerRenderer::new();
    renderer.render_page(&tree).unwrap();

    assert!(!renderer.output().contains("clipPath"));
    assert!(renderer.output().contains("#332211"));

    #[cfg(feature = "native-skia")]
    {
        use rhwp::renderer::layer_renderer::LayerRasterRenderer;
        use rhwp::renderer::skia::SkiaLayerRenderer;

        let png = SkiaLayerRenderer::new().render_png(&tree).unwrap();
        let image = image::load_from_memory(&png).unwrap().to_rgba8();
        assert!(
            image.get_pixel(30, 20)[3] > 0,
            "disabled canonical clip must expose the full control"
        );
    }
}

#[test]
fn out_of_body_control_is_replayed_only_inside_the_canonical_clip() {
    let mut render_tree = PageRenderTree::new(0, 80.0, 60.0);
    render_tree.root.node_type = RenderNodeType::Page(PageNode {
        page_index: 0,
        width: 80.0,
        height: 60.0,
        section_index: 0,
    });
    let clip = BoundingBox::new(10.0, 10.0, 40.0, 30.0);
    let mut body = RenderNode::new(
        90,
        RenderNodeType::Body {
            clip_rect: Some(clip),
        },
        clip,
    );
    body.children.push(RenderNode::new(
        91,
        RenderNodeType::Rectangle(RectangleNode::new(
            0.0,
            ShapeStyle {
                fill_color: Some(0x000000FF),
                ..Default::default()
            },
            None,
        )),
        BoundingBox::new(45.0, 15.0, 25.0, 15.0),
    ));
    render_tree.root.children.push(body);
    let layer_tree = LayerBuilder::new(RenderProfile::Screen).build(&render_tree);
    let marked_tree = LayerBuilder::new(RenderProfile::Screen)
        .with_output_options(LayerOutputOptions {
            show_paragraph_marks: true,
            ..Default::default()
        })
        .build(&render_tree);
    assert_eq!(body_clip(&layer_tree.root).unwrap().width, 40.0);
    assert_eq!(body_clip(&marked_tree.root).unwrap().width, 88.0);
    let mut svg = SvgLayerRenderer::new();
    svg.render_page(&layer_tree).unwrap();
    assert!(svg.output().contains("body-clip-90"));
    assert!(svg.output().contains("#ff0000"));
    let mut marked_svg = SvgLayerRenderer::new();
    marked_svg.render_page(&marked_tree).unwrap();
    assert!(marked_svg.output().contains("width=\"88\""));

    #[cfg(feature = "native-skia")]
    {
        use rhwp::renderer::layer_renderer::LayerRasterRenderer;
        use rhwp::renderer::skia::SkiaLayerRenderer;

        let png = SkiaLayerRenderer::new().render_png(&layer_tree).unwrap();
        let image = image::load_from_memory(&png).unwrap().to_rgba8();
        assert!(
            image.get_pixel(47, 20)[3] > 0,
            "in-clip control portion must paint"
        );
        assert_eq!(
            image.get_pixel(60, 20)[3],
            0,
            "declared body clip must suppress out-of-body paint"
        );
        let marked_png = SkiaLayerRenderer::new().render_png(&marked_tree).unwrap();
        let marked_image = image::load_from_memory(&marked_png).unwrap().to_rgba8();
        assert!(
            marked_image.get_pixel(60, 20)[3] > 0,
            "producer-published mark allowance must expose right-edge paint"
        );
    }
}

#[test]
fn table_cell_uses_published_clip_instead_of_node_bounds() {
    let bounds = BoundingBox::new(0.0, 0.0, 40.0, 30.0);
    let clip = BoundingBox::new(0.0, 0.0, 20.0, 30.0);
    let tree = PageLayerTree::new(
        40.0,
        30.0,
        LayerNode::clip_rect(
            bounds,
            Some(92),
            clip,
            LayerNode::leaf(bounds, Some(93), vec![solid_rect(bounds, 0x000000FF)]),
            rhwp::paint::ClipKind::TableCell,
        ),
    );
    let mut svg = SvgLayerRenderer::new();
    svg.render_page(&tree).unwrap();
    assert!(svg.output().contains("width=\"20\""));

    #[cfg(feature = "native-skia")]
    {
        use rhwp::renderer::layer_renderer::LayerRasterRenderer;
        use rhwp::renderer::skia::SkiaLayerRenderer;

        let png = SkiaLayerRenderer::new().render_png(&tree).unwrap();
        let image = image::load_from_memory(&png).unwrap().to_rgba8();
        assert!(image.get_pixel(10, 15)[3] > 0);
        assert_eq!(image.get_pixel(30, 15)[3], 0);
    }
}
