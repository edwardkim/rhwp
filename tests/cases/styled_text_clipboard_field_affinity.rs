//! 순수 서식 글 조각은 일반 입력의 누름틀 소속과 논리 주석 슬롯을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::{
    MarkpenMark, OrphanFieldEnd, Paragraph, RangeTag, TitleMark, TAB_EXT_PLACEHOLDER,
};
use serde_json::{json, Value};

const COPY: &str = "가🦦";
const PREFIX: &str = "앞🦦";
const SUFFIX: &str = "나다";

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
            json!(core.document().doc_info.char_shapes[id])
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
    core.insert_text_native(0, 0, 0, "가🦦XX끝").unwrap();
    core.insert_click_here_field_at(0, 0, 4, "원문 안내", "원문 메모", "source", true)
        .unwrap();
    core.set_field_value_by_name("source", "ABCD").unwrap();
    core.apply_char_format_native(0, 0, 0, 1, r##"{"bold":true,"textColor":"#cc0000"}"##)
        .unwrap();
    core.apply_char_format_native(0, 0, 1, 2, r##"{"italic":true,"textColor":"#003399"}"##)
        .unwrap();
    insert_note(&mut core, 0, 3, kind, "원문 주석🦦");
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
    assert_eq!(para(core, 0).text, "가🦦XXABCD끝");
    assert_eq!(
        para(core, 1).text,
        format!("{PREFIX}{}{SUFFIX}", if empty { "" } else { "ABCD" })
    );
    let field = fields(core);
    assert_eq!(field.as_array().unwrap().len(), 2);
    assert_eq!(field[0]["name"], "source");
    assert_eq!(field[0]["value"], "ABCD");
    assert_eq!(field[1]["name"], "target");
    assert_eq!(field[1]["guide"], "대상 안내");
    assert_eq!(field[1]["value"], if empty { "" } else { "ABCD" });
    assert_eq!(field[1]["startCharIdx"], 2);
    assert_eq!(field[1]["endCharIdx"], if empty { 2 } else { 6 });
    assert_eq!(field[1]["editableInForm"], true);
    // 공개 커서 조회는 scalar·8유닛 슬롯·가상 안내문 축이며 HWP UTF-16 축과 다르다.
    let content_start = if before { 18 } else { 10 };
    assert_eq!(field[1]["startPos"], content_start);
    assert_eq!(
        field[1]["endPos"],
        content_start + if empty { 5 } else { 4 }
    );
    assert_eq!(
        para(core, 1).char_offsets[2],
        if before { 11 } else { 3 } + if empty { 16 } else { 8 },
        "앞·surrogate·주석과 실제 BEGIN/END의 UTF-16 스트림 위치",
    );
    assert_eq!(
        field[1]["location"],
        json!({"sectionIndex":0,"paraIndex":1})
    );
    assert_eq!(note(core, 0).1, 3);
    assert_eq!(
        note(core, 1).1,
        if before {
            1
        } else if empty {
            3
        } else {
            7
        }
    );
    for index in [0, 1] {
        let (control, _, _) = note(core, index);
        assert!(matches!(
            (&para(core, index).controls[control], kind),
            (Control::Footnote(_), Note::Foot) | (Control::Endnote(_), Note::End)
        ));
        let info = ok(&core.get_footnote_info_native(0, index, control).unwrap());
        assert_eq!(
            info["texts"],
            json!([if index == 0 {
                "  원문 주석🦦"
            } else {
                "  대상 주석🦦"
            }])
        );
        assert_eq!(info["number"], index + 1);
    }
    for (at, color, bold, italic) in [(0, "#cc0000", true, false), (1, "#003399", false, true)] {
        let props: Value =
            serde_json::from_str(&core.get_char_properties_at_native(0, 0, at).unwrap()).unwrap();
        assert_eq!(props["textColor"], color);
        assert_eq!(props["bold"], bold);
        assert_eq!(props["italic"], italic);
    }
}
fn copy_text(core: &mut DocumentCore) {
    // 원문 문단에는 필드·주석이 있어도 선택한 두 글자는 모두 그 바깥이다.
    assert_eq!(
        ok(&core.copy_selection_logical_native(0, 0, 0, 0, 2).unwrap())["text"],
        COPY
    );
    assert!(core.clipboard_is_single_text_native());
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
fn assert_edit(core: &DocumentCore, before: &DocumentCore, expected: &Expected) {
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
        "원문 글·필드·주석·전체 글자 모양"
    );
    assert_eq!(content(core, 2), content(before, 2), "선택 밖 이웃 문단");
    let old_styles = full_styles(before, 1);
    let expected_styles: Vec<Value> = old_styles[..expected.from]
        .iter()
        .cloned()
        .chain(full_styles(before, 0).into_iter().take(2))
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
    expected_fields[1]["value"] = json!(expected.value);
    expected_fields[1]["startCharIdx"] = json!(expected.start);
    expected_fields[1]["endCharIdx"] = json!(expected.end);
    for (key, delta) in [
        ("startPos", expected.start_query_delta),
        ("endPos", expected.end_query_delta),
    ] {
        expected_fields[1][key] = json!(expected_fields[1][key].as_i64().unwrap() + delta);
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
fn each_source(empty: bool, mut edit: impl FnMut(&mut DocumentCore, bool) -> Expected) {
    for kind in [Note::Foot, Note::End] {
        for before in [true, false] {
            let original = fixture(kind, before, empty);
            assert_source(&original, kind, before, empty);
            for bytes in [
                original.export_hwp_with_adapter().unwrap(),
                original.export_hwpx_native().unwrap(),
            ] {
                // 각 원본 형식으로 직접 열고, 미편집 저장본의 정규화와 편집 차이를 구분한다.
                let baseline = DocumentCore::from_bytes(&bytes).unwrap();
                assert_source(&baseline, kind, before, empty);
                let mut edited = DocumentCore::from_bytes(&bytes).unwrap();
                assert_source(&edited, kind, before, empty);
                copy_text(&mut edited);
                let before_saved = saved(&baseline);
                let expected = edit(&mut edited, before);
                let mut typed = DocumentCore::from_bytes(&bytes).unwrap();
                typed.clear_active_field();
                if expected.deleted > 0 {
                    typed
                        .delete_text_native(0, 1, expected.from, expected.deleted)
                        .unwrap();
                }
                if expected.activate_field {
                    assert!(typed.set_active_field(0, 1, expected.from));
                }
                // 현재 일반 입력 API를 독립 대조군으로 삼아 조회 축을 실제로 확인한다.
                // 주석 양쪽은 같은 scalar·필드 조회 좌표를 가지며, 주석 소속은 별도 literal 검사로 지킨다.
                typed.insert_text_native(0, 1, expected.from, COPY).unwrap();
                assert_eq!(
                    fields(&edited),
                    fields(&typed),
                    "서식 붙이기와 일반 입력의 전체 필드 조회 계약"
                );
                assert_edit(&edited, &baseline, &expected);
                for ((before_format, after_format), typed_format) in before_saved
                    .iter()
                    .zip(saved(&edited).iter())
                    .zip(saved(&typed).iter())
                {
                    assert_eq!(
                        fields(after_format),
                        fields(typed_format),
                        "각 저장 형식의 일반 입력 대조군"
                    );
                    assert_edit(after_format, before_format, &expected);
                }
            }
        }
    }
}
fn logical(scalar: usize, note_at: usize) -> usize {
    scalar + usize::from(note_at < scalar)
}

#[test]
fn styled_text_paste_active_and_inactive_field_boundaries_follow_typing() {
    for scalar_api in [false, true] {
        for active in [false, true] {
            for from in [2, 3, 6] {
                each_source(false, |core, before| {
                    core.clear_active_field();
                    if active {
                        assert!(core.set_active_field(0, 1, from));
                    }
                    let result = if scalar_api {
                        core.paste_internal_native(0, 1, from)
                    } else {
                        core.paste_internal_text_logical_native(
                            0,
                            1,
                            logical(from, if before { 1 } else { 7 }),
                        )
                    }
                    .unwrap();
                    assert_eq!(
                        ok(&result)["charOffset"],
                        from + 2,
                        "API는 실제 끝 scalar를 반환한다"
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
fn styled_text_suffix_replacement_reactivates_field_at_new_end() {
    each_source(false, |core, before| {
        core.clear_active_field();
        core.delete_text_native(0, 1, 3, 3).unwrap();
        assert!(core.set_active_field(0, 1, 3));
        assert_eq!(
            ok(&core
                .paste_internal_text_logical_native(0, 1, logical(3, if before { 1 } else { 4 }))
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
fn styled_text_first_input_populates_empty_guide_with_or_without_activation() {
    for active in [false, true] {
        each_source(true, |core, before| {
            core.clear_active_field();
            if active {
                assert!(core.set_active_field(0, 1, 2));
            }
            assert_eq!(
                ok(&core
                    .paste_internal_text_logical_native(
                        0,
                        1,
                        logical(2, if before { 1 } else { 3 })
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
                // 기존 조회 축은 빈 안내문 다섯 글자를 실제 값 두 글자로 바꾼다.
                end_query_delta: -3,
                note_at: if before { 1 } else { 5 },
                activate_field: active,
            }
        });
    }
}

#[test]
fn styled_text_paste_respects_both_sides_of_logical_note_slot() {
    for after in [false, true] {
        each_source(false, |core, before| {
            core.clear_active_field();
            let from = if before { 1 } else { 7 };
            assert_eq!(
                ok(&core
                    .paste_internal_text_logical_native(0, 1, from + usize::from(after))
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
fn styled_text_plain_target_is_control_free_and_keeps_all_character_shapes() {
    for scalar_api in [false, true] {
        let mut seed = blank();
        seed.insert_text_native(0, 0, 0, COPY).unwrap();
        seed.apply_char_format_native(0, 0, 0, 1, r##"{"bold":true,"textColor":"#cc0000"}"##)
            .unwrap();
        seed.apply_char_format_native(0, 0, 1, 2, r##"{"italic":true,"textColor":"#003399"}"##)
            .unwrap();
        seed.insert_paragraph_native(0, 1).unwrap();
        seed.insert_text_native(0, 1, 0, "ABCD").unwrap();
        for mut core in saved(&seed) {
            let before = content(&core, 0);
            let styles = full_styles(&core, 1);
            let copied = full_styles(&core, 0);
            copy_text(&mut core);
            let result = if scalar_api {
                core.paste_internal_native(0, 1, 2)
            } else {
                core.paste_internal_text_logical_native(0, 1, 2)
            }
            .unwrap();
            assert_eq!(ok(&result)["charOffset"], 4);
            let expected = styles[..2]
                .iter()
                .cloned()
                .chain(copied)
                .chain(styles[2..].iter().cloned())
                .collect::<Vec<_>>();
            assert_eq!(para(&core, 1).text, "AB가🦦CD");
            assert!(para(&core, 1).controls.is_empty());
            assert_eq!(full_styles(&core, 1), expected);
            assert_eq!(content(&core, 0), before);
        }
    }
}

fn unchanged(core: &DocumentCore) -> Value {
    json!({"document":format!("{:?}", core.document()),"events":core.serialize_event_log(),
        "clipboard":core.get_clipboard_text_native(),"query":core.clipboard_is_single_text_native()})
}
fn reject_nontext(core: &mut DocumentCore) {
    assert!(!core.clipboard_is_single_text_native());
    let before = unchanged(core);
    assert_eq!(
        serde_json::from_str::<Value>(&core.paste_internal_text_logical_native(0, 1, 0).unwrap())
            .unwrap()["ok"],
        false
    );
    assert_eq!(
        unchanged(core),
        before,
        "미지원 조각은 문서·이벤트·클립보드를 바꾸지 않는다"
    );
}
#[test]
fn styled_text_query_rejects_note_field_multiple_paragraphs_and_empty_clipboard() {
    for kind in [Note::Foot, Note::End] {
        let mut core = fixture(kind, true, false);
        reject_nontext(&mut core);
        core.copy_selection_logical_native(0, 0, 3, 0, 4).unwrap();
        reject_nontext(&mut core);
        core.copy_selection_logical_native(0, 0, 5, 0, 9).unwrap();
        reject_nontext(&mut core);
        core.copy_selection_logical_native(0, 0, 0, 1, 0).unwrap();
        reject_nontext(&mut core);
        core.copy_selection_logical_native(0, 0, 0, 0, 0).unwrap();
        reject_nontext(&mut core);
        copy_text(&mut core);
        let before = unchanged(&core);
        for _ in 0..2 {
            assert!(core.clipboard_is_single_text_native());
        }
        assert_eq!(unchanged(&core), before, "분류 조회는 읽기 전용이다");
    }
    // 개체·필드가 없는 두 문단만으로도 단일 문단 조건을 따로 확인한다.
    let mut core = blank();
    core.insert_text_native(0, 0, 0, "AB").unwrap();
    core.insert_paragraph_native(0, 1).unwrap();
    core.insert_text_native(0, 1, 0, "CD").unwrap();
    core.copy_selection_logical_native(0, 0, 0, 1, 2).unwrap();
    reject_nontext(&mut core);
}

#[test]
fn styled_text_query_rejects_annotations_and_real_tab_extensions() {
    for annotation in 0..5 {
        let mut core = blank();
        core.insert_text_native(0, 0, 0, "가\t🦦").unwrap();
        core.insert_paragraph_native(0, 1).unwrap();
        let mut doc = core.document().clone();
        let p = &mut doc.sections[0].paragraphs[0];
        match annotation {
            0 => p.range_tags.push(RangeTag {
                start: p.char_offsets[0],
                end: p.char_offsets[2],
                tag: 0x02000001,
            }),
            1 => p.title_marks.push(TitleMark {
                char_idx: 1,
                ignore: false,
            }),
            2 => p.markpen_marks.push(MarkpenMark {
                char_idx: 1,
                color: Some("#ffff00".into()),
                utf16_pos: None,
            }),
            3 => p.orphan_field_ends.push(OrphanFieldEnd {
                char_idx: 1,
                begin_id_ref: 900,
                field_id: 901,
                begin_ctrl_id: 0,
            }),
            4 => p.tab_extended = vec![[37, 0, 0, 0, 0, 0, 9]],
            _ => unreachable!(),
        }
        core.set_document(doc);
        core.copy_selection_logical_native(0, 0, 0, 0, 3).unwrap();
        reject_nontext(&mut core);
    }
    let mut core = blank();
    core.insert_text_native(0, 0, 0, "가\t🦦").unwrap();
    let mut doc = core.document().clone();
    doc.sections[0].paragraphs[0].tab_extended = vec![TAB_EXT_PLACEHOLDER];
    core.set_document(doc);
    core.copy_selection_logical_native(0, 0, 0, 0, 3).unwrap();
    assert!(
        core.clipboard_is_single_text_native(),
        "저장 정보 없는 탭의 자리표는 허용한다"
    );
}

#[test]
fn styled_text_logical_paste_invalid_positions_do_not_mutate_or_consume_clipboard() {
    let mut core = fixture(Note::Foot, true, false);
    copy_text(&mut core);
    let before = unchanged(&core);
    let bytes = [
        core.export_hwp_with_adapter().unwrap(),
        core.export_hwpx_native().unwrap(),
    ];
    for (section, paragraph, offset) in [(9, 1, 0), (0, 99, 0), (0, 1, 100)] {
        assert!(core
            .paste_internal_text_logical_native(section, paragraph, offset)
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
}
