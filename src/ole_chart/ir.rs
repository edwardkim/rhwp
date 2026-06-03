//! Renderer-neutral OLE chart IR export helpers.

use base64::Engine;
use serde::Serialize;

use super::OleChart;

pub const OLE_CHART_IR_SCHEMA: &str = "rhwp.oleChartIr";
pub const OLE_CHART_IR_VERSION: u32 = 1;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OleChartIrPayload<'a> {
    pub schema: &'static str,
    pub version: u32,
    pub chart: &'a OleChart,
}

impl<'a> OleChartIrPayload<'a> {
    pub fn new(chart: &'a OleChart) -> Self {
        Self {
            schema: OLE_CHART_IR_SCHEMA,
            version: OLE_CHART_IR_VERSION,
            chart,
        }
    }
}

pub fn ole_chart_ir_json(chart: &OleChart) -> Result<String, serde_json::Error> {
    serde_json::to_string(&OleChartIrPayload::new(chart))
}

pub fn ole_chart_ir_base64(chart: &OleChart) -> Result<String, serde_json::Error> {
    Ok(base64::engine::general_purpose::STANDARD.encode(ole_chart_ir_json(chart)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ole_chart::{OleChartSeries, OleChartType};

    fn sample_chart() -> OleChart {
        OleChart {
            chart_type: OleChartType::Unknown,
            title: Some("연금 재정 전망".to_string()),
            categories: vec!["2010년".to_string(), "2020년".to_string()],
            series: vec![OleChartSeries {
                name: Some("적립금".to_string()),
                values: vec![328.0, 812.0],
            }],
        }
    }

    #[test]
    fn serializes_renderer_neutral_ir_payload() {
        let json = ole_chart_ir_json(&sample_chart()).expect("serialize chart ir");

        assert!(json.contains("\"schema\":\"rhwp.oleChartIr\""));
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"chartType\":\"unknown\""));
        assert!(json.contains("연금 재정 전망"));
        assert!(json.contains("적립금"));
    }

    #[test]
    fn serializes_renderer_neutral_ir_payload_as_base64() {
        let encoded = ole_chart_ir_base64(&sample_chart()).expect("serialize chart ir");

        assert!(!encoded.is_empty());
        assert!(!encoded.contains("연금 재정 전망"));
    }
}
