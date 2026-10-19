//! 독립 한컴 2020 Print의 명령 최장 일치·문자 범위·내용 소유 계약.
//! Native/fresh WASM 전쪽 99% 이상 및 같은 영역 확대 판독 뒤 추가한다.
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
