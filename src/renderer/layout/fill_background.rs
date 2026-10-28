//! BorderFill의 paint 결과. 문단과 셀이 같은 색·무늬·그라데이션·그림 의미를 쓴다.

use super::*;
use crate::renderer::style_resolver::ResolvedBorderStyle;

/// 실제 줄 전진에서 생산한 배경 밴드. anchor는 후속 배치 이동까지 따라간다.
#[derive(Clone, Debug)]
pub(super) struct ParagraphFillBand {
    pub line_index: Option<usize>,
    pub anchor: Option<(NodeId, f64)>,
    pub top: f64,
    pub bottom: f64,
}

/// 저장 줄의 가로 조각은 같은 물리 줄의 채우기를 공유한다.
pub(super) fn paragraph_fill_line_index(para: Option<&Paragraph>, mut index: usize) -> usize {
    while para.is_some_and(|para| {
        crate::renderer::height_measurer::stored_seg_is_row_fragment(para, index)
    }) {
        index -= 1;
    }
    index
}

impl LayoutEngine {
    pub(super) fn border_fill_background_nodes(
        tree: &mut PageLayoutContext,
        style: Option<&ResolvedBorderStyle>,
        bounds: BoundingBox,
        bin_data_content: &[BinDataContent],
    ) -> Vec<RenderNode> {
        let mut nodes = Vec::new();
        let fill_color = style.and_then(|s| s.fill_color);
        let pattern = style.and_then(|s| s.pattern);
        let gradient = style.and_then(|s| s.gradient.clone());
        if fill_color.is_some() || pattern.is_some() || gradient.is_some() {
            nodes.push(RenderNode::new(
                tree.next_id(),
                RenderNodeType::Rectangle(RectangleNode::new(
                    0.0,
                    ShapeStyle {
                        fill_color,
                        pattern,
                        ..Default::default()
                    },
                    gradient,
                )),
                bounds,
            ));
        }
        if let Some(fill) = style.and_then(|s| s.image_fill.as_ref()) {
            if let Some(bytes) = find_bin_data_bytes(bin_data_content, fill.bin_data_id) {
                // ResolvedImageFill은 이진 순서, ImageNode는 화면 순서다(#6895).
                let (brightness, contrast) = fill.display_brightness_contrast();
                nodes.push(RenderNode::new(
                    tree.next_id(),
                    RenderNodeType::Image(ImageNode {
                        fill_mode: Some(fill.fill_mode),
                        brightness,
                        contrast,
                        effect: fill.effect,
                        ..ImageNode::new(fill.bin_data_id, Some(bytes))
                    }),
                    bounds,
                ));
            }
        }
        nodes
    }
}
