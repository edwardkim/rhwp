//! #5140 follow-up: an independent Hancom SaveAs pair establishes the 5.0.1.6
//! body-text boundary and the UTF-16 expansion. See the sample README for hashes.

use rhwp::model::document::Document;
use rhwp::parser::parse_document;
use rhwp::serializer::hwpx::{
    roundtrip::{diff_documents, hwpx_export_expected_document, strip_hwp_to_hwpx_noise},
    serialize_hwpx,
};

fn load(name: &str) -> Document {
    let path = format!("samples/issue5140-version-boundary/{name}");
    parse_document(&std::fs::read(&path).expect(&path)).expect("parse sample")
}

fn save(doc: &Document) -> Document {
    parse_document(&serialize_hwpx(doc).expect("serialize HWPX")).expect("reopen HWPX")
}

fn texts(doc: &Document) -> Vec<&str> {
    doc.sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .map(|p| p.text.as_str())
        .collect()
}

fn verify(source: &Document, reloaded: &Document) -> Vec<String> {
    strip_hwp_to_hwpx_noise(diff_documents(
        &hwpx_export_expected_document(source),
        &hwpx_export_expected_document(reloaded),
    ))
    .differences
    .iter()
    .map(ToString::to_string)
    .collect()
}

#[test]
fn legacy_body_text_matches_independent_hancom_save() {
    let source = load("legacy-symbols-v5.0.1.5.hwp");
    let oracle = load("legacy-symbols-v5.0.1.5.hwpx");
    let saved = save(&source);
    assert_eq!(texts(&saved), texts(&oracle));
    assert_eq!(saved.sections[0].paragraphs.len(), 16);
    assert_ne!(
        texts(&source),
        texts(&oracle),
        "oracle must exercise conversion"
    );
}

#[test]
fn modern_hwp_and_both_hwpx_inputs_preserve_all_codepoints() {
    for name in [
        "legacy-symbols-v5.0.1.6.hwp",
        "legacy-symbols-v5.0.1.5.hwpx",
        "legacy-symbols-v5.0.1.6.hwpx",
    ] {
        let source = load(name);
        let saved = save(&source);
        assert_eq!(texts(&source), texts(&saved), "{name}");
    }
}

// The generated HWP inputs intentionally have no stored line cache. Use the
// independently saved modern Hancom cache on the same BMP text, then check its
// projection against the independently saved legacy Hancom output. This tests
// preservation of actual cached starts without inventing a line-breaking rule.
fn with_hancom_line_cache(mut source: Document) -> Document {
    let modern = load("legacy-symbols-v5.0.1.6.hwpx");
    for i in 1..16 {
        assert_eq!(
            source.sections[0].paragraphs[i].text,
            modern.sections[0].paragraphs[i].text
        );
        source.sections[0].paragraphs[i].line_segs =
            modern.sections[0].paragraphs[i].line_segs.clone();
    }
    source
}

#[test]
fn legacy_run_and_line_positions_follow_utf16_expansion() {
    let source = with_hancom_line_cache(load("legacy-symbols-v5.0.1.5.hwp"));
    let oracle = load("legacy-symbols-v5.0.1.5.hwpx");
    let saved = save(&source);
    for i in 1..16 {
        let bold_start = |doc: &Document| {
            doc.sections[0].paragraphs[i]
                .char_shapes
                .iter()
                .find(|r| doc.doc_info.char_shapes[r.char_shape_id as usize].bold)
                .expect("bold run")
                .start_pos
        };
        assert_eq!(bold_start(&source), 10, "source paragraph {i}");
        assert_eq!(bold_start(&oracle), 11, "Hancom paragraph {i}");
        assert_eq!(bold_start(&saved), bold_start(&oracle), "paragraph {i}");
        let second_line = |doc: &Document| doc.sections[0].paragraphs[i].line_segs[1].text_start;
        assert_eq!(second_line(&source), 12);
        assert_eq!(second_line(&oracle), 13);
        assert_eq!(second_line(&saved), second_line(&oracle), "paragraph {i}");
    }
}

#[test]
fn repeated_saves_do_not_accumulate_symbol_or_position_shifts() {
    for name in ["legacy-symbols-v5.0.1.5.hwp", "legacy-symbols-v5.0.1.6.hwp"] {
        let source = load(name);
        let first = save(&source);
        let second = save(&first);
        let third = save(&second);
        assert_eq!(texts(&first), texts(&second), "{name}");
        assert_eq!(texts(&second), texts(&third), "{name}");
        for i in 1..16 {
            let p = |d: &Document| {
                let p = &d.sections[0].paragraphs[i];
                (
                    p.char_shapes
                        .iter()
                        .map(|c| (c.start_pos, c.char_shape_id))
                        .collect::<Vec<_>>(),
                    p.line_segs.iter().map(|l| l.text_start).collect::<Vec<_>>(),
                )
            };
            assert_eq!(p(&first), p(&second), "{name} paragraph {i}");
            assert_eq!(p(&second), p(&third), "{name} paragraph {i}");
        }
    }
}

#[test]
fn verify_accepts_only_the_intended_expansion() {
    let source = with_hancom_line_cache(load("legacy-symbols-v5.0.1.5.hwp"));
    let saved = save(&source);
    assert!(
        verify(&source, &saved).is_empty(),
        "{:?}",
        verify(&source, &saved)
    );

    let mut bad_style = saved.clone();
    bad_style.sections[0].paragraphs[1].char_shapes[1].start_pos += 1;
    assert!(
        !verify(&source, &bad_style).is_empty(),
        "style loss must be reported"
    );
    let mut bad_line = saved.clone();
    bad_line.sections[0].paragraphs[1].line_segs[1].text_start += 1;
    assert!(
        !verify(&source, &bad_line).is_empty(),
        "line drift must be reported"
    );
}

#[test]
fn edits_survive_hwp_and_hwpx_reopen_and_resave() {
    use rhwp::document_core::DocumentCore;
    for name in ["legacy-symbols-v5.0.1.5.hwp", "legacy-symbols-v5.0.1.6.hwp"] {
        let bytes = std::fs::read(format!("samples/issue5140-version-boundary/{name}")).unwrap();
        let mut core = DocumentCore::from_bytes(&bytes).unwrap();
        core.insert_text_native(0, 1, 0, "EDIT ").unwrap();
        for hwpx in [false, true] {
            let export = |c: &DocumentCore| {
                if hwpx {
                    c.export_hwpx_native()
                } else {
                    c.export_hwp_with_adapter()
                }
            };
            let first = DocumentCore::from_bytes(&export(&core).unwrap()).unwrap();
            let second = DocumentCore::from_bytes(&export(&first).unwrap()).unwrap();
            assert!(first.document().sections[0].paragraphs[1]
                .text
                .starts_with("EDIT "));
            assert_eq!(
                texts(first.document()),
                texts(second.document()),
                "{name}, HWPX={hwpx}"
            );
            let cross = DocumentCore::from_bytes(&first.export_hwpx_native().unwrap()).unwrap();
            let again = DocumentCore::from_bytes(&cross.export_hwpx_native().unwrap()).unwrap();
            assert_eq!(
                texts(cross.document()),
                texts(again.document()),
                "cross save {name}"
            );
        }
    }
}
