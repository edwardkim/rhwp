//! 관계형·고정형: 빈 cmap 기호와 실제 fallback의 잉크를 최종 Native paint에서 확인한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! 이 검사는 합성 입력을 사용하며 아래 수치는 관련 편람의 별도 시각 검증이다.
//! HWP 200쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 200쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
#![cfg(all(feature = "native-skia", not(target_arch = "wasm32")))]

#[path = "../common/symbol_font_program.rs"]
mod symbol_font_program;

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
