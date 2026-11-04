//! 구형 문자와 명시적 그룹의 논리 전진을 한 번 계산한다.
//!
//! 문자별 16개 패턴·두 저장 너비, 독립 Print 256개:
//! e0aead61-b456-4887-afc9-f067bf1b3472. 여백은 root 여백 대비 비율이다.
//! 글꼴 전환·첨자·구조 연산자는 각자의 소유 계약을 확인하기 전에는 합치지 않는다.

use super::{advance, EqNode};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Edge {
    Character,
    Group,
    Operator(Operator),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operator {
    Relation,
    Binary,
}

impl Operator {
    fn before(self) -> f64 {
        match self {
            Self::Relation => 5.0 / 8.0,
            Self::Binary => 1.0 / 2.0,
        }
    }

    fn after(self) -> f64 {
        match self {
            Self::Relation => 3.0 / 4.0,
            Self::Binary => 1.0 / 2.0,
        }
    }
}

pub(super) struct Plan {
    pub width: f64,
    pub glyphs: Vec<(char, f64)>,
    first: Option<Edge>,
    last: Option<Edge>,
}

impl Plan {
    pub(super) fn source(node: &EqNode) -> Option<Self> {
        match node {
            EqNode::Text(text) | EqNode::Number(text) | EqNode::Quoted(text) => {
                let mut width = 0.0;
                let mut glyphs = Vec::new();
                for character in text.chars() {
                    glyphs.push((character, width));
                    width += advance(character)?;
                }
                let edge = (!glyphs.is_empty()).then_some(Edge::Character);
                Some(Self {
                    width,
                    glyphs,
                    first: edge,
                    last: edge,
                })
            }
            EqNode::Symbol(text) | EqNode::MathSymbol(text) => {
                let mut chars = text.chars();
                let character = chars.next()?;
                if chars.next().is_some() {
                    return None;
                }
                let width = advance(character)?;
                let edge = match character {
                    '=' => Edge::Operator(Operator::Relation),
                    '+' | '-' => Edge::Operator(Operator::Binary),
                    _ => Edge::Character,
                };
                Some(Self {
                    width,
                    glyphs: vec![(character, 0.0)],
                    first: Some(edge),
                    last: Some(edge),
                })
            }
            EqNode::Group(body) => {
                let mut inner = Self::source(body)?;
                // 빈 그룹도 논리 공간을 소유한다. 잉크가 없는지를 전진 없음으로
                // 대체하지 않으며, 뒤 문자는 이 예약을 실제로 소비한다.
                if inner.first.is_none() {
                    inner.width = 0.5;
                }
                for (_, x) in &mut inner.glyphs {
                    *x += 1.0;
                }
                Some(Self {
                    width: 1.0 + inner.width,
                    glyphs: inner.glyphs,
                    first: Some(Edge::Group),
                    last: Some(Edge::Group),
                })
            }
            EqNode::Row(items) => {
                let mut row = Self::empty();
                for item in items {
                    let child = Self::source(item)?;
                    let gap = match (row.last, child.first) {
                        (Some(Edge::Group), Some(_)) => 0.5,
                        // 독립 그룹/연산자 반례9개: 그룹 경계가 자동 여백보다
                        // 우선한다. 두 여백을 합산하면 같은 원점이 이중 이동한다.
                        (Some(_), Some(Edge::Group)) => -0.25,
                        (Some(Edge::Operator(operator)), Some(_)) => operator.after(),
                        (Some(_), Some(Edge::Operator(operator))) => operator.before(),
                        _ => 0.0,
                    };
                    row.width += gap;
                    row.glyphs.extend(
                        child
                            .glyphs
                            .into_iter()
                            .map(|(character, x)| (character, row.width + x)),
                    );
                    row.width += child.width;
                    row.first = row.first.or(child.first);
                    row.last = child.last.or(row.last);
                }
                Some(row)
            }
            EqNode::Empty => Some(Self::empty()),
            _ => None,
        }
    }

    fn empty() -> Self {
        Self {
            width: 0.0,
            glyphs: Vec::new(),
            first: None,
            last: None,
        }
    }
}
