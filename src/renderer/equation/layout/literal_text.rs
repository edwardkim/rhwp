//! 인용 문자열은 명령을 해석하지 않고 문자별 글꼴 계약을 유지한다.

use super::{variable_style, EqLayout, EqNode, LayoutBox, SpaceKind};

impl EqLayout {
    pub(super) fn layout_literal_text(&self, text: &str, fs: f64) -> LayoutBox {
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
