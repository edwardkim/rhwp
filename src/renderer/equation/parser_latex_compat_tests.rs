//! LaTeX·한컴 수식 입력의 구문·피연산자 소유 계약.
//! 실제 화면의 픽셀 위치나 SVG 해시를 기대값으로 사용하지 않는다.

use super::symbols::{DecoKind, FontStyleKind};
use super::*;

#[test]
fn test_latex_dfrac_tfrac() {
    for cmd in [r"\dfrac{1}{2}", r"\tfrac{1}{2}"] {
        let ast = parse(cmd);
        assert!(
            matches!(&ast, EqNode::Fraction { .. }),
            "{cmd}: Expected Fraction, got {:?}",
            ast
        );
    }
}

#[test]
fn test_latex_mathrm() {
    let ast = parse(r"\mathrm{kg}");
    assert!(
        matches!(
            &ast,
            EqNode::FontStyle {
                style: FontStyleKind::Roman,
                ..
            }
        ),
        r"Expected FontStyle(Roman) for \mathrm, got {:?}",
        ast
    );
}

#[test]
fn test_latex_mathbf() {
    let ast = parse(r"\mathbf{F}");
    assert!(
        matches!(
            &ast,
            EqNode::FontStyle {
                style: FontStyleKind::Bold,
                ..
            }
        ),
        r"Expected FontStyle(Bold) for \mathbf, got {:?}",
        ast
    );
}

#[test]
fn test_latex_mathbb() {
    let ast = parse(r"\mathbb{R}");
    assert!(
        matches!(
            &ast,
            EqNode::FontStyle {
                style: FontStyleKind::Blackboard,
                ..
            }
        ),
        r"Expected FontStyle(Blackboard) for \mathbb, got {:?}",
        ast
    );
}

#[test]
fn test_latex_mathcal() {
    let ast = parse(r"\mathcal{L}");
    assert!(
        matches!(
            &ast,
            EqNode::FontStyle {
                style: FontStyleKind::Calligraphy,
                ..
            }
        ),
        r"Expected FontStyle(Calligraphy) for \mathcal, got {:?}",
        ast
    );
}

#[test]
fn test_latex_mathfrak_mathsf_mathtt() {
    let ast = parse(r"\mathfrak{g}");
    assert!(matches!(
        &ast,
        EqNode::FontStyle {
            style: FontStyleKind::Fraktur,
            ..
        }
    ));

    let ast = parse(r"\mathsf{AB}");
    assert!(matches!(
        &ast,
        EqNode::FontStyle {
            style: FontStyleKind::SansSerif,
            ..
        }
    ));

    let ast = parse(r"\mathtt{code}");
    assert!(matches!(
        &ast,
        EqNode::FontStyle {
            style: FontStyleKind::Monospace,
            ..
        }
    ));
}

#[test]
fn test_latex_text() {
    let ast = parse(r"\text{if }");
    assert!(
        matches!(
            &ast,
            EqNode::FontStyle {
                style: FontStyleKind::Roman,
                ..
            }
        ),
        r"Expected FontStyle(Roman) for \text, got {:?}",
        ast
    );
}

#[test]
fn test_latex_overline_lowercase() {
    let ast = parse(r"\overline{AB}");
    assert!(
        matches!(
            &ast,
            EqNode::Decoration {
                kind: DecoKind::Overline,
                ..
            }
        ),
        r"Expected Decoration(Overline) for \overline, got {:?}",
        ast
    );
}

#[test]
fn test_latex_underline_lowercase() {
    let ast = parse(r"\underline{x}");
    assert!(
        matches!(
            &ast,
            EqNode::Decoration {
                kind: DecoKind::Underline,
                ..
            }
        ),
        r"Expected Decoration(Underline) for \underline, got {:?}",
        ast
    );
}

#[test]
fn test_latex_widehat_widetilde() {
    let ast = parse(r"\widehat{ABC}");
    assert!(matches!(
        &ast,
        EqNode::Decoration {
            kind: DecoKind::Hat,
            ..
        }
    ));

    let ast = parse(r"\widetilde{x}");
    assert!(matches!(
        &ast,
        EqNode::Decoration {
            kind: DecoKind::Tilde,
            ..
        }
    ));
}

#[test]
fn test_latex_overrightarrow() {
    let ast = parse(r"\overrightarrow{AB}");
    assert!(matches!(
        &ast,
        EqNode::Decoration {
            kind: DecoKind::Vec,
            ..
        }
    ));
}

#[test]
fn test_latex_not_lowercase() {
    let ast = parse(r"\not{=}");
    assert!(
        matches!(
            &ast,
            EqNode::Decoration {
                kind: DecoKind::StrikeThrough,
                ..
            }
        ),
        r"Expected Decoration(StrikeThrough) for \not, got {:?}",
        ast
    );
}

#[test]
fn test_latex_quadratic_formula() {
    let ast = parse(r"x = \frac{-b \pm \sqrt{b^2 - 4ac}}{2a}");
    let s = format!("{:?}", ast);
    assert!(s.contains("Fraction"), "분수 있어야 함");
    assert!(s.contains("Sqrt"), "제곱근 있어야 함");
    assert!(s.contains("±"), "± 기호 있어야 함");
}

#[test]
fn latex_binomial_preserves_the_owned_choose_operands() {
    for (latex, hancom, top, bottom) in [
        (r"\binom{n}{k}", "n CHOOSE k", "n", "k"),
        (r"\binom{n+1}{k-1}", "{n+1} CHOOSE {k-1}", "n+1", "k-1"),
    ] {
        let ast = parse(latex);
        assert_eq!(ast, parse(hancom), "두 표기의 조합 의미가 같아야 한다");
        let EqNode::Binomial {
            top: upper,
            bottom: lower,
        } = ast
        else {
            panic!("조합의 위·아래 피연산자 소유를 보존해야 한다");
        };
        assert_eq!(*upper, parse(top), "위쪽 피연산자의 내용·순서 보존");
        assert_eq!(*lower, parse(bottom), "아래쪽 피연산자의 내용·순서 보존");
    }
}

#[test]
fn test_hwpeq_not_regressed() {
    assert!(matches!(parse("1 over 2"), EqNode::Fraction { .. }));
    assert!(matches!(parse("SQRT x"), EqNode::Sqrt { .. }));
    assert!(matches!(parse("SUM_{i=0}^n"), EqNode::BigOp { .. }));
    assert!(matches!(
        parse("rm abc"),
        EqNode::FontStyle {
            style: FontStyleKind::Roman,
            ..
        }
    ));
    assert!(matches!(parse("hat x"), EqNode::Decoration { .. }));
    assert!(matches!(
        parse("OVERLINE{abc}"),
        EqNode::Decoration {
            kind: DecoKind::Overline,
            ..
        }
    ));
}

#[test]
fn test_latex_begin_pmatrix() {
    let ast = parse(r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}");
    match &ast {
        EqNode::Matrix { rows, style } => {
            assert_eq!(*style, super::super::ast::MatrixStyle::Paren);
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 2);
            assert_eq!(rows[1].len(), 2);
        }
        _ => panic!("Expected Matrix(Paren), got {:?}", ast),
    }
}

#[test]
fn test_latex_begin_bmatrix() {
    let ast = parse(r"\begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix}");
    match &ast {
        EqNode::Matrix { rows, style } => {
            assert_eq!(*style, super::super::ast::MatrixStyle::Bracket);
            assert_eq!(rows.len(), 2);
        }
        _ => panic!("Expected Matrix(Bracket), got {:?}", ast),
    }
}

#[test]
fn test_latex_begin_cases() {
    let ast = parse(r"\begin{cases} x & x > 0 \\ -x & x \leq 0 \end{cases}");
    match &ast {
        EqNode::Cases { rows } => {
            assert_eq!(rows.len(), 2);
        }
        _ => panic!("Expected Cases, got {:?}", ast),
    }
}

#[test]
fn test_latex_begin_aligned() {
    let ast = parse(r"\begin{aligned} a &= b + c \\ d &= e \end{aligned}");
    match &ast {
        EqNode::EqAlign { rows, .. } => {
            assert_eq!(rows.len(), 2);
        }
        _ => panic!("Expected EqAlign, got {:?}", ast),
    }
}

#[test]
fn test_latex_backslash_backslash_tokenizes_as_newline() {
    use super::super::tokenizer::{tokenize, TokenType};
    let tokens = tokenize(r"a \\ b");
    let types: Vec<_> = tokens.iter().map(|t| t.ty).collect();
    assert!(
        types.contains(&TokenType::Whitespace),
        "\\\\는 Whitespace(#)로 토큰화돼야 함: {:?}",
        tokens
    );
}

#[test]
fn test_latex_operatorname() {
    let ast = parse(r"\operatorname{argmax}");
    match &ast {
        EqNode::FontStyle { style, body } => {
            assert_eq!(*style, FontStyleKind::Roman);
            assert!(format!("{:?}", body).contains("argmax"));
        }
        _ => panic!(
            r"Expected FontStyle(Roman) for \operatorname, got {:?}",
            ast
        ),
    }
}

#[test]
fn test_latex_spacing_quad() {
    let ast = parse(r"a \quad b");
    let s = format!("{:?}", ast);
    assert!(
        s.contains("Text") || s.contains("Symbol"),
        r"\quad should produce a text node: {}",
        s
    );
}

#[test]
fn test_latex_thin_space() {
    use super::super::tokenizer::{tokenize, TokenType};
    let tokens = tokenize(r"a \, b");
    let has_cmd = tokens
        .iter()
        .any(|t| t.ty == TokenType::Command && t.value == "THINSPACE");
    assert!(has_cmd, r"\, should tokenize as THINSPACE: {:?}", tokens);
}

#[test]
fn test_latex_rightarrow() {
    let ast = parse(r"\rightarrow");
    match &ast {
        EqNode::MathSymbol(s) => assert_eq!(s, "→"),
        _ => panic!(r"Expected → for \rightarrow, got {:?}", ast),
    }
}

#[test]
fn test_latex_implies() {
    let ast = parse(r"\implies");
    match &ast {
        EqNode::MathSymbol(s) => assert_eq!(s, "⇒"),
        _ => panic!(r"Expected ⇒ for \implies, got {:?}", ast),
    }
}

#[test]
fn test_latex_infty() {
    let ast = parse(r"\infty");
    match &ast {
        EqNode::MathSymbol(s) => assert_eq!(s, "∞"),
        _ => panic!(r"Expected ∞ for \infty, got {:?}", ast),
    }
}

#[test]
fn test_latex_nabla() {
    let ast = parse(r"\nabla");
    match &ast {
        EqNode::MathSymbol(s) => assert_eq!(s, "∇"),
        _ => panic!(r"Expected ∇ for \nabla, got {:?}", ast),
    }
}

#[test]
fn test_latex_leq_geq() {
    let ast = parse(r"a \leq b \geq c");
    let s = format!("{:?}", ast);
    assert!(
        s.contains("≤") && s.contains("≥"),
        r"\leq \geq should produce ≤ ≥: {}",
        s
    );
}

#[test]
fn test_latex_phantom() {
    let ast = parse(r"\phantom{x}");
    assert!(
        !matches!(ast, EqNode::Empty),
        r"\phantom should produce a space node, not Empty: {:?}",
        ast
    );
}

#[test]
fn test_latex_overset() {
    let ast = parse(r"\overset{n}{=}");
    match &ast {
        EqNode::Superscript { base, sup } => {
            assert!(format!("{:?}", base).contains("="));
            assert!(format!("{:?}", sup).contains("n"));
        }
        _ => panic!(r"Expected Superscript for \overset, got {:?}", ast),
    }
}

#[test]
fn test_latex_underset() {
    let ast = parse(r"\underset{x}{y}");
    match &ast {
        EqNode::Subscript { base, sub } => {
            assert!(format!("{:?}", base).contains("y"));
            assert!(format!("{:?}", sub).contains("x"));
        }
        _ => panic!(r"Expected Subscript for \underset, got {:?}", ast),
    }
}

#[test]
fn test_latex_stackrel() {
    let ast = parse(r"\stackrel{def}{=}");
    match &ast {
        EqNode::Superscript { sup, .. } => {
            let s = format!("{:?}", sup);
            assert!(
                s.contains("def"),
                r"\stackrel sup should contain 'def': {}",
                s
            );
        }
        _ => panic!(r"Expected Superscript for \stackrel, got {:?}", ast),
    }
}

#[test]
fn test_latex_begin_array() {
    let ast = parse(r"\begin{array} a & b \\ c & d \end{array}");
    match &ast {
        EqNode::Matrix { rows, .. } => {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 2);
        }
        _ => panic!(r"Expected Matrix for \begin{{array}}, got {:?}", ast),
    }
}

#[test]
fn test_latex_begin_smallmatrix() {
    let ast = parse(r"\begin{smallmatrix} 1 & 0 \\ 0 & 1 \end{smallmatrix}");
    match &ast {
        EqNode::Matrix { rows, .. } => {
            assert_eq!(rows.len(), 2);
        }
        _ => panic!(r"Expected Matrix for \begin{{smallmatrix}}, got {:?}", ast),
    }
}

#[test]
fn test_latex_begin_split() {
    let ast = parse(r"\begin{split} a &= b \\ c &= d \end{split}");
    match &ast {
        EqNode::EqAlign { rows, .. } => {
            assert_eq!(rows.len(), 2);
        }
        _ => panic!(r"Expected EqAlign for \begin{{split}}, got {:?}", ast),
    }
}

#[test]
fn test_latex_escaped_braces() {
    use super::super::tokenizer::{tokenize, TokenType};
    let tokens = tokenize(r"\left\{ x \right\}");
    let types: Vec<_> = tokens.iter().map(|t| t.ty).collect();
    assert!(
        types.contains(&TokenType::LBrace),
        r"\{{ should tokenize as LBrace: {:?}",
        tokens
    );
}

#[test]
fn test_latex_langle_rangle() {
    let ast = parse(r"\left\langle x \right\rangle");
    match &ast {
        EqNode::Paren { left, right, .. } => {
            assert_eq!(left, "⟨");
            assert_eq!(right, "⟩");
        }
        _ => panic!(r"Expected Paren for \langle, got {:?}", ast),
    }
}

#[test]
fn test_hwpeq_inf_remains_symbol() {
    let ast = parse("lim _{n→inf}");
    fn has_infinity(node: &EqNode) -> bool {
        match node {
            EqNode::MathSymbol(s) if s == "∞" => true,
            EqNode::Row(ch) => ch.iter().any(has_infinity),
            EqNode::Subscript { base, sub } => has_infinity(base) || has_infinity(sub),
            EqNode::Superscript { base, sup } => has_infinity(base) || has_infinity(sup),
            EqNode::Limit { sub, .. } => sub.as_ref().map_or(false, |s| has_infinity(s)),
            _ => false,
        }
    }
    assert!(
        has_infinity(&ast),
        "hwpeq 'inf' must produce ∞, not function text: {:?}",
        ast
    );
}

#[test]
fn test_hwpeq_deg_remains_symbol() {
    let ast = parse("90 deg");
    fn contains_degree(node: &EqNode) -> bool {
        match node {
            EqNode::MathSymbol(s) if s == "°" => true,
            EqNode::Row(children) => children.iter().any(contains_degree),
            _ => false,
        }
    }
    assert!(
        contains_degree(&ast),
        "hwpeq 'deg' must produce °, not function text: {:?}",
        ast
    );
}

// ── #1304: 무브레이스 첨자 공백 구분 ────────────────────────────────

/// 트리에서 첫 BigOp 노드를 찾는다.
fn find_big_op(node: &EqNode) -> Option<&EqNode> {
    match node {
        EqNode::BigOp { .. } => Some(node),
        EqNode::Row(children) => children.iter().find_map(find_big_op),
        _ => None,
    }
}

/// `sum_k=1 ^6` (브레이스 없음, 공백 구분) → ∑ 하한 `k=1` 전체, 상한 `6`.
/// 기존엔 하한이 `k` 하나로 잘리고 `=1` 이 본문으로, `^6` 이 `1` 의 위첨자로 깨졌다.
#[test]
fn task1304_unbraced_sum_lower_limit_full() {
    let ast = parse("sum_k=1 ^6 (k+1)^2");
    let big = find_big_op(&ast).expect("BigOp(∑) 가 있어야 함");
    let (sub, sup) = match big {
        EqNode::BigOp { symbol, sub, sup } => {
            assert_eq!(symbol, "∑");
            (sub.as_ref(), sup.as_ref())
        }
        _ => unreachable!(),
    };
    // 하한 = Row[k, =, 1]
    let sub = sub.expect("하한(sub)이 있어야 함");
    match sub.as_ref() {
        EqNode::Row(items) => {
            let dbg = format!("{:?}", items);
            assert!(
                items.len() == 3
                    && matches!(&items[0], EqNode::Text(t) if t == "k")
                    && matches!(&items[1], EqNode::Symbol(s) if s == "=")
                    && matches!(&items[2], EqNode::Number(n) if n == "1"),
                "하한은 k=1 전체여야 함: {dbg}"
            );
        }
        other => panic!("하한이 Row[k,=,1] 여야 함, got {:?}", other),
    }
    // 상한 = 6
    let sup = sup.expect("상한(sup)이 있어야 함");
    assert!(
        matches!(sup.as_ref(), EqNode::Number(n) if n == "6"),
        "상한은 6 이어야 함: {:?}",
        sup
    );
}

/// `lim_x->0` (공백 없음) → 극한 하한 `x->0` 전체 (x → 0).
#[test]
fn task1304_unbraced_lim_lower_limit_full() {
    let ast = parse("lim_x->0 f(x)");
    fn find_limit(node: &EqNode) -> Option<&EqNode> {
        match node {
            EqNode::Limit { .. } => Some(node),
            EqNode::Row(children) => children.iter().find_map(find_limit),
            _ => None,
        }
    }
    let lim = find_limit(&ast).expect("Limit 노드가 있어야 함");
    let sub = match lim {
        EqNode::Limit { sub, .. } => sub.as_ref().expect("하한이 있어야 함"),
        _ => unreachable!(),
    };
    match sub.as_ref() {
        EqNode::Row(items) => assert!(
            items.len() == 3
                && matches!(&items[0], EqNode::Text(t) if t == "x")
                && matches!(&items[1], EqNode::MathSymbol(s) if s == "→")
                && matches!(&items[2], EqNode::Number(n) if n == "0"),
            "극한 하한은 x→0 전체여야 함: {:?}",
            items
        ),
        other => panic!("극한 하한이 Row[x,→,0] 여야 함, got {:?}", other),
    }
}

/// 회귀: 브레이스 표기 `sum _{k=1} ^{6}` 는 기존대로 정상.
#[test]
fn task1304_braced_sum_unchanged() {
    let ast = parse("sum _{k=1} ^{6}");
    let big = find_big_op(&ast).expect("BigOp(∑)");
    if let EqNode::BigOp { sub, sup, .. } = big {
        let sub_dbg = format!("{:?}", sub);
        assert!(
            sub_dbg.contains("\"k\"") && sub_dbg.contains("\"=\"") && sub_dbg.contains("\"1\""),
            "브레이스 하한도 k=1 전체: {sub_dbg}"
        );
        assert!(
            matches!(sup.as_deref(), Some(EqNode::Number(n)) if n == "6"),
            "상한 6: {:?}",
            sup
        );
    }
}

/// 회귀: 공백 있는 등식 `x^2 = 4` 는 위첨자에 `=4` 가 흡수되면 안 된다 (sup=2).
#[test]
fn task1304_spaced_equation_not_merged() {
    let ast = parse("x^2 = 4");
    // 위첨자는 2 하나, 이후 = 4 는 본문(Row)에 남아야 한다.
    let dbg = format!("{:?}", ast);
    // Superscript{ base: x, sup: 2 } 형태가 존재하고, sup 안에 '=' 가 없어야 함.
    fn sup_has_no_equals(node: &EqNode) -> bool {
        match node {
            EqNode::Superscript { sup, .. } => !format!("{:?}", sup).contains("\"=\""),
            EqNode::Row(children) => children.iter().all(sup_has_no_equals),
            _ => true,
        }
    }
    assert!(
        sup_has_no_equals(&ast),
        "x^2 = 4 의 위첨자에 =4 가 흡수되면 안 됨: {dbg}"
    );
    assert!(dbg.contains("Superscript"), "x^2 위첨자 유지: {dbg}");
}

/// 회귀: 인접 식별자 `a_n b` 는 하한이 `n` 하나여야 한다 (b 흡수 금지).
#[test]
fn task1304_adjacent_identifier_not_merged() {
    let ast = parse("a_n b");
    let dbg = format!("{:?}", ast);
    fn first_subscript_sub<'a>(node: &'a EqNode) -> Option<&'a EqNode> {
        match node {
            EqNode::Subscript { sub, .. } => Some(sub.as_ref()),
            EqNode::Row(children) => children.iter().find_map(first_subscript_sub),
            _ => None,
        }
    }
    let sub = first_subscript_sub(&ast).expect("Subscript 가 있어야 함");
    assert!(
        matches!(sub, EqNode::Text(t) if t == "n"),
        "a_n b 의 하한은 n 하나여야 함 (b 흡수 금지): {dbg}"
    );
}

/// 독립 한컴 Print에서 중괄호 없는 n+1은 함께 아래첨자이며 중괄호로 범위를 닫는다.
#[test]
fn task1304_arithmetic_subscript_not_merged() {
    let ast = parse("a_n+1");
    fn first_subscript_sub<'a>(node: &'a EqNode) -> Option<&'a EqNode> {
        match node {
            EqNode::Subscript { sub, .. } => Some(sub.as_ref()),
            EqNode::Row(children) => children.iter().find_map(first_subscript_sub),
            _ => None,
        }
    }
    let sub = first_subscript_sub(&ast).expect("Subscript");
    let EqNode::Row(items) = sub else {
        panic!("중괄호 없는 아래첨자 n+1의 내용 순서: {sub:?}");
    };
    assert_eq!(items.len(), 3, "아래첨자의 누락·중복 금지");
    assert!(matches!(&items[0], EqNode::Text(text) if text == "n"));
    assert!(matches!(&items[1], EqNode::Symbol(text) if text == "+"));
    assert!(matches!(&items[2], EqNode::Number(text) if text == "1"));

    let braced = parse("a_{n}+1");
    assert!(matches!(first_subscript_sub(&braced), Some(EqNode::Text(text)) if text == "n"));
    let EqNode::Row(outer) = braced else {
        panic!("중괄호 뒤 +1은 아래첨자 바깥 본문에 남아야 함");
    };
    assert_eq!(outer.len(), 3, "아래첨자 뒤 본문의 소유와 순서");
    assert!(matches!(&outer[0], EqNode::Subscript { .. }));
    assert!(matches!(&outer[1], EqNode::Symbol(text) if text == "+"));
    assert!(matches!(&outer[2], EqNode::Number(text) if text == "1"));
}

// ── #1305: 괄호 그룹 뒤 위첨자 ─────────────────────────────────────

fn find_superscript(node: &EqNode) -> Option<&EqNode> {
    match node {
        EqNode::Superscript { .. } => Some(node),
        EqNode::Row(children) => children.iter().find_map(find_superscript),
        _ => None,
    }
}
fn find_subscript(node: &EqNode) -> Option<&EqNode> {
    match node {
        EqNode::Subscript { .. } => Some(node),
        EqNode::Row(children) => children.iter().find_map(find_subscript),
        _ => None,
    }
}
fn contains_paren(node: &EqNode) -> bool {
    match node {
        EqNode::Paren { .. } => true,
        EqNode::Row(children) => children.iter().any(contains_paren),
        _ => false,
    }
}

/// `(k+1)^2` → 위첨자가 Paren 그룹에 결합 (base 가 비지 않은 Superscript).
#[test]
fn task1305_paren_superscript_binds_to_group() {
    let ast = parse("(k+1)^2");
    let sup = find_superscript(&ast).expect("Superscript 가 있어야 함");
    match sup {
        EqNode::Superscript { base, sup } => {
            assert!(
                matches!(base.as_ref(), EqNode::Paren { .. }),
                "위첨자 base 가 Paren 그룹이어야 함(orphan Empty 금지): {:?}",
                base
            );
            assert!(
                matches!(sup.as_ref(), EqNode::Number(n) if n == "2"),
                "지수는 2: {:?}",
                sup
            );
        }
        _ => unreachable!(),
    }
}

/// `(k+1)_i` → 아래첨자도 Paren 그룹에 결합.
#[test]
fn task1305_paren_subscript_binds_to_group() {
    let ast = parse("(k+1)_i");
    let sub = find_subscript(&ast).expect("Subscript 가 있어야 함");
    if let EqNode::Subscript { base, .. } = sub {
        assert!(
            matches!(base.as_ref(), EqNode::Paren { .. }),
            "아래첨자 base 가 Paren 그룹이어야 함: {:?}",
            base
        );
    }
}

/// 회귀: 첨자 없는 일반 괄호는 Paren 그룹으로 묶이지 않는다 (느슨한 Symbol 유지).
#[test]
fn task1305_plain_paren_not_grouped() {
    let ast = parse("(k+1)");
    assert!(
        !contains_paren(&ast),
        "첨자 없는 괄호는 Paren 노드를 만들지 않아야 함: {:?}",
        ast
    );
    // a(b) 같은 함수꼴도 영향 없음
    let ast2 = parse("a(b)");
    assert!(!contains_paren(&ast2), "a(b) 도 Paren 미생성: {:?}", ast2);
}

/// 회귀: 숫자 base 위첨자 `7^2`, LEFT-RIGHT 그룹 첨자(#1226)는 정상 유지.
#[test]
fn task1305_regression_number_and_leftright_scripts() {
    let n = format!("{:?}", parse("7^2"));
    assert!(n.contains("Superscript"), "7^2 정상: {n}");
    let lr = format!("{:?}", parse("left ( x right ) ^2"));
    assert!(
        lr.contains("Superscript") && lr.contains("Paren") && !lr.contains("base: Empty"),
        "left(x)right^2 결합 정상: {lr}"
    );
}

// --- DoS 하드닝 회귀 (적대적 깊은 중첩/괄호 O(n^2)) ---

/// EqNode 트리의 최대 깊이(가드가 트리 깊이를 실제로 묶는지 검증용).
fn eq_depth(node: &EqNode) -> u32 {
    let kids: Vec<&EqNode> = match node {
        EqNode::Row(v) => v.iter().collect(),
        EqNode::Fraction { numer, denom } => vec![numer, denom],
        EqNode::Atop { top, bottom } | EqNode::Binomial { top, bottom } => vec![top, bottom],
        EqNode::Sqrt { index, body } => index
            .as_deref()
            .into_iter()
            .chain(std::iter::once(body.as_ref()))
            .collect(),
        EqNode::Superscript { base, sup } => vec![base, sup],
        EqNode::Subscript { base, sub } => vec![base, sub],
        EqNode::SubSup { base, sub, sup } => vec![base, sub, sup],
        EqNode::LeftScripts { base, sub, sup } => std::iter::once(base.as_ref())
            .chain(sub.as_deref())
            .chain(sup.as_deref())
            .collect(),
        EqNode::Paren { body, .. } => vec![body],
        EqNode::Decoration { body, .. } => vec![body],
        EqNode::FontStyle { body, .. } => vec![body],
        EqNode::Color { body, .. } => vec![body],
        EqNode::Rel { over, under, .. } => {
            let mut v = vec![over.as_ref()];
            if let Some(u) = under {
                v.push(u.as_ref());
            }
            v
        }
        EqNode::Matrix { rows, .. } => rows.iter().flatten().collect(),
        EqNode::Cases { rows } | EqNode::Pile { rows, .. } => rows.iter().collect(),
        _ => vec![],
    };
    1 + kids.iter().map(|k| eq_depth(k)).max().unwrap_or(0)
}

/// 적대적으로 깊은 중첩은 스택 오버플로/패닉 없이 파싱되고, 생성된 트리 깊이는
/// 상한 근방으로 묶여 재귀적 layout/svg/Drop 도 안전하다.
#[test]
fn dos_deep_nesting_is_bounded_not_overflow() {
    let cases = [
        "{".repeat(20000) + "x" + &"}".repeat(20000),
        "sqrt ".repeat(20000) + "x",
        "LEFT ( ".repeat(20000) + "x",
        "1 ".to_string() + &"over 1 ".repeat(20000),
        "cases{".repeat(20000) + "a" + &"}".repeat(20000),
        "pile{".repeat(20000) + "a" + &"}".repeat(20000),
        "(".repeat(20000) + "x" + &")".repeat(20000) + "^2",
    ];
    for s in cases {
        let ast = parse(&s); // 오버플로/패닉 없이 반환해야 함
        let d = eq_depth(&ast);
        assert!(
            d <= MAX_EQ_DEPTH + 8,
            "트리 깊이 {d} 가 상한 {MAX_EQ_DEPTH}(+여유) 를 넘었다 — 깊이 가드 회귀"
        );
        // layout 도 오버플로 없이 완료되고 유한한 크기를 낸다.
        let lb = super::super::layout::EqLayout::new(16.0).layout(&ast);
        assert!(
            lb.width.is_finite() && lb.height.is_finite(),
            "layout 크기가 유한해야 함"
        );
    }
}
