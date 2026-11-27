//! 확인된 원본 프로그램의 수직 줄 프레임. 표시 대체 face의 잉크 상자와 구분한다.

use super::{find_metric_decision, MetricMatchKind};

struct VerticalMetric {
    name: &'static str,
    bold: bool,
    italic: bool,
    units_per_em: u16,
    ascent: u16,
    win_descent: u16,
    /// 쪽 번호 글자 칸의 기준선 아래 여유(`win_descent_em`)까지 독립 Print 로 확인했다.
    page_number_frame: bool,
}

// HANDotum.ttf (HCR Dotum Regular), UPEM1000, OS/2 Win1070/230,
// hhea1070/-230. 파일 SHA-256:
// 1d0437906d52e96af36943008f144ad74d6e994493df3a4210a9761e077ea898.
// 독립 Print의 PII/7336/biz_plan 및 쪽 여백 대조와 출처는 stage8 C437에서 다룬다.
// HCR Batang Regular/Bold: 한/글 2020(11.0.0.1623) `Hancom PDF` Print 가 내장한
// `HCRBatang`·`HCRBatang-Bold` 부분 글꼴의 hhea 1070/-230, UPEM 1000(#9379, KAF 회의자료
// Print SHA-256 fa1b9e36…6c4c). 같은 한/글이 다시 저장한 글꼴에 어울리는 줄 높이 문단의
// 저장 줄(13pt: 줄 높이 1691·기준선 1391)과 일치한다.
// 확인되지 않은 프로그램·정체에 수직 프레임을 추측해 적용하지 않는다.
const VERTICAL_METRICS: &[VerticalMetric] = &[
    VerticalMetric {
        name: "HCR Dotum",
        bold: false,
        italic: false,
        units_per_em: 1000,
        ascent: 1070,
        win_descent: 230,
        page_number_frame: true,
    },
    VerticalMetric {
        name: "HCR Batang",
        bold: false,
        italic: false,
        units_per_em: 1000,
        ascent: 1070,
        win_descent: 230,
        page_number_frame: false,
    },
    VerticalMetric {
        name: "HCR Batang",
        bold: true,
        italic: false,
        units_per_em: 1000,
        ascent: 1070,
        win_descent: 230,
        page_number_frame: false,
    },
];

fn exact_vertical_metric(name: &str, bold: bool, italic: bool) -> Option<&'static VerticalMetric> {
    let decision = find_metric_decision(name, bold, italic)?;
    if decision.match_kind != MetricMatchKind::Exact {
        return None;
    }
    VERTICAL_METRICS.iter().find(|entry| {
        entry.name == decision.metric.name && entry.bold == bold && entry.italic == italic
    })
}

/// 글꼴에 어울리는 줄 높이(HWP 5.0 문단 모양 속성 1 bit 22)가 쓰는 글꼴의 세로 상자.
/// (위, 아래) 를 em 비율로 돌려준다. 확인된 글꼴만 답한다.
pub(crate) fn font_line_box_em(name: &str, bold: bool, italic: bool) -> Option<(f64, f64)> {
    let metric = exact_vertical_metric(name, bold, italic)?;
    let em = f64::from(metric.units_per_em);
    Some((
        f64::from(metric.ascent) / em,
        f64::from(metric.win_descent) / em,
    ))
}

pub(crate) fn win_descent_em(name: &str, bold: bool, italic: bool) -> Option<f64> {
    let metric = exact_vertical_metric(name, bold, italic).filter(|m| m.page_number_frame)?;
    Some(f64::from(metric.win_descent) / f64::from(metric.units_per_em))
}
