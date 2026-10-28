//! Fullwidth source spaces must not acquire double-width saved character shapes.
//! Independent source: resource-limit.pdf p43 and Hancom 2020 one-up Print.
#![cfg(feature = "pdf-import")]
use rhwp::renderer::{
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    TextStyle,
};

#[test]
fn missing_ideographic_space_metric_keeps_fullwidth_without_changing_other_spaces() {
    let measurer = EmbeddedTextMeasurer;
    for family in [
        "__unregistered_space_control__",
        "HCRBatang",
        "Malgun Gothic",
    ] {
        let style = TextStyle {
            font_family: family.into(),
            font_size: 20.,
            ratio: 1.,
            ..Default::default()
        };
        let advance = |text: &str| measurer.compute_char_positions(text, &style)[1];
        assert!(
            (advance("\u{3000}A") - 20.).abs() < 0.02,
            "ideographic space: {family}"
        );
        assert!(
            (advance(" A") - advance("\u{00a0}A")).abs() < 0.02,
            "ASCII/NBSP retain their shared contract"
        );
        assert!(advance(" A") < advance("\u{3000}A"));
        assert!(
            (advance("\u{2007}A") - 5.).abs() < 0.02,
            "HWP figure space remains a quarter em"
        );
    }
}
