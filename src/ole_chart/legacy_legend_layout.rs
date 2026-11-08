//! 하단 범례의 실제 행 구성과 플롯 예약이 공유하는 점유 결과.

use super::legacy_column_layout::{
    SECTION_EXTERNAL_TWIP, SECTION_INTERNAL_TWIP, SECTION_PADDING_TWIP,
};
use super::legacy_presentation::{Font, Presentation};
use super::legacy_text_metrics::{cached_size, measure};

const TEXT_SLOT_INSET_CACHE_UNITS: f64 = 50.0;

pub(super) struct LegendLayout {
    pub x: f64,
    pub y: f64,
    pub section_top: f64,
    pub marker: f64,
    pub column_step: f64,
    pub row_step: f64,
    pub rows: usize,
    label_centers: Vec<f64>,
    text_units: Vec<f64>,
    physical_text_gap: f64,
}

impl LegendLayout {
    pub fn new(
        p: &Presentation,
        font: &Font,
        width: f64,
        height: f64,
        dpi: f64,
        inset: f64,
    ) -> Option<Self> {
        p.legend_section.as_ref()?;
        let unit = dpi / 1440.0;
        let mut marker: f64 = 0.0;
        let mut label_width: f64 = 0.0;
        let mut paint_centers = Vec::new();
        let mut fresh = Vec::new();
        let mut text_units = Vec::new();
        for series in &p.series {
            let label = series.legend.as_ref()?;
            if label.metrics.horizontal != 0
                || label.font.family != font.family
                || label.font.style != 0
                || label.font.effects != 0
            {
                return None;
            }
            let cached = cached_size(label)?;
            let measured = measure(label, font, dpi)?;
            let font_unit = measured.cache_unit;
            let refreshed = label.font != *font;
            let size = if refreshed {
                [measured.width, measured.height]
            } else {
                [cached[0] * unit, cached[1] * unit]
            };
            marker = marker.max(size[1]);
            label_width = label_width.max(size[0]);
            paint_centers.push(
                (f64::from(label.metrics.paint_rect[1]) + f64::from(label.metrics.paint_rect[3]))
                    * font_unit
                    / 2.0,
            );
            fresh.push(refreshed);
            text_units.push(font_unit);
        }
        for (center, refreshed) in paint_centers.iter_mut().zip(&fresh) {
            if *refreshed {
                *center = marker / 2.0;
            }
        }
        let column_width = marker * 1.5 + label_width;
        let gap = SECTION_INTERNAL_TWIP * unit;
        // 바깥여백은 글꼴과 같은 물리 twip 단위이며 저장 프레임의 배율을 적용하지 않는다.
        let frame = p
            .backdrop
            .as_ref()
            .filter(|b| b.frame_enabled)
            .map_or(0.0, |b| b.frame_width);
        // 행 구성 전에 root·Section 안쪽 여백과 오른쪽 패딩을 예약한다.
        let available_width =
            width - (2.0 * (frame + SECTION_EXTERNAL_TWIP + inset) + SECTION_PADDING_TWIP) * unit;
        let columns = ((available_width + gap) / (column_width + gap)).floor() as usize;
        if columns == 0 || p.series.is_empty() {
            return None;
        }
        let columns = columns.min(p.series.len());
        let rows = p.series.len().div_ceil(columns);
        let grid_width = columns as f64 * (column_width + gap) - gap;
        let minimum_row_step = marker * 1.5;
        let grid_height = rows as f64 * minimum_row_step - marker * 0.5;
        // 저장 Section의 원점과 물리 내부 여백을 구분한다. 범례는 새 외곽의 아래에 붙는다.
        let bottom = height - (frame + SECTION_EXTERNAL_TWIP + inset + SECTION_PADDING_TWIP) * unit;
        let interior_top = bottom - grid_height;
        let interior_height = grid_height + SECTION_PADDING_TWIP * unit;
        // 예약 상자에 붙인 50twip는 실제 셀 높이에 포함된다. 한 행이면
        // 마커는 그 셀 중앙에 놓이고, 여러 행이면 최소 행 높이를 지킨다.
        let minimum_cell_height = if rows == 1 { marker } else { minimum_row_step };
        let row_step = (interior_height / rows as f64).max(minimum_cell_height);
        let y = interior_top
            + if interior_height / rows as f64 >= minimum_cell_height {
                (interior_height / rows as f64 - marker) / 2.0
            } else {
                0.0
            };
        let section_top = interior_top - inset * unit;
        Some(Self {
            x: (width - grid_width - SECTION_PADDING_TWIP * unit) / 2.0,
            y,
            section_top,
            marker,
            column_step: column_width + gap,
            row_step,
            rows,
            label_centers: paint_centers,
            text_units,
            physical_text_gap: SECTION_PADDING_TWIP * unit,
        })
    }

    /// 범례는 열 안에서 위에서 아래로 채운 뒤 다음 열로 이어진다.
    pub fn origin(&self, index: usize) -> [f64; 2] {
        [
            self.x + (index / self.rows) as f64 * self.column_step,
            self.y + (index % self.rows) as f64 * self.row_step,
        ]
    }
    /// 마커 뒤 물리 간격과 TextBlock 슬롯 내부 여유는 서로 다른 좌표계다.
    /// 후자는 현재 선언 글꼴로 복원한 캐시 단위를 사용한다.
    pub fn label_origin(&self, index: usize) -> [f64; 2] {
        let [x, y] = self.origin(index);
        let row = index % self.rows;
        let text_top = -self.marker / 4.0;
        let text_bottom = if row == 0 {
            self.marker * 3.0 / 4.0
        } else {
            self.marker
        };
        [
            x + self.marker
                + self.physical_text_gap
                + TEXT_SLOT_INSET_CACHE_UNITS * self.text_units[index],
            y + text_top + (text_bottom - text_top - self.marker) / 2.0 + self.label_centers[index],
        ]
    }
}
