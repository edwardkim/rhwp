//! 독립 한컴 Print로 확인한 옛 글꼴 프로그램의 전진폭.
//!
//! HWP3 원본과 굵게 속성만 제거한 대조군의 세 크기(30/32/52pt)에서
//! 휴먼명조의 굵은 한글은 일반 전진보다 약 5% 넓다. 표시할 TTF의
//! 윤곽선 폭을 바꾸지 않고 글자 사이의 전진에만 반영한다.

use super::style_resolver::ResolvedStyleSet;
use super::TextStyle;
use crate::model::document::Document;

pub(crate) const HUMAN_HFT_METRIC_FACE: &str = "HumanMyeongJo-HFT";

pub(crate) fn select_native_programs(document: &Document, styles: &mut ResolvedStyleSet) {
    if document.provenance.format != crate::model::provenance::SourceFormat::Hwp3 {
        return;
    }
    for (shape, style) in document
        .doc_info
        .char_shapes
        .iter()
        .zip(&mut styles.char_styles)
    {
        for (lang, &font_id) in shape.font_ids.iter().enumerate() {
            let face = document
                .doc_info
                .font_faces
                .get(lang)
                .and_then(|faces| faces.get(font_id as usize));
            if face.is_some_and(|face| !face.is_embedded && face.name == "휴먼명조") {
                style.font_families_metric_face[lang] = Some(HUMAN_HFT_METRIC_FACE.to_string());
            }
        }
    }
}

/// HCI Tulip의 한컴 Type3 /Widths는 숫자 0~9 모두498/1000이다.
/// 확인하지 않은 글자는 기존 표시 대체 글꼴의 메트릭을 유지한다.
pub(crate) fn digit_width(face: &str, ch: char, font_size: f64) -> Option<f64> {
    (face == "HCI Tulip" && ch.is_ascii_digit()).then_some(font_size * 0.498)
}

pub(crate) fn measurement_face<'a>(declared: &'a str, display: &'a str) -> &'a str {
    match declared {
        HUMAN_HFT_METRIC_FACE => "HumanMyeongJo",
        "HCI Tulip" => display,
        _ => declared,
    }
}

pub(crate) fn bold_advance(style: &TextStyle, ch: char, ordinary_advance: f64) -> f64 {
    if style.bold
        && style.metric_font_family.as_deref() == Some(HUMAN_HFT_METRIC_FACE)
        && ('\u{AC00}'..='\u{D7A3}').contains(&ch)
    {
        ordinary_advance * 0.05
    } else {
        0.0
    }
}
