//! SVG 선과 도형이 같은 반복 구간을 소비하도록 속성을 만든다.

use crate::renderer::StrokeDash;

pub(super) fn append_attrs(attrs: &mut String, dash: StrokeDash, width: f64) {
    let intervals = dash.intervals(width);
    if intervals.is_empty() {
        return;
    }
    let pattern = intervals
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    attrs.push_str(&format!(" stroke-dasharray=\"{pattern}\""));
    if dash.uses_round_cap() {
        attrs.push_str(" stroke-linecap=\"round\"");
    }
}
