//! 자동 Section의 제목·축·범례 예약과 실제 배치가 소비하는 공통 결과.
use super::legacy_legend_layout::LegendLayout;
use super::legacy_presentation::{Presentation, Section};
use super::legacy_text_metrics::measure;

pub(super) const SECTION_EXTERNAL_TWIP: f64 = 160.0;
pub(super) const SECTION_INTERNAL_TWIP: f64 = 150.0;
pub(super) const SECTION_PADDING_TWIP: f64 = 50.0;
const PLOT_PADDING_WITHOUT_AXES_TWIP: f64 = 100.0;

pub(super) struct ColumnLayout {
    pub plot: [f64; 4],
    pub title: Option<[f64; 2]>,
    pub legend: Option<LegendLayout>,
    pub value_width: f64,
    pub value_gap: f64,
    pub category_offset: f64,
    pub axes_visible: bool,
}
pub(super) fn inset(section: &Section) -> f64 {
    match section.backdrop.as_ref() {
        Some(b) if b.frame_enabled || b.fill_kind != 0 => {
            f64::from(section.internal) + if b.frame_enabled { b.frame_width } else { 0.0 }
        }
        _ => 40.0,
    }
}
impl ColumnLayout {
    pub fn new(p: &Presentation, width: f64, height: f64, dpi: f64) -> Option<Self> {
        let value = p.labels[1].as_ref()?;
        let category = p.labels[0].as_ref()?;
        let value_measure = measure(value, &value.font, dpi)?;
        let value_width = value_measure.width;
        let value_height = value_measure.height;
        let category_height = measure(category, &category.font, dpi)?.height;
        let unit = dpi / 1440.0;
        let frame = p
            .backdrop
            .as_ref()
            .filter(|b| b.frame_enabled)
            .map_or(0.0, |b| b.frame_width);
        let frame_pad = frame * unit;
        // Section 기본 여백은 160/150twip이며 제목은 내부 여백을 항상 예약한다.
        let external = SECTION_EXTERNAL_TWIP * unit;
        let title_internal = SECTION_INTERNAL_TWIP * unit;
        let mut reserved_top = frame_pad;
        let title = if let Some(text) = &p.title {
            let line_height = measure(text, &text.font, dpi)?.height;
            let block_height = line_height + SECTION_PADDING_TWIP * unit;
            let center = reserved_top + external + title_internal + block_height / 2.0;
            reserved_top += external + 2.0 * title_internal + block_height;
            Some([width / 2.0, center])
        } else {
            None
        };
        let legend = if let Some(font) = p.legend_font.as_ref() {
            Some(LegendLayout::new(
                p,
                font,
                width,
                height,
                dpi,
                inset(p.legend_section.as_ref()?),
            )?)
        } else {
            None
        };
        let margin = inset(&p.plot_section) * unit;
        let value_gap = -f64::from(value.metrics.paint_rect[2]) * unit;
        let category_gap = -f64::from(category.metrics.paint_rect[3]) * unit;
        if value_gap < 0.0 || category_gap < 0.0 {
            return None;
        }
        let left = frame_pad + external + margin + value_width + value_gap;
        let right = width - frame_pad - external - margin;
        let top = reserved_top + external + margin + value_height / 2.0;
        let bottom = if let Some(legend) = &legend {
            legend.section_top - category_height - category_gap - external - margin
        } else {
            height
                - frame_pad
                - external
                - margin
                - value_height / 2.0
                - category_height
                - category_gap
        };
        let axes_visible = (bottom - top) / f64::from(p.axes[0].divisions) >= value_height;
        // 축을 숨기면 글자·눈금의 점유 대신 플롯의 최소 내부 여유를 네 변에 예약한다.
        // 이 여유는 100twip이며, 비표시 글꼴·눈금 길이 변경에 따라 움직이지 않는다.
        let (left, top, right, bottom) = if axes_visible {
            (left, top, right, bottom)
        } else {
            let padding = PLOT_PADDING_WITHOUT_AXES_TWIP * unit;
            (
                left - value_width - value_gap + padding,
                top - value_height / 2.0 + padding,
                right - padding,
                bottom + category_height + category_gap - padding,
            )
        };
        if ![left, right, top, bottom].iter().all(|v| v.is_finite())
            || left >= right
            || top >= bottom
        {
            return None;
        }
        Some(Self {
            plot: [left, top, right, bottom],
            title,
            legend,
            value_width,
            value_gap,
            category_offset: category_gap + category_height / 2.0,
            axes_visible,
        })
    }
}
pub(super) fn tick_sides(style: u16, length: f64) -> Option<(f64, f64)> {
    match style {
        0 => Some((0.0, 0.0)),
        1 => Some((length / 2.0, length / 2.0)),
        2 => Some((length, 0.0)),
        3 => Some((0.0, length)),
        _ => None,
    }
}
