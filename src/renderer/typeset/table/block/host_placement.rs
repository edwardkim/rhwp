//! Read-only occupied-band constraint shared by whole and split placement.
use crate::renderer::{
    float_placement::ParagraphFloatPlacement,
    typeset::{hwpunit_to_px, TypesetState},
};

/// 표가 빈 호스트 줄 앞에 놓이면 앞 점유 끝·표 바깥 상자·뒤 저장 줄이 모두 닫힌다.
pub(super) fn saved_table_before_whitespace_line(
    host: &crate::model::paragraph::Paragraph,
    next: &crate::model::paragraph::Paragraph,
    table: &crate::model::table::Table,
    previous_bottom_hu: i64,
    measured_height: f64,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
    dpi: f64,
) -> Option<ParagraphFloatPlacement> {
    use crate::model::{control::Control, paragraph::ColumnBreakType, shape::VertAlign};
    use crate::renderer::{
        float_placement::{is_para_topbottom_float, ParagraphFloatFlow},
        px_to_hwpunit,
        typeset::is_synthetic_line_seg,
    };
    let [line] = host.line_segs.as_slice() else {
        return None;
    };
    let next_line = next.line_segs.first()?;
    if host.text.is_empty()
        || !host.text.trim().is_empty()
        || !matches!(host.controls.as_slice(), [Control::Table(_)])
        || !next.controls.is_empty()
        || [host, next].iter().any(|para| {
            para.column_type != ColumnBreakType::None
                || para.stored_text_partition_is_dirty()
                || para.cell_format_vpos_dirty
                || para.source_line_seg_vertical_pos.is_some()
        })
        || [line, next_line].iter().any(|line| {
            is_synthetic_line_seg(line) || line.line_height <= 0 || line.line_spacing < 0
        })
        || line.segment_width <= 0
        || !is_para_topbottom_float(&table.common)
        || !table.common.flow_with_text
        || table.common.vert_align != VertAlign::Top
        || table.common.vertical_offset != 0
        || table.caption.is_some()
        || table.outer_margin_top < 0
        || table.outer_margin_bottom < 0
        || !measured_height.is_finite()
        || table.common.height == 0
        || i64::from(px_to_hwpunit(measured_height, dpi)) != i64::from(table.common.height)
    {
        return None;
    }
    let host_style = styles.para_styles.get(host.para_shape_id as usize)?;
    let next_style = styles.para_styles.get(next.para_shape_id as usize)?;
    let line_top = i64::from(line.vertical_pos);
    let next_top = line_top
        + i64::from(line.line_height)
        + i64::from(line.line_spacing)
        + i64::from(px_to_hwpunit(host_style.spacing_after, dpi))
        + i64::from(px_to_hwpunit(next_style.spacing_before, dpi));
    let table_top =
        line_top - i64::from(table.common.height) - i64::from(table.outer_margin_bottom);
    let outer_top = table_top - i64::from(table.outer_margin_top);
    if outer_top < 0
        || (outer_top - previous_bottom_hu).abs() > 2
        || (next_top - i64::from(next_line.vertical_pos)).abs() > 2
    {
        return None;
    }
    let to_px = |value: i64| value as f64 * dpi / 7200.0;
    Some(ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::NextLine,
        anchor_y: to_px(outer_top),
        stored_host_origin: Some(to_px(line_top)),
        stored_successor_line_origin: Some(to_px(i64::from(next_line.vertical_pos))),
        table_left: None,
        table_top: to_px(table_top),
        occupied_bottom: to_px(line_top),
    })
}

pub(in crate::renderer::typeset) struct HostPlacementConstraint<'a> {
    pub(in crate::renderer::typeset) table: &'a crate::model::table::Table,
    pub(in crate::renderer::typeset) has_preceding_coanchored_float: bool,
    pub(in crate::renderer::typeset) dpi: f64,
}
impl HostPlacementConstraint<'_> {
    pub(in crate::renderer::typeset) fn constrain(
        &self,
        mut placement: ParagraphFloatPlacement,
        st: &TypesetState,
    ) -> ParagraphFloatPlacement {
        let table = self.table;
        let has_preceding_coanchored_float = self.has_preceding_coanchored_float;
        if table.common.allow_overlap {
            return placement;
        }
        let outer_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
        // A zero-offset co-anchored float consumes flow without adding a
        // visible exclusion. Preserve that occupied floor before publishing
        // a final placement; consulting only the exclusion list loses it.
        if has_preceding_coanchored_float {
            let shift = (st.current_height + outer_top - placement.table_top).max(0.0);
            placement.table_top += shift;
            placement.occupied_bottom += shift;
        }
        // 흐름이 소유한 같은 앵커의 표 상자는 현재 커서에서 배치한다.
        // 본문 배제 띠는 글줄의 수용 조건이며 앞 문단의 떠 있는 표와
        // 이 흐름 표의 물리 충돌을 입증하지 않는다.
        if matches!(
            placement.flow,
            crate::renderer::float_placement::ParagraphFloatFlow::CoanchoredTable
        ) {
            return placement;
        }
        placement.clear_occupied_bands(
            st.visible_float_exclusions
                .iter()
                .map(|zone| zone.top..zone.bottom + outer_top),
        )
    }
}
