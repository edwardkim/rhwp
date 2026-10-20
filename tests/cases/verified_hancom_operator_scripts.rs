//! 독립 한컴 Print에서 확인한 연산자·괄호 뒤 첨자와 나머지 항의 보존을 검사한다.
//! 전쪽 Native 최저 99.18280%, fresh WASM 최저 99.16442%, 전체 8쪽 일치.
//! 좌표를 보호하는 7·8쪽은 양쪽 100%. 출처는 fixture의 HANCOM_OPERATOR_SCRIPTS.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::{BTreeMap, BTreeSet};

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/equation-font-metrics/operator-script-saved-2024.hwp");
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

// 글리프의 실제 원점을 자식의 상대 위치까지 누적한다. 로만 C도 빠뜨리지 않는다.
fn glyphs(value: &LayoutBox, x: f64, y: f64, out: &mut Vec<(char, f64, f64)>) {
    let x = x + value.x;
    let y = y + value.y;
    match &value.kind {
        LayoutKind::HancomGlyphs(run) => {
            for (glyph, position) in run.glyphs.chars().zip(&run.positions) {
                if !glyph.is_whitespace() {
                    out.push((glyph, x + position, y + value.baseline + run.baseline_shift));
                }
            }
        }
        LayoutKind::Text(text)
        | LayoutKind::Number(text)
        | LayoutKind::Symbol(text)
        | LayoutKind::MathSymbol(text)
        | LayoutKind::FontGlyph { text, .. }
        | LayoutKind::Function(text) => {
            for glyph in text.chars().filter(|c| !c.is_whitespace()) {
                out.push((glyph, x, y + value.baseline));
            }
        }
        LayoutKind::Row(items) => {
            for child in items {
                glyphs(child, x, y, out);
            }
        }
        LayoutKind::Subscript { base, sub } => {
            glyphs(base, x, y, out);
            glyphs(sub, x, y, out);
        }
        LayoutKind::Superscript { base, sup } => {
            glyphs(base, x, y, out);
            glyphs(sup, x, y, out);
        }
        LayoutKind::SubSup { base, sub, sup } => {
            glyphs(base, x, y, out);
            glyphs(sub, x, y, out);
            glyphs(sup, x, y, out);
        }
        LayoutKind::Decoration { body, .. }
        | LayoutKind::FontStyle { body, .. }
        | LayoutKind::Color { body, .. } => glyphs(body, x, y, out),
        LayoutKind::Empty | LayoutKind::Space(_) => {}
        other => panic!("단순 첨자 대조군에 예상하지 않은 구조: {other:?}"),
    }
}

#[test]
fn printed_operator_scripts_preserve_every_source_owner_and_all_following_terms() {
    let core = load();
    assert_eq!(core.page_count(), 8, "독립 Print 전체 쪽수");
    let mut source = BTreeMap::new();
    for (pi, paragraph) in core.document().sections[0].paragraphs.iter().enumerate() {
        for (ci, control) in paragraph.controls.iter().enumerate() {
            if let Control::Equation(value) = control {
                assert!(source.insert((pi, ci), &value.script).is_none());
            }
        }
    }
    assert_eq!(source.len(), 64);
    // Print의 공백을 제외한 실제 글리프 수. PDF span 수나 현재 parser 결과가 아니다.
    let expected = [11, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 0, 3, 3, 4];
    let mut seen = BTreeSet::new();
    for page in 0..8 {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut actual = Vec::new();
        equations(&tree.root, &mut actual);
        assert_eq!(actual.len(), 8, "쪽별 수식 제어의 누락·중복");
        let mut previous = None;
        for (node, e) in actual {
            assert_eq!(e.section_index, Some(0));
            let owner = (e.para_index.unwrap(), e.control_index.unwrap());
            assert!(seen.insert(owner), "수식 제어 중복: {owner:?}");
            assert_eq!(source.get(&owner).copied(), Some(&e.script));
            assert_eq!(owner.0 / 8, page as usize, "원본 수식의 쪽 소유");
            if let Some((pi, y)) = previous {
                assert!(owner.0 > pi && node.bbox.y > y, "원본 순서와 줄 순서");
            }
            previous = Some((owner.0, node.bbox.y));
            let mut ink = Vec::new();
            glyphs(&e.layout_box, node.bbox.x, node.bbox.y, &mut ink);
            assert_eq!(
                ink.len(),
                expected[owner.0 % 16],
                "문단 {}: {}",
                owner.0,
                e.script
            );
            if owner.0 % 16 == 12 {
                assert!(e.script.starts_with('_'), "진짜 선행 첨자의 음성 대조군");
                assert!(ink.is_empty(), "한컴도 비우는 고아 첨자를 되살리지 않음");
            }
        }
    }
    assert_eq!(seen.len(), source.len(), "전체 문서 제어 보존");
}

#[test]
fn printed_subscripts_and_superscripts_keep_their_base_and_following_term_relations() {
    let core = load();
    let mut checked = 0;
    for page in 0..8 {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut actual = Vec::new();
        equations(&tree.root, &mut actual);
        for (node, e) in actual {
            let case = e.para_index.unwrap() % 16;
            if case == 0 || case == 12 {
                continue;
            }
            let mut ink = Vec::new();
            glyphs(&e.layout_box, node.bbox.x, node.bbox.y, &mut ink);
            if case == 11 {
                assert_eq!(ink.len(), 2);
                assert!(ink[0].2 > ink[1].2, "빈 본체의 아래첨자와 뒤 C");
                assert!(ink[1].1 > ink[0].1, "뒤 항의 순서");
            } else {
                assert!(ink.len() >= 3, "첨자와 뒤 항의 누락: {}", e.script);
                let base = ink[0];
                let sub = ink[1];
                let next = *ink.last().unwrap();
                assert!(sub.1 > base.1 && next.1 > sub.1, "본체·첨자·뒤 항의 순서");
                if case == 14 {
                    assert!(sub.2 < base.2 && sub.2 < next.2, "위첨자의 기준선");
                } else {
                    assert!(sub.2 > base.2 && sub.2 > next.2, "아래첨자의 기준선");
                }
                if case == 15 {
                    assert_eq!(ink.len(), 4);
                    assert!(ink[2].2 < base.2, "위·아래첨자의 본체 공유");
                    assert!((ink[2].1 - sub.1).abs() < e.font_size * 0.1);
                }
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 56, "두 face·두 크기의 모든 적용 경계");
}

fn near_print(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print ±2 CSS px: {actual} / {expected}"
    );
}

#[test]
fn completed_operator_pages_keep_independent_print_script_origins() {
    let core = load();
    // 96dpi·좌상단 CSS 좌표. 양쪽 100%인 7·8쪽의 독립 PDF 글리프 원점이다.
    let cases: &[(u32, usize, &[(usize, f64, f64)])] = &[
        (
            6,
            48,
            &[
                (0, 313.27022, 110.11966),
                (1, 328.62189, 114.91442),
                (2, 333.57920, 109.80001),
                (8, 378.51485, 110.91878),
            ],
        ),
        (
            7,
            57,
            &[
                (0, 254.10227, 135.21223),
                (1, 260.17900, 140.00699),
                (2, 265.13631, 136.01136),
            ],
        ),
        (
            7,
            63,
            &[
                (0, 281.60738, 302.86902),
                (1, 296.95907, 307.66378),
                (2, 296.95907, 296.31616),
                (3, 302.07627, 303.66813),
            ],
        ),
    ];
    for &(page, paragraph, anchors) in cases {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut actual = Vec::new();
        equations(&tree.root, &mut actual);
        let matches: Vec<_> = actual
            .into_iter()
            .filter(|(_, e)| e.para_index == Some(paragraph))
            .collect();
        assert_eq!(matches.len(), 1, "원본 문단의 고유 수식");
        let (node, e) = matches[0];
        let mut ink = Vec::new();
        glyphs(&e.layout_box, node.bbox.x, node.bbox.y, &mut ink);
        for &(ordinal, x, y) in anchors {
            // PDF의 사적 영역 코드와 실제 face의 코드가 다를 수 있다.
            // 원본 스크립트·전체 글자 수를 위에서 확인하고 독립 Print의 내용 순서로 대응한다.
            let glyph = ink.get(ordinal).expect("独立 Print の対応グリフ欠落");
            near_print(glyph.1, x);
            near_print(glyph.2, y);
        }
    }
}
