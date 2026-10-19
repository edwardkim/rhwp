//! 펼친 자식 표에도 부모 셀의 물리 표시 영역을 보존한다.

use super::{BoundingBox, PageLayoutContext, RenderNode, RenderNodeType, TableCellNode};
use crate::model::{
    style::Alignment,
    table::{Cell, Table},
};
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 빈 글자만 있어도 셀 채움과 자식 문단의 가로 프레임은 독립적인 소유자다.
/// 펼침에서 원점을 다시 추측하지 않고 일반 셀·문단 배치가 이 정보를 소비한다.
pub(super) fn owns_paint_or_horizontal_frame(table: &Table, styles: &ResolvedStyleSet) -> bool {
    let Some(cell) = table.cells.first() else {
        return false;
    };
    let has_fill = |id: u16| {
        id.checked_sub(1)
            .and_then(|index| styles.border_styles.get(usize::from(index)))
            .is_some_and(|style| {
                style.fill_color.is_some()
                    || style.pattern.is_some()
                    || style.gradient.is_some()
                    || style.image_fill.is_some()
            })
    };
    if has_fill(table.border_fill_id)
        || has_fill(cell.border_fill_id)
        || table
            .zones
            .iter()
            .any(|zone| has_fill(zone.border_fill_id))
    {
        return true;
    }
    cell.paragraphs.iter().any(|paragraph| {
        crate::renderer::height_measurer::sole_nested_table_in_paragraph(paragraph)
            .is_some_and(|child| child.common.treat_as_char)
            && styles
                .para_styles
                .get(paragraph.para_shape_id as usize)
                .is_some_and(|style| {
                    matches!(
                        style.alignment,
                        Alignment::Center | Alignment::Right | Alignment::Distribute
                    ) || style.margin_left != 0.0
                        || style.margin_right != 0.0
                        || style.indent != 0.0
                })
    })
}

/// 부모 테두리와 흐름은 호출자가 소유한다. 이 셀은 이번에 배치한 자식들만 감싼다.
/// 자식의 원본 셀 주소는 이미 명시적으로 전달됐으므로 새 개체 주소를 만들지 않는다.
pub(super) fn retain(
    tree: &mut PageLayoutContext,
    container: &mut RenderNode,
    cell: &Cell,
    first_child: usize,
    bounds: BoundingBox,
) {
    let mut viewport = RenderNode::new(
        tree.next_id(),
        RenderNodeType::TableCell(TableCellNode {
            col: cell.col,
            row: cell.row,
            col_span: cell.col_span,
            row_span: cell.row_span,
            border_fill_id: 0,
            text_direction: cell.text_direction,
            clip: true,
            page_fragment: false,
            model_cell_index: None,
        }),
        bounds,
    );
    viewport.children = container.children.drain(first_child..).collect();
    container.children.push(viewport);
}
