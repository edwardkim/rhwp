//! 원본 HFT 그룹·부착 첨자의 논리 여백은 표시 자식의 축소 크기와 구분한다.
//! 독립 정상 저장·Print의 170개 그룹과 50개 첨자 반례에서 같은 root 단위를 확인했다.

use super::{EqLayout, EqNode, FontGlyphPaint, LayoutBox, LayoutKind};

impl EqLayout {
    /// 원본 두 글줄 분수의 LEFT/RIGHT는 같은 괄호 프로그램을 세로로
    /// 확대한다. 일반 문자 괄호나 다른 표시 세대의 stretch path와 구분한다.
    pub(super) fn original_fraction_delimiters(
        &self,
        left: &str,
        right: &str,
        body: &LayoutBox,
        fs: f64,
    ) -> Option<LayoutBox> {
        if !self.legacy_math_face || (left, right) != ("(", ")") {
            return None;
        }
        let (height, baseline) = super::legacy_logical_frame(body, fs)?;
        // 인증된 두 논리 글줄 프레임만 소비한다. 단일 첨자의 잉크 높이를
        // 두 글줄로 오인하거나 미확인 중첩 프레임을 같은 값으로 축약하지 않는다.
        if (height - fs * 2.34).abs() > 1e-8 {
            return None;
        }
        // 기존 옛 편집기 구분자와 독립 6/9/12/18pt Print의 Tf/Tz 계약.
        // PDF 추출기의 기하평균 크기(약 1.421배)를 균일 배율로 쓰지 않는다.
        let glyph_size = fs * 2.03;
        let slot = fs * 0.375;
        let pad = fs * 0.115;
        let mut inner = body.clone();
        inner.x = slot + pad;
        inner.y = baseline - inner.baseline;
        let glyph_baseline = fs * 1.2 + glyph_size * super::AXIS_HEIGHT;
        let glyph = |text: &str, x| LayoutBox {
            font_size: glyph_size,
            x,
            y: 0.0,
            width: slot,
            height,
            baseline: glyph_baseline,
            kind: LayoutKind::FontGlyph {
                text: text.to_owned(),
                font_family: super::super::legacy_glyphs::FONT_FAMILY,
                outline: None,
                paint: Some(FontGlyphPaint {
                    scale_x: 0.49,
                    stroke_width: 0.0,
                }),
            },
        };
        let closing_x = inner.x + inner.width + pad;
        Some(LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: closing_x + slot,
            height,
            baseline,
            kind: LayoutKind::Row(vec![glyph(left, 0.0), inner, glyph(right, closing_x)]),
        })
    }

    pub(super) fn original_group_unit(&self) -> Option<f64> {
        self.legacy_math_face.then(|| {
            // 기존 HFT 논리 em과 같은 짝수 화면 화소 격자다. child의 0.64배
            // 표시 글자 크기로 그룹·첨자 프레임의 root 여백까지 축소하지 않는다.
            let logical_em = (self.font_size * 96.0 / self.dpi / 2.0).round() * 2.0;
            logical_em * self.dpi / 96.0 * 2.0 / 9.0
        })
    }

    pub(super) fn original_group_frame(&self, mut body: LayoutBox, empty: bool) -> LayoutBox {
        // 지원하는 구형 복합 분수는 legacy::row_children/fraction이 이미
        // 피연산자의 논리 여백을 소유한다. 같은 중괄호에 원본 CHAR 평면의
        // 그룹 여백까지 더하면 분자·분모와 분수선이 중복 확장된다.
        if self.legacy_equation {
            return body;
        }
        let Some(unit) = self.original_group_unit() else {
            return body;
        };
        body.x += unit;
        let width = unit + if empty { unit * 0.5 } else { body.width };
        LayoutBox {
            font_size: body.font_size,
            x: 0.0,
            y: 0.0,
            width,
            height: body.height,
            baseline: body.baseline,
            kind: LayoutKind::Row(vec![body]),
        }
    }

    pub(super) fn original_script_base(&self, mut body: LayoutBox, source: &EqNode) -> LayoutBox {
        let Some(unit) = self.original_group_unit() else {
            return body;
        };
        if matches!(source, EqNode::Group(_)) {
            body = self.original_group_frame(body, false);
        }
        // 첨자의 시작 간격은 base의 논리 종료가 소유한다. 위첨자 전용의
        // 대문자 여백을 다시 붙이지 않고 아래·위·병렬 슬롯이 같이 소비한다.
        body.width += unit * 0.5;
        body
    }

    pub(super) fn original_group_boundary_gap(&self, left: &EqNode, right: &EqNode) -> Option<f64> {
        let unit = self.original_group_unit()?;
        let opening = |node: &EqNode| matches!(node, EqNode::Symbol(text) if text == "(");
        if closing_character(left) && closing_character(right) {
            // 연속 닫는 문자는 같은 종료 밴드다.
            return Some(0.0);
        }
        if opening(left) {
            return Some(unit * if opening(right) { 1.0 } else { 0.5 });
        }
        super::super::legacy_coordinate_plane::group_boundary_gap(left, right)
            .map(|gap| gap * unit)
            .or_else(|| {
                if starts_round_delimiter(right) {
                    Some(unit * 0.75)
                } else if closing_character(left) {
                    Some(unit * 0.5)
                } else if closing_character(right) {
                    Some(unit * 0.25)
                } else {
                    None
                }
            })
    }

    pub(super) fn original_opening_inset(&self, source: &EqNode) -> f64 {
        if starts_round_delimiter(source) {
            self.original_group_unit().map_or(0.0, |unit| unit * 0.5)
        } else {
            0.0
        }
    }
}

fn starts_round_delimiter(node: &EqNode) -> bool {
    match node {
        EqNode::Symbol(text) => text == "(",
        EqNode::Paren { left, .. } => left == "(",
        EqNode::FontStyle { body, .. } | EqNode::Color { body, .. } => starts_round_delimiter(body),
        EqNode::Subscript { base, .. }
        | EqNode::Superscript { base, .. }
        | EqNode::SubSup { base, .. } => starts_round_delimiter(base),
        _ => false,
    }
}

fn closing_character(node: &EqNode) -> bool {
    match node {
        EqNode::Symbol(text) => text == ")",
        EqNode::FontStyle { body, .. } | EqNode::Color { body, .. } => closing_character(body),
        _ => false,
    }
}
