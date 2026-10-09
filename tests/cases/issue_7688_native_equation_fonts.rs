//! #7688: inspect real document equation paint, with independent font outlines.
//! Visual prerequisite: task_m100_7688_stage3.md; no pixel golden or coordinates.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

use std::collections::BTreeSet;
use std::path::PathBuf;

use rhwp::document_core::DocumentCore;
use rhwp::paint::{LayerNode, LayerNodeKind, PageLayerTree, PaintOp, RenderProfile};
use rhwp::renderer::layer_renderer::LayerRasterRenderer;
use rhwp::renderer::skia::SkiaLayerRenderer;

struct Fonts(PathBuf);
impl Fonts {
    fn new(label: &str, bytes: &[u8]) -> Self {
        let path = std::env::temp_dir().join(format!("rhwp-7688-{}-{label}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("face.ttf"), bytes).unwrap();
        Self(path)
    }
}
impl Drop for Fonts {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn equations_only(node: &mut LayerNode, chars: &mut BTreeSet<char>) -> usize {
    match &mut node.kind {
        LayerNodeKind::Group { children, .. } => {
            children.iter_mut().map(|n| equations_only(n, chars)).sum()
        }
        LayerNodeKind::ClipRect { child, .. } => equations_only(child, chars),
        LayerNodeKind::Leaf { ops } => {
            ops.retain(|op| matches!(op, PaintOp::Equation { .. }));
            for op in ops.iter() {
                let PaintOp::Equation { equation, .. } = op else {
                    unreachable!()
                };
                let layout = serde_json::to_string(&equation.layout_box).unwrap();
                chars.extend(
                    layout
                        .chars()
                        .filter(|c| ('\u{ac00}'..='\u{d7a3}').contains(c)),
                );
            }
            ops.len()
        }
    }
}

fn render(tree: &PageLayerTree, fonts: &[PathBuf]) -> image::RgbaImage {
    let png = SkiaLayerRenderer::new()
        .with_font_paths(fonts)
        .render_png(tree)
        .unwrap();
    image::load_from_memory(&png).unwrap().to_rgba8()
}

fn ink(image: &image::RgbaImage) -> u64 {
    image
        .pixels()
        .map(|p| u64::from(p[3]) * u64::from(255 - p[0]))
        .sum()
}

#[test]
fn real_equations_use_custom_outlines_and_skip_a_face_without_hangul() {
    const REGULAR: &[u8] = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
    const LIGHT: &[u8] = include_bytes!("../fixtures/fonts/RHWPEquationCJKLight.ttf");
    const LATIN: &[u8] = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf");
    let regular = Fonts::new("regular", REGULAR);
    let light = Fonts::new("light", LIGHT);
    let latin = Fonts::new("latin", LATIN);
    let core = DocumentCore::from_bytes(include_bytes!("../../samples/eq-01.hwp")).unwrap();
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        let mut tree = core.build_page_layer_tree_with_profile(0, profile).unwrap();
        let mut chars = BTreeSet::new();
        assert_eq!(equations_only(&mut tree.root, &mut chars), 3);
        assert!(chars.contains(&'평') && chars.contains(&'점'));
        // Rebind text sources after projecting the real page to equation ops.
        let tree =
            PageLayerTree::with_profile(tree.page_width, tree.page_height, tree.root, profile);
        let face = ttf_parser::Face::parse(LIGHT, 0).unwrap();
        let latin_face = ttf_parser::Face::parse(LATIN, 0).unwrap();
        for ch in chars {
            assert!(face.glyph_index(ch).is_some(), "light fixture lacks {ch}");
            assert!(
                latin_face.glyph_index(ch).is_none(),
                "negative fixture has {ch}"
            );
        }
        let regular_image = render(&tree, std::slice::from_ref(&regular.0));
        let light_image = render(&tree, std::slice::from_ref(&light.0));
        // The OFL source's wght=400 outlines contain more ink than wght=200.
        // The old equation path ignores both custom faces and paints identical pixels.
        assert!(
            regular_image != light_image,
            "equations ignored supplied font outlines"
        );
        assert!(ink(&regular_image) > ink(&light_image));
        assert!(ink(&light_image) > 0);
        let fallback_image = render(&tree, &[latin.0.clone(), regular.0.clone()]);
        // Latin-only Batang is first for CJK equation text, but its missing glyph
        // must fall through to the same real Noto Sans KR outline as the control.
        assert!(
            regular_image == fallback_image,
            "a missing CJK glyph masked the supplied fallback"
        );
    }
}
