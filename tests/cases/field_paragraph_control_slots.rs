//! 누름틀이 있는 문단을 편집해도 같은 문단의 각주·도형·책갈피 슬롯을 지키는지 확인한다.
//!
//! 입력·값 설정·나누기·붙이기는 다른 컨트롤의 8유닛 슬롯을 그대로 두고 누름틀 경계만 옮겨야
//! 한다. 원시 위치까지 비교해 편집 중·HWP 재열기·HWPX 재열기에서 같은 문단인지 본다.

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
        Control::Header(_) => "머리말",
        Control::Footer(_) => "꼬리말",
        Control::Table(_) => "표",
        Control::Picture(_) => "그림",
        _ => "기타",
    }
}

/// 누름틀 목록에서 `location` 이 같은 것의 (시작, 끝, 값).
fn fields_at(fields: &Value, location: &Value) -> Vec<(u64, u64, String)> {
    fields
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| &f["location"] == location)
        .map(|f| {
            (
                f["startCharIdx"].as_u64().unwrap(),
                f["endCharIdx"].as_u64().unwrap(),
                f["value"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn states(core: &DocumentCore) -> Vec<State> {
    let fields: Value = serde_json::from_str(&core.get_field_list_json()).unwrap();
    core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .map(|(index, p)| State {
            text: p.text.clone(),
            fields: fields_at(
                &fields,
                &serde_json::json!({ "sectionIndex": 0, "paraIndex": index }),
            ),
            controls: p.controls.iter().map(kind).collect(),
            char_offsets: p.char_offsets.clone(),
            char_count: p.char_count,
        })
        .collect()
}

/// 구역 0 본문 문단에 든 표마다 셀 문단의 상태.
fn cell_states(core: &DocumentCore) -> Vec<State> {
    let fields: Value = serde_json::from_str(&core.get_field_list_json()).unwrap();
    let mut out = Vec::new();
    for (index, p) in core.document().sections[0].paragraphs.iter().enumerate() {
        for (control_index, control) in p.controls.iter().enumerate() {
            let Control::Table(table) = control else {
                continue;
            };
            for (cell_index, cell) in table.cells.iter().enumerate() {
                for (para_index, cp) in cell.paragraphs.iter().enumerate() {
                    let location = serde_json::json!({
                        "sectionIndex": 0,
                        "paraIndex": index,
                        "path": [{
                            "type": "cell",
                            "controlIndex": control_index,
                            "cellIndex": cell_index,
                            "paraIndex": para_index,
                        }],
                    });
                    out.push(State {
                        text: cp.text.clone(),
                        fields: fields_at(&fields, &location),
                        controls: cp.controls.iter().map(kind).collect(),
                        char_offsets: cp.char_offsets.clone(),
                        char_count: cp.char_count,
                    });
                }
            }
        }
    }
    out
}

/// 편집 중·HWP 재열기·HWPX 재열기 가운데 기대와 다른 상태를 모은다.
fn broken(core: &DocumentCore, expected: &[State]) -> Vec<String> {
    broken_in(core, expected, states)
}

/// `broken` 과 같되 `read` 로 읽은 문단들을 비교한다.
fn broken_in(
    core: &DocumentCore,
    expected: &[State],
    read: fn(&DocumentCore) -> Vec<State>,
) -> Vec<String> {
    let mut broken = Vec::new();
    for (name, actual) in [
        ("편집 중", read(core)),
        (
            "HWP 재열기",
            read(&DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap()),
        ),
        (
            "HWPX 재열기",
            read(&DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap()),
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

/// 네 컨트롤을 모두 앞 문단에 두고 "음" 앞에서 나눈 결과. 나누기 오프셋은 캐럿 위치라
/// 각주만 한 칸으로 세어 7이다 — 떠 있는 도형은 캐럿 칸이 없다.
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
    core.split_paragraph_native(0, 0, 7, None).unwrap();
    let broken = broken(&core, &split_before_last_char());
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn page_break_keeps_footnote_shape_and_bookmark_slots() {
    let mut core = four_controls();
    core.insert_page_break_native(0, 0, 7).unwrap();
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

/// "앞뒤끝": 맨 앞 빈 누름틀 "첫째", 끝 앞 빈 누름틀 "둘째".
fn first_field_empty_at_start() -> DocumentCore {
    let mut core = blank("앞뒤끝");
    core.insert_click_here_field_at(0, 0, 2, "안내문", "메모", "둘째", true)
        .unwrap();
    core.insert_click_here_field_at(0, 0, 0, "안내문", "메모", "첫째", true)
        .unwrap();
    // secd·cold 16, 첫째 BEGIN·END 16(앞 앞), 둘째 BEGIN·END 16(끝 앞)
    let broken = broken(
        &core,
        &[state(
            "앞뒤끝",
            &[(0, 0, ""), (2, 2, "")],
            &["구역", "단", "누름틀", "누름틀"],
            &[32, 33, 50],
            52,
        )],
    );
    assert!(broken.is_empty(), "준비 문서: {broken:#?}");
    core
}

#[test]
fn typing_into_first_empty_field_keeps_later_field() {
    let mut core = first_field_empty_at_start();
    core.insert_text_native(0, 0, 0, "가").unwrap();
    let broken = broken(
        &core,
        &[state(
            "가앞뒤끝",
            &[(0, 1, "가"), (3, 3, "")],
            &["구역", "단", "누름틀", "누름틀"],
            &[24, 33, 34, 51],
            53,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn typing_into_later_field_keeps_first_empty_field_at_start() {
    let mut core = first_field_empty_at_start();
    assert!(core.set_active_field(0, 0, 2));
    core.insert_text_native(0, 0, 2, "가").unwrap();
    let broken = broken(
        &core,
        &[state(
            "앞뒤가끝",
            &[(0, 0, ""), (2, 3, "가")],
            &["구역", "단", "누름틀", "누름틀"],
            &[32, 33, 42, 51],
            53,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

/// "앞뒤끝": 맨 앞 빈 누름틀, 뒤 앞 각주, 끝 앞 책갈피.
fn first_field_empty_before_controls() -> DocumentCore {
    let mut core = blank("앞뒤BMK끝");
    core.insert_footnote_native(0, 0, 1).unwrap();
    core.insert_click_here_field_at(0, 0, 0, "안내문", "메모", "첫째", true)
        .unwrap();
    let core = with_bookmark(&core, "BMK");
    let broken = broken(
        &core,
        &[state(
            "앞뒤끝",
            &[(0, 0, "")],
            &["구역", "단", "누름틀", "각주", "책갈피"],
            &[32, 41, 50],
            52,
        )],
    );
    assert!(broken.is_empty(), "준비 문서: {broken:#?}");
    core
}

#[test]
fn typing_into_first_empty_field_keeps_footnote_and_bookmark_slots() {
    let mut core = first_field_empty_before_controls();
    core.insert_text_native(0, 0, 0, "가").unwrap();
    let broken = broken(
        &core,
        &[state(
            "가앞뒤끝",
            &[(0, 1, "가")],
            &["구역", "단", "누름틀", "각주", "책갈피"],
            &[24, 33, 42, 51],
            53,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

/// "가나다라마": 다 앞(2)과 마 앞(4)에 빈 누름틀.
fn two_empty_fields() -> DocumentCore {
    let mut core = blank("가나다라마");
    core.insert_click_here_field_at(0, 0, 4, "안내문", "메모", "둘째", true)
        .unwrap();
    core.insert_click_here_field_at(0, 0, 2, "안내문", "메모", "첫째", true)
        .unwrap();
    let broken = broken(
        &core,
        &[state(
            "가나다라마",
            &[(2, 2, ""), (4, 4, "")],
            &["구역", "단", "누름틀", "누름틀"],
            &[16, 17, 34, 35, 52],
            54,
        )],
    );
    assert!(broken.is_empty(), "준비 문서: {broken:#?}");
    core
}

#[test]
fn breaking_before_empty_field_keeps_both_fields_in_new_paragraph() {
    for kind in ["문단", "쪽", "단"] {
        let mut core = two_empty_fields();
        match kind {
            "문단" => core.split_paragraph_native(0, 0, 2, None),
            "쪽" => core.insert_page_break_native(0, 0, 2),
            _ => core.insert_column_break_native(0, 0, 2),
        }
        .unwrap();
        // 새 문단 맨 앞에 첫째 BEGIN·END 16, 마 앞에 둘째 BEGIN·END 16.
        let broken = broken(
            &core,
            &[
                state("가나", &[], &["구역", "단"], &[16, 17], 19),
                state(
                    "다라마",
                    &[(0, 0, ""), (2, 2, "")],
                    &["누름틀", "누름틀"],
                    &[16, 17, 34],
                    36,
                ),
            ],
        );
        assert!(broken.is_empty(), "{kind} 나누기: {broken:#?}");
    }
}

/// "가XY나다라": 값 XY 인 누름틀(1..3), 다 앞 각주.
fn filled_field_before_footnote() -> DocumentCore {
    let mut core = blank("가나다라");
    core.insert_footnote_native(0, 0, 2).unwrap();
    core.insert_click_here_field_at(0, 0, 1, "안내문", "메모", "칸", true)
        .unwrap();
    core.set_field_value_by_name("칸", "XY").unwrap();
    let broken = broken(
        &core,
        &[state(
            "가XY나다라",
            &[(1, 3, "XY")],
            &["구역", "단", "누름틀", "각주"],
            &[16, 25, 26, 35, 44, 45],
            47,
        )],
    );
    assert!(broken.is_empty(), "준비 문서: {broken:#?}");
    core
}

#[test]
fn setting_field_value_keeps_later_footnote_slot() {
    filled_field_before_footnote();
}

#[test]
fn splitting_inside_filled_field_keeps_later_footnote_slot() {
    let mut core = filled_field_before_footnote();
    core.split_paragraph_native(0, 0, 2, None).unwrap();
    // 잘린 누름틀의 END 는 앞 문단 끝으로 가고, 새 문단에는 각주 슬롯만 남는다.
    let broken = broken(
        &core,
        &[
            state(
                "가X",
                &[(1, 2, "X")],
                &["구역", "단", "누름틀"],
                &[16, 25],
                35,
            ),
            state("Y나다라", &[], &["각주"], &[0, 1, 10, 11], 13),
        ],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn setting_field_value_keeps_adjacent_field_value() {
    // "앞가뒤": 가 를 담은 A(1..2) 바로 뒤에 빈 B. B 에 값을 넣어도 A 가 그 값을 품지 않는다.
    let mut core = blank("앞뒤");
    core.insert_click_here_field_at(0, 0, 1, "안내문", "메모", "A", true)
        .unwrap();
    core.set_field_value_by_name("A", "가").unwrap();
    core.insert_click_here_field_at(0, 0, 2, "안내문", "메모", "B", true)
        .unwrap();
    core.set_field_value_by_name("B", "나").unwrap();
    let broken = broken(
        &core,
        &[state(
            "앞가나뒤",
            &[(1, 2, "가"), (2, 3, "나")],
            &["구역", "단", "누름틀", "누름틀"],
            &[16, 25, 42, 51],
            53,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

/// 구역 첫 문단 "앞값 뒤끝 다음": 값 "값"인 누름틀(1..2), 끝 앞 글자처럼 취급한 표, 다 앞
/// 글자처럼 취급한 그림, 문단 끝 머리말·꼬리말. 둘째 문단은 "둘째 문단".
fn mixed_controls() -> DocumentCore {
    let mut core = blank("앞 뒤끝 다음");
    core.split_paragraph_native(0, 0, 7, None).unwrap();
    core.insert_text_native(0, 1, 0, "둘째 문단").unwrap();
    core.insert_click_here_field_at(0, 0, 1, "안내문", "메모", "값칸", true)
        .unwrap();
    core.set_field_value_by_name("값칸", "값").unwrap();
    core.create_table_ex_native(0, 0, 4, 1, 1, true, None, None)
        .unwrap();
    let png = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/logo/logo-16.png"
    ))
    .unwrap();
    core.insert_picture_native(
        0,
        0,
        6,
        &[],
        &png,
        1200,
        1200,
        16,
        16,
        "png",
        "그림",
        None,
        None,
    )
    .unwrap();
    core.set_picture_properties_native(0, 0, 4, r#"{"treatAsChar":true}"#)
        .unwrap();
    core.create_header_footer_native(0, true, 0).unwrap();
    core.create_header_footer_native(0, false, 0).unwrap();
    let core = reopen_hwp(&core);
    // 값 앞 BEGIN, 값 뒤 END, 끝 앞 표, 다 앞 그림, 문단 끝 머리말·꼬리말
    let broken = broken(
        &core,
        &[
            mixed_first(
                "앞값 뒤끝 다음",
                &[(1, 2, "값")],
                &[16, 25, 34, 35, 44, 45, 54, 55],
                73,
            ),
            plain_second(),
        ],
    );
    assert!(broken.is_empty(), "준비 문서: {broken:#?}");
    core
}

fn mixed_first(text: &str, fields: &[(u64, u64, &str)], offsets: &[u32], count: u32) -> State {
    state(
        text,
        fields,
        &["구역", "단", "누름틀", "표", "그림", "머리말", "꼬리말"],
        offsets,
        count,
    )
}

fn plain_second() -> State {
    state("둘째 문단", &[], &[], &[0, 1, 2, 3, 4], 6)
}

#[test]
fn typing_after_field_keeps_header_table_and_picture_slots() {
    let mut core = mixed_controls();
    core.insert_text_native(0, 0, 2, "가").unwrap();
    let broken = broken(
        &core,
        &[
            mixed_first(
                "앞값가 뒤끝 다음",
                &[(1, 2, "값")],
                &[16, 25, 34, 35, 36, 45, 46, 55, 56],
                74,
            ),
            plain_second(),
        ],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn typing_inside_field_keeps_header_table_and_picture_slots() {
    let mut core = mixed_controls();
    assert!(core.set_active_field(0, 0, 2));
    core.insert_text_native(0, 0, 2, "가").unwrap();
    let broken = broken(
        &core,
        &[
            mixed_first(
                "앞값가 뒤끝 다음",
                &[(1, 3, "값가")],
                &[16, 25, 26, 35, 36, 45, 46, 55, 56],
                74,
            ),
            plain_second(),
        ],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn breaking_before_last_char_keeps_header_table_and_picture_slots() {
    for kind in ["문단", "쪽"] {
        let mut core = mixed_controls();
        // "음" 앞 캐럿 위치: 글자 7 + 글자처럼 취급한 표·그림 2.
        match kind {
            "문단" => core.split_paragraph_native(0, 0, 9, None),
            _ => core.insert_page_break_native(0, 0, 9),
        }
        .unwrap();
        let broken = broken(
            &core,
            &[
                mixed_first(
                    "앞값 뒤끝 다",
                    &[(1, 2, "값")],
                    &[16, 25, 34, 35, 44, 45, 54],
                    72,
                ),
                state("음", &[], &[], &[0], 2),
                plain_second(),
            ],
        );
        assert!(broken.is_empty(), "{kind} 나누기: {broken:#?}");
    }
}

#[test]
fn pasting_field_copied_from_header_paragraph_leaves_header_behind() {
    // 첫 문단의 "값 "(누름틀 포함)을 복사해 둘째 문단 2번 글자 자리에 붙인다. 선택 밖의
    // 머리말·꼬리말은 따라오지 않는다.
    let mut core = mixed_controls();
    core.copy_selection_native(0, 0, 1, 0, 3).unwrap();
    core.paste_internal_native(0, 1, 2).unwrap();
    let broken = broken(
        &core,
        &[
            mixed_first(
                "앞값 뒤끝 다음",
                &[(1, 2, "값")],
                &[16, 25, 34, 35, 44, 45, 54, 55],
                73,
            ),
            state(
                "둘째값  문단",
                &[(2, 3, "값")],
                &["누름틀"],
                &[0, 1, 10, 19, 20, 21, 22],
                24,
            ),
        ],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn pasting_after_field_that_ends_paragraph_keeps_field_end_slot() {
    // 첫 문단 "앞값"은 값 누름틀(1..2)로 끝나고, 둘째 문단 "가나"는 가 뒤에 각주가 있다.
    let mut core = blank("앞");
    core.split_paragraph_native(0, 0, 1, None).unwrap();
    core.insert_text_native(0, 1, 0, "가나").unwrap();
    core.insert_footnote_native(0, 1, 1).unwrap();
    core.insert_click_here_field_at(0, 0, 1, "안내문", "메모", "끝칸", true)
        .unwrap();
    core.set_field_value_by_name("끝칸", "값").unwrap();
    let second = || state("가나", &[], &["각주"], &[0, 9], 11);
    let ready = broken(
        &core,
        &[
            state(
                "앞값",
                &[(1, 2, "값")],
                &["구역", "단", "누름틀"],
                &[16, 25],
                35,
            ),
            second(),
        ],
    );
    assert!(ready.is_empty(), "준비 문서: {ready:#?}");

    core.copy_selection_native(0, 1, 0, 1, 2).unwrap();
    core.paste_internal_native(0, 0, 2).unwrap();
    let broken = broken(
        &core,
        &[
            state(
                "앞값가나",
                &[(1, 2, "값")],
                &["구역", "단", "누름틀", "각주"],
                &[16, 25, 34, 43],
                45,
            ),
            second(),
        ],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

/// 본문 "본문" 끝에 글자처럼 취급한 1×1 표. 셀 문단 "앞뒤끝": 뒤 앞 책갈피, 끝 앞 빈 누름틀.
fn cell_with_bookmark_and_field() -> DocumentCore {
    let mut core = blank("본문");
    core.create_table_ex_native(0, 0, 2, 1, 1, true, None, None)
        .unwrap();
    core.insert_text_in_cell_native(0, 0, 2, 0, 0, 0, "앞BMK뒤끝")
        .unwrap();
    let mut core = with_bookmark(&core, "BMK");
    core.insert_click_here_field_at_in_cell(
        0,
        0,
        2,
        0,
        0,
        2,
        false,
        "안내문",
        "메모",
        "셀칸",
        true,
    )
    .unwrap();
    let broken = broken_in(
        &core,
        &[state(
            "앞뒤끝",
            &[(2, 2, "")],
            &["책갈피", "누름틀"],
            &[0, 9, 26],
            28,
        )],
        cell_states,
    );
    assert!(broken.is_empty(), "준비 문서: {broken:#?}");
    core
}

#[test]
fn typing_in_cell_keeps_bookmark_and_field_slots() {
    // 누름틀 밖(셀 문단 맨 앞)에 입력
    let mut core = cell_with_bookmark_and_field();
    core.insert_text_in_cell_native(0, 0, 2, 0, 0, 0, "가")
        .unwrap();
    let outside = broken_in(
        &core,
        &[state(
            "가앞뒤끝",
            &[(3, 3, "")],
            &["책갈피", "누름틀"],
            &[0, 1, 10, 27],
            29,
        )],
        cell_states,
    );
    // 빈 누름틀 자리에 입력하면 값이 된다.
    let mut core = cell_with_bookmark_and_field();
    core.insert_text_in_cell_native(0, 0, 2, 0, 0, 2, "가")
        .unwrap();
    let inside = broken_in(
        &core,
        &[state(
            "앞뒤가끝",
            &[(2, 3, "가")],
            &["책갈피", "누름틀"],
            &[0, 9, 18, 27],
            29,
        )],
        cell_states,
    );
    assert!(
        outside.is_empty() && inside.is_empty(),
        "{outside:#?} {inside:#?}"
    );
}

#[test]
fn merging_after_field_that_ends_paragraph_keeps_field_end_slot() {
    // "앞값뒤"를 값 누름틀 끝 바로 뒤에서 나눴다가 다시 합친다.
    let mut core = blank("앞뒤");
    core.insert_click_here_field_at(0, 0, 1, "안내문", "메모", "칸", true)
        .unwrap();
    core.set_field_value_by_name("칸", "값").unwrap();
    let whole = || {
        state(
            "앞값뒤",
            &[(1, 2, "값")],
            &["구역", "단", "누름틀"],
            &[16, 25, 34],
            36,
        )
    };
    let ready = broken(&core, &[whole()]);
    assert!(ready.is_empty(), "준비 문서: {ready:#?}");
    core.split_paragraph_native(0, 0, 2, None).unwrap();
    let split = broken(
        &core,
        &[
            state(
                "앞값",
                &[(1, 2, "값")],
                &["구역", "단", "누름틀"],
                &[16, 25],
                35,
            ),
            state("뒤", &[], &[], &[0], 2),
        ],
    );
    core.merge_paragraph_native(0, 1).unwrap();
    let merged = broken(&core, &[whole()]);
    assert!(
        split.is_empty() && merged.is_empty(),
        "{split:#?} {merged:#?}"
    );
}

#[test]
fn merging_paragraph_that_starts_with_empty_field_counts_its_end_slot() {
    // 둘째 문단 "뒤" 맨 앞의 빈 누름틀을 첫 문단 "앞"에 이어 붙인다.
    let mut core = blank("앞");
    core.split_paragraph_native(0, 0, 1, None).unwrap();
    core.insert_text_native(0, 1, 0, "뒤").unwrap();
    core.insert_click_here_field_at(0, 1, 0, "안내문", "메모", "칸", true)
        .unwrap();
    core.merge_paragraph_native(0, 1).unwrap();
    let broken = broken(
        &core,
        &[state(
            "앞뒤",
            &[(1, 1, "")],
            &["구역", "단", "누름틀"],
            &[16, 33],
            35,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}

#[test]
fn typing_after_footnote_marker_by_logical_caret_goes_after_it() {
    // 논리 위치 3 은 각주 바로 뒤, 뒤 앞이다. 글은 각주 뒤에 들어가고 각주 슬롯은 남는다.
    let mut doc =
        rhwp::wasm_api::HwpDocument::from_bytes(&field_and_footnote().export_hwp_native().unwrap())
            .unwrap();
    doc.insert_text_logical(0, 0, 3, "가").unwrap();
    let broken = broken(
        &doc,
        &[state(
            "앞 가뒤끝",
            &[(1, 1, "")],
            &["구역", "단", "누름틀", "각주"],
            &[16, 33, 42, 43, 44],
            46,
        )],
    );
    assert!(broken.is_empty(), "{broken:#?}");
}
