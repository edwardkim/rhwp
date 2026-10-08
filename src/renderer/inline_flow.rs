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
    layout::{estimate_text_width, estimate_text_width_unrounded, resolved_to_text_style},
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
    /// Width-neutral object anchored to the physical text row.
    FloatingTable { control: usize },
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

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InlineFlowRow {
    pub boxes: Range<usize>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub baseline: f64,
}

/// 좌표와 높이는 같은 계산의 결과다. 확정 후 단 상대 좌표로 보관한다.
#[derive(Debug, Clone, PartialEq)]
pub struct InlineFlowPlan {
    /// NO_LS 본문의 확정 frame 행. 원본 IR은 바꾸지 않고 fit와 paint에 함께 전달한다.
    pub(crate) text_rows: Option<Vec<crate::model::paragraph::LineSeg>>,
    /// Square table owns the object; this plan owns only its host text rows.
    pub(crate) square_host_control: Option<usize>,
    pub(crate) text_spacing_before: Option<f64>,
    pub(crate) square_host_placement: Option<super::float_placement::ParagraphFloatPlacement>,
    /// Host-relative object geometry; survives after the host text has ended.
    pub(crate) square_host_exclusion: Option<FrameExclusion>,
    pub start: f64,
    pub end: f64,
    pub boxes: Vec<InlineFlowBox>,
    pub(crate) rows: Vec<InlineFlowRow>,
    /// 원래 가용 폭/높이에 실제로 간섭한 제외 영역이 있었는가.
    pub(crate) carved: bool,
    next_row_top: f64,
    fallback_font_size: f64,
}

impl InlineFlowPlan {
    pub(crate) fn relative_to(&mut self, x: f64, y: f64) {
        self.start -= y;
        self.end -= y;
        self.next_row_top -= y;
        if let Some(p) = &mut self.square_host_placement {
            p.anchor_y -= y;
            p.table_top -= y;
            p.occupied_bottom -= y;
            p.table_left = p.table_left.map(|left| left - x);
        }
        for item in &mut self.boxes {
            item.x -= x;
            item.y -= y;
        }
        for row in &mut self.rows {
            row.x -= x;
            row.y -= y;
        }
    }
}

enum Atom {
    Box(InlineFlowBox),
    Float(FrameExclusion),
    Break(InlineFlowBox),
}

#[derive(Clone, Copy, Default)]
struct RowMetrics {
    width: f64,
    baseline: f64,
    descent: f64,
}

impl RowMetrics {
    fn combined(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            baseline: self.baseline.max(other.baseline),
            descent: self.descent.max(other.descent),
        }
    }

    fn with(self, item: &InlineFlowBox) -> Self {
        Self {
            width: self.width + item.width,
            baseline: self.baseline.max(item.baseline),
            descent: self.descent.max(item.height - item.baseline),
        }
    }
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
    // A control-only carrier has no glyph offsets. Its logical object cursor
    // still advances (0, 1, ...), but those slots are not positions in `text`.
    // Keep control order in the atom stream; the shared row composer decides
    // whether their boxes fit together or advance to the next physical row.
    let positions = if para.text.is_empty() {
        vec![0; para.controls.len()]
    } else {
        para.control_text_positions()
    };
    let chars: Vec<_> = para.text.chars().collect();
    let mut controls: Vec<_> = positions.into_iter().enumerate().collect();
    controls.sort_by_key(|&(ci, position)| (position, ci));
    let mut controls = controls.into_iter().peekable();
    let mut atoms = Vec::new();
    for position in 0..=chars.len() {
        while controls.peek().is_some_and(|&(_, p)| p <= position) {
            let (ci, _) = controls.next()?;
            match &para.controls[ci] {
                Control::Table(table) if !table.common.treat_as_char => {
                    atoms.push(Atom::Box(InlineFlowBox {
                        content: InlineFlowContent::FloatingTable { control: ci },
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                        baseline: 0.0,
                    }));
                }
                Control::Table(table) => {
                    let measured = tables
                        .iter()
                        .find(|m| m.para_index == para_index && m.control_index == ci)?;
                    let left = hwpunit_to_px(i32::from(table.outer_margin_left), frame.dpi);
                    let right = hwpunit_to_px(i32::from(table.outer_margin_right), frame.dpi);
                    let top = hwpunit_to_px(i32::from(table.outer_margin_top), frame.dpi);
                    let bottom = hwpunit_to_px(i32::from(table.outer_margin_bottom), frame.dpi);
                    let height = measured.total_height + top + bottom;
                    let metrics = super::composer::frame_metrics_for_line(
                        height,
                        height,
                        style.line_spacing_type,
                        style.line_spacing,
                        frame.dpi,
                    );
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
                        baseline: hwpunit_to_px(metrics.baseline_distance, frame.dpi),
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
        // CharShapeRef and char_offsets share the original UTF-16 stream,
        // including the control slots removed from the visible text.
        let stream_position = para
            .char_offsets
            .get(position)
            .copied()
            .unwrap_or(position as u32);
        let cs = super::composer::find_active_char_shape(&para.char_shapes, stream_position);
        let lang = detect_lang_category(ch);
        let text_style = resolved_to_text_style(styles, cs, lang);
        let width = if ch == ' ' {
            estimate_text_width_unrounded(" ", &text_style)
        } else {
            estimate_text_width(&ch.to_string(), &text_style)
        };
        let metrics = super::composer::frame_metrics_for_line(
            text_style.font_size,
            12.0,
            style.line_spacing_type,
            style.line_spacing,
            frame.dpi,
        );
        let height = hwpunit_to_px(metrics.line_height, frame.dpi);
        let mut item = InlineFlowBox {
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
        };
        if ch == '\n' || ch == '\r' {
            item.content = InlineFlowContent::Text {
                range: position..position,
                style: cs,
                lang,
            };
            item.width = 0.0;
            atoms.push(Atom::Break(item));
        } else {
            atoms.push(Atom::Box(item));
        }
    }
    if controls.peek().is_some() {
        // 범위 밖 anchor를 누락시킨 부분 결과를 확정하지 않는다.
        return None;
    }
    // Reuse the composer's language/style word boundaries. A word that fits
    // a whole line moves together; an overlong word retains character fallback.
    let word_ranges = super::composer::text_word_ranges(para, styles);
    let mut words = word_ranges.iter().peekable();
    let mut word_metrics = std::collections::BTreeMap::new();
    let mut separator_spaces = std::collections::BTreeSet::new();
    let mut followed_by_flow = false;
    for atom in atoms.iter().rev() {
        match atom {
            Atom::Box(InlineFlowBox {
                content: InlineFlowContent::Text { range, .. },
                ..
            }) if !range.is_empty() && chars[range.clone()].iter().all(|ch| *ch == ' ') => {
                if followed_by_flow {
                    separator_spaces.insert(range.start);
                }
            }
            Atom::Box(InlineFlowBox {
                content: InlineFlowContent::Text { .. } | InlineFlowContent::Table { .. },
                ..
            }) => followed_by_flow = true,
            // A floating anchor owns its authored physical row. Spaces before
            // it cannot be consumed as separators before the next flow box.
            _ => followed_by_flow = false,
        }
    }
    let mut index = 0;
    while index < atoms.len() {
        let Atom::Box(first) = &atoms[index] else {
            index += 1;
            continue;
        };
        let InlineFlowContent::Text {
            range: first_range, ..
        } = &first.content
        else {
            index += 1;
            continue;
        };
        while words
            .peek()
            .is_some_and(|word| word.end <= first_range.start)
        {
            words.next();
        }
        let Some(word) = words
            .peek()
            .filter(|word| word.contains(&first_range.start))
        else {
            index += 1;
            continue;
        };
        let start = first_range.start;
        let mut metrics = RowMetrics::default();
        while let Some(Atom::Box(item)) = atoms.get(index) {
            let InlineFlowContent::Text { range, .. } = &item.content else {
                break;
            };
            if !word.contains(&range.start) {
                break;
            }
            metrics = metrics.with(item);
            index += 1;
        }
        word_metrics.insert(start, metrics);
    }
    let mut exclusions = preceding.to_vec();
    let horizontal = frame.container.x..frame.container.x + frame.container.width;
    let mut result = InlineFlowPlan {
        text_rows: None,
        square_host_control: None,
        text_spacing_before: None,
        square_host_placement: None,
        square_host_exclusion: None,
        start: frame.paragraph_y,
        end: top,
        boxes: Vec::new(),
        rows: Vec::new(),
        carved: false,
        next_row_top: top,
        fallback_font_size: para
            .char_shapes
            .first()
            .and_then(|r| styles.char_styles.get(r.char_shape_id as usize))
            .map_or(12.0, |s| s.font_size),
    };
    let mut row = Vec::new();
    let mut row_metrics = RowMetrics::default();
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
                    Some(&chars),
                )?;
                exclusions.push(exclusion);
                row_metrics = RowMetrics::default();
            }
            Atom::Break(empty_line) => {
                // Consecutive authored breaks reserve real lines even without ink.
                if row.is_empty() {
                    row.push(empty_line);
                }
                finish_row(
                    &mut result,
                    &mut row,
                    &horizontal,
                    &exclusions,
                    style,
                    frame.dpi,
                    Some(&chars),
                )?;
                row_metrics = RowMetrics::default();
            }
            Atom::Box(mut item) => {
                if ![item.width, item.height, item.baseline]
                    .iter()
                    .all(|v| v.is_finite())
                    || item.width < 0.0
                    || (item.height <= 0.0
                        && !matches!(item.content, InlineFlowContent::FloatingTable { .. }))
                {
                    return None;
                }
                let row_horizontal =
                    paragraph_row_horizontal(&horizontal, style, result.boxes.is_empty());
                let row_width = row_horizontal.end - row_horizontal.start;
                // A separator consumed by a soft wrap remains in the source
                // row. It must not create a whitespace-only row before the
                // next word/table. Space-only paragraphs and authored breaks
                // retain their real line boxes.
                if matches!(&item.content, InlineFlowContent::Text { range, .. }
                    if separator_spaces.contains(&range.start))
                    && row_metrics.width + item.width > row_width + 0.01
                    && row.iter().any(|existing| match &existing.content {
                        InlineFlowContent::Text { range, .. } => {
                            chars[range.clone()].iter().any(|ch| !ch.is_whitespace())
                        }
                        InlineFlowContent::Table { .. } => true,
                        InlineFlowContent::FloatingTable { .. } => false,
                    })
                {
                    item.width = 0.0;
                }
                let lookahead = match &item.content {
                    InlineFlowContent::Text { range, .. } => word_metrics
                        .get(&range.start)
                        .filter(|metrics| metrics.width <= row_width + 0.01)
                        .copied()
                        .unwrap_or_else(|| RowMetrics::default().with(&item)),
                    _ => RowMetrics::default().with(&item),
                };
                let moves_existing_row = if row.is_empty() {
                    false
                } else {
                    let old = row_geometry(
                        row_metrics,
                        &row_horizontal,
                        result.next_row_top,
                        &exclusions,
                        style,
                        frame.dpi,
                    )?;
                    let next = row_geometry(
                        row_metrics.combined(lookahead),
                        &row_horizontal,
                        result.next_row_top,
                        &exclusions,
                        style,
                        frame.dpi,
                    )?;
                    next.1 > old.1 + 0.01
                };
                if !row.is_empty()
                    && (row_metrics.width + lookahead.width > row_width + 0.01
                        || moves_existing_row)
                {
                    finish_row(
                        &mut result,
                        &mut row,
                        &horizontal,
                        &exclusions,
                        style,
                        frame.dpi,
                        Some(&chars),
                    )?;
                    row_metrics = RowMetrics::default();
                }
                row_metrics = row_metrics.with(&item);
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
        None,
    )?;
    result.end = result.end.max(result.next_row_top) + style.spacing_after;
    (result.start.is_finite() && result.end.is_finite()).then_some(result)
}

fn finish_row(
    plan: &mut InlineFlowPlan,
    row: &mut Vec<InlineFlowBox>,
    horizontal: &Range<f64>,
    exclusions: &[FrameExclusion],
    style: &ResolvedParaStyle,
    dpi: f64,
    trim_separator_spaces: Option<&[char]>,
) -> Option<()> {
    if row.is_empty() {
        return Some(());
    }
    let mut row_metrics = row
        .iter()
        .fold(RowMetrics::default(), |metrics, item| metrics.with(item));
    if row_metrics.baseline + row_metrics.descent == 0.0 {
        // A row containing only a floating anchor still has a paragraph line.
        let metrics = super::composer::frame_metrics_for_line(
            plan.fallback_font_size,
            plan.fallback_font_size,
            style.line_spacing_type,
            style.line_spacing,
            dpi,
        );
        row_metrics.baseline = hwpunit_to_px(metrics.baseline_distance, dpi);
        row_metrics.descent = hwpunit_to_px(metrics.line_height - metrics.baseline_distance, dpi);
    }
    let baseline = row_metrics.baseline;
    let height = row_metrics.baseline + row_metrics.descent;
    if let Some(chars) = trim_separator_spaces {
        // Spaces consumed before a soft wrap or authored break do not move
        // the centered/right-aligned ink. Paragraph-end spaces remain authored.
        row_metrics.width -= row
            .iter()
            .rev()
            .take_while(|item| {
                matches!(&item.content, InlineFlowContent::Text { range, .. }
                    if !range.is_empty() && chars[range.clone()].iter().all(|ch| *ch == ' '))
            })
            .map(|item| item.width)
            .sum::<f64>();
    }
    let row_horizontal = paragraph_row_horizontal(horizontal, style, plan.boxes.is_empty());
    let (mut x, y, carved) = row_geometry(
        row_metrics,
        &row_horizontal,
        plan.next_row_top,
        exclusions,
        style,
        dpi,
    )?;
    plan.carved |= carved;
    let has_table = row
        .iter()
        .any(|b| matches!(b.content, InlineFlowContent::Table { .. }));
    let row_font_size = row
        .iter()
        .filter(|item| matches!(item.content, InlineFlowContent::Text { .. }))
        .map(|item| item.height)
        .reduce(f64::max)
        .unwrap_or(plan.fallback_font_size);
    let first_box = plan.boxes.len();
    let row_x = x;
    for mut item in row.drain(..) {
        item.x = x;
        item.y = if matches!(item.content, InlineFlowContent::FloatingTable { .. }) {
            y
        } else {
            y + baseline - item.baseline
        };
        x += item.width;
        plan.boxes.push(item);
    }
    plan.rows.push(InlineFlowRow {
        boxes: first_box..plan.boxes.len(),
        x: row_x,
        y,
        width: row_metrics.width,
        height,
        baseline,
    });
    let metrics = super::composer::frame_metrics_for_line(
        row_font_size,
        plan.fallback_font_size,
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

fn paragraph_row_horizontal(
    horizontal: &Range<f64>,
    style: &ResolvedParaStyle,
    first: bool,
) -> Range<f64> {
    let indent = super::equation_tac_flow::paragraph_effective_margin_left(
        0.0,
        style.indent,
        usize::from(!first),
    );
    horizontal.start + indent..horizontal.end
}

fn row_geometry(
    metrics: RowMetrics,
    horizontal: &Range<f64>,
    top: f64,
    exclusions: &[FrameExclusion],
    style: &ResolvedParaStyle,
    dpi: f64,
) -> Option<(f64, f64, bool)> {
    let width = metrics.width;
    let height = metrics.baseline + metrics.descent;
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

/// 저장 행과 자체 inline 소유자가 없는 본문만 현재 물리 frame에서 재조판한다.
///
/// [#7548] 저장 LineSeg 가 없는 HWPX 문단은 로드 시 합성 줄(구현 태그)을 받는다.
/// 합성 줄은 앞 개체의 배제 영역을 모르는 전폭 재조판 결과라 저장 증거가 아니다 —
/// 저장 줄이 하나도 없으면 빈 줄 목록과 같이 이 frame 에서 다시 조판한다.
pub(crate) fn supports_plain_text(para: &Paragraph) -> bool {
    super::para_has_no_stored_line_segs(para)
        && !para.text.trim().is_empty()
        && para.controls.is_empty()
}

pub(crate) fn plan_plain_text(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    placement: &ObjectPlacementFrame<'_>,
    preceding: &[FrameExclusion],
) -> Option<InlineFlowPlan> {
    use super::composer::{layout_paragraph_in_frame, ParagraphBox};
    let style = styles.para_styles.get(para.para_shape_id as usize)?;
    if !supports_plain_text(para) || style.head_type != crate::model::style::HeadType::None {
        return None;
    }
    let mut exclusions = preceding.to_vec();
    let text_top = placement.paragraph_y + style.spacing_before;
    let dx = px_to_hwpunit(placement.column.x, placement.dpi);
    let dy = px_to_hwpunit(text_top, placement.dpi);
    for e in &mut exclusions {
        e.horizontal = e.horizontal.start.saturating_sub(dx)..e.horizontal.end.saturating_sub(dx);
        e.vertical = e.vertical.start.saturating_sub(dy)..e.vertical.end.saturating_sub(dy);
    }
    let paragraph_box =
        ParagraphBox::body_for_style(placement.column.width, Some(style), placement.dpi);
    if !paragraph_box.is_usable() {
        return None;
    }
    let mut frame = paragraph_box.frame_with(0, exclusions);
    let rows = layout_paragraph_in_frame(para, &mut frame, styles, placement.dpi)?;
    // 배제 없는 frame과 행이 같으면 기존 owner를 유지한다.
    let mut clear_frame = paragraph_box.frame(0);
    let clear_rows = layout_paragraph_in_frame(para, &mut clear_frame, styles, placement.dpi)?;
    let same_rows = rows.len() == clear_rows.len()
        && rows.iter().zip(&clear_rows).all(|(a, b)| {
            a.text_start == b.text_start
                && a.vertical_pos == b.vertical_pos
                && a.column_start == b.column_start
                && a.segment_width == b.segment_width
        });
    let end = text_top + hwpunit_to_px(frame.top, placement.dpi) + style.spacing_after;
    Some(InlineFlowPlan {
        start: placement.paragraph_y,
        end,
        boxes: Vec::new(),
        rows: Vec::new(),
        carved: !same_rows,
        next_row_top: end,
        fallback_font_size: 12.0,
        text_rows: Some(rows),
        square_host_control: None,
        text_spacing_before: None,
        square_host_placement: None,
        square_host_exclusion: None,
    })
}

/// Resolve the text beside a single Square table once, before fit and paint.
/// Inline controls and other object owners keep their existing transaction.
pub(crate) fn plan_square_table_host(
    para: &Paragraph,
    para_index: usize,
    styles: &ResolvedStyleSet,
    tables: &[MeasuredTable],
    column_width: f64,
    dpi: f64,
) -> Option<InlineFlowPlan> {
    use super::float_placement::{signed_hwpunit, ParagraphFloatFlow, ParagraphFloatPlacement};
    use super::layout_frame::FrameExclusionPolicy;
    use crate::model::shape::{HorzAlign, HorzRelTo, TextFlow, VertAlign, VertRelTo};
    if !super::is_no_lineseg_visible_text_host(para) {
        return None;
    }
    let control = para.controls.iter().position(|c| {
        matches!(c, Control::Table(t)
        if !t.common.treat_as_char && t.common.text_wrap == crate::model::shape::TextWrap::Square)
    })?;
    if para.controls.iter().enumerate().any(|(i, c)| {
        i != control
            && !super::composer::control_is_width_neutral_marker(c)
            && !matches!(c, Control::Picture(p) if !p.common.treat_as_char
                    && matches!(p.common.text_wrap,
                        crate::model::shape::TextWrap::InFrontOfText
                            | crate::model::shape::TextWrap::BehindText))
    }) {
        return None;
    }
    let style = styles.para_styles.get(para.para_shape_id as usize)?;
    // The floating table has its own paint owner and contributes no inline token.
    // Keep text/shape offsets intact; only the text frame is projected here.
    let mut text = para.clone();
    // Overlay pictures keep their separate Shape paint owners. They neither
    // consume inline width nor carve the text frame; preserve source offsets.
    text.controls
        .retain(super::composer::control_is_width_neutral_marker);
    text.line_segs.clear();
    let Control::Table(table) = &para.controls[control] else {
        return None;
    };
    let measured = tables
        .iter()
        .find(|m| m.para_index == para_index && m.control_index == control)?;
    let c = &table.common;
    if c.vert_rel_to != VertRelTo::Para
        || !matches!(c.vert_align, VertAlign::Top | VertAlign::Inside)
    {
        return None;
    }
    // Side captions own a second horizontal box; retain their existing owner.
    if table.caption.as_ref().is_some_and(|cap| {
        matches!(
            cap.direction,
            crate::model::shape::CaptionDirection::Left
                | crate::model::shape::CaptionDirection::Right
        )
    }) {
        return None;
    }
    let (ref_left, ref_width) = match c.horz_rel_to {
        HorzRelTo::Column => (0.0, column_width),
        HorzRelTo::Para => (
            style.margin_left,
            column_width - style.margin_left - style.margin_right,
        ),
        _ => return None,
    };
    let ml = hwpunit_to_px(i32::from(table.outer_margin_left), dpi);
    let mr = hwpunit_to_px(i32::from(table.outer_margin_right), dpi);
    let object_width = hwpunit_to_px(c.width as i32, dpi) + ml + mr;
    let offset = hwpunit_to_px(signed_hwpunit(c.horizontal_offset), dpi);
    let outer_left = match c.horz_align {
        HorzAlign::Left | HorzAlign::Inside => ref_left + offset,
        HorzAlign::Center => ref_left + (ref_width - object_width) / 2.0 + offset,
        HorzAlign::Right | HorzAlign::Outside => ref_left + ref_width - object_width - offset,
    };
    let outer_top = hwpunit_to_px(signed_hwpunit(c.vertical_offset), dpi);
    let table_top = outer_top + hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
    let occupied_bottom = table_top
        + measured.total_height
        + hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
    let placement = ParagraphFloatPlacement {
        flow: ParagraphFloatFlow::Exclusion,
        anchor_y: 0.0,
        stored_host_origin: None,
        stored_successor_line_origin: None,
        table_left: Some(outer_left + ml),
        table_top,
        occupied_bottom,
    };
    let spacing_before =
        if para.line_segs.is_empty() && std::env::var("RHWP_EXP_BODY_FRESH").is_err() {
            0.0
        } else {
            style.spacing_before
        };
    let exclusion = FrameExclusion {
        horizontal: px_to_hwpunit(outer_left, dpi)..px_to_hwpunit(outer_left + object_width, dpi),
        vertical: px_to_hwpunit(outer_top, dpi)..px_to_hwpunit(occupied_bottom, dpi),
        policy: match c.text_flow {
            TextFlow::BothSides => FrameExclusionPolicy::BothSides,
            TextFlow::LargestOnly => FrameExclusionPolicy::LargestSide,
            TextFlow::LeftOnly => FrameExclusionPolicy::LeftSide,
            TextFlow::RightOnly => FrameExclusionPolicy::RightSide,
        },
    };
    let box_ = super::composer::ParagraphBox::body_for_style(column_width, Some(style), dpi);
    let mut text_exclusion = exclusion.clone();
    let spacing_hu = px_to_hwpunit(spacing_before, dpi);
    text_exclusion.vertical.start -= spacing_hu;
    text_exclusion.vertical.end -= spacing_hu;
    let mut frame = box_.frame_with(0, vec![text_exclusion]);
    let rows = super::composer::layout_paragraph_in_frame(&text, &mut frame, styles, dpi)?;
    // Following paragraphs can use the free side of the same object. The
    // object remains an exclusion and fit budget; it is not a text advance.
    let end = spacing_before + hwpunit_to_px(frame.top, dpi) + style.spacing_after;
    Some(InlineFlowPlan {
        text_rows: Some(rows),
        square_host_control: Some(control),
        text_spacing_before: Some(spacing_before),
        square_host_placement: Some(placement),
        square_host_exclusion: Some(exclusion),
        start: 0.0,
        end,
        boxes: Vec::new(),
        rows: Vec::new(),
        carved: true,
        next_row_top: end,
        fallback_font_size: 12.0,
    })
}

/// Reflowed TAC rows also own the text preceding, separating and following
/// their objects. Stored rows and controls with other owners retain that path.
pub(crate) fn supports_table_text_rows(para: &Paragraph) -> bool {
    // Keep a standalone multi-row RowBreak table with its fragment/host owner,
    // including inputs without a LineSeg. A one-row table has no inter-row
    // cut: reuse its existing owner only when composition already supplies
    // the current object row, otherwise build the physical row here.
    let flow_objects = para
        .controls
        .iter()
        .filter(|control| matches!(control, Control::Table(_)))
        .count();
    let has_text = super::composer::expand_pua_display_text(&para.text)
        .chars()
        .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace());
    let has_standalone_fragment_owner = flow_objects == 1
        && !has_text
        && para.controls.iter().any(|control| {
            matches!(control, Control::Table(table)
                if table.common.treat_as_char
                && table.row_count > 1
                && table.page_break == crate::model::table::TablePageBreak::RowBreak)
        });
    let has_composed_object_row = flow_objects == 1
        && !has_text
        && para.line_segs.len() == 1
        && para.line_segs[0].tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY
            != 0
        && para.controls.iter().enumerate().any(|(control, _)| {
            super::composer::owned_rowbreak_tac_height(para, control).is_some()
        });
    super::para_has_no_stored_line_segs(para)
        && !has_standalone_fragment_owner
        && !has_composed_object_row
        && !para.text.contains('\t')
        && para
            .controls
            .iter()
            .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
        && para.controls.iter().all(|c| match c {
            Control::Table(t) => {
                // Captions and cell notes have their own placement/reservation owner.
                t.caption.is_none()
                    && !t.cells.iter().any(|cell| {
                        cell.paragraphs.iter().any(|p| {
                            p.controls
                                .iter()
                                .any(|c| matches!(c, Control::Footnote(_) | Control::Table(_)))
                        })
                    })
                    && (t.common.treat_as_char
                        || (matches!(
                            t.common.text_wrap,
                            crate::model::shape::TextWrap::InFrontOfText
                                | crate::model::shape::TextWrap::BehindText
                        ) && matches!(
                            t.common.vert_rel_to,
                            crate::model::shape::VertRelTo::Para
                        ) && matches!(
                            t.common.vert_align,
                            crate::model::shape::VertAlign::Top
                                | crate::model::shape::VertAlign::Inside
                        ) && matches!(
                            t.common.horz_rel_to,
                            crate::model::shape::HorzRelTo::Column
                                | crate::model::shape::HorzRelTo::Para
                        )))
            }
            c => super::composer::control_is_width_neutral_marker(c),
        })
}
