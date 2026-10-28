//! 관계형·고정형: 실제 글꼴 공급 → 공통 Print 계획 → JSON/SVG·본문/셀 clip 계약.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! 이 검사는 합성 입력을 사용하며 아래 수치는 관련 편람의 별도 시각 검증이다.
//! 관련 실사례 HWP 9쪽 Native 100.00000% / fresh WASM 100.00000%.
//! 관련 실사례 HWPX 9쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 실사례와 합성 검사의 관계: mydocs/plans/issue_8153_continuous_text_shadow_20261026.md.
//! 저장소가 직접 만든 MIT 합성 TTF를 메모리에서 읽는다. 한컴 시각 일치의 증거와 구분한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::bin_data::{BinData, BinDataContent, BinDataType};
use rhwp::paint::{ClipKind, LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::LayerRenderer;
use rhwp::renderer::render_tree::{
    BoundingBox, PageRenderTree, RenderNode, RenderNodeType, TableCellNode, TextRunNode,
};
use rhwp::renderer::svg_layer::SvgLayerRenderer;
use rhwp::DocumentCore;

const FAMILY: &str = "RHWP Host Fixture";
const GLYPH: &str = "A";

fn fixture_core(percent: [i8; 2], embedded: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, GLYPH).unwrap();
    let bytes = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    let mut doc = core.document().clone();
    for fonts in &mut doc.doc_info.font_faces {
        for font in fonts {
            font.name = FAMILY.into();
            font.is_embedded = embedded;
            font.resolved_bin_data_id = embedded.then_some(1);
        }
    }
    for style in &mut doc.doc_info.char_shapes {
        style.base_size = 900;
        style.shadow_type = 2;
        style.shadow_offset_x = percent[0];
        style.shadow_offset_y = percent[1];
    }
    if embedded {
        doc.doc_info.bin_data_list.push(BinData {
            data_type: BinDataType::Embedding,
            storage_id: 1,
            extension: Some("ttf".into()),
            ..Default::default()
        });
        doc.bin_data_content.push(BinDataContent {
            id: 1,
            extension: "ttf".into(),
            data: bytes.to_vec().into(),
        });
    }
    core.set_document(doc);
    core.set_hangul2024_compat(true);
    if !embedded {
        core.register_metafile_font_native(FAMILY, bytes, 0)
            .unwrap();
    }
    core
}

fn runs<'a>(node: &'a LayerNode, out: &mut Vec<&'a TextRunNode>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                runs(child, out);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            runs(child, out)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::TextRun { run, .. } = op {
                    out.push(run);
                }
            }
        }
    }
}

fn source_run(percent: [i8; 2], embedded: bool) -> TextRunNode {
    let core = fixture_core(percent, embedded);
    let original = core.export_hwp_native().unwrap();
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let mut found = Vec::new();
    runs(&tree.root, &mut found);
    let selected: Vec<_> = found
        .into_iter()
        .filter(|run| run.display_or_text() == GLYPH)
        .collect();
    assert_eq!(selected.len(), 1, "원본 표시 글자 누락·중복 금지");
    let result = selected[0].clone();
    assert!(
        result.style.continuous_shadow.is_some(),
        "실제 supplied/embedded face가 WMF 없는 공통 Print 계획에 도달한다"
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        original,
        "paint 공급은 저장 원본을 바꾸지 않는다"
    );
    result
}

fn samples(value: &serde_json::Value) -> Option<Vec<[f64; 2]>> {
    match value {
        serde_json::Value::Object(map) => map
            .get("shadowOffsets")
            .map(|value| serde_json::from_value(value.clone()).unwrap())
            .or_else(|| map.values().find_map(samples)),
        serde_json::Value::Array(values) => values.iter().find_map(samples),
        _ => None,
    }
}

fn standalone(run: TextRunNode) -> PageRenderTree {
    let mut tree = PageRenderTree::new(0, 100.0, 100.0);
    tree.root.children.push(RenderNode::new(
        1,
        RenderNodeType::TextRun(run),
        BoundingBox::new(10.0, 20.0, 12.0, 12.0),
    ));
    tree
}

#[test]
fn mixed_axis_shadow_samples_reach_json_and_svg_without_reinterpolation() {
    // 합성 face Win800/200·1000UPEM·12px: 600dpi cell75, signed7/3%→(5,2).
    // 독립 Print에서 확인한 signed 절삭 순서이며 생산자 계산값을 기대 배열로 재인용하지 않는다.
    for (percent, expected) in [
        (
            [7, 3],
            [
                [0.0, 0.0],
                [0.16, 0.0],
                [0.32, 0.16],
                [0.48, 0.16],
                [0.64, 0.16],
                [0.8, 0.32],
            ],
        ),
        (
            [-3, 7],
            [
                [0.0, 0.0],
                [0.0, 0.16],
                [-0.16, 0.32],
                [-0.16, 0.48],
                [-0.16, 0.64],
                [-0.32, 0.8],
            ],
        ),
    ] {
        let run = source_run(percent, false);
        let plan = run.style.continuous_shadow.as_ref().unwrap();
        assert_eq!(plan.face_index, 0);
        assert_eq!(plan.offsets().collect::<Vec<_>>(), expected);
        let paint_style = rhwp::paint::PaintTextStyle::from(&run.style);
        assert_eq!(paint_style.continuous_shadow, run.style.continuous_shadow);
        let tree = LayerBuilder::new(RenderProfile::Print).build(&standalone(run));
        let wire: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
        assert_eq!(samples(&wire).unwrap(), expected);
        let mut svg = SvgLayerRenderer::new();
        svg.render_page(&tree).unwrap();
        let xml = roxmltree::Document::parse(svg.output()).unwrap();
        assert_eq!(
            xml.descendants()
                .filter(|node| node.has_tag_name("text") && node.text() == Some(GLYPH))
                .count(),
            7,
            "같은 6개 표본 뒤 원문 한 번"
        );
    }
    let zero = source_run([0, 0], false);
    assert_eq!(
        zero.style
            .continuous_shadow
            .unwrap()
            .offsets()
            .collect::<Vec<_>>(),
        [[0.0, 0.0]]
    );
    let equal = source_run([77, 77], false);
    let plan = equal.style.continuous_shadow.unwrap();
    assert_eq!(plan.offsets().count(), 59);
    assert_eq!(plan.offsets().last(), Some([9.28, 9.28]));
}

#[test]
fn source_shadow_kind_overrides_old_plan_in_wire_and_actual_svg_consumer() {
    let supplied = source_run([7, 3], false);
    for (kind, expected_draws) in [(0, 1), (1, 2), (2, 7)] {
        let mut run = supplied.clone();
        run.style.shadow_type = kind;
        run.style.shadow_offset_x = 3.0;
        run.style.shadow_offset_y = -2.0;
        let tree = LayerBuilder::new(RenderProfile::Print).build(&standalone(run));
        let wire: serde_json::Value = serde_json::from_str(&tree.to_json()).unwrap();
        assert_eq!(
            samples(&wire).is_some(),
            kind == 2,
            "없음·비연속으로 이전 표본을 보내지 않는다"
        );
        let mut svg = SvgLayerRenderer::new();
        svg.render_page(&tree).unwrap();
        let xml = roxmltree::Document::parse(svg.output()).unwrap();
        assert_eq!(
            xml.descendants()
                .filter(|node| node.has_tag_name("text") && node.text() == Some(GLYPH))
                .count(),
            expected_draws
        );
    }
    let embedded = fixture_core([10, 5], true);
    let print: serde_json::Value = serde_json::from_str(
        &embedded
            .build_page_layer_tree_with_profile(0, RenderProfile::Print)
            .unwrap()
            .to_json(),
    )
    .unwrap();
    let offsets = samples(&print).expect("내장 TTF face를 그림자 단독 경로에도 공급한다");
    assert_eq!(offsets.len(), 9);
    assert_eq!(offsets.last(), Some(&[1.28, 0.64]));
    let screen: serde_json::Value = serde_json::from_str(
        &embedded
            .build_page_layer_tree_with_profile(0, RenderProfile::Screen)
            .unwrap()
            .to_json(),
    )
    .unwrap();
    assert!(
        samples(&screen).is_none(),
        "Print 장치 계획을 Screen에 강제하지 않는다"
    );
}

fn body_clip(node: &LayerNode) -> Option<BoundingBox> {
    match &node.kind {
        LayerNodeKind::ClipRect {
            clip,
            clip_kind: ClipKind::Body,
            ..
        } => Some(*clip),
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            body_clip(child)
        }
        LayerNodeKind::Group { children, .. } => children.iter().find_map(body_clip),
        _ => None,
    }
}

#[test]
fn displayed_shadow_ink_expands_body_clip_and_preserves_owned_cell_clip() {
    let supplied = source_run([-7, 3], false);
    let bounds = BoundingBox::new(10.0, 20.0, 12.0, 12.0);
    for (display, kind, cell, expected_x, expected_height) in [
        (GLYPH, 2, false, 9.2, 12.32),
        (" \t", 2, false, 10.0, 12.0),
        (GLYPH, 0, false, 10.0, 12.0),
        (GLYPH, 2, true, 10.0, 12.0),
    ] {
        let mut run = supplied.clone();
        run.display_text = Some(display.into());
        run.style.shadow_type = kind;
        let mut leaf = RenderNode::new(2, RenderNodeType::TextRun(run), bounds);
        if cell {
            let mut owner = RenderNode::new(
                3,
                RenderNodeType::TableCell(TableCellNode {
                    row: 0,
                    col: 0,
                    row_span: 1,
                    col_span: 1,
                    border_fill_id: 0,
                    text_direction: 0,
                    clip: true,
                    page_fragment: false,
                    model_cell_index: Some(0),
                }),
                bounds,
            );
            owner.children.push(leaf);
            leaf = owner;
        }
        let mut body = RenderNode::new(
            1,
            RenderNodeType::Body {
                clip_rect: Some(bounds),
            },
            bounds,
        );
        body.children.push(leaf);
        let mut tree = PageRenderTree::new(0, 100.0, 100.0);
        tree.root.children.push(body);
        let tree = LayerBuilder::new(RenderProfile::Print).build(&tree);
        let actual = body_clip(&tree.root).unwrap();
        assert!((actual.x - expected_x).abs() < 1e-9);
        assert!((actual.height - expected_height).abs() < 1e-9);
        assert_eq!(actual.y, bounds.y, "빈 표시여도 원래 점유 원점은 유지한다");
    }
}
