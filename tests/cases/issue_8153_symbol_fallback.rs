//! 관계형: 빈 cmap 기호와 실제 fallback의 잉크를 최종 Native paint에서 확인한다.
//! 주석 갱신: 2026-11-06 04:45:01 KST; 직전 파일 커밋: c417310cab 2026-10-30T19:50:11+09:00.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! 이 검사는 합성 입력을 사용하며 아래 수치는 관련 편람의 별도 시각 검증이다.
//! HWP 200쪽 Native 100.00% / fresh WASM 100.00%.
//! HWP 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! HWPX 200쪽 Native 100.00% / fresh WASM 100.00%.
//! HWPX 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

// 파생 suite 루트에서 한 번 로딩한 동일 helper를 공유한다. 검사 본문은 유지한다.
use super::symbol_font_program;

use rhwp::paint::RenderProfile;
use rhwp::renderer::{layer_renderer::LayerRasterRenderer, skia::SkiaLayerRenderer};
use rhwp::{model::style::SubstFont, DocumentCore};

#[test]
fn mapped_empty_symbols_use_real_fallback_without_changing_source_or_positions() {
    let primary = symbol_font_program::mapped_program(
        include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf"),
        &[(' ', 1), ('A', 2), ('⚫', 1), ('∑', 1), ('€', 1)],
    );
    let fallback = symbol_font_program::mapped_program(
        include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf"),
        &[('A', 1), ('⚫', 1), ('∑', 1), ('€', 1)],
    );
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "⚫∑€").unwrap();
    let mut document = core.document().clone();
    for language in &mut document.doc_info.font_faces {
        for font in language {
            font.name = "RHWP Kerning Boundary".into();
            font.subst_font = Some(SubstFont {
                face: "RHWP Host Fixture".into(),
                font_type: 1,
                ..Default::default()
            });
        }
    }
    core.set_document(document);
    let source = core.export_hwp_native().unwrap();
    let tree = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let with_empty = SkiaLayerRenderer::new()
        .try_with_font_data(&[primary, fallback.clone()])
        .unwrap();
    let reference = SkiaLayerRenderer::new()
        .try_with_font_data(&[fallback])
        .unwrap();
    let expected = reference.render_png(&tree).unwrap();
    let actual = with_empty.render_png(&tree).unwrap();
    assert!(
        actual == expected,
        "앞 face의 빈 기호가 같은 fallback의 실제 잉크를 가리지 않는다"
    );
    let trace: serde_json::Value = serde_json::from_str(
        &with_empty
            .get_font_decision_trace(&core, 0, r#"{"maxCharacters":16}"#)
            .unwrap(),
    )
    .unwrap();
    for symbol in ['⚫', '∑', '€'] {
        let record = trace["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["source"]["character"] == symbol.to_string())
            .unwrap();
        assert_eq!(record["paint"]["native"]["resolved"], "RHWP Host Fixture");
    }
    assert_eq!(
        core.export_hwp_native().unwrap(),
        source,
        "공급 정책은 원문/글꼴 선언/저장본을 바꾸지 않는다"
    );
}

// 관계형: 같은 주 프로그램의 결측 기호는 파일의 유형 선언 유무와 무관하게 같은 반각 칸에 그린다.
// 주석 갱신: 2026-11-09T01:08:28+09:00. 실물 좌표를 고정하지 않는 합성 Native paint 검사다.
// 관련 편람 HWP 198쪽 보정 전 raw: Native 99.71%, fresh WASM 99.68%. 독립 Print와 직접 판독 후 추가.
// MIT 합성 TTF·CFF를 메모리에서만 공급하며 한컴·개인 글꼴이나 OS 설치에 의존하지 않는다.
#[test]
fn missing_primary_symbol_keeps_the_same_slot_with_unclassified_font_type() {
    use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
    use rhwp::renderer::native_pua_fonts::NativePuaFonts;
    use rhwp::renderer::render_tree::{BoundingBox, TextRunNode};
    use rhwp::renderer::TextStyle;

    let primary = include_bytes!("../fixtures/fonts/RHWPEquationLatinOnly.ttf").to_vec();
    let fallback = symbol_font_program::mapped_program(
        include_bytes!("../fixtures/fonts/RHWPHftOutline.otf"),
        &[('∙', 1)],
    );
    let renderer = SkiaLayerRenderer::new()
        .try_with_font_data(&[primary.clone(), fallback.clone()])
        .unwrap();
    let family = "RHWP Equation Contract,RHWP HFT Outline Fixture";
    let mut fonts = NativePuaFonts::default();
    for declared_ttf in [false, true] {
        fonts
            .register_otf_for_source(family, false, false, declared_ttf, &primary, 0)
            .unwrap();
    }
    // 유형 미상 공급은 실제 표시 후보도 적재한다. 뒤 CFF의 coverage는 주 TTF의 coverage가 아니다.
    fonts
        .register_otf_for_source(family, false, false, false, &fallback, 0)
        .unwrap();
    let render = |declared_ttf| {
        let run = TextRunNode {
            text: "∙".into(),
            style: TextStyle {
                font_family: family.into(),
                font_size: 12.0,
                color: 0,
                letter_spacing: -0.4,
                source_font_is_ttf: declared_ttf,
                native_otf_metrics: fonts.get_otf_for_source(family, false, false, declared_ttf),
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
            baseline: 12.0,
            field_marker: Default::default(),
            layout_positions: Some(vec![0.0, 6.0]),
            display_text: None,
        };
        let tree = PageLayerTree::new(
            48.0,
            32.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 48.0, 32.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(20.0, 4.0, 6.0, 12.0),
                    run,
                )],
            ),
        );
        renderer.render_png(&tree).unwrap()
    };
    let explicit = render(true);
    let unclassified = render(false);
    assert!(
        unclassified == explicit,
        "같은 실제 TTF 결측과 같은 fallback 잉크는 유형 선언만으로 원점이 달라지지 않는다"
    );
}
