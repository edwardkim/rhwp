//! 인라인 표의 텍스트 줄과 개체 전용 줄이 같은 자식 셀 주소를 사용한다.

use crate::renderer::layout::{CellContext, CellPathEntry};

pub(super) fn child_context(
    parent: Option<&CellContext>,
    control_index: usize,
) -> (usize, Option<CellContext>) {
    let context = parent.map(|parent| {
        let mut context = parent.clone();
        context.path.push(CellPathEntry {
            control_index,
            cell_index: 0,
            cell_para_index: 0,
            text_direction: 0,
        });
        context
    });
    (usize::from(parent.is_some()), context)
}
