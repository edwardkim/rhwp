//! [#5534] 수식 Thin 공백(`) + 첨자 계약.
//!
//! 한컴은 `a_{2}` 와 `` a`_{2} `` 를 다르게 렌더한다 — 공백이 있으면 첨자가
//! 1/4 공백만큼 오른쪽에 놓인다. 종전 파서는 첨자 앞 Thin 공백 토큰을 삭제해
//! (exam_math `log`_{2}` 첨자 파싱 실패의 과잉 교정) 두 스크립트가 같은 AST 가
//! 됐다. 수정 후: 첨자 결합은 유지하되 공백 폭을 base 뒤 `Space(Thin)` 으로
//! 보존한다 — AST 소비자도 두 표기를 구분할 수 있다.

use rhwp::renderer::equation::ast::{EqNode, SpaceKind};
use rhwp::renderer::equation::parser::parse;

fn base_of(node: &EqNode) -> Option<&EqNode> {
    match node {
        EqNode::Subscript { base, .. } => Some(base),
        EqNode::Superscript { base, .. } => Some(base),
        _ => None,
    }
}

fn base_ends_with_thin_space(node: &EqNode) -> bool {
    matches!(
        base_of(node),
        Some(EqNode::Row(children))
            if matches!(children.last(), Some(EqNode::Space(SpaceKind::Thin)))
    )
}

#[test]
fn thin_space_before_script_is_preserved_not_collapsed() {
    // 공백 유무는 다른 AST 다 — 첨자 결합은 양쪽 모두 유지.
    let tight = parse("a_{2}");
    let spaced = parse("a`_{2}");
    assert_ne!(spaced, tight, "Thin 공백의 정보/시각 폭이 사라지면 안 된다");
    assert!(base_of(&tight).is_some(), "a_{{2}} 는 첨자 결합");
    assert!(
        base_ends_with_thin_space(&spaced),
        "a`_{{2}} 는 base 꼬리에 Space(Thin) 을 보존한 첨자 결합이어야 한다: {spaced:?}"
    );
}

#[test]
fn function_script_keeps_binding_and_space_width() {
    // exam_math 원 결함(첨자 파싱 실패) 회귀 방지: log`_{2} 는 여전히 첨자 결합.
    let spaced = parse("log`_{2}");
    assert_ne!(spaced, parse("log_{2}"));
    assert!(
        base_ends_with_thin_space(&spaced),
        "log`_{{2}} 도 결합 + 공백 폭 보존이어야 한다: {spaced:?}"
    );
    // 함수 뒤 공백의 종전 소비(첨자가 아닐 때)는 불변 — Space 노드가 생기지 않는다.
    fn has_space(node: &EqNode) -> bool {
        match node {
            EqNode::Space(_) => true,
            EqNode::Row(children) => children.iter().any(has_space),
            _ => false,
        }
    }
    assert!(
        !has_space(&parse("log`x")),
        "함수명 직후 Thin 공백(비첨자)은 종전대로 소비된다"
    );
}

#[test]
fn superscript_thin_space_also_preserved() {
    let spaced = parse("x`^{2}");
    assert_ne!(spaced, parse("x^{2}"));
    assert!(base_ends_with_thin_space(&spaced), "{spaced:?}");
}
