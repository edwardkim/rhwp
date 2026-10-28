//! 독립 한컴 Print로 완료된 수식 문서의 실제 소유·기호·글리프 원점을 보호한다.
//! Native/fresh WASM 전 1쪽 raw: rm/±/그리스 문자 100%, fraction 99.56633/99.48605%,
//! state 99.68240/99.59905%. source 3cb5a4c290, print·96dpi·compat2022.
//! 독립 PDF의 글리프 origin을 72pt→96px로 환산한 값에만 ±2px를 사용한다.
//! 기존 구문 단위 검사와 달리 원본 파일을 열고 최종 EquationNode·LayoutBox를 검사한다.
//! 증적: mydocs/pr/assets/planet6897_completed_equation_pages_20261022/README.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::equation::symbols::FontStyleKind;
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

#[derive(Debug)]
struct Glyph {
    ch: char,
    x: f64,
    baseline: f64,
    style: Option<FontStyleKind>,
}

fn glyphs(value: &LayoutBox, x: f64, y: f64, style: Option<FontStyleKind>, out: &mut Vec<Glyph>) {
    let x = x + value.x;
    let y = y + value.y;
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            assert_eq!(run.glyphs.chars().count(), run.positions.len());
            for (ch, offset) in run.glyphs.chars().zip(&run.positions) {
                if !ch.is_whitespace() {
                    out.push(Glyph {
                        ch,
                        x: x + offset,
                        baseline: y + value.baseline + run.baseline_shift,
                        style,
                    });
                }
            }
        }
        LayoutKind::Text(text)
        | LayoutKind::Number(text)
        | LayoutKind::Symbol(text)
        | LayoutKind::MathSymbol(text)
        | LayoutKind::Function(text)
        | LayoutKind::FontGlyph { text, .. } => {
            for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
                out.push(Glyph {
                    ch,
                    x,
                    baseline: y + value.baseline,
                    style,
                });
            }
        }
        LayoutKind::Row(children) => {
            for child in children {
                glyphs(child, x, y, style, out);
            }
        }
        LayoutKind::Fraction { numer, denom, .. } => {
            glyphs(numer, x, y, style, out);
            glyphs(denom, x, y, style, out);
        }
        LayoutKind::Subscript { base, sub } => {
            glyphs(base, x, y, style, out);
            glyphs(sub, x, y, style, out);
        }
        LayoutKind::Superscript { base, sup } => {
            glyphs(base, x, y, style, out);
            glyphs(sup, x, y, style, out);
        }
        LayoutKind::SubSup { base, sub, sup } => {
            glyphs(base, x, y, style, out);
            glyphs(sub, x, y, style, out);
            glyphs(sup, x, y, style, out);
        }
        LayoutKind::FontStyle { style, body } => glyphs(body, x, y, Some(*style), out),
        LayoutKind::Color { body, .. } => glyphs(body, x, y, style, out),
        LayoutKind::Space(_) | LayoutKind::Empty => {}
        other => panic!("선택한 독립 Print 글줄에서 예상하지 않은 배치: {other:?}"),
    }
}

fn descendants<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        descendants(child, out);
    }
}

fn near(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "{label}: 독립 Print {expected:.5}px, 실제 {actual:.5}px, 허용 ±2px"
    );
}

fn with_document(
    input: &str,
    expected_count: usize,
    inspect: impl FnOnce(&[(&RenderNode, &EquationNode)]),
) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(input);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("커밋된 수식 입력")).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 1, "독립 Print의 전 1쪽");
    let source: Vec<_> = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .flat_map(|(pi, p)| {
            p.controls
                .iter()
                .enumerate()
                .filter_map(move |(ci, c)| match c {
                    Control::Equation(eq) => Some((pi, ci, eq.script.as_str())),
                    _ => None,
                })
        })
        .collect();
    assert_eq!(source.len(), expected_count, "독립 입력의 수식 수");
    let page = core.build_page_render_tree(0).unwrap();
    let mut nodes = Vec::new();
    descendants(&page.root, &mut nodes);
    let column = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Column(0)))
        .unwrap();
    let equations: Vec<_> = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Equation(eq) => Some((*n, eq)),
            _ => None,
        })
        .collect();
    assert_eq!(
        equations.len(),
        expected_count,
        "실제 그린 수식의 누락·중복 없음"
    );
    for ((node, eq), &(pi, ci, script)) in equations.iter().zip(&source) {
        assert_eq!(
            (eq.section_index, eq.para_index, eq.control_index),
            (Some(0), Some(pi), Some(ci)),
            "원본 문단·컨트롤 소유"
        );
        assert_eq!(eq.script, script, "원문·순서 보존");
        assert!(node.bbox.width > 0.0 && node.bbox.height > 0.0);
        assert!(node.bbox.x >= column.bbox.x && node.bbox.y >= column.bbox.y);
        assert!(node.bbox.y + node.bbox.height <= column.bbox.y + column.bbox.height);
    }
    assert!(
        equations.windows(2).all(|p| p[0].0.bbox.y < p[1].0.bbox.y),
        "원래 문서의 위아래 수식 순서"
    );
    inspect(&equations);
}

fn painted(node: &RenderNode, eq: &EquationNode) -> Vec<Glyph> {
    let mut result = Vec::new();
    glyphs(&eq.layout_box, node.bbox.x, node.bbox.y, None, &mut result);
    result
}

fn same_printed_letter(actual: char, expected: char) -> bool {
    // 독립 PDF의 HyhwpEQ는 기울임 라틴 글자를 이 PUA 구역으로 인코딩한다.
    actual == expected
        || (expected.is_ascii_lowercase() && actual as u32 == 0xe0e5 + expected as u32 - 'a' as u32)
        || (expected.is_ascii_uppercase() && actual as u32 == 0xe000 + expected as u32 - 'A' as u32)
}

fn one(all: &[Glyph], ch: char) -> &Glyph {
    let found: Vec<_> = all
        .iter()
        .filter(|g| same_printed_letter(g.ch, ch))
        .collect();
    assert_eq!(found.len(), 1, "최종 출력의 {ch} 누락·중복: {all:?}");
    found[0]
}

#[test]
fn roman_unit_document_preserves_the_printed_unit_origin() {
    with_document("samples/issue7873/eq-rm-quarterspace.hwpx", 2, |eqs| {
        let (node, eq) = eqs[0];
        assert_eq!(eq.script, "x=6 rm m");
        near(node.bbox.x, 104.0, "첫 수식 x");
        near(
            node.bbox.y + eq.layout_box.baseline,
            200.0,
            "첫 수식 기준선",
        );
        let all = painted(node, eq);
        let m = one(&all, 'm');
        assert_eq!(m.style, Some(FontStyleKind::Roman), "Print의 정체 단위 m");
        near(m.x, 135.68, "m 글리프 원점");
        near(m.baseline, 199.2, "m 기준선");
        assert!(
            !all.iter().any(|g| g.ch == '`'),
            "간격 토큰은 가시 글자가 아니다"
        );
        let (second_node, second_eq) = eqs[1];
        assert_eq!(
            second_eq.script,
            "{4 rm ``m} over {8 rm ``s} = {1} over {2} rm ``m/s"
        );
        let second = painted(second_node, second_eq);
        assert!(
            !second.iter().any(|g| g.ch == '`'),
            "분수의 명시 공백도 가시 문자가 아니다"
        );
        for (ch, origins) in [
            ('m', [(201.76, 314.24), (247.84, 322.88)]),
            ('s', [(204.16, 331.52), (266.88, 322.88)]),
        ] {
            let units: Vec<_> = second
                .iter()
                .filter(|g| same_printed_letter(g.ch, ch))
                .collect();
            assert_eq!(units.len(), 2, "분수·후속 단위 {ch} 소유와 누락·중복");
            for (glyph, (x, y)) in units.iter().zip(origins) {
                assert_eq!(glyph.style, Some(FontStyleKind::Roman));
                near(glyph.x, x, "분수·후속 단위 원점");
                near(glyph.baseline, y, "분수·후속 단위 기준선");
            }
        }
    });
}

#[test]
fn plus_minus_document_paints_one_of_each_sign_at_print_origins() {
    with_document("samples/issue7865/eq-plusminus.hwpx", 1, |eqs| {
        let (node, eq) = eqs[0];
        assert_eq!(eq.script, "x = a +- b -+ c");
        near(node.bbox.x, 172.32, "첫 수식 x");
        near(
            node.bbox.y + eq.layout_box.baseline,
            144.32,
            "첫 수식 기준선",
        );
        let all = painted(node, eq);
        for (ch, x) in [('±', 205.76), ('∓', 226.4)] {
            let g = one(&all, ch);
            near(g.x, x, "±/∓ 글리프 x");
            near(g.baseline, 144.32, "±/∓ 기준선");
        }
    });
}

#[test]
fn uppercase_greek_document_keeps_the_sigma_and_attached_subscripts() {
    with_document("samples/issue7390/eq-uppercase-greek.hwpx", 2, |eqs| {
        let (node, eq) = eqs[0];
        assert_eq!(eq.script, "SIGMA  _{k} y _{ik}");
        near(node.bbox.x, 104.0, "Σ 수식 원점");
        near(
            node.bbox.y + eq.layout_box.baseline,
            199.84,
            "Σ 수식 기준선",
        );
        let all = painted(node, eq);
        let sigma: Vec<_> = all
            .iter()
            .filter(|g| matches!(g.ch, 'Σ' | '\u{e096}' | '\u{1d6f4}'))
            .collect();
        assert_eq!(sigma.len(), 1, "Print의 대문자 Σ 한 개: {all:?}");
        near(sigma[0].x, 104.0, "Σ 글리프 원점");
        near(sigma[0].baseline, 199.84, "Σ 글리프 기준선");
        let ks: Vec<_> = all
            .iter()
            .filter(|g| same_printed_letter(g.ch, 'k'))
            .collect();
        assert_eq!(ks.len(), 2, "서로 다른 소유의 k 첨자 두 개");
        for (glyph, x) in ks.iter().zip([115.04, 130.08]) {
            near(glyph.x, x, "Σ/y 각각의 첨자 k 원점");
            near(glyph.baseline, 203.04, "첨자 k 기준선");
            assert!(glyph.baseline > sigma[0].baseline);
        }
        let i = one(&all, 'i');
        let y = one(&all, 'y');
        near(i.x, 127.04, "y의 첫 첨자 i 원점");
        near(i.baseline, 203.04, "i 기준선");
        near(y.x, 120.0, "두 첨자 소유자 y 원점");
        near(y.baseline, 199.84, "y 기준선");
        assert!(
            sigma[0].x < ks[0].x && ks[0].x < y.x && y.x < i.x && i.x < ks[1].x,
            "Σ의 k와 y의 ik가 다른 개체 순서·첨자 소속을 유지한다"
        );
        let (second_node, second_eq) = eqs[1];
        assert_eq!(
            second_eq.script,
            "{SIGMA  _{k} y _{ik}} over {SIGMA  _{k} N _{k}} = PI r ^{2} + pi"
        );
        let second = painted(second_node, second_eq);
        let capital: Vec<_> = second
            .iter()
            .filter(|g| matches!(g.ch, 'Π' | '\u{e094}' | '\u{1d6f1}'))
            .collect();
        let small: Vec<_> = second
            .iter()
            .filter(|g| matches!(g.ch, 'π' | '\u{e0ac}' | '\u{1d70b}'))
            .collect();
        assert_eq!(
            capital.len(),
            1,
            "PI가 대문자 Π 한 글리프로 표시된다: {second:?}"
        );
        assert_eq!(
            small.len(),
            1,
            "pi는 별도의 소문자 π 한 글리프로 표시된다: {second:?}"
        );
        near(capital[0].x, 240.48, "대문자 Π 원점");
        near(small[0].x, 279.04, "소문자 π 원점");
        for glyph in [capital[0], small[0]] {
            near(glyph.baseline, 326.24, "Π/π 기준선");
        }
        assert!(
            capital[0].x < small[0].x,
            "서로 다른 대소문자와 후속 항 순서"
        );
    });
}

#[test]
fn roman_fraction_document_keeps_numerator_denominator_and_following_term() {
    with_document(
        "mydocs/report/eq-rm-space-7873/eq-font-fraction.hwpx",
        16,
        |eqs| {
            let (node, eq) = eqs[0];
            assert_eq!(eq.script, "a over {rm b} c");
            let all = painted(node, eq);
            for (ch, x, y) in [
                ('a', 84.48, 124.16),
                ('b', 84.0, 141.44),
                ('c', 95.2, 132.64),
            ] {
                let g = one(&all, ch);
                near(g.x, x, "분자·분모·후속 항 x");
                near(g.baseline, y, "분자·분모·후속 항 기준선");
            }
            assert!(
                one(&all, 'a').baseline < one(&all, 'c').baseline
                    && one(&all, 'c').baseline < one(&all, 'b').baseline,
                "분자·후속 항·분모의 독립 Print 관계"
            );
            let (last_node, last_eq) = eqs[15];
            assert_eq!(last_eq.script, "a = {rm b} over {c}");
            let last = painted(last_node, last_eq);
            for (ch, x, y) in [
                ('a', 87.52, 703.68),
                ('b', 114.56, 694.08),
                ('c', 115.52, 712.16),
            ] {
                let glyph = one(&last, ch);
                near(glyph.x, x, "마지막 수식 분수 글리프 원점");
                near(glyph.baseline, y, "분수 뒤 누적 글줄 예약과 기준선");
            }
            assert_eq!(one(&last, 'b').style, Some(FontStyleKind::Roman));
            assert!(
                one(&last, 'b').baseline < one(&last, 'c').baseline,
                "마지막 분수의 정체 분자와 분모 소속"
            );
        },
    );
}

#[test]
fn font_state_document_preserves_roman_comma_spaces_and_following_letter() {
    with_document(
        "mydocs/report/eq-rm-space-7873/eq-font-state.hwpx",
        24,
        |eqs| {
            let (node, eq) = eqs[0];
            assert_eq!(eq.script, "rm A,````B");
            let all = painted(node, eq);
            for (ch, x) in [('A', 74.88), ('B', 97.44)] {
                let g = one(&all, ch);
                assert_eq!(g.style, Some(FontStyleKind::Roman));
                near(g.x, x, "정체 A/B 원점");
                near(g.baseline, 124.48, "정체 A/B 기준선");
            }
            assert!(
                one(&all, 'A').x < one(&all, 'B').x,
                "쉼표·명시 공백 뒤 B의 순서"
            );
            let (last_node, last_eq) = eqs[23];
            assert_eq!(last_eq.script, "a rm {b} c");
            let last = painted(last_node, last_eq);
            for (ch, x) in [('b', 80.64), ('c', 88.64)] {
                let glyph = one(&last, ch);
                assert_eq!(glyph.style, Some(FontStyleKind::Roman));
                near(glyph.x, x, "뒤 글줄의 정체 b/c 원점");
                near(glyph.baseline, 692.32, "정체 상태와 마지막 글줄 예약 높이");
            }
            assert!(
                one(&last, 'a').x < one(&last, 'b').x && one(&last, 'b').x < one(&last, 'c').x,
                "괄호 안 정체 상태 뒤 내용 순서"
            );
            assert!(
                !all.iter().any(|g| g.ch == '`'),
                "명시 간격은 문자로 출력하지 않는다"
            );
        },
    );
}
