use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::shape::{TextWrap, VertRelTo};
use crate::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

pub(super) fn hit(
    document: &Document,
    page: &RenderNode,
    page_index: u32,
    x: f64,
    y: f64,
) -> Option<String> {
    let point = BoundingBox::new(x, y, 0.0, 0.0);
    // Floating body equations are promoted to the page's paper layer.
    // The layout already sorts its children from back to front.
    let (node, section, paragraph, offset) = page
        .children
        .iter()
        .rev()
        .filter(|node| node.visible)
        .find_map(|node| {
            flow_anchor(document, node, &point)
                .map(|(section, paragraph, offset)| (node, section, paragraph, offset))
        })?;
    let frame = node.bbox;
    Some(
        serde_json::json!({
            "sectionIndex": section,
            "paragraphIndex": paragraph,
            "charOffset": offset,
            "cursorRect": {
                "pageIndex": page_index,
                "x": frame.x,
                "y": frame.y,
                "height": frame.height,
            },
        })
        .to_string(),
    )
}

fn flow_anchor(
    document: &Document,
    node: &RenderNode,
    point: &BoundingBox,
) -> Option<(usize, usize, usize)> {
    let RenderNodeType::Equation(rendered) = &node.node_type else {
        return None;
    };
    if rendered.note_ref.is_some()
        || rendered.cell_index.is_some()
        || node.header_footer_source.is_some()
        || node.bbox.width <= 0.0
        || node.bbox.height <= 0.0
        || !node.bbox.contains(point)
    {
        return None;
    }
    let (section, paragraph, index) = (
        rendered.section_index?,
        rendered.para_index?,
        rendered.control_index?,
    );
    let owner = document.sections.get(section)?.paragraphs.get(paragraph)?;
    let Control::Equation(source) = owner.controls.get(index)? else {
        return None;
    };
    let frame = &source.common;
    // A paragraph-anchored equation in the text flow occupies a block on
    // screen. Pointing inside it resolves to its anchor, without adding an
    // inline character to the document's cursor offsets.
    if frame.treat_as_char
        || !frame.flow_with_text
        || frame.vert_rel_to != VertRelTo::Para
        || frame.text_wrap != TextWrap::TopAndBottom
    {
        return None;
    }
    let offset = *owner.logical_control_positions().get(index)?;
    Some((section, paragraph, offset))
}
