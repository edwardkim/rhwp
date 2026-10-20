//! OOXML 차트의 선언 글꼴과 SVG 표시가 같은 측정 폭을 소비하게 한다.

use crate::ooxml_chart::OoxmlChart;
use crate::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
use crate::renderer::TextStyle;

pub(crate) fn render_ooxml_chart(chart: &OoxmlChart, bounds: [f64; 4]) -> String {
    chart.render_svg_with_text_measure(bounds, &|text, font_size, family| {
        let style = TextStyle {
            font_family: family.to_string(),
            font_size,
            // SVG 공백은 HWP 기본 반각이 아니라 선언 글꼴의 U+0020 글리프다.
            font_space_em: crate::renderer::style_resolver::declared_space_advance_em(
                family, false, false,
            ),
            ..TextStyle::default()
        };
        EmbeddedTextMeasurer.estimate_text_width(text, &style)
    })
}
