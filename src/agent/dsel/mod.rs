//! DSEL — 문서 선택자 언어.
//!
//! ## 무엇을 푸는가
//!
//! rhwp 의 편집 명령 여섯 개는 각자 다른 방식으로 대상을 지목한다 — 셀은
//! `--table/--row/--col`, 필드는 `--data 이름=값[k]`, 치환은 `--find/--occurrence`.
//! 지목 문법이 명령마다 다르므로 "3절 두 번째 표의 마지막 행"처럼 명령이 미리
//! 뚫어 두지 않은 대상은 **표현할 방법 자체가 없다**. DSEL 은 그 지목을 명령에서
//! 분리해 하나의 값으로 만든다.
//!
//! ```text
//!   section:nth(2) > table:last cell[row=-1]
//!   └──────┬─────┘   └───┬────┘ └────┬─────┘
//!      구역 지목      표 지목      셀 조건
//! ```
//!
//! ## CSS 를 닮게 만든 이유
//!
//! 새 문법을 발명하면 그 문법을 배우는 비용이 채택을 막는다. 선택자를 쓰는 쪽은
//! 대부분 언어모델이고, 모델은 CSS 선택자를 이미 안다. 결합자(` `, `>`, `+`, `~`),
//! 속성 술어(`[k=v]`), 의사 선택자(`:first`)를 같은 뜻으로 두면 문법 학습이
//! 사실상 0 이 된다. 반대로 **CSS 에 있지만 여기 없는 것**(`#id`, `.class`,
//! `::before`)은 문서 모델에 대응물이 없어서 뺐다 — 있는 척하면 그게 더 나쁘다.
//!
//! ## 문법 요약
//!
//! ```text
//! selector := path ("," path)*
//! path     := step (combinator step)*
//! comb     := " " (자손) | ">" (직계) | "+" (다음 형제) | "~" (이후 형제)
//! step     := axis predicate*
//! axis     := section|para|run|control|table|cell|picture|equation
//!           | field|footnote|endnote|header|footer|bookmark|hyperlink|shape|*
//! pred     := "[" name (op value)? "]" | ":" pseudo ("(" arg ")")?
//! op       := "=" | "!=" | ">" | "<" | ">=" | "<=" | "^=" | "$=" | "*=" | "~="
//! pseudo   := first|last|empty|nth(n)|range(a..b)|contains(s)|matches(s)|not(sel)|has(sel)
//! ```
//!
//! ## 신뢰 경계
//!
//! 선택자는 **문서 밖에서** 온다(사람·계획서·모델). 선택자가 맞대는 값은
//! **문서 안에서** 온다. 둘을 섞지 않는 것이 이 모듈의 규약이다 — 문서에서 읽은
//! 문자열이 선택자 문법으로 재해석되는 경로는 존재하지 않는다. 그래서 문서에
//! `para:has(...)` 라고 적혀 있어도 그건 그냥 글자다(`provenance` 가 말하는
//! "문서 파생 = 데이터, 지시 아님"과 같은 원칙).
//!
//! ## 정지성
//!
//! 파싱은 상한(길이·중첩·스텝·술어)으로 유계이고, 글롭 판정은 역추적 지점이
//! `*` 하나뿐이라 지수 경로가 없다. 손상·적대적 입력으로 파서나 판정기를 멈추게
//! 만드는 경로는 설계상 존재하지 않는다.

mod ast;
mod error;
mod eval;
mod glob;
mod lex;
mod node;
mod parse;
mod suggest;
mod token;

pub use ast::{
    AttrDef, AttrPred, AttrType, Axis, AxisKind, CmpOp, Combinator, Literal, Path, Pred, Pseudo,
    PseudoArity, PseudoDef, Selector, Step, AXIS_NAMES, COMMON_ATTRS, PSEUDO_DEFS,
};
pub use error::{SelectorError, SelectorErrorKind};
pub use eval::{count, select, select_with, EvalLimits};
pub use glob::Glob;
pub use node::{
    control_axis, control_kind_name, paragraphs_of_control, runs_of, Node, NodeId, NodeRef,
    NodeStep, PathStack, RunView,
};
pub use parse::{parse, MAX_NESTING, MAX_PATHS, MAX_PREDS, MAX_SOURCE_CHARS, MAX_STEPS};

/// 토큰 표면 — 문법 강조·편집기 지원이 쓴다. 커널 내부 소비자는 없다.
pub use token::{Tok, Token};

/// 렉싱만 수행한다 — 편집기가 문법 강조에 쓴다.
///
/// 파싱까지 하지 않는 이유: 편집 중인 선택자는 대개 문법적으로 미완성이라
/// (`para[te` 상태) 파싱은 실패하지만 강조는 되어야 한다.
pub fn tokenize(source: &str) -> Result<Vec<Token>, SelectorError> {
    lex::lex(source).map(|l| l.tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_surface_parses_a_realistic_selector() {
        let sel = parse("section:nth(2) > table:last cell[row=-1]").unwrap();
        assert_eq!(sel.paths.len(), 1);
        assert_eq!(sel.paths[0].steps.len(), 3);
        assert_eq!(sel.result_axes(), vec![Axis::Kind(AxisKind::Cell)]);
    }

    #[test]
    fn tokenize_survives_an_incomplete_selector() {
        // 파싱은 실패해도 렉싱은 되어야 편집기 강조가 산다.
        assert!(parse("para[te").is_err());
        assert!(tokenize("para[te").is_ok());
    }

    #[test]
    fn document_derived_text_is_never_reinterpreted_as_syntax() {
        // 문서에 선택자처럼 생긴 글자가 있어도 값으로만 쓰인다. 이 테스트는
        // 값 자리에 들어간 문법 문자열이 중첩 선택자로 파싱되지 않음을 고정한다.
        let sel = parse(r#"para[text="para:has(table)"]"#).unwrap();
        match &sel.paths[0].steps[0].preds[0] {
            Pred::Attr(a) => assert_eq!(
                a.compare,
                Some((CmpOp::Eq, Literal::Str("para:has(table)".into())))
            ),
            other => panic!("{other:?}"),
        }
        // 술어는 하나뿐이다 — 값 안의 `:has` 가 술어로 새지 않았다.
        assert_eq!(sel.paths[0].steps[0].preds.len(), 1);
    }
}
