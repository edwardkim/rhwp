//! `charming` SSR renderer for OLE chart work.

use std::fmt;

use charming::{
    component::{Axis, Grid, Legend, Title},
    element::AxisType,
    series::{Bar, Line, Pie, PieRoseType},
    Chart, ImageRenderer,
};

use super::{OleChart, OleChartType};

/// OLE chart rendering error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OleChartRenderError {
    EmptyChart,
    CharmingSsr(String),
}

impl OleChartRenderError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::EmptyChart => "EMPTY_OLE_CHART",
            Self::CharmingSsr(_) => "CHARMING_SSR_RENDER_FAILED",
        }
    }
}

impl fmt::Display for OleChartRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChart => f.write_str("empty OLE chart"),
            Self::CharmingSsr(message) => write!(f, "charming SSR render failed: {message}"),
        }
    }
}

impl std::error::Error for OleChartRenderError {}

/// Renders a parsed OLE chart through `charming` SSR and returns a standalone SVG document.
pub fn render_ole_chart_charming_svg(
    chart: &OleChart,
    width: u32,
    height: u32,
) -> Result<String, OleChartRenderError> {
    if chart.categories.is_empty() || chart.series.is_empty() {
        return Err(OleChartRenderError::EmptyChart);
    }

    let chart_option = build_charming_chart(chart);
    ImageRenderer::new(width.max(1), height.max(1))
        .render(&chart_option)
        .map_err(|error| OleChartRenderError::CharmingSsr(error.to_string()))
}

/// Renders a minimal chart through `charming` SSR and returns SVG text.
pub fn render_smoke_chart_svg(width: u32, height: u32) -> Result<String, OleChartRenderError> {
    let chart = Chart::new().legend(Legend::new().top("bottom")).series(
        Pie::new()
            .name("RHWP OLE Chart Smoke")
            .rose_type(PieRoseType::Radius)
            .radius(vec!["30", "70"])
            .center(vec!["50%", "45%"])
            .data(vec![
                (40.0, "alpha"),
                (32.0, "beta"),
                (24.0, "gamma"),
                (16.0, "delta"),
            ]),
    );

    ImageRenderer::new(width, height)
        .render(&chart)
        .map_err(|error| OleChartRenderError::CharmingSsr(error.to_string()))
}

fn build_charming_chart(ole_chart: &OleChart) -> Chart {
    let mut chart = Chart::new()
        .legend(Legend::new().top("bottom"))
        .grid(
            Grid::new()
                .left("8%")
                .right("6%")
                .top(if ole_chart.title.is_some() {
                    "18%"
                } else {
                    "10%"
                })
                .bottom("18%")
                .contain_label(true),
        )
        .x_axis(
            Axis::new()
                .type_(AxisType::Category)
                .data(ole_chart.categories.clone()),
        )
        .y_axis(Axis::new().type_(AxisType::Value));

    if let Some(title) = ole_chart.title.as_ref() {
        chart = chart.title(Title::new().text(title.clone()).left("center").top("2%"));
    }

    for series in &ole_chart.series {
        let name = series.name.clone().unwrap_or_default();
        chart = match ole_chart.chart_type {
            OleChartType::Line => chart.series(
                Line::new()
                    .name(name)
                    .show_symbol(true)
                    .data(series.values.clone()),
            ),
            _ => chart.series(Bar::new().name(name).data(series.values.clone())),
        };
    }

    chart
}
