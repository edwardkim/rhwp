//! Issue #1585: caption paragraphs must render TopAndBottom picture controls.
//!
//! #1551 restored inline TAC picture payload threading for caption paragraphs.
//! This follow-up covers caption pictures whose wrap mode is TopAndBottom,
//! including the HWPX case where a caption-local logo is stored in a nested
//! table caption.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{
    Caption, CaptionDirection, HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo,
};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/hwpx/hy-001.hwpx";
const CAPTION_CELL_SENTINEL: usize = 65534;

fn first_picture_para(paragraphs: &[Paragraph]) -> Option<Paragraph> {
    for para in paragraphs {
        if para
            .controls
            .iter()
            .any(|ctrl| matches!(ctrl, Control::Picture(pic) if pic.image_attr.bin_data_id > 0))
        {
            return Some(para.clone());
        }
        for ctrl in &para.controls {
            if let Control::Table(table) = ctrl {
                for cell in &table.cells {
                    if let Some(found) = first_picture_para(&cell.paragraphs) {
                        return Some(found);
                    }
                }
            }
        }
    }
    None
}

fn make_caption_floating_picture_para(mut para: Paragraph) -> (Paragraph, u16) {
    para.text.clear();
    para.char_offsets.clear();
    para.char_count = 0;
    let mut bin_id = None;
    for ctrl in &mut para.controls {
        if let Control::Picture(pic) = ctrl {
            pic.common.treat_as_char = false;
            pic.common.text_wrap = TextWrap::TopAndBottom;
            pic.common.vert_rel_to = VertRelTo::Para;
            pic.common.horz_rel_to = HorzRelTo::Column;
            pic.common.vert_align = VertAlign::Top;
            pic.common.horz_align = HorzAlign::Left;
            pic.common.vertical_offset = 0;
            pic.common.horizontal_offset = 0;
            bin_id = Some(pic.image_attr.bin_data_id);
            break;
        }
    }
    (
        para,
        bin_id.expect("fixture paragraph must contain a picture"),
    )
}

fn top_caption(caption_para: Paragraph) -> Caption {
    Caption {
        direction: CaptionDirection::Top,
        width: 10_000,
        spacing: 0,
        max_width: 50_000,
        paragraphs: vec![caption_para],
        ..Default::default()
    }
}

fn attach_top_caption_to_first_table(paragraphs: &mut [Paragraph], caption: Caption) -> bool {
    for para in paragraphs {
        for ctrl in &mut para.controls {
            if let Control::Table(table) = ctrl {
                table.caption = Some(caption);
                return true;
            }
        }
    }
    false
}

fn clone_first_table(paragraphs: &[Paragraph]) -> Option<rhwp::model::table::Table> {
    for para in paragraphs {
        for ctrl in &para.controls {
            if let Control::Table(table) = ctrl {
                return Some((**table).clone());
            }
        }
    }
    None
}

fn attach_nested_caption_table_to_first_table(
    paragraphs: &mut [Paragraph],
    mut nested_table: rhwp::model::table::Table,
    caption: Caption,
) -> bool {
    nested_table.caption = Some(caption);
    nested_table.common.treat_as_char = true;
    nested_table.common.text_wrap = TextWrap::TopAndBottom;
    nested_table.common.vert_rel_to = VertRelTo::Para;
    nested_table.common.horz_rel_to = HorzRelTo::Para;
    nested_table.common.vert_align = VertAlign::Top;
    nested_table.common.horz_align = HorzAlign::Left;
    nested_table.common.vertical_offset = 0;
    nested_table.common.horizontal_offset = 0;

    for para in paragraphs {
        for ctrl in &mut para.controls {
            if let Control::Table(table) = ctrl {
                let Some(cell) = table.cells.first_mut() else {
                    return false;
                };
                let Some(cell_para) = cell.paragraphs.first_mut() else {
                    return false;
                };
                cell_para.text.clear();
                cell_para.char_offsets.clear();
                cell_para.char_count = 0;
                cell_para.controls.clear();
                cell_para
                    .controls
                    .push(Control::Table(Box::new(nested_table)));
                return true;
            }
        }
    }
    false
}

fn collect_caption_images(node: &RenderNode, out: &mut Vec<(u16, Option<TextWrap>, bool)>) {
    if let RenderNodeType::Image(img) = &node.node_type {
        let is_caption_image = img.cell_index == Some(CAPTION_CELL_SENTINEL)
            || img.cell_context.as_ref().is_some_and(|ctx| {
                ctx.path
                    .last()
                    .is_some_and(|entry| entry.cell_index == CAPTION_CELL_SENTINEL)
            });
        if is_caption_image {
            out.push((img.bin_data_id, img.text_wrap, img.data.is_some()));
        }
    }
    for child in &node.children {
        collect_caption_images(child, out);
    }
}

fn load_doc() -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn table_caption_topbottom_picture_emits_image_node() {
    let mut doc = load_doc();
    let source_para = first_picture_para(&doc.document().sections[0].paragraphs)
        .expect("fixture must contain a picture paragraph");
    let (caption_para, bin_id) = make_caption_floating_picture_para(source_para);
    assert!(
        doc.document()
            .bin_data_content
            .iter()
            .any(|content| content.id == bin_id),
        "fixture must contain BinData payload for bin_id={bin_id}"
    );

    assert!(
        attach_top_caption_to_first_table(
            &mut doc.document_mut().sections[0].paragraphs,
            top_caption(caption_para),
        ),
        "fixture must contain a top-level table"
    );

    let tree = doc.build_page_render_tree(0).expect("render page 1");
    let mut caption_images = Vec::new();
    collect_caption_images(&tree.root, &mut caption_images);

    assert_eq!(
        caption_images,
        vec![(bin_id, Some(TextWrap::TopAndBottom), true)],
        "caption TopAndBottom picture must be emitted exactly once with payload"
    );
}

#[test]
fn nested_table_caption_topbottom_picture_emits_image_node() {
    let mut doc = load_doc();
    let source_para = first_picture_para(&doc.document().sections[0].paragraphs)
        .expect("fixture must contain a picture paragraph");
    let (caption_para, bin_id) = make_caption_floating_picture_para(source_para);
    let nested_table = clone_first_table(&doc.document().sections[0].paragraphs)
        .expect("fixture must contain a cloneable table");

    assert!(
        attach_nested_caption_table_to_first_table(
            &mut doc.document_mut().sections[0].paragraphs,
            nested_table,
            top_caption(caption_para),
        ),
        "fixture must allow inserting a nested caption table"
    );

    let tree = doc.build_page_render_tree(0).expect("render page 1");
    let mut caption_images = Vec::new();
    collect_caption_images(&tree.root, &mut caption_images);

    assert!(
        caption_images.contains(&(bin_id, Some(TextWrap::TopAndBottom), true)),
        "nested table caption TopAndBottom image must be emitted with payload: {caption_images:?}"
    );
    assert_eq!(
        caption_images
            .iter()
            .filter(|(seen_bin_id, wrap, _)| {
                *seen_bin_id == bin_id && *wrap == Some(TextWrap::TopAndBottom)
            })
            .count(),
        1,
        "nested table caption image must not be duplicated: {caption_images:?}"
    );
}
