//! HWP OLE `Contents` 기반 차트 파싱 골격.
//!
//! 구버전/Neo 계열 HWP 차트 OLE는 `OOXMLChartContents`나 `OlePres000`
//! 미리보기 없이 `Contents` 스트림만 담는 경우가 있다. 이 모듈은 해당
//! 스트림을 차트 IR로 해석하기 위한 전용 진입점이다.

#[cfg(all(not(target_arch = "wasm32"), feature = "charming-renderer"))]
pub mod charming_renderer;
pub mod ir;
pub mod parser;
pub mod svg_renderer;

#[cfg(all(not(target_arch = "wasm32"), feature = "charming-renderer"))]
pub use charming_renderer::{
    render_ole_chart_charming_svg, render_smoke_chart_svg, OleChartRenderError,
};
pub use ir::{
    ole_chart_ir_base64, ole_chart_ir_json, OleChartIrPayload, OLE_CHART_IR_SCHEMA,
    OLE_CHART_IR_VERSION,
};
pub use parser::{
    parse_ole_chart_contents, probe_ole_chart_contents, OleChart, OleChartContentsProbe,
    OleChartParseError, OleChartSeries, OleChartType,
};
pub use svg_renderer::{
    render_ole_chart_standalone_svg, render_ole_chart_svg_body, render_ole_chart_svg_fragment,
};
