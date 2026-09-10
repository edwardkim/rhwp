//! Source-backed host-line and physical outer-box placement for anchored objects.

use super::{
    hwpunit_to_px, para_has_non_whitespace_text, signed_hwpunit, CommonObjAttr, Control, HorzAlign,
    HorzRelTo, Paragraph, ShapeObject, TablePageBreak, TextWrap, VertAlign, VertRelTo,
};
use crate::model::table::Table;

/// A behind-text image alone retains the legacy zero-advance behavior. When its
/// otherwise empty host also owns a visible fixed title textbox, retain the
/// host's content line, but do not add its trailing line spacing a second time.
pub(super) fn backdrop_title_host_floor(
    para: &Paragraph,
    picture: &CommonObjAttr,
    para_y: f64,
    dpi: f64,
) -> Option<f64> {
    if para_has_non_whitespace_text(para)
        || para.line_segs.len() != 1
        || picture.treat_as_char
        || !matches!(picture.text_wrap, TextWrap::BehindText)
        || !matches!(picture.vert_rel_to, VertRelTo::Para)
        || !matches!(picture.vert_align, VertAlign::Top)
        || signed_hwpunit(picture.vertical_offset) != 0
    {
        return None;
    }
    let has_title = para.controls.iter().any(|control| {
        let Control::Shape(shape) = control else {
            return false;
        };
        let ShapeObject::Rectangle(rectangle) = shape.as_ref() else {
            return false;
        };
        let common = shape.common();
        !common.treat_as_char
            && matches!(common.text_wrap, TextWrap::InFrontOfText)
            && matches!(common.vert_rel_to, VertRelTo::Paper | VertRelTo::Page)
            && matches!(common.horz_rel_to, HorzRelTo::Paper | HorzRelTo::Page)
            && matches!(common.vert_align, VertAlign::Top)
            && rectangle.drawing.border_line.attr & 0x3f == 0
            && rectangle.drawing.text_box.as_ref().is_some_and(|text_box| {
                text_box.paragraphs.iter().any(para_has_non_whitespace_text)
            })
    });
    let line = para.line_segs.first()?;
    (has_title && line.line_height > 0 && para_y.is_finite())
        .then(|| para_y + hwpunit_to_px(line.line_height, dpi))
}

/// The stored host-to-next-paragraph advance proves that the vertical offset
/// addresses the physical outer box, not the painted table border. Equal outer
/// margins permit using the same inset on the horizontal axis as well.
pub(super) fn offset_table_has_stored_outer_box(
    native_hwp5_layout: bool,
    para: &Paragraph,
    table: &Table,
    next_para: Option<&Paragraph>,
) -> bool {
    let common = &table.common;
    if !native_hwp5_layout
        || para_has_non_whitespace_text(para)
        || para.controls.len() != 1
        || !matches!(para.controls.first(), Some(Control::Table(_)))
        || common.treat_as_char
        || !matches!(common.text_wrap, TextWrap::TopAndBottom)
        || !matches!(common.vert_rel_to, VertRelTo::Para)
        || !matches!(common.vert_align, VertAlign::Top)
        || !matches!(common.horz_rel_to, HorzRelTo::Column)
        || !matches!(common.horz_align, HorzAlign::Left)
        || signed_hwpunit(common.vertical_offset) <= 0
        || signed_hwpunit(common.horizontal_offset) < 0
        || signed_hwpunit(common.height) <= 0
        || !matches!(table.page_break, TablePageBreak::None)
        || table.caption.is_some()
        || table.outer_margin_top <= 0
        || table.outer_margin_top != table.outer_margin_bottom
        || table.outer_margin_top != table.outer_margin_left
        || table.outer_margin_top != table.outer_margin_right
    {
        return false;
    }
    let Some(next) = next_para else {
        return false;
    };
    if para_has_non_whitespace_text(next) || !next.controls.is_empty() {
        return false;
    }
    let stored_line = |paragraph: &Paragraph| {
        paragraph
            .line_segs
            .iter()
            .find(|line| {
                line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                    && line.line_height > 0
            })
            .map(|line| line.vertical_pos)
    };
    let (Some(host_y), Some(next_y)) = (stored_line(para), stored_line(next)) else {
        return false;
    };
    let stored_advance = i64::from(next_y) - i64::from(host_y);
    let outer_advance = i64::from(signed_hwpunit(common.vertical_offset))
        + i64::from(signed_hwpunit(common.height))
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    stored_advance > 0 && (stored_advance - outer_advance).abs() <= 1
}
