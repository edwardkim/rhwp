//! [#7865] 수식 스크립트의 `+-`·`-+` 축약은 ±·∓ 다.
//!
//! 한컴 수식 편집기는 `a +- b` 를 «a ± b», `a -+ b` 를 «a ∓ b» 로 읽는다. 종전
//! 토크나이저는 두 글자를 따로 떼어 `+`·`-` 두 기호로 그렸다. devel의 기호 토큰 계약을
//! 공개 API와 문서 경로가 공유한다. #7976의 종료 댓글과 기존 devel 계약에서 `+->`는
//! `±`·`>`, `<-+`는 `←`·`+`다. 기여 원본의 이웃 예외를 공개 API에 남기면 잘못된 해석이다.
//! 따옴표와 공백으로 떨어진 부호는 보존한다.

use rhwp::renderer::equation::ast::EqNode;
use rhwp::renderer::equation::parser::parse;
use rhwp::renderer::equation::tokenizer::{tokenize, TokenType};

fn values(script: &str) -> Vec<String> {
    tokenize(script)
        .into_iter()
        .filter(|t| t.ty != TokenType::Eof)
        .map(|t| t.value)
        .collect()
}

fn math_symbols(node: &EqNode, out: &mut Vec<String>) {
    match node {
        EqNode::MathSymbol(s) => out.push(s.clone()),
        EqNode::Row(children) => children.iter().for_each(|c| math_symbols(c, out)),
        EqNode::Fraction { numer, denom } => {
            math_symbols(numer, out);
            math_symbols(denom, out);
        }
        EqNode::Sqrt { body, .. } => math_symbols(body, out),
        EqNode::Superscript { base, sup } => {
            math_symbols(base, out);
            math_symbols(sup, out);
        }
        _ => {}
    }
}

#[test]
fn plus_minus_abbreviations_become_math_symbols() {
    assert_eq!(values("a +- b"), vec!["a", "±", "b"]);
    assert_eq!(values("a -+ b"), vec!["a", "∓", "b"]);
    let tokens = tokenize("a +- b");
    assert_eq!(
        tokens[1].ty,
        TokenType::Symbol,
        "±는 devel과 같은 기호 토큰이다: {tokens:?}"
    );
    let mut symbols = Vec::new();
    math_symbols(
        &parse("x = {-b +- sqrt {b^2 - 4ac}} over {2a}"),
        &mut symbols,
    );
    assert!(
        symbols.iter().any(|s| s == "±"),
        "근의 공식의 +- 가 ± 로 그려져야 한다: {symbols:?}"
    );
    let mut symbols = Vec::new();
    math_symbols(&parse("a -+ b"), &mut symbols);
    assert_eq!(symbols, vec!["∓".to_string()]);
}

#[test]
fn quoted_text_and_operator_neighbors_keep_hancom_order() {
    assert_eq!(values("\"+-\""), vec!["+-"]);
    assert_eq!(tokenize("\"+-\"")[0].ty, TokenType::Quoted);
    assert_eq!(values("a +-> b"), vec!["a", "±", ">", "b"]);
    assert_eq!(values("a <-+ b"), vec!["a", "<-", "+", "b"]);
    assert_eq!(values("a -+> b"), vec!["a", "∓", ">", "b"]);
    assert_eq!(values("a <+- b"), vec!["a", "<", "±", "b"]);
    assert_eq!(values("a ->+ b"), vec!["a", "->", "+", "b"]);
    // 공백으로 떨어진 두 부호는 여전히 두 기호다.
    assert_eq!(values("a + - b"), vec!["a", "+", "-", "b"]);
    assert_eq!(values("a +− b"), vec!["a", "+", "−", "b"]);
}
