//! 한컴 Print의 논리 기호와 명시적 공간에 따른 분자 소유를 보호한다.
//! 관련 원본/중괄호 대조군은 전쪽 Native·fresh WASM raw 최저 100%다.
//! 출처·SHA·독립 기대 좌표는 HANCOM_LOGICAL_TERM_PRINT.md에 기록한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::{Control, Equation};
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::{BTreeMap, BTreeSet};

const ORIGINAL: &str = "logical-symbol-matrix";
const BRACED: &str = "logical-symbol-isolated-fractions";

fn load(stem: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("pdf/issue7642-logical-symbol-diagnostic-20261017")
        .join(format!("{stem}-saved-2024.hwp"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    core
}

fn equations<'a>(node: &'a RenderNode, out: &mut Vec<(&'a RenderNode, &'a EquationNode)>) {
    if let RenderNodeType::Equation(value) = &node.node_type {
        out.push((node, value));
    }
    for child in &node.children {
        equations(child, out);
    }
}

fn boxes<'a>(value: &'a LayoutBox, out: &mut Vec<&'a LayoutBox>) {
    out.push(value);
    match &value.kind {
        LayoutKind::Row(items) => {
            for item in items {
                boxes(item, out);
            }
        }
        LayoutKind::Fraction { numer, denom, .. } => {
            boxes(numer, out);
            boxes(denom, out);
        }
        LayoutKind::FontStyle { body, .. } | LayoutKind::Color { body, .. } => boxes(body, out),
        _ => {}
    }
}

fn glyphs(value: &LayoutBox, x: f64, y: f64, out: &mut Vec<(char, f64, f64)>) {
    let x = x + value.x;
    let y = y + value.y;
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            out.extend(
                run.glyphs.chars().zip(&run.positions).map(|(ch, advance)| {
                    (ch, x + advance, y + value.baseline + run.baseline_shift)
                }),
            );
        }
        LayoutKind::Row(items) => {
            for item in items {
                glyphs(item, x, y, out);
            }
        }
        LayoutKind::Fraction { numer, denom, .. } => {
            glyphs(numer, x, y, out);
            glyphs(denom, x, y, out);
        }
        LayoutKind::FontStyle { body, .. } | LayoutKind::Color { body, .. } => {
            glyphs(body, x, y, out);
        }
        _ => {}
    }
}

fn one_glyph(ink: &[(char, f64, f64)], ch: char) -> (f64, f64) {
    let found: Vec<_> = ink.iter().filter(|value| value.0 == ch).collect();
    assert_eq!(found.len(), 1, "대응 글리프의 누락·중복: {ch:?}");
    (found[0].1, found[0].2)
}

#[test]
fn printed_logical_symbols_keep_the_source_face_owner_and_single_em_advance() {
    for stem in [ORIGINAL, BRACED] {
        let core = load(stem);
        assert_eq!(core.page_count(), 6, "독립 Print 전체 쪽수");
        let mut source = BTreeMap::<_, &Equation>::new();
        for (pi, paragraph) in core.document().sections[0].paragraphs.iter().enumerate() {
            for (ci, control) in paragraph.controls.iter().enumerate() {
                if let Control::Equation(eq) = control {
                    assert!(source.insert((pi, ci), eq).is_none());
                }
            }
        }
        assert_eq!(source.len(), 48, "독립 입력의 두 face·네 크기·여섯 식");
        let ordinals: BTreeMap<_, _> = source.keys().copied().zip(0..).collect();
        let mut seen = BTreeSet::new();
        let mut sizes = BTreeSet::new();
        let mut faces = BTreeSet::new();
        let mut previous = None;
        for page in 0..6 {
            let tree = core.build_page_render_tree(page).unwrap();
            let mut actual = Vec::new();
            equations(&tree.root, &mut actual);
            assert_eq!(actual.len(), 8, "독립 Print의 쪽별 수식 소유");
            for (_, eq) in actual {
                assert_eq!(eq.section_index, Some(0));
                let owner = (eq.para_index.unwrap(), eq.control_index.unwrap());
                assert!(seen.insert(owner), "수식 소유의 중복");
                assert!(previous.is_none_or(|old| old < owner), "원본 내용 순서");
                previous = Some(owner);
                let original = source[&owner];
                assert_eq!(eq.script, original.script, "원본 스크립트 보존");
                assert_eq!(ordinals[&owner] / 8, page as usize, "쪽 소속 보존");
                sizes.insert(original.font_size);
                faces.insert(original.font_name.clone());
                let expected = if eq.script.starts_with("BECAUSE") {
                    '∵'
                } else {
                    '∴'
                };
                let mut all = Vec::new();
                boxes(&eq.layout_box, &mut all);
                let logical: Vec<_> = all
                    .iter()
                    .filter_map(|item| {
                        if let LayoutKind::HancomGlyphs(run) = &item.kind {
                            run.glyphs.contains(expected).then_some((*item, run))
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(logical.len(), 1, "논리 기호를 원래 수학 face로 한 번 표시");
                let (item, run) = logical[0];
                assert_eq!(run.glyphs, expected.to_string());
                assert!(run.font_family.eq_ignore_ascii_case(&original.font_name));
                // 두 face의 독립 Print에서 확인한 1em이다. 이항 연산자 여백을 더하지 않는다.
                assert!((run.width - item.font_size).abs() <= item.font_size / 1024.0);
            }
        }
        assert_eq!(seen.len(), source.len(), "전쪽 수식 누락 없음");
        assert_eq!(sizes, BTreeSet::from([600, 900, 1200, 1800]));
        assert_eq!(faces.len(), 2);
    }
}

#[test]
fn printed_attached_space_keeps_the_logical_symbol_in_the_unbraced_numerator() {
    for stem in [ORIGINAL, BRACED] {
        let core = load(stem);
        let mut checked = 0;
        for page in 0..core.page_count() {
            let tree = core.build_page_render_tree(page).unwrap();
            let mut actual = Vec::new();
            equations(&tree.root, &mut actual);
            for (_, eq) in actual {
                if !eq.script.contains("over") {
                    continue;
                }
                let mut all = Vec::new();
                boxes(&eq.layout_box, &mut all);
                let fractions: Vec<_> = all
                    .iter()
                    .filter_map(|item| {
                        if let LayoutKind::Fraction { numer, denom, .. } = &item.kind {
                            Some((numer.as_ref(), denom.as_ref()))
                        } else {
                            None
                        }
                    })
                    .collect();
                assert_eq!(fractions.len(), 3, "세 분수의 보존");
                let (numer, _) = fractions[0];
                let mut ink = Vec::new();
                glyphs(numer, 0.0, 0.0, &mut ink);
                assert_eq!(ink.iter().any(|value| value.0 == '∴'), stem == ORIGINAL);
                for (numer, denom) in fractions {
                    assert!(numer.y + numer.baseline < denom.y + denom.baseline);
                }
                let mut ink = Vec::new();
                glyphs(&eq.layout_box, 0.0, 0.0, &mut ink);
                let logical = one_glyph(&ink, '∴');
                let q = one_glyph(&ink, '\u{e0f5}');
                assert!(logical.0 < q.0, "논리 기호와 q의 내용 순서");
                if stem == ORIGINAL {
                    assert!((logical.1 - q.1).abs() <= eq.font_size / 1024.0);
                } else {
                    assert!(q.1 < logical.1, "중괄호의 q만 분자에 속한다");
                }
                checked += 1;
            }
        }
        assert_eq!(checked, 8, "두 face·네 크기의 분수 경계");
    }
}

#[test]
fn completed_logical_pages_keep_independent_print_anchors_within_two_pixels() {
    // 두 문서의 6쪽은 전쪽 Native/fresh WASM raw가 각각 100%다.
    // 96dpi의 독립 PDF 글리프 원점이며 구현 출력에서 복사한 좌표가 아니다.
    let anchors = [
        (ORIGINAL, (332.77964, 262.59302), (368.76013, 262.59302)),
        (BRACED, (299.67759, 278.09609), (342.21452, 262.59302)),
    ];
    for (stem, logical_expected, q_expected) in anchors {
        let core = load(stem);
        let tree = core.build_page_render_tree(5).unwrap();
        let mut actual = Vec::new();
        equations(&tree.root, &mut actual);
        let matches: Vec<_> = actual
            .into_iter()
            .filter(|(_, eq)| {
                eq.script.contains("over") && (eq.font_size - 24.0).abs() < f64::EPSILON
            })
            .collect();
        assert_eq!(matches.len(), 1, "완료 쪽의 18pt 원본 수식");
        let (node, eq) = matches[0];
        let mut ink = Vec::new();
        glyphs(&eq.layout_box, node.bbox.x, node.bbox.y, &mut ink);
        for (ch, expected) in [('∴', logical_expected), ('\u{e0f5}', q_expected)] {
            let actual = one_glyph(&ink, ch);
            assert!(
                (actual.0 - expected.0).abs() <= 2.0,
                "독립 Print x: {actual:?}"
            );
            assert!(
                (actual.1 - expected.1).abs() <= 2.0,
                "독립 Print y: {actual:?}"
            );
        }
    }
}
