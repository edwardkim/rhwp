//! 저장 TAC 줄의 표 본체·캡션·바깥여백 소유 관계.

use crate::model::shape::CaptionDirection;
use crate::model::table::Table;
use crate::renderer::hwpunit_to_px;
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 줄이 표의 바깥 상자를 정확히 소유하면 캡션을 포함한 본체 높이를 돌려준다.
pub(super) fn owned_body_height(
    table: &Table,
    body_height: f64,
    line_height: f64,
    styles: &ResolvedStyleSet,
    dpi: f64,
    tolerance: f64,
) -> Option<f64> {
    let top = hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    let bottom = hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    if top <= 0.0 && bottom <= 0.0 {
        return None;
    }
    let caption_extra = table.caption.as_ref().map_or(0.0, |caption| {
        if !matches!(
            caption.direction,
            CaptionDirection::Top | CaptionDirection::Bottom
        ) {
            return 0.0;
        }
        let height = crate::renderer::composer::caption_height_px_in_context(
            &table.caption,
            Some(styles),
            dpi,
        );
        if height > 0.0 {
            height + hwpunit_to_px(i32::from(caption.spacing), dpi)
        } else {
            0.0
        }
    });
    [body_height, body_height + caption_extra]
        .into_iter()
        .find(|height| (line_height - (height + top + bottom)).abs() <= tolerance)
}

/// 한 저장 줄을 소유한 TAC 표(줄 높이 = 위 바깥여백 + 본체 + 아래 바깥여백)와 같은 줄에 놓인
/// 다른 TAC 표의 위끝.
///
/// 소유자가 `줄 위끝 + owner_om_top` 에 앉으면 그 본체 높이의 85% 지점이 공유 기준선이다
/// (글자처럼 취급 개체는 높이의 85% 가 기준선 위). 동반 표는 같은 기준선에 자기 높이의 85% 를
/// 올려 `줄 위끝 + owner_om_top + 0.85 × (owner_h − table_h)` 에 앉는다. 자기 바깥여백은 더하지
/// 않는다(#7049·#7150 실측). 글줄 흐름 경로와 TAC 표만 담은 문단 경로가 이 하나를 함께 쓴다.
pub(super) fn companion_table_top(
    line_top: f64,
    owner_h: f64,
    owner_om_top: f64,
    table_h: f64,
) -> f64 {
    (line_top + owner_om_top + (owner_h - table_h) * 0.85).max(line_top)
}

/// 방출된 뒤 조각이 표와 다른 실제 저장 줄을 소유하는지 확인한다.
/// 공백·제어뿐인 줄도 높이와 문단 끝 간격을 소유하며 표 밴드의 사본과 구분한다.
pub(super) fn independent_stored_tail(
    para: &crate::model::paragraph::Paragraph,
    start: usize,
    end: usize,
) -> bool {
    use crate::model::{control::Control, paragraph::LineSeg};
    let Some(lines) = para
        .line_segs
        .get(start..end)
        .filter(|lines| !lines.is_empty())
    else {
        return false;
    };
    lines
        .iter()
        .all(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0 && line.line_height > 0)
        && para.controls.iter().enumerate().all(|(index, control)| {
            if !matches!(control, Control::Table(table) if table.common.treat_as_char) {
                return true;
            }
            super::control_line_seg_index(para, index)
                .filter(|line| *line < start)
                .and_then(|line| para.line_segs.get(line))
                .is_some_and(|host| {
                    i64::from(lines[0].vertical_pos)
                        >= i64::from(host.vertical_pos)
                            + i64::from(host.line_height)
                            + i64::from(host.line_spacing)
                })
        })
}
