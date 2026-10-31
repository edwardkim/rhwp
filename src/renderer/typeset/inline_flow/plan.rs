//! 읽기 전용 단/배제 영역에서 inline 배치 후보를 계산한다. 페이지 상태는 변경하지 않는다.

use std::collections::BTreeMap;

use crate::model::paragraph::Paragraph;
use crate::renderer::float_placement::ObjectPlacementFrame;
use crate::renderer::height_measurer::MeasuredTable;
use crate::renderer::inline_flow::{self, InlineFlowPlan};
use crate::renderer::layout_frame::{FrameExclusion, FrameExclusionPolicy};
use crate::renderer::page_layout::LayoutRect;
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 참조만 보관하며 배제 영역 복사는 기존 스타일 조회 이후에 수행한다.
pub(in crate::renderer::typeset) struct InlineFlowInput<'a> {
    pub column: LayoutRect,
    pub body: &'a LayoutRect,
    pub page_width: f64,
    pub page_height: f64,
    pub start: f64,
    pub exclusions: Option<&'a BTreeMap<(usize, usize), FrameExclusion>>,
    pub visible_float_exclusions: Option<&'a [super::super::VisibleFloatExclusion]>,
}

pub(in crate::renderer::typeset) fn build_plan_with_storage<S: crate::storage::Storage>(
    input: InlineFlowInput<'_>,
    para: &Paragraph,
    para_index: usize,
    styles: &ResolvedStyleSet,
    tables: &[MeasuredTable],
    dpi: f64,
    storage: &S,
) -> Result<Option<InlineFlowPlan>, S::Error> {
    let column = input.column;
    let Some(style) = styles.para_styles.get(para.para_shape_id as usize) else {
        return Ok(None);
    };
    let mut container = LayoutRect {
        x: column.x + style.margin_left,
        width: (column.width - style.margin_left - style.margin_right).max(0.0),
        ..column
    };
    // 정상 저장본의 일정한 줄 폭은 실제 사용한 가로 프레임이다. 용지 여백의
    // 환산 폭으로 넓히면 정확히 경계에 놓인 TAC가 앞 앵커와 같은 줄로 합쳐진다.
    // 줄 소속은 다시 계산하되 유효한 가로 프레임은 보존한다. 편집·합성 줄이나
    // 어울림으로 폭이 달라진 저장 줄은 이 단축의 근거가 아니다.
    if inline_flow::supports_page_square_table_rows(para)
        && !para.stored_text_partition_is_dirty()
        && !para.cell_format_vpos_dirty
    {
        if let Some(first) = para.line_segs.first().filter(|first| {
            first.segment_width > 0
                && first.column_start == 0
                && para.line_segs.iter().all(|line| {
                    line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                        && line.column_start == first.column_start
                        && line.segment_width == first.segment_width
                })
        }) {
            container.width = container
                .width
                .min(crate::renderer::hwpunit_to_px(first.segment_width, dpi));
        }
    }
    let paper = LayoutRect {
        x: 0.0,
        y: 0.0,
        width: input.page_width,
        height: input.page_height,
    };
    let mut exclusions: Vec<_> = if let Some(exclusions) = input.exclusions {
        storage.collect(exclusions.values().cloned())?
    } else {
        Vec::new()
    };
    if let Some(zones) = input.visible_float_exclusions {
        storage.extend(
            &mut exclusions,
            zones
                .iter()
                .filter(|zone| zone.para_index != para_index)
                .map(|zone| FrameExclusion {
                    horizontal: crate::renderer::px_to_hwpunit(column.x, dpi)
                        ..crate::renderer::px_to_hwpunit(column.x + column.width, dpi),
                    vertical: crate::renderer::px_to_hwpunit(column.y + zone.top, dpi)
                        ..crate::renderer::px_to_hwpunit(column.y + zone.bottom, dpi),
                    policy: FrameExclusionPolicy::BothSides,
                }),
        )?;
    }
    let frame = ObjectPlacementFrame {
        container: &container,
        column: &column,
        body: input.body,
        paper: &paper,
        paragraph_y: column.y + input.start,
        alignment: style.alignment,
        dpi,
    };
    // 쪽·종이 기준 표는 앵커 줄의 y와 독립적인 상자다. 같은 문단의
    // 줄 구성도 이 프레임의 배제 영역을 소비하며 paint는 아래 확정 좌표를 쓴다.
    for control in &para.controls {
        if let crate::model::control::Control::Table(table) = control {
            if let Some(exclusion) = frame.page_anchored_square_table_exclusion(table) {
                storage.push(&mut exclusions, exclusion)?;
            }
        }
    }
    let plan = if inline_flow::supports_plain_text(para) {
        inline_flow::plan_plain_text_with_storage(para, styles, &frame, &exclusions, storage)?
    } else {
        inline_flow::plan_with_storage(
            para,
            para_index,
            styles,
            tables,
            &frame,
            &exclusions,
            storage,
        )?
    };
    let Some(mut plan) = plan else {
        return Ok(None);
    };
    let Some(resolved) = inline_flow::floating_shapes::resolve_with_storage(
        plan,
        para,
        para_index,
        styles,
        tables,
        &frame,
        &exclusions,
        storage,
    )?
    else {
        return Ok(None);
    };
    plan = resolved;
    for item in &mut plan.boxes {
        let inline_flow::InlineFlowContent::FloatingTable {
            control,
            resolved_position,
        } = &mut item.content
        else {
            continue;
        };
        let crate::model::control::Control::Table(table) = &para.controls[*control] else {
            continue;
        };
        if frame.page_anchored_square_table_exclusion(table).is_some() {
            let margin = |value: i16| crate::renderer::hwpunit_to_px(i32::from(value), dpi);
            let width = crate::renderer::hwpunit_to_px(table.common.width as i32, dpi)
                + margin(table.outer_margin_left)
                + margin(table.outer_margin_right);
            let height = crate::renderer::hwpunit_to_px(table.common.height as i32, dpi)
                + margin(table.outer_margin_top)
                + margin(table.outer_margin_bottom);
            let (left, top) = frame.position(&table.common, width, height);
            *resolved_position = Some((
                left + margin(table.outer_margin_left),
                top + margin(table.outer_margin_top),
            ));
        }
    }
    plan.relative_to(column.x, column.y);
    Ok(Some(plan))
}
