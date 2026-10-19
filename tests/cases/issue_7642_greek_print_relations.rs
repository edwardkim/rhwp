//! 독립 한컴 Print와 Native/fresh WASM 전체 페이지·확대 판독을 확인한 그리스 계약.
//! 증적: mydocs/pr/assets/issue7642-greek-symbol-contract-20261012/README.md
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};
use std::collections::{BTreeMap, BTreeSet};

type GreekFrame = (String, String, f64, f64, f64);

fn load(version: usize, pages: u32) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-extended-base-script-diagnostic")
        .join(format!("extended-base-script-v{version}-2024.hwp"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).expect("독립 한컴 저장 입력"))
        .expect("한컴 수식 읽기");
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), pages, "독립 Print 전체 쪽수");
    core
}

fn equations<'a>(node: &'a RenderNode, out: &mut Vec<&'a EquationNode>) {
    if let RenderNodeType::Equation(eq) = &node.node_type {
        out.push(eq);
    }
    for child in &node.children {
        equations(child, out);
    }
}

fn leaves<'a>(
    value: &'a LayoutBox,
    parent_x: f64,
    parent_y: f64,
    out: &mut Vec<(&'a LayoutBox, f64, f64, String, String)>,
) {
    let x = parent_x + value.x;
    let y = parent_y + value.y;
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            assert_eq!(run.glyphs.chars().count(), 1, "붙인 단일 X의 표시 소유");
            assert_eq!(run.positions.len(), 1);
            let text = match run.glyphs.as_str() {
                "\u{e017}" => "𝑋",
                "X" => "X",
                other => panic!("원본 X와 다른 표시 문자: {other:?}"),
            };
            out.push((
                value,
                x + run.positions[0],
                y + run.baseline_shift,
                text.to_owned(),
                "HyhwpEQ".to_owned(),
            ));
        }
        LayoutKind::Row(items) => {
            for item in items {
                leaves(item, x, y, out);
            }
        }
        LayoutKind::FontStyle { body, .. } => leaves(body, x, y, out),
        LayoutKind::Text(text) | LayoutKind::MathSymbol(text) => {
            out.push((value, x, y, text.clone(), String::new()))
        }
        LayoutKind::FontGlyph {
            text, font_family, ..
        } => out.push((value, x, y, text.clone(), (*font_family).to_owned())),
        other => panic!("단일 기호/붙인 피연산자 대신 다른 구조가 나왔다: {other:?}"),
    }
}

fn is_greek_glyph(text: &str) -> bool {
    let mut chars = text.chars();
    matches!((chars.next(), chars.next()), (Some(ch), None)
        if ('\u{0370}'..='\u{03ff}').contains(&ch) || ('\u{1D6A8}'..='\u{1D7CB}').contains(&ch))
}

fn same_symbol_frame(first: &GreekFrame, other: &GreekFrame, stem: &str) {
    assert_eq!(
        first.0, other.0,
        "라틴 스타일이 기호 glyph를 바꾸면 안 된다: {stem}"
    );
    assert_eq!(first.1, other.1, "독립 Print와 같은 기호 face");
    for (a, b) in [(first.2, other.2), (first.3, other.3), (first.4, other.4)] {
        assert!(
            (a - b).abs() <= f64::EPSILON * 32.0,
            "기호의 상대 폭·기준선·크기 보존: {stem}"
        );
    }
}

#[test]
fn greek_symbols_keep_distinct_variants_and_the_same_frame_under_latin_styles() {
    let core = load(10, 5);
    let mut sources = BTreeMap::new();
    for (s, section) in core.document().sections.iter().enumerate() {
        for (p, paragraph) in section.paragraphs.iter().enumerate() {
            for (c, control) in paragraph.controls.iter().enumerate() {
                if let Control::Equation(eq) = control {
                    sources.insert((s, p, c), (eq.script.clone(), eq.font_size));
                }
            }
        }
    }
    assert_eq!(sources.len(), 160, "20개 이름·네 스타일·두 글자 크기");
    let mut emitted = BTreeSet::new();
    let mut groups: BTreeMap<(u32, String), Vec<GreekFrame>> = BTreeMap::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut found = Vec::new();
        equations(&tree.root, &mut found);
        for eq in found {
            let owner = (
                eq.section_index.unwrap(),
                eq.para_index.unwrap(),
                eq.control_index.unwrap(),
            );
            let (script, unit) = &sources[&owner];
            assert!(emitted.insert(owner), "원본 소유 수식의 중복 방출");
            assert_eq!(&eq.script, script);
            let stem = script
                .strip_prefix("rm ")
                .or_else(|| script.strip_prefix("it "))
                .or_else(|| script.strip_prefix("bold "))
                .unwrap_or(script);
            let mut glyphs = Vec::new();
            leaves(&eq.layout_box, 0.0, 0.0, &mut glyphs);
            assert_eq!(
                glyphs.len(),
                1,
                "그리스 이름이 문자 이름으로 남으면 안 된다: {script}"
            );
            let (glyph, _, y, text, family) = &glyphs[0];
            assert!(
                is_greek_glyph(text),
                "그리스 기호 의미 보존: {script} -> {text}"
            );
            assert!(eq.font_size > 0.0);
            let observed = (
                text.clone(),
                family.clone(),
                glyph.width / eq.font_size,
                (y + glyph.baseline) / eq.font_size,
                glyph.font_size / eq.font_size,
            );
            let values = groups.entry((*unit, stem.to_owned())).or_default();
            if let Some(first) = values.first() {
                same_symbol_frame(first, &observed, stem);
            }
            values.push(observed);
        }
    }
    assert_eq!(
        emitted.len(),
        sources.len(),
        "원본 수식의 누락 없는 단일 소유"
    );
    assert_eq!(groups.len(), 40);
    for ((_, stem), values) in &groups {
        assert_eq!(values.len(), 4, "기본/rm/it/bold 대조군: {stem}");
    }
    for unit in [1500, 2400] {
        for (plain, variant) in [
            ("phi", "varphi"),
            ("epsilon", "varepsilon"),
            ("rho", "varrho"),
        ] {
            let a = &groups[&(unit, plain.to_owned())][0];
            let b = &groups[&(unit, variant.to_owned())][0];
            assert_ne!(
                (&a.0, &a.1),
                (&b.0, &b.1),
                "독립 Print가 구분하는 기호를 합치면 안 된다"
            );
        }
    }
}

#[test]
fn glued_greek_variant_keeps_the_trailing_operand_after_its_source_symbol() {
    let core = load(11, 2);
    let mut selected = 0;
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut found = Vec::new();
        equations(&tree.root, &mut found);
        for eq in found {
            if !matches!(eq.script.as_str(), "varrhoX" | "varphiX" | "varepsilonX") {
                continue;
            }
            selected += 1;
            let mut glyphs = Vec::new();
            leaves(&eq.layout_box, 0.0, 0.0, &mut glyphs);
            assert_eq!(
                glyphs.len(),
                2,
                "기호와 붙인 피연산자의 완전한 보존: {}",
                eq.script
            );
            assert!(is_greek_glyph(&glyphs[0].3));
            assert!(
                matches!(glyphs[1].3.as_str(), "X" | "𝑋"),
                "뒤 피연산자 X의 보존"
            );
            assert!(glyphs[0].1 < glyphs[1].1, "원문 기호→피연산자의 순서");
            let precision = eq.font_size * f64::EPSILON * 32.0;
            assert!(
                glyphs[0].1 + glyphs[0].0.width <= glyphs[1].1 + precision,
                "두 소유 예약 영역이 겹치면 안 된다"
            );
        }
    }
    assert_eq!(selected, 6, "세 붙인 식을 두 글자 크기에서 검증");
}
