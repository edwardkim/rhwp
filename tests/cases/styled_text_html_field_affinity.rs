//! 다른 문서의 순수 서식 HTML은 입력의 필드 소속을 따르고 내부 클립보드를 소비하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use serde_json::{json, Value};

const COPY: &str = "가🦦";
const PREFIX: &str = "앞🦦";
const SUFFIX: &str = "나다";
const SENTINEL: &str = "기존🦦버퍼";

#[derive(Clone, Copy, Debug)]
enum Note {
    Foot,
    End,
}

fn ok(text: &str) -> Value {
    let value: Value = serde_json::from_str(text).unwrap();
    assert_eq!(value["ok"], true, "{value}");
    value
}
fn blank() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core
}
fn saved(core: &DocumentCore) -> [DocumentCore; 2] {
    [
        DocumentCore::from_bytes(&core.export_hwp_with_adapter().unwrap()).unwrap(),
        DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap(),
    ]
}
fn para(core: &DocumentCore, index: usize) -> &Paragraph {
    &core.document().sections[0].paragraphs[index]
}
fn note(core: &DocumentCore, index: usize) -> (usize, usize, Value) {
    let paragraph = para(core, index);
    let (control, payload) = paragraph
        .controls
        .iter()
        .enumerate()
        .find(|(_, control)| matches!(control, Control::Footnote(_) | Control::Endnote(_)))
        .unwrap();
    (
        control,
        paragraph.control_text_positions()[control],
        json!(payload),
    )
}
fn full_styles(core: &DocumentCore, index: usize) -> Vec<Value> {
    let paragraph = para(core, index);
    (0..paragraph.text.chars().count())
        .map(|at| {
            let id = paragraph.char_shape_id_at(at).unwrap() as usize;
            let info = &core.document().doc_info;
            let shape = &info.char_shapes[id];
            let fonts: Vec<Value> = shape
                .font_ids
                .iter()
                .enumerate()
                .map(|(language, font)| {
                    json!(info
                        .font_faces
                        .get(language)
                        .and_then(|faces| faces.get(*font as usize)))
                })
                .collect();
            json!({"shape":shape,"fonts":fonts})
        })
        .collect()
}
fn content(core: &DocumentCore, index: usize) -> Value {
    let p = para(core, index);
    json!({"text":p.text,"offsets":p.char_offsets,"count":p.char_count,"mask":p.control_mask,
        "para_shape":p.para_shape_id,"style":p.style_id,"controls":p.controls,
        "records":p.ctrl_data_records,"ranges":p.field_ranges,"orphan":p.orphan_field_ends,
        "tags":p.range_tags,"tabs":p.tab_extended,"title":p.title_marks,"markpen":p.markpen_marks,
        "styles":full_styles(core,index)})
}
fn fields(core: &DocumentCore) -> Value {
    serde_json::from_str(&core.get_field_list_json()).unwrap()
}
fn insert_note(core: &mut DocumentCore, paragraph: usize, at: usize, kind: Note, text: &str) {
    let result = match kind {
        Note::Foot => core.insert_footnote_native(0, paragraph, at),
        Note::End => core.insert_endnote_native(0, paragraph, at),
    }
    .unwrap();
    let control = ok(&result)["controlIdx"].as_u64().unwrap() as usize;
    core.insert_text_in_footnote_native(0, paragraph, control, 0, 2, text)
        .unwrap();
}

fn fixture(kind: Note, before: bool, empty: bool) -> DocumentCore {
    let mut core = blank();
    core.insert_text_native(0, 0, 0, SENTINEL).unwrap();
    core.apply_char_format_native(0, 0, 0, 1, r##"{"bold":true,"textColor":"#669933"}"##)
        .unwrap();
    core.apply_char_format_native(0, 0, 1, 5, r##"{"italic":true,"textColor":"#996699"}"##)
        .unwrap();
    core.insert_paragraph_native(0, 1).unwrap();
    core.insert_text_native(0, 1, 0, &format!("{PREFIX}{SUFFIX}"))
        .unwrap();
    core.insert_click_here_field_at(0, 1, 2, "대상 안내", "메모 <&> 🦦", "target", true)
        .unwrap();
    if !empty {
        core.set_field_value_by_name("target", "ABCD").unwrap();
    }
    let end = if empty { 2 } else { 6 };
    core.apply_char_format_native(
        0,
        1,
        0,
        2,
        r##"{"bold":false,"italic":false,"textColor":"#111111"}"##,
    )
    .unwrap();
    if !empty {
        core.apply_char_format_native(0, 1, 2, end, r##"{"bold":true,"textColor":"#00aa00"}"##)
            .unwrap();
    }
    core.apply_char_format_native(
        0,
        1,
        end,
        end + 2,
        r##"{"italic":true,"textColor":"#aa00aa"}"##,
    )
    .unwrap();
    insert_note(
        &mut core,
        1,
        if before { 1 } else { end + 1 },
        kind,
        "대상 주석🦦",
    );
    core.insert_paragraph_native(0, 2).unwrap();
    core.insert_text_native(0, 2, 0, "이웃\t끝🦦").unwrap();
    core
}
fn assert_source(core: &DocumentCore, kind: Note, before: bool, empty: bool) {
    assert_eq!(para(core, 0).text, SENTINEL);
    assert_eq!(
        para(core, 1).text,
        format!("{PREFIX}{}{SUFFIX}", if empty { "" } else { "ABCD" })
    );
    assert_eq!(para(core, 2).text, "이웃\t끝🦦");
    // 가져오는 두 색은 대상 서식표에 없으므로 기존 ID만 재사용해 통과할 수 없다.
    assert!(core
        .document()
        .doc_info
        .char_shapes
        .iter()
        .all(|shape| ![0x000000cc, 0x00993300].contains(&shape.text_color)));
    let f = fields(core);
    assert_eq!(f.as_array().unwrap().len(), 1);
    assert_eq!(f[0]["name"], "target");
    assert_eq!(f[0]["guide"], "대상 안내");
    assert_eq!(f[0]["value"], if empty { "" } else { "ABCD" });
    assert_eq!(f[0]["startCharIdx"], 2);
    assert_eq!(f[0]["endCharIdx"], if empty { 2 } else { 6 });
    assert_eq!(f[0]["editableInForm"], true);
    assert_eq!(f[0]["location"], json!({"sectionIndex":0,"paraIndex":1}));
    // 조회 축과 실제 UTF-16 스트림 축을 원문부터 따로 확정한다.
    let start = if before { 18 } else { 10 };
    assert_eq!(f[0]["startPos"], start);
    assert_eq!(f[0]["endPos"], start + if empty { 5 } else { 4 });
    assert_eq!(
        para(core, 1).char_offsets[2],
        if before { 11 } else { 3 } + if empty { 16 } else { 8 }
    );
    let (control, at, _) = note(core, 1);
    assert_eq!(
        at,
        if before {
            1
        } else if empty {
            3
        } else {
            7
        }
    );
    assert!(matches!(
        (&para(core, 1).controls[control], kind),
        (Control::Footnote(_), Note::Foot) | (Control::Endnote(_), Note::End)
    ));
    let info = ok(&core.get_footnote_info_native(0, 1, control).unwrap());
    assert_eq!(info["texts"], json!(["  대상 주석🦦"]));
    assert_eq!(info["number"], 1);
    assert_eq!(para(core, 1).field_ranges.len(), 1);
    assert_eq!(para(core, 1).field_ranges[0].inner_slot_count, 0);
}
fn binaries(core: &DocumentCore) -> Vec<(u16, String, Vec<u8>)> {
    // Lazy의 Debug 표현 대신 실제 전체 바이트와 BinDataContent의 모든 필드를 비교한다.
    core.document()
        .bin_data_content
        .iter()
        .map(|binary| (binary.id, binary.extension.clone(), binary.data.load()))
        .collect()
}
fn unchanged(core: &DocumentCore) -> Value {
    json!({"document":format!("{:?}", core.document()), "binary":binaries(core),
        "events":core.serialize_event_log(), "clipboard":core.get_clipboard_text_native(),
        "query":core.clipboard_is_single_text_native()})
}
fn source_html(kind: Note, format: usize, own: bool) -> String {
    let mut original = blank();
    original.insert_text_native(0, 0, 0, "가🦦XX끝").unwrap();
    original
        .insert_click_here_field_at(0, 0, 4, "원문 안내", "원문 메모", "source", true)
        .unwrap();
    original.set_field_value_by_name("source", "ABCD").unwrap();
    original
        .apply_char_format_native(
            0,
            0,
            0,
            1,
            r##"{"bold":true,"italic":false,"textColor":"#cc0000"}"##,
        )
        .unwrap();
    original
        .apply_char_format_native(
            0,
            0,
            1,
            2,
            r##"{"bold":false,"italic":true,"textColor":"#003399"}"##,
        )
        .unwrap();
    insert_note(&mut original, 0, 3, kind, "원문 주석🦦");
    let mut source = if format == 0 {
        DocumentCore::from_bytes(&original.export_hwp_with_adapter().unwrap()).unwrap()
    } else {
        DocumentCore::from_bytes(&original.export_hwpx_native().unwrap()).unwrap()
    };
    assert_eq!(para(&source, 0).text, "가🦦XXABCD끝");
    assert_eq!(fields(&source)[0]["startCharIdx"], 4);
    assert_eq!(fields(&source)[0]["endCharIdx"], 8);
    assert_eq!(note(&source, 0).1, 3);
    assert_eq!(
        ok(&source.copy_selection_logical_native(0, 0, 0, 0, 2).unwrap())["text"],
        COPY
    );
    assert!(
        source.clipboard_is_single_text_native(),
        "선택 밖 원문 필드·주석은 조각에 없다"
    );
    let before = unchanged(&source);
    let bytes = [
        source.export_hwp_with_adapter().unwrap(),
        source.export_hwpx_native().unwrap(),
    ];
    let html = source.export_selection_html_native(0, 0, 0, 0, 2).unwrap();
    assert!(html.contains("font-weight:bold;") && html.contains("font-style:italic;"));
    assert!(html.contains("#cc0000") && html.contains("#003399"));
    assert!(!html.contains("ABCD") && !html.contains("원문 주석"));
    assert_eq!(
        unchanged(&source),
        before,
        "내보내기는 원문·클립보드를 바꾸지 않는다"
    );
    assert_eq!(
        [
            source.export_hwp_with_adapter().unwrap(),
            source.export_hwpx_native().unwrap()
        ],
        bytes
    );
    if own {
        html.replacen(
            "<!--StartFragment-->",
            "<!--StartFragment-->\n<!--clipboard-source-->",
            1,
        )
    } else {
        html
    }
}
fn seed_clipboard(core: &mut DocumentCore) {
    assert_eq!(
        ok(&core.copy_selection_logical_native(0, 0, 0, 0, 5).unwrap())["text"],
        SENTINEL
    );
    assert!(core.clipboard_is_single_text_native());
}
fn assert_clipboard(core: &mut DocumentCore, before: &DocumentCore) {
    assert_eq!(core.get_clipboard_text_native(), SENTINEL);
    assert!(core.clipboard_is_single_text_native());
    assert_eq!(
        ok(&core.paste_internal_text_logical_native(0, 2, 0).unwrap())["charOffset"],
        5
    );
    assert_eq!(para(core, 2).text, format!("{SENTINEL}이웃\t끝🦦"));
    assert_eq!(
        &full_styles(core, 2)[..5],
        full_styles(before, 0).as_slice(),
        "기존 내부 클립보드의 전체 글자 모양도 HTML 조각으로 교체되지 않는다"
    );
    assert_eq!(core.get_clipboard_text_native(), SENTINEL);
    assert_eq!(content(core, 0), content(before, 0));
}
struct Expected {
    from: usize,
    deleted: usize,
    value: String,
    start: usize,
    end: usize,
    start_query_delta: i64,
    end_query_delta: i64,
    note_at: usize,
    activate_field: bool,
}
fn assert_edit(
    core: &DocumentCore,
    before: &DocumentCore,
    expected: &Expected,
    imported: &[Value],
) {
    let old = para(before, 1);
    let text: String = old
        .text
        .chars()
        .take(expected.from)
        .chain(COPY.chars())
        .chain(old.text.chars().skip(expected.from + expected.deleted))
        .collect();
    let p = para(core, 1);
    assert_eq!(p.text, text);
    assert_eq!(core.document().sections[0].paragraphs.len(), 3);
    assert_eq!(
        content(core, 0),
        content(before, 0),
        "선택 밖 클립보드 원문·전체 글자 모양"
    );
    assert_eq!(content(core, 2), content(before, 2), "선택 밖 이웃 문단");
    let old_info = &before.document().doc_info;
    let info = &core.document().doc_info;
    assert_eq!(
        json!(&info.char_shapes[..old_info.char_shapes.len()]),
        json!(old_info.char_shapes),
        "기존 전체 글자 모양 표는 append 외에 바뀌지 않는다"
    );
    assert_eq!(info.font_faces.len(), old_info.font_faces.len());
    for (current, old) in info.font_faces.iter().zip(&old_info.font_faces) {
        assert_eq!(
            json!(&current[..old.len()]),
            json!(old),
            "기존 글꼴 슬롯의 실제 모델"
        );
    }
    assert_eq!(
        json!(&info.para_shapes[..old_info.para_shapes.len()]),
        json!(old_info.para_shapes),
        "기존 전체 문단 모양 표"
    );
    assert_eq!(binaries(core), binaries(before));
    let old_styles = full_styles(before, 1);
    let expected_styles: Vec<Value> = old_styles[..expected.from]
        .iter()
        .cloned()
        .chain(imported.iter().cloned())
        .chain(
            old_styles[expected.from + expected.deleted..]
                .iter()
                .cloned(),
        )
        .collect();
    assert_eq!(
        full_styles(core, 1),
        expected_styles,
        "삽입·선택 밖 모든 scalar의 전체 CharShape"
    );
    assert_eq!(
        (p.para_shape_id, p.style_id, p.control_mask),
        (old.para_shape_id, old.style_id, old.control_mask)
    );
    assert_eq!(
        json!(core.document().doc_info.para_shapes[p.para_shape_id as usize]),
        json!(before.document().doc_info.para_shapes[old.para_shape_id as usize]),
        "대상 문단 모양"
    );
    for (at, color, bold, italic) in [
        (expected.from, "#cc0000", true, false),
        (expected.from + 1, "#003399", false, true),
    ] {
        let props: Value =
            serde_json::from_str(&core.get_char_properties_at_native(0, 1, at).unwrap()).unwrap();
        assert_eq!(props["textColor"], color);
        assert_eq!(props["bold"], bold);
        assert_eq!(props["italic"], italic);
    }
    let range = &p.field_ranges[0];
    let old_range = &old.field_ranges[0];
    assert_eq!(p.field_ranges.len(), 1);
    assert_eq!(
        (range.start_char_idx, range.end_char_idx),
        (expected.start, expected.end)
    );
    assert_eq!(
        json!(p.controls[range.control_idx]),
        json!(old.controls[old_range.control_idx]),
        "필드 ID·command·모든 속성"
    );
    assert_eq!(
        (
            range.control_idx,
            range.end_field_id,
            range.inner_slot_count
        ),
        (
            old_range.control_idx,
            old_range.end_field_id,
            old_range.inner_slot_count
        )
    );
    assert_eq!(note(core, 1).1, expected.note_at);
    assert_eq!(
        note(core, 1).2,
        note(before, 1).2,
        "주석 본문·번호·전체 메타데이터"
    );
    assert_eq!(p.controls.len(), old.controls.len());
    assert_eq!(
        p.ctrl_data_records, old.ctrl_data_records,
        "필드·주석의 원문 CTRL_DATA 바이트"
    );
    let mut expected_fields = fields(before);
    expected_fields[0]["value"] = json!(expected.value);
    expected_fields[0]["startCharIdx"] = json!(expected.start);
    expected_fields[0]["endCharIdx"] = json!(expected.end);
    for (key, delta) in [
        ("startPos", expected.start_query_delta),
        ("endPos", expected.end_query_delta),
    ] {
        expected_fields[0][key] = json!(expected_fields[0][key].as_i64().unwrap() + delta);
    }
    assert_eq!(
        fields(core),
        expected_fields,
        "필드 조회의 소속·기존 커서 축·전체 속성"
    );
    // scalar 글과 BEGIN/END·주석의 8유닛 슬롯을 각각 독립적으로 센다.
    let mut raw = 0_u32;
    let mut offsets = Vec::new();
    for (index, ch) in p.text.chars().enumerate() {
        raw += [expected.start, expected.end, expected.note_at]
            .iter()
            .filter(|&&at| at == index)
            .count() as u32
            * 8;
        offsets.push(raw);
        raw += ch.len_utf16() as u32;
    }
    raw += [expected.start, expected.end, expected.note_at]
        .iter()
        .filter(|&&at| at == p.text.chars().count())
        .count() as u32
        * 8;
    assert_eq!(p.char_offsets, offsets, "surrogate와 필드 종료·주석 슬롯");
    assert_eq!(p.char_count, raw + 1);
}

fn each_source(
    empty: bool,
    own: bool,
    mut edit: impl FnMut(&mut DocumentCore, bool, &str) -> Expected,
) {
    for kind in [Note::Foot, Note::End] {
        for before in [true, false] {
            let original = fixture(kind, before, empty);
            assert_source(&original, kind, before, empty);
            let sources = [
                original.export_hwp_with_adapter().unwrap(),
                original.export_hwpx_native().unwrap(),
            ];
            for (format, bytes) in sources.iter().enumerate() {
                let html = source_html(kind, format, own);
                let baseline = DocumentCore::from_bytes(bytes).unwrap();
                assert_source(&baseline, kind, before, empty);
                let mut edited = DocumentCore::from_bytes(bytes).unwrap();
                assert_source(&edited, kind, before, empty);
                seed_clipboard(&mut edited);
                // 필드가 없는 독립 문단의 기존 HTML 경로로 가져온 전체 CharShape를 대조한다.
                let mut imported = DocumentCore::from_bytes(bytes).unwrap();
                assert_eq!(
                    ok(&imported.paste_html_native(0, 2, 0, &html).unwrap())["charOffset"],
                    2
                );
                let before_saved = saved(&baseline);
                let expected = edit(&mut edited, before, &html);
                let mut typed = DocumentCore::from_bytes(bytes).unwrap();
                typed.clear_active_field();
                if expected.deleted > 0 {
                    typed
                        .delete_text_native(0, 1, expected.from, expected.deleted)
                        .unwrap();
                }
                if expected.activate_field {
                    assert!(typed.set_active_field(0, 1, expected.from));
                }
                // 서식 가져오기는 별도 대조하고, 필드 소속·조회 축은 기존 입력과 직접 비교한다.
                typed.insert_text_native(0, 1, expected.from, COPY).unwrap();
                assert_eq!(fields(&edited), fields(&typed));
                assert_edit(
                    &edited,
                    &baseline,
                    &expected,
                    &full_styles(&imported, 2)[..2],
                );
                assert_eq!(edited.get_clipboard_text_native(), SENTINEL);
                assert!(edited.clipboard_is_single_text_native());
                for (((before_format, after_format), typed_format), imported_format) in before_saved
                    .iter()
                    .zip(saved(&edited).iter())
                    .zip(saved(&typed).iter())
                    .zip(saved(&imported).iter())
                {
                    assert_eq!(
                        fields(after_format),
                        fields(typed_format),
                        "양 저장 형식의 입력 대조군"
                    );
                    assert_edit(
                        after_format,
                        before_format,
                        &expected,
                        &full_styles(imported_format, 2)[..2],
                    );
                }
                assert_clipboard(&mut edited, &baseline);
            }
        }
    }
}
fn logical(scalar: usize, note_at: usize) -> usize {
    scalar + usize::from(note_at < scalar)
}

#[test]
fn foreign_html_active_and_inactive_field_boundaries_follow_typing() {
    for own in [false, true] {
        for active in [false, true] {
            for from in [2, 3, 6] {
                each_source(false, own, |core, before, html| {
                    core.clear_active_field();
                    if active {
                        assert!(core.set_active_field(0, 1, from));
                    }
                    assert_eq!(
                        ok(&core
                            .paste_html_text_logical_native(
                                0,
                                1,
                                logical(from, if before { 1 } else { 7 }),
                                html
                            )
                            .unwrap())["charOffset"],
                        from + 2,
                        "HTML API도 실제 끝 scalar를 반환한다"
                    );
                    let outside_start = from == 2 && !active;
                    let inside = from == 3 || active;
                    Expected {
                        from,
                        deleted: 0,
                        value: if inside {
                            match from {
                                2 => format!("{COPY}ABCD"),
                                3 => format!("A{COPY}BCD"),
                                6 => format!("ABCD{COPY}"),
                                _ => unreachable!(),
                            }
                        } else {
                            "ABCD".into()
                        },
                        start: 2 + if outside_start { 2 } else { 0 },
                        end: 6 + if outside_start || inside { 2 } else { 0 },
                        start_query_delta: if outside_start { 2 } else { 0 },
                        end_query_delta: if outside_start || inside { 2 } else { 0 },
                        note_at: if before { 1 } else { 9 },
                        activate_field: active,
                    }
                });
            }
        }
    }
}
#[test]
fn foreign_html_suffix_replacement_reactivates_new_field_end() {
    each_source(false, true, |core, before, html| {
        core.clear_active_field();
        core.delete_text_native(0, 1, 3, 3).unwrap();
        assert!(core.set_active_field(0, 1, 3));
        assert_eq!(
            ok(&core
                .paste_html_text_logical_native(0, 1, logical(3, if before { 1 } else { 4 }), html)
                .unwrap())["charOffset"],
            5
        );
        Expected {
            from: 3,
            deleted: 3,
            value: format!("A{COPY}"),
            start: 2,
            end: 5,
            start_query_delta: 0,
            end_query_delta: -1,
            note_at: if before { 1 } else { 6 },
            activate_field: true,
        }
    });
}
#[test]
fn foreign_html_first_input_populates_empty_guide_with_or_without_activation() {
    for active in [false, true] {
        each_source(true, false, |core, before, html| {
            core.clear_active_field();
            if active {
                assert!(core.set_active_field(0, 1, 2));
            }
            assert_eq!(
                ok(&core
                    .paste_html_text_logical_native(
                        0,
                        1,
                        logical(2, if before { 1 } else { 3 }),
                        html
                    )
                    .unwrap())["charOffset"],
                4
            );
            Expected {
                from: 2,
                deleted: 0,
                value: COPY.into(),
                start: 2,
                end: 4,
                start_query_delta: 0,
                end_query_delta: -3,
                note_at: if before { 1 } else { 5 },
                activate_field: active,
            }
        });
    }
}
#[test]
fn foreign_html_keeps_note_ownership_on_both_sides_of_logical_slot() {
    for after in [false, true] {
        each_source(false, false, |core, before, html| {
            core.clear_active_field();
            let from = if before { 1 } else { 7 };
            assert_eq!(
                ok(&core
                    .paste_html_text_logical_native(0, 1, from + usize::from(after), html)
                    .unwrap())["charOffset"],
                from + 2
            );
            Expected {
                from,
                deleted: 0,
                value: "ABCD".into(),
                start: if before { 4 } else { 2 },
                end: if before { 8 } else { 6 },
                start_query_delta: if before { 2 } else { 0 },
                end_query_delta: if before { 2 } else { 0 },
                note_at: from + if after { 0 } else { 2 },
                activate_field: false,
            }
        });
    }
}
#[test]
fn foreign_html_plain_target_imports_styles_without_replacing_internal_clipboard() {
    for own in [false, true] {
        let original = fixture(Note::Foot, true, false);
        for mut core in saved(&original) {
            // 일반 div 포장은 한 문단·안쪽 span 서식을 그대로 허용한다.
            let html = source_html(Note::End, 1, own)
                .replacen("<!--StartFragment-->", "<!--StartFragment--><div>", 1)
                .replacen("<!--EndFragment-->", "</div><!--EndFragment-->", 1);
            let before = content(&core, 0);
            let untouched_field = content(&core, 1);
            let old_styles = full_styles(&core, 2);
            seed_clipboard(&mut core);
            let result = ok(&core.paste_html_text_logical_native(0, 2, 2, &html).unwrap());
            assert_eq!(result["charOffset"], 4);
            assert_eq!(para(&core, 2).text, "이웃가🦦\t끝🦦");
            let new_styles = full_styles(&core, 2);
            assert_eq!(&new_styles[..2], &old_styles[..2]);
            assert_eq!(&new_styles[4..], &old_styles[2..]);
            assert!(para(&core, 2).controls.is_empty());
            assert_eq!(content(&core, 0), before);
            assert_eq!(content(&core, 1), untouched_field);
            assert_eq!(core.get_clipboard_text_native(), SENTINEL);
            assert!(core.clipboard_is_single_text_native());
            for (at, color, bold, italic) in
                [(2, "#cc0000", true, false), (3, "#003399", false, true)]
            {
                let props: Value =
                    serde_json::from_str(&core.get_char_properties_at_native(0, 2, at).unwrap())
                        .unwrap();
                assert_eq!(props["textColor"], color);
                assert_eq!(props["bold"], bold);
                assert_eq!(props["italic"], italic);
            }
        }
    }
}
fn rejected(core: &mut DocumentCore, html: &str) {
    let before = unchanged(core);
    let bytes = [
        core.export_hwp_with_adapter().unwrap(),
        core.export_hwpx_native().unwrap(),
    ];
    let result: Value =
        serde_json::from_str(&core.paste_html_text_logical_native(0, 1, 3, html).unwrap()).unwrap();
    assert_eq!(
        result["ok"], false,
        "미지원 HTML을 평문으로 바꾸어 붙이지 않는다"
    );
    assert_eq!(
        unchanged(core),
        before,
        "문서·서식표·그림 데이터·이벤트·기존 클립보드의 원자성"
    );
    assert_eq!(
        [
            core.export_hwp_with_adapter().unwrap(),
            core.export_hwpx_native().unwrap()
        ],
        bytes
    );
}
#[test]
fn foreign_html_rejects_unsupported_original_markup_without_fallback_or_leaks() {
    let cases = [
        "<p>가<img src=\"https://example.invalid/image.png\">🦦</p>".into(),
        "<p>가<img src=\"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jAtcAAAAASUVORK5CYII=\">🦦</p>".into(),
        "<p>가🦦</p><table></table>".into(),
        "<p><a href=\"https://example.invalid/\">가🦦</a></p>".into(),
        "<section><p>가🦦</p></section>".into(),
        "<p>가<script>🦦</script></p>".into(),
        "<p><b>가🦦</p></b>".into(),
        "<p><span>가🦦</p>".into(),
        "<span><p>가</p><p>🦦</p></span>".into(),
        "<p>가<p>🦦</p></p>".into(),
        "<p><div>가🦦</div></p>".into(),
        format!("<p>{}가🦦{}</p>", "<span>".repeat(40), "</span>".repeat(40)),
        format!("<p>{}</p>", "x".repeat(2_000_001)),
        format!("<p>data:{}</p>", "x".repeat(2_000_001)),
        "<p>가🦦</p><!--[data-hwpjson]{\"blocks\":[]}[end-data-hwpjson]-->".into(),
        "<p>가🦦</p><table></table>".into(),
    ];
    let original = fixture(Note::End, true, false);
    for (mut core, baseline) in saved(&original).into_iter().zip(saved(&original)) {
        seed_clipboard(&mut core);
        assert!(core.set_active_field(0, 1, 2));
        for html in &cases {
            rejected(&mut core, html);
        }
        assert_clipboard(&mut core, &baseline);
    }
}
#[test]
fn foreign_html_rolls_back_parser_styles_for_empty_or_multiple_paragraphs() {
    let cases = [
        "<p style=\"margin-left:23pt;line-height:167%\"><span style=\"font-family:'Foreign Empty';font-size:27pt;color:#123456\"></span></p>",
        "<p><span style=\"font-family:'Foreign One';font-size:27pt;color:#123456\">가</span></p><p><span style=\"font-family:'Foreign Two';font-size:31pt;color:#abcdef\">🦦</span></p>",
        "<p></p>",
    ];
    for mut core in saved(&fixture(Note::Foot, false, false)) {
        seed_clipboard(&mut core);
        assert!(core.set_active_field(0, 1, 2));
        for html in cases {
            rejected(&mut core, html);
        }
        // 실패가 활성 필드를 끄지 않았는지 첫 경계 입력으로 확인한다.
        let html = source_html(Note::Foot, 0, true);
        assert_eq!(
            ok(&core.paste_html_text_logical_native(0, 1, 2, &html).unwrap())["charOffset"],
            4
        );
        assert_eq!(fields(&core)[0]["value"], format!("{COPY}ABCD"));
        assert_eq!(core.get_clipboard_text_native(), SENTINEL);
    }
}
#[test]
fn foreign_html_invalid_positions_leave_document_and_clipboard_unchanged() {
    let original = fixture(Note::Foot, true, false);
    for (mut core, baseline) in saved(&original).into_iter().zip(saved(&original)) {
        seed_clipboard(&mut core);
        let html = source_html(Note::Foot, 0, true);
        let before = unchanged(&core);
        let bytes = [
            core.export_hwp_with_adapter().unwrap(),
            core.export_hwpx_native().unwrap(),
        ];
        for (section, paragraph, offset) in [(9, 1, 0), (0, 99, 0), (0, 1, 100)] {
            assert!(core
                .paste_html_text_logical_native(section, paragraph, offset, &html)
                .is_err());
            assert_eq!(unchanged(&core), before);
            assert_eq!(
                [
                    core.export_hwp_with_adapter().unwrap(),
                    core.export_hwpx_native().unwrap()
                ],
                bytes
            );
        }
        assert_clipboard(&mut core, &baseline);
    }
}

#[test]
fn foreign_html_preserves_floating_picture_clipboard_and_its_primed_cascade() {
    const PNG: &[u8] = include_bytes!("../../assets/logo/logo-16.png");
    let mut original = fixture(Note::End, true, false);
    ok(&original
        .insert_picture_native(
            0,
            0,
            0,
            &[],
            PNG,
            1001,
            1303,
            16,
            16,
            "png",
            "기존 그림 버퍼",
            Some(101),
            Some(203),
        )
        .unwrap());
    let html = source_html(Note::Foot, 1, true);
    for accepted in [false, true] {
        for mut core in saved(&original) {
            let source_control = para(&core, 0)
                .controls
                .iter()
                .position(|control| matches!(control, Control::Picture(_)))
                .unwrap();
            let source_control_model = json!(para(&core, 0).controls[source_control]);
            let source = match &para(&core, 0).controls[source_control] {
                Control::Picture(picture) => picture.as_ref().clone(),
                _ => unreachable!(),
            };
            assert!(!source.common.treat_as_char);
            assert_eq!(
                (
                    source.common.horizontal_offset,
                    source.common.vertical_offset
                ),
                (101, 203)
            );
            assert_eq!((source.common.width, source.common.height), (1001, 1303));
            core.copy_control_native(0, 0, &[], source_control).unwrap();
            assert_eq!(core.get_clipboard_text_native(), "[그림]");
            assert!(!core.clipboard_is_single_text_native());
            let first = ok(&core.paste_control_native(0, 2, 0).unwrap());
            let first_para = first["paraIdx"].as_u64().unwrap() as usize;
            let first_picture = match &para(&core, first_para).controls[0] {
                Control::Picture(picture) => picture,
                _ => panic!("첫 클립보드 그림"),
            };
            assert_eq!(
                (
                    first_picture.common.horizontal_offset,
                    first_picture.common.vertical_offset
                ),
                (668, 770)
            );
            let first_id = first_picture.common.instance_id;
            let binary = binaries(&core);
            let original_note = note(&core, 1);
            let original_field =
                json!(para(&core, 1).controls[para(&core, 1).field_ranges[0].control_idx]);
            if accepted {
                assert!(core.set_active_field(0, 1, 3));
                assert_eq!(
                    ok(&core.paste_html_text_logical_native(0, 1, 4, &html).unwrap())["charOffset"],
                    5
                );
                assert_eq!(fields(&core)[0]["value"], format!("A{COPY}BCD"));
                assert_eq!(
                    (
                        para(&core, 1).field_ranges[0].start_char_idx,
                        para(&core, 1).field_ranges[0].end_char_idx
                    ),
                    (2, 8)
                );
            } else {
                rejected(
                    &mut core,
                    "<p>가<img src=\"https://example.invalid/image.png\">🦦</p>",
                );
            }
            assert_eq!(note(&core, 1), original_note);
            assert_eq!(
                json!(para(&core, 1).controls[para(&core, 1).field_ranges[0].control_idx]),
                original_field
            );
            assert_eq!(core.get_clipboard_text_native(), "[그림]");
            assert!(!core.clipboard_is_single_text_native());
            let last = core.document().sections[0].paragraphs.len() - 1;
            let second = ok(&core.paste_control_native(0, last, 0).unwrap());
            let second_para = second["paraIdx"].as_u64().unwrap() as usize;
            let picture = match &para(&core, second_para).controls[0] {
                Control::Picture(picture) => picture,
                _ => panic!("기존 그림 버퍼가 남아야 한다"),
            };
            // 원래 567HU씩 누적하던 두 번째 붙이기를 1회째로 되돌리면 실패한다.
            assert_eq!(
                (
                    picture.common.horizontal_offset,
                    picture.common.vertical_offset
                ),
                (1235, 1337)
            );
            assert_eq!((picture.common.width, picture.common.height), (1001, 1303));
            assert_eq!(picture.common.description, "기존 그림 버퍼");
            assert!(!picture.common.treat_as_char);
            assert_ne!(picture.common.instance_id, first_id);
            assert_eq!(json!(picture.image_attr), json!(source.image_attr));
            assert_eq!(json!(picture.crop), json!(source.crop));
            assert_eq!(json!(picture.img_dim), json!(source.img_dim));
            assert_eq!(binaries(&core), binary, "복사한 PNG 원문 바이트");
            assert_eq!(
                json!(para(&core, 0).controls[source_control]),
                source_control_model
            );
        }
    }
}
