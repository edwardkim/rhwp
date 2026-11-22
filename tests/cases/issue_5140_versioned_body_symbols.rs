//! 관계형·고정형 회귀. HWP5와 저장한 HWPX의 문자·서식·문단 수, 단독 부호의 표시와 후속 문자 위치를 검사한다.
//! 주석 갱신 2026-11-22 21:04 KST. 직전 변경 eb83e873566cf6dd079c4d8be430b1fa386e6af1 (2026-11-22 19:51:19 KST).
//! 2026-11-22 Windows 한컴 2020 독립 Print와 96 dpi 2px 실루엣을 전쪽 비교했다. 각 원본·저장본은 한 쪽이다.
//! 구형·현대 원본의 Native/fresh WASM은 각각 100.00%/100.00%, 99.64%/99.63%다.
//! 단독 부호의 HWP와 HWPX 중 낮은 Native/fresh WASM 값은 기본 99.47%/99.46%, 경계 99.22%/99.19%, 대체 받침 99.17%/98.97%다.
//! 원본별 값과 독립 표시 문자열은 `mydocs/pr/assets/pr9280/`의 `focus-*.tsv`, `syloti-pages.tsv`, `syloti-hancom.tsv`에 있다.

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

// 입력 파일과 기대 표시 문자열은 Windows 한컴 2020의 본문과 독립 Print에서 얻었다.
#[test]
fn isolated_syloti_hasanta_preserves_source_positions_and_paints_printed_clusters() {
    use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp};

    #[derive(Default)]
    struct Line {
        logical: String,
        painted: String,
        following_c: Option<f64>,
    }

    fn collect(node: &LayerNode, lines: &mut [Line]) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    collect(child, lines);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                collect(child, lines);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    let PaintOp::TextRun { bbox, run, .. } = op else {
                        continue;
                    };
                    let Some(line) = run
                        .para_index
                        .filter(|&index| index > 0)
                        .and_then(|index| lines.get_mut(index))
                    else {
                        continue;
                    };
                    assert_eq!(run.char_start, Some(line.logical.chars().count()));
                    line.logical.push_str(&run.text);
                    line.painted.push_str(run.display_or_text());
                    if run.text == "C" {
                        line.following_c = Some(bbox.x);
                    }
                }
            }
        }
    }

    fn scalars(field: &str) -> String {
        field
            .split_whitespace()
            .map(|code| {
                let value = u32::from_str_radix(code.strip_prefix("U+").unwrap(), 16).unwrap();
                char::from_u32(value).unwrap()
            })
            .collect()
    }

    let oracle = include_str!("../../mydocs/pr/assets/pr9280/syloti-hancom.tsv");
    for stem in [
        "syloti-mark-control",
        "syloti-boundaries",
        "syloti-placeholders",
    ] {
        let path = format!("samples/issue5140-version-boundary/{stem}.hwp");
        let input = std::fs::read(&path).unwrap();
        let original = DocumentCore::from_bytes(&input).unwrap();
        let reopened = DocumentCore::from_bytes(&original.export_hwpx_native().unwrap()).unwrap();
        let rows: Vec<_> = oracle
            .lines()
            .skip(1)
            .map(|row| row.split('\t').collect::<Vec<_>>())
            .filter(|row| row[0] == stem)
            .collect();
        assert!(!rows.is_empty());
        for core in [&original, &reopened] {
            let tree = core.build_page_layer_tree(0).unwrap();
            let mut lines: Vec<_> = (0..=rows.len()).map(|_| Line::default()).collect();
            collect(&tree.root, &mut lines);
            for row in &rows {
                let para = row[1].parse::<usize>().unwrap();
                let source = scalars(row[2]);
                assert_eq!(core.document().sections[0].paragraphs[para].text, source);
                assert_eq!(lines[para].logical, source, "{stem} 문단 {para} 원문");
                // 한컴 PDF의 문자표는 폭이 없는 joiner를 공백으로 기록한다.
                let printed = lines[para].painted.replace(['\u{200c}', '\u{200d}'], " ");
                assert_eq!(printed, scalars(row[3]), "{stem} 문단 {para} 표시");
            }
            if stem == "syloti-mark-control" {
                let c = |para: usize| lines[para].following_c.expect("후속 라틴 C");
                assert_eq!(c(1), c(4), "단독 부호 반복은 후속 C를 밀지 않는다");
                assert!(c(5) > c(3), "원문에 입력한 점선 원은 글자 폭을 차지한다");
            }
        }
    }
}
