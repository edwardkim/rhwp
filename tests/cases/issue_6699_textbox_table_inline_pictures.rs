//! Textbox table pictures must use the same inline flow as their adjacent text.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::style::Alignment;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn document() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/table-in-tbox.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("fixture")).expect("document")
}

fn walk<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        walk(child, nodes);
    }
}

#[test]
fn first_page_inline_pictures_follow_their_text_line() {
    let core = document();
    let page = core.build_page_render_tree(0).expect("page one");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    for bin in [4, 8] {
        let pictures: Vec<_> = nodes.iter().filter(|node| {
            matches!(&node.node_type, RenderNodeType::Image(image) if image.bin_data_id == bin)
        }).collect();
        assert_eq!(pictures.len(), 1, "picture {bin} must be emitted once");
        if bin == 4 {
            // Original Hancom PDF, scaled uniformly to the source page height.
            assert!((pictures[0].bbox.x - 128.70).abs() < 1.0);
        }
        let RenderNodeType::Image(image) = &pictures[0].node_type else {
            unreachable!()
        };
        let context = image.cell_context.as_ref().expect("nested cell context");
        assert_eq!(context.path.len(), 2);
        assert_eq!(image.para_index, Some(context.parent_para_index));
        assert!(image.data.as_ref().is_some_and(|data| !data.is_empty()));
    }
    // The original paragraph has a space after the logo. Do not remove it or
    // move the text to compensate for the image's former cell-left placement.
    let text = nodes.iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.starts_with("충남중부"))
    }).expect("text after logo");
    // Preserve the existing text origin instead of shifting the whole centered
    // table to hide the remaining font-metric difference from the PDF.
    assert!((text.bbox.x - 334.40).abs() < 0.1, "text x={}", text.bbox.x);
    let logo = nodes.iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::Image(image) if image.bin_data_id == 8)
    }).unwrap();
    let gap = text.bbox.x - logo.bbox.x - logo.bbox.width;
    assert!((gap - 15.97).abs() < 0.1, "single source space: gap={gap}");
}

#[test]
fn pictures_outside_tables_keep_their_original_positions() {
    let core = document();
    let page = core.build_page_render_tree(0).expect("page one");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let pictures: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Image(_))
                && (node.bbox.width - 17.0).abs() < 0.1
        })
        .collect();
    assert_eq!(pictures.len(), 3);
    for (picture, (x, y)) in pictures
        .iter()
        .zip([(89.4, 890.5), (86.4, 928.6), (86.4, 963.3)])
    {
        assert!((picture.bbox.x - x).abs() < 0.06);
        assert!((picture.bbox.y - y).abs() < 0.06);
    }
}

#[test]
fn second_page_repeated_pictures_keep_distinct_cell_paths() {
    let core = document();
    let page = core.build_page_render_tree(1).expect("page two");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let pictures: Vec<_> = nodes.iter().filter(|node| {
        matches!(&node.node_type, RenderNodeType::Image(image) if image.bin_data_id == 2)
    }).collect();
    assert_eq!(pictures.len(), 3, "same bin data occurs in three tables");
    let mut paths = std::collections::BTreeSet::new();
    for picture in pictures {
        assert!(
            (picture.bbox.x - 123.10).abs() < 1.0,
            "x={}",
            picture.bbox.x
        );
        let RenderNodeType::Image(image) = &picture.node_type else {
            unreachable!()
        };
        let path: Vec<_> = image
            .cell_context
            .as_ref()
            .expect("cell context")
            .path
            .iter()
            .map(|entry| (entry.control_index, entry.cell_index, entry.cell_para_index))
            .collect();
        assert!(
            paths.insert(path),
            "separate inline pictures must not share a key"
        );
    }
}

#[test]
fn inline_logos_are_painted_after_the_textbox_background() {
    let core = document();
    let svg = core.render_page_svg_native(1).expect("page two SVG");
    let xml = roxmltree::Document::parse(&svg).expect("valid SVG");
    let number = |node: roxmltree::Node<'_, '_>, name| {
        node.attribute(name).and_then(|v| v.parse::<f64>().ok())
    };
    let background = xml
        .descendants()
        .find(|node| {
            node.has_tag_name("rect")
                && node
                    .attribute("fill")
                    .is_some_and(|v| v.starts_with("url("))
                && number(*node, "width").is_some_and(|v| v > 600.0)
                && number(*node, "height").is_some_and(|v| v > 800.0)
        })
        .expect("large colored textbox background");
    let logos: Vec<_> = xml
        .descendants()
        .filter(|node| {
            node.has_tag_name("image")
                && number(*node, "x").is_some_and(|v| (v - 123.10).abs() < 1.0)
        })
        .collect();
    assert_eq!(logos.len(), 3);
    for logo in logos {
        assert!(
            background.range().start < logo.range().start,
            "inline logo must not be covered by its textbox background"
        );
    }
}

#[test]
fn left_aligned_table_prefix_does_not_expand_to_fill_the_line() {
    let mut core = document();
    let mut doc = core.document().clone();
    let Control::Shape(shape) = &doc.sections[0].paragraphs[0].controls[2] else {
        panic!("fixture textbox");
    };
    let prefix = &shape
        .drawing()
        .unwrap()
        .text_box
        .as_ref()
        .unwrap()
        .paragraphs[8];
    assert!(prefix.text.chars().all(|c| c == ' '));
    assert!(prefix
        .controls
        .iter()
        .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char)));
    let style_id = prefix.para_shape_id as usize;
    doc.doc_info.para_shapes[style_id].alignment = Alignment::Left;
    core.set_document(doc);
    let page = core.build_page_render_tree(0).expect("page one");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    let picture = nodes.iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::Image(image) if image.bin_data_id == 4)
    }).expect("notice picture");
    assert!(
        (picture.bbox.x - 128.70).abs() < 1.0,
        "x={}",
        picture.bbox.x
    );
}

#[test]
fn centered_table_borders_keep_their_original_positions() {
    let core = document();
    let page = core.build_page_render_tree(0).expect("page one");
    let mut nodes = Vec::new();
    walk(&page.root, &mut nodes);
    for (width, pdf_x) in [(430.9, 174.49), (590.2, 94.29)] {
        let table = nodes
            .iter()
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::Table(_))
                    && (node.bbox.width - width).abs() < 0.1
            })
            .expect("centered table");
        assert!((table.bbox.x - pdf_x).abs() < 0.5, "x={}", table.bbox.x);
    }
}
