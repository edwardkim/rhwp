//! 글앞·글뒤 표의 단/문단 앵커는 괘선이 아닌 바깥여백 상자를 가리킨다.

use crate::model::shape::{HorzAlign, HorzRelTo, TextWrap};
use crate::model::table::Table;

pub(crate) fn overlay_table_outer_margin_left_hu(table: &Table) -> Option<i32> {
    (!table.common.treat_as_char
        && matches!(
            table.common.text_wrap,
            TextWrap::BehindText | TextWrap::InFrontOfText
        )
        && matches!(
            table.common.horz_rel_to,
            HorzRelTo::Para | HorzRelTo::Column
        )
        && matches!(table.common.horz_align, HorzAlign::Left | HorzAlign::Inside)
        && table.outer_margin_left > 0)
        .then_some(i32::from(table.outer_margin_left))
}
