//! 하단 범례의 동일한 행 구성 결과를 공간 예약과 표시가 공유한다.

use std::ops::Range;

pub(crate) const TEXT_LINE_EM: f64 = 1.3;

pub(crate) struct LegendRow {
    pub items: Range<usize>,
    pub width: f64,
}

pub(crate) struct LegendLayout {
    pub widths: Vec<f64>,
    pub rows: Vec<LegendRow>,
    pub width: f64,
    pub row_height: f64,
    pub bottom_padding: f64,
}

impl LegendLayout {
    pub fn new(widths: Vec<f64>, available: f64, font_size: f64, frame_padding: f64) -> Self {
        let mut rows = Vec::new();
        let (mut start, mut width) = (0, 0.0);
        for (i, item_width) in widths.iter().enumerate() {
            if i > start && width + item_width > available {
                rows.push(LegendRow {
                    items: start..i,
                    width,
                });
                start = i;
                width = 0.0;
            }
            width += item_width;
        }
        if start < widths.len() {
            rows.push(LegendRow {
                items: start..widths.len(),
                width,
            });
        }
        let width = rows.iter().map(|row| row.width).fold(0.0, f64::max);
        Self {
            widths,
            rows,
            width,
            row_height: font_size * TEXT_LINE_EM,
            bottom_padding: frame_padding * 0.8,
        }
    }

    pub fn height(&self) -> f64 {
        if self.rows.is_empty() {
            0.0
        } else {
            self.rows.len() as f64 * self.row_height + self.bottom_padding
        }
    }
}
