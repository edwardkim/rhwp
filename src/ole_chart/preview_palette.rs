//! OLE 미리보기에서 문서 전용 OOXML 차트 색상표를 복원한다.
//!
//! `hncChartStyle colorIndex=-1`은 고정 Office 팔레트가 아니다. 미리보기의
//! 막대 면색을 읽되, 값과 라벨은 항상 편집 가능한 OOXML 차트에서 그린다.

use std::collections::HashMap;

use base64::Engine;
use image::{DynamicImage, GenericImageView};

use crate::ole_chart::OleChart;
use crate::ooxml_chart::{BarGrouping, OoxmlChart, OoxmlChartType};

fn saturated([r, g, b]: [u8; 3]) -> bool {
    r.max(g).max(b) - r.min(g).min(b) >= 30 && r.min(g).min(b) < 245
}

fn preview_image(svg: &str) -> Option<DynamicImage> {
    let mut best = None;
    let mut best_score = 0usize;
    for encoded in svg.split("data:image/png;base64,").skip(1) {
        let Some(encoded) = encoded.split('"').next() else {
            continue;
        };
        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(encoded) else {
            continue;
        };
        if bytes.len() > 16 * 1024 * 1024 {
            continue;
        }
        let Ok(image) = image::load_from_memory(&bytes) else {
            continue;
        };
        if image.width() > 4096 || image.height() > 4096 {
            continue;
        }
        let score = image.to_rgb8().pixels().filter(|p| saturated(p.0)).count();
        if score > best_score {
            best_score = score;
            best = Some(image);
        }
    }
    if best_score > 100 {
        best
    } else {
        None
    }
}

fn bar_centers(image: &DynamicImage, wanted: usize) -> Vec<u32> {
    let (w, h) = image.dimensions();
    let rgb = image.to_rgb8();
    let start_y = h / 6;
    let end_y = h * 4 / 5;
    let counts: Vec<usize> = (0..w)
        .map(|x| {
            (start_y..end_y)
                .filter(|&y| saturated(rgb.get_pixel(x, y).0))
                .count()
        })
        .collect();
    let peak = *counts.iter().max().unwrap_or(&0);
    if peak < 12 {
        return Vec::new();
    }
    let threshold = (peak * 2 / 5).max(5);
    let mut runs = Vec::new();
    let mut start = None;
    for x in 0..=w {
        let active = x < w && counts[x as usize] >= threshold;
        match (start, active) {
            (None, true) => start = Some(x),
            (Some(s), false) => {
                if x - s >= 3 {
                    runs.push((s + x - 1) / 2);
                }
                start = None;
            }
            _ => {}
        }
    }
    if runs.len() == wanted {
        runs
    } else {
        Vec::new()
    }
}

fn sample_color(image: &DynamicImage, x: u32, y: f64) -> Option<u32> {
    let rgb = image.to_rgb8();
    let cy = y.round() as i32;
    let mut counts = HashMap::<u32, usize>::new();
    for dy in -2..=2 {
        for dx in -4..=4 {
            let (xx, yy) = (x as i32 + dx, cy + dy);
            if xx < 0 || yy < 0 || xx >= rgb.width() as i32 || yy >= rgb.height() as i32 {
                continue;
            }
            let [r, g, b] = rgb.get_pixel(xx as u32, yy as u32).0;
            if [r, g, b] == [255, 255, 255] {
                continue;
            }
            let color = ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            *counts.entry(color).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .filter(|(_, count)| *count >= 6)
        .map(|(color, _)| color)
}

/// 미리보기의 원래 계열 색을 쓰되 편집된 OOXML 값·라벨은 그대로 유지한다.
/// 기존 `Contents`의 값 배열을 좌표 추정에 쓰므로 데이터 편집 후에도 낡은
/// 미리보기 값으로 막대 길이를 되돌리지 않는다. 지원하지 않는 구조는 무변경이다.
pub fn apply_preview_palette(
    chart: &mut OoxmlChart,
    legacy_contents: &[u8],
    preview_emf: &[u8],
    width: f64,
    height: f64,
) -> bool {
    if chart.color_index != Some(-1)
        || chart.chart_type != OoxmlChartType::Column
        || chart.grouping != BarGrouping::Stacked
        || chart.is_3d
        || chart.series.len() < 2
    {
        return false;
    }
    let Ok(legacy) = crate::ole_chart::parse_ole_chart_contents(legacy_contents) else {
        return false;
    };
    if legacy.series.len() != chart.series.len()
        || legacy.series.iter().zip(&chart.series).any(|(old, new)| {
            old.values.len() != new.values.len()
                || old.name.as_deref().is_some_and(|name| name != new.name)
        })
    {
        return false;
    }
    let Ok(svg) = crate::emf::convert_to_svg(preview_emf, (0.0, 0.0, width as f32, height as f32))
    else {
        return false;
    };
    let Some(image) = preview_image(&svg) else {
        return false;
    };
    let Some(category_count) = legacy.series.first().map(|s| s.values.len()) else {
        return false;
    };
    let centers = bar_centers(&image, category_count);
    if centers.is_empty() {
        return false;
    }
    let rgb = image.to_rgb8();
    let mut palette = Vec::with_capacity(legacy.series.len());
    for series_idx in 0..legacy.series.len() {
        let Some((category, value)) = legacy.series[series_idx]
            .values
            .iter()
            .copied()
            .enumerate()
            .filter(|(ci, v)| *ci < centers.len() && *v > 0.0)
            .max_by(|a, b| a.1.total_cmp(&b.1))
        else {
            return false;
        };
        let x = centers[category];
        let colored_y: Vec<u32> = (rgb.height() / 6..rgb.height() * 4 / 5)
            .filter(|&y| saturated(rgb.get_pixel(x, y).0))
            .collect();
        let (Some(top), Some(bottom)) = (colored_y.first(), colored_y.last()) else {
            return false;
        };
        let total: f64 = legacy
            .series
            .iter()
            .map(|s| s.values[category].max(0.0))
            .sum();
        if total <= 0.0 || bottom - top < 20 {
            return false;
        }
        let scale = (bottom - top + 1) as f64 / total;
        let preceding: f64 = legacy.series[..series_idx]
            .iter()
            .map(|s| s.values[category].max(0.0))
            .sum();
        let y = *bottom as f64 - (preceding + value / 2.0) * scale;
        let Some(color) = sample_color(&image, x, y) else {
            return false;
        };
        palette.push(color);
    }
    for (series, color) in chart.series.iter_mut().zip(palette) {
        series.color = Some(color);
    }
    true
}
