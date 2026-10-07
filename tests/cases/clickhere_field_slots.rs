//! 누름틀 삽입·제거는 자기 슬롯만 넣고 빼며, 이웃 컨트롤과 다른 누름틀의 원시 위치를 지킨다.
//!
//! 입력은 공개 편집 API로 만든 문서를 HWP로 저장·재열기한 상태에서 시작한다. 기대값은 원시
//! 스트림 규칙(확장 컨트롤·FIELD_BEGIN·FIELD_END 각 8유닛, 글자 1유닛)으로 직접 셈한 위치이거나,
//! 지울 누름틀 없이 처음부터 만든 같은 문서다.

use std::io::{Cursor, Read, Write};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use serde_json::Value;

const TEXT: &str = "앞 뒤끝";

fn reopen(core: &DocumentCore, hwpx: bool) -> DocumentCore {
    let bytes = if hwpx {
        core.export_hwpx_native()
    } else {
        core.export_hwp_native()
    };
    DocumentCore::from_bytes(&bytes.unwrap()).unwrap()
}

/// 본문 첫 문단에 `text`를 넣고 꾸민, 저장하기 전 문서.
fn build(text: &str, edit: impl FnOnce(&mut DocumentCore)) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    edit(&mut core);
    core
}

/// "앞 뒤끝" 문단을 꾸민 뒤 HWP로 저장·재열기한 입력.
fn doc(edit: impl FnOnce(&mut DocumentCore)) -> DocumentCore {
    reopen(&build(TEXT, edit), false)
}

/// HWPX 본문의 `ANCHOR` 글자를 책갈피 하나로 바꿔 연 뒤 HWP로 저장·재열기한다.
/// 책갈피 추가 API의 별도 좌표 결함에 기대지 않고 파서가 만든 입력을 쓴다.
fn with_bookmark(core: &DocumentCore) -> DocumentCore {
    let bytes = core.export_hwpx_native().unwrap();
    let mut input = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Contents/section0.xml" {
            let xml = String::from_utf8(bytes).unwrap();
            assert_eq!(xml.matches("ANCHOR").count(), 1);
            bytes = xml
                .replace(
                    "ANCHOR",
                    "</hp:t><hp:ctrl><hp:bookmark name=\"책갈피\"/></hp:ctrl><hp:t>",
                )
                .into_bytes();
        }
        output
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let core = DocumentCore::from_bytes(&output.finish().unwrap().into_inner()).unwrap();
    reopen(&core, false)
}

fn field(core: &mut DocumentCore, at: usize, name: &str) {
    core.insert_click_here_field_at(0, 0, at, "안내", "", name, true)
        .unwrap();
}

/// 누름틀에 들어가 값을 입력한다.
fn fill(core: &mut DocumentCore, at: usize, value: &str) {
    assert!(core.set_active_field(0, 0, at));
    core.insert_text_native(0, 0, at, value).unwrap();
    core.clear_active_field();
}

fn footnote(core: &mut DocumentCore, at: usize) {
    core.insert_footnote_native(0, 0, at).unwrap();
}

fn shape(core: &mut DocumentCore, at: usize) {
    core.create_shape_control_native(
        0,
        0,
        at,
        3000,
        3000,
        20000,
        15000,
        false,
        "InFrontOfText",
        "rectangle",
        false,
        false,
        &[],
    )
    .unwrap();
}

/// 글자, 글자별 원시 위치, 문단 길이와 컨트롤 순서.
fn para_state(p: &Paragraph) -> String {
    let controls: Vec<String> = p
        .controls
        .iter()
        .map(|control| match control {
            Control::SectionDef(_) => "secd".into(),
            Control::ColumnDef(_) => "cold".into(),
            Control::Footnote(_) => "fn".into(),
            Control::Shape(_) => "shape".into(),
            Control::Bookmark(_) => "bookmark".into(),
            Control::Field(field) => format!("field:{}", field.field_name().unwrap_or("")),
            _ => "other".into(),
        })
        .collect();
    format!(
        "{} {:?} {} {}",
        p.text,
        p.char_offsets,
        p.char_count,
        controls.join(",")
    )
}

/// 문서 순서의 누름틀 `이름[시작,끝]=값`.
fn fields(core: &DocumentCore) -> String {
    let fields: Value = serde_json::from_str(&core.get_field_list_json()).unwrap();
    fields
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            format!(
                "{}[{},{}]={}",
                f["name"].as_str().unwrap(),
                f["startCharIdx"],
                f["endCharIdx"],
                f["value"].as_str().unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn state(core: &DocumentCore) -> String {
    format!(
        "{} | {}",
        para_state(&core.document().sections[0].paragraphs[0]),
        fields(core)
    )
}

/// 편집 직후 상태와 HWP·HWPX 저장·재열기 상태가 모두 기대값과 같아야 한다.
fn assert_saved(core: &DocumentCore, state: fn(&DocumentCore) -> String, expected: &str) {
    assert_eq!(state(core), expected, "편집 직후");
    for hwpx in [false, true] {
        assert_eq!(
            state(&reopen(core, hwpx)),
            expected,
            "{} 저장·재열기",
            if hwpx { "HWPX" } else { "HWP" }
        );
    }
}

/// 누름틀 N을 `at`에 넣은 결과.
fn check_insert(edit: impl FnOnce(&mut DocumentCore), at: usize, expected: &str) {
    let mut core = doc(edit);
    field(&mut core, at, "N");
    assert_saved(&core, state, expected);
}

/// `at`의 누름틀을 지운 결과는 그 누름틀 없이 만든 문서와 같아야 한다.
fn check_remove(mut core: DocumentCore, expected: DocumentCore, at: usize) {
    core.remove_field_at(0, 0, at).unwrap();
    assert_eq!(state(&core), state(&expected), "제거 직후");
    for hwpx in [false, true] {
        assert_eq!(
            state(&reopen(&core, hwpx)),
            state(&reopen(&expected, hwpx)),
            "{} 저장·재열기",
            if hwpx { "HWPX" } else { "HWP" }
        );
    }
}

// secd·cold가 0..16을 차지하므로 본문 첫 글자는 16에서 시작한다.

#[test]
fn insert_into_plain_paragraph_or_around_single_field() {
    check_insert(
        |_| {},
        0,
        "앞 뒤끝 [32, 33, 34, 35] 37 secd,cold,field:N | N[0,0]=",
    );
    check_insert(
        |_| {},
        2,
        "앞 뒤끝 [16, 17, 34, 35] 37 secd,cold,field:N | N[2,2]=",
    );
    check_insert(
        |_| {},
        4,
        "앞 뒤끝 [16, 17, 18, 19] 37 secd,cold,field:N | N[4,4]=",
    );
    check_insert(
        |core| field(core, 3, "B"),
        1,
        "앞 뒤끝 [16, 33, 34, 51] 53 secd,cold,field:N,field:B | N[1,1]= B[3,3]=",
    );
    check_insert(
        |core| {
            field(core, 3, "B");
            fill(core, 3, "값");
        },
        1,
        "앞 뒤값끝 [16, 33, 34, 43, 52] 54 secd,cold,field:N,field:B | N[1,1]= B[3,4]=값",
    );
    check_insert(
        |core| field(core, 2, "B"),
        0,
        "앞 뒤끝 [32, 33, 50, 51] 53 secd,cold,field:N,field:B | N[0,0]= B[2,2]=",
    );
    let filled_first = |core: &mut DocumentCore| {
        field(core, 0, "A");
        fill(core, 0, "가나");
    };
    check_insert(
        filled_first,
        2,
        "가나앞 뒤끝 [24, 25, 50, 51, 52, 53] 55 secd,cold,field:A,field:N | A[0,2]=가나 N[2,2]=",
    );
    check_insert(
        filled_first,
        3,
        "가나앞 뒤끝 [24, 25, 34, 51, 52, 53] 55 secd,cold,field:A,field:N | A[0,2]=가나 N[3,3]=",
    );
}

#[test]
fn insert_between_fields_keeps_both_neighbours() {
    check_insert(
        |core| {
            field(core, 1, "B");
            field(core, 3, "C");
        },
        2,
        "앞 뒤끝 [16, 33, 50, 67] 69 secd,cold,field:B,field:N,field:C | B[1,1]= N[2,2]= C[3,3]=",
    );
}

#[test]
fn insert_after_field_at_paragraph_start_keeps_text_outside() {
    check_insert(
        |core| field(core, 0, "B"),
        2,
        "앞 뒤끝 [32, 33, 50, 51] 53 secd,cold,field:B,field:N | B[0,0]= N[2,2]=",
    );
}

#[test]
fn insert_keeps_footnote_slot() {
    check_insert(
        |core| footnote(core, 3),
        1,
        "앞 뒤끝 [16, 33, 34, 43] 45 secd,cold,field:N,fn | N[1,1]=",
    );
    check_insert(
        |core| footnote(core, 1),
        3,
        "앞 뒤끝 [16, 25, 26, 43] 45 secd,cold,fn,field:N | N[3,3]=",
    );
}

#[test]
fn insert_next_to_empty_field_keeps_order() {
    check_insert(
        |core| field(core, 2, "B"),
        2,
        "앞 뒤끝 [16, 17, 50, 51] 53 secd,cold,field:B,field:N | B[2,2]= N[2,2]=",
    );
}

/// 값을 입력한 직후(저장 전)에도 이웃 누름틀 곁에 넣을 수 있다.
#[test]
fn insert_next_to_field_filled_in_this_session() {
    let mut core = build(TEXT, |core| {
        field(core, 0, "A");
        fill(core, 0, "가나");
    });
    field(&mut core, 3, "N");
    assert_saved(
        &core,
        state,
        "가나앞 뒤끝 [24, 25, 34, 51, 52, 53] 55 secd,cold,field:A,field:N | A[0,2]=가나 N[3,3]=",
    );

    let mut core = build(TEXT, |core| {
        field(core, 2, "B");
        fill(core, 2, "값");
    });
    field(&mut core, 0, "N");
    assert_saved(
        &core,
        state,
        "앞 값뒤끝 [32, 33, 42, 51, 52] 54 secd,cold,field:N,field:B | N[0,0]= B[2,3]=값",
    );
}

#[test]
fn remove_single_or_neighbouring_field() {
    check_remove(doc(|core| field(core, 2, "F")), doc(|_| {}), 2);
    check_remove(
        doc(|core| {
            field(core, 2, "F");
            fill(core, 2, "값값");
        }),
        doc(|_| {}),
        2,
    );
    check_remove(
        doc(|core| {
            field(core, 2, "F");
            field(core, 3, "B");
        }),
        doc(|core| field(core, 3, "B")),
        2,
    );
    // 저장하지 않고 바로 값을 입력한 누름틀
    check_remove(
        build(TEXT, |core| {
            field(core, 2, "F");
            fill(core, 2, "값값");
        }),
        doc(|_| {}),
        2,
    );
}

#[test]
fn remove_keeps_other_control_slots() {
    check_remove(
        doc(|core| {
            field(core, 2, "F");
            footnote(core, 3);
        }),
        doc(|core| footnote(core, 3)),
        2,
    );
    check_remove(
        doc(|core| {
            field(core, 2, "F");
            shape(core, 3);
        }),
        doc(|core| shape(core, 3)),
        2,
    );
    // ANCHOR 자리가 책갈피가 되어 "앞 뒤끝"의 1(누름틀 앞)이나 3(누름틀 뒤)에 놓인다.
    for (text, field_at) in [("앞ANCHOR 뒤끝", 8), ("앞 뒤ANCHOR끝", 2)] {
        check_remove(
            with_bookmark(&build(text, |core| field(core, field_at, "F"))),
            with_bookmark(&build(text, |_| {})),
            2,
        );
    }
}

#[test]
fn remove_field_at_paragraph_start_keeps_next_field() {
    check_remove(
        doc(|core| {
            field(core, 2, "B");
            field(core, 0, "A");
        }),
        doc(|core| field(core, 2, "B")),
        0,
    );
    check_remove(
        doc(|core| {
            field(core, 2, "B");
            field(core, 0, "A");
            fill(core, 0, "가나");
            fill(core, 4, "다");
        }),
        doc(|core| {
            field(core, 2, "B");
            fill(core, 2, "다");
        }),
        0,
    );
}

#[test]
fn remove_targets_field_reported_at_caret() {
    let filled = |core: &mut DocumentCore| {
        field(core, 0, "A");
        fill(core, 0, "가나");
    };
    let core = doc(|core| {
        filled(core);
        field(core, 2, "B");
    });
    let info: Value = serde_json::from_str(&core.get_field_info_at(0, 0, 2)).unwrap();
    assert_eq!(fields(&core), "A[0,2]=가나 B[2,2]=");
    assert_eq!(info["isGuide"], true, "캐럿은 빈 누름틀 B에 있다");
    check_remove(core, doc(filled), 2);
}

/// 굵게 한 글자들.
fn bold_chars(core: &DocumentCore) -> String {
    let p = &core.document().sections[0].paragraphs[0];
    let shapes = &core.document().doc_info.char_shapes;
    p.text
        .chars()
        .enumerate()
        .filter(|&(i, _)| shapes[p.char_shape_id_at(i).unwrap() as usize].bold)
        .map(|(_, ch)| ch)
        .collect()
}

#[test]
fn insert_and_remove_keep_following_char_format() {
    let bold = |core: &mut DocumentCore| {
        core.apply_char_format_native(0, 0, 3, 4, r#"{"bold":true}"#)
            .unwrap();
    };
    let mut core = doc(bold);
    field(&mut core, 2, "N");
    assert_saved(&core, bold_chars, "끝");

    let mut core = doc(|core| {
        field(core, 2, "F");
        bold(core);
    });
    core.remove_field_at(0, 0, 2).unwrap();
    assert_saved(&core, bold_chars, "끝");
}

/// 앞 누름틀을 지운 뒤에도 활성 상태인 뒤 누름틀 끝에서 이어 입력된다.
#[test]
fn remove_keeps_following_active_field() {
    let mut core = doc(|core| {
        field(core, 2, "B");
        fill(core, 2, "기존");
        field(core, 0, "A");
    });
    assert!(core.set_active_field(0, 0, 4));
    core.remove_field_at(0, 0, 0).unwrap();
    core.insert_text_native(0, 0, 4, "이어").unwrap();
    assert_eq!(fields(&core), "B[2,6]=기존이어");
}

/// 1×1 표 셀 문단 "앞 뒤끝"을 꾸민 뒤 HWP로 저장·재열기한 입력과 표 주소.
fn cell_doc(edit: impl FnOnce(&mut DocumentCore, usize, usize)) -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let table: Value =
        serde_json::from_str(&core.create_table_native(0, 0, 0, 1, 1).unwrap()).unwrap();
    let para = table["paraIdx"].as_u64().unwrap() as usize;
    let control = table["controlIdx"].as_u64().unwrap() as usize;
    core.insert_text_in_cell_native(0, para, control, 0, 0, 0, TEXT)
        .unwrap();
    edit(&mut core, para, control);
    (reopen(&core, false), para, control)
}

fn cell_field(core: &mut DocumentCore, para: usize, control: usize, at: usize, name: &str) {
    core.insert_click_here_field_at_in_cell(
        0, para, control, 0, 0, at, false, "안내", "", name, true,
    )
    .unwrap();
}

fn cell_state(core: &DocumentCore) -> String {
    let cell = core.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .find_map(|control| match control {
            Control::Table(table) => Some(&table.cells[0].paragraphs[0]),
            _ => None,
        })
        .unwrap();
    format!("{} | {}", para_state(cell), fields(core))
}

#[test]
fn cell_insert_after_field_at_paragraph_start() {
    let (mut core, para, control) = cell_doc(|core, para, control| {
        cell_field(core, para, control, 0, "B");
    });
    cell_field(&mut core, para, control, 2, "N");
    assert_saved(
        &core,
        cell_state,
        "앞 뒤끝 [16, 17, 34, 35] 37 field:B,field:N | B[0,0]= N[2,2]=",
    );
}

#[test]
fn cell_remove_field_at_paragraph_start() {
    let (mut core, para, control) = cell_doc(|core, para, control| {
        cell_field(core, para, control, 2, "B");
        cell_field(core, para, control, 0, "A");
    });
    core.remove_field_at_in_cell(0, para, control, 0, 0, 0, false)
        .unwrap();
    assert_saved(
        &core,
        cell_state,
        "앞 뒤끝 [0, 1, 18, 19] 21 field:B | B[2,2]=",
    );
}
