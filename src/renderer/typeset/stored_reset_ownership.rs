//! 닫힌 float의 저장 끝 안내줄과 현재 흐름이 같은 물리 프레임인지 확인한다.

use crate::model::{
    control::Control,
    paragraph::{ColumnBreakType, Paragraph},
    provenance::LayoutCompatibilityProfile,
};
use crate::renderer::{
    float_placement::ParagraphFloatPlacement, hwpunit_to_px, pagination::PageItem,
};

/// HWPX에 옮긴 저장 reset은 원본 HWP5의 쪽 원점이다. 현재 수용한 표의
/// 끝이 원본 안내줄과 달라졌다면 그 reset을 현재 쪽의 경계로 빌리지 않는다.
/// 빈 후속 줄을 수용하기 전과 그 줄 상자를 실제 소비한 뒤를 함께 인증한다.
/// 잔여 fit이나 채움률이 아니라 같은 원본 상자·실제 소비 위치를 인증한다.
pub(super) fn closed_float_reset_is_stale(
    profile: LayoutCompatibilityProfile,
    paragraphs: &[Paragraph],
    para_index: usize,
    current_items: &[PageItem],
    source_frame_base: Option<i32>,
    current_height: f64,
    dpi: f64,
) -> bool {
    fn inspect(
        paragraphs: &[Paragraph],
        para_index: usize,
        current_items: &[PageItem],
        current_height: f64,
        dpi: f64,
    ) -> Option<bool> {
        let current = paragraphs.get(para_index)?;
        let guide_index = para_index.checked_sub(1)?;
        let host_index = para_index.checked_sub(2)?;
        let guide = paragraphs.get(guide_index)?;
        let host = paragraphs.get(host_index)?;
        let [guide_line] = guide.line_segs.as_slice() else {
            return None;
        };
        if current.column_type != ColumnBreakType::None
            || !current.controls.is_empty()
            || current.line_segs.first()?.vertical_pos != 0
            || super::is_synthetic_line_seg(current.line_segs.first()?)
            || current.stored_text_partition_is_dirty()
            || current.cell_format_vpos_dirty
            || !guide.text.is_empty()
            || !guide.controls.is_empty()
            || guide.column_type != ColumnBreakType::None
            || guide.stored_text_partition_is_dirty()
            || guide.cell_format_vpos_dirty
            || host.source_line_seg_vertical_pos.is_some()
            || guide.source_line_seg_vertical_pos.is_some()
            || super::is_synthetic_line_seg(guide_line)
            || guide_line.line_height <= 0
            || guide_line.line_spacing < 0
            || !current_height.is_finite()
        {
            return None;
        }
        let (table_item, guide_consumed) = match current_items {
            [.., table @ PageItem::Table { .. }] => (table, false),
            [.., table @ PageItem::Table { .. }, PageItem::FullParagraph { para_index }]
                if *para_index == guide_index =>
            {
                (table, true)
            }
            _ => return None,
        };
        let PageItem::Table {
            para_index: owner,
            control_index,
        } = table_item
        else {
            return None;
        };
        if *owner != host_index {
            return None;
        }
        let Control::Table(table) = host.controls.get(*control_index)? else {
            return None;
        };
        let caption =
            crate::renderer::float_placement::stored_empty_host_frame::stored_bottom_caption_hu(
                table,
            )?;
        let declared_height = i64::from(table.common.height).checked_add(caption)?;
        let declared_height = i32::try_from(declared_height).ok()?;
        let origin = host
            .layout_stored_first_origin
            .unwrap_or(host.line_segs.first()?.vertical_pos);
        let frame = ParagraphFloatPlacement::from_empty_stored_ladder_host(
            host,
            guide,
            table,
            hwpunit_to_px(origin, dpi),
            0.0,
            hwpunit_to_px(declared_height, dpi),
            false,
            dpi,
        )?;
        // 공유 프레임의 원본 등식은 ±2 HWPUNIT의 저장 양자화만 허용한다.
        let precision = hwpunit_to_px(2, dpi);
        let guide_top = hwpunit_to_px(
            guide
                .layout_stored_first_origin
                .unwrap_or(guide_line.vertical_pos),
            dpi,
        );
        if (frame.occupied_bottom - guide_top).abs() > precision {
            return None;
        }
        let host_shift =
            hwpunit_to_px(host.line_segs.first()?.vertical_pos, dpi) - hwpunit_to_px(origin, dpi);
        let current_guide_top = hwpunit_to_px(guide_line.vertical_pos, dpi);
        if (frame.occupied_bottom + host_shift - current_guide_top).abs() > precision {
            return None;
        }
        let guide_advance = if guide_consumed {
            hwpunit_to_px(
                guide_line
                    .line_height
                    .checked_add(guide_line.line_spacing)?,
                dpi,
            )
        } else {
            0.0
        };
        // 항목에 남았다는 사실은 높이 소비의 증거가 아니다. 숨긴 guide나
        // 미전진 항목은 현재 줄 끝에 도달하지 않았으므로 인증하지 않는다.
        if (current_height - (current_guide_top + guide_advance)).abs() > precision {
            return None;
        }
        Some((current_height - (guide_top + guide_advance)).abs() > precision)
    }
    // 다른 단/zone으로 재기준한 축은 원본 쪽 좌표와 직접 비교하지 않는다.
    let certificate =
        (profile.hwp5_origin_hwpx() && !profile.session_edited() && source_frame_base == Some(0))
            .then(|| inspect(paragraphs, para_index, current_items, current_height, dpi))
            .flatten();
    if std::env::var("RHWP_DIAG_RESET_OWNER")
        .is_ok_and(|filter| filter == "all" || filter.parse().ok() == Some(para_index))
    {
        let source = |index: usize| {
            paragraphs.get(index).map(|paragraph| {
                (
                    paragraph.line_segs.first(),
                    paragraph.layout_stored_first_origin,
                    paragraph
                        .source_line_seg_vertical_pos
                        .as_ref()
                        .map(Vec::len),
                    paragraph.controls.len(),
                    paragraph.text.is_empty(),
                    paragraph.column_type,
                )
            })
        };
        eprintln!("RESET_OWNER pi={para_index} origin={} base={source_frame_base:?} current={current_height} certificate={certificate:?} host={:?} guide={:?} tail={:?}", profile.hwp5_origin_hwpx(), source(para_index.saturating_sub(2)), source(para_index.saturating_sub(1)), &current_items[current_items.len().saturating_sub(3)..]);
    }
    certificate == Some(true)
}
