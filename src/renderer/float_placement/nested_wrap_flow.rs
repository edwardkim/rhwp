//! 어울림 표의 시각적 상자와 본문 전진을 구분하는 공통 질의.

use crate::model::{paragraph::Paragraph, shape::TextWrap, table::Table};

/// 저장 앵커 줄 옆으로 흐르는 표는 줄만 전진시킨다. 표의 전체 시각 하단은
/// 별도 개체 상자로 합친다. 저장 줄이 없으면 기존 표 높이 폴백을 사용한다.
pub(crate) fn stored_nested_wrap_anchor_height_hu(
    table: &Table,
    paragraph: &Paragraph,
) -> Option<i32> {
    if table.common.treat_as_char
        || !table.common.flow_with_text
        || !matches!(
            table.common.text_wrap,
            TextWrap::Square | TextWrap::Tight | TextWrap::Through
        )
    {
        return None;
    }
    paragraph
        .line_segs
        .first()
        .map(|line| line.line_height)
        .filter(|height| *height > 0)
}
