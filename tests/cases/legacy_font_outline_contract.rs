//! 검사 유형: 관계형. 합성 글꼴 프로그램의 선택·대체 계약을 검증한다.
//! 주석 갱신: 2026-11-15T11:40:50+09:00; 직전 파일 커밋: 2026-11-14T05:31:54+09:00 (9e2794796787b46815462fcecd1536ab74ac8528).
//! 추가 검사는 MIT 합성 프로그램 또는 내부 글자모양 소유 관계이며 설치 글꼴·절대 배치를 가정하지 않는다.
//! 주석 갱신: 2026-11-12T01:00:43+09:00; 직전 변경: 2026-11-09T21:28:38+09:00, 89ef796bf5e72418d7ad9a120bf1be5435d934a3.
//! 실제 입력 `samples/hwp3-sample10.hwp`, 한컴 2020 Print, 96 DPI, 2px 이웃 raw.
//! 2026-10-30 측정: 1쪽 Native/fresh WASM 각 99.96%; 전 763쪽 최저 각 99.41% / 99.40%.
//! 근거: `mydocs/pr/assets/issue_8371/sample10-native.tsv`, `sample10-wasm.tsv`, `sample10-validation.md`.
//! #8946 HWPX: 2026-11-12, 같은 외부 그림의 한컴 2024 Print, 96 DPI, 2px raw.
//! 전 763쪽 Native/fresh WASM 최저 96.34886%/99.56218%, 표지 각 100.00%.
//! 문장부호 350·608·686쪽 각 100.00%; 근거: mydocs/pr/assets/issue8946/stage4-20261112/
//! native-full.tsv·wasm-full.tsv, stage3-20261111/punctuation-before-after-native-300dpi.png.
//! 합성 fixture의 계약 검사와 실제 문서의 한컴 출력 판독은 별도다.
//! Linux Fontconfig의 합성 Bold 재현은 유지하며, Mac·Windows도 명시 공급한 분할 CFF의
//! 글리프 소유를 실제 Native backend에서 검사한다. 이 합성 계약의 raw·독립 PDF는 비해당이다.
//! 주석 갱신: 2026-11-25T20:30:09+09:00. 명시 선택한 원본의 구두점 소유는 관계형 검사로 확인한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::style::Font;

const FAMILY: &str = "RHWP HFT Outline Fixture";
const CFF: &[u8] = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
const CFF_JAMO: &[u8] = include_bytes!("../fixtures/fonts/RHWPHftOutlineJamo.otf");
const SOURCE_SANS_FIRST: &[u8] =
    include_bytes!("../fixtures/fonts/source-sans-3/SourceSans3-3.028-Regular.otf");
const SOURCE_SANS_SECOND: &[u8] =
    include_bytes!("../fixtures/fonts/source-sans-3/SourceSans3-3.052-Regular.otf");

#[test]
fn source_legacy_outline_follows_the_selected_environment_family_only() {
    // 관계형: 저장된 HFT 이름과 실제 공급 family가 달라도 같은 선택을 소비한다.
    // 비 HFT 원본은 실제 프로그램 신원 없이 허용하지 않으며 미선택 family도 거부한다.
    use rhwp::renderer::font_environment::FontEnvironment;
    let mut core = DocumentCore::new_empty();
    let mut document = core.document().clone();
    document.doc_info.font_faces = vec![vec![
        Font {
            name: "원본 HFT".into(),
            alt_type: 2,
            ..Default::default()
        },
        Font {
            name: "원본 TTF".into(),
            ..Default::default()
        },
    ]];
    core.set_document(document);
    let before = rhwp::serializer::serialize_document(core.document()).unwrap();
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"outline-selection","substitutions":{"원본 HFT":"Selected CFF","원본 TTF":"Other CFF"}}"#,
    ).unwrap())).unwrap();
    assert!(!core
        .register_hft_outline_font_native("Other CFF", CFF, 0)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_native("Unselected CFF", CFF, 0)
        .unwrap());
    assert!(
        core.register_hft_outline_font_native("Selected CFF", CFF, 0)
            .unwrap(),
        "선택한 표시용 CFF가 인쇄 윤곽을 공급해야 한다"
    );
    assert_eq!(
        rhwp::serializer::serialize_document(core.document()).unwrap(),
        before,
        "선택된 paint 프로그램은 저장 IR을 바꾸지 않는다"
    );
}

#[test]
fn verified_cff_query_alias_accepts_only_the_selected_program() {
    // 관계형: 원본 유형이 미정이어도 실제 선택된 CFF 신원·조회 이름을 함께 확인한다.
    // 원본을 HFT로 재분류하거나 확인하지 않은 별칭·프로그램을 허용하지 않는다.
    use rhwp::renderer::font_environment::FontEnvironment;
    let mut core = DocumentCore::new_empty();
    let mut document = core.document().clone();
    document.doc_info.font_faces = vec![vec![Font {
        name: "원본 미정 글꼴".into(),
        alt_type: 0,
        ..Default::default()
    }]];
    core.set_document(document);
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"verified-cff-query","substitutions":{"원본 미정 글꼴":"RHWP HFT Outline Fixture"},"selectedPrograms":{"원본 미정 글꼴":{"query":"실제 조회 별칭","family":"RHWP HFT Outline Fixture","trueType":false,"openTypeCff":true,"programId":"sweep-c3eb4ea18997f2a79442f80e51e421bfe496edb431a9247f27dafd4fcc305dda"}}}"#,
    ).unwrap())).unwrap();
    assert!(!core
        .register_hft_outline_font_native("임의 별칭", CFF, 0)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_native("실제 조회 별칭", CFF_JAMO, 0)
        .unwrap());
    assert!(
        core.register_hft_outline_font_native("실제 조회 별칭", CFF, 0)
            .unwrap(),
        "선택 프로그램의 실제 조회 별칭도 같은 윤곽을 공급한다"
    );
    assert!(core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
}

#[test]
fn legacy_outline_registration_preserves_unicode_family_case_folding() {
    // 관계형: 같은 family의 유니코드 대소문자 정규화는 기존 등록 계약을 보존한다.
    use rhwp::renderer::font_environment::FontEnvironment;
    let mut core = DocumentCore::new_empty();
    let mut document = core.document().clone();
    document.doc_info.font_faces = vec![vec![Font {
        name: "원본 HFT".into(),
        alt_type: 2,
        ..Default::default()
    }]];
    core.set_document(document);
    core.set_font_environment(Some(
        FontEnvironment::from_json(
            r#"{"id":"unicode-hft-family","substitutions":{"원본 HFT":"Σ Display"}}"#,
        )
        .unwrap(),
    ))
    .unwrap();
    assert!(core
        .register_hft_outline_font_native("σ display", CFF, 0)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_native("Σ Display", CFF, 0)
        .unwrap());
}

#[test]
fn split_legacy_outline_candidates_replace_atomically_and_retain_order() {
    // 관계형: 저장소의 두 CFF 프로그램을 공급 순서대로 교체한다.
    let mut core = DocumentCore::new_empty();
    let mut document = core.document().clone();
    document.doc_info.font_faces = vec![vec![Font {
        name: FAMILY.into(),
        alt_type: 2,
        ..Default::default()
    }]];
    core.set_document(document);
    let first_order = [(CFF, 0), (CFF_JAMO, 0)];
    assert!(core
        .register_hft_outline_font_candidates_native(FAMILY, &first_order)
        .unwrap());
    assert!(!core
        .register_hft_outline_font_candidates_native(FAMILY, &first_order)
        .unwrap());
    assert!(core
        .register_hft_outline_font_candidates_native(FAMILY, &[])
        .is_err());
    assert!(core
        .register_hft_outline_font_candidates_native(FAMILY, &[(CFF, 0); 65])
        .is_err());
    assert!(core
        .register_hft_outline_font_candidates_native(FAMILY, &[(CFF, 0), (&[], 0)])
        .is_err());
    assert!(
        !core
            .register_hft_outline_font_candidates_native(FAMILY, &first_order)
            .unwrap(),
        "한도 오류로 기존 후보를 일부만 교체하지 않는다"
    );
    assert!(core
        .register_hft_outline_font_candidates_native(FAMILY, &[(CFF_JAMO, 0), (CFF, 0)])
        .unwrap());
    let ttf = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    assert!(
        core.register_hft_outline_font_candidates_native(FAMILY, &[(CFF, 0), (ttf, 0)])
            .unwrap(),
        "혼합 형식은 기존 Canvas 소유 판단으로 되돌린다"
    );
    assert!(core
        .register_hft_outline_font_native(FAMILY, CFF, 0)
        .unwrap());
}

#[test]
fn romanized_myeongjo_names_keep_serif_fallback() {
    use rhwp::renderer::generic_fallback;
    for name in ["HY Sinmyeongjo", "Sinmyeong Jungmyeongjo", "AppleMyungjo"] {
        assert!(generic_fallback(name).ends_with(",serif"), "{name}");
    }
    assert!(generic_fallback("Sinmyeong Junggothic").ends_with(",sans-serif"));
}

#[test]
fn verified_legacy_advance_survives_a_supplied_display_otf() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;
    use rhwp::renderer::TextStyle;

    let mut fonts = NativePuaFonts::default();
    fonts
        .register_otf("Source Sans 3", false, false, SOURCE_SANS_FIRST, 0)
        .unwrap();
    let mut style = TextStyle {
        font_family: "HCI Poppy".into(),
        font_size: 20.0,
        source_font_is_hft: true,
        ..Default::default()
    };
    let measure = |style: &TextStyle| EmbeddedTextMeasurer.compute_char_positions("A", style)[1];
    let source_width = measure(&style);
    style.native_otf_metrics = fonts.get_otf("Source Sans 3", false, false);
    assert_eq!(
        measure(&style),
        source_width,
        "원본 글꼴의 전진폭을 보존한다"
    );
    style.source_font_is_hft = false;
    assert_ne!(
        measure(&style),
        source_width,
        "명시 OTF는 해당 프로그램의 전진을 사용한다"
    );
}

#[test]
fn declared_legacy_hyphenation_point_keeps_its_half_em_before_display_fallback() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;
    use rhwp::renderer::TextStyle;

    // 관계형·원본 전진 계약: worklife Print의 Type3 Widths=500/1000과
    // nested 17쪽의 구분점→뒤 글자 원점을 대조했다. 실물 좌표를 고정하지 않는다.
    // MIT 합성 CFF는 표시 글꼴만 공급하며 비공개 HFT 설치를 요구하지 않는다.
    let mut fonts = NativePuaFonts::default();
    fonts
        .register_otf("synthetic", false, false, CFF, 0)
        .unwrap();
    let mut style = TextStyle {
        font_family: "HY Sinmyeongjo".into(),
        original_glyph_metric_family: Some(std::sync::Arc::new("한양신명조".into())),
        source_font_is_hft: true,
        font_size: 20.0,
        native_otf_metrics: fonts.get_otf("synthetic", false, false),
        ..Default::default()
    };
    let measure = |style: &TextStyle| {
        EmbeddedTextMeasurer.compute_char_positions("‧", style)[1] / style.font_size
    };
    // 실제 전진은 독립 Print로 확인한 4HU 장치 격자에서 반올림된다.
    // 자연 전진 0.5em을 검사하되 그 반 격자인 2HU를 계산에서 구분한다.
    let half_device_unit = |style: &TextStyle| 2.0 / 75.0 / style.font_size;
    assert!(
        (measure(&style) - 0.5).abs() <= half_device_unit(&style) + f64::EPSILON,
        "표시용 결측 폭으로 원본 전진을 바꾸지 않는다: {}",
        measure(&style)
    );
    style.font_size *= 2.0;
    assert!(
        (measure(&style) - 0.5).abs() <= half_device_unit(&style) + f64::EPSILON,
        "원본 반각 계약은 장치 반올림 범위에서 크기에 비례한다"
    );
    style.source_font_is_hft = false;
    style.original_glyph_metric_family = None;
    assert_ne!(
        measure(&style),
        0.5,
        "원본 글꼴과 연결되지 않은 명시 OTF에는 이 폭을 적용하지 않는다"
    );
}

#[test]
fn legacy_mapped_symbol_retains_source_advance_with_display_otf() {
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;
    use rhwp::renderer::supplemental_metrics::{
        MetricBackend, MetricContext, SupplementalMetric, SupplementalMetricStore,
    };
    use rhwp::renderer::TextStyle;

    const FAMILY: &str = "Source Code Pro";
    const PROGRAM: &[u8] =
        include_bytes!("../fixtures/fonts/source-code-pro/SourceCodePro-Regular.otf");
    let face = ttf_parser::Face::parse(PROGRAM, 0).unwrap();
    let glyph = face.glyph_index('═').unwrap();
    assert_eq!(face.units_per_em(), 1000);
    assert_eq!(face.glyph_hor_advance(glyph), Some(600));

    let mut fonts = NativePuaFonts::default();
    fonts
        .register_otf(FAMILY, false, false, PROGRAM, 0)
        .unwrap();
    let mut style = TextStyle {
        font_family: FAMILY.into(),
        font_size: 20.0,
        source_font_is_hft: true,
        ..Default::default()
    };
    let measure =
        |text: &str, style: &TextStyle| EmbeddedTextMeasurer.compute_char_positions(text, style)[1];
    let source_width = measure("\u{F0827}", &style);
    style.native_otf_metrics = fonts.get_otf(FAMILY, false, false);
    assert_eq!(
        measure("\u{F0827}", &style),
        source_width,
        "원본 기호 전진을 보존한다"
    );
    assert_eq!(measure("═", &style), 12.0, "공급 원본의 600/1000em 전진");
    let context = MetricContext {
        document_generation: 1,
        font_generation: 1,
        backend: MetricBackend::Canvas2d,
    };
    let mut store = SupplementalMetricStore::new(context);
    style.native_otf_metrics = None;
    let canvas = SupplementalMetric::backend_measured(&style, "═", FAMILY.into(), 18.0).unwrap();
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

    #[cfg(feature = "native-skia")]
    {
        use rhwp::renderer::font_environment::FontEnvironment;
        use rhwp::renderer::skia::SkiaLayerRenderer;
        use sha2::{Digest, Sha256};

        let latin_family = "RHWP Equation Contract";
        let mut supplied_document = core.document().clone();
        for (slot, faces) in supplied_document.doc_info.font_faces.iter_mut().enumerate() {
            if let Some(face) = faces.first_mut() {
                face.name = if slot == 1 { latin_family } else { FAMILY }.into();
            }
        }
        core.set_document(supplied_document);
        let digest: String = Sha256::digest(korean_missing)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let renderer = SkiaLayerRenderer::new()
            .try_with_font_data(&[korean_missing.to_vec(), latin.to_vec()])
            .unwrap();
        let mut environment = serde_json::json!({
            "id": "selected-punctuation-owner",
            "preserveSourceMetrics": true,
            "substitutions": {(FAMILY): FAMILY, (latin_family): latin_family},
            "glyphFallbacks": {(FAMILY): [latin_family]},
        });
        for expected in [FAMILY, latin_family] {
            if expected == latin_family {
                environment["selectedPrograms"] = serde_json::json!({
                    (FAMILY): {
                        "query": FAMILY,
                        "family": FAMILY,
                        "programId": format!("sweep-{digest}"),
                        "trueType": false,
                        "openTypeCff": true,
                    },
                });
            }
            core.set_font_environment(Some(
                FontEnvironment::from_json(&environment.to_string()).unwrap(),
            ))
            .unwrap();
            renderer.prepare_native_otf_fonts(&mut core).unwrap();
            let tree = core.build_page_render_tree(0).unwrap();
            let mut punctuation = Vec::new();
            families(&tree.root, '(', &mut punctuation);
            assert_eq!(punctuation.len(), 1);
            assert_eq!(
                punctuation[0].split(',').next(),
                Some(expected),
                "다른 공급 글꼴의 구두점이 명시 선택한 프로그램의 결측을 가리지 않는다"
            );
        }
    }
}

#[test]
fn hwpx_supplied_cff_punctuation_keeps_glyph_owner_and_content_order() {
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
    let input = include_bytes!("../../mydocs/pr/assets/issue7774/sql-sample10-regressions-20261015/hwp3-sample10-hwpx.hwpx");
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
    // 원본에서 파생한 합성 경계: 글꼴은 MIT fixture만 공급한다.
    // 저장 줄 경로와 저장 줄을 제거한 재조판 경로에 같은 소유 계약을 적용한다.
    let latin_cff = SOURCE_SANS_SECOND;
    let latin_ttf = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf");
    let korean_ttf = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
    for reflow in [false, true] {
        for (korean, latin, source_hft, expected) in [
            (CFF, Some(latin_cff), true, "fixture-latin"),
            (CFF, None, true, "fixture-korean"),
            (latin_cff, Some(latin_cff), true, "fixture-korean"),
            (CFF, Some(latin_ttf.as_slice()), true, "fixture-korean"),
            (
                korean_ttf.as_slice(),
                Some(latin_cff),
                true,
                "fixture-korean",
            ),
            (CFF, Some(latin_cff), false, "fixture-korean"),
        ] {
            let mut input = document.clone();
            if reflow {
                input.sections[0].paragraphs[0].line_segs.clear();
            }
            if !source_hft {
                for faces in &mut input.doc_info.font_faces {
                    for face in faces {
                        face.alt_type = 0;
                    }
                }
            }
            core.set_document(input);
            core.prepare_native_otf_fonts_with(|request| match request.family.as_str() {
                "fixture-korean" => Some((korean.to_vec(), 0)),
                "fixture-latin" => latin.map(|bytes| (bytes.to_vec(), 0)),
                _ => None,
            })
            .unwrap();
            let tree = core.build_page_render_tree(0).unwrap();
            let mut punctuation = Vec::new();
            families(&tree.root, '(', &mut punctuation);
            assert_eq!(punctuation.len(), 1, "구두점은 한 번 표시된다");
            assert!(
                punctuation[0].contains(expected),
                "reflow={reflow}: {punctuation:?}, expected={expected}"
            );
            fn ordered_runs(node: &RenderNode, output: &mut Vec<(String, f64, f64)>) {
                if let RenderNodeType::TextRun(run) = &node.node_type {
                    output.push((run.text.clone(), node.bbox.x, node.bbox.x + node.bbox.width));
                }
                for child in &node.children {
                    ordered_runs(child, output);
                }
            }
            let mut runs = Vec::new();
            ordered_runs(&tree.root, &mut runs);
            assert_eq!(
                runs.iter().map(|run| run.0.as_str()).collect::<String>(),
                "가(가"
            );
            for pair in runs.windows(2) {
                assert!(
                    pair[0].2 <= pair[1].1 + 0.02,
                    "같은 줄의 앞뒤 run이 겹치지 않는다: {pair:?}"
                );
            }
            let mut hangul = Vec::new();
            families(&tree.root, '가', &mut hangul);
            assert!(!hangul.is_empty());
            assert!(hangul
                .iter()
                .all(|family| family.contains("fixture-korean")));
        }
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
        std::fs::write(dir.join("fonts/first.otf"), SOURCE_SANS_FIRST).unwrap();
        std::fs::write(dir.join("fonts/second.otf"), SOURCE_SANS_SECOND).unwrap();
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
                "legacy_font_outline_contract::skia_synthetic_bold_selects_both_split_family_glyphs",
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
    assert_split_family_glyphs(&SkiaLayerRenderer::new(), "system");
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
fn assert_split_family_glyphs(renderer: &rhwp::renderer::skia::SkiaLayerRenderer, source: &str) {
    const FAMILY: &str = "Source Sans 3";
    let first = ttf_parser::Face::parse(SOURCE_SANS_FIRST, 0).unwrap();
    let second = ttf_parser::Face::parse(SOURCE_SANS_SECOND, 0).unwrap();
    assert!(first.glyph_index('A').is_some());
    assert!(first.glyph_index('Ɓ').is_none());
    assert!(second.glyph_index('Ɓ').is_some());
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "AƁ").unwrap();
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
    let trace: serde_json::Value = serde_json::from_str(
        &renderer
            .get_font_decision_trace(&core, 0, r#"{"maxCharacters":16}"#)
            .unwrap(),
    )
    .unwrap();
    for character in ["A", "Ɓ"] {
        let record = trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["source"]["character"] == character)
            .expect("입력 글자마다 글꼴 선택 기록");
        assert_eq!(record["paint"]["native"]["source"], source);
        assert_eq!(
            record["paint"]["native"]["resolved"], FAMILY,
            "각 글리프를 같은 분할 family에서 선택한다"
        );
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[test]
fn supplied_synthetic_bold_keeps_both_split_glyphs_on_each_native_os() {
    let renderer = rhwp::renderer::skia::SkiaLayerRenderer::new()
        .try_with_font_data(&[SOURCE_SANS_FIRST.to_vec(), SOURCE_SANS_SECOND.to_vec()])
        .unwrap();
    assert_split_family_glyphs(&renderer, "custom");
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
    fn print_legacy_contour_preserves_quarter_pixel_baseline() {
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
    fn screen_and_non_legacy_keep_existing_font_raster() {
        for y in [4.1, 4.35] {
            assert_eq!(
                paint(y, true, RenderProfile::Screen),
                paint(y, false, RenderProfile::Screen),
                "화면 출력에 인쇄용 윤곽을 적용하지 않는다"
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
        // 같은 family의 두 배포 버전 중 3.052에만 Ɓ가 있다.
        let first = ttf_parser::Face::parse(super::SOURCE_SANS_FIRST, 0).unwrap();
        let second = ttf_parser::Face::parse(super::SOURCE_SANS_SECOND, 0).unwrap();
        assert!(first.glyph_index('Ɓ').is_none());
        assert!(second.glyph_index('Ɓ').is_some());
        let renderer = SkiaLayerRenderer::new()
            .try_with_font_data(&[
                super::SOURCE_SANS_FIRST.to_vec(),
                super::SOURCE_SANS_SECOND.to_vec(),
            ])
            .unwrap();
        let paint_at = |y| {
            let run = TextRunNode {
                text: "Ɓ".into(),
                style: TextStyle {
                    font_family: "Source Sans 3".into(),
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

    #[test]
    fn split_family_keeps_selected_legacy_outline_origin_in_2022_print() {
        let first = ttf_parser::Face::parse(SOURCE_SANS_FIRST, 0).unwrap();
        let second = ttf_parser::Face::parse(SOURCE_SANS_SECOND, 0).unwrap();
        assert!(first.glyph_index('Ɓ').is_none());
        assert!(second.glyph_index('Ɓ').is_some());
        let renderer = SkiaLayerRenderer::new()
            .try_with_font_data(&[SOURCE_SANS_FIRST.to_vec(), SOURCE_SANS_SECOND.to_vec()])
            .unwrap();
        let mut core =
            DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp")).unwrap();
        core.set_hangul2024_compat(false);
        let reference = core
            .build_page_layer_tree_with_profile(0, RenderProfile::Print)
            .unwrap();
        assert!(!reference.preserves_raster_text_geometry());
        let paint_at = |y| {
            let run = TextRunNode {
                text: "Ɓ".into(),
                style: TextStyle {
                    font_family: "Source Sans 3".into(),
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
            let mut tree = PageLayerTree::with_profile(
                64.0,
                64.0,
                LayerNode::leaf(
                    BoundingBox::new(0.0, 0.0, 64.0, 64.0),
                    None,
                    vec![PaintOp::text_run(BoundingBox::new(8.0, y, 40.0, 40.0), run)],
                ),
                RenderProfile::Print,
            );
            tree.resources = reference.resources.clone();
            center(
                &renderer
                    .render_pixels_with_options(&tree, RasterRenderOptions::default())
                    .unwrap(),
            )
        };
        let original = paint_at(4.1);
        let quarter = paint_at(4.35);
        let whole = paint_at(5.1);
        assert!(
            (quarter - original - 0.25).abs() <= 0.08,
            "같은 family의 선택된 face는 소수 원점을 보존한다: {original}, {quarter}"
        );
        assert!((whole - original - 1.0).abs() <= 0.02);
    }

    #[test]
    fn unrelated_fallback_does_not_inherit_original_legacy_outline_policy() {
        let renderer = SkiaLayerRenderer::new()
            .try_with_font_data(&[CFF.to_vec(), super::SOURCE_SANS_SECOND.to_vec()])
            .unwrap();
        let mut core =
            DocumentCore::from_bytes(include_bytes!("../../saved/blank2010.hwp")).unwrap();
        core.set_hangul2024_compat(false);
        let reference = core
            .build_page_layer_tree_with_profile(0, RenderProfile::Print)
            .unwrap();
        assert!(!reference.preserves_raster_text_geometry());
        let paint = |hft| {
            let run = TextRunNode {
                text: "Ɓ".into(),
                style: TextStyle {
                    font_family: format!("{FAMILY},Source Sans 3"),
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
            let mut tree = PageLayerTree::with_profile(
                64.0,
                64.0,
                LayerNode::leaf(
                    BoundingBox::new(0.0, 0.0, 64.0, 64.0),
                    None,
                    vec![PaintOp::text_run(
                        BoundingBox::new(8.0, 4.35, 40.0, 40.0),
                        run,
                    )],
                ),
                RenderProfile::Print,
            );
            tree.resources = reference.resources.clone();
            renderer
                .render_pixels_with_options(&tree, RasterRenderOptions::default())
                .unwrap()
        };
        let hft = paint(true);
        assert_eq!(
            hft,
            paint(false),
            "다른 family의 글꼴은 원래 래스터 경로를 유지한다"
        );
        assert!(
            center(&hft).is_finite(),
            "대체 글리프가 실제로 표시되어야 한다"
        );
    }
}

mod native_otf_program_contract {
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;

    #[test]
    fn standalone_cff_and_selected_collection_face_supply_their_own_glyphs() {
        let cff = include_bytes!("../fixtures/fonts/RHWPHftOutline.otf");
        let collection = include_bytes!("../fixtures/fonts/RHWPExactFaceSmoke.ttc");
        assert!(ttf_parser::Face::parse(cff, 0)
            .unwrap()
            .tables()
            .cff
            .is_some());
        let first = ttf_parser::Face::parse(collection, 0).unwrap();
        let second = ttf_parser::Face::parse(collection, 1).unwrap();
        assert!(second.tables().glyf.is_some());
        assert!(first.glyph_index('\u{e103}').is_some());
        assert!(second.glyph_index('\u{e104}').is_some());
        assert!(ttf_parser::Face::parse(collection, 2).is_err());

        let mut fonts = NativePuaFonts::default();
        fonts
            .register_otf("collection", false, false, cff, 0)
            .unwrap();
        fonts
            .register_otf("collection", false, false, collection, 1)
            .unwrap();
        let metrics = fonts.get_otf("collection", false, false).unwrap();
        assert!(metrics.advance_em('가').is_some());
        assert_eq!(metrics.advance_em('\u{e104}'), None);
        assert_eq!(metrics.source_ttf_advance_em('\u{e104}'), Some(1.0));
        assert!(!metrics.has_glyph('\u{e103}'));
    }

    #[test]
    fn split_family_uses_the_program_that_contains_each_character() {
        let mut fonts = NativePuaFonts::default();
        fonts
            .register_otf("Source Sans 3", false, false, super::SOURCE_SANS_FIRST, 0)
            .unwrap();
        let first = fonts.get_otf("Source Sans 3", false, false).unwrap();
        assert!(first.advance_em('A').is_some());
        assert!(first.advance_em('Ɓ').is_none());
        fonts
            .register_otf("Source Sans 3", false, false, super::SOURCE_SANS_SECOND, 0)
            .unwrap();
        let metrics = fonts.get_otf("Source Sans 3", false, false).unwrap();
        assert_eq!(metrics.advance_em('A'), first.advance_em('A'));
        let second = ttf_parser::Face::parse(super::SOURCE_SANS_SECOND, 0).unwrap();
        let glyph = second.glyph_index('Ɓ').unwrap();
        let advance =
            f64::from(second.glyph_hor_advance(glyph).unwrap()) / f64::from(second.units_per_em());
        assert_eq!(metrics.advance_em('Ɓ'), Some(advance));
        assert!(metrics.advance_em('가').is_none());
    }

    #[test]
    fn earlier_true_type_glyph_blocks_a_later_cff_advance() {
        let ttf = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");
        let cff = super::SOURCE_SANS_FIRST;
        let mut before = NativePuaFonts::default();
        before.register_otf("same", false, false, ttf, 0).unwrap();
        before.register_otf("same", false, false, cff, 0).unwrap();
        assert_eq!(
            before
                .get_otf("same", false, false)
                .unwrap()
                .advance_em('A'),
            None
        );

        let mut after = NativePuaFonts::default();
        after.register_otf("same", false, false, cff, 0).unwrap();
        after.register_otf("same", false, false, ttf, 0).unwrap();
        assert!(after
            .get_otf("same", false, false)
            .unwrap()
            .advance_em('A')
            .is_some());
    }
}

#[test]
fn declared_hwpx_tracking_family_reaches_the_final_run_without_authenticating_a_fallback() {
    use rhwp::model::paragraph::CharShapeRef;
    use rhwp::renderer::layout::{EmbeddedTextMeasurer, TextMeasurer};
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    use rhwp::renderer::TextStyle;
    fn run_styles(node: &RenderNode, result: &mut Vec<TextStyle>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text == "가" {
                result.push(run.style.clone());
            }
        }
        for child in &node.children {
            run_styles(child, result);
        }
    }
    let input = include_bytes!("../../mydocs/pr/assets/issue7774/sql-sample10-regressions-20261015/hwp3-sample10-hwpx.hwpx");
    let source = DocumentCore::from_bytes(input).unwrap().document().clone();
    // 같은 source 유형이거나 조회 이름만 같다는 이유로 다른 프로그램에 꾸밈을 빌려주지 않는다.
    for family in ["Senaru", "Kidney bean", "Strawberry"] {
        for (source_hft, embedded, declared) in [
            (true, false, true),
            (false, false, false),
            (true, true, false),
        ] {
            let mut document = source.clone();
            document.sections.truncate(1);
            document.sections[0].paragraphs.truncate(1);
            let para = &mut document.sections[0].paragraphs[0];
            para.text = "가".into();
            para.char_count = 2;
            para.char_offsets = vec![0];
            para.char_shapes = vec![CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            }];
            para.line_segs.clear();
            para.controls.clear();
            for faces in &mut document.doc_info.font_faces {
                if let Some(face) = faces.first_mut() {
                    face.name = family.into();
                    face.alt_type = if source_hft { 2 } else { 0 };
                    face.is_embedded = embedded;
                }
            }
            let shape = &mut document.doc_info.char_shapes[0];
            shape.font_ids = [0; 7];
            shape.ratios = [100; 7];
            shape.spacings = [0; 7];
            shape.relative_sizes = [100; 7];
            shape.base_size = 2000;
            shape.bold = true;
            let mut core = DocumentCore::new_empty();
            core.set_document(document);
            core.prepare_native_otf_fonts_with(|request| {
                (request.family == family).then(|| (CFF.to_vec(), 0))
            })
            .unwrap();
            let tree = core.build_page_render_tree(0).unwrap();
            let mut runs = Vec::new();
            run_styles(&tree.root, &mut runs);
            assert_eq!(runs.len(), 1, "내용을 누락·중복하지 않는다");
            let style = &runs[0];
            let program_verified = style
                .native_otf_metrics
                .as_ref()
                .is_some_and(|font| font.has_verified_hft_bold_tracking('가'));
            assert_eq!(
                style.source_verified_hft_bold_tracking, declared,
                "source declaration: {family}/{source_hft}/{embedded}"
            );
            assert!(
                !program_verified,
                "a query name cannot certify the fallback program"
            );
            if declared {
                let program = style.native_otf_metrics.as_ref().unwrap();
                assert!(program.has_glyph('가'));
                assert!(program.needs_faux_bold('가'));
                // 기존 전각 fallback 전진은 유지한 채 이 선언의 효과만 확인한다.
                let mut without_verification = style.clone();
                without_verification.source_verified_hft_bold_tracking = false;
                let measure =
                    |style: &TextStyle| EmbeddedTextMeasurer.compute_char_positions("가", style)[1];
                assert_eq!(measure(style), measure(&without_verification));
            }
        }
    }
}

#[test]
fn static_cff_registration_preserves_source_cmap_and_advances() {
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;
    let source = ttf_parser::Face::parse(CFF, 0).unwrap();
    let mut fonts = NativePuaFonts::default();
    fonts.register_otf(FAMILY, false, false, CFF, 0).unwrap();
    let metrics = fonts.get_otf(FAMILY, false, false).unwrap();
    // 원본 cmap의 모든 지원 문자와 결측 대조군을 공개 등록·측정 경로로 대조한다.
    let mut characters = vec!['A', ' ', '?'];
    for subtable in source.tables().cmap.unwrap().subtables {
        if subtable.is_unicode() {
            subtable.codepoints(|code| {
                if let Some(ch) = char::from_u32(code) {
                    characters.push(ch);
                }
            });
        }
    }
    for ch in characters {
        let glyph = source.glyph_index(ch);
        assert_eq!(metrics.has_glyph(ch), glyph.is_some());
        let expected = glyph
            .and_then(|id| source.glyph_hor_advance(id))
            .map(|width| f64::from(width) / f64::from(source.units_per_em()));
        assert_eq!(metrics.advance_em(ch), expected, "원본 cmap/hmtx: {ch:?}");
    }
}

#[cfg(feature = "native-skia")]
#[test]
fn original_legacy_latin_punctuation_precedes_a_hangul_slots_display_ttf_fallback() {
    use rhwp::renderer::{
        font_environment::FontEnvironment,
        render_tree::{RenderNode, RenderNodeType},
        skia::SkiaLayerRenderer,
    };
    fn punctuation(node: &RenderNode, output: &mut Vec<String>) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if run.text.contains('(') {
                output.push(run.style.font_family.clone());
            }
        }
        for child in &node.children {
            punctuation(child, output);
        }
    }
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가(가").unwrap();
    let mut document = core.document().clone();
    document.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
    for (lang, faces) in document.doc_info.font_faces.iter_mut().enumerate() {
        faces[0].name = if lang == 1 { "Source Sans 3" } else { FAMILY }.into();
        faces[0].alt_type = 2;
    }
    for style in &mut document.doc_info.char_shapes {
        style.font_ids = [0; 7];
        style.relative_sizes = [100; 7];
        style.ratios = [100; 7];
        style.spacings = [0; 7];
    }
    core.set_document(document);
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"primary-cff-with-ttf-fallback","substitutions":{},"preserveSourceMetrics":true,"glyphFallbacks":{"RHWP HFT Outline Fixture":["RHWP Equation Contract"]}}"#
    ).unwrap())).unwrap();
    let ttf = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf").to_vec();
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[CFF.to_vec(), SOURCE_SANS_SECOND.to_vec(), ttf])
        .unwrap();
    renderer.prepare_native_otf_fonts(&mut core).unwrap();
    let mut owners = Vec::new();
    punctuation(&core.build_page_render_tree(0).unwrap().root, &mut owners);
    assert_eq!(owners.len(), 1, "원문의 구두점 누락·중복 금지");
    assert!(
        owners[0].starts_with("Source Sans 3"),
        "원본 영문 글꼴이 괄호의 윤곽을 공급한다: {owners:?}"
    );
    assert_eq!(core.document().sections[0].paragraphs[0].text, "가(가");
}
