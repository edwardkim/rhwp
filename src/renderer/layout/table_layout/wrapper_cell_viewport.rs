//! 펼친 자식 표에도 부모 셀의 물리 표시 영역을 보존한다.

use super::{BoundingBox, PageLayoutContext, RenderNode, RenderNodeType, TableCellNode};
use crate::model::table::Cell;

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
