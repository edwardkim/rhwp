//! 표의 배치 방식과 문서 주소를 공통 paint layer에 전달한다.

use super::LayoutEngine;
use crate::model::table::Table;
use crate::renderer::render_tree::{RenderNode, RenderNodeType};

impl LayoutEngine {
    pub(super) fn publish_table_paint_layer(table: &Table, node: &mut RenderNode) {
        if table.common.treat_as_char {
            return;
        }
        let RenderNodeType::Table(owner) = &node.node_type else {
            return;
        };
        let layer = Self::render_layer_from_common(
            &table.common,
            owner.para_index.unwrap_or(0),
            owner.control_index.unwrap_or(0),
        );
        node.set_layer(layer);
    }
}
