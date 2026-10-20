//! 빈 글줄만 소유한 저장 단 밴드의 닫힘 간격.

use crate::model::{
    control::Control, paragraph::LineSeg, paragraph::Paragraph,
    provenance::LayoutCompatibilityProfile,
};
use crate::renderer::{hwpunit_to_px, pagination::PageItem};

/// 새 단 정의의 간격이 이어받는 후행 줄간격을 한 번만 제외한다.
/// 빈 문단의 실제 줄 높이와 문단 앞뒤 여백은 계속 점유한다.
pub(crate) fn stored_blank_tail_spacing(
    profile: LayoutCompatibilityProfile,
    items: &[PageItem],
    paragraphs: &[Paragraph],
    dpi: f64,
) -> Option<f64> {
    if !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
        || profile.session_edited()
        || !dpi.is_finite()
        || dpi <= 0.0
    {
        return None;
    }
    let [PageItem::FullParagraph { para_index }] = items else {
        return None;
    };
    let para = paragraphs.get(*para_index)?;
    let [line] = para.line_segs.as_slice() else {
        return None;
    };
    if !para.text.trim().is_empty()
        || para.stored_text_partition_is_dirty()
        || para.cell_format_vpos_dirty
        || line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || line.line_height <= 0
        || line.line_spacing < 0
        || para.controls.iter().any(|control| {
            !matches!(
                control,
                Control::SectionDef(_)
                    | Control::ColumnDef(_)
                    | Control::Header(_)
                    | Control::Footer(_)
                    | Control::PageNumberPos(_)
                    | Control::NewNumber(_)
            )
        })
    {
        return None;
    }
    Some(hwpunit_to_px(line.line_spacing, dpi))
}
