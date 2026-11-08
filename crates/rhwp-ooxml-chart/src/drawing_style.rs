//! 차트의 선·글꼴 선언을 정확한 소유 경로에서 읽는다.
//!
//! 계열의 데이터 라벨·표식·개별 점도 spPr/ln을 갖는다. 그 선을 계열 자체의
//! 테두리로 승격하지 않으며, 축의 격자선과 축선도 별도로 보존한다.

use quick_xml::{events::Event, Reader};

use crate::{LineSpec, OoxmlChart};

#[derive(Clone, Copy)]
enum Owner {
    Chart,
    Plot,
    Series(usize),
    Axis(usize),
    Grid(usize),
}

struct Line {
    owner: Owner,
    depth: usize,
    spec: LineSpec,
    width: Option<u32>,
    color: Option<u32>,
    luminance: (f64, f64),
}

fn owner(path: &[String], series: Option<usize>, axis: Option<usize>) -> Option<Owner> {
    let names: Vec<_> = path.iter().map(String::as_str).collect();
    if names.ends_with(&["ser", "spPr", "ln"]) {
        series.map(Owner::Series)
    } else if names.ends_with(&["majorGridlines", "spPr", "ln"]) {
        axis.map(Owner::Grid)
    } else if names.len() >= 3 && names[names.len() - 2..] == ["spPr", "ln"] {
        match names[names.len() - 3] {
            "catAx" | "valAx" | "dateAx" | "serAx" => axis.map(Owner::Axis),
            "plotArea" => Some(Owner::Plot),
            "chartSpace" => Some(Owner::Chart),
            _ => None,
        }
    } else {
        None
    }
}

fn store(chart: &mut OoxmlChart, line: Line) {
    match line.owner {
        Owner::Chart => chart.chart_line = line.spec,
        Owner::Plot => chart.plot_area_line = line.spec,
        Owner::Series(i) => {
            if let Some(ser) = chart.series.get_mut(i) {
                ser.shape_line = line.spec;
                ser.line_none = line.spec == LineSpec::None;
                ser.line_width_emu = line.width;
            }
        }
        Owner::Axis(i) => {
            if let Some(axis) = chart.axes.get_mut(i) {
                axis.line = line.spec;
            }
        }
        Owner::Grid(i) => {
            if let Some(axis) = chart.axes.get_mut(i) {
                axis.grid_line = line.spec;
            }
        }
    }
}

/// 값·계열 순서는 기존 파서가 소유한다. 표현 속성만 같은 원본에서 보완한다.
pub(crate) fn resolve(xml: &[u8], chart: &mut OoxmlChart) {
    // 이전의 넓은 문맥으로 읽은 라벨·표식 선은 계열 기본값으로 남기지 않는다.
    // 계열 자체 ln이 없는 경우에도 중첩 noFill/선 굵기의 오염을 제거한다.
    for series in &mut chart.series {
        series.line_none = false;
        series.line_width_emu = None;
    }
    let mut reader = Reader::from_reader(xml);
    let mut buf = Vec::new();
    let mut path = Vec::<String>::new();
    let (mut series_count, mut axis_count) = (0, 0);
    let (mut series, mut axis) = (None, None);
    let mut line = None::<Line>;
    while let Ok(event) = reader.read_event_into(&mut buf) {
        match &event {
            Event::Start(e) | Event::Empty(e) => {
                let name = e.local_name().as_ref().to_string();
                path.push(name.clone());
                let attr = |key| crate::parser::attr_val(e, key);
                match name.as_str() {
                    "ser" => {
                        series = Some(series_count);
                        series_count += 1;
                    }
                    "idx" if path.iter().rev().nth(1).is_some_and(|name| name == "ser") => {
                        if let Some(ser) = series.and_then(|i| chart.series.get_mut(i)) {
                            ser.source_index = attr("val").and_then(|v| v.parse().ok());
                        }
                    }
                    "catAx" | "valAx" | "dateAx" | "serAx" => {
                        axis = Some(axis_count);
                        axis_count += 1;
                    }
                    "ln" => {
                        line = owner(&path, series, axis).map(|owner| Line {
                            owner,
                            depth: path.len(),
                            spec: LineSpec::Unspecified,
                            width: attr("w").and_then(|v| v.parse().ok()),
                            color: None,
                            luminance: (1.0, 0.0),
                        });
                    }
                    "noFill" => {
                        if let Some(line) = &mut line {
                            line.spec = LineSpec::None;
                        }
                    }
                    "srgbClr" | "sysClr" | "schemeClr" => {
                        if let Some(line) = &mut line {
                            line.color = if name == "schemeClr" {
                                attr("val").and_then(|v| crate::parser::scheme_color(&v))
                            } else {
                                attr(if name == "sysClr" { "lastClr" } else { "val" })
                                    .and_then(|v| crate::parser::parse_rgb_hex(&v))
                            };
                            line.luminance = (1.0, 0.0);
                        }
                    }
                    "lumMod" | "lumOff" => {
                        if let (Some(line), Some(value)) =
                            (&mut line, attr("val").and_then(|v| v.parse::<f64>().ok()))
                        {
                            if name == "lumMod" {
                                line.luminance.0 = value / 100_000.0;
                            } else {
                                line.luminance.1 = value / 100_000.0;
                            }
                        }
                    }
                    "defRPr" => {
                        let names: Vec<_> = path.iter().map(String::as_str).collect();
                        if names.ends_with(&["legend", "txPr", "p", "pPr", "defRPr"]) {
                            chart.legend_size_pt = attr("sz")
                                .and_then(|value| value.parse::<f64>().ok())
                                .filter(|size| size.is_finite() && *size > 0.0)
                                .map(|size| size / 100.0);
                        }
                    }
                    "latin" => {
                        let names: Vec<_> = path.iter().map(String::as_str).collect();
                        if names.ends_with(&["txPr", "p", "pPr", "defRPr", "latin"]) {
                            let scope = names.get(names.len().wrapping_sub(6)).copied();
                            let target = match scope {
                                Some("chartSpace") => Some(&mut chart.text_font_family),
                                Some("legend") => Some(&mut chart.legend_font_family),
                                Some("title") => Some(&mut chart.title_font_family),
                                _ => None,
                            };
                            if let Some(target) = target {
                                *target = attr("typeface");
                            }
                        } else if names
                            .ends_with(&["title", "tx", "rich", "p", "pPr", "defRPr", "latin"])
                            || names.ends_with(&["title", "tx", "rich", "p", "r", "rPr", "latin"])
                        {
                            chart.title_font_family = attr("typeface");
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        if matches!(event, Event::End(_) | Event::Empty(_)) {
            let name = path.last().map(String::as_str).unwrap_or("");
            if matches!(name, "srgbClr" | "sysClr" | "schemeClr") {
                if let Some(line) = &mut line {
                    if let Some(rgb) = line.color.take() {
                        line.spec = LineSpec::Solid {
                            rgb: crate::parser::apply_luminance(
                                rgb,
                                line.luminance.0,
                                line.luminance.1,
                            ),
                            width_emu: line.width,
                        };
                    }
                }
            }
            if name == "ln" && line.as_ref().is_some_and(|line| line.depth == path.len()) {
                store(chart, line.take().unwrap());
            }
            match name {
                "ser" => series = None,
                "catAx" | "valAx" | "dateAx" | "serAx" => axis = None,
                _ => {}
            }
            path.pop();
        }
        if matches!(event, Event::Eof) {
            break;
        }
        buf.clear();
    }
}
