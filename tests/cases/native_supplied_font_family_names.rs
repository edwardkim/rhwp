//! Native font identity contract, not a Hancom layout or pixel oracle.
//! The small renamed OFL font retains only A and declares independent SFNT
//! family, typographic, WWS and full-face naming records. The document below
//! is explicitly synthetic and has no stored line-segment expectations.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::renderer::skia::SkiaLayerRenderer;
use rhwp::DocumentCore;

const PRIMARY: &str = "Rhwp Family Contract";

fn decision(requested: &str) -> serde_json::Value {
    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/task-001.hwp"))
        .expect("public document supplies valid section and style defaults");
    let mut doc = core.document().clone();
    doc.sections.truncate(1);
    let mut paragraph = Paragraph::new_empty();
    paragraph.text = "A".into();
    paragraph.char_count = 2;
    paragraph.char_offsets = vec![0];
    paragraph.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: 0,
    }];
    paragraph.line_segs.clear();
    doc.sections[0].paragraphs = vec![paragraph];
    for font in doc.doc_info.font_faces.iter_mut().flatten() {
        font.name = requested.into();
        font.is_embedded = false;
        font.alt_name = None;
        font.default_name = None;
        font.subst_font = None;
    }
    core.set_document(doc);
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native_font_family_names");
    let renderer = SkiaLayerRenderer::new().with_font_paths(&[directory]);
    let trace: serde_json::Value = serde_json::from_str(
        &renderer
            .get_font_decision_trace(&core, 0, r#"{"maxCharacters":64}"#)
            .expect("renderer-bound font observation"),
    )
    .expect("trace JSON");
    let records: Vec<_> = trace["records"]
        .as_array()
        .expect("font records")
        .iter()
        .filter(|record| record["source"]["character"] == "A")
        .collect();
    assert_eq!(records.len(), 1, "one synthetic source character");
    let native = records[0]["paint"]["native"].clone();
    assert_eq!(native["status"], "complete");
    assert_eq!(native["requested"], requested);
    native
}

fn assert_supplied(requested: &str) {
    let native = decision(requested);
    assert_eq!(
        native["resolved"], PRIMARY,
        "declared family owns its program"
    );
    assert_eq!(native["source"], "custom");
    assert_eq!(native["failures"], serde_json::json!([]));
}

#[test]
fn primary_family_keeps_its_supplied_program() {
    assert_supplied(PRIMARY);
}

#[test]
fn localized_family_selects_the_same_supplied_program() {
    assert_supplied("검사용 가족 이름");
}

#[test]
fn typographic_family_selects_the_same_supplied_program() {
    assert_supplied("검사용 표기 가족");
}

#[test]
fn wws_family_selects_the_same_supplied_program() {
    assert_supplied("검사용 WWS 가족");
}

#[test]
fn full_face_record_does_not_become_a_family_alias() {
    assert_ne!(decision("검사용 전체 얼굴 이름")["resolved"], PRIMARY);
}
