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

/// 앞 실제 쪽이 자동 reset 뒤의 연속 저장 prefix를 전량 소비한 경우,
/// 다음 float가 다시 쓸 수 없는 원본 단의 선행 높이(HWPUNIT).
pub(super) fn consumed_prefix_before_float(
    st: &super::TypesetState,
    paragraphs: &[Paragraph],
    para_index: usize,
) -> Option<i32> {
    if !st.profile.hwp5_origin_hwpx()
        || st.profile.session_edited()
        || st.col_count != 1
        || st.current_zone_y_offset != 0.0
        || st.current_endnote_flow
    {
        return None;
    }
    if let Some(base) = accepted_terminal_cell_axis(st, paragraphs, para_index) {
        return Some(base);
    }
    let first = match st.current_items.first() {
        None if st.current_height == 0.0 => para_index,
        Some(
            PageItem::Table {
                para_index: first, ..
            }
            | PageItem::FullParagraph { para_index: first },
        ) if *first <= para_index => *first,
        _ => return None,
    };
    let page = st
        .pages
        .iter()
        .rev()
        .find(|page| !page.column_contents.is_empty())?;
    if page.section_index != st.section_index {
        return None;
    }
    let column = page.column_contents.last()?;
    let PageItem::FullParagraph { para_index: before } = column.items.last()? else {
        return None;
    };
    if before.checked_add(1)? != first {
        return None;
    }
    let valid = |paragraph: &Paragraph| {
        paragraph.column_type == ColumnBreakType::None
            && !paragraph.stored_text_partition_is_dirty()
            && !paragraph.cell_format_vpos_dirty
            && paragraph.source_line_seg_vertical_pos.is_none()
            && !paragraph.line_segs.is_empty()
            && paragraph.line_segs.iter().all(|line| {
                !super::is_synthetic_line_seg(line)
                    && line.line_height > 0
                    && line.line_spacing >= 0
            })
    };
    let current = paragraphs.get(first)?;
    if !valid(current) {
        return None;
    }
    let origin = current.line_segs.first()?.vertical_pos;
    if origin <= 0 {
        return None;
    }
    if !st.current_items.is_empty()
        && st.vpos_page_base != Some(origin)
        && st
            .paragraph_origins
            .get(&first)
            .is_none_or(|frame| frame.source_vpos_base != origin || frame.flow_start != 0.0)
    {
        return None;
    }
    let mut previous = None;
    for paragraph in paragraphs.get(first..=para_index)? {
        if !valid(paragraph) {
            return None;
        }
        for line in &paragraph.line_segs {
            if previous.is_some_and(|vpos| line.vertical_pos < vpos) {
                return None;
            }
            previous = Some(line.vertical_pos);
        }
    }
    let mut expected = origin;
    let mut reset = *before;
    loop {
        let paragraph = paragraphs.get(reset)?;
        if !paragraph.controls.is_empty() {
            // 실제 앞 쪽이 전체 float와 그 뒤의 빈 prefix를 이미 소비했다.
            // 저장본에서는 표의 끝 조각이 다음 원본 쪽에 남아 후속 줄이 양수
            // 원점에서 시작할 수 있다. 현재 전체 표는 앞 쪽에 닫혔으므로 그
            // 옛 끝 조각의 높이를 현재 단의 후속 float에 다시 더하지 않는다.
            // 0 reset과 달리 guide의 선언 높이를 추정하지 않고 실제 소유를 읽는다.
            let prefix = reset.checked_add(1)?;
            let first_tail = paragraphs.get(prefix)?;
            let source_tail = first_tail.line_segs.first()?.vertical_pos;
            let source_host = paragraph
                .layout_stored_first_origin
                .unwrap_or(paragraph.line_segs.first()?.vertical_pos);
            let position = column.items.iter().rposition(
                |item| matches!(item, PageItem::Table { para_index, .. } if *para_index == reset),
            )?;
            let PageItem::Table { control_index, .. } = &column.items[position] else {
                return None;
            };
            let Control::Table(table) = paragraph.controls.get(*control_index)? else {
                return None;
            };
            let accepted = column
                .paragraph_float_placements
                .get(&(reset, *control_index))?;
            let owns_prefix = prefix < first
                && column.items.len() - position - 1 == first - prefix
                && column.items[position + 1..].iter().enumerate().all(|(offset, item)| {
                    let owner = prefix + offset;
                    matches!(item, PageItem::FullParagraph { para_index } if *para_index == owner)
                        && !st.hidden_empty_paras.contains(&owner)
                        && paragraphs.get(owner).is_some_and(|p| {
                            valid(p) && p.controls.is_empty() && p.text.is_empty()
                        })
                });
            return (valid(paragraph)
                && paragraph.text.is_empty()
                && source_tail > 0
                && source_tail < source_host
                && crate::renderer::float_placement::is_para_topbottom_float(&table.common)
                && table.common.flow_with_text
                && accepted.occupied_bottom.is_finite()
                && accepted.occupied_bottom > accepted.table_top
                && owns_prefix)
                .then_some(origin);
        }
        if !valid(paragraph) || !paragraph.controls.is_empty() {
            return None;
        }
        let last = paragraph.line_segs.last()?;
        let end = last
            .vertical_pos
            .checked_add(last.line_height)?
            .checked_add(last.line_spacing)?;
        if end != expected {
            return None;
        }
        expected = paragraph.line_segs.first()?.vertical_pos;
        if expected == 0 {
            break;
        }
        reset = reset.checked_sub(1)?;
    }
    let guide_index = reset.checked_sub(1)?;
    let guide_position = column.items.iter().rposition(
        |item| matches!(item, PageItem::FullParagraph { para_index } if *para_index == guide_index),
    )?;
    // 실제 앞 쪽이 prefix의 모든 문단을 온전히 소유하며 감추지 않았다.
    if column.items.len() - guide_position - 1 != first - reset
        || column.items[guide_position + 1..].iter().enumerate().any(|(offset, item)| {
            !matches!(item, PageItem::FullParagraph { para_index } if *para_index == reset + offset)
                || st.hidden_empty_paras.contains(&(reset + offset))
        })
        || st.hidden_empty_paras.contains(&guide_index)
    { return None; }
    let PageItem::Table {
        para_index: host,
        control_index,
    } = column.items.get(guide_position.checked_sub(1)?)?
    else {
        return None;
    };
    let accepted = column
        .paragraph_float_placements
        .get(&(*host, *control_index))?;
    let guide = paragraphs.get(guide_index)?.line_segs.first()?;
    let consumed_end = accepted.occupied_bottom
        + hwpunit_to_px(
            guide.line_height.checked_add(guide.line_spacing)?,
            st.layout.dpi,
        );
    closed_float_reset_is_stale(
        st.profile,
        paragraphs,
        reset,
        &column.items[..=guide_position],
        Some(0),
        consumed_end,
        st.layout.dpi,
    )
    .then_some(origin)
}

/// 측정에서 확정한 단 원점을 paint와 뒤 문단의 저장 좌표 소비에 전달한다.
pub(super) fn record_consumed_prefix_origin(
    st: &mut super::TypesetState,
    paragraphs: &[Paragraph],
    para_index: usize,
    placement: Option<ParagraphFloatPlacement>,
) {
    if !st.current_items.is_empty() {
        return;
    }
    if let Some((base, placement)) =
        consumed_prefix_before_float(st, paragraphs, para_index).zip(placement)
    {
        st.record_paragraph_origin(crate::renderer::pagination::ParagraphOrigin {
            paragraph_index: para_index,
            spacing_before: 0.0,
            source_vpos_base: base,
            flow_start: 0.0,
            flow_end: placement.occupied_bottom,
        });
    }
}

/// 실제 첫 문단·표가 수용한 원본 단 축은 같은 사다리의 후속 float도 공유한다.
pub(super) fn accepted_source_prefix_origin(
    st: &super::TypesetState,
    paragraphs: &[Paragraph],
    para_index: usize,
) -> Option<i32> {
    if st.current_items.is_empty() {
        return None;
    }
    consumed_prefix_before_float(st, paragraphs, para_index)
}

/// 종료한 셀 조각이 인증한 단 축을 뒤의 연속 원본 사다리에 전달한다.
fn accepted_terminal_cell_axis(
    st: &super::TypesetState,
    paragraphs: &[Paragraph],
    para_index: usize,
) -> Option<i32> {
    let PageItem::PartialTable {
        para_index: host,
        start_row: 0,
        end_row: 1,
        is_continuation: true,
        start_cut,
        end_cut,
        is_block_split: false,
        start_cut_is_block: false,
        row_cursor_is_nested: false,
        ..
    } = st.current_items.first()?
    else {
        return None;
    };
    if start_cut.len() != 1 || start_cut[0] == 0 || !end_cut.is_empty() {
        return None;
    }
    let origin = st.paragraph_origins.get(host)?;
    if origin.flow_start != 0.0
        || origin.spacing_before != 0.0
        || origin.source_vpos_base <= 0
        || !origin.flow_end.is_finite()
    {
        return None;
    }
    let first = host.checked_add(1)?;
    let source = paragraphs.get(first)?.line_segs.first()?.vertical_pos;
    let expected_end = source.checked_sub(origin.source_vpos_base)?;
    let precision = hwpunit_to_px(2, st.layout.dpi);
    if (hwpunit_to_px(expected_end, st.layout.dpi) - origin.flow_end).abs() > precision {
        return None;
    }
    let mut previous = None;
    for paragraph in paragraphs.get(first..=para_index)? {
        if paragraph.column_type != ColumnBreakType::None
            || paragraph.stored_text_partition_is_dirty()
            || paragraph.cell_format_vpos_dirty
            || paragraph.source_line_seg_vertical_pos.is_some()
            || paragraph.line_segs.is_empty()
        {
            return None;
        }
        for line in &paragraph.line_segs {
            if super::is_synthetic_line_seg(line)
                || previous.is_some_and(|vpos| line.vertical_pos < vpos)
            {
                return None;
            }
            previous = Some(line.vertical_pos);
        }
    }
    let source_top = paragraphs.get(para_index)?.line_segs.first()?.vertical_pos;
    let expected_start = hwpunit_to_px(
        source_top.checked_sub(origin.source_vpos_base)?,
        st.layout.dpi,
    );
    ((expected_start - st.current_height).abs() <= precision).then_some(origin.source_vpos_base)
}
