//! 한컴 Print가 동일한 크기 보호 대조군의 표시와 저장 보존을 검사한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::model::control::Control;
use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
use rhwp::renderer::render_tree::{EquationNode, RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::BTreeSet;

fn load(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/equation-font-metrics/size-protection")
        .join(name);
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    core
}

#[derive(Debug, PartialEq)]
struct Source {
    owner: (usize, usize),
    script: String,
    font: String,
    width: u32,
    height: u32,
    protected: bool,
}

fn source(core: &DocumentCore) -> Vec<Source> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .flat_map(|(pi, p)| {
            p.controls.iter().enumerate().filter_map(move |(ci, c)| {
                let Control::Equation(e) = c else { return None };
                Some(Source {
                    owner: (pi, ci),
                    script: e.script.clone(),
                    font: e.font_name.clone(),
                    width: e.common.width,
                    height: e.common.height,
                    protected: e.common.size_protect,
                })
            })
        })
        .collect()
}

fn equations<'a>(node: &'a RenderNode, out: &mut Vec<(&'a RenderNode, &'a EquationNode)>) {
    if let RenderNodeType::Equation(e) = &node.node_type {
        out.push((node, e));
    }
    for child in &node.children {
        equations(child, out);
    }
}

// 독립 Print가 같은 두 입력의 글리프·원점·글자 크기를 서로 대조한다.
fn glyphs(b: &LayoutBox, x: f64, y: f64, out: &mut Vec<(String, f64, f64, f64)>) {
    let (x, y) = (x + b.x, y + b.y);
    match &b.kind {
        LayoutKind::HancomGlyphs(run) => {
            for (ch, dx) in run.glyphs.chars().zip(&run.positions) {
                if !ch.is_whitespace() {
                    out.push((
                        ch.to_string(),
                        x + dx,
                        y + b.baseline + run.baseline_shift,
                        b.font_size,
                    ));
                }
            }
        }
        LayoutKind::Text(t)
        | LayoutKind::Number(t)
        | LayoutKind::Symbol(t)
        | LayoutKind::MathSymbol(t)
        | LayoutKind::FontGlyph { text: t, .. }
        | LayoutKind::Function(t) => {
            for ch in t.chars().filter(|c| !c.is_whitespace()) {
                out.push((ch.to_string(), x, y + b.baseline, b.font_size));
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
        other => panic!("첨자 대조군에 예상하지 않은 구조: {other:?}"),
    }
}

#[test]
fn hancom_printed_size_protection_keeps_glyphs_baselines_and_source_owners() {
    for ext in ["hwpx", "hwp"] {
        let suffix = if ext == "hwp" {
            "-saved-2020.hwp"
        } else {
            ".hwpx"
        };
        let unlocked = load(&format!("operator-unlocked{suffix}"));
        let locked = load(&format!("operator-locked{suffix}"));
        let a = source(&unlocked);
        let mut b = source(&locked);
        assert_eq!(a.len(), 64);
        assert_eq!(a.len(), b.len());
        assert!(a.iter().all(|v| !v.protected));
        assert!(b.iter().all(|v| v.protected));
        for row in &mut b {
            row.protected = false;
        }
        assert_eq!(a, b, "보호 외 내용·저장 크기·소유가 같은 원본");
        assert_eq!(unlocked.page_count(), 8, "독립 Print 전체 쪽수");
        assert_eq!(
            locked.page_count(),
            8,
            "보호 속성으로 쪽 소유가 바뀌지 않음"
        );
        let mut seen = BTreeSet::new();
        for page in 0..8 {
            let ua = unlocked.build_page_render_tree(page).unwrap();
            let lb = locked.build_page_render_tree(page).unwrap();
            let (mut us, mut ls) = (Vec::new(), Vec::new());
            equations(&ua.root, &mut us);
            equations(&lb.root, &mut ls);
            assert_eq!(us.len(), 8);
            assert_eq!(ls.len(), 8);
            let mut previous_y = None;
            for ((_, ue), (ln, le)) in us.into_iter().zip(ls) {
                let owner = (le.para_index.unwrap(), le.control_index.unwrap());
                assert!(seen.insert(owner), "중복 수식");
                assert_eq!(ue.section_index, le.section_index);
                assert_eq!(ue.para_index, le.para_index);
                assert_eq!(ue.control_index, le.control_index);
                assert_eq!(ue.script, le.script);
                assert_eq!(owner.0 / 8, page as usize, "원본 문단의 쪽 소유");
                if let Some(y) = previous_y {
                    assert!(ln.bbox.y > y, "줄 순서");
                }
                previous_y = Some(ln.bbox.y);
                let (mut ug, mut lg) = (Vec::new(), Vec::new());
                // 보호된 저장 프레임과 비보호 논리 프레임의 줄 원점은 구분한다.
                // 여기서는 같은 Print 글리프의 본체 기준 상대 위치를 비교하고,
                // 완료 페이지의 실제 원점은 아래 독립 Print 좌표 검사로 보호한다.
                glyphs(&ue.layout_box, -ue.layout_box.x, -ue.layout_box.y, &mut ug);
                glyphs(&le.layout_box, -le.layout_box.x, -le.layout_box.y, &mut lg);
                let expected = [11, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 0, 3, 3, 4];
                assert_eq!(ug.len(), expected[owner.0 % 16], "Print의 표시 글리프 수");
                assert_eq!(ug, lg, "독립 Print의 본체·첨자·뒤 항 관계: {}", le.script);
            }
        }
        assert_eq!(seen.len(), 64, "전체 소유 보존");
    }
}

#[test]
fn hancom_size_protection_survives_hwpx_save_and_reopen() {
    for name in [
        "operator-unlocked.hwpx",
        "operator-locked.hwpx",
        "operator-unlocked-saved-2020.hwp",
        "operator-locked-saved-2020.hwp",
    ] {
        let core = load(name);
        let expected = source(&core);
        assert_eq!(expected.len(), 64);
        let output = core.export_hwpx_native().unwrap();
        let reopened = DocumentCore::from_bytes(&output).unwrap();
        assert_eq!(
            source(&reopened),
            expected,
            "크기 보호·글꼴·스크립트·저장 크기·소유 보존: {name}"
        );
    }
}

#[test]
fn protected_completed_pages_preserve_independent_print_glyph_origins() {
    // 96dpi·좌상단 CSS 좌표. 두 출력 경로가 100%인 7·8쪽의 Print 원점이다.
    // 2020 Print의 글리프·원점은 기존 2024 Print와 전쪽에서 독립 대조했다.
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
    for name in ["operator-locked.hwpx", "operator-locked-saved-2020.hwp"] {
        let core = load(name);
        for &(page, paragraph, anchors) in cases {
            let tree = core.build_page_render_tree(page).unwrap();
            let mut actual = Vec::new();
            equations(&tree.root, &mut actual);
            let selected: Vec<_> = actual
                .into_iter()
                .filter(|(_, e)| e.para_index == Some(paragraph))
                .collect();
            assert_eq!(selected.len(), 1, "원본 수식의 유일한 소유");
            let (node, e) = selected[0];
            let mut ink = Vec::new();
            glyphs(&e.layout_box, node.bbox.x, node.bbox.y, &mut ink);
            for &(ordinal, x, y) in anchors {
                let glyph = ink.get(ordinal).expect("독립 Print 대응 글리프 누락");
                assert!(
                    (glyph.1 - x).abs() <= 2.0,
                    "독립 Print 가로 원점: {glyph:?} / {x}"
                );
                assert!(
                    (glyph.2 - y).abs() <= 2.0,
                    "독립 Print 기준선: {glyph:?} / {y}"
                );
            }
        }
    }
}
