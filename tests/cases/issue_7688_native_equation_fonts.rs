//! 관계형: 실제 수식 producer의 원문 문자와 Native 윤곽·원점·글꼴 공급 관계를 검사한다.
//! 주석 갱신: 2026-11-07T19:23:49+09:00; 직전 파일 커밋: 2026-11-07T14:25:54+09:00
//! (`263b1e753c006d1631d25794588b6638fa1172ae`).
//! 2026-11-07 측정: 96 DPI, PDFium, 색상 경계 보정 전 2px silhouette_raw_match_percent.
//! eq-01.hwp / eq-01-hwp-2020.pdf 전 1쪽: Native 95.46% / fresh WASM Canvas 95.51%.
//! issue7593/repro-equation-issue7593.hwpx / 동명 hwpx-2020.pdf 전 1쪽:
//! 원 수식 face 공급 Native 99.60% / fresh WASM 99.48%; 수식 face 부재 Native 99.63%.
//! 두 원본 모두 해당 쪽이 전쪽 최저다. 원문 4·3과 AB의 직접 확대 판독을 완료했다.
//! 근거: mydocs/pr/assets/seo-rii-20261107/positioned-equation-fallback-finding.md,
//! 같은 경로의 visual/{eq-01-native,eq-01-wasm}.tsv 및 issue8337/after-*.tsv.
//! 나머지 positioned-run 함수는 공개 fixture 글꼴의 합성 surface 계약이다.
//! 합성 원점·기준선은 동일 입력의 독립 Skia font paint와 비교하며 실물 페이지 px 핀이 아니다.
//! 같은 family의 분리 공급 검사는 공개 글꼴만 공급하고, 원 family의 coverage와 다른 family
//! fallback의 경계를 독립 Skia paint로 비교한다. 설치 글꼴이나 실물 페이지 좌표에 의존하지 않는다.
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
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            equations_only(child, chars)
        }
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

fn render(tree: &PageLayerTree, fonts: &[PathBuf], substitute: bool) -> image::RgbaImage {
    let mut renderer = SkiaLayerRenderer::new().with_font_paths(fonts);
    if substitute {
        renderer = renderer
            .with_font_family_alias("Haansoft Batang", "RHWP Equation Contract")
            .unwrap();
    }
    let png = renderer.render_png(tree).unwrap();
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
    const REGULAR: &[u8] = include_bytes!("../fixtures/fonts/RHWPEquationCJKRegular.ttf");
    const GENERAL: &[u8] = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
    const LIGHT: &[u8] = include_bytes!("../fixtures/fonts/RHWPEquationCJKLight.ttf");
    const LATIN: &[u8] = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf");
    for bytes in [REGULAR, LIGHT, LATIN] {
        let face = ttf_parser::Face::parse(bytes, 0).unwrap();
        let families: Vec<_> = face
            .names()
            .into_iter()
            .filter(|name| matches!(name.name_id, 1 | 16))
            .filter_map(|name| name.to_string())
            .collect();
        assert!(!families.is_empty());
        assert!(families.iter().all(|name| name == "RHWP Equation Contract"));
    }
    let regular = Fonts::new("regular", REGULAR);
    let general = Fonts::new("general-fallback", GENERAL);
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
        let regular_image = render(&tree, std::slice::from_ref(&regular.0), true);
        let light_image = render(&tree, std::slice::from_ref(&light.0), true);
        // The OFL Regular program contains more ink than the ExtraLight program.
        // The old equation path ignores both custom faces and paints identical pixels.
        assert!(
            regular_image != light_image,
            "equations ignored supplied font outlines"
        );
        assert!(ink(&regular_image) > ink(&light_image));
        assert!(ink(&light_image) > 0);
        let fallback_image = render(&tree, &[latin.0.clone(), general.0.clone()], true);
        let fallback_reference = render(&tree, std::slice::from_ref(&general.0), false);
        // 첫 수식 family를 명시적으로 공급해 실제 프로그램의 양성 대조를 한다.
        // 음성 대조는 첫 공급 프로그램에 한글이 없을 때의 정상 family 사다리다.
        // 시스템의 앞선 명조 글꼴이 뒤의 Noto보다 먼저 선택되는 계약도 보존한다.
        assert!(
            fallback_reference == fallback_image,
            "a missing CJK glyph masked the supplied fallback"
        );
    }
}

#[test]
fn explicit_font_substitution_rejects_missing_supply_and_empty_requests() {
    assert!(SkiaLayerRenderer::new()
        .with_font_family_alias("Haansoft Batang", "RHWP Equation Contract")
        .is_err());
    let supplied = Fonts::new(
        "empty-request",
        include_bytes!("../fixtures/fonts/RHWPEquationCJKRegular.ttf"),
    );
    assert!(SkiaLayerRenderer::new()
        .with_font_paths(std::slice::from_ref(&supplied.0))
        .with_font_family_alias(" ", "RHWP Equation Contract")
        .is_err());
}

fn check_positioned_hancom_run(
    family: &'static str,
    text: &str,
    expect_primary: bool,
    source_run: Option<rhwp::renderer::equation::hancom_glyphs::GlyphRun>,
) {
    use rhwp::renderer::equation::hancom_glyphs::GlyphRun;
    use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
    use rhwp::renderer::render_tree::{BoundingBox, EquationNode};
    use skia_safe::{
        font, surfaces, AlphaType, Color, ColorType, Data, Font, FontMgr, ImageInfo, Paint,
    };

    const PRIMARY: &[u8] = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
    const FALLBACK: &[u8] = include_bytes!("../fixtures/fonts/RHWPEquationCJKRegular.ttf");
    let font_mgr = FontMgr::default();
    let original = font_mgr
        .new_from_data(Data::new_copy(PRIMARY), Some(0))
        .unwrap();
    let fallback = font_mgr
        .new_from_data(Data::new_copy(FALLBACK), Some(0))
        .unwrap();
    assert_ne!(original.unichar_to_glyph('A' as i32), 0);
    assert_eq!(original.unichar_to_glyph('평' as i32), 0);
    let run = source_run.unwrap_or_else(|| GlyphRun {
        font_family: family,
        glyphs: text.into(),
        positions: vec![3.0, 70.0, 121.0],
        shears: vec![0.3, -0.2, 0.5],
        // Natural fallback advances deliberately differ from the source positions.
        width: 191.0,
        baseline_shift: 7.0,
    });
    let layout = LayoutBox {
        font_size: 32.0,
        x: 11.0,
        y: 4.0,
        width: run.width,
        height: 70.0,
        baseline: 41.0,
        kind: LayoutKind::HancomGlyphs(run.clone()),
    };
    let equation = EquationNode {
        svg_content: String::new(),
        layout_box: layout,
        color_str: "#000000".into(),
        color: 0,
        font_size: 32.0,
        script: String::new(),
        section_index: None,
        para_index: None,
        control_index: None,
        cell_index: None,
        cell_para_index: None,
        note_ref: None,
    };
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[PRIMARY.to_vec(), FALLBACK.to_vec()])
        .unwrap()
        .with_font_family_alias("RHWP Original Equation", &original.family_name())
        .unwrap()
        .with_font_family_alias("Haansoft Batang", &fallback.family_name())
        .unwrap()
        .with_font_family_alias("HyhwpEQ", &original.family_name())
        .unwrap()
        .with_font_family_alias("Latin Modern Math", &fallback.family_name())
        .unwrap();
    let reference = |use_primary: bool| {
        let mut surface = surfaces::raster_n32_premul((240, 100)).unwrap();
        let canvas = surface.canvas();
        canvas.clear(Color::TRANSPARENT);
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::BLACK);
        for ((ch, offset), shear) in text.chars().zip(&run.positions).zip(&run.shears) {
            let face = if use_primary && original.unichar_to_glyph(ch as i32) != 0 {
                &original
            } else {
                &fallback
            };
            assert_ne!(face.unichar_to_glyph(ch as i32), 0, "fixture lacks {ch}");
            let mut font = Font::new(face.clone(), 32.0);
            font.set_edging(font::Edging::AntiAlias);
            font.set_hinting(skia_safe::FontHinting::None);
            font.set_subpixel(true);
            font.set_skew_x(-(*shear as f32));
            canvas.draw_str(
                ch.to_string(),
                ((24.0 + offset) as f32, (51.0 + run.baseline_shift) as f32),
                &font,
                &paint,
            );
        }
        let image = surface.image_snapshot();
        let info = ImageInfo::new((240, 100), ColorType::RGBA8888, AlphaType::Unpremul, None);
        let mut pixels = vec![0u8; 240 * 100 * 4];
        assert!(image.read_pixels(
            &info,
            &mut pixels,
            240 * 4,
            (0, 0),
            skia_safe::image::CachingHint::Disallow
        ));
        image::RgbaImage::from_raw(240, 100, pixels).unwrap()
    };
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        let tree = PageLayerTree::with_profile(
            240.0,
            100.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 240.0, 100.0),
                None,
                vec![PaintOp::equation(
                    BoundingBox::new(13.0, 6.0, 210.0, 80.0),
                    equation.clone(),
                )],
            ),
            profile,
        );
        let actual = image::load_from_memory(&renderer.render_png(&tree).unwrap())
            .unwrap()
            .to_rgba8();
        assert!(
            actual.pixels().any(|pixel| pixel[3] > 0),
            "fallback must paint ink"
        );
        assert!(
            actual == reference(expect_primary),
            "fallback changed source character, positions, baseline or shear ({profile:?})"
        );
        if expect_primary {
            assert!(
                actual != reference(false),
                "available original outlines must remain selected"
            );
        }
    }
}

#[test]
fn missing_hancom_family_uses_character_fallback_at_source_positions() {
    check_positioned_hancom_run("RHWP Missing Hancom Equation Fixture", "A평A", false, None);
}

#[test]
fn partial_hancom_face_falls_back_only_for_the_missing_character() {
    check_positioned_hancom_run("RHWP Original Equation", "A평A", true, None);
}

#[test]
fn complete_hancom_face_retains_original_outlines_and_placement() {
    check_positioned_hancom_run("RHWP Original Equation", "AAA", true, None);
}

fn real_digit_runs(
    node: &LayerNode,
    runs: &mut Vec<(String, rhwp::renderer::equation::hancom_glyphs::GlyphRun)>,
) {
    use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
    fn number_run(layout: &LayoutBox) -> &rhwp::renderer::equation::hancom_glyphs::GlyphRun {
        match &layout.kind {
            LayoutKind::HancomGlyphs(run) => run,
            LayoutKind::Row(children) if children.len() == 1 => number_run(&children[0]),
            LayoutKind::FontStyle { body, .. } => number_run(body),
            other => panic!("real number must retain a positioned glyph run: {other:?}"),
        }
    }
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                real_digit_runs(child, runs);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            real_digit_runs(child, runs)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Equation { equation, .. } = op {
                    let source = equation.script.trim();
                    if source == "4" || source == "3" {
                        runs.push((source.to_owned(), number_run(&equation.layout_box).clone()));
                    }
                }
            }
        }
    }
}

#[test]
fn real_producer_private_digits_fall_back_to_the_source_number() {
    // Public #8337 attachment and this fixture have the same SHA-256; the
    // independent Print shows 4 in [4점] and 3 in the third answer choice.
    let core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue7593/repro-equation-issue7593.hwpx"
    ))
    .unwrap();
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        let tree = core.build_page_layer_tree_with_profile(0, profile).unwrap();
        let mut runs = Vec::new();
        real_digit_runs(&tree.root, &mut runs);
        assert_eq!(
            runs.iter()
                .map(|(text, _)| text.as_str())
                .collect::<Vec<_>>(),
            ["4", "3"]
        );
        for (source, run) in runs {
            assert_eq!(run.font_family, "HyhwpEQ");
            assert_ne!(
                run.glyphs, source,
                "test must exercise the actual private code boundary"
            );
            assert_eq!(run.glyphs.chars().count(), source.chars().count());
            // The supplied primary program deliberately lacks private codes.
            // Compare to the public fallback's source-number outlines at the
            // producer's positions/shears, not its natural text advances.
            check_positioned_hancom_run(run.font_family, &source, false, Some(run));
        }
    }
}

#[test]
fn positioned_equation_exhausts_split_original_family_before_semantic_fallback() {
    use rhwp::renderer::equation::hancom_glyphs::GlyphRun;
    use rhwp::renderer::equation::layout::{LayoutBox, LayoutKind};
    use rhwp::renderer::render_tree::{BoundingBox, EquationNode};
    use skia_safe::{
        font, surfaces, AlphaType, Color, ColorType, Data, Font, FontMgr, ImageInfo, Paint,
    };

    const LATIN: &[u8] = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf");
    const HANGUL: &[u8] = include_bytes!("../fixtures/fonts/RHWPEquationCJKLight.ttf");
    const FALLBACK: &[u8] = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
    let manager = FontMgr::default();
    let face = |bytes: &[u8]| {
        manager
            .new_from_data(Data::new_copy(bytes), Some(0))
            .unwrap()
    };
    let latin = face(LATIN);
    let hangul = face(HANGUL);
    let fallback = face(FALLBACK);
    assert_eq!(latin.family_name(), hangul.family_name());
    assert_eq!(latin.unichar_to_glyph('평' as i32), 0);
    assert_ne!(hangul.unichar_to_glyph('평' as i32), 0);

    // 원점·전진은 실제 face의 자연 폭과 다르게 정해 놓은 합성 producer 계약이다.
    let positions = [4.0, 57.0, 125.0];
    let shears = [0.15, 0.0, -0.1];
    let layout = LayoutBox {
        font_size: 32.0,
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 80.0,
        baseline: 48.0,
        kind: LayoutKind::HancomGlyphs(GlyphRun {
            font_family: "RHWP Equation Contract",
            glyphs: "A평A".into(),
            positions: positions.to_vec(),
            shears: shears.to_vec(),
            width: 180.0,
            baseline_shift: 3.0,
        }),
    };
    let equation = EquationNode {
        svg_content: String::new(),
        layout_box: layout,
        color_str: "#000000".into(),
        color: 0,
        font_size: 32.0,
        script: String::new(),
        section_index: None,
        para_index: None,
        control_index: None,
        cell_index: None,
        cell_para_index: None,
        note_ref: None,
    };
    let reference = |hangul_face: &skia_safe::Typeface| {
        let mut surface = surfaces::raster_n32_premul((200, 90)).unwrap();
        let canvas = surface.canvas();
        canvas.clear(Color::TRANSPARENT);
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::BLACK);
        for ((ch, offset), shear) in "A평A".chars().zip(positions).zip(shears) {
            let source = if ch == '평' { hangul_face } else { &latin };
            let mut font = Font::new(source.clone(), 32.0);
            font.set_edging(font::Edging::AntiAlias);
            font.set_hinting(skia_safe::FontHinting::None);
            font.set_subpixel(true);
            font.set_skew_x(-(shear as f32));
            canvas.draw_str(ch.to_string(), (offset as f32, 51.0), &font, &paint);
        }
        let image = surface.image_snapshot();
        let info = ImageInfo::new((200, 90), ColorType::RGBA8888, AlphaType::Unpremul, None);
        let mut pixels = vec![0; 200 * 90 * 4];
        assert!(image.read_pixels(
            &info,
            &mut pixels,
            200 * 4,
            (0, 0),
            skia_safe::image::CachingHint::Disallow
        ));
        image::RgbaImage::from_raw(200, 90, pixels).unwrap()
    };
    assert_ne!(
        reference(&hangul),
        reference(&fallback),
        "원 프로그램과 대체 프로그램을 구별해야 한다"
    );
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[LATIN.to_vec(), HANGUL.to_vec(), FALLBACK.to_vec()])
        .unwrap()
        .with_font_family_alias("Haansoft Batang", &fallback.family_name())
        .unwrap();
    for profile in [RenderProfile::Screen, RenderProfile::Print] {
        let tree = PageLayerTree::with_profile(
            200.0,
            90.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 200.0, 90.0),
                None,
                vec![PaintOp::equation(
                    BoundingBox::new(0.0, 0.0, 180.0, 80.0),
                    equation.clone(),
                )],
            ),
            profile,
        );
        let actual = image::load_from_memory(&renderer.render_png(&tree).unwrap())
            .unwrap()
            .to_rgba8();
        assert_eq!(
            actual,
            reference(&hangul),
            "같은 원 family의 나머지 프로그램을 먼저 소비해야 한다: {profile:?}"
        );
    }
}
