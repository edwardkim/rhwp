//! 부정 장식은 본체의 기준선 위에 겹치며 논리 줄 높이를 늘리지 않는다.
//!
//! 독립 한컴 Print는 관계 기호와 빗금을 같은 원점에 그린다. 평평한 등호와
//! 꺾인 부등호의 빗금 장평을 구분한다. 글리프 윤곽 대신 선분을 사용하며
//! 글자 크기와 논리 기준선으로 모든 backend의 표시 기하를 함께 정한다.

use super::layout::{LayoutBox, LayoutKind};

#[derive(Clone, Debug, serde::Serialize)]
pub struct NegationGeometry {
    pub start: (f64, f64),
    pub end: (f64, f64),
    pub thickness: f64,
}

/// 독립 Print의 600dpi 글꼴 원점 격자. 반환값은 현재 출력 DPI의 논리 길이다.
pub(super) fn math_origin_shift(fs: f64, dpi: f64) -> f64 {
    (fs * 600.0 / dpi * 0.05).round() * dpi / 600.0
}

fn ordinary_glyph(body: &LayoutBox) -> bool {
    match &body.kind {
        LayoutKind::Text(text) | LayoutKind::Number(text) => {
            text.chars().count() == 1 && !super::text_has_cjk(text)
        }
        LayoutKind::FontGlyph { text, .. } => {
            matches!(text.as_str(), "{" | "}")
                || (text.chars().count() == 1
                    && (text.chars().all(|ch| ch.is_ascii_alphabetic())
                        || super::layout::is_styled_variable(text)))
        }
        LayoutKind::Color { body, .. } | LayoutKind::FontStyle { body, .. } => ordinary_glyph(body),
        LayoutKind::Row(children) if children.len() == 1 => ordinary_glyph(&children[0]),
        _ => false,
    }
}

pub(super) fn source_geometry(body: &LayoutBox, dpi: f64) -> NegationGeometry {
    // 다른 관계 기호·복합 인자·본문 대체 face는 기존 기하를 보존한다.
    // 문자·숫자·보통 중괄호는 전진의 1.2배, 등호는 전진 전체를 소비한다.
    let ordinary = ordinary_glyph(body);
    let equality = matches!(&body.kind,
        LayoutKind::Symbol(text) | LayoutKind::FontGlyph { text, .. } if text == "=");
    if !ordinary && !equality {
        return geometry(body);
    }
    let fs = body.font_size;
    let natural_advance = (fs * 600.0 / dpi * 0.5).round();
    if natural_advance <= 0.0 {
        // 인쇄 글꼴 전진을 측정할 수 없는 극소 크기는 기존 표시 경로를 유지한다.
        return geometry(body);
    }
    let width_scale = if ordinary { 1.2 } else { 1.0 };
    let horizontal_scale = (body.width * 600.0 / dpi * width_scale).round() / natural_advance;
    let dx = 396.0_f64;
    let dy = 1029.0_f64;
    let length = dx.hypot(dy);
    let half_stroke = 22.0;
    let start = (
        57.0 + half_stroke * dy / length,
        206.0 - half_stroke * dx / length,
    );
    let end = (
        453.0 - half_stroke * dy / length,
        -823.0 + half_stroke * dx / length,
    );
    let base_dx = end.0 - start.0;
    let base_dy = end.1 - start.1;
    let origin_y = body.y + body.baseline - math_origin_shift(fs, dpi);
    NegationGeometry {
        start: (
            body.x + fs * horizontal_scale * start.0 / 1024.0,
            origin_y + fs * start.1 / 1024.0,
        ),
        end: (
            body.x + fs * horizontal_scale * end.0 / 1024.0,
            origin_y + fs * end.1 / 1024.0,
        ),
        // 선분의 수직 굵기도 인쇄 글리프의 가로 변환을 함께 소비한다.
        thickness: fs * 44.0 / 1024.0 * horizontal_scale * base_dx.hypot(base_dy)
            / (base_dx * horizontal_scale).hypot(base_dy),
    }
}

pub(crate) fn geometry(body: &LayoutBox) -> NegationGeometry {
    let horizontal_scale: f64 = if angular_relation(body) { 2.0 } else { 1.5 };
    let fs = body.font_size;
    // Print의 Tz는 끝점뿐 아니라 빗금의 획도 가로로 늘린다.
    // 기울어진 선의 수직 두께는 변환 전·후 길이와 행렬식으로 구한다.
    // 원 face의 수직 획 메트릭은 약 0.044em이며 글꼴 윤곽은 포함하지 않는다.
    let base_length = 0.36_f64.hypot(0.97);
    let stretched_length = (0.36 * horizontal_scale).hypot(0.97);
    // 표시 빗금의 끝은 논리 줄 상자 밖으로 나갈 수 있다. 본체와 같은
    // 기준선에서 겹쳐 그리며 상자 높이나 다음 문단의 위치를 늘리지 않는다.
    NegationGeometry {
        start: (
            body.x + fs * horizontal_scale * 0.07,
            body.y + body.baseline + fs * 0.18,
        ),
        end: (
            body.x + fs * horizontal_scale * 0.43,
            body.y + body.baseline - fs * 0.79,
        ),
        thickness: fs * 0.044 * horizontal_scale * base_length / stretched_length,
    }
}

fn angular_relation(body: &LayoutBox) -> bool {
    match &body.kind {
        LayoutKind::Symbol(text) | LayoutKind::MathSymbol(text) => {
            matches!(text.as_str(), "<" | ">" | "≤" | "≥")
        }
        LayoutKind::Color { body, .. } | LayoutKind::FontStyle { body, .. } => {
            angular_relation(body)
        }
        LayoutKind::Row(children) if children.len() == 1 => angular_relation(&children[0]),
        _ => false,
    }
}
