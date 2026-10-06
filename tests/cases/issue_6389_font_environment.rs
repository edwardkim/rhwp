//! Explicit environment contract; the real saved-line oracle is in the companion #6389 test.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use serde_json::{json, Value};

fn environment() -> FontEnvironment {
    FontEnvironment::from_json(
        r#"{"id":"hancom-no-kopub","substitutions":{"KoPub돋움체 Light":"바탕"}}"#,
    )
    .unwrap()
}

fn core(embedded: bool) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나다라마바사아자차카타파하")
        .unwrap();
    let mut doc = core.document().clone();
    for fonts in &mut doc.doc_info.font_faces {
        for font in fonts {
            font.name = "KoPub돋움체 Light".into();
            font.is_embedded = embedded;
        }
    }
    for style in &mut doc.doc_info.char_shapes {
        style.base_size = 1000;
        style.ratios = [100; 7];
        style.spacings = [0; 7];
    }
    core.set_document(doc);
    core
}

fn trace(core: &DocumentCore) -> Value {
    serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":100}"#)
            .unwrap(),
    )
    .unwrap()
}

fn first(trace: &Value) -> &Value {
    trace["records"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["source"]["character"] == "가")
        .expect("Hangul record")
}

#[test]
fn issue_6389_environment_changes_measurement_and_paint_without_editing_document() {
    let mut core = core(false);
    let original = core.export_hwp_native().unwrap();
    let before = trace(&core);
    let default_record = first(&before);
    assert_eq!(default_record["layoutMetric"]["widthSource"], "kopubTable");
    // #6389 embedded PDF CIDFont /W: 872/1000, independent of the environment API.
    assert_eq!(default_record["layoutMetric"]["baseAdvanceHwpunit"], 872);
    assert!(core.set_font_environment(Some(environment())).unwrap());
    let after = trace(&core);
    let record = first(&after);
    assert_eq!(record["document"]["face"], "KoPub돋움체 Light");
    assert_eq!(record["layoutName"]["normalizedFace"], "바탕");
    assert_eq!(record["paint"]["native"]["requested"], "바탕");
    let info: Value = serde_json::from_str(&core.get_document_info()).unwrap();
    assert!(info["fontsUsed"]
        .as_array()
        .unwrap()
        .contains(&json!("바탕")));
    assert!(!info["fontsUsed"]
        .as_array()
        .unwrap()
        .contains(&json!("KoPub돋움체 Light")));
    assert_ne!(record["layoutMetric"]["widthSource"], "kopubTable");
    // Batang's full-width Hangul in the no-KoPub oracle is 1000/1000 em.
    assert_eq!(record["layoutMetric"]["baseAdvanceHwpunit"], 1000);
    assert_eq!(record["oracle"]["status"], "declared");
    assert_eq!(record["oracle"]["profileId"], "hancom-no-kopub");
    assert_eq!(core.export_hwp_native().unwrap(), original);
    assert!(!core.set_font_environment(Some(environment())).unwrap());
    assert_eq!(trace(&core), after);
    core.set_dpi(96.0); // style rebuild must retain the environment
    assert_eq!(first(&trace(&core))["layoutName"]["normalizedFace"], "바탕");
    assert!(core.set_font_environment(None).unwrap());
    assert_eq!(trace(&core), before);
    assert_eq!(core.export_hwp_native().unwrap(), original);
    let reopened = DocumentCore::from_bytes(&original).unwrap();
    assert!(reopened.font_environment().is_none());
}

#[test]
fn issue_6389_environment_is_session_local_and_preserves_embedded_fonts() {
    let mut selected = core(false);
    let other = core(false);
    selected.set_font_environment(Some(environment())).unwrap();
    assert_eq!(
        first(&trace(&other))["layoutMetric"]["widthSource"],
        "kopubTable"
    );
    let mut embedded = core(true);
    embedded.set_font_environment(Some(environment())).unwrap();
    assert_eq!(
        first(&trace(&embedded))["layoutName"]["normalizedFace"],
        "KoPub돋움체 Light"
    );
    assert_eq!(
        first(&trace(&embedded))["layoutMetric"]["widthSource"],
        "kopubTable"
    );
}

#[test]
fn issue_6389_environment_rejects_invalid_declarations_and_active_batches() {
    for invalid in [
        json!({"id":"", "substitutions":{}}),
        json!({"id":"x", "substitutions":{"KoPub돋움체 Light":""}}),
        json!({"id":"x", "substitutions":{"KoPub돋움체 Light":"바탕,serif"}}),
        json!({"id":"x", "substitutions":{" KoPub돋움체 Light":"바탕"}}),
        json!({"id":"x", "substitutions":{}, "typo":true}),
    ] {
        assert!(FontEnvironment::from_json(&invalid.to_string()).is_err());
    }
    let mut core = core(false);
    core.begin_batch_native().unwrap();
    assert!(core.set_font_environment(Some(environment())).is_err());
    assert!(core.font_environment().is_none());
}

#[test]
fn issue_6389_cli_uses_the_environment_for_svg_tree_and_pdf() {
    use std::process::Command;
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6389-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&dir).unwrap();
    let input = dir.join("source.hwp");
    let profile = dir.join("environment.json");
    std::fs::write(&input, core(false).export_hwp_native().unwrap()).unwrap();
    std::fs::write(&profile, serde_json::to_string(&environment()).unwrap()).unwrap();
    for command in ["export-svg", "export-render-tree", "export-pdf"] {
        let out = dir.join(command);
        let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
            .arg(command)
            .arg(&input)
            .arg("--font-environment")
            .arg(&profile)
            .arg("-o")
            .arg(&out)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{command}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        if command == "export-svg" {
            let svg = std::fs::read_to_string(out.join("source.svg")).unwrap();
            assert!(svg.contains("바탕"));
            assert!(!svg.contains("KoPub돋움체 Light"));
        } else if command == "export-pdf" {
            assert!(std::fs::read(out).unwrap().starts_with(b"%PDF-"));
        } else {
            assert!(std::fs::read_dir(out).unwrap().next().is_some());
        }
    }
    std::fs::write(
        &profile,
        r#"{"id":"invalid","substitutions":{"x":"바탕,serif"}}"#,
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-svg")
        .arg(&input)
        .arg("--font-environment")
        .arg(&profile)
        .arg("-o")
        .arg(dir.join("invalid-output"))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(!dir.join("invalid-output").exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn issue_6389_reflow_uses_selected_metrics_without_a_stored_line_cache() {
    use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
    fn text(node: &RenderNode) -> String {
        let own = match &node.node_type {
            RenderNodeType::TextRun(run) => run.text.clone(),
            _ => String::new(),
        };
        own + &node.children.iter().map(text).collect::<String>()
    }
    fn lines(node: &RenderNode, out: &mut Vec<usize>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let count = text(node)
                .chars()
                .filter(|c| ('가'..='힣').contains(c))
                .count();
            if count > 0 {
                out.push(count);
            }
        } else {
            for child in &node.children {
                lines(child, out);
            }
        }
    }
    let mut core = core(false);
    core.insert_text_native(0, 0, 14, &"가".repeat(46)).unwrap();
    let mut document = core.document().clone();
    let page = &mut document.sections[0].section_def.page_def;
    page.width = 20000;
    page.margin_left = 1000;
    page.margin_right = 1000;
    page.margin_gutter = 0;
    document.sections[0].paragraphs[0].line_segs.clear();
    core.set_document(document);
    let mut portable = Vec::new();
    lines(&core.build_page_render_tree(0).unwrap().root, &mut portable);
    // Independent geometry: 18000 HU / 872 HU = 20 glyphs; / 1000 HU = 18.
    assert_eq!(portable, [20, 20, 20]);
    core.set_font_environment(Some(environment())).unwrap();
    let mut substituted = Vec::new();
    lines(
        &core.build_page_render_tree(0).unwrap().root,
        &mut substituted,
    );
    assert_eq!(substituted, [18, 18, 18, 6]);
    assert!(core.document().sections[0].paragraphs[0]
        .line_segs
        .is_empty());
}
