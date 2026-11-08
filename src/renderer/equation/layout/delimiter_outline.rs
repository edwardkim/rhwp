//! 구분자 선택·논리 슬롯이 정한 동일한 표시 윤곽을 각 backend에 전달한다.
use super::super::GlyphOutlineCommand;
use super::EqLayout;

impl EqLayout {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn delimiter_outline(
        &self,
        text: &str,
        width: f64,
        height: f64,
        baseline: f64,
        fs: f64,
        glyph: bool,
    ) -> Option<Vec<GlyphOutlineCommand>> {
        if matches!(text, "{" | "}") {
            Some(if self.hancom_math_face {
                super::super::brace_stretch::source_commands(
                    0.0,
                    0.0,
                    width,
                    height,
                    fs,
                    self.dpi,
                    text == "}",
                )
            } else {
                super::super::brace_stretch::commands(0.0, 0.0, width, height, fs, text == "}")
            })
        } else if self.hancom_math_face
            && text == "|"
            && super::source_font_advance::vertical_slot(fs, self.dpi).is_some()
        {
            super::super::relation_outline::vertical_delimiter_commands(height, fs)
        } else if self.hancom_math_face && glyph && matches!(text, "(" | ")") {
            super::super::relation_outline::commands(text, fs, self.dpi, baseline)
        } else if self.hancom_math_face && !glyph && matches!(text, "(" | ")") {
            super::super::round_stretch::source_commands(height, fs, self.dpi, text == ")").or_else(
                || {
                    super::super::relation_outline::small_round_commands(
                        height,
                        fs,
                        self.dpi,
                        text == ")",
                    )
                },
            )
        } else {
            None
        }
    }
}
