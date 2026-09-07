//! 텍스트와 TAC 표의 공통 물리 줄 배치. 원본 LineSeg/문자 offset은 변경하지 않는다.
//!
//! 페이지 배치 소유자가 원점과 선행 제외 영역을 전달하고, 확정 결과의 높이와 좌표를
//! 함께 소비한다. 이 모듈은 RenderNode를 만들거나 문서 캐시를 수정하지 않는다.

use std::ops::Range;

use crate::model::{control::Control, paragraph::Paragraph, style::Alignment};

use super::{
    float_placement::ObjectPlacementFrame,
    height_measurer::MeasuredTable,
    hwpunit_to_px,
    layout::{estimate_text_width, resolved_to_text_style},
    layout_frame::{FrameExclusion, LayoutFrame},
    px_to_hwpunit,
    style_resolver::{detect_lang_category, ResolvedParaStyle, ResolvedStyleSet},
};

/// 문자 또는 표. 식별자는 paragraph.text의 scalar index / control index다.
#[derive(Debug, Clone, PartialEq)]
pub enum InlineFlowContent {
    Text {
        range: Range<usize>,
        style: u32,
        lang: usize,
    },
    Table {
        control: usize,
        margin_left: f64,
        margin_top: f64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineFlowBox {
    pub content: InlineFlowContent,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub baseline: f64,
}

/// 좌표와 높이는 같은 계산의 결과다. 확정 후 단 상대 좌표로 보관한다.
#[derive(Debug, Clone, PartialEq)]
pub struct InlineFlowPlan {
    pub start: f64,
    pub end: f64,
    pub boxes: Vec<InlineFlowBox>,
    /// 원래 가용 폭/높이에 실제로 간섭한 제외 영역이 있었는가.
    pub(crate) carved: bool,
    next_row_top: f64,
}

impl InlineFlowPlan {
    pub(crate) fn relative_to(&mut self, x: f64, y: f64) {
        self.start -= y;
        self.end -= y;
        self.next_row_top -= y;
        for item in &mut self.boxes {
            item.x -= x;
            item.y -= y;
        }
    }
}

enum Atom {
    Box(InlineFlowBox),
    Float(FrameExclusion),
    Break,
}

/// 현재 연결된 계약은 본문의 텍스트 / TAC 표 / 그림 / 구조 marker다.
/// 다른 소유자의 수식·각주·필드를 누락시키지 않고 연결 전까지 기존 경로로 반환한다.
pub(crate) fn supports(para: &Paragraph, width_hu: i32) -> bool {
    para.controls.iter().any(|c| {
        matches!(c, Control::Table(t)
        if super::height_measurer::is_tac_table_inline_in_para(t, width_hu, para))
    }) && para.controls.iter().all(|c| match c {
        Control::Table(t) => t.common.treat_as_char,
        Control::Picture(p) => !p.common.treat_as_char,
        Control::SectionDef(_)
        | Control::ColumnDef(_)
        | Control::PageNumberPos(_)
        | Control::PageNumCtrl(_) => true,
        _ => false,
    })
}

/// 호출자가 측정한 표 높이와 기존 text measurer의 advance를 사용한다.
pub(crate) fn plan(
    para: &Paragraph,
    para_index: usize,
    styles: &ResolvedStyleSet,
    tables: &[MeasuredTable],
    frame: &ObjectPlacementFrame<'_>,
    preceding: &[FrameExclusion],
) -> Option<InlineFlowPlan> {
    let style = styles.para_styles.get(para.para_shape_id as usize)?;
    let top = frame.paragraph_y + style.spacing_before;
    let positions = para.control_text_positions();
    let chars: Vec<_> = para.text.chars().collect();
    let mut controls: Vec<_> = positions.into_iter().enumerate().collect();
    controls.sort_by_key(|&(ci, position)| (position, ci));
    let mut controls = controls.into_iter().peekable();
    let mut atoms = Vec::new();
    for position in 0..=chars.len() {
        while controls.peek().is_some_and(|&(_, p)| p <= position) {
            let (ci, _) = controls.next()?;
            match &para.controls[ci] {
                Control::Table(table) => {
                    let measured = tables
                        .iter()
                        .find(|m| m.para_index == para_index && m.control_index == ci)?;
                    let left = hwpunit_to_px(i32::from(table.outer_margin_left), frame.dpi);
                    let right = hwpunit_to_px(i32::from(table.outer_margin_right), frame.dpi);
                    let top = hwpunit_to_px(i32::from(table.outer_margin_top), frame.dpi);
                    let bottom = hwpunit_to_px(i32::from(table.outer_margin_bottom), frame.dpi);
                    let height = measured.total_height + top + bottom;
                    atoms.push(Atom::Box(InlineFlowBox {
                        content: InlineFlowContent::Table {
                            control: ci,
                            margin_left: left,
                            margin_top: top,
                        },
                        x: 0.0,
                        y: 0.0,
                        width: hwpunit_to_px(table.common.width as i32, frame.dpi) + left + right,
                        height,
                        baseline: height,
                    }));
                }
                Control::Picture(picture) => {
                    if let Some(exclusion) = frame.picture_exclusion(picture) {
                        atoms.push(Atom::Float(exclusion));
                    }
                }
                _ => {}
            }
        }
        let Some(&ch) = chars.get(position) else {
            break;
        };
        if ch == '\n' || ch == '\r' {
            atoms.push(Atom::Break);
            continue;
        }
        let cs = super::composer::find_active_char_shape_visible(&para.char_shapes, position);
        let lang = detect_lang_category(ch);
        let text_style = resolved_to_text_style(styles, cs, lang);
        let width = estimate_text_width(&ch.to_string(), &text_style);
        let metrics = super::composer::frame_metrics_for_line(
            text_style.font_size,
            12.0,
            style.line_spacing_type,
            style.line_spacing,
            frame.dpi,
        );
        let height = hwpunit_to_px(metrics.line_height, frame.dpi);
        atoms.push(Atom::Box(InlineFlowBox {
            content: InlineFlowContent::Text {
                range: position..position + 1,
                style: cs,
                lang,
            },
            x: 0.0,
            y: 0.0,
            width,
            height,
            baseline: hwpunit_to_px(metrics.baseline_distance, frame.dpi),
        }));
    }
    let mut exclusions = preceding.to_vec();
    let horizontal = frame.container.x..frame.container.x + frame.container.width;
    let mut result = InlineFlowPlan {
        start: frame.paragraph_y,
        end: top,
        boxes: Vec::new(),
        carved: false,
        next_row_top: top,
    };
    let mut row = Vec::new();
    for atom in atoms {
        match atom {
            Atom::Float(exclusion) => {
                // 후행 anchor가 선행 텍스트/표를 소급 이동시키지 않는다.
                finish_row(
                    &mut result,
                    &mut row,
                    &horizontal,
                    &exclusions,
                    style,
                    frame.dpi,
                )?;
                exclusions.push(exclusion);
            }
            Atom::Break => {
                finish_row(
                    &mut result,
                    &mut row,
                    &horizontal,
                    &exclusions,
                    style,
                    frame.dpi,
                )?;
            }
            Atom::Box(item) => {
                if ![item.width, item.height, item.baseline]
                    .iter()
                    .all(|v| v.is_finite())
                    || item.width < 0.0
                    || item.height <= 0.0
                {
                    return None;
                }
                let width: f64 = row.iter().map(|b: &InlineFlowBox| b.width).sum();
                let moves_existing_row = if row.is_empty() {
                    false
                } else {
                    let old = row_geometry(
                        &row,
                        &horizontal,
                        result.next_row_top,
                        &exclusions,
                        style,
                        frame.dpi,
                    )?;
                    row.push(item.clone());
                    let next = row_geometry(
                        &row,
                        &horizontal,
                        result.next_row_top,
                        &exclusions,
                        style,
                        frame.dpi,
                    )?;
                    row.pop();
                    next.1 > old.1 + 0.01
                };
                if !row.is_empty()
                    && (width + item.width > frame.container.width + 0.01 || moves_existing_row)
                {
                    finish_row(
                        &mut result,
                        &mut row,
                        &horizontal,
                        &exclusions,
                        style,
                        frame.dpi,
                    )?;
                }
                row.push(item);
            }
        }
    }
    finish_row(
        &mut result,
        &mut row,
        &horizontal,
        &exclusions,
        style,
        frame.dpi,
    )?;
    result.end = result.end.max(result.next_row_top) + style.spacing_after;
    Some(result)
}

fn finish_row(
    plan: &mut InlineFlowPlan,
    row: &mut Vec<InlineFlowBox>,
    horizontal: &Range<f64>,
    exclusions: &[FrameExclusion],
    style: &ResolvedParaStyle,
    dpi: f64,
) -> Option<()> {
    if row.is_empty() {
        return Some(());
    }
    let baseline = row.iter().map(|b| b.baseline).fold(0.0, f64::max);
    let descent = row
        .iter()
        .map(|b| b.height - b.baseline)
        .fold(0.0, f64::max);
    let height = baseline + descent;
    let (mut x, y, carved) =
        row_geometry(row, horizontal, plan.next_row_top, exclusions, style, dpi)?;
    plan.carved |= carved;
    let has_table = row
        .iter()
        .any(|b| matches!(b.content, InlineFlowContent::Table { .. }));
    for mut item in row.drain(..) {
        item.x = x;
        item.y = y + baseline - item.baseline;
        x += item.width;
        plan.boxes.push(item);
    }
    let metrics = super::composer::frame_metrics_for_line(
        height,
        height,
        style.line_spacing_type,
        style.line_spacing,
        dpi,
    );
    let gap = hwpunit_to_px(metrics.line_spacing, dpi);
    // 텍스트의 sub-100% 간격은 유지하되, 표의 물리 하단을 후속 줄이 침범하지 않는다.
    plan.next_row_top = y + height + if has_table { gap.max(0.0) } else { gap };
    plan.end = plan.end.max(y + height);
    Some(())
}

fn row_geometry(
    row: &[InlineFlowBox],
    horizontal: &Range<f64>,
    top: f64,
    exclusions: &[FrameExclusion],
    style: &ResolvedParaStyle,
    dpi: f64,
) -> Option<(f64, f64, bool)> {
    let width: f64 = row.iter().map(|b| b.width).sum();
    let baseline = row.iter().map(|b| b.baseline).fold(0.0, f64::max);
    let descent = row
        .iter()
        .map(|b| b.height - b.baseline)
        .fold(0.0, f64::max);
    let height = baseline + descent;
    let base = px_to_hwpunit(horizontal.start, dpi)..px_to_hwpunit(horizontal.end, dpi);
    let base_width = base.end.checked_sub(base.start).filter(|w| *w > 0)?;
    let mut frame = LayoutFrame::new(base.clone(), px_to_hwpunit(top, dpi), exclusions.to_vec());
    frame.minimum_width = px_to_hwpunit(width, dpi).max(1).min(base_width);
    let intervals = frame.carve(px_to_hwpunit(height, dpi).max(1));
    let lane = if style.alignment == Alignment::Right {
        intervals.last()?
    } else {
        intervals.first()?
    }
    .clone();
    let left = hwpunit_to_px(lane.start, dpi);
    let spare = (hwpunit_to_px(lane.end - lane.start, dpi) - width).max(0.0);
    let x = left
        + match style.alignment {
            Alignment::Right => spare,
            Alignment::Center => spare / 2.0,
            _ => 0.0,
        };
    let y = hwpunit_to_px(frame.top, dpi).max(top);
    let carved = y > top + 0.01 || lane.start > base.start || lane.end < base.end;
    Some((x, y, carved))
}
