//! 텍스트와 TAC 표의 공통 물리 줄 배치. 원본 LineSeg/문자 offset은 변경하지 않는다.
//!
//! 페이지 배치 소유자가 원점과 선행 제외 영역을 전달하고, 확정 결과의 높이와 좌표를
//! 함께 소비한다. 이 모듈은 RenderNode를 만들거나 문서 캐시를 수정하지 않는다.

use std::ops::Range;

mod picture_rows;
pub(crate) use picture_rows::supports_picture_rows;
mod separator_spaces;
mod table_rows;
pub(crate) use table_rows::{standalone_table_fragment_owner, supports_table_text_rows};

mod stored_host;
mod stored_square_rows;
pub(crate) use stored_host::{plan_stored_body_wide_host, plan_stored_empty_square_host};

use crate::model::{control::Control, paragraph::Paragraph, style::Alignment};

use super::{
    float_placement::ObjectPlacementFrame,
    height_measurer::MeasuredTable,
    hwpunit_to_px,
    layout::{estimate_text_width_unrounded, resolved_to_text_style},
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
        /// 줄 안 정렬점과 실제 글꼴 기준선을 구분한다.
        alignment_baseline: Option<f64>,
    },
    Table {
        control: usize,
        margin_left: f64,
        margin_top: f64,
    },
    /// Width-neutral object anchored to the physical text row.
    FloatingTable { control: usize },
    /// 글자처럼 그림의 줄 상자와 실제 잉크 사이 바깥 여백.
    Picture {
        control: usize,
        margin_left: f64,
        margin_top: f64,
        affect_line_spacing: bool,
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

impl InlineFlowBox {
    fn alignment_baseline(&self) -> f64 {
        match self.content {
            InlineFlowContent::Text {
                alignment_baseline: Some(value),
                ..
            } => value,
            _ => self.baseline,
        }
    }
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
    /// 정상 저장 증언으로 확정한 빈 부동 개체 호스트의 실제 줄 예약.
    pub(crate) stored_empty_host: bool,
    /// 절대 전폭 개체의 예약 뒤에서 시작하는 첫 저장 부분 문단.
    pub(crate) stored_float_host: bool,
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
    /// 확정한 행의 물리 하단. 다음 줄 전진과 문단 뒤 여백을 구분한다.
    pub(crate) fn occupied_end(&self) -> f64 {
        self.rows
            .iter()
            .map(|row| row.y + row.height)
            .fold(self.start, f64::max)
    }

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
        let baseline = item.alignment_baseline();
        Self {
            width: self.width + item.width,
            baseline: self.baseline.max(baseline),
            descent: self.descent.max(item.height - baseline),
        }
    }
}

/// 현재 연결된 계약은 본문의 텍스트 / TAC 표 / 그림 / 구조 marker다.
/// 다른 소유자의 수식·각주·필드를 누락시키지 않고 연결 전까지 기존 경로로 반환한다.
pub(crate) fn supports(para: &Paragraph, width_hu: i32) -> bool {
    if supports_picture_rows(para) {
        return true;
    }
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
    let picture_rows = supports_picture_rows(para);
    let top = frame.paragraph_y + style.spacing_before;
    // 가시 글자가 없는 문단의 개체 순번은 문자 위치가 아니다. 각 개체의
    // 순서를 유지해 토큰으로 넣고, 공통 줄 구성 결과가 같은 줄 수용 여부와 줄 소유를 결정한다.
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
                    if picture.common.treat_as_char {
                        atoms.push(Atom::Box(picture_rows::picture_box(
                            picture, ci, style, frame.dpi,
                        )));
                    } else if let Some(exclusion) =
                        frame.picture_exclusion(picture, para, ci, styles)
                    {
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
        // 같은 글리프 전진량을 줄 수용과 실제 box 원점에 보존한다.
        // 문자마다 정수 px로 반올림하면 긴 prefix의 줄 경계가 달라진다.
        let width = estimate_text_width_unrounded(&ch.to_string(), &text_style);
        // 표시 relSz는 글리프 크기다. 새 그림 글줄도 일반 문단과 같이
        // 선언 글자 크기를 줄 상자·간격에 사용하고 표시 크기는 style에 보존한다.
        let line_font_size = if picture_rows {
            styles
                .char_styles
                .get(cs as usize)
                .map_or(text_style.font_size, |s| s.font_size)
        } else {
            text_style.font_size
        };
        let metrics = super::composer::frame_metrics_for_line(
            line_font_size,
            12.0,
            style.line_spacing_type,
            style.line_spacing,
            frame.dpi,
        );
        let height = hwpunit_to_px(metrics.line_height, frame.dpi);
        let alignment_baseline = (picture_rows && style.vertical_alignment != 0).then(|| {
            height * super::composer::inline_picture_ascent_ratio(style.vertical_alignment)
        });
        let mut item = InlineFlowBox {
            content: InlineFlowContent::Text {
                range: position..position + 1,
                style: cs,
                lang,
                alignment_baseline,
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
                alignment_baseline,
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
    let mut attached_word_metrics = std::collections::BTreeMap::new();
    let spaces = separator_spaces::classify(&atoms, &chars);
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
        if let Some(Atom::Box(next)) = atoms.get(index) {
            if matches!(next.content, InlineFlowContent::Table { .. }) {
                // 구분 공백 없이 붙은 TAC는 앞 어절의 수용 후보에 속한다.
                // 후보가 한 줄보다 크면 새 줄에서 개체 앞 경계를 분리한다.
                attached_word_metrics.insert(start, metrics.with(next));
            }
        }
    }
    let mut exclusions = preceding.to_vec();
    let horizontal = frame.container.x..frame.container.x + frame.container.width;
    let mut result = InlineFlowPlan {
        stored_empty_host: false,
        stored_float_host: false,
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
                // 작성 개행의 글자 크기도 현재 줄의 기준선·높이를 소유한다.
                // 폭 0 상자로 빈 줄과 연속 개행의 공간도 같은 결과에 보존한다.
                row.push(empty_line);
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
                // 개체 앞 작성 공백 그룹의 첫 공백만 우단 밖으로 전진한다.
                // 그룹의 후속 공백이 처음 넘쳤다는 이유로 hanging하지 않는다.
                let object_separator = matches!(&item.content, InlineFlowContent::Text { range, .. }
                    if spaces.object.contains(&range.start));
                let first_overhang_space = matches!(&item.content, InlineFlowContent::Text { range, .. }
                    if spaces.first_object.contains(&range.start))
                    && separator_spaces::has_flow_content(&row, &chars)
                    && row_metrics.width <= row_width + 0.01
                    && row_metrics.width + item.width > row_width + 0.01;
                // A separator consumed by a soft wrap remains in the source
                // row. It must not create a whitespace-only row before the
                // next word/table. Space-only paragraphs and authored breaks
                // retain their real line boxes.
                if matches!(&item.content, InlineFlowContent::Text { range, .. }
                    if spaces.wrap.contains(&range.start))
                    && row_metrics.width + item.width > row_width + 0.01
                    && !first_overhang_space
                    && separator_spaces::has_flow_content(&row, &chars)
                {
                    item.width = 0.0;
                    // 개체 앞의 넘친 공백은 앞 줄에서 소비하고 그 줄을 끝낸다.
                    // 이후 작성된 공백은 다음 물리 줄에 남아 개체의 전진량과
                    // 함께 fit한다. 모두 폭 0으로 흡수하면 표 소유 줄을 잃는다.
                    if object_separator {
                        row.push(item);
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
                        continue;
                    }
                }
                let lookahead = match &item.content {
                    InlineFlowContent::Text { range, .. } => attached_word_metrics
                        .get(&range.start)
                        .copied()
                        .or_else(|| {
                            word_metrics
                                .get(&range.start)
                                .filter(|metrics| metrics.width <= row_width + 0.01)
                                .copied()
                        })
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
                    && ((row_metrics.width + lookahead.width > row_width + 0.01
                        && !first_overhang_space)
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
        // 개행 메트릭 상자는 폭 0이며 후행 공백 조회를 막지 않는다.
        // 줄 경계에서 소비한 공백은 가운데·오른쪽 잉크 정렬을 밀지 않는다.
        row_metrics.width -= row
            .iter()
            .rev()
            .take_while(|item| {
                matches!(&item.content, InlineFlowContent::Text { range, .. }
                    if chars[range.clone()].iter().all(|ch| *ch == ' '))
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
        .filter(|item| {
            matches!(
                item.content,
                InlineFlowContent::Text { .. }
                    | InlineFlowContent::Picture {
                        affect_line_spacing: true,
                        ..
                    }
            )
        })
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
            y + baseline - item.alignment_baseline()
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
        stored_empty_host: false,
        stored_float_host: false,
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

/// 실제 단일 어울림 표의 배제 영역으로 호스트 줄을 fit·paint 전에 한 번 확정한다.
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
    let uncached = super::is_no_lineseg_visible_text_host(para);
    if !uncached
        && (!para
            .text
            .chars()
            .any(|ch| ch > '\u{001F}' && ch != '\u{FFFC}' && !ch.is_whitespace())
            || styles.hwp3_variant
            || super::composer::stored_rows_cross_physical_frame(para))
    {
        // 하나의 배제 프레임은 다른 쪽/단에 속한 저장 줄을 재구성하지 않는다.
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
    let box_ = super::composer::ParagraphBox::body_for_style(column_width, Some(style), dpi);
    let stored_rows = stored_square_rows::source(para, control, table, measured, &box_, dpi);
    let spacing_before =
        if para.line_segs.is_empty() && std::env::var("RHWP_EXP_BODY_FRESH").is_err() {
            0.0
        } else {
            style.spacing_before
        };
    let geometry = |anchor_offset_hwp| {
        let outer_top = hwpunit_to_px(
            signed_hwpunit(c.vertical_offset).saturating_add(anchor_offset_hwp),
            dpi,
        );
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
        let exclusion = FrameExclusion {
            horizontal: px_to_hwpunit(outer_left, dpi)
                ..px_to_hwpunit(outer_left + object_width, dpi),
            vertical: px_to_hwpunit(outer_top, dpi)..px_to_hwpunit(occupied_bottom, dpi),
            policy: match c.text_flow {
                TextFlow::BothSides => FrameExclusionPolicy::BothSides,
                TextFlow::LargestOnly => FrameExclusionPolicy::LargestSide,
                TextFlow::LeftOnly => FrameExclusionPolicy::LeftSide,
                TextFlow::RightOnly => FrameExclusionPolicy::RightSide,
            },
        };
        let mut text_exclusion = exclusion.clone();
        let spacing_hu = px_to_hwpunit(spacing_before, dpi);
        text_exclusion.vertical.start -= spacing_hu;
        text_exclusion.vertical.end -= spacing_hu;
        (
            placement,
            exclusion,
            box_.frame_with(0, vec![text_exclusion]),
        )
    };
    let (mut placement, mut exclusion, mut frame) = geometry(
        stored_rows
            .as_ref()
            .map_or(0, |source| source.anchor_offset_hwp),
    );
    let projected = stored_rows
        .as_ref()
        .and_then(|source| stored_square_rows::project(para, source, &mut frame));
    let uses_stored_rows = projected.is_some();
    let rows = if let Some(rows) = projected {
        rows
    } else {
        (placement, exclusion, frame) = geometry(0);
        super::composer::layout_paragraph_in_frame(&text, &mut frame, styles, dpi)?
    };
    if !uses_stored_rows
        && !uncached
        && rows.len() == para.line_segs.len()
        && rows.iter().zip(&para.line_segs).all(|(row, source)| {
            row.column_start == source.column_start && row.segment_width == source.segment_width
        })
    {
        // 같은 물리 구간의 저장 줄은 기존 소유자에게 남긴다.
        return None;
    }
    // Following paragraphs can use the free side of the same object. The
    // object remains an exclusion and fit budget; it is not a text advance.
    let end = spacing_before + hwpunit_to_px(frame.top, dpi) + style.spacing_after;
    Some(InlineFlowPlan {
        stored_empty_host: false,
        stored_float_host: false,
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
