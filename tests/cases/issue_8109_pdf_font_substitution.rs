//! Source/independent Print qualification: task_m100_8109_stage2.md.
#![cfg(feature = "pdf-import")]

use rhwp::{parser::pdf::PdfImportOptions, DocumentCore};
use std::path::Path;

fn import() -> DocumentCore {
    DocumentCore::from_pdf_bytes(
        &std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue7998/body-list.pdf"))
            .unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap()
}

fn expect_fallback(core: &DocumentCore) {
    let fonts = &core.document().doc_info.font_faces[1];
    let font = fonts.iter().find(|font| font.name == "Helvetica").unwrap();
    assert_eq!(
        font.subst_font.as_ref().map(|font| font.face.as_str()),
        Some("Arial"),
        "the declared source is Helvetica; its explicit fallback is Arial"
    );
    assert_eq!(
        font.subst_font.as_ref().unwrap().font_type,
        1,
        "independent TrueType substitute type"
    );
    assert!(!font.is_embedded, "no source program was supplied");
}

#[test]
fn source_declaration_and_fallback_survive_both_saves() {
    let core = import();
    expect_fallback(&core);
    for bytes in [
        core.export_hwp_with_adapter_snapshot().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&bytes).unwrap();
        expect_fallback(&reopened);
        let cross = DocumentCore::from_bytes(&reopened.export_hwpx_native().unwrap()).unwrap();
        expect_fallback(&cross);
    }
}

#[test]
fn standard_hwp_alternative_is_consumed_and_preserved() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let core = DocumentCore::from_bytes(
        &std::fs::read(root.join("pdf/issue8109/standard-substitute.hwpx")).unwrap(),
    )
    .unwrap();
    let hwp = DocumentCore::from_bytes(&core.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
    expect_fallback(&hwp);
    let hwpx = DocumentCore::from_bytes(&hwp.export_hwpx_native().unwrap()).unwrap();
    expect_fallback(&hwpx);
}

#[test]
fn program_presence_and_resource_scope_gate_the_policy() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue8109");
    for (name, expected) in [
        ("absent", true),
        ("inline-absent", true),
        ("indirect-absent", true),
        ("embedded", false),
        ("unreadable-program", false),
        ("invalid-descriptor", false),
        ("arial", false),
        ("non-type1", false),
    ] {
        let core = DocumentCore::from_pdf_bytes(
            &std::fs::read(root.join(format!("{name}.pdf"))).unwrap(),
            &PdfImportOptions::default(),
        )
        .unwrap();
        let fonts = &core.document().doc_info.font_faces[1];
        let declared = if name == "arial" {
            "Arial"
        } else {
            "Helvetica"
        };
        let font = fonts.iter().find(|f| f.name == declared).unwrap();
        assert_eq!(
            font.subst_font.as_ref().is_some_and(|s| s.face == "Arial"),
            expected,
            "source resource {name}"
        );
    }
    let core = DocumentCore::from_pdf_bytes(
        &std::fs::read(root.join("form-scopes.pdf")).unwrap(),
        &PdfImportOptions::default(),
    )
    .unwrap();
    let fonts: Vec<_> = core.document().doc_info.font_faces[1]
        .iter()
        .filter(|f| f.name == "Helvetica")
        .collect();
    assert_eq!(
        fonts.len(),
        2,
        "same F1/family in distinct Forms keeps two source identities"
    );
    assert_eq!(
        fonts.iter().filter(|f| f.subst_font.is_some()).count(),
        1,
        "only the absent program gets an explicit substitute"
    );
}

#[test]
fn unresolvable_font_resource_preserves_the_existing_bounded_rejection() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue8109/unresolved-descriptor.pdf");
    let result =
        DocumentCore::from_pdf_bytes(&std::fs::read(path).unwrap(), &PdfImportOptions::default());
    let error = result
        .err()
        .expect("unbounded PDF object recovery stays disabled");
    assert!(
        error.to_string().contains("PDF_RESOURCE_LIMIT"),
        "unexpected rejection: {error}"
    );
}

fn painted(core: &DocumentCore) -> Vec<(String, String, bool, bool)> {
    fn walk(
        node: &rhwp::renderer::render_tree::RenderNode,
        out: &mut Vec<(String, String, bool, bool)>,
    ) {
        if let rhwp::renderer::render_tree::RenderNodeType::TextRun(run) = &node.node_type {
            if let Some(last) = out.last_mut().filter(|last| {
                last.1 == run.style.font_family
                    && last.2 == run.style.bold
                    && last.3 == run.style.italic
            }) {
                last.0.push_str(&run.text);
            } else {
                out.push((
                    run.text.clone(),
                    run.style.font_family.clone(),
                    run.style.bold,
                    run.style.italic,
                ));
            }
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(&core.build_page_render_tree(0).unwrap().root, &mut out);
    out
}

#[test]
fn scoped_resources_reach_the_correct_final_paint_owner() {
    let input =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue8109/form-scopes.pdf"))
            .unwrap();
    let core = DocumentCore::from_pdf_bytes(&input, &PdfImportOptions::default()).unwrap();
    let hwp = DocumentCore::from_bytes(&core.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    for core in [&core, &hwp, &hwpx] {
        let runs = painted(core);
        let text: String = runs.iter().map(|r| r.0.as_str()).collect();
        assert_eq!(
            text, "UNEMBEDDEDEMBEDDED",
            "both Form contents reach paint once in stream order"
        );
        assert_eq!(
            runs.len(),
            2,
            "same-name resources produce separate paint font chains"
        );
        assert_eq!(
            runs.iter().find(|r| r.0 == "UNEMBEDDED").unwrap().1,
            "Helvetica,Arial"
        );
        assert_eq!(
            runs.iter().find(|r| r.0 == "EMBEDDED").unwrap().1,
            "Helvetica",
            "a same-name embedded program must not inherit the other Form's fallback"
        );
    }
}

#[test]
fn four_helvetica_styles_keep_their_style_and_explicit_substitute() {
    let input =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue8109/styles.pdf"))
            .unwrap();
    let core = DocumentCore::from_pdf_bytes(&input, &PdfImportOptions::default()).unwrap();
    let hwp = DocumentCore::from_bytes(&core.export_hwp_with_adapter_snapshot().unwrap()).unwrap();
    let hwpx = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    for core in [&core, &hwp, &hwpx] {
        let runs = painted(core);
        for (index, face, bold, italic) in [
            (1, "Helvetica", false, false),
            (2, "Helvetica-Bold", true, false),
            (3, "Helvetica-Oblique", false, true),
            (4, "Helvetica-BoldOblique", true, true),
        ] {
            let text = format!("STYLE {index} ALPHA");
            let matching: Vec<_> = runs.iter().filter(|r| r.0 == text).collect();
            assert_eq!(
                matching.len(),
                1,
                "each styled source row reaches final paint once"
            );
            let run = matching[0];
            assert_eq!(run.1, format!("{face},Arial"));
            assert_eq!((run.2, run.3), (bold, italic));
        }
    }
}
