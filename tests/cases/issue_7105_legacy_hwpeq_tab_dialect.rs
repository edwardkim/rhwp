//! [#7105] 레거시 hwpeq5 OLE 수식(`\CMD 인자 \TAB`)이 `TAB` 세 글자로 찍히고 첨자·분수가
//! 무너진다.
//!
//! 신고 원본 `실험4 Transistor-MOSFET.hwp`(HWP 5.0.4.0)는 수식이 native `$eqed` 컨트롤이
//! 하나도 없고 **OLE 개체 22개**다. 한/글이 `hwpeq5X.ocx` 로 로드하는 5.x·97 계열 저장본이며,
//! BinData 안쪽 CFB 의 `Contents`(`Hwp 5.0 Equation Editor` 서명)에서 스크립트 42개가 나오고
//! 그중 30개가 이 방언을 쓴다 — `\TAB` 78 · `\BAR` 43 · `\SUB` 27 · `\CDOT` 26 · `\RTN` 4 ·
//! `\OVER` 3 · `\SUP` 1.
//!
//! # 기대값의 근거
//!
//! 신고자가 함께 올린 한/글 22 출력(`Hwp 2022 12.0.0.4204`) 2쪽을 `pdftotext -bbox` 로
//! 96dpi 환산해 재면
//!
//! | 대상 | 정본 |
//! | --- | --- |
//! | 본체 글리프 | y=464.0 h=16.8 (baseline ≈480.8) |
//! | `\SUB` 인자 | y=472.1 h=10.5 (baseline ≈482.6 — **+1.8px 아래, 더 작다**) |
//! | `\OVER` | y=518.0 / 528.0 / 536.0 세 단으로 쌓인다 |
//! | `TAB` 글리프 | **없다** |
//!
//! 곧 한/글은 `\TAB` 을 인자 종결자로 소비하고, `\SUB` 를 앞 피연산자에 붙는 진짜 첨자로,
//! `\OVER a \TAB b \TAB` 을 분수로 조판한다.
//!
//! 수정 전 rhwp 는 `\TAB` 을 식별자로 흘려 문서 전체에 `TAB` 리터럴 **33개**를 찍었고,
//! 그 때문에 조판 폭이 저장 OLE extent 의 2.3~2.9배가 되어 렌더러가 가로로 압착했다
//! (수식군 10개의 가로 배율 min 0.341 / 중앙 0.438 / max 0.690 → 수정 후 0.794 / 0.912 / 1.167).
//!
//! # 비범위
//!
//! `&=&` 정렬과 `\RTN` 이 함께 오는 eqalign 형태(원본 4개 스크립트)는 다루지 않는다.
//! `\RTN` 은 행 구분자로만 옮기고 eqalign 로 감싸지 않는다 — 감싸는 규칙의 독립 근거가 없다.
//! 신고의 증상 ②(개체 편집·속성·삭제 불가)도 별개 축이라 이 검사 밖이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::equation::ast::EqNode;
use rhwp::renderer::equation::parser::EqParser;
use rhwp::renderer::equation::tokenizer::tokenize;

fn parse(script: &str) -> EqNode {
    EqParser::new(tokenize(script)).parse()
}

/// 트리를 훑어 조건에 맞는 노드가 있는지.
fn any(node: &EqNode, pred: &mut impl FnMut(&EqNode) -> bool) -> bool {
    if pred(node) {
        return true;
    }
    let mut hit = false;
    visit_children(node, &mut |child| {
        if !hit && any(child, pred) {
            hit = true;
        }
    });
    hit
}

fn visit_children(node: &EqNode, f: &mut impl FnMut(&EqNode)) {
    match node {
        EqNode::Row(items) => items.iter().for_each(f),
        EqNode::Fraction { numer, denom } => {
            f(numer);
            f(denom);
        }
        EqNode::Atop { top, bottom } => {
            f(top);
            f(bottom);
        }
        EqNode::Superscript { base, sup } => {
            f(base);
            f(sup);
        }
        EqNode::Subscript { base, sub } => {
            f(base);
            f(sub);
        }
        EqNode::SubSup { base, sub, sup } => {
            f(base);
            f(sub);
            f(sup);
        }
        other => {
            // 나머지 변형은 자식이 없거나 이 검사에 필요 없다. Debug 출력으로 잡는다.
            let _ = other;
        }
    }
}

/// 트리 어디에도 `TAB` 이라는 글자 노드가 없어야 한다.
fn has_tab_literal(node: &EqNode) -> bool {
    let mut pred = |n: &EqNode| {
        matches!(
            n,
            EqNode::Text(s) | EqNode::Function(s) | EqNode::Symbol(s) | EqNode::MathSymbol(s)
                if s.eq_ignore_ascii_case("TAB")
        )
    };
    any(node, &mut pred)
}

/// 원본 42개 중 `\TAB` 방언을 쓰는 형태의 대표 표본.
const LEGACY_SCRIPTS: &[&str] = &[
    r"I \SUB E \TAB =I \SUB C \TAB +I \SUB B \TAB ",
    r"I \SUB C \TAB =hfe \CDOT I \SUB B \TAB ",
    r"V \SUB CC \TAB =R \SUB C \TAB  \CDOT I \SUB C \TAB +V \SUB CE \TAB ",
    r"V \SUB CE \TAB =V \SUB CC \TAB -R \SUB C \TAB  \CDOT I \SUB C \TAB =10-0.5K \CDOT 10m=5`V",
    r"I \SUB B \TAB = \OVER 1.7-0.7 \TAB 20K \TAB =0.05mA",
    r"I \SUB B \TAB = \OVER V \SUB BB \TAB -V \SUB BE \TAB  \TAB R \SUB B \TAB  \TAB ",
    r" \OVER 1 \TAB 2 \TAB Vmod",
    r" \BAR A \TAB +AB= \BAR A \TAB +B",
    r" \BAR  \BAR A \TAB  \TAB =A",
    r"I \SUB D \TAB =K \( V \SUB GS \TAB -V \SUB GS(TH) \TAB  \TAB  \SUP 2 \TAB ",
];

#[test]
fn issue_7105_legacy_tab_dialect_never_paints_a_tab_literal() {
    for script in LEGACY_SCRIPTS {
        let ast = parse(script);
        assert!(
            !has_tab_literal(&ast),
            "#7105: `\\TAB` 은 인자 종결자라 글자로 남으면 안 된다 — 정본 한/글 22 출력에 \
             TAB 글리프가 없다.\n  스크립트: {script:?}\n  AST: {ast:?}"
        );
    }
}

#[test]
fn issue_7105_legacy_sub_attaches_to_the_preceding_operand() {
    // `I \SUB E \TAB` 은 I 의 아래첨자 E 다. 수정 전에는 `SUB` 가 전위 명령으로 처리되어
    // I 와 E 가 같은 크기·같은 baseline 의 형제로 흩어졌다.
    let ast = parse(r"I \SUB E \TAB =I \SUB C \TAB +I \SUB B \TAB ");
    let mut subscripts = 0;
    let mut count = |n: &EqNode| {
        if matches!(n, EqNode::Subscript { .. } | EqNode::SubSup { .. }) {
            subscripts += 1;
        }
        false
    };
    any(&ast, &mut count);
    assert_eq!(
        subscripts, 3,
        "#7105: I_E = I_C + I_B — 아래첨자 3개여야 한다 (정본 2쪽 첨자 글리프 3개): {ast:?}"
    );
}

/// 노드가 품은 글자를 이어 붙인다 — 분수 피연산자를 좌표 없이 비교하기 위한 최소 평탄화.
fn flatten(node: &EqNode) -> String {
    let mut out = String::new();
    fn walk(n: &EqNode, out: &mut String) {
        match n {
            EqNode::Text(s)
            | EqNode::Number(s)
            | EqNode::Symbol(s)
            | EqNode::MathSymbol(s)
            | EqNode::Function(s) => out.push_str(s),
            other => visit_children(other, &mut |child| walk(child, out)),
        }
    }
    walk(node, &mut out);
    out
}

/// 첫 번째 분수의 (분자, 분모) 평탄화 문자열.
fn first_fraction(node: &EqNode) -> Option<(String, String)> {
    if let EqNode::Fraction { numer, denom } = node {
        return Some((flatten(numer), flatten(denom)));
    }
    let mut found = None;
    visit_children(node, &mut |child| {
        if found.is_none() {
            found = first_fraction(child);
        }
    });
    found
}

#[test]
fn issue_7105_legacy_over_builds_a_stacked_fraction() {
    // `\OVER 분자 \TAB 분모 \TAB` 은 분수다 — 정본은 y=518.0/528.0/536.0 세 단으로 쌓인다.
    //
    // 수정 전에도 `OVER` 자체는 중위 분수를 만들었지만 `\TAB` 이 피연산자에 섞여 들어가
    // 분모가 `TAB20KTAB=0.05mA` 처럼 뒤 내용을 통째로 삼켰다. 경계까지 검사한다.
    for (script, numer, denom) in [
        (
            r"I \SUB B \TAB = \OVER 1.7-0.7 \TAB 20K \TAB =0.05mA",
            "1.7-0.7",
            "20K",
        ),
        (r" \OVER 1 \TAB 2 \TAB Vmod", "1", "2"),
        (
            r"I \SUB B \TAB = \OVER V \SUB BB \TAB -V \SUB BE \TAB  \TAB R \SUB B \TAB  \TAB ",
            "VBB-VBE",
            "RB",
        ),
    ] {
        let ast = parse(script);
        let (got_numer, got_denom) = first_fraction(&ast)
            .unwrap_or_else(|| panic!("#7105: 분수가 있어야 한다: {script:?} → {ast:?}"));
        assert_eq!(
            (got_numer.as_str(), got_denom.as_str()),
            (numer, denom),
            "#7105: `\\TAB` 이 분자/분모 경계다: {script:?} → {ast:?}"
        );
    }
}

#[test]
fn issue_7105_current_syntax_scripts_are_untouched() {
    // 게이트는 `\TAB` 존재다. 현행 문법 스크립트(`fixtures/catalog.tsv` 머리말이
    // "Scripts must not contain TAB" 라고 못박은 그 집합)는 정규화를 지나가지 않는다.
    for script in [
        "x^{2} over {y}",
        "A CDOT 1=A",
        r"A \CDOT 1=A",
        "sqrt {a+b}",
        "V_{CC}",
        "bar {A}",
    ] {
        let ast = parse(script);
        assert!(
            !has_tab_literal(&ast),
            "#7105: 현행 문법에는 손대지 않는다: {script:?} → {ast:?}"
        );
    }
    // 현행 아래첨자는 그대로 붙는다.
    let ast = parse("V_{CC}");
    let mut pred = |n: &EqNode| matches!(n, EqNode::Subscript { .. });
    assert!(
        any(&ast, &mut pred),
        "#7105: 현행 `_` 첨자 계약 유지: {ast:?}"
    );
}
