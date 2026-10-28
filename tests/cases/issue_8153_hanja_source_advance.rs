//! 관계형·고정형: 한자 고유 전진과 뒤 글자모양의 원점·원문 순서를 보호한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 166쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 166쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 2024 Print 166쪽, 96dpi. 두 원본 Native/fresh DEV WASM raw 100%·0픽셀.
//! 실제 프로그램 여섯 개의 cmap/hmtx와 수정 전 FAIL/후 PASS 근거는
//! `mydocs/plans/issue_8153_hanja_glyph_advance_20261027.md`에 연결한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        render_tree::{BoundingBox, TextRunNode},
        TextStyle,
    },
    DocumentCore,
};
fn collect<'a>(n: &'a LayerNode, out: &mut Vec<(&'a BoundingBox, &'a TextRunNode)>) {
    match &n.kind {
        LayerNodeKind::Group { children, .. } => {
            for c in children {
                collect(c, out)
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            collect(child, out)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::TextRun { bbox, run, .. } = op {
                    out.push((bbox, run))
                }
            }
        }
    }
}
#[test]
fn source_hanja_program_advances_do_not_use_hangul_width() {
    for family in [
        "KoPub바탕체 Light",
        "KoPub바탕체 Medium",
        "KoPub바탕체 Bold",
        "KoPub돋움체 Light",
        "KoPub돋움체 Medium",
        "KoPub돋움체 Bold",
    ] {
        let style = TextStyle {
            font_family: family.into(),
            font_size: 96.,
            ..Default::default()
        };
        let actual = EmbeddedTextMeasurer.compute_char_positions("官印公", &style);
        assert!(
            (actual[1] - actual[0] - 86.4).abs() < 0.01,
            "독립 SFNT 900/1000em, 96dpi 한 인치: {family} {actual:?}"
        );
    }
}
fn check(ext: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{ext}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383);
    let tree = core
        .build_page_layer_tree_with_profile(165, RenderProfile::Print)
        .unwrap();
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    let prefix: Vec<_> = runs
        .iter()
        .copied()
        .filter(|(_, r)| r.text.contains("대비 개념으로서의 공인("))
        .collect();
    assert_eq!(prefix.len(), 1);
    let anchor = prefix[0].1;
    let mut owned: Vec<_> = runs
        .into_iter()
        .filter(|(_, r)| {
            r.section_index == anchor.section_index
                && r.para_index == anchor.para_index
                && r.cell_context == anchor.cell_context
        })
        .collect();
    owned.sort_by(|a, b| a.0.x.total_cmp(&b.0.x));
    let text = owned
        .iter()
        .map(|(_, r)| r.display_or_text())
        .collect::<String>();
    assert_eq!(
        text,
        "1) 관인(官印) 대비 개념으로서의 공인(公印) (영 제40조)"
    );
    let found: Vec<_> = owned
        .iter()
        .copied()
        .filter(|(_, r)| r.text == "(")
        .collect();
    assert_eq!(found.len(), 1);
    let (bbox, run) = found[0];
    assert!(
        (bbox.x - 455.84001).abs() <= 2.,
        "독립 Print 원점 455.84001±2px, 실제 {}",
        bbox.x
    );
    assert!((bbox.y + run.baseline - 708.64).abs() <= 2.);
}
#[test]
fn hwpx_original_hanja_keeps_next_style_origin() {
    check("hwpx")
}
#[test]
fn hwp_original_hanja_keeps_next_style_origin() {
    check("hwp")
}
