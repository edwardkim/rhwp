//! #5551: caption ownership is source metadata, never a title/position heuristic.
//! In-memory model variations exercise the production layout and public JSON.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::image::Picture;
use rhwp::model::paragraph::{CharShapeRef, LineSeg, Paragraph};
use rhwp::model::shape::{Caption, CaptionDirection, RectangleShape, ShapeObject};
use rhwp::model::table::{Cell, Table};
use rhwp::renderer::render_tree::{
    BoundingBox, CaptionControlKind, CaptionOwner, RenderNode, RenderNodeType, TextLineNode,
};
use serde_json::{json, Value};

fn text(text: &str) -> Paragraph {
    Paragraph {
        text: text.into(),
        char_count: text.encode_utf16().count() as u32 + 1,
        char_offsets: (0..text.encode_utf16().count() as u32).collect(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }],
        line_segs: vec![LineSeg {
            line_height: 1200,
            text_height: 1000,
            baseline_distance: 850,
            segment_width: 15000,
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn caption(direction: CaptionDirection) -> Caption {
    let mut wrapped = text("SAME SECOND");
    wrapped.line_segs.push(LineSeg {
        text_start: 5,
        vertical_pos: 1200,
        ..wrapped.line_segs[0].clone()
    });
    Caption {
        direction,
        width: 15000,
        paragraphs: vec![wrapped, text("TAIL")],
        ..Default::default()
    }
}

fn fixture(direction: CaptionDirection) -> DocumentCore {
    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/hwpx/20260909-para-table.hwpx"
    ))
    .unwrap();
    let mut doc = core.document().clone();
    doc.sections.truncate(1);
    doc.sections[0].paragraphs.truncate(1);
    doc.sections[0]
        .paragraphs
        .push(text("BODY SAME SECOND TAIL"));
    for kind in ["table", "table", "shape", "image"] {
        let cap = Some(caption(direction));
        let common = rhwp::model::shape::CommonObjAttr {
            width: 15000,
            height: 3000,
            treat_as_char: false,
            vert_rel_to: rhwp::model::shape::VertRelTo::Para,
            horz_rel_to: rhwp::model::shape::HorzRelTo::Para,
            text_wrap: rhwp::model::shape::TextWrap::TopAndBottom,
            ..Default::default()
        };
        let control = match kind {
            "table" => Control::Table(Box::new(Table {
                common,
                row_count: 1,
                col_count: 1,
                row_sizes: vec![1],
                cells: vec![Cell {
                    row_span: 1,
                    col_span: 1,
                    width: 15000,
                    height: 3000,
                    paragraphs: vec![text("CELL")],
                    ..Default::default()
                }],
                caption: cap,
                ..Default::default()
            })),
            "shape" => {
                let mut rect = RectangleShape {
                    common,
                    ..Default::default()
                };
                rect.drawing.caption = cap;
                Control::Shape(Box::new(ShapeObject::Rectangle(rect)))
            }
            _ => Control::Picture(Box::new(Picture {
                common,
                caption: cap,
                ..Default::default()
            })),
        };
        let mut host = text("");
        host.controls.push(control);
        doc.sections[0].paragraphs.push(host);
    }
    core.set_document(doc);
    core.repaginate_if_needed();
    core
}

fn walk<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    out.push(node);
    if let Some(children) = node["children"].as_array() {
        for child in children {
            walk(child, out);
        }
    }
}

#[test]
fn caption_owner_matches_control_addresses_without_matching_titles() {
    for direction in [
        CaptionDirection::Top,
        CaptionDirection::Bottom,
        CaptionDirection::Left,
        CaptionDirection::Right,
    ] {
        let core = fixture(direction);
        let mut seen = std::collections::BTreeSet::new();
        let mut body_lines = 0;
        for page in 0..core.page_count() {
            let Ok(tree) = core.build_page_render_tree(page) else {
                break;
            };
            let root: Value = serde_json::from_str(&tree.root.to_json()).unwrap();
            let controls: Value =
                serde_json::from_str(&core.get_page_control_layout_native(page).unwrap()).unwrap();
            let mut nodes = Vec::new();
            walk(&root, &mut nodes);
            for line in nodes.into_iter().filter(|n| n["type"] == "TextLine") {
                let Some(owner) = line.get("captionOwner") else {
                    body_lines += 1;
                    continue;
                };
                let pi = owner["paraIdx"].as_u64().unwrap();
                let ordinal = owner["captionOrdinal"].as_u64().unwrap();
                let expected_kind = match pi {
                    2 | 3 => "table",
                    4 => "shape",
                    5 => "image",
                    _ => panic!("unexpected owner {owner}"),
                };
                assert_eq!(owner["secIdx"], 0);
                assert_eq!(owner["controlIdx"], 0);
                assert_eq!(owner["controlKind"], expected_kind);
                assert!(ordinal < 2);
                // Image without BinData is deliberately a placeholder. Table and
                // shape controls must still match the existing layout address API.
                if expected_kind == "table" {
                    assert!(
                        controls["controls"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|c| c["secIdx"] == owner["secIdx"]
                                && c["paraIdx"] == owner["paraIdx"]
                                && c["controlIdx"] == owner["controlIdx"]
                                && c["type"] == "table"),
                        "missing control for {owner}: {controls}"
                    );
                }
                seen.insert((pi, ordinal));
            }
        }
        assert!(body_lines > 0);
        for pi in 2..=5 {
            for ordinal in 0..=1 {
                assert!(
                    seen.contains(&(pi, ordinal)),
                    "missing {pi}/{ordinal}, got {seen:?}"
                );
            }
        }
    }
}

#[test]
fn optional_owner_is_absent_for_body_and_incomplete_provenance() {
    for address in [
        (None, Some(4), Some(2)),
        (Some(1), None, Some(2)),
        (Some(1), Some(4), None),
    ] {
        assert!(
            CaptionOwner::new(address.0, address.1, address.2, CaptionControlKind::Table).is_none()
        );
    }
    let line = TextLineNode::with_para(12.0, 9.0, 1, 4);
    assert!(serde_json::to_value(&line)
        .unwrap()
        .get("captionOwner")
        .is_none());
    let node = RenderNode::new(
        1,
        RenderNodeType::TextLine(line),
        BoundingBox::new(1.0, 2.0, 30.0, 12.0),
    );
    let value: Value = serde_json::from_str(&node.to_json()).unwrap();
    assert!(value.get("captionOwner").is_none());
    assert_eq!(value["pi"], 4);
}

#[test]
fn owner_json_is_additive_and_keeps_caption_paragraph_index_separate() {
    let line = TextLineNode::with_para(12.0, 9.0, 0, 0);
    let mut node = RenderNode::new(
        1,
        RenderNodeType::TextLine(line),
        BoundingBox::new(1.0, 2.0, 30.0, 12.0),
    );
    let before: Value = serde_json::from_str(&node.to_json()).unwrap();
    if let RenderNodeType::TextLine(line) = &mut node.node_type {
        line.caption_owner =
            CaptionOwner::new(Some(2), Some(42), Some(3), CaptionControlKind::Shape);
    }
    let mut after: Value = serde_json::from_str(&node.to_json()).unwrap();
    let owner = after
        .as_object_mut()
        .unwrap()
        .remove("captionOwner")
        .unwrap();
    assert_eq!(
        owner,
        json!({"secIdx":2,"paraIdx":42,"controlIdx":3,"controlKind":"shape","captionOrdinal":0})
    );
    assert_eq!(
        after, before,
        "all legacy fields and structure must be unchanged"
    );
}

#[test]
fn multiple_sections_keep_real_owner_sections_without_filling_empty_sections() {
    let mut core = fixture(CaptionDirection::Bottom);
    let mut doc = core.document().clone();
    doc.sections.push(doc.sections[0].clone());
    let mut empty_section = doc.sections[0].clone();
    empty_section.paragraphs = vec![text("NO CONTROLS IN THIS SECTION")];
    doc.sections.push(empty_section);
    core.set_document(doc);
    core.repaginate_if_needed();
    assert!(core.page_count() >= 3);
    let mut sections = std::collections::BTreeSet::new();
    let mut caption_sections = std::collections::BTreeSet::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let root: Value = serde_json::from_str(&tree.root.to_json()).unwrap();
        let controls: Value =
            serde_json::from_str(&core.get_page_control_layout_native(page).unwrap()).unwrap();
        for control in controls["controls"].as_array().unwrap() {
            let si = control["secIdx"]
                .as_u64()
                .expect("known section must be present");
            assert!(si < 2, "empty section must not gain fabricated controls");
            sections.insert(si);
        }
        let mut nodes = Vec::new();
        walk(&root, &mut nodes);
        for node in nodes {
            if let Some(owner) = node.get("captionOwner") {
                let si = owner["secIdx"].as_u64().unwrap();
                assert!(
                    controls["controls"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|c| c["secIdx"] == owner["secIdx"]
                            && c["paraIdx"] == owner["paraIdx"]
                            && c["controlIdx"] == owner["controlIdx"]),
                    "owner {owner} has no control on page {page}"
                );
                caption_sections.insert(si);
            }
        }
    }
    assert_eq!(sections, [0, 1].into_iter().collect());
    assert_eq!(caption_sections, sections);
    assert_eq!(
        core.document().sections.len(),
        3,
        "sectionCount is not a count of control owners"
    );
}

#[test]
fn wrapped_lines_share_ordinal_and_metadata_does_not_change_svg() {
    fn clear_owners(node: &mut RenderNode) {
        if let RenderNodeType::TextLine(line) = &mut node.node_type {
            line.caption_owner = None;
        }
        for child in &mut node.children {
            clear_owners(child);
        }
    }
    let core = fixture(CaptionDirection::Bottom);
    let mut counts = std::collections::BTreeMap::new();
    for page in 0..core.page_count() {
        let mut tree = core.build_page_render_tree(page).unwrap();
        let root: Value = serde_json::from_str(&tree.root.to_json()).unwrap();
        let mut nodes = Vec::new();
        walk(&root, &mut nodes);
        for node in nodes {
            if let Some(owner) = node.get("captionOwner") {
                *counts
                    .entry((
                        owner["paraIdx"].as_u64().unwrap(),
                        owner["captionOrdinal"].as_u64().unwrap(),
                    ))
                    .or_insert(0) += 1;
            }
        }
        let mut with = rhwp::renderer::svg::SvgRenderer::new();
        with.render_tree(&tree);
        clear_owners(&mut tree.root);
        let mut without = rhwp::renderer::svg::SvgRenderer::new();
        without.render_tree(&tree);
        assert_eq!(
            with.output(),
            without.output(),
            "metadata must not affect rendering"
        );
    }
    for pi in 2..=5 {
        assert!(
            counts.get(&(pi, 0)).copied().unwrap_or(0) >= 2,
            "wrapped caption lines missing: {counts:?}"
        );
        assert_eq!(counts.get(&(pi, 1)), Some(&1));
    }
}
