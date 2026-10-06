//! Measured TopAndBottom textbox bands for body flow, including lower-page boxes.
//!
//! Floating-object overlap permission does not cancel a TopAndBottom flow band.
//! InFrontOfText labels and decorations remain on the normal paint-only path.

use super::{
    hwpunit_to_px, CommonObjAttr, Control, HorzRelTo, LayoutRect, RenderNode, ShapeObject,
    TextWrap, VertAlign, VertRelTo, VisibleFloatExclusion,
};

pub(super) fn is_fixed_flow_textbox(control: &Control) -> bool {
    let Control::Shape(shape) = control else {
        return false;
    };
    let ShapeObject::Rectangle(rectangle) = shape.as_ref() else {
        return false;
    };
    let common = shape.common();
    !common.treat_as_char
        && matches!(common.text_wrap, TextWrap::TopAndBottom)
        && matches!(common.horz_rel_to, HorzRelTo::Paper | HorzRelTo::Page)
        && matches!(common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
        && matches!(common.vert_align, VertAlign::Top)
        && rectangle.drawing.shape_attr.rotation_angle == 0
        && rectangle.drawing.shape_attr.render_b.abs() <= 1e-6
        && rectangle.drawing.shape_attr.render_c.abs() <= 1e-6
        // The renderer supplies a minimum stroke even when stored width is zero.
        && rectangle.drawing.border_line.attr & 0x3f != 0
        && rectangle.drawing.text_box.is_some()
}

pub(super) fn reserve_painted_bounds(
    exclusions: &mut Vec<VisibleFloatExclusion>,
    parent: &RenderNode,
    common: &CommonObjAttr,
    owner_para: usize,
    col_area: &LayoutRect,
    dpi: f64,
) {
    let margin_top = hwpunit_to_px(common.margin.top as i32, dpi).max(0.0);
    let margin_bottom = hwpunit_to_px(common.margin.bottom as i32, dpi).max(0.0);
    let margin_left = hwpunit_to_px(common.margin.left as i32, dpi).max(0.0);
    let margin_right = hwpunit_to_px(common.margin.right as i32, dpi).max(0.0);
    for child in &parent.children {
        let bbox = child.bbox;
        let top = bbox.y - margin_top;
        let bottom = bbox.y + bbox.height + margin_bottom;
        let left = bbox.x - margin_left;
        let right = bbox.x + bbox.width + margin_right;
        if !top.is_finite()
            || !bottom.is_finite()
            || !left.is_finite()
            || !right.is_finite()
            || bbox.height <= 0.0
            || bbox.width <= 0.0
            || right <= col_area.x
            || left >= col_area.x + col_area.width
            || bottom <= col_area.y
            || top >= col_area.y + col_area.height
        {
            continue;
        }
        exclusions.push(VisibleFloatExclusion {
            fixed_textbox: true,
            top,
            bottom,
            owner_para,
            blocks_text: true,
        });
    }
}

/// Keep different paragraph owners separate, even when their bands intersect.
pub(super) fn merge_fixed_bands(exclusions: &mut Vec<VisibleFloatExclusion>) {
    exclusions.sort_by(|a, b| a.top.total_cmp(&b.top));
    let mut merged: Vec<VisibleFloatExclusion> = Vec::with_capacity(exclusions.len());
    for zone in exclusions.drain(..) {
        if let Some(previous) = merged.last_mut() {
            if zone.owner_para == previous.owner_para && zone.top <= previous.bottom {
                previous.bottom = previous.bottom.max(zone.bottom);
                continue;
            }
        }
        merged.push(zone);
    }
    *exclusions = merged;
}

/// A table starting above a band may still cross it. Probe the whole table and
/// repeat after each displacement so stacked bands cannot leave a new overlap.
/// None preserves the existing anchor calculation when no collision occurred.
pub(super) fn table_floor(
    exclusions: &[VisibleFloatExclusion],
    natural_top: f64,
    height: f64,
    gap: f64,
) -> Option<f64> {
    if !natural_top.is_finite() || !height.is_finite() || height <= 0.0 {
        return None;
    }
    let mut floor = natural_top;
    for _ in 0..exclusions.len() {
        let next = exclusions
            .iter()
            .filter(|zone| zone.fixed_textbox)
            .filter(|zone| floor < zone.bottom && floor + height > zone.top + 0.5)
            .map(|zone| zone.bottom + gap.max(0.0))
            .fold(floor, f64::max);
        if next <= floor {
            break;
        }
        floor = next;
    }
    (floor > natural_top).then_some(floor)
}
