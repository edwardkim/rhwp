// 셀 본문의 논리 clip과 이미 확정한 배경의 물리 Print clip을 분리한다.

use super::layer_tree::{CacheHint, LayerNode};
use super::{layer_tree, render_tree};
#[path = "cell_clip_core.rs"]
mod core;
use super::render_tree::{RenderNode, TableCellNode};
use std::collections::HashSet;

pub(super) fn build(
    node: &RenderNode,
    cell: &TableCellNode,
    children: Vec<LayerNode>,
    cache_hint: CacheHint,
) -> LayerNode {
    let background_ids: HashSet<_> = node
        .children
        .iter()
        .filter(|child| child.table_background_paint)
        .map(|child| child.id)
        .collect();
    core::build(node, cell, children, cache_hint, &background_ids)
}
