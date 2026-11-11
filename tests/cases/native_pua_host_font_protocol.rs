//! 관계형: 글꼴 소유자 등록과 해제의 세션 프로토콜 계약.
//! 주석 갱신: 2026-11-12T00:52:45+09:00; 직전 파일 커밋: 2026-11-09T21:28:10+09:00 (54b3f8b8bb00c8e51cd96494ce9dbf4a8f355c59).
//! 근거: tests/fixtures/fonts/RHWPHostFixture.LICENSE.md, tests/fixtures/fonts/RHWPEquationFonts.LICENSE.md.
//! 글꼴 등록·해제 전후의 세션 소유 관계 검사이며 페이지별 raw 측정은 대상이 아니다.
//! 실제 글꼴 요청 이름과 명시 exact 소유자의 세션 프로토콜 계약.
//! 실제 페이지 좌표나 한컴 피델리티의 기대값을 고정하지 않는다.
//! 작은 삼각형의 실제 입력은 중첩 표 17쪽, 독립 한컴 Print PDFium 96 DPI·색상 보정 전 2px raw다.
//! 원본: samples/basic/issue2007_nested_cell_pagination_42065.hwp; 기준 PDF와 입력 SHA는 보호 manifest에 있다.
//! 2026-11-12 Native 전쪽 최저 97.99083%, fresh WASM 전쪽 최저 98.05646%.
//! 8쪽 Native/fresh WASM 99.52149%/99.52472%, 9쪽 99.98562%/99.98759%.
//! 원 글리프·전진·해제 대체의 합성 관계 검사이며 실물 좌표는 고정하지 않는다.
//! 근거: mydocs/pr/assets/pr8896/protection/ 및 font-supply/README.md.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

#[test]
fn document_glyph_fallback_remains_bound_to_its_language_slot() {
    use rhwp::model::style::Font;
    use rhwp::renderer::font_environment::FontEnvironment;

    assert!(FontEnvironment::from_json(
        r#"{"id":"bad-slot","substitutions":{},"documentGlyphFallbacks":[{"languageSlot":7,"fontId":0,"source":"Scoped Fixture","faces":["HCR Batang"]}]}"#,
    ).is_err());

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    for fonts in &mut document.doc_info.font_faces {
        fonts[0] = Font {
            name: "Scoped Fixture".into(),
            alt_type: 1,
            ..Default::default()
        };
    }
    for shape in &mut document.doc_info.char_shapes {
        shape.font_ids = [0; 7];
    }
    core.set_document(document);
    core.insert_text_native(0, 0, 0, "가A").unwrap();
    let environment = FontEnvironment::from_json(
        r#"{"id":"slot-test","substitutions":{},"documentGlyphFallbacks":[{"languageSlot":0,"fontId":0,"source":"Scoped Fixture","faces":["HCR Batang"]}]}"#,
    ).unwrap();
    core.set_font_environment(Some(environment)).unwrap();
    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
            .unwrap(),
    )
    .unwrap();
    let records = trace["records"].as_array().unwrap();
    let names = |character: &str| {
        records
            .iter()
            .find(|item| item["source"]["character"] == character)
            .unwrap()["layoutName"]["cssFamilyChain"]
            .as_array()
            .unwrap()
    };
    assert!(names("가").iter().any(|name| name == "HCR Batang"));
    assert!(!names("A").iter().any(|name| name == "HCR Batang"));
}

#[cfg(feature = "native-skia")]
#[test]
fn exact_postscript_source_owns_its_true_type_metrics() {
    use rhwp::model::style::Font;
    use rhwp::renderer::skia::SkiaLayerRenderer;

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    document.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
    for fonts in &mut document.doc_info.font_faces {
        fonts[0] = Font {
            name: "RHWPEquationContract-LatinOnly".into(),
            alt_type: 1,
            ..Default::default()
        };
    }
    for shape in &mut document.doc_info.char_shapes {
        shape.font_ids = [0; 7];
    }
    core.set_document(document);
    core.insert_text_native(0, 0, 0, "A").unwrap();
    let source = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf").to_vec();
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[source])
        .unwrap();
    renderer.prepare_native_otf_fonts(&mut core).unwrap();
    let raw = core
        .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
        .unwrap();
    let trace: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let record = trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["source"]["character"] == "A")
        .unwrap();
    assert_eq!(record["layoutMetric"]["widthSource"], "nativeTrueType");
}

#[cfg(feature = "native-skia")]
#[test]
fn explicit_glyph_fallback_owns_advance_when_source_program_lacks_glyph() {
    use rhwp::model::style::Font;
    use rhwp::renderer::font_environment::FontEnvironment;
    use rhwp::renderer::skia::SkiaLayerRenderer;

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    document.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
    for fonts in &mut document.doc_info.font_faces {
        fonts[0] = Font {
            name: "RHWPEquationContract-LatinOnly".into(),
            alt_type: 1,
            ..Default::default()
        };
    }
    for shape in &mut document.doc_info.char_shapes {
        shape.font_ids = [0; 7];
    }
    core.set_document(document);
    core.insert_text_native(0, 0, 0, "A〇‧").unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"explicit-glyph-provider","substitutions":{},"glyphFallbacks":{"RHWPEquationContract-LatinOnly":["HCR Batang"]}}"#,
    ).unwrap())).unwrap();
    let source = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf").to_vec();
    assert!(ttf_parser::Face::parse(&source, 0)
        .unwrap()
        .glyph_index('〇')
        .is_none());
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[source])
        .unwrap();
    renderer.prepare_native_otf_fonts(&mut core).unwrap();
    let raw = core
        .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
        .unwrap();
    let trace: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let records = trace["records"].as_array().unwrap();
    let record = records
        .iter()
        .find(|r| r["source"]["character"] == "〇")
        .unwrap();
    assert_eq!(
        record["layoutMetric"]["widthSource"],
        "declaredGlyphFallbackMetric"
    );
    let source_record = records
        .iter()
        .find(|r| r["source"]["character"] == "A")
        .unwrap();
    assert_eq!(
        source_record["layoutMetric"]["widthSource"],
        "nativeTrueType"
    );
    let punctuation = records
        .iter()
        .find(|r| r["source"]["character"] == "‧")
        .unwrap();
    assert_ne!(
        punctuation["layoutMetric"]["widthSource"],
        "declaredGlyphFallbackMetric"
    );
}

#[cfg(feature = "native-skia")]
#[test]
fn missing_source_glyph_does_not_inherit_the_paint_fallback_advance() {
    use rhwp::model::style::{Font, SubstFont};
    use rhwp::renderer::skia::SkiaLayerRenderer;

    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut document = core.document().clone();
    document.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
    for fonts in &mut document.doc_info.font_faces {
        fonts[0] = Font {
            name: "RHWP Equation Contract".into(),
            alt_type: 1,
            subst_font: Some(SubstFont {
                face: "Noto Sans KR".into(),
                font_type: 1,
                ..Default::default()
            }),
            ..Default::default()
        };
    }
    for shape in &mut document.doc_info.char_shapes {
        shape.font_ids = [0; 7];
    }
    core.set_document(document);
    core.insert_text_native(0, 0, 0, "A€").unwrap();
    let source = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf").to_vec();
    let fallback = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf").to_vec();
    let primary = ttf_parser::Face::parse(&source, 0).unwrap();
    let alternate = ttf_parser::Face::parse(&fallback, 0).unwrap();
    assert!(primary.glyph_index('€').is_none());
    assert!(alternate.glyph_index('€').is_some());
    let trace = |core: &DocumentCore, ch: &str| -> serde_json::Value {
        let raw = core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":16}"#)
            .unwrap();
        let trace: serde_json::Value = serde_json::from_str(&raw).unwrap();
        trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["source"]["character"] == ch)
            .unwrap()
            .clone()
    };
    let before = trace(&core, "€");
    SkiaLayerRenderer::new()
        .try_with_font_data(&[source, fallback])
        .unwrap()
        .prepare_native_otf_fonts(&mut core)
        .unwrap();
    let after = trace(&core, "€");
    assert_eq!(
        after["layoutMetric"]["finalAdvanceHwpunit"],
        before["layoutMetric"]["finalAdvanceHwpunit"]
    );
    assert_ne!(after["layoutMetric"]["widthSource"], "nativeTrueType");
    assert_eq!(
        trace(&core, "A")["layoutMetric"]["widthSource"],
        "nativeTrueType"
    );
}

#[test]
fn same_family_hft_and_truetype_requests_keep_distinct_source_programs() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "A가").unwrap();
    let mut document = core.document().clone();
    // HWPX의 명시적 글꼴 유형을 시험한다. HWP5 하위 비트는 유형 증거가 아니다.
    document.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
    document.doc_info.font_faces = (0..7)
        .map(|lang| {
            vec![rhwp::model::style::Font {
                name: "RHWP unlisted source fixture".into(),
                alt_type: if lang == 0 { 2 } else { 1 },
                ..Default::default()
            }]
        })
        .collect();
    for shape in &mut document.doc_info.char_shapes {
        shape.font_ids = [0; 7];
    }
    core.set_document(document);
    let before = format!("{:?}", core.document());
    let requests = core.native_otf_font_requests();
    assert!(
        requests.iter().any(|request| request.source_ttf),
        "TrueType 요청이 없음: {requests:?}"
    );
    assert!(
        requests.iter().any(|request| !request.source_ttf),
        "HFT 요청이 없음: {requests:?}"
    );
    let ttf = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
    let cff = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
    assert!(core
        .prepare_native_otf_fonts_with(|request| {
            Some((
                if request.source_ttf {
                    ttf.to_vec()
                } else {
                    cff.to_vec()
                },
                0,
            ))
        })
        .unwrap());
    assert_eq!(format!("{:?}", core.document()), before);
}

#[test]
fn otf_session_supply_preserves_document_and_saved_bytes() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가").unwrap();
    let mut document = core.document().clone();
    for font in document.doc_info.font_faces.iter_mut().flatten() {
        font.alt_type = 2;
    }
    core.set_document(document);
    let before = format!("{:?}", core.document());
    let saved = core.export_hwp_native().unwrap();
    assert!(!core.native_otf_font_requests().is_empty());
    let font = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
    assert!(core
        .prepare_native_otf_fonts_with(|_| Some((font.to_vec(), 0)))
        .unwrap());
    assert_eq!(format!("{:?}", core.document()), before);
    assert_eq!(core.export_hwp_native().unwrap(), saved);
    assert!(core.prepare_native_otf_fonts_with(|_| None).unwrap());
    assert_eq!(format!("{:?}", core.document()), before);
    assert_eq!(core.export_hwp_native().unwrap(), saved);
}

#[test]
fn pua_supply_clear_preserves_explicit_exact_font_owner() {
    let mut core = DocumentCore::new_empty();
    let bytes = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    let before: serde_json::Value = serde_json::from_str(
        &core
            .register_exact_font_source_native(0, 0, bytes, 0)
            .unwrap(),
    )
    .unwrap();
    let document_before = format!("{:?}", core.document());
    let saved_before = core.export_hwp_native().unwrap();
    let literal_space_program = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf");
    assert!(core
        .register_literal_figure_space_font(
            "RHWP Equation Latin Only",
            false,
            false,
            literal_space_program,
            0,
        )
        .unwrap());
    assert_eq!(
        format!("{:?}", core.document()),
        document_before,
        "리터럴 공백의 세션 프로그램 등록은 문서 IR을 바꾸지 않는다"
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        saved_before,
        "실제 숫자 glyph를 공급해도 원본 HWP 저장 바이트를 바꾸지 않는다"
    );
    core.clear_native_pua_fonts();
    let after: serde_json::Value = serde_json::from_str(
        &core
            .register_exact_font_source_native(0, 0, bytes, 0)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        after["status"], "already-registered",
        "PUA만 해제해도 명시 exact 소유자는 동일한 채 유지한다"
    );
    assert_eq!(before["handle"], after["handle"]);
    assert_eq!(before["registry"], after["registry"]);
    assert_eq!(
        format!("{:?}", core.document()),
        document_before,
        "세션 해제는 문서 IR과 원본 raw 상태를 바꾸지 않는다"
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        saved_before,
        "원 PUA 공급 상태는 저장 HWP 바이트에 반영되지 않는다"
    );
}

#[test]
fn host_font_request_names_are_not_css_quoted() {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp"))
        .expect("구역과 첫 문단이 있는 실제 빈 문서");
    core.insert_text_native(0, 0, 0, "\u{f0854}").unwrap();
    let requests = core.native_pua_font_requests();
    assert!(!requests.is_empty(), "원 PUA의 실제 글꼴 요청");
    for request in requests {
        assert_eq!(request.paint_families.first(), Some(&request.family));
        assert!(request
            .paint_families
            .iter()
            .all(|family| !family.starts_with('\'')));
    }
}

#[test]
fn source_pointer_requests_its_program_without_reclassifying_unicode_pointer() {
    for (text, requires_source_program) in [
        ("\u{f02fb}", true),
        ("▸", false),
        ("\u{f02fc}", true),
        ("►", false),
        ("\u{f031c}", true),
        ("■", false),
        ("\u{f03a7}", true),
        ("⊟", false),
        ("\u{f03a8}", true),
        ("⊞", false),
        ("\u{f0848}", true),
        ("━", false),
    ] {
        let mut core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp"))
            .expect("실제 빈 문서");
        core.insert_text_native(0, 0, 0, text).unwrap();
        let before = format!("{:?}", core.document());
        let requests = core.native_pua_font_requests();
        assert_eq!(
            !requests.is_empty(),
            requires_source_program,
            "원 PUA와 일반 Unicode 기호의 공급 요청 계약: {text:?}"
        );
        assert_eq!(
            format!("{:?}", core.document()),
            before,
            "글꼴 공급 요청은 원문과 문서 속성을 변경하지 않는다"
        );
    }
}

/// 저장소가 직접 만든 MIT 합성 윤곽의 cmap만 메모리에서 바꾼다.
/// 외부 글꼴·새 배포 파일 없이 원 glyph가 실제로 있는 공급 프로그램을 만든다.
fn synthetic_symbol_program(symbol: char) -> Vec<u8> {
    let source = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    let face = ttf_parser::Face::parse(source, 0).unwrap();
    let glyph = face.glyph_index('A').unwrap();
    assert_eq!(face.units_per_em(), 1000);
    assert_eq!(face.glyph_hor_advance(glyph), Some(900));
    let mut bytes = source.to_vec();
    let table_count = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    let record = |tag: &[u8; 4]| {
        (0..table_count)
            .map(|index| 12 + index * 16)
            .find(|&offset| &source[offset..offset + 4] == tag)
            .unwrap()
    };
    let cmap_record = record(b"cmap");
    let head_record = record(b"head");
    let head_offset = u32::from_be_bytes(
        source[head_record + 8..head_record + 12]
            .try_into()
            .unwrap(),
    ) as usize;
    let mut cmap = Vec::new();
    for value in [0u16, 1, 3, 10] {
        cmap.extend(value.to_be_bytes());
    }
    cmap.extend(12u32.to_be_bytes());
    cmap.extend(12u16.to_be_bytes());
    cmap.extend(0u16.to_be_bytes());
    for value in [
        28u32,
        0,
        1,
        symbol as u32,
        symbol as u32,
        u32::from(glyph.0),
    ] {
        cmap.extend(value.to_be_bytes());
    }
    let checksum = |data: &[u8]| {
        data.chunks(4).fold(0u32, |sum, chunk| {
            let mut word = [0; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            sum.wrapping_add(u32::from_be_bytes(word))
        })
    };
    let offset = bytes.len().next_multiple_of(4);
    bytes.resize(offset, 0);
    bytes.extend(&cmap);
    bytes[cmap_record + 4..cmap_record + 8].copy_from_slice(&checksum(&cmap).to_be_bytes());
    bytes[cmap_record + 8..cmap_record + 12].copy_from_slice(&(offset as u32).to_be_bytes());
    bytes[cmap_record + 12..cmap_record + 16].copy_from_slice(&(cmap.len() as u32).to_be_bytes());
    bytes[head_offset + 8..head_offset + 12].fill(0);
    let adjustment = 0xB1B0AFBAu32.wrapping_sub(checksum(&bytes));
    bytes[head_offset + 8..head_offset + 12].copy_from_slice(&adjustment.to_be_bytes());
    assert_eq!(checksum(&bytes), 0xB1B0AFBA);
    let mapped = ttf_parser::Face::parse(&bytes, 0).unwrap();
    assert_eq!(mapped.glyph_index(symbol), Some(glyph));
    assert!(mapped.glyph_index('►').is_none());
    bytes
}

// 기존 테스트의 합성 글꼴 공급 전·후 및 해제 후에 호출한다.
fn assert_layer_display(core: &DocumentCore, symbol: char, expected: &str) {
    fn collect<'a>(
        value: &'a serde_json::Value,
        symbol: char,
        runs: &mut Vec<&'a serde_json::Value>,
    ) {
        match value {
            serde_json::Value::Object(object) => {
                if object.get("type").and_then(|v| v.as_str()) == Some("textRun")
                    && object
                        .get("text")
                        .and_then(|v| v.as_str())
                        .is_some_and(|text| text.contains(symbol))
                {
                    runs.push(value);
                }
                for child in object.values() {
                    collect(child, symbol, runs);
                }
            }
            serde_json::Value::Array(array) => {
                for child in array {
                    collect(child, symbol, runs);
                }
            }
            _ => {}
        }
    }
    let layer = core
        .build_page_layer_tree_with_profile(0, rhwp::paint::RenderProfile::Print)
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&layer.to_json()).unwrap();
    let mut runs = Vec::new();
    collect(&value, symbol, &mut runs);
    assert_eq!(runs.len(), 1, "원 기호의 레이어 소속을 보존한다");
    let run = runs[0];
    let rendered = run
        .get("displayText")
        .and_then(|v| v.as_str())
        .or_else(|| run.get("text").and_then(|v| v.as_str()))
        .unwrap();
    assert_eq!(rendered, expected, "CanvasKit이 소비하는 표시 문자열");
}

fn assert_prepared_source_symbol(symbol: char, fallback: &str) {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

    fn displayed_pointer(node: &RenderNode, symbol: char) -> Option<&str> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains(symbol) {
                return Some(run.display_or_text());
            }
        }
        node.children
            .iter()
            .find_map(|child| displayed_pointer(child, symbol))
    }

    fn source_advance(node: &RenderNode, symbol: char) -> Option<(f64, f64)> {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains(symbol) {
                assert_eq!(run.style.ratio, 1.0);
                assert_eq!(run.style.letter_spacing, 0.0);
                assert!(run.style.font_size > 0.0);
                assert!(!run.style.superscript && !run.style.subscript);
                return Some((node.bbox.width, run.style.font_size));
            }
        }
        node.children
            .iter()
            .find_map(|child| source_advance(child, symbol))
    }

    let mut core = DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp")).unwrap();
    core.insert_text_native(0, 0, 0, &symbol.to_string())
        .unwrap();
    let mut document = core.document().clone();
    for language in &mut document.doc_info.font_faces {
        for font in language {
            font.name = "RHWP Host Fixture".into();
        }
    }
    core.set_document(document);
    let original = format!("{:?}", core.document());
    let before = core.build_page_render_tree(0).unwrap();
    assert_eq!(displayed_pointer(&before.root, symbol), Some(fallback));
    assert_layer_display(&core, symbol, fallback);
    let program = synthetic_symbol_program(symbol);
    assert!(core
        .prepare_native_pua_fonts_with(|request| {
            assert!(request.family.starts_with("RHWP Host Fixture"));
            Some((program.clone(), 0))
        })
        .unwrap());
    let prepared = core.build_page_render_tree(0).unwrap();
    assert_eq!(
        displayed_pointer(&prepared.root, symbol),
        Some(symbol.to_string().as_str()),
        "실제 원 glyph를 선택한 뒤 이전 Unicode 표시를 남기지 않는다"
    );
    assert_layer_display(&core, symbol, &symbol.to_string());
    let (advance, size) = source_advance(&prepared.root, symbol).unwrap();
    assert!(
        (advance - size * 0.9).abs() <= 4.0 / 75.0,
        "합성 원본 SFNT의 독립 hmtx 900/1000 전진을 배치도 소비한다"
    );
    assert_eq!(format!("{:?}", core.document()), original);
    core.clear_native_pua_fonts();
    let cleared = core.build_page_render_tree(0).unwrap();
    assert_eq!(displayed_pointer(&cleared.root, symbol), Some(fallback));
    assert_layer_display(&core, symbol, fallback);
    assert_eq!(format!("{:?}", core.document()), original);
}

#[test]
fn prepared_source_glyph_replaces_an_earlier_unicode_display_and_clear_restores_it() {
    assert_prepared_source_symbol('\u{f02fc}', "►");
}

#[test]
fn small_source_triangle_keeps_its_program_and_clear_restores_unicode_fallback() {
    assert_prepared_source_symbol('\u{f02fb}', "▸");
}

#[test]
fn source_square_uses_supplied_hmtx_and_clear_restores_unicode_fallback() {
    assert_prepared_source_symbol('\u{f031c}', "■");
}

#[test]
fn additional_source_box_minus_preserves_its_program_and_fallback() {
    assert_prepared_source_symbol('\u{f03a7}', "⊟");
}

#[test]
fn additional_source_box_plus_preserves_its_program_and_fallback() {
    assert_prepared_source_symbol('\u{f03a8}', "⊞");
}

#[test]
fn additional_source_dash_preserves_its_program_and_fallback() {
    assert_prepared_source_symbol('\u{f0848}', "━");
}

use rhwp::renderer::native_pua_fonts::NativePuaFonts;

// 저장소에 포함된 테스트 글꼴만 공급해 시스템 글꼴 설치에 의존하지 않는다.
const REGULAR: &[u8] = include_bytes!("../../tests/fixtures/fonts/RHWPEquationLatinOnly.ttf");
const LIGHT: &[u8] = include_bytes!("../../tests/fixtures/fonts/RHWPEquationCJKLight.ttf");
const NO_DIGIT: &[u8] = include_bytes!("../../tests/fixtures/fonts/RHWPHftOutline.otf");

#[test]
fn original_truetype_provider_requires_program_family_ownership() {
    use rhwp::model::style::Font;

    let prepare = |family: &str, alt_type: u8| {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();
        let mut document = core.document().clone();
        for fonts in &mut document.doc_info.font_faces {
            fonts[0] = Font {
                name: family.into(),
                alt_type,
                ..Default::default()
            };
        }
        for shape in &mut document.doc_info.char_shapes {
            shape.font_ids = [0; 7];
        }
        core.set_document(document);
        core.prepare_native_otf_fonts_with(|_| Some((REGULAR.to_vec(), 0)))
            .unwrap()
    };

    // 공개 fixture의 name ID 1과 6은 같은 원본 프로그램을 식별한다.
    assert!(prepare("RHWP Equation Contract", 1));
    assert!(prepare("RHWPEquationContract-LatinOnly", 1));
    // ID 4는 표시용 full name이며 다른 family의 원본 전진을 인증하지 않는다.
    assert!(!prepare("RHWP Equation Contract LatinOnly", 1));
    assert!(!prepare("Missing Source", 1));
    assert!(!prepare("Missing Source,RHWP Equation Contract", 1));
    // HFT의 표시용 대체 프로그램은 원본 TrueType 전진 요청과 구분한다.
    assert!(prepare("Missing Source", 2));
}

#[test]
fn compact_truetype_retains_unicode_coverage_and_natural_advances() {
    let collection = include_bytes!("../fixtures/fonts/RHWPHostFixture.ttc");
    for (bytes, index) in [
        (REGULAR, 0),
        (LIGHT, 0),
        (collection.as_slice(), 0),
        (collection.as_slice(), 1),
        (collection.as_slice(), 2),
    ] {
        let mut fonts = NativePuaFonts::default();
        fonts
            .register_otf_for_source("fixture", false, false, true, bytes, index)
            .unwrap();
        let metrics = fonts
            .get_otf_for_source("fixture", false, false, true)
            .unwrap();
        let original = ttf_parser::Face::parse(bytes, index).unwrap();
        for code in 0..=0x10ffff {
            let Some(ch) = char::from_u32(code) else {
                continue;
            };
            if let Some(glyph) = original.glyph_index(ch).filter(|glyph| glyph.0 != 0) {
                assert!(metrics.has_glyph(ch), "문자 매핑 {index}: {ch:?}");
                let expected = original
                    .glyph_hor_advance(glyph)
                    .map(|v| f64::from(v) / f64::from(original.units_per_em()));
                assert_eq!(
                    metrics.source_ttf_advance_em(ch),
                    expected,
                    "자연 전진 {index}: {ch:?}"
                );
            }
        }
        assert!(!metrics.has_glyph('\u{10ffff}'));
    }
}

#[test]
fn truetype_metrics_do_not_exhaust_budget_with_unused_font_bytes() {
    let mut fonts = NativePuaFonts::default();
    for index in 0..7u8 {
        let mut padded = REGULAR.to_vec();
        padded.resize(10 * 1024 * 1024, index);
        let family = format!("fixture-{index}");
        fonts
            .register_otf_for_source(&family, false, false, true, &padded, 0)
            .expect("전진폭에 사용하지 않는 바이트는 보유 한도를 차지하지 않음");
        assert!(fonts
            .get_otf_for_source(&family, false, false, true)
            .and_then(|metrics| metrics.source_ttf_advance_em('A'))
            .is_some());
    }
}

#[test]
fn literal_space_programs_are_owned_by_source_type() {
    let mut fonts = NativePuaFonts::default();
    fonts
        .register_literal_space("Mixed", false, false, REGULAR, 0)
        .unwrap();
    fonts
        .register_literal_space_for_source("Mixed", false, false, false, LIGHT, 0)
        .unwrap();
    fonts
        .register_literal_space_for_source("Mixed", false, false, true, REGULAR, 0)
        .unwrap();
    assert_eq!(
        fonts.literal_space_em_for_source("Mixed", false, false, false),
        Some(0.528)
    );
    assert_eq!(
        fonts.literal_space_em_for_source("Mixed", false, false, true),
        Some(0.555)
    );
    assert!(fonts.has_literal_space_for_source("Mixed", false, false, true));

    fonts
        .register_literal_space_for_source("Mixed", false, false, true, NO_DIGIT, 0)
        .unwrap();
    assert!(
        !fonts.has_literal_space_for_source("Mixed", false, false, true),
        "공통 등록의 대체 가능 여부는 명시 유형 등록의 성공이 아니다"
    );
    assert_eq!(
        fonts.literal_space_em_for_source("Mixed", false, false, true),
        Some(0.555)
    );
    assert_eq!(
        fonts.literal_space_em_for_source("Mixed", false, false, false),
        Some(0.528)
    );
}

#[test]
fn unassigned_hwp5_font_bits_allow_source_probe_without_truetype_declaration() {
    use rhwp::model::provenance::SourceFormat;
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "A가").unwrap();
    let mut doc = core.document().clone();
    doc.doc_info.font_faces = (0..7)
        .map(|_| {
            vec![rhwp::model::style::Font {
                name: "RHWP unlisted source fixture".into(),
                alt_type: 1,
                ..Default::default()
            }]
        })
        .collect();
    for shape in &mut doc.doc_info.char_shapes {
        shape.font_ids = [0; 7];
    }
    doc.provenance.format = SourceFormat::Hwp5;
    doc.provenance.hwpx_lineage = false;
    core.set_document(doc.clone());
    let original = format!("{:?}", core.document());
    let requests = core.native_otf_font_requests();
    assert!(
        !requests.is_empty(),
        "미확인 원본 프로그램은 조회할 수 있다"
    );
    assert!(
        requests.iter().all(|request| {
            request.source_ttf && request.family == "RHWP unlisted source fixture"
        }),
        "원본과 같은 이름의 실제 TrueType만 요청한다"
    );

    // HWP5 사양의 미분류 비트는 조회 자격과 달리 TTF 선언의 근거가 아니다.
    // 실제 프로그램이 없는 동안 paint의 원본 유형과 문서 IR을 그대로 보존한다.
    fn collect_source_types(
        node: &rhwp::renderer::render_tree::RenderNode,
        text: &mut String,
        declarations: &mut Vec<bool>,
    ) {
        if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.chars().any(|ch| "A가".contains(ch)) {
                text.push_str(&run.text);
                declarations.push(run.style.source_font_is_ttf);
            }
        }
        for child in &node.children {
            collect_source_types(child, text, declarations);
        }
    }
    let tree = core.build_page_render_tree(0).unwrap();
    let mut text = String::new();
    let mut declarations = Vec::new();
    collect_source_types(&tree.root, &mut text, &mut declarations);
    assert!(
        text.contains('A') && text.contains('가'),
        "실제 본문 run을 확인한다"
    );
    assert!(!declarations.is_empty() && declarations.iter().all(|declared| !declared));
    assert_eq!(format!("{:?}", core.document()), original);
    doc.provenance.hwpx_lineage = true;
    core.set_document(doc.clone());
    assert!(core
        .native_otf_font_requests()
        .iter()
        .any(|request| request.source_ttf));
    doc.provenance.hwpx_lineage = false;
    doc.provenance.format = SourceFormat::Hwpx;
    core.set_document(doc.clone());
    assert!(core
        .native_otf_font_requests()
        .iter()
        .any(|request| request.source_ttf));
    for fonts in &mut doc.doc_info.font_faces {
        fonts[0].alt_type = 2;
    }
    core.set_document(doc);
    let requests = core.native_otf_font_requests();
    assert!(requests.iter().any(|request| !request.source_ttf));
    assert!(!requests.iter().any(|request| request.source_ttf));
}

#[test]
fn supplied_truetype_advance_is_distinct_from_hft_and_cff() {
    use rhwp::renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        TextStyle,
    };
    let ttf = include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf");
    let cff = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
    let mut programs = NativePuaFonts::default();
    programs
        .register_otf_for_source("unlisted-source", false, false, true, ttf, 0)
        .unwrap();
    programs
        .register_otf_for_source("unlisted-source", false, false, false, cff, 0)
        .unwrap();
    let mut style = TextStyle {
        font_family: "unlisted-source".into(),
        font_size: 24.0,
        source_font_is_ttf: true,
        native_otf_metrics: programs.get_otf_for_source("unlisted-source", false, false, true),
        ..Default::default()
    };
    let face = ttf_parser::Face::parse(ttf, 0).unwrap();
    for ch in "V0I".chars() {
        let expected = f64::from(
            face.glyph_hor_advance(face.glyph_index(ch).unwrap())
                .unwrap(),
        ) / f64::from(face.units_per_em())
            * style.font_size;
        let positions = EmbeddedTextMeasurer.compute_char_positions(&ch.to_string(), &style);
        assert!(
            (positions[1] - positions[0] - expected).abs() < 0.1,
            "{ch}: {:?} != {expected}",
            positions
        );
    }
    programs
        .register_otf_for_source("cff-only", false, false, true, cff, 0)
        .unwrap();
    style.font_family = "cff-only".into();
    style.native_otf_metrics = None;
    let baseline = EmbeddedTextMeasurer.compute_char_positions("0", &style);
    style.native_otf_metrics = programs.get_otf_for_source("cff-only", false, false, true);
    assert_eq!(
        EmbeddedTextMeasurer.compute_char_positions("0", &style),
        baseline
    );
}

#[test]
fn hft_bold_tracking_requires_verified_source_and_program() {
    use rhwp::renderer::{
        layout::{EmbeddedTextMeasurer, TextMeasurer},
        TextStyle,
    };
    let original = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
    let mut renamed = original.to_vec();
    let utf16 = |s: &str| {
        s.encode_utf16()
            .flat_map(u16::to_be_bytes)
            .collect::<Vec<_>>()
    };
    let from = utf16("RHWP HFT Outline Fixture");
    let to = utf16("Sinmyeong Junggothic    ");
    assert_eq!(from.len(), to.len());
    let mut count = 0;
    for offset in 0..=renamed.len() - from.len() {
        if renamed[offset..offset + from.len()] == from {
            renamed[offset..offset + to.len()].copy_from_slice(&to);
            count += 1;
        }
    }
    assert!(count > 0);
    let mut programs = NativePuaFonts::default();
    programs
        .register_otf("verified", true, false, &renamed, 0)
        .unwrap();
    programs
        .register_otf("other", true, false, original, 0)
        .unwrap();
    let mut style = TextStyle {
        font_size: 20.0,
        bold: true,
        source_font_is_hft: true,
        native_otf_metrics: programs.get_otf("other", true, false),
        ..Default::default()
    };
    let width = |style: &TextStyle| {
        let p = EmbeddedTextMeasurer.compute_char_positions("가", style);
        p[1] - p[0]
    };
    let regular = width(&style);
    style.source_hwp3_hft_bold = true;
    assert!(width(&style) > regular);
    style.source_hwp3_hft_bold = false;
    style.native_otf_metrics = programs.get_otf("verified", true, false);
    assert!(style
        .native_otf_metrics
        .as_ref()
        .unwrap()
        .has_verified_hft_bold_tracking('가'));
    let without_source = width(&style);
    style.source_verified_hft_bold_tracking = true;
    assert!((width(&style) - without_source - 1.0).abs() < 0.1);
    style.native_otf_metrics = programs.get_otf("other", true, false);
    assert!((width(&style) - regular).abs() < 1e-8);
}

// WMF paint도 같은 명시 공급 snapshot의 소유·중복 등록·저장 보존 계약을 따른다.
#[cfg(feature = "native-skia")]
mod metafile_font_snapshot {

    use rhwp::document_core::queries::png_export::NativePngExporter;
    use rhwp::document_core::queries::rendering::PngExportOptions;
    use rhwp::{
        model::{
            bin_data::BinDataContent,
            control::Control,
            image::Picture,
            paragraph::{CharShapeRef, LineSeg, Paragraph},
            shape::CommonObjAttr,
        },
        paint::RenderProfile,
        DocumentCore,
    };
    const SOURCE_FAMILY: &str = "RHWP Host Fixture";
    const FONT: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fonts/RHWPHostFixture-Regular.ttf"
    ));

    // 이름만 고유하게 바꿔 시스템·다른 병렬 검사의 동일 family 설치와 격리한다.
    // glyph·cmap·hmtx는 저장소 소유 합성 프로그램 그대로이며 정상 SFNT checksum을 유지한다.
    fn isolated_program(family: &str) -> Vec<u8> {
        assert_eq!(family.len(), SOURCE_FAMILY.len());
        let mut bytes = FONT.to_vec();
        for (from, to) in [
            (
                SOURCE_FAMILY.as_bytes().to_vec(),
                family.as_bytes().to_vec(),
            ),
            (
                SOURCE_FAMILY
                    .encode_utf16()
                    .flat_map(u16::to_be_bytes)
                    .collect(),
                family.encode_utf16().flat_map(u16::to_be_bytes).collect(),
            ),
        ] {
            let positions: Vec<_> = bytes
                .windows(from.len())
                .enumerate()
                .filter_map(|(index, data)| (data == from).then_some(index))
                .collect();
            for index in positions {
                bytes[index..index + to.len()].copy_from_slice(&to);
            }
        }
        fn checksum(data: &[u8]) -> u32 {
            data.chunks(4).fold(0u32, |sum, part| {
                let mut word = [0u8; 4];
                word[..part.len()].copy_from_slice(part);
                sum.wrapping_add(u32::from_be_bytes(word))
            })
        }
        let tables = u16::from_be_bytes(bytes[4..6].try_into().unwrap()) as usize;
        let mut head = None;
        for index in 0..tables {
            let record = 12 + index * 16;
            let start =
                u32::from_be_bytes(bytes[record + 8..record + 12].try_into().unwrap()) as usize;
            let length =
                u32::from_be_bytes(bytes[record + 12..record + 16].try_into().unwrap()) as usize;
            if &bytes[record..record + 4] == b"head" {
                head = Some(start);
                bytes[start + 8..start + 12].fill(0);
            }
            if matches!(&bytes[record..record + 4], b"head" | b"name") {
                let value = checksum(&bytes[start..start + length]);
                bytes[record + 4..record + 8].copy_from_slice(&value.to_be_bytes());
            }
        }
        let head = head.unwrap();
        let adjustment = 0xB1B0_AFBAu32.wrapping_sub(checksum(&bytes));
        bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
        bytes
    }

    fn record(function: u16, parameters: &[u8]) -> Vec<u8> {
        assert!(parameters.len().is_multiple_of(2));
        [
            ((3 + parameters.len() / 2) as u32).to_le_bytes().as_slice(),
            function.to_le_bytes().as_slice(),
            parameters,
        ]
        .concat()
    }

    fn document(family: &str) -> DocumentCore {
        let mut wmf = [
            1u16.to_le_bytes().as_slice(),
            9u16.to_le_bytes().as_slice(),
            0x300u16.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u16.to_le_bytes().as_slice(),
        ]
        .concat();
        wmf.extend(record(
            0x020c,
            &[100i16.to_le_bytes(), 100i16.to_le_bytes()].concat(),
        ));
        let mut logfont = [
            (-30i16).to_le_bytes(),
            0i16.to_le_bytes(),
            0i16.to_le_bytes(),
            0i16.to_le_bytes(),
            400i16.to_le_bytes(),
        ]
        .concat();
        logfont.extend([0u8; 8]);
        let mut name = [0u8; 32];
        name[..family.len()].copy_from_slice(family.as_bytes());
        logfont.extend(name);
        wmf.extend(record(0x02fb, &logfont));
        wmf.extend(record(0x012d, &0u16.to_le_bytes()));
        wmf.extend(record(0x012e, &24u16.to_le_bytes()));
        wmf.extend(record(
            0x0521,
            &[
                1u16.to_le_bytes().as_slice(),
                b"A\0",
                60i16.to_le_bytes().as_slice(),
                10i16.to_le_bytes().as_slice(),
            ]
            .concat(),
        ));
        wmf.extend(record(0, &[]));
        let words = wmf.len() as u32 / 2;
        wmf[6..10].copy_from_slice(&words.to_le_bytes());
        let mut core = DocumentCore::from_bytes(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/saved/blank2010.hwp"
        )))
        .unwrap();
        core.set_hangul2024_compat(false);
        let mut doc = core.document().clone();
        let mut picture = Picture {
            common: CommonObjAttr {
                treat_as_char: true,
                width: 7500,
                height: 7500,
                ..Default::default()
            },
            ..Default::default()
        };
        picture.image_attr.bin_data_id = 1;
        doc.sections[0].paragraphs = vec![Paragraph {
            char_count: 1,
            char_shapes: vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }],
            line_segs: vec![LineSeg {
                line_height: 7500,
                text_height: 7500,
                baseline_distance: 6000,
                ..Default::default()
            }],
            controls: vec![Control::Picture(Box::new(picture))],
            ..Default::default()
        }];
        doc.bin_data_content = vec![BinDataContent {
            id: 1,
            extension: "wmf".into(),
            data: wmf.into(),
        }];
        core.set_document(doc);
        core
    }

    #[test]
    fn png_exporter_supplies_wmf_only_font_from_its_snapshot_and_preserves_source() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        struct Directory(std::path::PathBuf);
        impl Drop for Directory {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let family = format!(
            "RHWPWMF{:010}",
            u64::from(std::process::id()) * 1000 + serial
        );
        let font = isolated_program(&family);
        let dir = Directory(std::env::temp_dir().join(format!(
            "rhwp-wmf-snapshot-{}-{}",
            std::process::id(),
            serial
        )));
        std::fs::create_dir_all(&dir.0).unwrap();
        std::fs::write(dir.0.join("fixture.ttf"), &font).unwrap();
        let exporter = NativePngExporter::new(
            RenderProfile::Print,
            PngExportOptions {
                font_paths: vec![dir.0.clone()],
                ..Default::default()
            },
        );
        let mut automatic = document(&family);
        let mut explicit = document(&family);
        let source = rhwp::serializer::serialize_document(automatic.document()).unwrap();
        let requests: serde_json::Value =
            serde_json::from_str(&automatic.get_metafile_font_requests_native().unwrap()).unwrap();
        assert!(
            requests.as_array().unwrap().iter().any(|x| x["families"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name.as_str() == Some(family.as_str()))),
            "WMF에만 존재하는 가족을 실제로 요청한다"
        );
        explicit
            .register_metafile_font_native(&family, &font, 0)
            .unwrap();
        let expected = exporter.render_page(&explicit, 0).unwrap();
        let missing = exporter.render_page(&automatic, 0).unwrap();
        assert_ne!(
            missing, expected,
            "원 프로그램 부재 대조군은 독립 합성 glyph와 다르다"
        );
        assert!(exporter.prepare_document(&mut automatic).unwrap());
        let supplied = exporter.render_page(&automatic, 0).unwrap();
        assert_eq!(
            supplied, expected,
            "exporter의 공급이 명시적으로 등록한 같은 원 프로그램을 소비한다"
        );
        assert!(
            !exporter.prepare_document(&mut automatic).unwrap(),
            "같은 snapshot의 중복 등록은 paint를 다시 바꾸지 않는다"
        );
        assert_eq!(
            rhwp::serializer::serialize_document(automatic.document()).unwrap(),
            source,
            "paint 공급이 저장 IR과 원래 WMF 바이트를 바꾸지 않는다"
        );
    }
}
