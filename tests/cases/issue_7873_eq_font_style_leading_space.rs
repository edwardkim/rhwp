//! [#7873] 수식 글꼴 명령(`rm`·`it`)의 공백 처리와 적용 범위 계약.
//!
//! 한/글 2020 Print(`pdf/issue7873/eq-rm-quarterspace-2020.pdf`)는 `{4 rm ``m}` 을 «4 m»,
//! `rm ``m/s` 를 «m/s» 로 그린다 — 공백은 간격으로만 남고 단위 글자는 로만체다.
//! 종전 파서는 공백 토큰을 `rm` 의 body 로 삼아 역따옴표 글자를 찍고, 뒤의 단위 글자를
//! 기본 이탤릭으로 두었으며, `m/s` 의 `/s` 는 글꼴 밖에 남았다.
//!
//! 적용 범위는 같은 엔진의 합성 문서 실측(`mydocs/report/eq-rm-space-7873/eq-font-state-2020.pdf`·`eq-font-fraction-2020.pdf`,
//! 아래 각 스크립트가 한 문단씩)으로 정했다: `rm`/`it` 는 그 지점부터 수식 끝까지의
//! 상태 전환이며 `{}`·첨자·`LEFT…RIGHT`·`sqrt`·행렬·`cases` 경계를 넘어 지속된다.
//! 분수(`over`/`atop`)는 분모를 분자 직전 상태로 먼저 읽고, 그 뒤 분자, 그 뒤 후속
//! 원소 순으로 상태를 잇는다. `bold` 는 다음 원소 하나에만 적용되고, 그리스 문자는
//! `rm` 의 영향을 받지 않는다. 수정 전에는 첫 검사(역따옴표 글자 없음)에서 실패한다.

use rhwp::renderer::equation::ast::{EqNode, SpaceKind};
use rhwp::renderer::equation::parser::parse;
use rhwp::renderer::equation::symbols::FontStyleKind;

fn children(node: &EqNode) -> Vec<&EqNode> {
    match node {
        EqNode::Row(c) => c.iter().collect(),
        EqNode::FontStyle { body, .. }
        | EqNode::Decoration { body, .. }
        | EqNode::Color { body, .. }
        | EqNode::Paren { body, .. } => vec![&**body],
        EqNode::Fraction { numer, denom } => vec![&**numer, &**denom],
        EqNode::Atop { top, bottom } => vec![&**top, &**bottom],
        EqNode::Sqrt { index, body } => index
            .iter()
            .map(|b| &**b)
            .chain(std::iter::once(&**body))
            .collect(),
        EqNode::Superscript { base, sup } => vec![&**base, &**sup],
        EqNode::Subscript { base, sub } => vec![&**base, &**sub],
        EqNode::SubSup { base, sub, sup } => vec![&**base, &**sub, &**sup],
        EqNode::BigOp { sub, sup, .. } => sub.iter().chain(sup.iter()).map(|b| &**b).collect(),
        EqNode::Limit { sub, .. } => sub.iter().map(|b| &**b).collect(),
        EqNode::Matrix { rows, .. } => rows.iter().flatten().collect(),
        EqNode::Cases { rows } | EqNode::Pile { rows, .. } => rows.iter().collect(),
        EqNode::EqAlign { rows, .. } => rows.iter().flat_map(|(l, r)| [l, r]).collect(),
        EqNode::Rel { over, under, .. } => std::iter::once(&**over)
            .chain(under.iter().map(|b| &**b))
            .collect(),
        _ => vec![],
    }
}

fn walk(node: &EqNode, f: &mut dyn FnMut(&EqNode)) {
    f(node);
    for c in children(node) {
        walk(c, f);
    }
}

fn has_backtick_text(node: &EqNode) -> bool {
    let mut found = false;
    walk(node, &mut |n| {
        if let EqNode::Text(t) = n {
            found |= t.contains('`');
        }
    });
    found
}

/// 글자(`Text`)마다 그리기에 쓰이는 글꼴 — 가장 안쪽 글꼴 노드 — 을 문서 순서로 모은다.
/// 렌더러는 안쪽 `FontStyle` 이 바깥 것을 덮어쓰므로, 같은 글자를 감싼 바깥 묶음(현재
/// 그룹의 전환 묶음)은 세지 않는다.
fn effective_letters(node: &EqNode) -> Vec<(String, Option<FontStyleKind>)> {
    fn go(
        node: &EqNode,
        style: Option<FontStyleKind>,
        out: &mut Vec<(String, Option<FontStyleKind>)>,
    ) {
        match node {
            EqNode::Text(t) => out.push((t.clone(), style)),
            EqNode::FontStyle { style: s, body } => go(body, Some(*s), out),
            _ => children(node).into_iter().for_each(|c| go(c, style, out)),
        }
    }
    let mut out = Vec::new();
    go(node, None, &mut out);
    out
}

/// 가장 안쪽 글꼴이 `style` 인 글자.
fn styled_letters(node: &EqNode, style: FontStyleKind) -> Vec<String> {
    effective_letters(node)
        .into_iter()
        .filter(|(_, s)| *s == Some(style))
        .map(|(t, _)| t)
        .collect()
}

/// 어떤 글꼴 노드에도 감싸이지 않은 글자(기본 이탤릭).
fn plain_letters(node: &EqNode) -> Vec<String> {
    effective_letters(node)
        .into_iter()
        .filter(|(_, s)| s.is_none())
        .map(|(t, _)| t)
        .collect()
}

/// 글꼴 노드와 `Row` 를 펼친 문서 순서의 말단 노드.
fn flatten_leaves<'a>(node: &'a EqNode, out: &mut Vec<&'a EqNode>) {
    match node {
        EqNode::Row(children) => children.iter().for_each(|c| flatten_leaves(c, out)),
        EqNode::FontStyle { body, .. } => flatten_leaves(body, out),
        other => out.push(other),
    }
}

/// 첫 글자(`Text`) 바로 앞의 연속 Thin 공백 수. 공백이 글꼴 묶음 안팎 어디에 있든 센다.
fn thin_spaces_before_first_letter(node: &EqNode) -> Option<usize> {
    let mut seq = Vec::new();
    flatten_leaves(node, &mut seq);
    let at = seq.iter().position(|c| matches!(c, EqNode::Text(_)))?;
    Some(
        seq[..at]
            .iter()
            .rev()
            .take_while(|c| matches!(c, EqNode::Space(SpaceKind::Thin)))
            .count(),
    )
}

fn roman(script: &str) -> Vec<String> {
    styled_letters(&parse(script), FontStyleKind::Roman)
}

fn plain(script: &str) -> Vec<String> {
    plain_letters(&parse(script))
}

fn strs(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn quarter_spaces_after_rm_stay_spaces_and_unit_is_roman() {
    let ast = parse("4 rm ``m");
    assert!(
        !has_backtick_text(&ast),
        "공백 명령이 글자로 남으면 안 된다: {ast:?}"
    );
    assert_eq!(
        styled_letters(&ast, FontStyleKind::Roman),
        strs(&["m"]),
        "단위 m 이 로만체여야 한다: {ast:?}"
    );
    assert_eq!(
        thin_spaces_before_first_letter(&ast),
        Some(2),
        "``(1/4 공백 둘)이 글꼴 글자 앞 간격으로 보존된다: {ast:?}"
    );
}

#[test]
fn issue_fixture_units_are_roman_including_slash_tail() {
    let ast = parse("{4 rm ``m} over {8 rm ``s} = {1} over {2} rm ``m/s");
    assert!(!has_backtick_text(&ast), "{ast:?}");
    assert_eq!(
        styled_letters(&ast, FontStyleKind::Roman),
        strs(&["m", "s", "m", "s"]),
        "한/글은 m/s 전체를 로만체로 그린다: {ast:?}"
    );
    assert!(plain_letters(&ast).is_empty(), "{ast:?}");
}

#[test]
fn rm_state_persists_to_the_end_of_the_equation() {
    // 한/글 2020 Print 합성 실측(eq-font-state): 그룹·첨자·괄호·sqrt·행렬·cases 경계를 넘는다.
    assert_eq!(roman("{rm a} b"), strs(&["a", "b"]));
    assert_eq!(roman("rm a {b} c"), strs(&["a", "b", "c"]));
    assert_eq!(roman("{a rm b} c"), strs(&["b", "c"]));
    assert_eq!(roman("a rm {b} c"), strs(&["b", "c"]));
    assert_eq!(roman("a rm b ^{c} d"), strs(&["b", "c", "d"]));
    assert_eq!(roman("a rm b _{c} d"), strs(&["b", "c", "d"]));
    assert_eq!(roman("rm a sqrt {b}"), strs(&["a", "b"]));
    assert_eq!(roman("sqrt {rm a} b"), strs(&["a", "b"]));
    assert_eq!(roman("rm a LEFT ( b RIGHT ) c"), strs(&["a", "b", "c"]));
    assert_eq!(roman("LEFT ( rm a RIGHT ) b"), strs(&["a", "b"]));
    assert_eq!(roman("sigma ( rm X)=E(X)"), strs(&["X", "E", "X"]));
    assert_eq!(roman("rm A+B = c rm m s"), strs(&["A", "B", "c", "m", "s"]));
    assert_eq!(
        roman("TRIANGLE  rm PDA = TRIANGLE  QEC"),
        strs(&["PDA", "QEC"])
    );
    assert_eq!(
        roman("matrix{rm a & b # c & d} e"),
        strs(&["a", "b", "c", "d", "e"])
    );
    assert_eq!(roman("cases{rm a # b} c"), strs(&["a", "b", "c"]));
    // 상태 앞의 글자는 기본(이탤릭)이다.
    assert_eq!(plain("{a rm b} c"), strs(&["a"]));
    assert_eq!(plain("a rm b ^{c} d"), strs(&["a"]));
}

#[test]
fn fractions_read_the_denominator_before_the_numerator() {
    // 한/글 실측: `a over {rm b} c` 는 a·b·c 모두 로만체, `{rm a} over {b} c` 는 b 만 이탤릭.
    assert_eq!(roman("a over {rm b} c"), strs(&["a", "b", "c"]));
    assert_eq!(roman("{a} over {rm b} c"), strs(&["a", "b", "c"]));
    assert_eq!(roman("{rm a} over {b} c"), strs(&["a", "c"]));
    assert_eq!(plain("{rm a} over {b} c"), strs(&["b"]));
    assert_eq!(roman("{rm a} atop {b} c"), strs(&["a", "c"]));
    assert_eq!(plain("{rm a} atop {b} c"), strs(&["b"]));
    assert_eq!(roman("a = {rm b} over {c}"), strs(&["b"]));
    assert_eq!(plain("a = {rm b} over {c}"), strs(&["a", "c"]));
    assert_eq!(roman("rm a over b c"), strs(&["a", "b", "c"]));
    assert_eq!(roman("{rm a + b} over {c + d} e"), strs(&["a", "b", "e"]));
    assert_eq!(plain("{rm a + b} over {c + d} e"), strs(&["c", "d"]));
    assert_eq!(roman("{rm a} over {{b} over {c}} d"), strs(&["a", "d"]));
    assert_eq!(plain("{rm a} over {{b} over {c}} d"), strs(&["b", "c"]));
    assert_eq!(roman("rm a over {b over c} d"), strs(&["a", "b", "c", "d"]));
    // 분자를 다시 읽어도 구조는 같다.
    assert!(matches!(
        parse("a over {rm b} c"),
        EqNode::Row(ref items) if matches!(items[0], EqNode::Fraction { .. })
    ));
}

#[test]
fn it_switches_back_and_greek_is_untouched() {
    let ast = parse("rm a it b c");
    assert_eq!(styled_letters(&ast, FontStyleKind::Roman), strs(&["a"]));
    assert_eq!(
        styled_letters(&ast, FontStyleKind::Italic),
        strs(&["b", "c"])
    );
    let ast = parse("it A rm B it C");
    assert_eq!(
        styled_letters(&ast, FontStyleKind::Italic),
        strs(&["A", "C"])
    );
    assert_eq!(styled_letters(&ast, FontStyleKind::Roman), strs(&["B"]));
    // 제보 문서군의 `{rmV} LEFT ( itX` RIGHT )` — 괄호 안 `it` 가 X 를 되돌린다.
    let ast = parse("{rmV} LEFT ( itX` RIGHT ) ={rmE} LEFT (it X  ^{2} RIGHT )");
    assert_eq!(
        styled_letters(&ast, FontStyleKind::Roman),
        strs(&["V", "E"])
    );
    assert_eq!(
        styled_letters(&ast, FontStyleKind::Italic),
        strs(&["X", "X"])
    );
    // 그리스 문자는 기호이며 rm 의 영향을 받지 않는다(한/글 «αβx» 의 x 만 로만체).
    let ast = parse("rm alpha beta x");
    assert_eq!(styled_letters(&ast, FontStyleKind::Roman), strs(&["x"]));
    let mut symbols = Vec::new();
    walk(&ast, &mut |n| {
        if let EqNode::MathSymbol(s) = n {
            symbols.push(s.clone());
        }
    });
    assert_eq!(symbols, strs(&["α", "β"]));
}

#[test]
fn bold_applies_to_the_next_element_only() {
    // 한/글 «𝒂b»: bold 는 지속 상태가 아니다.
    let ast = parse("bold a b");
    assert_eq!(styled_letters(&ast, FontStyleKind::Bold), strs(&["a"]));
    assert_eq!(plain_letters(&ast), strs(&["b"]));
}

#[test]
fn plain_rm_body_is_unchanged() {
    // `rm abc` 의 글자는 모두 로만체로 그려진다.
    assert_eq!(roman("rm abc"), strs(&["abc"]));
    assert!(plain("rm abc").is_empty());
    assert_eq!(roman("x=6 rm m"), strs(&["m"]));
    assert_eq!(plain("x=6 rm m"), strs(&["x"]));
}

/// 문서 렌더 경로(저장 문서 수식 파서)에서 라벨별 수식의 이탤릭 글리프 수.
/// 이탤릭 글리프는 수식 글꼴의 사용자 정의 영역 글자이거나 SVG `transform` 기울임(`-0.333`)이다.
fn document_italic_counts() -> std::collections::BTreeMap<String, usize> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7873/eq-font-state-doc.hwpx");
    let core = rhwp::document_core::DocumentCore::from_bytes(&std::fs::read(path).expect("read"))
        .expect("parse");
    let svg = core.render_page_svg_native(0).expect("svg");
    let mut counts = std::collections::BTreeMap::new();
    let mut label: Option<String> = None;
    let mut previous_text = String::new();
    for chunk in svg.split("<text").skip(1) {
        let (attrs, rest) = chunk.split_once('>').unwrap_or((chunk, ""));
        let content = rest.split("</text>").next().unwrap_or("");
        if attrs.contains("font-size=\"12\"") {
            // 본문 라벨 «k.» — 글자 노드 뒤 «.» 노드가 온다.
            if content == "." {
                label = Some(previous_text.clone());
                counts.entry(previous_text.clone()).or_insert(0);
            }
            previous_text = content.to_string();
            continue;
        }
        if let Some(label) = &label {
            // 수식 글꼴의 이탤릭 소문자는 사용자 정의 영역 글리프로, 대문자 등은 기울임
            // 변환으로 그려진다. 로만체 글자는 ASCII 그대로다.
            let private_use = content
                .chars()
                .any(|ch| ('\u{E000}'..='\u{F8FF}').contains(&ch));
            if private_use || attrs.contains("matrix(1 0 -0.333 1") {
                *counts.get_mut(label).expect("label") += 1;
            }
        }
    }
    counts
}

#[test]
fn document_path_keeps_rm_state_across_groups_and_fractions() {
    // 정본 `pdf/issue7873/eq-font-state-doc-2024.pdf`(한/글 2024 Print) 판독 — 같은 스크립트의
    // 한/글 2020 Print(`mydocs/report/eq-rm-space-7873/eq-font-state-2020.pdf`)도 같다.
    let counts = document_italic_counts();
    for (label, script, italic) in [
        ("k", "{rm a} b", 0),
        ("m", "rm a {b} c", 0),
        ("n", "{rm a} over {b} c", 1),
        ("o", "it A rm B it C", 2),
        ("p", "rm a it b c", 2),
        ("r", "a rm b ^{c} d", 1),
        ("t", "{a rm b} c", 1),
        ("u", "rm a LEFT ( b RIGHT ) c", 0),
        ("y", "a rm {b} c", 1),
    ] {
        assert_eq!(
            counts.get(label).copied(),
            Some(italic),
            "{label}. `{script}` 의 이탤릭 글리프 수 — 전체 {counts:?}"
        );
    }
}
