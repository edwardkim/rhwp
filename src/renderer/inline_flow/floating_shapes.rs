//! 재조판한 글줄의 부동 도형 앵커와 실제 배제 띠를 한 번 확정한다.

use super::{InlineFlowContent, InlineFlowPlan};
use crate::model::{
    control::Control,
    paragraph::Paragraph,
    shape::{TextWrap, VertAlign},
};
use crate::renderer::{
    float_placement::{ObjectPlacementFrame, ParagraphFloatFlow, ParagraphFloatPlacement},
    height_measurer::MeasuredTable,
    hwpunit_to_px,
    layout_frame::{FrameExclusion, FrameExclusionPolicy},
    px_to_hwpunit,
    style_resolver::ResolvedStyleSet,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FloatingShapePlacement {
    pub control: usize,
    pub placement: ParagraphFloatPlacement,
    /// 다음 문단도 같은 실제 단 좌표의 배제 영역을 소비한다.
    pub exclusion: Option<FrameExclusion>,
}

crate::storage::clone_fields!(FloatingShapePlacement, {control, placement, exclusion});

pub(super) fn supported(control: &Control) -> bool {
    let Control::Shape(shape) = control else {
        return false;
    };
    let c = shape.common();
    !c.treat_as_char
        && c.width > 0
        && c.width <= i32::MAX as u32
        && c.height > 0
        && c.height <= i32::MAX as u32
        && matches!(c.vert_align, VertAlign::Top | VertAlign::Inside)
        && matches!(
            c.text_wrap,
            TextWrap::TopAndBottom | TextWrap::BehindText | TextWrap::InFrontOfText
        )
        && matches!(
            shape.as_ref(),
            crate::model::shape::ShapeObject::Rectangle(_)
        )
        && crate::renderer::composer::shape_caption_for_layout(shape).is_none()
        && shape
            .drawing()
            .and_then(|drawing| drawing.text_box.as_ref())
            .is_none_or(|text| {
                text.paragraphs.iter().all(|para| {
                    para.controls
                        .iter()
                        .all(crate::renderer::composer::control_is_width_neutral_marker)
                })
            })
}

/// 도형의 앵커는 배제 전 글줄의 원시 control 슬롯 소유다. 그 띠 때문에 뒤로 밀린
/// 최종 글줄을 다시 앵커로 쓰면 도형과 글줄이 서로를 계속 밀어내므로 원래 앵커를 보존한다.
pub(crate) fn resolve(
    plan: InlineFlowPlan,
    para: &Paragraph,
    para_index: usize,
    styles: &ResolvedStyleSet,
    tables: &[MeasuredTable],
    frame: &ObjectPlacementFrame<'_>,
    preceding: &[FrameExclusion],
) -> Option<InlineFlowPlan> {
    match resolve_with_storage(
        plan,
        para,
        para_index,
        styles,
        tables,
        frame,
        preceding,
        &crate::storage::Unbounded,
    ) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn resolve_with_storage<S: crate::storage::Storage>(
    mut plan: InlineFlowPlan,
    para: &Paragraph,
    para_index: usize,
    styles: &ResolvedStyleSet,
    tables: &[MeasuredTable],
    frame: &ObjectPlacementFrame<'_>,
    preceding: &[FrameExclusion],
    storage: &S,
) -> Result<Option<InlineFlowPlan>, S::Error> {
    crate::storage::with_optional(|failure| {
        if !para.controls.iter().any(supported) {
            return Some(plan);
        }
        let mut placements = Vec::new();
        let mut exclusions =
            crate::storage::optional(storage.collect(preceding.iter().cloned()), failure)?;
        for item in &plan.boxes {
            let InlineFlowContent::FloatingShape { control } = item.content else {
                continue;
            };
            let Control::Shape(shape) = &para.controls[control] else {
                return None;
            };
            let c = shape.common();
            let m = &c.margin;
            let width = hwpunit_to_px(
                i32::try_from(i64::from(c.width) + i64::from(m.left) + i64::from(m.right)).ok()?,
                frame.dpi,
            );
            let height = hwpunit_to_px(
                i32::try_from(i64::from(c.height) + i64::from(m.top) + i64::from(m.bottom)).ok()?,
                frame.dpi,
            );
            let anchored = ObjectPlacementFrame {
                paragraph_y: item.y,
                ..*frame
            };
            let (left, top) = anchored.position(c, width, height);
            if ![left, top, width, height].iter().all(|n| n.is_finite()) {
                return None;
            }
            let exclusion = (c.text_wrap == TextWrap::TopAndBottom
                && left < frame.column.x + frame.column.width
                && left + width > frame.column.x)
                .then(|| FrameExclusion {
                    horizontal: px_to_hwpunit(frame.column.x, frame.dpi)
                        ..px_to_hwpunit(frame.column.x + frame.column.width, frame.dpi),
                    vertical: px_to_hwpunit(top, frame.dpi)..px_to_hwpunit(top + height, frame.dpi),
                    policy: FrameExclusionPolicy::BothSides,
                });
            if let Some(band) = &exclusion {
                crate::storage::optional(storage.push(&mut exclusions, band.clone()), failure)?;
            }
            crate::storage::optional(
                storage.push(
                    &mut placements,
                    FloatingShapePlacement {
                        control,
                        placement: ParagraphFloatPlacement {
                            flow: ParagraphFloatFlow::AbsolutePageFrame,
                            anchor_y: item.y,
                            table_left: Some(left + hwpunit_to_px(i32::from(m.left), frame.dpi)),
                            table_top: top + hwpunit_to_px(i32::from(m.top), frame.dpi),
                            occupied_bottom: top + height,
                            stored_host_origin: None,
                            stored_successor_line_origin: None,
                        },
                        exclusion,
                    },
                ),
                failure,
            )?;
        }
        if placements.is_empty() {
            return Some(plan);
        }
        if exclusions.len() > preceding.len() {
            plan = crate::storage::optional(
                super::plan_with_storage(
                    para,
                    para_index,
                    styles,
                    tables,
                    frame,
                    &exclusions,
                    storage,
                ),
                failure,
            )??;
        }
        plan.floating_shapes = placements;
        Some(plan)
    })
}

pub(super) fn anchor(control: usize) -> super::InlineFlowBox {
    super::InlineFlowBox {
        content: InlineFlowContent::FloatingShape { control },
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        baseline: 0.0,
    }
}

pub(super) fn relative_to(items: &mut [FloatingShapePlacement], y: f64) {
    for item in items {
        let p = &mut item.placement;
        p.anchor_y -= y;
        p.table_top -= y;
        p.occupied_bottom -= y;
        // AbsolutePageFrame의 가로 원점은 종이 절대 좌표다. 기존 도형
        // 소비자와 같이 세로 좌표만 단 상대 좌표로 전달한다.
    }
}
