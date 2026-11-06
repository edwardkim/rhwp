//! 수용 단계에서 확정한 소유 단의 각주를 같은 폭과 줄 메트릭으로 배치한다.
use super::*;
use crate::renderer::footnote_frames::FramePolicy;
use std::collections::BTreeMap;

/// 글줄의 잉크가 아니라 실제 수용한 본문 흐름의 끝을 단별로 공유한다.
pub(super) struct BodyFlowBottoms {
    start: f64,
    maximum: f64,
    columns: BTreeMap<u16, f64>,
}

impl BodyFlowBottoms {
    pub(super) fn new(start: f64) -> Self {
        Self {
            start,
            maximum: start,
            columns: BTreeMap::new(),
        }
    }

    pub(super) fn record(&mut self, column: u16, bottom: f64) {
        if bottom.is_finite() {
            self.maximum = self.maximum.max(bottom);
            self.columns
                .entry(column)
                .and_modify(|end| *end = end.max(bottom))
                .or_insert(bottom);
        }
    }

    fn for_column(&self, column: u16) -> f64 {
        self.columns.get(&column).copied().unwrap_or(self.start)
    }
}

impl LayoutEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_footnote_area(
        &self,
        tree: &mut PageRenderTree,
        page: &PageContent,
        paragraphs: &[Paragraph],
        shape: &FootnoteShape,
        styles: &ResolvedStyleSet,
        layout: &PageLayoutInfo,
        images: &[BinDataContent],
        body_bottoms: &BodyFlowBottoms,
    ) {
        let policy = FramePolicy {
            placement: shape.placement,
            columns: layout.column_areas.len() as u16,
        };
        let mut groups = BTreeMap::<u16, Vec<usize>>::new();
        for (index, note) in page.footnotes.iter().enumerate() {
            groups
                .entry(policy.target(note.column_index))
                .or_default()
                .push(index);
        }
        for (column, indices) in groups {
            let notes = indices
                .iter()
                .map(|&index| page.footnotes[index].clone())
                .collect::<Vec<_>>();
            let mut area = policy.area(layout, column);
            area.height =
                self.estimate_footnote_area_height(&notes, paragraphs, shape, styles, area.width);
            area.y = layout.body_area.y + layout.body_area.height - area.height;
            if shape.print_inline_after_text {
                area.y = if policy.spans_page() {
                    body_bottoms.maximum
                } else {
                    body_bottoms.for_column(column)
                };
            }

            let id = tree.next_id();
            let mut node =
                RenderNode::new(id, RenderNodeType::FootnoteArea, layout_rect_to_bbox(&area));
            self.layout_footnote_area(
                tree.frame_mut(),
                &mut node,
                &notes,
                paragraphs,
                styles,
                &area,
                shape,
                &indices,
                images,
            );
            tree.root.children.push(node);
        }
    }
}
