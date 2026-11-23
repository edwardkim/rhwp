//! 검사 유형: 관계형 — 원본 HFT 글머리표의 전각 전진과 명시 공급자의 실제 paint 소유를 검사한다.
//! 주석 갱신: 2026-11-21 04:10:34 KST.
//! 화학 원본 72쪽 보정 전 Native 99.38114% / fresh WASM 99.43167%.
//! 96 DPI·compat2022·PDFium·Hancom PDF Print·색상 경계 보정 전 2px raw다.
//! 독립 2020 Print의 T11 U+2022는 1000/1000em이며 뒤 본문은 그 한 칸 뒤에 속한다.
//! 91쪽은 기존 글꼴 조건에서 Native 99.75467% → 99.99166%로 확인했다.
//! 91쪽의 T11/T18은 전진과 별개로 원형 잉크의 크기와 장평 적용 위치를 검증한다.
//! 절대 좌표를 고정하지 않는다. 공급 순서 검사는 기존 fixture와 Adobe 배포 원본을 공급한다.
//! 개인 글꼴 설치·비공개 프로그램·OS의 우연한 fallback을 요구하지 않는다.

use rhwp::DocumentCore;

fn chemical_document() -> DocumentCore {
    DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue6782/1480000-201900042-chemical-labeling-standards.hwp"
    ))
    .unwrap()
}

#[test]
fn original_legacy_bullets_keep_one_em_before_the_following_text() {
    let core = chemical_document();
    assert_eq!(core.page_count(), 103);
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(71, r#"{"maxCharacters":4096}"#)
            .unwrap(),
    )
    .unwrap();
    let mut paragraphs = std::collections::BTreeSet::new();
    for record in trace["records"].as_array().unwrap() {
        if record["source"]["character"] != "•" || record["document"]["face"] != "한양중고딕"
        {
            continue;
        }
        assert_eq!(record["document"]["altType"], 2);
        let shape = record["source"]["charShapeId"].as_u64().unwrap() as usize;
        let slot = record["document"]["languageSlot"].as_u64().unwrap() as usize;
        let style = &core.document().doc_info.char_shapes[shape];
        let size = f64::from(style.base_size) * f64::from(style.relative_sizes[slot]) / 100.0;
        let advance = record["layoutMetric"]["baseAdvanceHwpunit"]
            .as_f64()
            .unwrap();
        assert!(
            (advance / size - 1.0).abs() <= 1.0 / size,
            "독립 Print의 한 칸을 표시 glyph 결측으로 반각 추정하면 뒤 본문이 당겨진다: {advance}/{size}"
        );
        paragraphs.insert(record["source"]["nestedPath"].to_string());
    }
    assert_eq!(paragraphs.len(), 5, "다섯 본문 문단의 표식 소속·수량");
}

#[test]
fn original_legacy_sinmyeongjo_bullets_keep_their_print_ink_inside_the_source_slot() {
    use rhwp::{paint::RenderProfile, renderer::render_tree::RenderNodeType};

    let core = chemical_document();
    let tree = core.build_page_render_tree(90).unwrap();
    let svg = core
        .render_page_svg_layer_with_profile_native(90, RenderProfile::Print)
        .unwrap();
    let ellipses: Vec<_> = svg
        .split("<ellipse ")
        .skip(1)
        .filter_map(|part| part.split_once('>').map(|(tag, _)| tag))
        .collect();
    let coordinate = |tag: &str, attribute: &str| -> Option<f64> {
        let value = tag.split_once(&format!("{attribute}=\""))?.1;
        value.split_once('"')?.0.parse().ok()
    };
    let mut pending = vec![&tree.root];
    let mut regular = 0;
    let mut bold = 0;
    while let Some(node) = pending.pop() {
        pending.extend(&node.children);
        let RenderNodeType::TextRun(run) = &node.node_type else {
            continue;
        };
        if run.text != "•" || !run.style.source_font_is_hft {
            continue;
        }
        let em = run.style.font_size;
        let width = em * run.style.ratio;
        let extra_stroke = if run.style.bold { 0.01 } else { 0.0 };
        let expected = [
            ("cx", node.bbox.x + width * 0.501375),
            ("cy", node.bbox.y + run.baseline - em * 0.348),
            ("rx", width * (0.185625 + extra_stroke)),
            ("ry", em * (0.186 + extra_stroke)),
        ];
        assert!(
            ellipses.iter().any(|tag| expected.iter().all(
                |(key, value)| coordinate(tag, key).is_some_and(|px| (px - value).abs() < 0.05)
            )),
            "기호 원점 {:?}에 독립 Print와 같은 잉크가 있어야 한다",
            node.bbox
        );
        if run.style.bold {
            bold += 1;
        } else {
            regular += 1;
        }
    }
    assert_eq!((regular, bold), (11, 1));
}

#[cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]
fn supplied_core(explicit: bool) -> DocumentCore {
    use rhwp::renderer::font_environment::FontEnvironment;
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "A€").unwrap();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        for font in fonts {
            font.name = "HY중고딕".into();
        }
    }
    core.set_document(document);
    if explicit {
        core.set_font_environment(Some(
            FontEnvironment::from_json(
                r#"{"id":"explicit-glyph-program-order","substitutions":{},"glyphFallbacks":{"HY중고딕":["Source Code Pro"]}}"#,
            )
            .unwrap(),
        ))
        .unwrap();
    }
    core
}

#[cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]
fn supplied_renderer(with_alias_fallback: bool) -> rhwp::renderer::skia::SkiaLayerRenderer {
    use rhwp::renderer::skia::SkiaLayerRenderer;
    let primary = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
    let supplier = include_bytes!("../fixtures/fonts/source-code-pro/SourceCodePro-Regular.otf");
    let generic = include_bytes!("../fixtures/fonts/source-sans-3/SourceSans3-3.052-Regular.otf");
    let primary_face = ttf_parser::Face::parse(primary, 0).unwrap();
    assert!(primary_face.glyph_index('A').is_some());
    assert!(primary_face.glyph_index('€').is_none());
    for program in [supplier.as_slice(), generic.as_slice()] {
        let face = ttf_parser::Face::parse(program, 0).unwrap();
        let euro = face.glyph_index('€').expect("배포 원본의 유로 기호");
        assert!(face.glyph_bounding_box(euro).is_some());
    }
    let mut renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[primary.to_vec(), supplier.to_vec(), generic.to_vec()])
        .unwrap()
        .with_font_family_alias("HY중고딕", "RHWP Kerning Boundary")
        .unwrap();
    if with_alias_fallback {
        renderer = renderer
            .with_font_family_alias("HCR Dotum", "Source Sans 3")
            .unwrap();
    }
    renderer
}

#[cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]
#[test]
fn explicit_supplier_owns_native_trace_and_actual_paint_before_generic_aliases() {
    use rhwp::{paint::RenderProfile, renderer::layer_renderer::LayerRasterRenderer};
    let core = supplied_core(true);
    let renderer = supplied_renderer(true);
    let trace: serde_json::Value = serde_json::from_str(
        &renderer
            .get_font_decision_trace(&core, 0, r#"{"maxCharacters":16}"#)
            .unwrap(),
    )
    .unwrap();
    let records = trace["records"].as_array().unwrap();
    for (ch, supplier) in [('A', "RHWP Kerning Boundary"), ('€', "Source Code Pro")] {
        let record = records
            .iter()
            .find(|record| record["source"]["character"] == ch.to_string())
            .unwrap();
        assert_eq!(record["paint"]["native"]["resolved"], supplier);
    }
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    assert_eq!(
        renderer.render_png(&tree).unwrap(),
        supplied_renderer(false).render_png(&tree).unwrap(),
        "선언하지 않은 OS 별칭의 glyph가 명시 공급자의 실제 잉크를 바꾸지 않는다"
    );
}

#[cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]
#[test]
fn ordinary_selection_keeps_its_existing_alias_fallback() {
    let core = supplied_core(false);
    let trace: serde_json::Value = serde_json::from_str(
        &supplied_renderer(true)
            .get_font_decision_trace(&core, 0, r#"{"maxCharacters":16}"#)
            .unwrap(),
    )
    .unwrap();
    let record = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["source"]["character"] == "€")
        .unwrap();
    assert_eq!(record["paint"]["native"]["resolved"], "Source Sans 3");
}
