//! 독립 한컴 Print로 확인한 선언 글꼴과 꾸밈의 전진폭.
//!
//! HWP3 원본과 굵게 속성만 제거한 대조군의 세 크기(30/32/52pt)에서
//! 휴먼명조의 굵은 한글은 일반 전진보다 약 5% 넓다. 표시할 TTF의
//! 윤곽선 폭을 바꾸지 않고 글자 사이의 전진에만 반영한다.

use super::style_resolver::ResolvedStyleSet;
use super::TextStyle;
use crate::model::document::Document;
use crate::model::style::NativeFontProgram;

pub(crate) const HUMAN_HFT_METRIC_FACE: &str = "HumanMyeongJo-HFT";
pub(crate) const HANYANG_HFT_METRIC_FACE: &str = "한양신명조 (HFT)";
// 실제 표시 family나 파일 이름이 아닌, HWP 꾸밈의 측정 계약 식별자다.
const MALGUN_BOLD_DECORATION_METRIC_FACE: &str = "MalgunGothic-HancomBoldDecoration";

pub(crate) fn select_native_programs(document: &Document, styles: &mut ResolvedStyleSet) {
    use crate::model::provenance::SourceFormat;
    let source_format = document.provenance.format;
    if !matches!(
        source_format,
        SourceFormat::Hwp3 | SourceFormat::Hwp5 | SourceFormat::Hwpx
    ) {
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
            if let Some(face) = face.filter(|face| !face.is_embedded) {
                let metric_face = match face.native_program {
                    Some(NativeFontProgram::HumanMyeongJo) => Some(HUMAN_HFT_METRIC_FACE),
                    Some(NativeFontProgram::HanyangSinMyeongJo) => Some(HANYANG_HFT_METRIC_FACE),
                    None if source_format == SourceFormat::Hwp3 && face.name == "휴먼명조" => {
                        Some(HUMAN_HFT_METRIC_FACE)
                    }
                    None => None,
                };
                if let Some(metric_face) = metric_face {
                    style.font_families_metric_face[lang] = Some(metric_face.to_string());
                }
            }
            // 새 줄 구성에서 사용할 꾸밈 메트릭의 후보를 표시한다. 저장 줄의
            // 실제 굵은 전진에는 적용하지 않으며, 소비자가 줄 메트릭 출처를 함께 전달한다.
            // 독립 2020/2024 Print에서 굵은 숫자의 원점 간격은0.55em이고,
            // 실제 굵은 glyph의 PDF /Widths는0.579em이다. 괄호도 같은 분리다.
            // 명시한 Bold face·임베디드 프로그램·PDF 입력에는 이 계약을 옮기지 않는다.
            if matches!(source_format, SourceFormat::Hwp5 | SourceFormat::Hwpx)
                && shape.bold
                && !shape.italic
                && face.is_some_and(|face| {
                    !face.is_embedded
                        && matches!(
                            face.name.as_str(),
                            "맑은 고딕" | "Malgun Gothic" | "MalgunGothic"
                        )
                })
            {
                style.font_families_metric_face[lang] =
                    Some(MALGUN_BOLD_DECORATION_METRIC_FACE.to_string());
            }
        }
    }
}

pub(crate) fn measurement_face(declared: &str) -> &str {
    match declared {
        HUMAN_HFT_METRIC_FACE => "HumanMyeongJo",
        MALGUN_BOLD_DECORATION_METRIC_FACE => "Malgun Gothic",
        _ => declared,
    }
}

/// 표시 굵기는 유지하고 원본 꾸밈이 소유하는 전진폭의 굵기만 고른다.
pub(crate) fn measurement_bold(
    declared: &str,
    requested_bold: bool,
    regenerated_font_metrics: bool,
) -> bool {
    requested_bold && !(regenerated_font_metrics && declared == MALGUN_BOLD_DECORATION_METRIC_FACE)
}

/// 원 프로그램의 굵게 속성은 글리프 폭과 별개인 자간을 가진다.
/// 독립 Print의 Type3 /Widths는 일반 폭과 같고 TJ만 약0.05em 더 전진한다.
pub(crate) fn has_bold_tracking(style: &TextStyle) -> bool {
    style.bold && style.metric_font_family.as_deref() == Some(HANYANG_HFT_METRIC_FACE)
}

fn fixed_ascii_face(face: Option<&str>) -> bool {
    matches!(face, Some("고딕" | "명조" | "산세리프"))
}

pub(crate) fn has_fixed_ascii_bold_tracking(style: &TextStyle) -> bool {
    style.bold
        && !style.italic
        && (fixed_ascii_face(
            style
                .original_glyph_metric_family
                .as_deref()
                .map(String::as_str),
        ) || style.ascii_punct_latin.as_deref().is_some_and(|slot| {
            fixed_ascii_face(
                slot.original_glyph_metric_family
                    .as_deref()
                    .map(String::as_str),
            )
        }))
}

pub(crate) fn bold_advance(
    style: &TextStyle,
    ch: char,
    ordinary_advance: f64,
    em_advance: f64,
    original_glyph_face: Option<&str>,
) -> f64 {
    // 독립 ASCII95, 고딕/명조/산세리프의10/20pt Print에서 자연 폭은
    // 일반·굵게 모두0.5em이고 굵은 비공백 원점만 약0.05em 더 전진한다.
    // 실제 TTF 선택은 원본 부분 메트릭 이름이 없으므로 적용되지 않는다.
    if style.bold && !style.italic && fixed_ascii_face(original_glyph_face) && ch.is_ascii_graphic()
    {
        return em_advance * 0.05;
    }
    if has_bold_tracking(style) && !ch.is_whitespace() && !ch.is_control() {
        return em_advance * 0.05;
    }
    if style.bold
        && style.metric_font_family.as_deref() == Some(HUMAN_HFT_METRIC_FACE)
        && ('\u{AC00}'..='\u{D7A3}').contains(&ch)
    {
        ordinary_advance * 0.05
    } else {
        0.0
    }
}
