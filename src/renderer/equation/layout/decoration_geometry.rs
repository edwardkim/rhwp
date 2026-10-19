//! 장식과 글꼴 스타일의 측정 및 자식 원점.

use super::{EqLayout, EqNode, LayoutBox, LayoutKind};

impl EqLayout {
    pub(super) fn layout_decoration(
        &self,
        kind: super::super::symbols::DecoKind,
        body: &EqNode,
        fs: f64,
    ) -> LayoutBox {
        let b = self.layout_node(body, fs);
        if self.hancom_eqn && kind == super::super::symbols::DecoKind::Hat {
            return self.layout_hancom_hat(b, fs);
        }
        // 부정 빗금은 위쪽 악센트가 아니다. 독립 Print의 저장 프레임은
        // 본체와 같은 높이·기준선을 가지며 표시는 그 위에 겹친다.
        if kind == super::super::symbols::DecoKind::StrikeThrough {
            let negation = if self.hancom_math_face {
                super::super::negation_geometry::source_geometry(&b, self.dpi)
            } else {
                super::super::negation_geometry::geometry(&b)
            };
            return LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width: b.width,
                height: b.height,
                baseline: b.baseline,
                kind: LayoutKind::Decoration {
                    kind,
                    body: Box::new(b),
                    negation: Some(negation),
                    outline: None,
                },
            };
        }
        // BOX는 위쪽 장식이 아니라 내용 전체를 감싼다. 원본 한컴 Print의
        // 일반 글자 상자는 글자 높이에 양쪽 0.1em 여유를 더한 높이다.
        if kind == super::super::symbols::DecoKind::Box {
            let padding = fs * 0.1;
            let mut body_box = b;
            body_box.x = padding;
            body_box.y = padding;
            return LayoutBox {
                font_size: fs,
                x: 0.0,
                y: 0.0,
                width: body_box.width + 2.0 * padding,
                height: body_box.height + 2.0 * padding,
                baseline: padding + body_box.baseline,
                kind: LayoutKind::Decoration {
                    kind,
                    body: Box::new(body_box),
                    negation: None,
                    outline: None,
                },
            };
        }
        let deco_h = fs * 0.25;

        let mut body_box = b;
        body_box.y = deco_h;
        // 독립 정상 저장/Print의 단순·첨자·분수 본체 모두 같은 논리 em 여유를 쓴다.
        // 본체 원점과 장식의 중심을 함께 이동하며 글리프 크기는 바꾸지 않는다.
        let side_padding = if self.hancom_math_face
            && matches!(
                kind,
                super::super::symbols::DecoKind::Bar
                    | super::super::symbols::DecoKind::Vec
                    | super::super::symbols::DecoKind::Dyad
            ) {
            super::source_roman_advance::em_advance(fs, self.dpi).unwrap_or(fs) * 0.05
        } else {
            0.0
        };
        body_box.x = side_padding;
        let outline = self
            .hancom_math_face
            .then(|| super::super::accent_outline::commands(kind, &body_box, fs, self.dpi))
            .flatten();

        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: body_box.width + 2.0 * side_padding,
            height: body_box.height + deco_h,
            baseline: body_box.y + body_box.baseline,
            kind: LayoutKind::Decoration {
                kind,
                body: Box::new(body_box),
                negation: None,
                outline,
            },
        }
    }

    pub(super) fn layout_font_style(
        &self,
        style: super::super::symbols::FontStyleKind,
        body: &EqNode,
        fs: f64,
    ) -> LayoutBox {
        use super::super::symbols::FontStyleKind;
        let (variable_italic, variable_bold) = match style {
            FontStyleKind::Roman | FontStyleKind::SansSerif | FontStyleKind::Monospace => {
                (false, false)
            }
            FontStyleKind::Italic => (true, self.variable_bold),
            FontStyleKind::Bold => (self.variable_italic, true),
            FontStyleKind::Blackboard => (false, true),
            FontStyleKind::Calligraphy | FontStyleKind::Fraktur => (false, false),
        };
        let mut styled = Self {
            variable_italic,
            variable_bold,
            source_variable_glyphs: match style {
                FontStyleKind::Roman => true,
                FontStyleKind::Italic | FontStyleKind::Bold => self.source_variable_glyphs,
                _ => false,
            },
            ..*self
        };
        match style {
            FontStyleKind::Roman => styled.latin_italic = false,
            FontStyleKind::Italic => styled.latin_italic = true,
            _ => styled.hancom_latin = false,
        }
        let b = styled.layout_node(body, fs);
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: b.width,
            height: b.height,
            baseline: b.baseline,
            kind: LayoutKind::FontStyle {
                style,
                body: Box::new(b),
            },
        }
    }
}
