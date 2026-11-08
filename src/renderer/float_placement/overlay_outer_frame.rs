//! 글앞·글뒤 표의 왼쪽 앵커는 괘선이 아닌 바깥여백 상자를 가리킨다.

use crate::model::shape::{CommonObjAttr, HorzAlign, TextWrap};
use crate::model::table::Table;

pub(super) fn overlay_outer_margin_left_hu(common: &CommonObjAttr) -> Option<i32> {
    (!common.treat_as_char
        && matches!(
            common.text_wrap,
            TextWrap::BehindText | TextWrap::InFrontOfText
        )
        && matches!(common.horz_align, HorzAlign::Left | HorzAlign::Inside))
    .then_some(i32::from(common.margin.left))
}

pub(crate) fn overlay_table_outer_margin_left_hu(table: &Table) -> Option<i32> {
    overlay_outer_margin_left_hu(&table.common).map(|_| i32::from(table.outer_margin_left))
}
