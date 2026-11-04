//! HYhwpEQ에 없는 미분 기호의 독립 Print 대체 face를 소비한다.
//! 원본 11월 Print의 Haansoft Batang subset과 설치 HBATANG.TTF는
//! U+2032의 hmtx 전진폭 276, UPM 1024가 같다. 글꼴 바이트·윤곽은 포함하지 않는다.

use super::{LayoutBox, LayoutKind};

/// 한컴 Print의 수학 전진폭과 자동 간격. 정체 ASCII 글꼴의 폭과 구분한다.
pub(super) fn operator_metrics(text: &str, fs: f64, dpi: f64) -> Option<(f64, f64)> {
    let (advance, padding) = match text {
        "+" => (795, 0.14),
        "-" | "−" => (774, 0.14),
        "=" => (795, 0.21),
        // 정상 한컴 생성본의 비례식·첨자 뒤 콜론은 관계 기호 간격을 소비한다.
        ":" => (283, 0.21),
        // 독립 Print의 정상/부정 부등호는 같은 글자 전진폭을 사용한다.
        // NOT 내부에서 대체 표시 글꼴의 폭과 연산자 여백을 함께 더하면
        // 다음 피연산자가 선언 프레임 밖으로 밀린다.
        "<" | ">" => (864, 0.21),
        "≤" | "≥" => (1024, 0.21),
        "≠" => (1024, 0.15),
        "×" => (831, 0.14),
        // 독립 6/9/12/18pt Print와 설치 HYhwpEQ의 UPM 1024 전진폭.
        "±" => (831, 0.14),
        "∓" => (1024, 0.14),
        // 독립 9/11/16/22pt Print: BULLET은 U+2219, CDOT은 U+22C5다.
        // 같은 논리 em 전진을 갖지만 CDOT에는 이항 자동 여백이 없다.
        "∙" => (1024, 0.14),
        "⋅" => (1024, 0.0),
        // slash는 이항 연산자 여백이 없는 수학 구분자다.
        "/" => (512, 0.0),
        // 수학 PUA U+E042는 853/1024다. 본문 ASCII %의 938/1024와 구분한다.
        "%" => (853, 0.0),
        _ => return None,
    };
    // 독립 800/1200/1600HU의 단독·분수·중첩 표현식에서 같은 논리 em
    // 격자를 소비한다. 명목 크기를 직접 쓰면 ×·−의 전진폭이 한 격자 어긋난다.
    let advance_size = super::source_roman_advance::em_advance(fs, dpi).unwrap_or(fs);
    Some((
        super::source_font_text::logical_advance(advance, advance_size, dpi),
        fs * padding,
    ))
}

pub(super) fn layout(text: &str, fs: f64, dpi: f64) -> Option<LayoutBox> {
    if matches!(text, "←" | "→") {
        let em = super::source_roman_advance::em_advance(fs, dpi).unwrap_or(fs);
        return Some(LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: super::source_font_text::logical_advance(1024, em, dpi),
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::MathSymbol(text.to_owned()),
        });
    }
    if matches!(text, "{" | "}") {
        return Some(LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            // 독립 Print의 보통 크기 중괄호와 설치 HYhwpEQ의 UPM 1024 전진폭.
            width: super::source_font_text::logical_advance(594, fs, dpi),
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::FontGlyph {
                text: text.to_string(),
                font_family: "HYhwpEQ, Latin Modern Math, STIX Two Math, serif",
                paint: None,
                outline: Some(super::super::brace_outline::commands(
                    fs,
                    dpi,
                    fs * 0.8,
                    text == "}",
                )),
            },
        });
    }
    if text == "%" {
        let (width, _) = operator_metrics(text, fs, dpi)?;
        return Some(LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height: fs,
            baseline: fs * 0.8,
            // 현대 Print의 수학 PUA다. 구형 Type3 원문 %나 본문 ASCII %와
            // 다른 실제 글리프를 같은 논리 전진 결과에 보존한다.
            kind: LayoutKind::FontGlyph {
                text: "\u{e042}".to_string(),
                font_family: "HYhwpEQ, Latin Modern Math, STIX Two Math, serif",
                paint: None,
                outline: None,
            },
        });
    }
    if matches!(text, "±" | "∓" | "∙" | "⋅") {
        let (width, _) = operator_metrics(text, fs, dpi)?;
        return Some(LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::FontGlyph {
                text: text.to_string(),
                // 측정에 쓴 face를 표시에도 사용한다. 설치되지 않은 환경은 공개 face로 대체한다.
                font_family: "HYhwpEQ, Latin Modern Math, STIX Two Math, serif",
                paint: None,
                outline: super::super::dot_geometry::commands(text, fs, dpi, fs * 0.8),
            },
        });
    }
    if matches!(text, "×" | "≠") {
        let (width, _) = operator_metrics(text, fs, dpi)?;
        let baseline = fs * 0.8;
        return Some(LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width,
            height: fs,
            baseline,
            kind: LayoutKind::FontGlyph {
                text: text.to_string(),
                font_family: "Latin Modern Math, STIX Two Math, serif",
                paint: None,
                outline: Some(if text == "×" {
                    super::super::multiplication_outline::commands(fs, width, baseline)
                } else {
                    super::super::not_equal_geometry::commands(fs, baseline)
                }),
            },
        });
    }
    if !matches!(text, "'" | "′") {
        return None;
    }
    Some(LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width: super::source_font_advance::measured(fs, dpi, 2).unwrap_or(fs * 276.0 / 1024.0),
        height: fs,
        baseline: fs * 0.8,
        kind: LayoutKind::FontGlyph {
            text: "′".to_string(),
            font_family: "Haansoft Batang, 한컴바탕, Batang, 바탕, serif",
            paint: None,
            outline: None,
        },
    })
}

/// 보통 굵기 수학 연산자의 논리 전진과 실제 잉크 원점을 함께 전달한다.
pub(super) fn layout_operator_outline(text: &str, fs: f64, dpi: f64) -> Option<LayoutBox> {
    let display = if text == "-" { "−" } else { text };
    let baseline = fs * 0.8;
    let commands = super::super::operator_outline::commands(display, fs, dpi, baseline)
        .or_else(|| super::super::relation_outline::commands(display, fs, dpi, baseline))?;
    let (width, _) = operator_metrics(display, fs, dpi)?;
    Some(LayoutBox {
        font_size: fs,
        x: 0.0,
        y: 0.0,
        width,
        height: fs,
        baseline,
        kind: LayoutKind::FontGlyph {
            text: display.to_string(),
            font_family: "Latin Modern Math, STIX Two Math, serif",
            paint: None,
            outline: Some(commands),
        },
    })
}
