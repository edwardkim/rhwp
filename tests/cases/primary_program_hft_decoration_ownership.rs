//! 관계형: 실제 주 프로그램의 전진과 HFT 대체 프로그램의 굵기 전진 소유를 구분한다.
//! 주석 갱신: 2026-11-08T18:32:04+09:00.
//! 공개 CFF·문서/글꼴 환경 API의 합성 계약이며 설치 글꼴·실문서 px·쪽수 기대값에 의존하지 않는다.
//! 실문서 raw·DPI·Print 쪽수는 이 합성 계약에 비해당이다. 원인 및 실제 Print 대조는
//! mydocs/pr/assets/planet6897_20261106/native-weight-r46g-analysis.md의 R52 절을 따른다.
use rhwp::renderer::{
    font_environment::FontEnvironment,
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    render_tree::{RenderNode, RenderNodeType},
    TextStyle,
};
use rhwp::DocumentCore;
use sha2::{Digest, Sha256};

fn text_style(node: &RenderNode) -> Option<TextStyle> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains('가') {
            return Some(run.style.clone());
        }
    }
    node.children.iter().find_map(text_style)
}

fn selected_style(bold: bool, spacing: i8, original_hwp3: bool, program_id: &str) -> TextStyle {
    supplied_style(bold, spacing, original_hwp3, program_id, false)
}

fn supplied_style(
    bold: bool,
    spacing: i8,
    original_hwp3: bool,
    program_id: &str,
    substitute: bool,
) -> TextStyle {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fonts/RHWPHftOutline.otf"
    ));
    let family = "RHWP HFT Outline Fixture";
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let mut doc = core.document().clone();
    doc.provenance.format = if original_hwp3 {
        rhwp::model::provenance::SourceFormat::Hwp3
    } else {
        rhwp::model::provenance::SourceFormat::Hwp5
    };
    for face in doc.doc_info.font_faces.iter_mut().flatten() {
        face.name = if substitute {
            "Unavailable HFT fixture"
        } else {
            family
        }
        .into();
        face.alt_type = 2;
        face.subst_font = substitute.then(|| rhwp::model::style::SubstFont {
            face: family.into(),
            font_type: 2,
            ..Default::default()
        });
        face.native_program = None;
    }
    for shape in &mut doc.doc_info.char_shapes {
        shape.bold = bold;
        shape.italic = false;
        shape.base_size = 1500;
        shape.spacings = [spacing; 7];
        shape.ratios = [100; 7];
        shape.relative_sizes = [100; 7];
    }
    let json = format!(
        r#"[{{"query":"{family}","family":"{family}","programId":"{program_id}","trueType":false,"openTypeCff":true}}]"#
    );
    let environment = FontEnvironment::from_available_programs(&doc.doc_info, &json)
        .unwrap()
        .unwrap();
    core.set_document(doc);
    core.insert_text_native(0, 0, 0, "가가").unwrap();
    core.set_font_environment(Some(environment)).unwrap();
    core.prepare_native_otf_fonts_with(|_| Some((bytes.to_vec(), 0)))
        .unwrap();
    text_style(&core.build_page_render_tree(0).unwrap().root).unwrap()
}

fn positions(style: &TextStyle) -> Vec<f64> {
    EmbeddedTextMeasurer.compute_char_positions("가가", style)
}

#[test]
fn real_primary_program_does_not_inherit_inferred_hft_decoration() {
    let id = program_identity(false);
    for spacing in [-10, 0, 10] {
        let regular = selected_style(false, spacing, false, &id);
        let bold = selected_style(true, spacing, false, &id);
        assert_eq!(
            positions(&bold),
            positions(&regular),
            "실제 주 CFF의 합성 굵기는 전진을 바꾸지 않는다; 원본 자간 {spacing}"
        );
    }
}

#[test]
fn another_program_does_not_borrow_primary_selection_identity() {
    let id = "0000000000000000000000000000000000000000000000000000000000000000";
    let regular = selected_style(false, 0, false, id);
    let bold = selected_style(true, 0, false, id);
    assert!(
        positions(&bold)[1] > positions(&regular)[1],
        "다른 프로그램은 기존 HFT 대체 꾸밈 경계를 유지한다"
    );
}

#[test]
fn original_hwp3_decoration_keeps_its_independent_ownership() {
    let id = program_identity(false);
    let regular = selected_style(false, 0, true, &id);
    let bold = selected_style(true, 0, true, &id);
    assert!(
        positions(&bold)[1] > positions(&regular)[1],
        "원본 HWP3의 확인된 꾸밈 계약은 실제 주 프로그램 공급 뒤에도 유지한다"
    );
}

#[test]
fn document_substitute_does_not_become_the_original_primary_program() {
    let id = program_identity(false);
    let regular = supplied_style(false, 0, false, &id, true);
    let bold = supplied_style(true, 0, false, &id, true);
    assert!(
        positions(&bold)[1] > positions(&regular)[1],
        "문서 지정 대체 프로그램도 원본 주 프로그램 선택으로 승격하지 않는다"
    );
}

fn program_identity(standalone: bool) -> String {
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/fonts/RHWPHftOutline.otf"
    ));
    let mut hash = Sha256::new();
    hash.update(bytes);
    if !standalone {
        hash.update(0u32.to_be_bytes());
    }
    let digest: String = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if standalone {
        format!("sweep-{digest}")
    } else {
        digest
    }
}

#[test]
fn standalone_browser_program_uses_the_same_primary_ownership() {
    let id = program_identity(true);
    for spacing in [-10, 0, 10] {
        let regular = selected_style(false, spacing, false, &id);
        let bold = selected_style(true, spacing, false, &id);
        assert_eq!(
            positions(&bold),
            positions(&regular),
            "브라우저의 실제 독립 SFNT 신원도 같은 주 프로그램 전진을 소비한다: {spacing}"
        );
    }
}
