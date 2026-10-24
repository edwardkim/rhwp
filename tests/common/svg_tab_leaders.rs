// SVG 점선 탭 리더: 개별 원과 둥근 끝 dash 선을 함께 읽는다.
// 태그 표현과 분리해 실제 점 간격·지름·끝 위치의 계약을 유지한다.
use std::collections::BTreeMap;

pub struct DotLeader {
    pub start: f64,
    pub end: f64,
    pub y: f64,
    pub diameter: f64,
    pub pitch: f64,
    pub max_step_error: f64,
}

pub fn dot_leaders(svg: &str) -> Vec<DotLeader> {
    let document = roxmltree::Document::parse(svg).expect("유효한 SVG");
    let number = |node: roxmltree::Node<'_, '_>, key| {
        node.attribute(key)
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
    };
    let mut leaders = Vec::new();
    let mut circles: BTreeMap<(i64, i64), Vec<f64>> = BTreeMap::new();
    for node in document.descendants().filter(|node| node.is_element()) {
        if node.has_tag_name("circle") {
            if let (Some(x), Some(y), Some(radius)) =
                (number(node, "cx"), number(node, "cy"), number(node, "r"))
            {
                if radius > 0.0 && node.attribute("fill") != Some("none") {
                    circles
                        .entry(((y * 1e6).round() as i64, (radius * 1e6).round() as i64))
                        .or_default()
                        .push(x);
                }
            }
        } else if node.has_tag_name("line") && node.attribute("stroke-linecap") == Some("round") {
            let dash: Vec<f64> = node
                .attribute("stroke-dasharray")
                .unwrap_or_default()
                .split_whitespace()
                .filter_map(|value| value.parse().ok())
                .collect();
            if dash.len() != 2 || dash[0] > 0.5 || dash[1] <= 0.0 {
                continue;
            }
            if let (Some(start), Some(end), Some(y), Some(y2), Some(diameter)) = (
                number(node, "x1"),
                number(node, "x2"),
                number(node, "y1"),
                number(node, "y2"),
                number(node, "stroke-width"),
            ) {
                if (y - y2).abs() < 0.1 && end > start && diameter > 0.0 {
                    leaders.push(DotLeader {
                        start,
                        end,
                        y,
                        diameter,
                        pitch: dash[0] + dash[1],
                        max_step_error: 0.0,
                    });
                }
            }
        }
    }
    for ((y, radius), mut centers) in circles {
        let y = y as f64 / 1e6;
        let radius = radius as f64 / 1e6;
        centers.sort_by(f64::total_cmp);
        let mut start = 0;
        for end in 1..=centers.len() {
            if end < centers.len() && centers[end] - centers[end - 1] <= radius * 6.0 {
                continue;
            }
            let group = &centers[start..end];
            if group.len() >= 3 {
                let pitch = (group[group.len() - 1] - group[0]) / (group.len() - 1) as f64;
                let max_step_error = group
                    .windows(2)
                    .map(|pair| (pair[1] - pair[0] - pitch).abs())
                    .fold(0.0, f64::max);
                leaders.push(DotLeader {
                    start: group[0] - radius,
                    end: group[group.len() - 1] + radius,
                    y,
                    diameter: radius * 2.0,
                    pitch,
                    max_step_error,
                });
            }
            start = end;
        }
    }
    leaders
}
