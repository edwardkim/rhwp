//! 문단 시작과 위 간격을 이미 포함한 첫 글줄 시작을 구분한다.

use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 합성 본문 뒤에서 원본 첫 줄과 수용된 흐름이 같은 물리 원점을 가리키면,
/// 로드 재구성이 옮겨 적은 줄 좌표의 축을 그 원점에 연결한다.
pub(super) fn source_aligned_rebuilt_base(
    previous: &Paragraph,
    current: &Paragraph,
    styles: &ResolvedStyleSet,
    source_column: (i32, f64),
    paragraph_y: f64,
    dpi: f64,
) -> Option<i32> {
    if previous.stored_text_partition_is_dirty()
        || previous.cell_format_vpos_dirty
        || !previous.controls.is_empty()
        || previous.source_line_seg_vertical_pos.is_some()
        || previous.line_segs.is_empty()
        || previous
            .line_segs
            .iter()
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0)
        || current.stored_text_partition_is_dirty()
        || current.cell_format_vpos_dirty
        || !current
            .controls
            .iter()
            .all(crate::renderer::control_is_body_flow_neutral_marker)
    {
        return None;
    }
    let source = current.source_line_seg_vertical_pos.as_ref()?;
    let first = current.line_segs.first()?;
    let original = *source.first()?;
    if source.len() != current.line_segs.len()
        || first.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || original == first.vertical_pos
        || original < source_column.0
    {
        return None;
    }
    let before = styles
        .para_styles
        .get(current.para_shape_id as usize)?
        .spacing_before;
    let accepted_first =
        crate::renderer::px_to_hwpunit(paragraph_y + before - source_column.1, dpi)
            .checked_add(source_column.0)?;
    if accepted_first != original {
        return None;
    }
    source_column
        .0
        .checked_add(first.vertical_pos.checked_sub(original)?)
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FlowOrigin {
    Paragraph(f64),
    Line(f64),
}

impl FlowOrigin {
    pub(crate) fn y(self) -> f64 {
        match self {
            Self::Paragraph(y) | Self::Line(y) => y,
        }
    }

    pub(crate) fn first_line_y(self, spacing_before: f64) -> f64 {
        match self {
            Self::Paragraph(y) => y + spacing_before,
            Self::Line(y) => y,
        }
    }
}
