//! 관계형·고정형 회귀. 같은 본문의 HWP5 버전에 따른 문자·서식 보존과 문단 수를 검사한다.
//! 주석 갱신 2026-11-22 19:22 KST. 직전 변경 70b7bf355e66c6d435afeb969295d2625977598d (2026-11-22 18:37:23 KST).
//! Windows 한컴 2020 독립 Print와 96 dpi 2px 실루엣을 비교했다. 두 원본은 각 한 쪽이다.
//! 구형 원본은 Native/fresh WASM 100.00%/100.00%, 현대 원본은 98.46%/98.49%였다.
//! 근거: `mydocs/pr/assets/pr9280/`, `pdf/issue5140-version-boundary/`. 글꼴 설치 없이 문자 관계를 검사한다.

use rhwp::document_core::DocumentCore;
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

#[test]
fn modern_body_characters_and_legacy_rules_survive_reopening_for_extraction() {
    let pairs = [
        ('\u{A807}', '┬'),
        ('\u{A80E}', '┘'),
        ('\u{A810}', '│'),
        ('\u{A832}', '═'),
    ];
    for revision in [5, 6] {
        for format in ["hwp", "hwpx"] {
            let name = format!(
                "samples/issue5140-version-boundary/legacy-symbols-v5.0.1.{revision}.{format}"
            );
            let source = std::fs::read(&name).expect(&name);
            let core = DocumentCore::from_bytes(&source).unwrap();
            for bytes in [
                source,
                core.export_hwp_native().unwrap(),
                core.export_hwpx_native().unwrap(),
            ] {
                let reopened = DocumentCore::from_bytes(&bytes).unwrap();
                let text = reopened.extract_page_text_native(0).unwrap();
                for (raw, rule) in pairs {
                    let expected = if revision == 5 { rule } else { raw };
                    let label = format!("U+{:04X}: A{expected}B", raw as u32);
                    assert!(text.lines().any(|line| line == label), "{name}: {label}");
                }
            }
        }
    }
}

#[test]
fn editing_and_repeated_saves_keep_the_four_neighbor_symbols_and_bold_start() {
    let typed = "\u{A805}\u{A809}\u{A847}\u{A849}😀\t";
    for revision in [5, 6] {
        let name =
            format!("samples/issue5140-version-boundary/legacy-symbols-v5.0.1.{revision}.hwp");
        let mut core = DocumentCore::from_bytes(&std::fs::read(&name).expect(&name)).unwrap();
        core.insert_text_native(0, 1, 0, typed).unwrap();
        let prefix = if revision == 5 {
            "\u{F0805}\u{F0809}\u{F0847}\u{F0849}😀\t"
        } else {
            typed
        };
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let first = DocumentCore::from_bytes(&bytes).unwrap();
            let hwpx = first.export_hwpx_native().unwrap();
            let second = DocumentCore::from_bytes(&hwpx).unwrap();
            let doc = parse_document(&second.export_hwpx_native().unwrap()).unwrap();
            let para = &doc.sections[0].paragraphs[1];
            assert!(para.text.starts_with(prefix), "{name}: {:?}", para.text);
            let expected_symbol = if revision == 5 {
                '\u{F0806}'
            } else {
                '\u{A806}'
            };
            let bold = para
                .char_shapes
                .iter()
                .find(|r| doc.doc_info.char_shapes[r.char_shape_id as usize].bold)
                .unwrap();
            // A normal tab occupies eight HWP character slots. The other controls
            // in this paragraph follow the original Hancom style-run contract.
            let b = para.text.chars().position(|ch| ch == 'B').unwrap();
            assert_eq!(para.text.chars().nth(b - 1), Some(expected_symbol));
            assert_eq!(bold.start_pos, para.char_offsets[b]);
        }
    }
}
