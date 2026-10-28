//! 수용한 본문 조각의 물리 점유 끝과 실제 흐름 종료점을 함께 보존한다.

use super::TypesetState;
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::float_placement::StoredColumnFrame;
use crate::renderer::typeset::paragraph::metrics::FormattedParagraph;
use crate::renderer::{hwpunit_to_px, px_to_hwpunit};

#[derive(Clone, Copy)]
pub(in crate::renderer::typeset) struct ParagraphFragmentBounds {
    pub content_bottom: f64,
    pub flow_end: f64,
    /// 내용 하단과 같은 구성 결과가 발행한 마지막 줄간격.
    pub last_line_spacing: Option<f64>,
    /// 실제 조각 원점과 같은 저장 축 및 단 앵커다. 재조판 줄은 발행하지 않는다.
    pub stored_frame: Option<StoredColumnFrame>,
}

impl TypesetState {
    /// pre/post-text의 배치 원점에서 같은 구성 높이를 한 번 기록한다.
    pub(in crate::renderer::typeset) fn record_host_text_fragment(
        &mut self,
        owner: usize,
        range: std::ops::Range<usize>,
        para: &Paragraph,
        fmt: &FormattedParagraph,
        dpi: f64,
    ) {
        if let Some(composition) = fmt.host_composition.as_ref() {
            self.data
                .host_compositions
                .insert(owner, composition.clone());
        }
        let content_height = fmt.line_content_height(range.clone(), 0.0);
        let advance = fmt.line_advances_sum(range.clone());
        let stored_frame = self.host_fragment_stored_frame(range.clone(), para, fmt, dpi);
        self.data.paragraph_fragment_content_bottoms.insert(
            (owner, range.end),
            ParagraphFragmentBounds {
                content_bottom: self.data.current_height + content_height,
                flow_end: self.data.current_height + advance,
                last_line_spacing: fmt.line_tail_spacing(range.end),
                stored_frame,
            },
        );
    }

    /// 실제 배치가 소비한 줄과 저장 줄의 기하가 같은 경우에만 평행 이동 축을 발행한다.
    fn host_fragment_stored_frame(
        &self,
        range: std::ops::Range<usize>,
        para: &Paragraph,
        fmt: &FormattedParagraph,
        dpi: f64,
    ) -> Option<StoredColumnFrame> {
        if self.data.profile.hwp3_layout()
            || !(self.data.profile.hwp5_stored_pagination_layout()
                || self.data.profile.hwpx_stored_layout())
            || self.data.profile.session_edited()
            || self.data.current_endnote_flow
            || para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || fmt.exact_reflow_text_box
            || fmt.computed_host_lines.is_some()
            || fmt.square_host_plan.is_some()
            || fmt.line_count() != para.line_segs.len()
            || range.is_empty()
            || range.end > para.line_segs.len()
            || !dpi.is_finite()
            || dpi <= 0.0
            || !self.data.current_height.is_finite()
            || !self.data.vpos_col_anchor.is_finite()
        {
            return None;
        }
        if para
            .source_line_seg_vertical_pos
            .as_ref()
            .is_some_and(|source| {
                source.len() != para.line_segs.len()
                    || source
                        .iter()
                        .zip(&para.line_segs)
                        .any(|(source, row)| *source != row.vertical_pos)
            })
        {
            return None;
        }
        let rows = &para.line_segs[range.clone()];
        if rows.iter().enumerate().any(|(offset, row)| {
            let index = range.start + offset;
            row.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                || row.line_height <= 0
                || fmt.line_heights[index] != hwpunit_to_px(row.line_height, dpi)
                || fmt.line_spacings[index] != hwpunit_to_px(row.line_spacing, dpi)
        }) || rows.windows(2).any(|pair| {
            pair[0]
                .vertical_pos
                .checked_add(pair[0].line_height)
                .and_then(|end| end.checked_add(pair[0].line_spacing))
                != Some(pair[1].vertical_pos)
        }) {
            return None;
        }
        let anchor = self.data.vpos_col_anchor;
        let offset = px_to_hwpunit(self.data.current_height - anchor, dpi);
        rows[0]
            .vertical_pos
            .checked_sub(offset)
            .map(|source_origin_hu| StoredColumnFrame {
                source_origin_hu,
                column_anchor_y: anchor,
            })
    }
}
