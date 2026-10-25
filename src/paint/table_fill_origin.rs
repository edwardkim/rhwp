//! 표·셀 배경의 원본 참조 축과 실제 소유 단의 좌표계.

use crate::renderer::render_tree::{RenderNode, RenderNodeType};

#[derive(Clone, Copy, Default)]
pub(super) struct TableFillOrigin {
    column: Option<(f64, f64)>,
    background: [Option<f64>; 2],
}

impl TableFillOrigin {
    pub(super) fn for_children(mut self, node: &RenderNode) -> Self {
        match &node.node_type {
            RenderNodeType::Column(_) => {
                self.column = Some((node.bbox.x, node.bbox.y));
                self.background = [None; 2];
            }
            RenderNodeType::Table(table) => {
                // 셀 내부의 중첩 표는 바깥 본문 단을 자신의 원점으로 쓰지 않는다.
                self.background = match (
                    self.column,
                    node.print_table_column_axes,
                    table.cell_context.as_ref(),
                ) {
                    (Some((x, y)), Some([horizontal, vertical]), None) => {
                        [horizontal.then_some(x), vertical.then_some(y)]
                    }
                    _ => [None; 2],
                };
                self.column = None;
            }
            // 직접 셀 배경만 표의 참조 축을 상속한다. 아래 글줄·개체는 별도다.
            RenderNodeType::TableCell(_) => self.column = None,
            RenderNodeType::MasterPage
            | RenderNodeType::Header
            | RenderNodeType::Footer
            | RenderNodeType::FootnoteArea
            | RenderNodeType::TextBox => return Self::default(),
            RenderNodeType::Group(group) if group.paint_transform.is_some() => {
                return Self::default();
            }
            _ => self.background = [None; 2],
        }
        self
    }

    pub(super) fn axis(self, axis: usize) -> Option<f64> {
        self.background[axis].filter(|value| value.is_finite())
    }
}
