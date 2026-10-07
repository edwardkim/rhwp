//! [#7593] 수식 `dyad`(양쪽 화살표 장식) 계약.
//!
//! 1. `rm dyadAB` 처럼 `dyad` 가 피연산자에 공백 없이 붙으면 `bar`·`vec` 과 같이
//!    키워드로 분리돼야 한다(#1204-E glued allowlist). 종전에는 `dyadAB` 가 글자
//!    한 덩이로 그려졌다.
//! 2. `dyad{AB}` 는 대상 폭 전체에 걸친 가로선 + 양 끝 화살촉으로 그려야 한다.
//!    종전에는 기본 분기(폭 절반 가로선, 화살촉 없음)로 떨어졌다.

use rhwp::renderer::equation::layout::EqLayout;
use rhwp::renderer::equation::parser::parse;
use rhwp::renderer::equation::svg_render::render_equation_svg;
use rhwp::renderer::equation::tokenizer::{tokenize, TokenType};

const FONT_SIZE: f64 = 20.0;

fn values(script: &str) -> Vec<String> {
    tokenize(script)
        .into_iter()
        .filter(|t| t.ty != TokenType::Eof)
        .map(|t| t.value)
        .collect()
}

fn svg_of(script: &str) -> String {
    let layout = EqLayout::new(FONT_SIZE).layout(&parse(script));
    render_equation_svg(&layout, "#000000", FONT_SIZE)
}

fn elements<'a>(svg: &'a str, tag: &str) -> Vec<&'a str> {
    svg.lines().filter(|l| l.starts_with(tag)).collect()
}

#[test]
fn glued_dyad_splits_like_vec() {
    assert_eq!(values("dyadAB"), vec!["dyad", "AB"]);
    assert_eq!(values("rm dyadAB"), vec!["rm", "dyad", "AB"]);
    assert_eq!(values("rmdyadAB"), vec!["rm", "dyad", "AB"]);
    // 공백형·단독형은 기존대로
    assert_eq!(values("dyad AB"), vec!["dyad", "AB"]);
    assert_eq!(values("dyad"), vec!["dyad"]);
}

#[test]
fn glued_dyad_renders_same_as_spaced() {
    assert_eq!(svg_of("rm dyadAB"), svg_of("rm dyad AB"));
}

#[test]
fn dyad_draws_full_width_line_with_heads_on_both_ends() {
    let vec_svg = svg_of("vec{rm AB}");
    let dyad_svg = svg_of("dyad{rm AB}");

    // 가로선: vec 과 같은 위치·폭(대상 폭 전체)
    let vec_lines = elements(&vec_svg, "<line");
    assert_eq!(vec_lines.len(), 1, "{vec_svg}");
    assert_eq!(
        elements(&dyad_svg, "<line"),
        vec_lines,
        "dyad 가로선은 vec 과 같이 대상 폭 전체여야 한다: {dyad_svg}"
    );

    // 화살촉: 오른쪽은 vec 과 동일, 왼쪽에 하나 더
    let vec_heads = elements(&vec_svg, "<path");
    let dyad_heads = elements(&dyad_svg, "<path");
    assert_eq!(vec_heads.len(), 1, "{vec_svg}");
    assert_eq!(dyad_heads.len(), 2, "dyad 는 양 끝 화살촉 2개: {dyad_svg}");
    assert!(
        dyad_heads.contains(&vec_heads[0]),
        "오른쪽 화살촉은 vec 과 같아야 한다: {dyad_svg}"
    );
}
