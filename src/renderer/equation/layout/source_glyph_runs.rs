//! 실제 수식 face의 글리프와 전진을 측정 결과에 보존한다.
use super::{estimate_text_width, glyph_advances, EqLayout, LayoutBox, LayoutKind};

impl EqLayout {
    /// 독립 Print의 HYhwpEQ·HancomEQN 첨자는 명목 크기의 0.68배다.
    /// 다른 face의 계약과 실제 자식 상자의 글자 크기는 보존한다.
    pub(super) fn script_font_size(&self, fs: f64) -> f64 {
        fs * if self.hancom_math_face || self.hancom_eqn {
            0.68
        } else {
            super::SCRIPT_SCALE
        }
    }

    /// 정상 한컴 Print의 네 크기에서 본문 기준선과 분수선 중심을 대조한 축.
    /// HYhwpEQ의 0.3285em과 HancomEQN의 실제 분수선 중심을 구분한다.
    /// 원본의 논리 축과 표시 글꼴의 TeX 축을 서로 대신하지 않는다.
    pub(super) fn math_axis_height(&self, fs: f64) -> f64 {
        fs * if self.hancom_eqn {
            // HancomEQN 분수선의 bbox 중심 619/1024와 원래 0.3em 원점.
            619.0 / 1024.0 - 0.3
        } else if self.hancom_math_face {
            0.3285
        } else {
            super::AXIS_HEIGHT
        }
    }

    pub(super) fn layout_hancom_glyphs(
        &self,
        text: &str,
        role: super::super::hancom_glyphs::Role,
        fs: f64,
        pad: f64,
    ) -> Option<LayoutBox> {
        if !self.hancom_latin {
            return None;
        }
        let family = if self.hancom_eqn {
            "HancomEQN"
        } else {
            super::super::hancom_glyphs::FONT_FAMILY
        };
        let run = super::super::hancom_glyphs::measure(text, role, fs, self.dpi, pad, family)?;
        Some(LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: run.width,
            height: fs,
            baseline: fs * super::TEXT_BASELINE,
            kind: LayoutKind::HancomGlyphs(run),
        })
    }

    pub(super) fn layout_function(&self, name: &str, fs: f64) -> LayoutBox {
        if self.hancom_eqn {
            if let Some(body) = self.layout_hancom_glyphs(
                name,
                super::super::hancom_glyphs::Role::Variable { italic: false },
                fs,
                0.0,
            ) {
                return body;
            }
        }
        if self.hancom_math_face && !self.variable_bold {
            if let Some(body) = super::source_font_text::layout(
                name,
                fs,
                self.dpi,
                false,
                false,
                self.source_variable_glyphs,
            ) {
                return LayoutBox {
                    font_size: fs,
                    x: 0.0,
                    y: 0.0,
                    width: body.width,
                    height: body.height,
                    baseline: body.baseline,
                    kind: LayoutKind::FontStyle {
                        style: super::FontStyleKind::Roman,
                        body: Box::new(body),
                    },
                };
            }
        }
        let w = glyph_advances::text_width(name, fs)
            .unwrap_or_else(|| estimate_text_width(name, fs, false));
        LayoutBox {
            font_size: fs,
            x: 0.0,
            y: 0.0,
            width: w + fs * 0.02,
            height: fs,
            baseline: fs * 0.8,
            kind: LayoutKind::Function(name.to_string()),
        }
    }
}
