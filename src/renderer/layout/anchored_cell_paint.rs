//! 셀 개체의 편집 소유를 보존하면서 쪽·표의 그리기 순서를 조립한다.

use super::super::page_layout::LayoutRect;
use super::super::render_tree::*;
use super::super::style_resolver::ResolvedStyleSet;
use super::LayoutEngine;
use crate::model::bin_data::BinDataContent;
use crate::model::shape::TextWrap;
use crate::model::style::Alignment;

impl LayoutEngine {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn layout_picture(
        &self,
        tree: &mut PageLayoutContext,
        parent_node: &mut RenderNode,
        picture: &crate::model::image::Picture,
        container: &LayoutRect,
        bin_data_content: &[BinDataContent],
        alignment: Alignment,
        section_index: Option<usize>,
        para_index: Option<usize>,
        control_index: Option<usize>,
        cell_ctx: Option<&crate::renderer::layout::CellContext>,
        // [#6284] 캡션 문단 조판에 필요하다.
        styles: &ResolvedStyleSet,
    ) {
        let children_before = parent_node.children.len();
        // [Task #825] 본문 picture 경로 — header_footer_ref = None
        self.layout_picture_full(
            tree,
            parent_node,
            picture,
            container,
            bin_data_content,
            alignment,
            section_index,
            para_index,
            control_index,
            None,
            cell_ctx,
            styles,
        );
        if cell_ctx.is_some_and(|ctx| !ctx.in_textbox)
            && !picture.common.treat_as_char
            && matches!(picture.common.text_wrap, TextWrap::BehindText)
        {
            let stable_index = cell_ctx
                .and_then(|ctx| ctx.path.last())
                .map(|entry| {
                    Self::object_stable_index(entry.cell_para_index, control_index.unwrap_or(0))
                })
                .unwrap_or(0);
            let mut layer = RenderLayerInfo::new(
                Some(TextWrap::BehindText),
                picture.common.z_order,
                stable_index,
            );
            // 셀의 편집 소유는 ImageNode.cell_context에 남기고 그리기 소속만 구분한다.
            if !picture.common.flow_with_text
                && !crate::model::border_paint::is_cell_frame_attributes(&picture.common)
            {
                layer.independent_page_paint = parent_node
                    .children
                    .get(children_before)
                    .map(|first| first.id);
            }
            for child in parent_node.children.iter_mut().skip(children_before) {
                child.set_layer(layer);
                child.cell_frame_paint =
                    crate::model::border_paint::is_cell_frame_attributes(&picture.common);
            }
        }
    }

    /// 쪽 순서를 따르는 그림은 같은 쪽의 도형들과 정렬한다. 셀 주소·절대 좌표는
    /// ImageNode에 이미 있으므로 재배치하거나 편집 소유를 쪽 본문으로 바꾸지 않는다.
    pub(super) fn settle_cell_anchored_objects(root: &mut RenderNode) {
        let mut page_paint = Vec::new();
        Self::order_cell_anchored_objects(root, None, &mut page_paint);
        page_paint.sort_by_key(|parts| Self::page_picture_sort_key(parts));
        for parts in page_paint {
            let key = Self::page_picture_sort_key(&parts);
            let mut insert_at = root.children.len();
            for (index, child) in root.children.iter().enumerate() {
                if child.layer.is_none_or(|layer| layer.master_page) {
                    continue;
                }
                insert_at = index + 1;
                if Self::paper_node_sort_key(child) > key {
                    insert_at = index;
                    break;
                }
            }
            root.children.splice(insert_at..insert_at, parts);
        }
    }

    fn page_picture_sort_key(parts: &[RenderNode]) -> (u8, i32, DocPath) {
        // 원본 주소가 없는 테두리의 빈 경로가 bitmap보다 앞서지 않게, 하나의
        // 원본 그림에서 만든 모든 부분에 같은 개체 정렬키를 적용한다.
        let owner = parts
            .iter()
            .find(|part| crate::renderer::render_tree::doc_path_for_node(part).is_some())
            .unwrap_or(&parts[0]);
        Self::paper_node_sort_key(owner)
    }

    fn order_cell_anchored_objects(
        node: &mut RenderNode,
        inherited_layer: Option<RenderLayerInfo>,
        page_paint: &mut Vec<Vec<RenderNode>>,
    ) {
        if !node.visible {
            return;
        }
        let active_layer = node.layer.or(inherited_layer);
        for child in &mut node.children {
            Self::order_cell_anchored_objects(child, active_layer, page_paint);
        }
        if !matches!(node.node_type, RenderNodeType::TableCell(_)) {
            return;
        }
        let lifts = |child: &RenderNode| {
            child
                .layer
                .is_some_and(|layer| !matches!(layer.text_wrap, Some(TextWrap::BehindText)))
        };
        if !node.children.iter().any(|child| child.layer.is_some()) {
            return;
        }
        // 셀 채우기 등 순수 장식은 내용 앞에 유지한다. 셀 흐름을 따르는 그림은
        // 이 배경 뒤, 셀 본문 앞에 놓고 그리기 계층을 표에서 상속한다.
        let content_start = node
            .children
            .iter()
            .position(|child| {
                child.layer.is_some()
                    || matches!(child.node_type, RenderNodeType::TextLine(_))
                    || crate::renderer::render_tree::doc_path_for_node(child).is_some()
            })
            .unwrap_or(node.children.len());
        let content = node.children.split_off(content_start);
        let mut kept: Vec<RenderNode> = Vec::with_capacity(node.children.len());
        let mut lifted: Vec<RenderNode> = Vec::new();
        let mut behind: Vec<RenderNode> = Vec::new();
        let mut previous_page_group = None;
        for child in content {
            if let Some(group) = child.layer.and_then(|layer| layer.independent_page_paint) {
                if previous_page_group == Some(group) {
                    page_paint.last_mut().unwrap().push(child);
                } else {
                    page_paint.push(vec![child]);
                }
                previous_page_group = Some(group);
                continue;
            }
            previous_page_group = None;
            if lifts(&child) {
                lifted.push(child);
            } else if child
                .layer
                .is_some_and(|layer| matches!(layer.text_wrap, Some(TextWrap::BehindText)))
            {
                behind.push(child);
            } else {
                kept.push(child);
            }
        }
        let order = |child: &RenderNode| {
            child
                .layer
                .map(|layer| (layer.z_order, layer.stable_index))
                .unwrap_or((0, 0))
        };
        behind.sort_by_key(order);
        lifted.sort_by_key(order);
        for child in &mut behind {
            child.layer = active_layer;
            if let RenderNodeType::Image(image) = &mut child.node_type {
                image.text_wrap = active_layer
                    .and_then(|layer| layer.text_wrap)
                    .or(Some(TextWrap::Square));
            }
        }
        node.children.extend(behind);
        kept.extend(lifted);
        node.children.extend(kept);
    }
}
