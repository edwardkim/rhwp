//! 독립 한컴 전진폭과 대조한 일반 수식 글자 줄의 캐시 재발행 범위.

use super::ast::{EqNode, SpaceKind};
use super::symbols::FontStyleKind;

/// 미지원 구조의 자연 폭을 원본 저장 폭의 정답으로 삼지 않는다.
pub(super) fn has_source_advances(node: &EqNode) -> bool {
    match node {
        EqNode::Text(text) => !text.is_empty() && text.bytes().all(|ch| ch.is_ascii_alphabetic()),
        EqNode::Number(text) => !text.is_empty() && text.bytes().all(|ch| ch.is_ascii_digit()),
        EqNode::Symbol(text) | EqNode::MathSymbol(text) => {
            matches!(
                text.as_str(),
                "(" | ")" | "'" | "′" | "+" | "=" | "×" | "∙" | "⋅"
            )
        }
        EqNode::Row(items) => items.iter().all(has_source_advances),
        EqNode::FontStyle {
            style: FontStyleKind::Roman | FontStyleKind::Italic,
            body,
        }
        | EqNode::Color { body, .. } => has_source_advances(body),
        EqNode::Space(SpaceKind::Normal | SpaceKind::Thin) | EqNode::Empty => true,
        _ => false,
    }
}
