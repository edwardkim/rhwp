//! 관계형: 미분류 원본의 프로그램 조회와 명시적 자연 전진 선택을 구분한다.
//! 주석 갱신: 2026-11-09T20:15:00+09:00.
//! OFL 합성 TTF를 직접 공급하며 설치 글꼴·실문서 좌표·px 기대값에 의존하지 않는다.
//! 실제 본문 run의 공급 전후 전진 소유를 검사하므로 PDF/Native/WASM 일치율은 비해당이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::{
    font_environment::FontEnvironment,
    layout::{EmbeddedTextMeasurer, TextMeasurer},
    render_tree::{RenderNode, RenderNodeType},
    TextStyle,
};
use rhwp::DocumentCore;

const FAMILY: &str = "Rhwp Family Contract";
const PROGRAM: &[u8] = include_bytes!("../fixtures/native_font_family_names/family-contract.ttf");

fn source() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "AVI AV").unwrap();
    let mut document = core.document().clone();
    document.provenance.format = rhwp::model::provenance::SourceFormat::Hwp5;
    document.provenance.hwpx_lineage = false;
    for font in document.doc_info.font_faces.iter_mut().flatten() {
        font.name = FAMILY.into();
        font.alt_type = 1;
        font.subst_font = None;
    }
    core.set_document(document);
    core
}

fn style(node: &RenderNode) -> Option<TextStyle> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text.contains("AVI") {
            return Some(run.style.clone());
        }
    }
    node.children.iter().find_map(style)
}

fn advances(core: &DocumentCore) -> Vec<f64> {
    let tree = core.build_page_render_tree(0).unwrap();
    let style = style(&tree.root).expect("공급 전후 동일한 본문 run");
    EmbeddedTextMeasurer.compute_char_positions("AVI AV", &style)
}

fn supply(core: &mut DocumentCore) {
    let requests = core.native_otf_font_requests();
    assert!(requests
        .iter()
        .any(|request| request.family == FAMILY && request.source_ttf));
    assert!(core
        .prepare_native_otf_fonts_with(|request| {
            (request.family == FAMILY).then(|| (PROGRAM.to_vec(), 0))
        })
        .unwrap());
}

#[test]
fn unclassified_program_probe_does_not_replace_existing_advances() {
    let mut core = source();
    let original = format!("{:?}", core.document());
    let before = advances(&core);
    supply(&mut core);
    assert_eq!(
        advances(&core),
        before,
        "같은 이름의 프로그램을 찾은 사실만으로 미분류 원본의 전진을 바꾸지 않는다"
    );
    assert_eq!(format!("{:?}", core.document()), original);
}

#[test]
fn explicitly_selected_natural_program_still_supplies_its_advances() {
    let mut core = source();
    core.set_font_environment(Some(FontEnvironment::from_json(
        r#"{"id":"natural-program-contract","substitutions":{"Rhwp Family Contract":"Rhwp Family Contract"},"trueTypeFaces":["Rhwp Family Contract"],"preserveSourceMetrics":false}"#,
    ).unwrap())).unwrap();
    let before = advances(&core);
    supply(&mut core);
    assert_ne!(
        advances(&core),
        before,
        "명시적 자연 전진의 실제 공급은 계속 소비한다"
    );
}
