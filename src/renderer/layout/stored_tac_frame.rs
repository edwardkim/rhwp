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
