//! 본문 흐름을 남긴 채 다음 단으로 이월되는 OLE의 확정 상자.

use crate::model::shape::{HorzRelTo, ShapeObject, TextWrap, VertAlign, VertRelTo};

use super::{hwpunit_to_px, signed_hwpunit, ParagraphFloatFlow, ParagraphFloatPlacement};

pub(crate) fn column_ole_placement(
    shape: &ShapeObject,
    anchor_y: f64,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    let ShapeObject::Ole(ole) = shape else {
        return None;
    };
    let common = shape.common();
    if common.treat_as_char
        || !common.flow_with_text
        || common.text_wrap != TextWrap::TopAndBottom
        || common.vert_rel_to != VertRelTo::Para
        || common.vert_align != VertAlign::Top
        || common.horz_rel_to != HorzRelTo::Column
        || ole.caption.is_some()
        || ole.drawing.caption.is_some()
        || common.height == 0
    {
        return None;
    }
    let table_top = anchor_y
        + hwpunit_to_px(signed_hwpunit(common.vertical_offset), dpi)
        + hwpunit_to_px(i32::from(common.margin.top), dpi);
    let occupied_bottom = table_top
        + hwpunit_to_px(common.height as i32, dpi)
        + hwpunit_to_px(i32::from(common.margin.bottom), dpi);
    (anchor_y.is_finite() && table_top >= anchor_y && occupied_bottom > table_top).then_some(
        ParagraphFloatPlacement {
            flow: ParagraphFloatFlow::StoredPicture {
                next_flow_y: occupied_bottom,
            },
            anchor_y,
            stored_host_origin: None,
            stored_successor_line_origin: None,
            table_left: None,
            table_top,
            occupied_bottom,
        },
    )
}
