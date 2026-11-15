//! 검사 유형: 관계형. 독립 Print의 장치 em과 논리 글자 크기·전진을 구분한다.
//! 주석 갱신: 2026-11-16 01:53:01 KST.
//! 실문서 좌표의 px 고정 검사가 아닌 합성 글꼴의 단위·소비 계약이다.
//! 독립 한컴 2020 Print 표본의 8/10/11/13pt는 각각 67/83/92/108 장치 em이다.
//! 설치 글꼴 없이 MIT 합성 TTF를 공급하며 Screen·2024·윤곽 보존 선택을 구분한다.

use rhwp::paint::{LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::native_pua_fonts::NativePuaFonts;
use rhwp::renderer::render_tree::{
    BoundingBox, PageRenderTree, RenderNode, RenderNodeType, TextRunNode,
};
use rhwp::renderer::TextStyle;

fn prepared(
    points: f64,
    profile: RenderProfile,
    preserve: bool,
    supplied: bool,
    overlap: bool,
    decorated: bool,
) -> (TextStyle, Vec<f64>) {
    let mut fonts = NativePuaFonts::default();
    fonts
        .register_otf(
            "test",
            false,
            false,
            include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf"),
            0,
        )
        .unwrap();
    let style = TextStyle {
        font_family: "test".into(),
        font_size: points * 96.0 / 72.0,
        resolved_font_size: true,
        source_font_is_ttf: true,
        preserve_glyph_ink: preserve,
        underline: if decorated {
            rhwp::model::style::UnderlineType::Bottom
        } else {
            rhwp::model::style::UnderlineType::None
        },
        native_otf_metrics: supplied.then(|| fonts.get_otf("test", false, false).unwrap()),
        ..Default::default()
    };
    let run = TextRunNode {
        text: "AA".into(),
        style,
        baseline: 25.0,
        layout_positions: Some(vec![0.0, 11.25, 22.5]),
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
        char_overlap: overlap.then_some(rhwp::renderer::composer::CharOverlapInfo {
            border_type: 1,
            inner_char_size: 0,
            composition: None,
            empty_ink: false,
        }),
        border_fill_id: 0,
        field_marker: Default::default(),
        display_text: None,
    };
    let mut tree = PageRenderTree::new(0, 800.0, 1100.0);
    tree.root.children.push(RenderNode::new(
        1,
        RenderNodeType::TextRun(run),
        BoundingBox::new(25.0, 40.0, 22.5, 30.0),
    ));
    let layer = LayerBuilder::new(profile).build(&tree);
    fn find(node: &LayerNode) -> Option<&TextRunNode> {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children.iter().find_map(find),
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                find(child)
            }
            LayerNodeKind::Leaf { ops } => ops.iter().find_map(|op| match op {
                PaintOp::TextRun { run, .. } => Some(run.as_ref()),
                _ => None,
            }),
        }
    }
    rhwp::paint::validate_text_variant_scope(&layer).expect("공통 표시 변형 계약");
    let result = find(&layer.root).unwrap();
    let json: serde_json::Value = serde_json::from_str(&layer.to_json()).unwrap();
    fn painted(value: &serde_json::Value) -> Option<f64> {
        if value.get("type").and_then(|v| v.as_str()) == Some("textRun") {
            return value.get("paintStyle")?.get("fontSize")?.as_f64();
        }
        match value {
            serde_json::Value::Object(o) => o.values().find_map(painted),
            serde_json::Value::Array(a) => a.iter().find_map(painted),
            _ => None,
        }
    }
    let mut projected = result.style.clone();
    // 직렬화 소비자도 Native와 같은 표시 크기를 받는다.
    projected.font_size = painted(&json).unwrap();
    assert_eq!(
        result.style.font_size,
        points * 96.0 / 72.0,
        "논리 크기는 유지"
    );
    assert_eq!(result.baseline, 25.0, "원문 기준선은 유지");
    (projected, result.layout_positions.clone().unwrap())
}

#[test]
fn legacy_print_uses_independent_device_em_without_remeasuring_source_advances() {
    for (points, device_em) in [(8.0, 67.0), (10.0, 83.0), (11.0, 92.0), (13.0, 108.0)] {
        let (paint, positions) = prepared(points, RenderProfile::Print, false, true, false, false);
        assert!(
            (paint.font_size - device_em * 96.0 / 600.0).abs() < 0.000_001,
            "{points}pt의 독립 Print em: {}",
            paint.font_size
        );
        assert_eq!(positions, [0.0, 11.25, 22.5]);
    }
}

#[test]
fn screen_quality_and_explicit_ink_preservation_keep_their_size() {
    for (profile, preserve, supplied) in [
        (RenderProfile::Screen, false, true),
        (RenderProfile::HighQuality, false, true),
        (RenderProfile::Print, true, true),
        (RenderProfile::Print, false, false),
    ] {
        let (paint, _) = prepared(10.0, profile, preserve, supplied, false, false);
        assert!((paint.font_size - 13.333).abs() < 0.000_001);
    }
}

#[test]
fn actual_document_print_generation_selects_legacy_em_only() {
    for (modern, position) in [(false, 0), (true, 0), (false, 25)] {
        let mut core = rhwp::DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        core.insert_text_native(0, 0, 0, "AA").unwrap();
        let mut doc = core.document().clone();
        for font in doc.doc_info.font_faces.iter_mut().flatten() {
            font.name = "RHWP Host Fixture".into();
            font.alt_type = 1;
        }
        for style in &mut doc.doc_info.char_shapes {
            style.base_size = 1000;
            style.ratios = [100; 7];
            style.spacings = [0; 7];
            style.relative_sizes = [100; 7];
            style.char_offsets = [position; 7];
        }
        core.set_document(doc);
        core.set_hangul2024_compat(modern);
        assert!(core
            .prepare_native_otf_fonts_with(|_| Some((
                include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf").to_vec(),
                0
            )))
            .unwrap());
        let tree = core
            .build_page_layer_tree_with_profile(0, RenderProfile::Print)
            .unwrap();
        fn collect(node: &LayerNode, sizes: &mut Vec<Option<f64>>) {
            match &node.kind {
                LayerNodeKind::Group { children, .. } => {
                    for child in children {
                        collect(child, sizes);
                    }
                }
                LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                    collect(child, sizes)
                }
                LayerNodeKind::Leaf { ops } => {
                    for op in ops {
                        if let PaintOp::TextRun { run, .. } = op {
                            if run.text.contains('A') {
                                assert!(
                                    run.style.native_otf_metrics.is_some(),
                                    "실제 TTF 공급 소비"
                                );
                                sizes.push(run.style.print_ink_font_size);
                            }
                        }
                    }
                }
            }
        }
        let mut sizes = Vec::new();
        collect(&tree.root, &mut sizes);
        assert!(!sizes.is_empty());
        assert!(
            sizes.iter().all(|size| if modern || position != 0 {
                size.is_none()
            } else {
                size.is_some_and(|value| (value - 13.28).abs() < 1e-9)
            }),
            "modern={modern}, position={position}: {sizes:?}"
        );
    }
}

#[test]
fn char_overlap_retains_its_separate_canonical_paint_contract() {
    let (paint, _) = prepared(10.0, RenderProfile::Print, false, true, true, false);
    assert!((paint.font_size - 13.333).abs() < 0.000_001);
}

#[test]
fn underline_and_text_share_the_same_canonical_run_after_ink_em_projection() {
    let (paint, positions) = prepared(10.0, RenderProfile::Print, false, true, false, true);
    assert!((paint.font_size - 13.28).abs() < 1e-9);
    assert_eq!(positions, [0.0, 11.25, 22.5]);
}
