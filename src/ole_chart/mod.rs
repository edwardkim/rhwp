//! HWP OLE `Contents` 기반 차트 파싱 골격.
//!
//! 구버전/Neo 계열 HWP 차트 OLE는 `OOXMLChartContents`나 `OlePres000`
//! 미리보기 없이 `Contents` 스트림만 담는 경우가 있다. 이 모듈은 해당
//! 스트림을 차트 IR로 해석하기 위한 전용 진입점이다.

pub mod grid;
pub mod ir;
mod legacy_column_renderer;
mod legacy_combo_renderer;
mod legacy_presentation;
pub mod orientation;
pub mod parser;
mod preview_palette;
pub mod svg_renderer;

pub use grid::{
    legacy_grid_window, scan_legacy_grid, GridCell, GridScanError, GridValue, LegacyChartGrid,
};
pub use ir::{
    ole_chart_ir_base64, ole_chart_ir_json, OleChartIrPayload, OLE_CHART_IR_SCHEMA,
    OLE_CHART_IR_VERSION,
};
pub use orientation::{decide_series_axis, SeriesAxis, SeriesAxisEvidence};
pub use parser::{
    parse_ole_chart_contents, probe_ole_chart_contents, OleChart, OleChartContentsProbe,
    OleChartParseError, OleChartSeries, OleChartType,
};
pub use preview_palette::apply_preview_palette;
pub use svg_renderer::{
    render_ole_chart_standalone_svg, render_ole_chart_svg_body, render_ole_chart_svg_fragment,
};

/// 지원하는 구형 차트의 저장 축·계열 속성을 데이터 IR과 함께 소비한다.
/// 지원하지 않는 archive 버전은 기존 렌더링 경로를 유지한다.
pub fn render_ole_chart_svg_fragment_with_contents(
    chart: &OleChart,
    contents: &[u8],
    bounds: [f64; 4],
    bin_data_id: u32,
) -> String {
    render_ole_chart_svg_fragment_with_contents_at_dpi(chart, contents, bounds, bin_data_id, 96.0)
}

/// 저장 포인트 크기를 레이아웃과 같은 DPI에서 출력한다.
pub fn render_ole_chart_svg_fragment_with_contents_at_dpi(
    chart: &OleChart,
    contents: &[u8],
    bounds: [f64; 4],
    bin_data_id: u32,
    dpi: f64,
) -> String {
    if let Some(presentation) = legacy_presentation::parse(contents) {
        if presentation.chart_type == 1
            && !presentation.series.iter().any(|series| series.secondary)
            && chart.series.len() == presentation.series.len()
            && chart
                .series
                .iter()
                .all(|series| series.values.len() == chart.categories.len())
        {
            return svg_renderer::render_legacy_column_fragment(
                chart,
                &presentation,
                bounds,
                bin_data_id,
                dpi,
            );
        }
        if let Some(svg) = legacy_combo_renderer::render(chart, &presentation, bounds) {
            return svg;
        }
    }
    let [x, y, width, height] = bounds;
    render_ole_chart_svg_fragment(chart, x, y, width, height, bin_data_id)
}
