//! 검사 유형: 관계형(수식과 본문의 문단 소속·순서, 수식 프레임과 클릭 소유·편집·저장 왕복).
//! 주석 갱신: 2026-11-27 17:57:02 KST. 직전 변경: 2026-11-27 08:29:03 +0900, 397546258e81783af0f325a1aa5c3f7838a11882.
//! 측정: 2026-11-27, 한컴 2024 Print PDFium 96dpi, compat2024, 색상 경계 보정 전 2px raw.
//! 7785-hml 전 1쪽 최저는 Native 100.00%, fresh WASM 100.00%였다.
//! 1쪽은 Native 100.00%, fresh WASM 100.00%였다.
//! 원본은 samples/hml/exambank_math_equations_min.hml이다.
//! 독립 Print PDF는 samples/hml/exambank_math_equations_min-2024.pdf다.
//! 전쪽 TSV는 mydocs/pr/assets/issues7777_7787/native/7785-hml/silhouette.tsv에 있다.
//! 전쪽 TSV는 mydocs/pr/assets/issues7777_7787/wasm/7785-hml/silhouette.tsv에 있다.
//! 수식 네 개를 독립 Print와 확대 대조했다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, Equation};
use rhwp::model::document::Document;
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{BoundingBox, EquationNode, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const INPUT: &[u8] = include_bytes!("../../samples/hml/exambank_math_equations_min.hml");
const SOURCE_OWNERS: [(usize, &str); 4] = [(2, "x^2 +1"), (3, "x^2 +1"), (12, "3"), (13, "3")];

fn equations(document: &Document) -> Vec<(usize, &Equation)> {
    document.sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .flat_map(|(index, para)| {
            para.controls.iter().filter_map(move |control| {
                if let Control::Equation(equation) = control {
                    Some((index, equation.as_ref()))
                } else {
                    None
                }
            })
        })
        .collect()
}

fn scripts(document: &Document) -> Vec<(usize, &str)> {
    equations(document)
        .into_iter()
        .map(|(owner, equation)| (owner, equation.script.as_str()))
        .collect()
}

#[test]
fn hml_equations_keep_source_owners_and_import_metadata() {
    let parsed = rhwp::parser::hml::parse_hml(INPUT).unwrap();
    assert_eq!(scripts(&parsed.document), SOURCE_OWNERS);
    assert!(equations(&parsed.document).iter().all(|(_, equation)| {
        equation.baseline == 65
            && equation.font_size == 1000
            && equation.color == 0
            && equation.version_info == "Equation Version 60"
    }));
    assert!(parsed
        .warnings
        .iter()
        .all(|warning| !warning.xml_path.contains("EQUATION")));
    let para = &parsed.document.sections[0].paragraphs[2];
    assert_eq!(para.text, "다항식 을 전개하시오.");
    assert_eq!(
        para.char_offsets,
        [0, 1, 2, 3, 12, 13, 14, 15, 16, 17, 18, 19]
    );
    assert_eq!(para.char_count, 21);
    let original = std::str::from_utf8(INPUT).unwrap();
    for identifier in [
        "ExamBank",
        "exambank",
        "serial_curated",
        "http://",
        "https://",
        "/Users/",
        "\\Users\\",
        "@",
    ] {
        assert!(!original.contains(identifier), "{identifier}");
    }
}

fn print_equations<'a>(
    node: &'a LayerNode,
    ordered: &mut Vec<(&'a BoundingBox, &'a EquationNode)>,
) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                print_equations(child, ordered);
            }
        }
        LayerNodeKind::ClipPath { child, .. } | LayerNodeKind::ClipRect { child, .. } => {
            print_equations(child, ordered);
        }
        LayerNodeKind::Leaf { ops } => {
            ordered.extend(ops.iter().filter_map(|op| match op {
                PaintOp::Equation { bbox, equation } => Some((bbox, equation.as_ref())),
                _ => None,
            }));
        }
    }
}

fn rendered_text(node: &RenderNode, paragraphs: &mut [String]) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let Some(owner) = run.para_index {
            paragraphs[owner].push_str(&run.text);
        }
    }
    for child in &node.children {
        rendered_text(child, paragraphs);
    }
}

#[test]
fn print_retains_equation_order_and_body_ownership() {
    let core = DocumentCore::from_bytes(INPUT).unwrap();
    let layer = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let mut printed = Vec::new();
    print_equations(&layer.root, &mut printed);
    let owners: Vec<_> = printed
        .iter()
        .map(|(_, eq)| (eq.para_index.unwrap(), eq.script.as_str()))
        .collect();
    assert_eq!(owners, SOURCE_OWNERS);
    let left = printed[0].0.x;
    assert!(printed
        .iter()
        .all(|(bbox, _)| bbox.width > 0.0 && bbox.height > 0.0 && (bbox.x - left).abs() <= 0.1));
    assert!(printed.windows(2).all(|pair| pair[0].0.y < pair[1].0.y));

    let input_paragraphs = &core.document().sections[0].paragraphs;
    let mut preserved = vec![String::new(); input_paragraphs.len()];
    rendered_text(
        &core.build_page_render_tree(0).unwrap().root,
        &mut preserved,
    );
    let expected_text: Vec<_> = input_paragraphs
        .iter()
        .map(|para| para.text.as_str())
        .collect();
    assert_eq!(preserved, expected_text);

    let layout: Value =
        serde_json::from_str(&core.get_page_control_layout_native(0).unwrap()).unwrap();
    let selectable: Vec<_> = layout["controls"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|control| control["type"] == "equation")
        .collect();
    assert_eq!(selectable.len(), SOURCE_OWNERS.len());
    for (control, (owner, _)) in selectable.into_iter().zip(SOURCE_OWNERS) {
        assert_eq!(control["paraIdx"].as_u64(), Some(owner as u64));
        assert_eq!(control["controlIdx"].as_u64(), Some(0));
        let width = control["w"].as_f64().unwrap();
        let height = control["h"].as_f64().unwrap();
        assert!(width > 0.0 && height > 0.0);
        let x = control["x"].as_f64().unwrap() + width / 2.0;
        let y = control["y"].as_f64().unwrap() + height / 2.0;
        let hit: Value = serde_json::from_str(&core.hit_test_native(0, x, y).unwrap()).unwrap();
        assert_eq!(hit["paragraphIndex"].as_u64(), Some(owner as u64), "{hit}");
    }
}

#[test]
fn equation_script_edit_preserves_hml_frame_and_other_scripts() {
    let mut core = DocumentCore::from_bytes(INPUT).unwrap();
    let frame = |equation: &Equation| {
        (
            equation.common.width,
            equation.common.height,
            equation.common.treat_as_char,
        )
    };
    let original_frame = frame(equations(core.document())[0].1);
    core.set_equation_properties_native(0, 2, 0, None, None, r#"{"script":"a < b & \"c\""}"#)
        .unwrap();
    assert_eq!(frame(equations(core.document())[0].1), original_frame);

    let bytes = core.export_hml_native().unwrap();
    let xml = std::str::from_utf8(&bytes).unwrap();
    assert!(xml.contains("<SCRIPT>a &lt; b &amp; \"c\"</SCRIPT>"));
    let reopened = DocumentCore::from_bytes(&bytes).unwrap();
    let mut expected = SOURCE_OWNERS;
    expected[0].1 = "a < b & \"c\"";
    assert_eq!(scripts(reopened.document()), expected);
    assert_eq!(frame(equations(reopened.document())[0].1), original_frame);
}

#[test]
fn equation_script_and_character_offset_errors_are_reported_together() {
    let mut core = DocumentCore::from_bytes(INPUT).unwrap();
    let para = &mut core.document_mut().sections[0].paragraphs[2];
    let Control::Equation(equation) = &mut para.controls[0] else {
        panic!("HML equation owner");
    };
    equation.script.push('\u{1}');
    para.char_offsets.pop();

    let error = core.export_hml_native().unwrap_err();
    assert!(error.blockers().iter().any(|blocker| {
        blocker.code == "HML_INVALID_XML_CHARACTER"
            && blocker.xml_path.ends_with("/EQUATION/SCRIPT")
    }));
    assert!(error.blockers().iter().any(|blocker| {
        blocker.code == "HML_UNSUPPORTED_IR" && blocker.message.contains("offset")
    }));
}

#[test]
fn unknown_equation_save_state_survives_edit_undo_and_redo() {
    let xml = br#"<HWPML Version="2.91"><HEAD/><BODY><SECTION><P><TEXT><EQUATION FutureAttr="1"><SCRIPT>x</SCRIPT><FUTURE Mode="matrix&amp;inline">unknown &lt; value</FUTURE></EQUATION></TEXT></P></SECTION></BODY><TAIL/></HWPML>"#;
    let mut document = HwpDocument::new(xml).expect("HML with unsupported equation properties");
    let save_state =
        |doc: &HwpDocument| -> Value { serde_json::from_str(&doc.get_hml_save_state()).unwrap() };
    let original = save_state(&document);
    assert_eq!(original["sourceFormat"], "hml");
    assert_eq!(original["hmlSavable"], false);
    let blockers = original["blockers"].as_array().unwrap();
    assert_eq!(blockers.len(), 4);
    assert!(blockers.iter().all(|blocker| {
        blocker["code"] == "HML_UNSUPPORTED_EQUATION_SEMANTICS" && blocker["preserved"] == false
    }));
    for (path, value) in [
        ("/EQUATION/@FutureAttr", "FutureAttr=1"),
        ("/EQUATION/FUTURE", "FUTURE"),
        ("/EQUATION/FUTURE/@Mode", "Mode=matrix&inline"),
        ("/EQUATION/FUTURE/#text", "#text=unknown < value"),
    ] {
        assert!(
            blockers.iter().any(|blocker| {
                blocker["xmlPath"].as_str().unwrap().ends_with(path)
                    && blocker["message"].as_str().unwrap().contains(value)
            }),
            "{path}"
        );
    }
    let before = document.save_snapshot_native();
    document
        .set_equation_properties_native(0, 0, 0, None, None, r#"{"script":"x^2 + 2"}"#)
        .unwrap();
    let after = document.save_snapshot_native();
    assert_eq!(save_state(&document), original);
    assert_eq!(scripts(document.document()), [(0, "x^2 + 2")]);
    document.restore_snapshot_native(before).unwrap();
    assert_eq!(save_state(&document), original);
    assert_eq!(scripts(document.document()), [(0, "x")]);
    document.restore_snapshot_native(after).unwrap();
    assert_eq!(save_state(&document), original);
    assert_eq!(scripts(document.document()), [(0, "x^2 + 2")]);
}
