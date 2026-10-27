//! 회피 공간 확정 뒤 단일 저장 TAC 줄의 물리 원점과 끝점을 공유한다.

use super::super::paragraph::metrics::FormattedParagraph;
use super::stored_tac::StoredTacControlPlacement;
use crate::model::{
    control::Control, paragraph::Paragraph, provenance::LayoutCompatibilityProfile,
};
use crate::renderer::{
    float_placement::{InlineBoxPlacement, StoredColumnFrame},
    hwpunit_to_px,
};

#[allow(clippy::too_many_arguments)]
pub(in crate::renderer::typeset) fn prepare(
    para: &Paragraph,
    control_index: usize,
    next_para: Option<&Paragraph>,
    next_spacing_before: f64,
    fmt: &FormattedParagraph,
    measured_body_height: f64,
    profile: LayoutCompatibilityProfile,
    source_frame_expanded: bool,
    flow_origin: f64,
    leading_spacing: f64,
    available_height: f64,
    source_frame: Option<StoredColumnFrame>,
    dpi: f64,
) -> Option<StoredTacControlPlacement> {
    // 투영으로 커진 줄은 원본의 물리 프레임이 아니다. 이 단축에서
    // 저장 줄로 승인하면 일반 배치의 뒤 본문 strict fit을 건너뛴다.
    if source_frame_expanded
        || profile.session_edited()
        || !(profile.hwp5_stored_pagination_layout() || profile.hwpx_stored_layout())
        || para.cell_format_vpos_dirty
        || para.line_segs.len() != 1
    {
        return None;
    }
    let seg = crate::renderer::composer::stored_first_tac_line(para)?;
    let Control::Table(table) = para.controls.get(control_index)? else {
        return None;
    };
    // 이 줄 전체를 소유한 표 하나만 확정한다. 그림·도형의 독립 줄이나
    // 표 안 각주 예약, 다른 표의 줄을 이 단축에서 대신 소비하지 않는다.
    if table.caption.is_some()
        || super::stored_tac::table_has_notes(table)
        || table.cells.iter().any(|cell| cell.dirty_flag)
        || para.controls.iter().enumerate().any(|(index, control)| {
            index != control_index
                && match control {
                    Control::Table(_) => true,
                    Control::Picture(picture) => picture.common.treat_as_char,
                    Control::Shape(shape) => shape.common().treat_as_char,
                    _ => false,
                }
        })
        || crate::renderer::layout::control_line_seg_index(para, control_index) != Some(0)
        || (measured_body_height - hwpunit_to_px(table.common.height as i32, dpi)).abs()
            > dpi / 7200.0
        || seg.line_spacing < 0
        || !crate::renderer::composer::tac_next_line_full_spacing(
            para,
            next_para,
            fmt.spacing_after,
            next_spacing_before,
            seg,
            profile,
            dpi,
        )
    {
        return None;
    }
    // 저장 줄 높이는 바깥 여백을 이미 담는다. 회피된 현재 원점에서
    // 이를 한 번 소비하고, 다음 저장 줄이 입증한 뒤 간격을 전량 남긴다.
    let origin = flow_origin + leading_spacing;
    let end = origin
        + hwpunit_to_px(seg.line_height, dpi)
        + hwpunit_to_px(seg.line_spacing, dpi)
        + fmt.spacing_after;
    if end > available_height {
        return None;
    }
    Some(StoredTacControlPlacement {
        control_index,
        inline: InlineBoxPlacement {
            x: 0.0,
            // paint가 위 바깥 여백을 소비하므로 metadata에는 pen만 전달한다.
            y: origin,
            clearance: 0.0,
            resolved_x: false,
            advance_end: Some(end),
            stored_horizontal_pen: None,
            stored_column_frame: source_frame
                .filter(|frame| frame.accepts_line_origin(seg.vertical_pos, origin, dpi)),
        },
        end,
        rebase_line_origin: false,
    })
}
