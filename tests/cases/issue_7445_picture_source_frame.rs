//! 원본 그림 프레임·줄 점유·테두리 소유를 같은 출력 경로에서 보호한다.
//! 독립 Hancom Print와 Native/fresh WASM 전쪽 결과는 fixture README에 기록한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{
    model::control::Control,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};
use serde_json::Value;

fn fixture(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/issue7445_picture_source_frame")
        .join(name);
    DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap()
}

fn nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        nodes(child, out);
    }
}

fn picture_on_paragraph(tree: &RenderNode, label: &str) -> RenderNode {
    let mut all = vec![];
    nodes(tree, &mut all);
    let line = all
        .iter()
        .find(|node| {
            matches!(&node.node_type, RenderNodeType::TextLine(_))
                && node.children.iter().any(|child| {
                    matches!(&child.node_type, RenderNodeType::TextRun(run) if run.text.trim() == label)
                })
        })
        .unwrap_or_else(|| panic!("원문 표지의 줄 소유가 있어야 한다: {label}"));
    let images: Vec<_> = line
        .children
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::Image(_)))
        .collect();
    assert_eq!(images.len(), 1, "표지와 그림은 같은 줄에서 한 번 소유한다");
    images[0].clone()
}

#[test]
fn rotated_picture_keeps_its_unrotated_paint_frame_and_both_query_routes() {
    let core = fixture("public-picture-outlines.hwpx");
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).unwrap();
    let image = picture_on_paragraph(&tree.root, "rotated");
    let RenderNodeType::Image(payload) = &image.node_type else {
        unreachable!()
    };
    let paint = payload
        .transform
        .effective_image_bbox(&payload.paint_bbox(&image.bbox));
    // Print의 회전 전 원본 크기는 45pt × 19.8pt다. 회전 외접 상자를 다시 돌리지 않는다.
    assert!((paint.width - 60.0).abs() <= 2.0);
    assert!((paint.height - 26.4).abs() <= 2.0);
    assert!((paint.x + paint.width / 2.0 - image.bbox.x - image.bbox.width / 2.0).abs() < 1e-6);
    assert!((paint.y + paint.height / 2.0 - image.bbox.y - image.bbox.height / 2.0).abs() < 1e-6);

    fn selected(value: &Value, out: &mut Vec<Value>) {
        if value["transform"]["rotation"].as_f64() == Some(30.0)
            && value.get("sourceImageKey").is_some()
        {
            out.push(value.clone());
        }
        match value {
            Value::Array(values) => values.iter().for_each(|v| selected(v, out)),
            Value::Object(values) => values.values().for_each(|v| selected(v, out)),
            _ => {}
        }
    }
    for json in [
        core.get_page_layer_tree_native(0).unwrap(),
        core.get_page_flow_image_ops_native(0).unwrap(),
    ] {
        let mut images = vec![];
        selected(&serde_json::from_str(&json).unwrap(), &mut images);
        assert_eq!(
            images.len(),
            1,
            "전체·흐름 이미지 질의에서 각각 한 번 발행한다"
        );
        let bbox = &images[0]["imagePaintBBox"];
        assert!((bbox["width"].as_f64().unwrap() - 60.0).abs() <= 2.0);
        assert!((bbox["height"].as_f64().unwrap() - 26.4).abs() <= 2.0);
    }
}

#[test]
fn picture_outlines_preserve_the_bitmap_interior_and_source_pen_joins() {
    let core = fixture("public-picture-outlines.hwpx");
    let tree = core.build_page_render_tree(0).unwrap();
    let image = picture_on_paragraph(&tree.root, "wide");
    let mut all = vec![];
    nodes(&tree.root, &mut all);
    let line = all
        .iter()
        .find(|node| node.children.iter().any(|child| child.id == image.id))
        .unwrap();
    let outlines: Vec<_> = line
        .children
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::Rectangle(rect) if rect.picture_border => Some((node, rect)),
            _ => None,
        })
        .collect();
    assert_eq!(outlines.len(), 1);
    let (outline, rect) = outlines[0];
    // 독립 Print의 36pt 외곽선도 그림 안쪽은 약 0.72pt만 덮고 로고를 보존한다.
    assert!(outline.bbox.width - rect.style.stroke_width > image.bbox.width * 0.9);
    assert!(outline.bbox.height - rect.style.stroke_width > image.bbox.height * 0.9);

    fn joins(value: &Value, counts: &mut [usize; 2]) {
        if value["type"].as_str() == Some("rectangle") {
            counts[usize::from(value["strokeJoin"].as_str() != Some("round"))] += 1;
        }
        match value {
            Value::Array(values) => values.iter().for_each(|v| joins(v, counts)),
            Value::Object(values) => values.values().for_each(|v| joins(v, counts)),
            _ => {}
        }
    }
    let mut counts = [0, 0];
    joins(
        &serde_json::from_str(&core.get_page_layer_tree_native(0).unwrap()).unwrap(),
        &mut counts,
    );
    assert_eq!(
        counts,
        [6, 2],
        "ROUND 6개·FLAT/SQUARE 2개, NONE 테두리 없음"
    );
}

#[test]
fn a_nested_cell_picture_publishes_its_source_border_once() {
    let core = fixture("nested-picture-border.hwpx");
    assert_eq!(core.page_count(), 1);
    let tree = core.build_page_render_tree(0).unwrap();
    let mut all = vec![];
    nodes(&tree.root, &mut all);
    let tables: Vec<_> = all
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Table(_)))
        .collect();
    assert!(!tables.is_empty());
    assert!(
        tables.iter().any(|table| {
            let mut children = vec![];
            nodes(table, &mut children);
            children
                .iter()
                .any(|n| matches!(n.node_type, RenderNodeType::Image(_)))
        }),
        "실제 일반 셀 그림 경로를 사용한다"
    );
    let borders = all
        .iter()
        .filter(|n| matches!(&n.node_type, RenderNodeType::Rectangle(r) if r.picture_border))
        .count();
    assert_eq!(
        borders, 1,
        "그림 append와 옛 후행 경로가 같은 외곽선을 중복 발행하지 않는다"
    );
}

fn public_picture() -> (DocumentCore, usize) {
    let source = fixture("public-picture-outlines.hwpx");
    let bytes = source.get_bin_data_image_data_native(1).unwrap();
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "none ").unwrap();
    let inserted: Value = serde_json::from_str(
        &core
            .insert_picture_native(
                0,
                0,
                5,
                &[],
                &bytes,
                4500,
                1980,
                100,
                44,
                "jpg",
                "margin control",
                None,
                None,
            )
            .unwrap(),
    )
    .unwrap();
    (core, inserted["controlIdx"].as_u64().unwrap() as usize)
}

#[test]
fn editing_inline_picture_margins_updates_the_owned_line_and_successor_once() {
    let (mut core, control) = public_picture();
    core.insert_paragraph_native(0, 1).unwrap();
    core.insert_text_native(0, 1, 0, "AFTER").unwrap();
    core.set_picture_properties_native(
        0,
        0,
        control,
        r#"{"treatAsChar":true,"outerMarginTop":2000,"outerMarginBottom":2000}"#,
    )
    .unwrap();
    let paragraphs = &core.document().sections[0].paragraphs;
    // 동일 원본의 독립 한컴 재저장: 그림1980 + 위아래4000, 다음 줄은 간격600 뒤다.
    // 편집으로 합성한 첫 줄 캐시는 HWPX 직렬화에서 생략한다. 편집 결과의
    // 측정값을 직접 검사하고, 재열기는 실제 후속 본문의 원점으로 대조한다.
    assert_eq!(
        (
            paragraphs[0].line_segs[0].line_height,
            paragraphs[0].line_segs[0].baseline_distance
        ),
        (5980, 5083)
    );
    assert_eq!(paragraphs[1].line_segs[0].vertical_pos, 6580);
    assert_eq!(paragraphs[1].text, "AFTER");
    let saved = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    assert_eq!(saved.document().sections[0].paragraphs[1].text, "AFTER");
    let successor_origin = |document: &DocumentCore| {
        let tree = document.build_page_render_tree(0).unwrap();
        let mut all = vec![];
        nodes(&tree.root, &mut all);
        let successors: Vec<_> = all
            .into_iter()
            .filter(|node| {
                matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == "AFTER")
            })
            .collect();
        assert_eq!(successors.len(), 1, "후속 본문은 저장 전후 한 번 소유한다");
        successors[0].bbox.y
    };
    assert!(
        (successor_origin(&core) - successor_origin(&saved)).abs() < 1e-6,
        "재열기 후에도 같은 줄 점유가 후속 본문의 실제 원점을 결정한다"
    );
    core.set_picture_properties_native(
        0,
        0,
        control,
        r#"{"outerMarginTop":0,"outerMarginBottom":0}"#,
    )
    .unwrap();
    assert_eq!(
        core.document().sections[0].paragraphs[1].line_segs[0].vertical_pos,
        2580
    );
    assert_eq!(core.document().sections[0].paragraphs[1].text, "AFTER");
}

#[test]
fn public_rotation_has_the_same_affine_after_hwp_and_hwpx_save() {
    fn affine(core: &DocumentCore) -> [f64; 6] {
        let picture = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|c| match c {
                Control::Picture(p) => Some(p),
                _ => None,
            })
            .unwrap();
        let a = &picture.shape_attr;
        [
            a.render_sx,
            a.render_b,
            a.render_tx,
            a.render_c,
            a.render_sy,
            a.render_ty,
        ]
    }
    for angle in [0, 30] {
        let (mut core, control) = public_picture();
        core.set_picture_properties_native(
            0,
            0,
            control,
            &format!(r#"{{"rotationAngle":{angle}}}"#),
        )
        .unwrap();
        let hwp =
            DocumentCore::from_bytes(&core.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
        let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
        let expected = affine(&hwp);
        if angle != 0 {
            assert!(expected[1].abs() > 0.1 && expected[3].abs() > 0.1);
        }
        for (a, b) in expected.into_iter().zip(affine(&hwpx)) {
            assert!(
                (a - b).abs() < 0.001,
                "같은 공개 모델을 저장할 때 회전 행렬을 identity로 잃지 않는다"
            );
        }
    }
}

#[test]
fn original_picture_border_bit_survives_save_and_precedes_body_print() {
    use rhwp::model::paragraph::Paragraph;
    use rhwp::paint::{LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile};
    fn flags(paragraphs: &[Paragraph], out: &mut Vec<bool>) {
        for paragraph in paragraphs {
            for control in &paragraph.controls {
                match control {
                    Control::Picture(picture) => out.push(picture.border_attr.attr & 63 != 0),
                    Control::Header(header) => flags(&header.paragraphs, out),
                    Control::Footer(footer) => flags(&footer.paragraphs, out),
                    _ => {}
                }
            }
        }
    }
    fn all_flags(core: &DocumentCore) -> Vec<bool> {
        let mut out = vec![];
        for section in &core.document().sections {
            flags(&section.paragraphs, &mut out);
        }
        out
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/issue_265.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 16);
    // HWP3 사양 표44의 picture 정보14바이트 위치 bit0. 꼬리말 로고만 테두리를 소유한다.
    assert_eq!(all_flags(&core), [true, false, false, false, false]);
    for bytes in [
        core.export_hwp_with_adapter_snapshot().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(all_flags(&reopened), [true, false, false, false, false]);
    }

    fn body(node: &LayerNode, found: &mut usize) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    body(child, found);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                body(child, found)
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::TextRun {
                        run,
                        paint_transform,
                        ..
                    } = op
                    {
                        if run.text.contains("빠른") && run.style.hft_print_frame {
                            *found += 1;
                            assert!(
                                paint_transform.is_none(),
                                "독립 Print의 꼬리말 외곽선이 첫 본문 이전의 장치 상태를 발행한다"
                            );
                        }
                    }
                }
            }
        }
    }
    let tree = core.build_page_render_tree(0).unwrap();
    let paint = LayerBuilder::new(RenderProfile::Print).build(&tree);
    let mut found = 0;
    body(&paint.root, &mut found);
    assert!(found > 0, "실제 첫 본문 글줄을 검사한다");
}
