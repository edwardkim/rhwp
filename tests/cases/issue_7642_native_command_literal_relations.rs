//! 검사 유형: 관계형은 명령·인용 문자·glyph/face·원본 전진·내용 소유를 비교한다.
//! 고정형은 독립 쪽수·수식 개수와 아래 3쪽 Print 원점의 ±2px 검사다.
//! 주석 갱신: 2026-11-06 04:13:18 KST.
//! 직전 파일 변경: 8f8bda282f 2026-10-18T17:35:33+09:00.
//! 새 base 170b886f16의 96개 수식·독립 한컴 2020 Print 전3쪽, canonical 96DPI/compat2022/2px raw.
//! 공통 투영 보정 전 Native 99.95137/99.94100/96.16576%, WASM 99.95196/99.95533/96.26274%.
//! 보정 후 Native 100/99.98115/99.95925%, fresh WASM 100/99.99209/100%, 전3쪽 누락0.
//! parent ae80442566 위 공통 glyph 보정 4파일의 출력이며, 양쪽 98%와 직접 판독 뒤 px를 고정한다.
//! 기대 좌표는 독립 Print SHA 12057cf4571d의 글리프 원점(72pt)을 96px로 환산한 값이다.
//! 기존 1쪽 명령 최장 일치·문자 범위 검사는 유지한다. 새 진단과 기존 입력의 범위를 섞지 않는다.
//! 근거: mydocs/plans/task_m100_7642_glyph_projection_20261106.md 및 fixture PRIME_WIDTH_DIAGNOSTIC.md.
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};

fn equations(node: &RenderNode, out: &mut Vec<EquationNode>) {
    if let RenderNodeType::Equation(eq) = &node.node_type {
        out.push(eq.clone());
    }
    for child in &node.children {
        equations(child, out);
    }
}

fn load(stem: &str, expected: usize) -> Vec<EquationNode> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-structure-argument-boundary-diagnostic")
        .join(format!("{stem}-hancom-2020.hwp"));
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(core.page_count(), 1, "독립 Print 한 쪽과 같은 내용 범위");
    let mut rendered = Vec::new();
    equations(&core.build_page_render_tree(0).unwrap().root, &mut rendered);
    let mut source_count = 0;
    for (si, section) in core.document().sections.iter().enumerate() {
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, control) in para.controls.iter().enumerate() {
                if let Control::Equation(source) = control {
                    source_count += 1;
                    let owned: Vec<_> = rendered
                        .iter()
                        .filter(|eq| {
                            (eq.section_index, eq.para_index, eq.control_index)
                                == (Some(si), Some(pi), Some(ci))
                        })
                        .collect();
                    assert_eq!(owned.len(), 1, "한 쪽의 원본 수식 소유·누락·중복");
                    assert_eq!(owned[0].script, source.script, "원본 내용 보존");
                    assert!(owned[0].cell_index.is_none() && owned[0].note_ref.is_none());
                }
            }
        }
    }
    assert_eq!(source_count, expected, "독립 저장본의 전체 수식 개수");
    assert_eq!(rendered.len(), source_count);
    rendered
}

fn find<'a>(equations: &'a [EquationNode], script: &str) -> &'a EquationNode {
    let found: Vec<_> = equations.iter().filter(|eq| eq.script == script).collect();
    assert_eq!(found.len(), 1, "표기의 고유 소유: {script}");
    found[0]
}

fn literal(value: &LayoutBox, out: &mut String) {
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            assert_eq!(run.glyphs.chars().count(), run.positions.len());
            assert_eq!(run.positions.len(), run.shears.len());
            // HYhwpEQ cmap의 표시 글리프를 독립 Print에서 판독한 문자와 대응시킨다.
            // 출력 backend가 바뀌어도 원문 내용·정체/이탤릭 검사는 유지한다.
            for glyph in run.glyphs.chars() {
                let scalar = match glyph {
                    '\u{e000}'..='\u{e019}' => 0x1d434 + glyph as u32 - 0xe000,
                    '\u{e0ec}' => 0x210e,
                    '\u{e0e5}'..='\u{e0fe}' => 0x1d44e + glyph as u32 - 0xe0e5,
                    '\u{e034}'..='\u{e03c}' => '1' as u32 + glyph as u32 - 0xe034,
                    '\u{e03d}' => '0' as u32,
                    '\u{e048}' => '+' as u32,
                    '\u{e046}' => '-' as u32,
                    '\u{e047}' => '=' as u32,
                    '\u{e055}' => '<' as u32,
                    '\u{e056}' => '>' as u32,
                    _ => glyph as u32,
                };
                out.push(char::from_u32(scalar).expect("독립 글리프의 문자 대응"));
            }
        }
        LayoutKind::Row(children) => {
            for child in children {
                literal(child, out);
            }
        }
        LayoutKind::Text(text)
        | LayoutKind::Number(text)
        | LayoutKind::Symbol(text)
        | LayoutKind::MathSymbol(text)
        | LayoutKind::Function(text)
        | LayoutKind::FontGlyph { text, .. } => out.push_str(text),
        LayoutKind::FontStyle { body, .. } | LayoutKind::Color { body, .. } => literal(body, out),
        LayoutKind::Space(_) | LayoutKind::Empty => {}
        kind => panic!("문자 범위를 구조 명령으로 소비함: {kind:?}"),
    }
}

fn assert_literal(equations: &[EquationNode], script: &str, expected: &str) {
    let mut text = String::new();
    literal(&find(equations, script).layout_box, &mut text);
    assert_eq!(
        text, expected,
        "Print의 정체/이탤릭·기호·괄호·순서: {script}"
    );
}

#[test]
fn native_escape_scope_preserves_roman_literals_and_explicit_italic_return() {
    let rendered = load("native-command-escape-scope", 30);
    // 기대 문자는 Print 확대 판독에서 정한다. 현재 출력의 좌표·해시를 복사하지 않는다.
    for (script, expected) in [
        (r"a \sqrt{x} b", "𝑎sqrtxb"),
        (r"a {\sqrt{x}} b", "𝑎sqrtxb"),
        (r"a \sqrt{x} it b", "𝑎sqrtx𝑏"),
        (r"a \sqrt{it x} b", "𝑎sqrt𝑥𝑏"),
        (r"a \underline{x} b", "𝑎underline{xb"),
        (r"a \UNDERLINE{x} b", "𝑎UNDERLINE{xb"),
        (r"a \underline{x} it b", "𝑎underline{x𝑏"),
        (r"a \underline{{x}} b", "𝑎underline{xb"),
        (r"a \underlin{x} b", "𝑎underlinxb"),
        (r"a \overline{x} b", "𝑎overlinexb"),
    ] {
        assert_literal(&rendered, script, expected);
    }
}

#[test]
fn native_case_aliases_preserve_single_symbols_and_neighbor_contents() {
    let rendered = load("native-command-escape-scope", 30);
    for (script, expected) in [
        ("a LEFTARROW{x} b", "𝑎⇐𝑥𝑏"),
        ("a leftarrow{x} b", "𝑎←𝑥𝑏"),
        ("a Rightarrow{x} b", "𝑎⇒𝑥𝑏"),
        ("a RIGHTARROW{x} b", "𝑎⇒𝑥𝑏"),
        ("a LINE b", "𝑎|𝑏"),
        ("a line b", "𝑎|𝑏"),
        ("a Line b", "𝑎|𝑏"),
    ] {
        assert_literal(&rendered, script, expected);
    }
}

fn decoration_count(value: &LayoutBox) -> usize {
    match &value.kind {
        LayoutKind::Row(children) => children.iter().map(decoration_count).sum(),
        LayoutKind::FontStyle { body, .. } | LayoutKind::Color { body, .. } => {
            decoration_count(body)
        }
        LayoutKind::Decoration { body, .. } => 1 + decoration_count(body),
        LayoutKind::Text(_)
        | LayoutKind::Number(_)
        | LayoutKind::Symbol(_)
        | LayoutKind::MathSymbol(_)
        | LayoutKind::Function(_)
        | LayoutKind::FontGlyph { .. }
        | LayoutKind::HancomGlyphs(_)
        | LayoutKind::Space(_)
        | LayoutKind::Empty => 0,
        kind => panic!("장식 최장 명령을 분수 등 다른 구조로 소비함: {kind:?}"),
    }
}

#[test]
fn native_longest_decoration_commands_own_the_body_before_fraction_prefixes() {
    let rendered = load("native-keyword-longest", 24);
    for script in [
        "a OVERLINE{x} b",
        "a overline{x} b",
        "a UNDERLINE{x} b",
        "a underline{x} b",
        "a UNDER{x} b",
    ] {
        assert_eq!(
            decoration_count(&find(&rendered, script).layout_box),
            1,
            "장식 한 개와 본체의 소유: {script}"
        );
    }
    assert_literal(&rendered, r"\sqrt {x}", "sqrtx");
    assert_literal(&rendered, r"a \overline{x} b", "𝑎overlinexb");
}

mod source_font_projection {
    use super::*;
    use std::collections::BTreeSet;

    struct Glyph {
        text: String,
        family: String,
        x: f64,
        baseline: f64,
        width: f64,
    }

    fn glyphs(value: &LayoutBox, parent_x: f64, parent_y: f64, out: &mut Vec<Glyph>) {
        let x = parent_x + value.x;
        let y = parent_y + value.y;
        match &value.kind {
            LayoutKind::Row(children) => {
                for child in children {
                    glyphs(child, x, y, out);
                }
            }
            LayoutKind::FontStyle { body, .. } | LayoutKind::Color { body, .. } => {
                glyphs(body, x, y, out)
            }
            LayoutKind::HancomGlyphs(run) => {
                let chars: Vec<_> = run.glyphs.chars().collect();
                assert_eq!(chars.len(), run.positions.len());
                assert_eq!(chars.len(), run.shears.len());
                for (i, ch) in chars.into_iter().enumerate() {
                    let start = run.positions[i];
                    let end = run.positions.get(i + 1).copied().unwrap_or(run.width);
                    out.push(Glyph {
                        text: ch.to_string(),
                        family: run.font_family.to_owned(),
                        x: x + start,
                        baseline: y + value.baseline + run.baseline_shift,
                        width: end - start,
                    });
                }
            }
            LayoutKind::FontGlyph {
                text, font_family, ..
            } => out.push(Glyph {
                text: text.clone(),
                family: (*font_family).to_owned(),
                x,
                baseline: y + value.baseline,
                width: value.width,
            }),
            LayoutKind::Text(text)
            | LayoutKind::Number(text)
            | LayoutKind::Symbol(text)
            | LayoutKind::MathSymbol(text) => out.push(Glyph {
                text: text.clone(),
                family: String::new(),
                x,
                baseline: y + value.baseline,
                width: value.width,
            }),
            LayoutKind::Empty | LayoutKind::Space(_) => {}
            other => panic!("선형 원본 기호 대신 다른 구조가 나왔다: {other:?}"),
        }
    }

    fn source<'a>(core: &'a DocumentCore, eq: &EquationNode) -> &'a rhwp::model::control::Equation {
        let control = &core.document().sections[eq.section_index.unwrap()].paragraphs
            [eq.para_index.unwrap()]
        .controls[eq.control_index.unwrap()];
        let Control::Equation(source) = control else {
            panic!("원본 수식 소유")
        };
        source
    }

    fn all_equations(core: &DocumentCore) -> Vec<EquationNode> {
        let mut rendered = Vec::new();
        for page in 0..core.page_count() {
            equations(
                &core.build_page_render_tree(page).unwrap().root,
                &mut rendered,
            );
        }
        let mut owners = BTreeSet::new();
        for eq in &rendered {
            assert!(
                owners.insert((eq.section_index, eq.para_index, eq.control_index)),
                "원본 수식의 중복 방출"
            );
            assert!(eq.cell_index.is_none() && eq.note_ref.is_none());
            assert_eq!(
                eq.script,
                source(core, eq).script,
                "원문 내용을 표시 문자로 덮어쓰지 않음"
            );
        }
        assert_eq!(rendered.len(), 96, "독립 Print의 여섯 크기×16개 내용 보존");
        rendered
    }

    fn diagnostic() -> (DocumentCore, Vec<EquationNode>) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/equation-font-metrics/equation-prime-width-diagnostic-2020.hwp");
        let core =
            DocumentCore::from_bytes(&std::fs::read(path).expect("독립 한컴 저장 입력")).unwrap();
        assert_eq!(
            core.page_count(),
            3,
            "동일 입력의 독립 한컴 Print 전체 쪽수"
        );
        let rendered = all_equations(&core);
        (core, rendered)
    }

    fn same_advance(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= f64::EPSILON * expected.abs().max(1.0) * 64.0,
            "저장 원본과 표시 glyph가 같은 논리 전진을 소비: {actual} / {expected}"
        );
    }

    #[test]
    fn triangle_command_uses_print_fallback_and_the_source_owned_slot() {
        let (core, rendered) = diagnostic();
        let triangles: Vec<_> = rendered
            .iter()
            .filter(|eq| eq.script == "rm triangle ABC")
            .collect();
        assert_eq!(triangles.len(), 6);
        for eq in triangles {
            let original = source(&core, eq);
            let plain: Vec<_> = rendered
                .iter()
                .filter(|v| {
                    v.script == "rm ABC" && source(&core, v).font_size == original.font_size
                })
                .collect();
            assert_eq!(plain.len(), 1, "같은 크기의 독립 ABC 대조군");
            let delta = original
                .common
                .width
                .checked_sub(source(&core, plain[0]).common.width)
                .expect("삼각형 명령이 추가한 독립 저장 전진");
            let mut painted = Vec::new();
            glyphs(&eq.layout_box, 0.0, 0.0, &mut painted);
            assert_eq!(painted.len(), 4, "삼각형과 ABC의 순서·누락·중복");
            assert_eq!(painted[0].text, "∆", "Print의 U+2206과 일반 △를 구분");
            assert!(painted[0].family.starts_with("Haansoft Batang"));
            assert_eq!(
                painted[1..]
                    .iter()
                    .map(|v| v.text.as_str())
                    .collect::<String>(),
                "ABC"
            );
            let slot = rhwp::renderer::hwpunit_to_px(delta as i32, 96.0);
            same_advance(painted[1].x - painted[0].x, slot);
            same_advance(painted[0].width, slot);
            assert!(
                eq.svg_content.contains(">∆</text>"),
                "SVG paint도 공통 표시 glyph를 소비"
            );
        }
    }

    #[test]
    fn quoted_apostrophe_keeps_its_own_glyph_and_never_becomes_a_derivative() {
        let (mut core, rendered) = diagnostic();
        let quoted: Vec<_> = rendered.iter().filter(|eq| eq.script == "\"'\"").collect();
        assert_eq!(quoted.len(), 6);
        for eq in quoted {
            let mut painted = Vec::new();
            glyphs(&eq.layout_box, 0.0, 0.0, &mut painted);
            assert_eq!(painted.len(), 1);
            assert_eq!(painted[0].text, "\u{e050}", "독립 Print의 인용 PUA");
            assert!(painted[0].family.eq_ignore_ascii_case("HyhwpEQ"));
            same_advance(
                eq.layout_box.width,
                rhwp::renderer::hwpunit_to_px(source(&core, eq).common.width as i32, 96.0),
            );
            let unit = source(&core, eq).font_size;
            let prime: Vec<_> = rendered
                .iter()
                .filter(|v| v.script == "rmP'" && source(&core, v).font_size == unit)
                .collect();
            assert_eq!(prime.len(), 1);
            let mut derivative = Vec::new();
            glyphs(&prime[0].layout_box, 0.0, 0.0, &mut derivative);
            assert!(derivative
                .iter()
                .any(|v| v.text == "′" && v.family.starts_with("Haansoft Batang")));
            assert!(!derivative.iter().any(|v| v.text.contains('\u{e050}')));
        }
        // 혼합 인용의 내용 계약만 확인한다. 별도 편집본의 픽셀 배치 검사가 아니다.
        let mut document = core.document().clone();
        for section in &mut document.sections {
            for para in &mut section.paragraphs {
                for control in &mut para.controls {
                    if let Control::Equation(eq) = control {
                        if eq.script == "\"'\"" {
                            eq.script = "\"A'B\"".to_owned();
                            eq.common.width = 0;
                            eq.common.height = 0;
                            eq.baseline = 0;
                        }
                    }
                }
            }
        }
        core.set_document(document);
        let mixed = all_equations(&core);
        let mixed: Vec<_> = mixed.iter().filter(|eq| eq.script == "\"A'B\"").collect();
        assert_eq!(mixed.len(), 6);
        for eq in mixed {
            let mut painted = Vec::new();
            glyphs(&eq.layout_box, 0.0, 0.0, &mut painted);
            assert_eq!(painted.len(), 3);
            assert_eq!(painted[1].text, "\u{e050}");
            assert!(painted[0].x < painted[1].x && painted[1].x < painted[2].x);
        }
    }

    fn owned_paint<'a>(node: &'a RenderNode, script: &str, out: &mut Vec<&'a RenderNode>) {
        if let RenderNodeType::Equation(eq) = &node.node_type {
            if eq.script == script {
                out.push(node);
            }
        }
        for child in &node.children {
            owned_paint(child, script, out);
        }
    }

    #[test]
    fn printed_page_three_keeps_triangle_quote_and_following_roman_origins() {
        let (core, _) = diagnostic();
        let page = core.build_page_render_tree(2).unwrap();
        // 고정형: 독립 Print 3쪽의 origin을 72pt→96px로 환산했다.
        // 글리프 선택·저장 전진은 위 관계형 검사가 별도로 보호한다.
        for (script, count, origins) in [
            (
                "rm triangle ABC",
                4,
                vec![
                    (1, "A", 191.06297, 183.86507),
                    (0, "∆", 167.08018, 185.46389),
                ],
            ),
            ("\"'\"", 1, vec![(0, "인용문자", 167.08018, 151.40888)]),
            ("rm ABC", 3, vec![(0, "A", 167.08018, 216.64101)]),
        ] {
            let mut owners = Vec::new();
            owned_paint(&page.root, script, &mut owners);
            assert_eq!(owners.len(), 1, "독립 Print와 같은 쪽의 고유 수식 소유");
            let node = owners[0];
            let RenderNodeType::Equation(eq) = &node.node_type else {
                unreachable!()
            };
            assert_eq!(source(&core, eq).font_size, 1800);
            let mut painted = Vec::new();
            glyphs(&eq.layout_box, node.bbox.x, node.bbox.y, &mut painted);
            assert_eq!(painted.len(), count, "원본 소유의 표시 슬롯 누락·중복");
            for (slot, text, x, baseline) in origins {
                let glyph = &painted[slot];
                assert!(
                    (glyph.x - x).abs() <= 2.0,
                    "Print 수평 원점 ±2px: {script} / {text}: {} / {x}",
                    glyph.x
                );
                assert!(
                    (glyph.baseline - baseline).abs() <= 2.0,
                    "Print 기준선 ±2px: {script} / {text}: {} / {baseline}",
                    glyph.baseline
                );
            }
        }
    }

    #[test]
    fn generic_math_face_keeps_its_literal_triangle_and_ascii_quote() {
        use rhwp::renderer::equation::{ast::EqNode, layout::EqLayout};
        for node in [
            EqNode::MathSymbol("△".to_owned()),
            EqNode::Quoted("'".to_owned()),
        ] {
            let layout = EqLayout::new(24.0).layout(&node);
            let mut painted = Vec::new();
            glyphs(&layout, 0.0, 0.0, &mut painted);
            assert_eq!(painted.len(), 1);
            assert!(matches!(painted[0].text.as_str(), "△" | "'"));
            assert!(painted[0].family.is_empty());
        }
    }
}
