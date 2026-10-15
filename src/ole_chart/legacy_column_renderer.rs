//! 저장된 소유 객체의 여백·레이블 점유·선 속성을 소비하는 2D 막대 차트.

use std::fmt::Write;

use super::legacy_presentation::{Backdrop, Font, Pen, Presentation, Section, Text};
use super::svg_renderer::{escape_xml_text, font_attributes, format_axis_value, series_fill};
use super::OleChart;

fn color(value: u32) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        value & 255,
        (value >> 8) & 255,
        (value >> 16) & 255
    )
}

fn plain(font: &Font) -> bool {
    font.style == 0 && font.effects == 0
}

fn supported(backdrop: Option<&Backdrop>) -> bool {
    backdrop.is_none_or(|b| !b.shadow_enabled && matches!(b.fill_kind, 0 | 1))
}

/// 프레임/채움이 없는 Section은 저장 InternalMargin 대신 40twip의
/// 기본 내부 여백을 쓴다(GetInteriorRect의 가상 함수 경로).
fn inset(section: &Section) -> f64 {
    match section.backdrop.as_ref() {
        Some(b) if b.frame_enabled || b.fill_kind != 0 => {
            f64::from(section.internal) + if b.frame_enabled { b.frame_width } else { 0.0 }
        }
        _ => 40.0,
    }
}

fn text_size(text: &Text) -> Option<[f64; 2]> {
    let m = &text.metrics;
    if !m.cache_valid
        || m.extent.iter().any(|v| *v < 0)
        || m.margin < 0
        || m.orientation != 0
        || m.horizontal > 2
        || m.vertical != 2
    {
        return None;
    }
    let width = f64::from(m.rect[2]) - f64::from(m.rect[0]);
    let height = f64::from(m.rect[3]) - f64::from(m.rect[1]);
    (width > 0.0 && height > 0.0).then_some([width, height])
}

fn line(svg: &mut String, bounds: [f64; 4], pen: Pen, unit: f64) {
    if !pen.enabled {
        return;
    }
    let [x1, y1, x2, y2] = bounds;
    let _ = write!(svg, "<path d=\"M{x1:.4} {y1:.4}L{x2:.4} {y2:.4}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.4}\"/>", color(pen.color), pen.width * unit);
}

/// 저장 조판의 Section/레이블 상자는 같은 root 좌표 변환을 소비한다.
/// 글꼴·펜의 물리 크기는 별도의 포인트/DPI 계약으로 출력한다.
pub(super) fn render(
    chart: &OleChart,
    p: &Presentation,
    width: f64,
    height: f64,
    dpi: f64,
) -> Option<String> {
    if p.chart_type != 1
        || ![width, height, dpi]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
        || chart.categories.is_empty()
        || chart.series.is_empty()
        || !supported(p.backdrop.as_ref())
        || !supported(p.plot_section.backdrop.as_ref())
        || !p.plot_section.visible
        || p.plot_section.location != 1
        || p.axis_styles.iter().any(|s| s.minor_grid.enabled)
    {
        return None;
    }
    let unit = dpi / 1440.0;
    let sx = width / (p.extent[2] - p.extent[0]);
    let sy = height / (p.extent[3] - p.extent[1]);
    let map_x = |x: f64| (x - p.extent[0]) * sx;
    let map_y = |y: f64| (p.extent[3] - y) * sy;
    let mut title_origin = None;
    if let Some(title) = p.title.as_ref() {
        let section = p.title_section.as_ref()?;
        if !plain(&title.font)
            || title.metrics.horizontal != 2
            || section.location != 1
            || !supported(section.backdrop.as_ref())
        {
            return None;
        }
        text_size(title)?;
        let rect = title.metrics.rect.map(f64::from);
        // 제목 TextBlock의 저장 위치를 root 좌표에서 변환한다.
        title_origin = Some([
            map_x((rect[0] + rect[2]) / 2.0),
            map_y((rect[1] + rect[3]) / 2.0),
        ]);
    }

    let mut legend_layout = None;
    if let Some(font) = p.legend_font.as_ref() {
        let section = p.legend_section.as_ref()?;
        if !plain(font) || section.location != 6 || !supported(section.backdrop.as_ref()) {
            return None;
        }
        let mut marker: f64 = 0.0;
        let mut label_width: f64 = 0.0;
        for series in &p.series {
            let label = series.legend.as_ref()?;
            if label.font != *font || label.metrics.horizontal != 0 {
                return None;
            }
            let size = text_size(label)?;
            marker = marker.max(size[1] * unit);
            label_width = label_width.max(size[0] * unit);
        }
        let column_width = marker * 1.5 + label_width;
        // 열 사이 간격은 저장 Section 좌표이며 마커/레이블은 물리 점유 크기다.
        let gap = f64::from(section.internal) * sx;
        let margin = inset(section);
        let available_width = width - 2.0 * f64::from(section.external) * sx;
        let cols = ((available_width + gap) / (column_width + gap)).floor() as usize;
        if cols == 0 {
            return None;
        }
        let cols = cols.min(chart.series.len());
        let rows = chart.series.len().div_ceil(cols);
        let grid_w = cols as f64 * (column_width + gap) - gap;
        let row_step = marker * 1.5;
        let grid_h = rows as f64 * row_step - marker * 0.5;
        // Section 최대 쪽의 50단위도 같은 저장 좌표 변환을 사용한다.
        let grid_bottom = map_y(section.rect[1] + margin + 50.0);
        let y = grid_bottom - grid_h;
        let x = (width - grid_w - 50.0 * sx) / 2.0;
        legend_layout = Some((x, y, marker, column_width + gap, row_step, cols));
    }

    let category = p.labels[0].as_ref()?;
    let value = p.labels[1].as_ref()?;
    if !plain(&category.font) || category.metrics.horizontal != 2 || !plain(&value.font) {
        return None;
    }
    text_size(category)?;
    let value_size = text_size(value)?;
    let margin = inset(&p.plot_section);
    // 값축 양 끝의 반 줄을 각각 예약한다. 범주 레이블의 저장 점유는 눈금/여백을 포함한다.
    let endpoint = value_size[1] / 2.0;
    let value_gutter = -f64::from(value.metrics.rect[0]);
    let category_gutter = -f64::from(category.metrics.rect[1]);
    if value_gutter < 0.0 || category_gutter < 0.0 {
        return None;
    }
    let left = map_x(p.plot_section.rect[0] + margin + value_gutter);
    let right = map_x(p.plot_section.rect[2] - margin);
    let top = map_y(p.plot_section.rect[3] - margin - endpoint);
    let bottom = map_y(p.plot_section.rect[1] + margin + endpoint + category_gutter);
    if left >= right || top >= bottom {
        return None;
    }
    let plot_w = right - left;
    let plot_h = bottom - top;
    let axis = p.axes[0];
    let mut svg = String::new();
    if let Some(b) = p.backdrop.as_ref() {
        if b.fill_kind == 1 {
            let _ = write!(
                svg,
                "<rect width=\"{width:.4}\" height=\"{height:.4}\" fill=\"{}\"/>",
                color(b.fill_color)
            );
        }
        if b.frame_enabled {
            let stroke = b.frame_width * unit;
            let _ = write!(svg, "<rect x=\"{:.4}\" y=\"{:.4}\" width=\"{:.4}\" height=\"{:.4}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{stroke:.4}\"/>", stroke / 2.0, stroke / 2.0, width - stroke, height - stroke, color(b.frame_color));
        }
    }
    if let (Some(title), Some([x, y])) = (p.title.as_ref(), title_origin) {
        let attrs = font_attributes(Some(&title.font), dpi, "");
        let _ = write!(svg, "<text x=\"{:.4}\" y=\"{:.4}\" {attrs} text-anchor=\"middle\" dominant-baseline=\"central\" xml:space=\"preserve\">{}</text>", x, y, escape_xml_text(&title.value));
    }
    let value_attrs = font_attributes(Some(&value.font), dpi, "");
    let vertical = p.axis_styles[1];
    for i in 0..=axis.divisions {
        let ratio = f64::from(i) / f64::from(axis.divisions);
        let y = bottom - plot_h * ratio;
        line(&mut svg, [left, y, right, y], vertical.major_grid, unit);
        let length = vertical.tick_length * unit;
        let (inside, outside) = tick_sides(vertical.tick_style, length)?;
        line(
            &mut svg,
            [left - outside, y, left + inside, y],
            vertical.pen,
            unit,
        );
        if vertical.labels_visible {
            let number = axis.min + (axis.max - axis.min) * ratio;
            let (label_x, anchor) = match value.metrics.horizontal {
                0 => (left + f64::from(value.metrics.paint_rect[0]) * sx, "start"),
                1 => (left + f64::from(value.metrics.paint_rect[2]) * sx, "end"),
                _ => (
                    left + (f64::from(value.metrics.paint_rect[0])
                        + f64::from(value.metrics.paint_rect[2]))
                        * sx
                        / 2.0,
                    "middle",
                ),
            };
            let _ = write!(svg, "<text x=\"{label_x:.4}\" y=\"{y:.4}\" {value_attrs} text-anchor=\"{anchor}\" dominant-baseline=\"central\">{}</text>", format_axis_value(number));
        }
    }
    line(&mut svg, [left, top, left, bottom], vertical.pen, unit);
    let horizontal = p.axis_styles[0];
    line(
        &mut svg,
        [left, bottom, right, bottom],
        horizontal.pen,
        unit,
    );
    let category_attrs = font_attributes(Some(&category.font), dpi, "");
    let cat_w = plot_w / chart.categories.len() as f64;
    let (inside, outside) = tick_sides(horizontal.tick_style, horizontal.tick_length * unit)?;
    for index in 0..=chart.categories.len() {
        let x = left + index as f64 * cat_w;
        line(
            &mut svg,
            [x, bottom - inside, x, bottom + outside],
            horizontal.pen,
            unit,
        );
    }
    for (index, label) in chart.categories.iter().enumerate() {
        let x = left + (index as f64 + 0.5) * cat_w;
        let y = bottom
            - (f64::from(category.metrics.paint_rect[1])
                + f64::from(category.metrics.paint_rect[3]))
                * sy
                / 2.0;
        if horizontal.labels_visible {
            let _ = write!(svg, "<text x=\"{x:.4}\" y=\"{y:.4}\" {category_attrs} text-anchor=\"middle\" dominant-baseline=\"central\">{}</text>", escape_xml_text(label));
        }
    }
    let count = chart.series.len();
    let bar_w = cat_w / (count as f64 + (count - 1) as f64 * p.bar_gap + p.x_gap);
    let position = |number: f64| bottom - (number - axis.min) / (axis.max - axis.min) * plot_h;
    let zero = position(0.0_f64.clamp(axis.min, axis.max));
    for (series_index, series) in chart.series.iter().enumerate() {
        for (index, value) in series.values.iter().copied().enumerate() {
            if !value.is_finite() {
                continue;
            }
            let x = left
                + index as f64 * cat_w
                + bar_w * p.x_gap / 2.0
                + series_index as f64 * bar_w * (1.0 + p.bar_gap);
            let end = position(value.clamp(axis.min, axis.max));
            let _ = write!(svg, "<rect x=\"{x:.4}\" y=\"{:.4}\" width=\"{bar_w:.4}\" height=\"{:.4}\" fill=\"{}\"/>", end.min(zero), (end - zero).abs(), series_fill(Some(p), series_index));
        }
    }
    if let Some((x, y, marker, col_step, row_step, cols)) = legend_layout {
        let attrs = font_attributes(p.legend_font.as_ref(), dpi, "");
        for (index, _) in chart.series.iter().enumerate() {
            let x = x + (index % cols) as f64 * col_step;
            let y = y + (index / cols) as f64 * row_step;
            let size = marker;
            let label = p.series[index].legend.as_ref()?;
            let name = &label.value;
            // 마커 packing의 물리 줄 높이와 글자의 저장 paint 상자를 구분한다.
            // TextBlock 원점은 painter의 위쪽 여유에서 시작하고, 정렬은
            // 축/제목과 같은 root 좌표계의 실제 글자 상자를 소비한다.
            let label_y = y - size / 4.0
                + (f64::from(label.metrics.paint_rect[1]) + f64::from(label.metrics.paint_rect[3]))
                    * sy
                    / 2.0;
            // 열 폭의 반 줄 여유와 실제 TextBlock 시작 여백은 별개다.
            // 범례 painter는 마커 오른쪽에서 50twip 떨어진 상자에 글자를 배치한다.
            let label_x = x + size + 50.0 * unit;
            let _ = write!(svg, "<rect x=\"{x:.4}\" y=\"{y:.4}\" width=\"{size:.4}\" height=\"{size:.4}\" fill=\"{}\"/><text x=\"{label_x:.4}\" y=\"{label_y:.4}\" {attrs} dominant-baseline=\"central\">{}</text>", series_fill(Some(p), index), escape_xml_text(name));
        }
    }
    Some(svg)
}

fn tick_sides(style: u16, length: f64) -> Option<(f64, f64)> {
    match style {
        0 => Some((0.0, 0.0)),
        1 => Some((length / 2.0, length / 2.0)),
        2 => Some((length, 0.0)),
        3 => Some((0.0, length)),
        _ => None,
    }
}
