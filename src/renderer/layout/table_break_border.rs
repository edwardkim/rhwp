//! 자동 분할 경계선은 셀의 원래 테두리와 별도의 표 속성이다.

use super::border_rendering::{border_line_type_from_code, BorderEdge};
use crate::model::style::{BorderLine, BorderLineType};
use crate::model::table::Table;
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 쪽 조각의 내부 컷에만 적용한다. 온전한 원본 시작·종료 변은 포함하지 않는다.
#[derive(Clone, Copy, Default)]
pub(crate) struct AutomaticBreakEdges {
    pub top_row: Option<usize>,
    pub bottom_row: Option<usize>,
}

/// 한컴 TableBorderFill의 경계선은 DiagonalType·Width·Color를 소비한다.
/// NONE도 명시적 덮어쓰기이며 기존의 굵은 셀 테두리로 다시 채우지 않는다.
pub(super) fn separate_break_line(table: &Table, styles: &ResolvedStyleSet) -> Option<BorderLine> {
    let style = table
        .border_fill_id
        .checked_sub(1)
        .and_then(|id| styles.border_styles.get(usize::from(id)))?;
    style.break_cell_separate_line.then_some(BorderLine {
        line_type: border_line_type_from_code(style.diagonal.diagonal_type),
        width: style.diagonal.width,
        color: style.diagonal.color,
    })
}

pub(super) fn replace_grid_edge(edge: &mut [Option<BorderEdge>], line: BorderLine) {
    edge.fill((line.line_type != BorderLineType::None).then(|| line.into()));
}
