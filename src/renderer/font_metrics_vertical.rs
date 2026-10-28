//! 확인된 원본 프로그램의 수직 줄 프레임. 표시 대체 face의 잉크 상자와 구분한다.

use super::{find_metric_decision, MetricMatchKind};

struct VerticalMetric {
    name: &'static str,
    bold: bool,
    italic: bool,
    units_per_em: u16,
    win_descent: u16,
}

// HANDotum.ttf (HCR Dotum Regular), UPEM1000, OS/2 Win1070/230,
// hhea1070/-230. 파일 SHA-256:
// 1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898.
// 독립 Print의 PII/7336/biz_plan 및 쪽 여백 대조와 출처는 stage8 C437에서 다룬다.
// 확인되지 않은 프로그램·정체에 수직 프레임을 추측해 적용하지 않는다.
const VERTICAL_METRICS: &[VerticalMetric] = &[VerticalMetric {
    name: "HCR Dotum",
    bold: false,
    italic: false,
    units_per_em: 1000,
    win_descent: 230,
}];

pub(crate) fn win_descent_em(name: &str, bold: bool, italic: bool) -> Option<f64> {
    let decision = find_metric_decision(name, bold, italic)?;
    if decision.match_kind != MetricMatchKind::Exact {
        return None;
    }
    let metric = VERTICAL_METRICS.iter().find(|entry| {
        entry.name == decision.metric.name && entry.bold == bold && entry.italic == italic
    })?;
    Some(f64::from(metric.win_descent) / f64::from(metric.units_per_em))
}
