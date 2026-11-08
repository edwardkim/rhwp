//! [#7390] 수식 스크립트의 전부 대문자 그리스 문자 이름(`SIGMA` 등)은 대문자 그리스 문자다.
//!
//! 한/글 2020 Print 실측(`pdf/issue7390/eq-uppercase-greek-2020.pdf`, 입력
//! `samples/issue7390/eq-uppercase-greek.hwpx`): `SIGMA _{k} y _{ik}` 는 「Σ_k y_ik」,
//! `sigma Sigma SIGMA` 는 「σ Σ Σ」, `PI r^{2} + pi` 는 「Π r² + π」다. 1790387 보고서 31쪽의
//! `{hat{d _{i}}} = N TIMES {SIGMA _{k} y _{ik}} over {SIGMA _{k} N _{k}}` 가 종전에는
//! 「SIGMA」 글자 그대로 찍혔다(#7390 병합 코멘트의 잔여 ①). 수정 전에는 첫 검사에서 실패한다.

use rhwp::renderer::equation::ast::EqNode;
use rhwp::renderer::equation::parser::parse;
use rhwp::renderer::equation::symbols::lookup_symbol;

fn children(node: &EqNode) -> Vec<&EqNode> {
    match node {
        EqNode::Row(c) => c.iter().collect(),
        EqNode::FontStyle { body, .. } | EqNode::Decoration { body, .. } => vec![&**body],
        EqNode::Fraction { numer, denom } => vec![&**numer, &**denom],
        EqNode::Superscript { base, sup } => vec![&**base, &**sup],
        EqNode::Subscript { base, sub } => vec![&**base, &**sub],
        EqNode::SubSup { base, sub, sup } => vec![&**base, &**sub, &**sup],
        EqNode::BigOp { sub, sup, .. } => sub.iter().chain(sup.iter()).map(|b| &**b).collect(),
        _ => vec![],
    }
}

fn leaves(node: &EqNode, out: &mut Vec<String>) {
    match node {
        EqNode::MathSymbol(s) => out.push(format!("sym:{s}")),
        EqNode::Text(t) => out.push(format!("text:{t}")),
        EqNode::BigOp { symbol, .. } => out.push(format!("bigop:{symbol}")),
        _ => {}
    }
    for c in children(node) {
        leaves(c, out);
    }
}

fn leaves_of(script: &str) -> Vec<String> {
    let mut out = Vec::new();
    leaves(&parse(script), &mut out);
    out
}

#[test]
fn all_caps_greek_names_are_uppercase_greek_letters() {
    assert_eq!(lookup_symbol("SIGMA"), Some("Σ"));
    assert_eq!(lookup_symbol("DELTA"), Some("Δ"));
    assert_eq!(lookup_symbol("OMEGA"), Some("Ω"));
    assert_eq!(lookup_symbol("PI"), Some("Π"));
    assert_eq!(lookup_symbol("XI"), Some("Ξ"));
    // 종전 표기는 그대로다.
    assert_eq!(lookup_symbol("sigma"), Some("σ"));
    assert_eq!(lookup_symbol("Sigma"), Some("Σ"));
    assert_eq!(lookup_symbol("pi"), Some("π"));
    assert_eq!(lookup_symbol("TIMES"), Some("×"));
    // 그리스 문자 이름이 아닌 대문자 식별자는 기호가 아니다.
    assert_eq!(lookup_symbol("ABC"), None);
    assert_eq!(lookup_symbol("N"), None);
}

#[test]
fn report_page31_sigma_subscript_is_a_symbol_not_text() {
    let leaves =
        leaves_of("{hat{d _{i}}} =N TIMES  {SIGMA  _{k} y _{ik}} over {SIGMA  _{k} N _{k}}");
    assert!(
        leaves.iter().filter(|l| l.as_str() == "sym:Σ").count() == 2,
        "SIGMA 둘은 Σ 기호여야 한다: {leaves:?}"
    );
    assert!(
        !leaves.iter().any(|l| l.contains("SIGMA")),
        "SIGMA 가 글자로 남으면 안 된다: {leaves:?}"
    );
    // 「Σ_k」: 첨자가 Σ 기호에 붙는다.
    let ast = parse("SIGMA  _{k} y _{ik}");
    let mut subs = Vec::new();
    fn find_sub(node: &EqNode, out: &mut Vec<String>) {
        if let EqNode::Subscript { base, .. } = node {
            if let EqNode::MathSymbol(s) = &**base {
                out.push(s.clone());
            }
        }
        for c in children(node) {
            find_sub(c, out);
        }
    }
    find_sub(&ast, &mut subs);
    assert_eq!(subs, vec!["Σ".to_string()], "{ast:?}");
}

#[test]
fn lowercase_and_capitalized_forms_and_sum_operator_are_unchanged() {
    assert_eq!(
        leaves_of("sigma Sigma SIGMA"),
        vec!["sym:σ", "sym:Σ", "sym:Σ"]
    );
    assert_eq!(
        leaves_of("PI r ^{2} + pi"),
        vec!["sym:Π", "text:r", "sym:π"]
    );
    // `sum` 은 큰 연산자 그대로다(한/글도 ∑ 아래 첨자).
    assert_eq!(leaves_of("sum _{k} y _{k}")[0], "bigop:∑");
}
