//! 인용 문자열은 명령을 해석하지 않고 문자별 글꼴 계약을 유지한다.

use super::{variable_style, EqLayout, EqNode, LayoutBox, SpaceKind};

impl EqLayout {
    pub(super) fn layout_literal_text(&self, text: &str, fs: f64) -> LayoutBox {
        // 인용은 명령 해석을 막지만 원본 수식 face의 문자 프로그램은 보존한다.
        // 유효한 공통 run이 있는 경우만 재진입하므로 미지원 face/크기는 기존
        // 문자 경로를 유지하며 인용 밖 미분 기호로 다시 해석하지 않는다.
        let source_quote = text.contains('\'')
            && self.hancom_math_face
            && !self.legacy_math_face
            && !self.legacy_hft_metrics
            && self
                .layout_hancom_glyphs(
                    "'",
                    super::super::hancom_glyphs::Role::QuotedLiteral,
                    fs,
                    0.0,
                )
                .is_some();
        if source_quote {
            if let Some(layout) = self.layout_hancom_glyphs(
                text,
                super::super::hancom_glyphs::Role::QuotedLiteral,
                fs,
                0.0,
            ) {
                return layout;
            }
        }
        let nodes: Vec<EqNode> = text
            .chars()
            .map(|ch| {
                let literal = ch.to_string();
                // 명령·첨자로 재해석하지 않고 표시할 글리프 종류만 구분한다.
                if ch.is_ascii_alphabetic()
                    || variable_style::is_greek_variable(ch)
                    || super::super::text_has_cjk(&literal)
                {
                    EqNode::Text(literal)
                } else {
                    match ch {
                        // 혼합 인용 문자열도 기존 가로 줄 합성·문자 순서를 쓴다.
                        '\'' if source_quote => EqNode::Quoted(literal),
                        // 인용 안의 기호도 공통 수학 글리프·인접 간격을 소비한다.
                        '+' | '-' | '−' | '±' | '∓' | '=' | '<' | '>' | '×' | '÷' | '(' | ')'
                        | '/' => EqNode::Symbol(literal),
                        ' ' => EqNode::Space(SpaceKind::Thin),
                        _ => EqNode::Number(literal),
                    }
                }
            })
            .collect();
        self.layout_row(&nodes, fs)
    }
}
