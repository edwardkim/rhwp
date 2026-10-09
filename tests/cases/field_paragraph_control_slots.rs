//! 누름틀이 있는 문단을 편집해도 같은 문단의 각주·도형·책갈피 슬롯을 지키는지 재현한다.
//!
//! 아래 편집 경로는 끝에서 `rebuild_char_offsets` 로 `char_offsets` 를 다시 만든다. 이 함수는
//! 첫 글자 앞 컨트롤과 누름틀 BEGIN·END 갭만 되살리므로 다른 컨트롤의 8유닛 슬롯이 사라진다.
//! 원시 위치까지 비교해 편집 중·HWP 재열기·HWPX 재열기에서 같은 문단인지 확인한다.

use std::io::{Cursor, Read, Write};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;

/// 문단 하나의 비교 단위: 글자, 누름틀(시작·끝·값), 컨트롤 순서, 원시 위치.
#[derive(Debug, PartialEq)]
struct State {
    text: String,
    fields: Vec<(u64, u64, String)>,
    controls: Vec<&'static str>,
    char_offsets: Vec<u32>,
    char_count: u32,
}

fn state(
    text: &str,
    fields: &[(u64, u64, &str)],
    controls: &[&'static str],
    char_offsets: &[u32],
    char_count: u32,
) -> State {
    State {
        text: text.to_string(),
        fields: fields
            .iter()
            .map(|&(start, end, value)| (start, end, value.to_string()))
            .collect(),
        controls: controls.to_vec(),
        char_offsets: char_offsets.to_vec(),
        char_count,
    }
}

fn kind(control: &Control) -> &'static str {
    match control {
        Control::SectionDef(_) => "구역",
        Control::ColumnDef(_) => "단",
        Control::Field(_) => "누름틀",
        Control::Footnote(_) => "각주",
        Control::Shape(_) => "도형",
        Control::Bookmark(_) => "책갈피",
        _ => "기타",
    }
}

fn states(core: &DocumentCore) -> Vec<State> {
    let fields: Value = serde_json::from_str(&core.get_field_list_json()).unwrap();
    core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .map(|(index, p)| State {
            text: p.text.clone(),
            fields: fields
                .as_array()
                .unwrap()
                .iter()
                .filter(|f| f["location"]["paraIndex"] == index)
                .map(|f| {
                    (
                        f["startCharIdx"].as_u64().unwrap(),
                        f["endCharIdx"].as_u64().unwrap(),
                        f["value"].as_str().unwrap().to_string(),
                    )
                })
                .collect(),
            controls: p.controls.iter().map(kind).collect(),
            char_offsets: p.char_offsets.clone(),
            char_count: p.char_count,
        })
        .collect()
}

/// 편집 중·HWP 재열기·HWPX 재열기 가운데 기대와 다른 상태를 모은다.
fn broken(core: &DocumentCore, expected: &[State]) -> Vec<String> {
    let mut broken = Vec::new();
    for (name, actual) in [
        ("편집 중", states(core)),
        (
            "HWP 재열기",
            states(&DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap()),
        ),
        (
            "HWPX 재열기",
            states(&DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap()),
        ),
    ] {
        if actual != expected {
            broken.push(format!("{name}: {actual:?}"));
        }
    }
    broken
}

fn blank(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    core
}

fn reopen_hwp(core: &DocumentCore) -> DocumentCore {
    DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap()
}

/// "앞 뒤끝": 1번 글자 앞 빈 누름틀, 2번 글자(뒤) 앞 각주. HWP로 저장해 다시 연다.
fn field_and_footnote() -> DocumentCore {
    let mut core = blank("앞 뒤끝");
    core.insert_click_here_field_at(0, 0, 1, "안내문", "메모", "빈 필드", true)
        .unwrap();
    core.insert_footnote_native(0, 0, 2).unwrap();
    let core = reopen_hwp(&core);
    // secd·cold 16, 누름틀 BEGIN·END 16(공백 앞), 각주 8(뒤 앞)
    let broken = broken(
        &core,
        &[state(
            "앞 뒤끝",
            &[(1, 1, "")],
            &["구역", "단", "누름틀", "각주"],
            &[16, 33, 42, 43],
            45,
        )],
    );
    assert!(broken.is_empty(), "준비 문서: {broken:#?}");
    core
}

/// 대조군: 다른 컨트롤 없이 빈 누름틀 두 개(1번·3번 글자 앞)만 있는 "앞 뒤끝".
fn fields_only() -> DocumentCore {
    let mut core = blank("앞 뒤끝");
    core.insert_click_here_field_at(0, 0, 3, "안내문", "메모", "뒤 필드", true)
        .unwrap();
    core.insert_click_here_field_at(0, 0, 1, "안내문", "메모", "빈 필드", true)
        .unwrap();
    reopen_hwp(&core)
}

/// HWPX 저장본의 `anchor` 자리에 책갈피 XML 하나를 넣고 다시 연다.
fn with_bookmark(core: &DocumentCore, anchor: &str) -> DocumentCore {
    let mut input = zip::ZipArchive::new(Cursor::new(core.export_hwpx_native().unwrap())).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Contents/section0.xml" {
            let xml = String::from_utf8(bytes).unwrap();
            assert_eq!(xml.matches(anchor).count(), 1);
            bytes = xml
                .replace(
                    anchor,
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
    DocumentCore::from_bytes(&output.finish().unwrap().into_inner()).unwrap()
}

/// "앞 뒤끝 다음": 1번 글자 앞 빈 누름틀, 뒤 앞 각주, 끝 앞 떠 있는 도형, 다 앞 책갈피.
fn four_controls() -> DocumentCore {
    let mut core = blank("앞 뒤끝 BMK다음");
    core.insert_click_here_field_at(0, 0, 1, "안내문", "메모", "빈 필드", true)
        .unwrap();
    core.insert_footnote_native(0, 0, 2).unwrap();
    core.create_shape_control_native(
        0,
        0,
        3,
        4000,
        3000,
        7500,
        9000,
        false,
        "InFrontOfText",
        "rectangle",
        false,
        false,
        &[],
    )
    .unwrap();
    let core = with_bookmark(&core, "BMK");
    let broken = broken(
        &core,
        &[state(
            "앞 뒤끝 다음",
            &[(1, 1, "")],
            &["구역", "단", "누름틀", "각주", "도형", "책갈피"],
            &[16, 33, 42, 51, 52, 61, 62],
            64,
        )],
    );
    assert!(broken.is_empty(), "준비 문서: {broken:#?}");
    core
}

/// 네 컨트롤을 모두 앞 문단에 두고 "음" 앞에서 나눈 결과. 나누기 오프셋은 각주·도형을
/// 한 칸씩 세는 논리 위치라 8이다.
fn split_before_last_char() -> [State; 2] {
    [
        state(
            "앞 뒤끝 다",
            &[(1, 1, "")],
            &["구역", "단", "누름틀", "각주", "도형", "책갈피"],
            &[16, 33, 42, 51, 52, 61],
            63,
        ),
        state("음", &[], &[], &[0], 2),
    ]
}

#[test]
fn control_fields_only_paragraph_keeps_slots_when_typing() {
    let mut core = fields_only();
    core.insert_text_native(0, 0, 0, "가").unwrap();
    let outside = broken(
        &core,
        &[state(
            "가앞 뒤끝",
            &[(2, 2, ""), (4, 4, "")],
            &["구역", "단", "누름틀", "누름틀"],
            &[16, 17, 34, 35, 52],
            54,
        )],
    );
    let mut core = fields_only();
    assert!(core.set_active_field(0, 0, 1));
    core.insert_text_native(0, 0, 1, "가").unwrap();
    let inside = broken(
        &core,
        &[state(
            "앞가 뒤끝",
            &[(1, 2, "가"), (4, 4, "")],
            &["구역", "단", "누름틀", "누름틀"],
            &[16, 25, 34, 35, 52],
            54,
        )],
    );
    assert!(
        outside.is_empty() && inside.is_empty(),
        "{outside:?} {inside:?}"
    );
}

#[test]
fn typing_outside_field_keeps_footnote_slot() {
    let mut core = field_and_footnote();
    core.insert_text_native(0, 0, 0, "가").unwrap();
    let broken = broken(
        &core,
        &[state(
            "가앞 뒤끝",
            &[(2, 2, "")],
            &["구역", "단", "누름틀", "각주"],
            &[16, 17, 34, 43, 44],
            46,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn typing_inside_active_field_keeps_footnote_slot() {
    let mut core = field_and_footnote();
    assert!(core.set_active_field(0, 0, 1));
    core.insert_text_native(0, 0, 1, "가").unwrap();
    let broken = broken(
        &core,
        &[state(
            "앞가 뒤끝",
            &[(1, 2, "가")],
            &["구역", "단", "누름틀", "각주"],
            &[16, 25, 34, 43, 44],
            46,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn local_replace_before_field_keeps_field_and_footnote() {
    let mut core = field_and_footnote();
    core.replace_body_text_local_native(0, 0, 0, 1, "전")
        .unwrap();
    let broken = broken(
        &core,
        &[state(
            "전 뒤끝",
            &[(1, 1, "")],
            &["구역", "단", "누름틀", "각주"],
            &[16, 33, 42, 43],
            45,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn split_keeps_footnote_shape_and_bookmark_slots() {
    let mut core = four_controls();
    core.split_paragraph_native(0, 0, 8, None).unwrap();
    let broken = broken(&core, &split_before_last_char());
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn page_break_keeps_footnote_shape_and_bookmark_slots() {
    let mut core = four_controls();
    core.insert_page_break_native(0, 0, 8).unwrap();
    let broken = broken(&core, &split_before_last_char());
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn pasted_footnote_keeps_slot_before_following_field() {
    // p0 "가나다"(가 뒤 각주) 전체를 복사해 p1 "앞 뒤끝"(끝 앞 빈 누름틀)의 1번 글자 자리에 붙인다.
    let mut core = blank("가나다");
    core.split_paragraph_native(0, 0, 3, None).unwrap();
    core.insert_text_native(0, 1, 0, "앞 뒤끝").unwrap();
    core.insert_footnote_native(0, 0, 1).unwrap();
    core.insert_click_here_field_at(0, 1, 3, "안내문", "메모", "뒤 필드", true)
        .unwrap();
    core.copy_selection_native(0, 0, 0, 0, 3).unwrap();
    core.paste_internal_native(0, 1, 1).unwrap();
    let broken = broken(
        &core,
        &[
            state("가나다", &[], &["구역", "단", "각주"], &[16, 25, 26], 28),
            state(
                "앞가나다 뒤끝",
                &[(6, 6, "")],
                &["각주", "누름틀"],
                &[0, 1, 10, 11, 12, 13, 30],
                32,
            ),
        ],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn click_here_insert_keeps_earlier_footnote_slot() {
    let mut core = blank("앞 뒤끝");
    core.insert_footnote_native(0, 0, 1).unwrap();
    core.insert_click_here_field_at(0, 0, 3, "안내문", "메모", "뒤 필드", true)
        .unwrap();
    let broken = broken(
        &core,
        &[state(
            "앞 뒤끝",
            &[(3, 3, "")],
            &["구역", "단", "각주", "누름틀"],
            &[16, 25, 26, 43],
            45,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}
