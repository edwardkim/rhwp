//! 검사 유형: 관계형. 합성 글꼴 프로그램의 선택·대체 계약을 검증한다.
//! 주석 갱신: 2026-10-31T07:12:47+09:00; 직전 파일 커밋: 2026-10-31T03:34:36+09:00 (`f3a88f72646d7682ae48804e78332f5dea1f34f4`).
//! 실제 입력 `samples/hwp3-sample10.hwp`, 한컴 2020 Print, 96 DPI, 2px 이웃 raw.
//! 2026-10-30 측정: 1쪽 Native/fresh WASM 각 99.96%; 전 763쪽 최저 각 99.41% / 99.40%.
//! 근거: `mydocs/pr/assets/issue_8371/sample10-native.tsv`, `sample10-wasm.tsv`, `sample10-validation.md`.
//! 합성 fixture의 계약 검사와 실제 문서의 한컴 출력 판독은 별도다.

use rhwp::document_core::DocumentCore;
use rhwp::model::style::Font;

const FAMILY: &str = "RHWP HFT Outline Fixture";
const CFF: &[u8] = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");

#[test]
fn romanized_myeongjo_names_keep_serif_fallback() {
    use rhwp::renderer::generic_fallback;
    for name in ["HY Sinmyeongjo", "Sinmyeong Jungmyeongjo", "AppleMyungjo"] {
        assert!(generic_fallback(name).ends_with(",serif"), "{name}");
    }
    assert!(generic_fallback("Sinmyeong Junggothic").ends_with(",sans-serif"));
}

fn synthetic_cff_for(character: char) -> Vec<u8> {
    let mut cff = CFF.to_vec();
    let count = u16::from_be_bytes([cff[4], cff[5]]) as usize;
    let cmap = (0..count)
        .find_map(|index| {
            let pos = 12 + index * 16;
            (cff.get(pos..pos + 4) == Some(b"cmap"))
                .then(|| u32::from_be_bytes(cff[pos + 8..pos + 12].try_into().unwrap()) as usize)
        })
        .unwrap();
    let sub = cmap + u32::from_be_bytes(cff[cmap + 8..cmap + 12].try_into().unwrap()) as usize;
    for offset in [sub + 14, sub + 20] {
        cff[offset..offset + 2].copy_from_slice(&(character as u16).to_be_bytes());
    }
    cff[sub + 24..sub + 26].copy_from_slice(&1u16.wrapping_sub(character as u16).to_be_bytes());
    cff
}

#[test]
fn verified_hft_advance_survives_a_supplied_display_otf() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;
    use rhwp::renderer::TextStyle;

    let mut fonts = NativePuaFonts::default();
    fonts
        .register_otf("synthetic", false, false, &synthetic_cff_for('A'), 0)
        .unwrap();
    let mut style = TextStyle {
        font_family: "HCI Poppy".into(),
        font_size: 20.0,
        source_font_is_hft: true,
        ..Default::default()
    };
    let measure = |style: &TextStyle| EmbeddedTextMeasurer.compute_char_positions("A", style)[1];
    let source_width = measure(&style);
    style.native_otf_metrics = fonts.get_otf("synthetic", false, false);
    assert_eq!(measure(&style), source_width, "원본 HFT 전진을 보존한다");
    style.source_font_is_hft = false;
    assert_ne!(
        measure(&style),
        source_width,
        "명시 OTF는 해당 프로그램의 전진을 사용한다"
    );
}

#[test]
fn hft_mapped_symbol_retains_source_advance_with_display_otf() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;
    use rhwp::renderer::supplemental_metrics::{
        MetricBackend, MetricContext, SupplementalMetric, SupplementalMetricStore,
    };
    use rhwp::renderer::TextStyle;

    let mut fonts = NativePuaFonts::default();
    fonts
        .register_otf("synthetic", false, false, &synthetic_cff_for('═'), 0)
        .unwrap();
    let mut style = TextStyle {
        font_family: "synthetic".into(),
        font_size: 20.0,
        source_font_is_hft: true,
        ..Default::default()
    };
    let measure =
        |text: &str, style: &TextStyle| EmbeddedTextMeasurer.compute_char_positions(text, style)[1];
    let source_width = measure("\u{F0827}", &style);
    style.native_otf_metrics = fonts.get_otf("synthetic", false, false);
    assert_eq!(
        measure("\u{F0827}", &style),
        source_width,
        "원본 기호 전진을 보존한다"
    );
    assert_ne!(
        measure("═", &style),
        source_width,
        "실제 문자는 OTF 전진을 사용한다"
    );
    let context = MetricContext {
        document_generation: 1,
        font_generation: 1,
        backend: MetricBackend::Canvas2d,
    };
    let mut store = SupplementalMetricStore::new(context);
    style.native_otf_metrics = None;
    let canvas =
        SupplementalMetric::backend_measured(&style, "═", "synthetic".into(), 18.0).unwrap();
    store.replace(context, vec![canvas]).unwrap();
    store.bind_style(context, &mut style).unwrap();
    assert_eq!(
        measure("\u{F0827}", &style),
        source_width,
        "Canvas 측정에서도 원본 전진을 보존한다"
    );
    assert_eq!(
        measure("═", &style),
        18.0,
        "실제 문자는 Canvas 측정을 사용한다"
    );
}

#[test]
fn hwp3_punctuation_uses_the_latin_program_only_when_its_glyph_is_needed() {
    use rhwp::model::paragraph::{CharShapeRef, LineSeg};
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn families(node: &RenderNode, character: char, out: &mut Vec<String>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains(character) {
                out.push(run.style.font_family.clone());
            }
        }
        for child in &node.children {
            families(child, character, out);
        }
    }
    let input = include_bytes!("../../samples/hwp3-sample10.hwp");
    let mut core = DocumentCore::from_bytes(input).unwrap();
    let mut document = core.document().clone();
    document.sections.truncate(1);
    document.sections[0].paragraphs.truncate(1);
    let para = &mut document.sections[0].paragraphs[0];
    para.text = "가(가".into();
    para.char_count = 4;
    para.char_offsets = vec![0, 1, 2];
    para.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    para.line_segs = vec![LineSeg {
        text_start: 0,
        line_height: 1000,
        text_height: 1000,
        baseline_distance: 800,
        ..Default::default()
    }];
    para.controls.clear();
    for (slot, faces) in document.doc_info.font_faces.iter_mut().enumerate() {
        if let Some(face) = faces.first_mut() {
            face.name = if slot == 1 {
                "fixture-latin"
            } else {
                "fixture-korean"
            }
            .into();
            face.alt_type = 2;
        }
    }
    for shape in &mut document.doc_info.char_shapes {
        shape.font_ids = [0; 7];
        shape.ratios = [100; 7];
        shape.spacings = [0; 7];
        shape.relative_sizes = [100; 7];
    }
    core.set_document(document);
    let korean_missing = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
    let korean_present = include_bytes!("../fixtures/fonts/RHWPEquationCJKRegular.ttf");
    let latin = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf");
    for (korean, latin_present, expected) in [
        (korean_missing.as_slice(), true, "fixture-latin"),
        (korean_missing.as_slice(), false, "fixture-korean"),
        (korean_present.as_slice(), true, "fixture-korean"),
    ] {
        core.prepare_native_otf_fonts_with(|request| match request.family.as_str() {
            "fixture-korean" => Some((korean.to_vec(), 0)),
            "fixture-latin" if latin_present => Some((latin.to_vec(), 0)),
            _ => None,
        })
        .unwrap();
        let tree = core.build_page_render_tree(0).unwrap();
        let mut punctuation = Vec::new();
        families(&tree.root, '(', &mut punctuation);
        assert_eq!(punctuation.len(), 1, "구두점은 한 번 표시된다");
        assert!(
            punctuation[0].contains(expected),
            "구두점의 실제 글꼴 소유: {:?}",
            punctuation
        );
        let mut hangul = Vec::new();
        families(&tree.root, '가', &mut hangul);
        assert!(
            hangul
                .iter()
                .all(|family| family.contains("fixture-korean")),
            "한글 글꼴 소유를 유지한다"
        );
    }
}

#[cfg(all(
    not(target_arch = "wasm32"),
    feature = "native-skia",
    target_os = "linux"
))]
#[test]
fn skia_synthetic_bold_selects_both_split_family_glyphs() {
    use rhwp::renderer::skia::SkiaLayerRenderer;
    use std::time::{SystemTime, UNIX_EPOCH};
    const MARKER: &str = "RHWP_TEST_SPLIT_CFF_CHILD";
    if std::env::var_os(MARKER).is_none() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("rhwp-split-bold-{}-{suffix}", std::process::id()));
        std::fs::create_dir_all(dir.join("fonts")).unwrap();
        std::fs::create_dir_all(dir.join("cache")).unwrap();
        std::fs::write(dir.join("fonts/first.otf"), CFF).unwrap();
        std::fs::write(dir.join("fonts/second.otf"), synthetic_cff_for('나')).unwrap();
        let conf = format!(
            r#"<?xml version="1.0"?><!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd"><fontconfig><dir>{}/fonts</dir><cachedir>{}/cache</cachedir><match target="font"><test name="weight" compare="less_eq"><const>medium</const></test><test target="pattern" name="weight" compare="more_eq"><const>bold</const></test><edit name="embolden" mode="assign"><bool>true</bool></edit><edit name="weight" mode="assign"><const>bold</const></edit></match></fontconfig>"#,
            dir.display(),
            dir.display()
        );
        let config = dir.join("fonts.conf");
        std::fs::write(&config, conf).unwrap();
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "hft_outline_font_contract::skia_synthetic_bold_selects_both_split_family_glyphs",
                "--nocapture",
            ])
            .env(MARKER, "1")
            .env("FONTCONFIG_FILE", config)
            .output()
            .unwrap();
        std::fs::remove_dir_all(dir).unwrap();
        assert!(
            result.status.success(),
            "격리된 분할 글꼴 선택: {}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나").unwrap();
    let mut document = core.document().clone();
    for faces in &mut document.doc_info.font_faces {
        for face in faces {
            face.name = FAMILY.into();
        }
    }
    for shape in &mut document.doc_info.char_shapes {
        shape.bold = true;
    }
    core.set_document(document);
    let renderer = SkiaLayerRenderer::new();
    let trace: serde_json::Value = serde_json::from_str(
        &renderer
            .get_font_decision_trace(&core, 0, r#"{"maxCharacters":16}"#)
            .unwrap(),
    )
    .unwrap();
    for character in ["가", "나"] {
        let record = trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["source"]["character"] == character)
            .expect("입력 글자마다 글꼴 선택 기록");
        assert_eq!(record["paint"]["native"]["source"], "system");
        assert_eq!(
            record["paint"]["native"]["resolved"], FAMILY,
            "각 글리프를 같은 분할 family에서 선택한다"
        );
    }
}

#[test]
fn registering_paint_source_preserves_document_and_replaces_unsupported_programs() {
    let mut core = DocumentCore::new_empty();
    let mut document = core.document().clone();
    document.doc_info.font_faces = vec![
        vec![Font {
            name: FAMILY.into(),
            alt_type: 2,
            ..Default::default()
        }];
        7
    ];
    core.set_document(document);
    let before = core.get_document_info();
    let document_before = format!("{:?}", core.document());
    let saved_before = rhwp::serializer::serialize_document(core.document()).unwrap();
    assert!(!core
        .register_hft_outline_font_native("Absent family", CFF, 0)
        .unwrap());
    assert!(core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
    let ttf = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    assert!(core
        .register_hft_outline_font_native(FAMILY, ttf, 0)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_native(FAMILY, ttf, 0)
        .unwrap());
    assert!(core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
    // FontFace가 허용하지만 SFNT parser는 읽지 않는 포맷은 원래 painter에 맡긴다.
    let woff = include_bytes!("../../assets/fonts/NotoSansKR-Bold.woff2");
    assert!(core
        .register_hft_outline_font_native(FAMILY, woff, 0)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_native(FAMILY, woff, 0)
        .unwrap());
    assert!(core
        .register_hft_outline_font_native(FAMILY, &[], 0)
        .is_err());
    assert!(!core
        .register_hft_outline_font_native(FAMILY, CFF, 999)
        .unwrap());
    core.clear_text_font_family_native(FAMILY);
    assert!(core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
    assert_eq!(
        core.get_document_info(),
        before,
        "paint 등록은 저장 문서·쪽수를 바꾸지 않는다"
    );
    assert_eq!(
        format!("{:?}", core.document()),
        document_before,
        "저장 IR·raw 상태를 보존한다"
    );
    assert_eq!(
        rhwp::serializer::serialize_document(core.document()).unwrap(),
        saved_before,
        "paint registry는 실제 저장 HWP 바이트에 반영되지 않는다"
    );

    // raw_stream を持つ実際の HWP 入力でも paint セッションは保存に影響しない。
    let mut opened = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp")).unwrap();
    assert!(opened
        .document()
        .sections
        .iter()
        .any(|section| section.raw_stream.is_some()));
    let original_ir = format!("{:?}", opened.document());
    let original_bytes = rhwp::serializer::serialize_document(opened.document()).unwrap();
    opened
        .register_text_font_glyph_coverage_native(FAMILY, CFF, 0)
        .unwrap();
    opened.clear_text_font_family_native(FAMILY);
    assert_eq!(format!("{:?}", opened.document()), original_ir);
    assert_eq!(
        rhwp::serializer::serialize_document(opened.document()).unwrap(),
        original_bytes,
        "実際の HWP を開いても paint セッションの変更は保存されない"
    );
}

#[cfg(feature = "native-skia")]
mod native {
    use super::*;
    use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    use rhwp::renderer::raster_pixels::RasterPixels;
    use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
    use rhwp::renderer::skia::SkiaLayerRenderer;
    use rhwp::renderer::TextStyle;

    fn paint(y: f64, hft: bool, profile: RenderProfile) -> RasterPixels {
        let run = TextRunNode {
            text: "가".into(),
            style: TextStyle {
                font_family: FAMILY.into(),
                font_size: 32.0,
                source_font_is_hft: hft,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 32.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        };
        let mut tree = PageLayerTree::new(
            64.0,
            64.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 64.0),
                None,
                vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
            ),
        );
        tree.profile = profile;
        SkiaLayerRenderer::new()
            .with_font_paths(&[
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts")
            ])
            .render_pixels_with_options(&tree, RasterRenderOptions::default())
            .unwrap()
    }

    fn center(image: &RasterPixels) -> f64 {
        let mut mass = 0.0;
        let mut weighted = 0.0;
        for (y, row) in image.bytes.chunks_exact(image.stride).enumerate() {
            for p in row[..image.width as usize * 4].chunks_exact(4) {
                mass += f64::from(p[3]);
                weighted += y as f64 * f64::from(p[3]);
            }
        }
        assert!(mass > 0.0, "실제 사각형 잉크가 있어야 한다");
        weighted / mass
    }

    #[test]
    fn print_hft_contour_preserves_quarter_pixel_baseline() {
        let original = center(&paint(4.1, true, RenderProfile::Print));
        let quarter = center(&paint(4.35, true, RenderProfile::Print));
        let whole = center(&paint(5.1, true, RenderProfile::Print));
        // 글꼴 계산 결과를 기대값으로 복사하지 않고 입력의 1/4px 이동을 검사한다.
        assert!(
            (quarter - original - 0.25).abs() <= 0.08,
            "{}",
            quarter - original
        );
        assert!((whole - original - 1.0).abs() <= 0.02);
    }

    #[test]
    fn screen_and_non_hft_keep_existing_font_raster() {
        for y in [4.1, 4.35] {
            assert_eq!(
                paint(y, true, RenderProfile::Screen),
                paint(y, false, RenderProfile::Screen),
                "화면 출력은 HFT 전용 Print 윤곽에 영향을 받지 않는다"
            );
        }
        let original = center(&paint(4.1, false, RenderProfile::Print));
        let quarter = center(&paint(4.35, false, RenderProfile::Print));
        let whole = center(&paint(5.1, false, RenderProfile::Print));
        assert!(
            whole > original && quarter >= original && quarter <= whole,
            "일반 글꼴 Print의 부분 이동은 전체 이동 사이에 있어야 한다"
        );
    }

    #[test]
    fn split_cff_uses_selected_glyph_outline_in_print_profile() {
        // 첫 face는 가, 같은 family의 두 번째 face는 나만 제공한다.
        let mut second = CFF.to_vec();
        let count = u16::from_be_bytes([second[4], second[5]]) as usize;
        let cmap = (0..count)
            .find_map(|index| {
                let pos = 12 + index * 16;
                (second.get(pos..pos + 4) == Some(b"cmap")).then(|| {
                    u32::from_be_bytes(second[pos + 8..pos + 12].try_into().unwrap()) as usize
                })
            })
            .unwrap();
        let sub =
            cmap + u32::from_be_bytes(second[cmap + 8..cmap + 12].try_into().unwrap()) as usize;
        assert_eq!(u16::from_be_bytes([second[sub], second[sub + 1]]), 4);
        for offset in [sub + 14, sub + 20] {
            second[offset..offset + 2].copy_from_slice(&('나' as u16).to_be_bytes());
        }
        second[sub + 24..sub + 26].copy_from_slice(&1u16.wrapping_sub('나' as u16).to_be_bytes());
        let renderer = SkiaLayerRenderer::new()
            .try_with_font_data(&[CFF.to_vec(), second])
            .unwrap();
        let paint_at = |y| {
            let run = TextRunNode {
                text: "나".into(),
                style: TextStyle {
                    font_family: FAMILY.into(),
                    font_size: 32.0,
                    source_font_is_hft: true,
                    ..Default::default()
                },
                char_shape_id: None,
                para_shape_id: None,
                section_index: None,
                para_index: None,
                char_start: None,
                cell_context: None,
                is_para_end: false,
                is_line_break_end: false,
                rotation: 0.0,
                is_vertical: false,
                char_overlap: None,
                border_fill_id: 0,
                baseline: 32.0,
                field_marker: Default::default(),
                layout_positions: None,
                display_text: None,
            };
            let mut tree = PageLayerTree::new(
                64.0,
                64.0,
                LayerNode::leaf(
                    BoundingBox::new(0.0, 0.0, 64.0, 64.0),
                    None,
                    vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
                ),
            );
            tree.profile = RenderProfile::Print;
            center(
                &renderer
                    .render_pixels_with_options(&tree, RasterRenderOptions::default())
                    .unwrap(),
            )
        };
        let first = paint_at(4.1);
        let quarter = paint_at(4.35);
        let whole = paint_at(5.1);
        assert!(
            first < quarter && quarter < whole,
            "선택된 두 번째 CFF도 소수 원점 이동을 보존해야 한다: {first}, {quarter}, {whole}"
        );
    }
}

mod native_otf_program_contract {
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;

    fn is_cff_program(bytes: &[u8], index: u32) -> bool {
        ttf_parser::Face::parse(bytes, index)
            .is_ok_and(|face| face.tables().cff.is_some() || face.tables().cff2.is_some())
    }

    fn collection(first: &[u8], second: &[u8]) -> Vec<u8> {
        let mut result = b"ttcf\0\x01\0\0\0\0\0\x02\0\0\0\0\0\0\0\0".to_vec();
        for (index, source) in [first, second].into_iter().enumerate() {
            while !result.len().is_multiple_of(4) {
                result.push(0);
            }
            let start = u32::try_from(result.len()).unwrap();
            result[12 + index * 4..16 + index * 4].copy_from_slice(&start.to_be_bytes());
            let mut face = source.to_vec();
            let tables = u16::from_be_bytes([face[4], face[5]]) as usize;
            for table in 0..tables {
                let pos = 12 + table * 16 + 8;
                let offset = u32::from_be_bytes(face[pos..pos + 4].try_into().unwrap());
                face[pos..pos + 4].copy_from_slice(&(offset + start).to_be_bytes());
            }
            result.extend(face);
        }
        result
    }

    #[test]
    fn collection_uses_each_faces_actual_outline_and_retains_both_faces() {
        let cff = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
        let ttf = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
        let data = collection(cff, ttf);
        assert!(is_cff_program(&data, 0));
        assert!(!is_cff_program(&data, 1));
        assert!(!is_cff_program(&data, 2));
        let mut fonts = NativePuaFonts::default();
        fonts
            .register_otf("collection", false, false, &data, 0)
            .unwrap();
        fonts
            .register_otf("collection", false, false, &data, 1)
            .unwrap();
        let metrics = fonts.get_otf("collection", false, false).unwrap();
        assert!(metrics.advance_em('가').is_some());
        assert_eq!(metrics.advance_em('A'), None);
        assert!(
            metrics.has_glyph('A'),
            "두 번째 TTF face가 글리프를 공급한다"
        );
    }

    #[test]
    fn split_family_uses_the_program_that_contains_each_character() {
        // 저장소의 합성 CFF 글꼴은 가만 지원한다. 같은 글꼴의 cmap을 나로
        // 바꿔 서로 다른 문자를 공급하는 두 번째 프로그램을 메모리에서 만든다.
        let first = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
        let mut second = first.to_vec();
        let table_count = u16::from_be_bytes([second[4], second[5]]) as usize;
        let cmap = (0..table_count)
            .find_map(|index| {
                let pos = 12 + index * 16;
                (second.get(pos..pos + 4) == Some(b"cmap")).then(|| {
                    u32::from_be_bytes(second[pos + 8..pos + 12].try_into().unwrap()) as usize
                })
            })
            .unwrap();
        let sub =
            cmap + u32::from_be_bytes(second[cmap + 8..cmap + 12].try_into().unwrap()) as usize;
        assert_eq!(u16::from_be_bytes([second[sub], second[sub + 1]]), 4);
        // 첫 세그먼트의 end/start와 idDelta를 가에서 나로 옮긴다.
        for offset in [sub + 14, sub + 20] {
            assert_eq!(&second[offset..offset + 2], &[0xac, 0x00]);
            second[offset..offset + 2].copy_from_slice(&[0xb0, 0x98]);
        }
        let delta = 1u16.wrapping_sub('나' as u16).to_be_bytes();
        second[sub + 24..sub + 26].copy_from_slice(&delta);
        let mut fonts = NativePuaFonts::default();
        fonts
            .register_otf("synthetic", false, false, first, 0)
            .unwrap();
        fonts
            .register_otf("synthetic", false, false, &second, 0)
            .unwrap();
        let metrics = fonts.get_otf("synthetic", false, false).unwrap();
        assert!(metrics.advance_em('가').is_some());
        assert!(metrics.advance_em('나').is_some());
        assert!(metrics.advance_em('다').is_none());
    }

    #[test]
    fn earlier_true_type_glyph_blocks_a_later_cff_advance() {
        let ttf = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
        let mut cff = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf").to_vec();
        let count = u16::from_be_bytes([cff[4], cff[5]]) as usize;
        let cmap = (0..count)
            .find_map(|index| {
                let pos = 12 + index * 16;
                (cff.get(pos..pos + 4) == Some(b"cmap")).then(|| {
                    u32::from_be_bytes(cff[pos + 8..pos + 12].try_into().unwrap()) as usize
                })
            })
            .unwrap();
        let sub = cmap + u32::from_be_bytes(cff[cmap + 8..cmap + 12].try_into().unwrap()) as usize;
        assert_eq!(u16::from_be_bytes([cff[sub], cff[sub + 1]]), 4);
        for offset in [sub + 14, sub + 20] {
            cff[offset..offset + 2].copy_from_slice(&('A' as u16).to_be_bytes());
        }
        cff[sub + 24..sub + 26].copy_from_slice(&1u16.wrapping_sub('A' as u16).to_be_bytes());

        let mut before = NativePuaFonts::default();
        before.register_otf("same", false, false, ttf, 0).unwrap();
        before.register_otf("same", false, false, &cff, 0).unwrap();
        assert_eq!(
            before
                .get_otf("same", false, false)
                .unwrap()
                .advance_em('A'),
            None
        );

        let mut after = NativePuaFonts::default();
        after.register_otf("same", false, false, &cff, 0).unwrap();
        after.register_otf("same", false, false, ttf, 0).unwrap();
        assert!(after
            .get_otf("same", false, false)
            .unwrap()
            .advance_em('A')
            .is_some());
    }
}
