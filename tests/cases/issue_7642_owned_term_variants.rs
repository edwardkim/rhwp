//! 독립 한컴 저장본의 네 크기·동치 표기 대조.
//! 전쪽 Native/fresh WASM 및 확대 판독: task_m100_7642_stage297.md.
//! 물리 px 위치나 SVG hash 대신 원본 소유·순서·상대 기준선을 검사한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};

fn equations(node: &RenderNode, output: &mut Vec<EquationNode>) {
    if let RenderNodeType::Equation(eq) = &node.node_type {
        output.push(eq.clone());
    }
    for child in &node.children {
        equations(child, output);
    }
}

#[derive(Debug)]
struct Glyph {
    text: String,
    x: f64,
    baseline: f64,
    font_size: f64,
}

fn glyphs(value: &LayoutBox, parent_x: f64, parent_y: f64, output: &mut Vec<Glyph>) {
    let x = parent_x + value.x;
    let y = parent_y + value.y;
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            // 그룹 경계 비교는 실제 표시 글리프의 순서와 전진을 보존한다.
            // 한컴 PUA를 임의 문자열로 치환하지 않고 같은 glyph id를 대조한다.
            assert_eq!(run.glyphs.chars().count(), run.positions.len());
            for (glyph, advance) in run.glyphs.chars().zip(&run.positions) {
                output.push(Glyph {
                    text: glyph.to_string(),
                    x: x + advance,
                    baseline: y + value.baseline + run.baseline_shift,
                    font_size: value.font_size,
                });
            }
        }
        LayoutKind::Row(children) => {
            for child in children {
                glyphs(child, x, y, output);
            }
        }
        LayoutKind::Subscript { base, sub } => {
            glyphs(base, x, y, output);
            glyphs(sub, x, y, output);
        }
        LayoutKind::FontStyle { body, .. } | LayoutKind::Color { body, .. } => {
            glyphs(body, x, y, output);
        }
        LayoutKind::Text(text)
        | LayoutKind::Number(text)
        | LayoutKind::Symbol(text)
        | LayoutKind::MathSymbol(text)
        | LayoutKind::FontGlyph { text, .. } => output.push(Glyph {
            text: text.clone(),
            x,
            baseline: y + value.baseline,
            font_size: value.font_size,
        }),
        LayoutKind::Space(_) | LayoutKind::Empty => {}
        other => panic!("대조 수식에 예상하지 않은 내용 소유 구조: {other:?}"),
    }
}

fn roundoff_equal(left: f64, right: f64, scale: f64) {
    // 같은 물리 항의 상대 관계다. 화면 위치나 고정 px 허용치를 쓰지 않는다.
    assert!(
        (left - right).abs() <= f64::EPSILON * scale.max(1.0) * 128.0,
        "동치 표기의 전진·기준선이 다르다: {left} / {right}"
    );
}

#[test]
fn issue7642_owned_term_variants_preserve_the_same_glyph_sequence_and_baselines() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-term-context-diagnostic/term-context-hancom-2020.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let mut rendered = Vec::new();
    for page in 0..core.page_count() {
        equations(
            &core.build_page_render_tree(page).unwrap().root,
            &mut rendered,
        );
    }
    let variants = ["a _n -3", "a _n - 3", "a _n {-3}", "{a _n} -3"];
    let sources: Vec<_> = core
        .document()
        .sections
        .iter()
        .enumerate()
        .flat_map(|(section, data)| {
            data.paragraphs
                .iter()
                .enumerate()
                .flat_map(move |(paragraph, para)| {
                    para.controls
                        .iter()
                        .enumerate()
                        .filter_map(move |(control, value)| match value {
                            Control::Equation(eq) if variants.contains(&eq.script.as_str()) => {
                                Some((section, paragraph, control, eq.as_ref()))
                            }
                            _ => None,
                        })
                })
        })
        .collect();
    assert_eq!(
        sources.len(),
        16,
        "독립 한컴 저장본의 네 크기·네 동치 표기 보존"
    );
    for (section, paragraph, control, source) in &sources {
        let count = rendered
            .iter()
            .filter(|eq| {
                (eq.section_index, eq.para_index, eq.control_index)
                    == (Some(*section), Some(*paragraph), Some(*control))
                    && eq.script == source.script
            })
            .count();
        assert_eq!(
            count, 1,
            "수식의 원본 소유·누락·중복 검사: {}",
            source.script
        );
    }
    let flats: Vec<_> = rendered
        .iter()
        .filter(|eq| eq.script == variants[0])
        .collect();
    assert_eq!(flats.len(), 4);
    for flat in flats {
        let mut expected = Vec::new();
        glyphs(
            &flat.layout_box,
            0.0,
            -flat.layout_box.baseline,
            &mut expected,
        );
        assert_eq!(expected.len(), 4, "a·아래첨자 n·빼기·3의 내용 보존");
        for variant in &variants[1..] {
            let alternatives: Vec<_> = rendered
                .iter()
                .filter(|eq| eq.script == *variant && eq.font_size == flat.font_size)
                .collect();
            assert_eq!(alternatives.len(), 1);
            let grouped = alternatives[0];
            roundoff_equal(
                grouped.layout_box.width,
                flat.layout_box.width,
                flat.font_size,
            );
            let mut actual = Vec::new();
            glyphs(
                &grouped.layout_box,
                0.0,
                -grouped.layout_box.baseline,
                &mut actual,
            );
            assert_eq!(actual.len(), expected.len());
            for (left, right) in actual.iter().zip(&expected) {
                assert_eq!(
                    left.text, right.text,
                    "그룹 경계가 내용 순서를 바꾸지 않는다"
                );
                roundoff_equal(left.x, right.x, flat.font_size);
                roundoff_equal(left.baseline, right.baseline, flat.font_size);
                roundoff_equal(left.font_size, right.font_size, flat.font_size);
            }
        }
    }
}
