//! F11(앞 개체 고르기)의 주소: 같은 글자 자리의 누름틀, 표 셀·머리말 안 누름틀.
//!
//! 한/글 12.30 Mac의 ⌃F11은 같은 자리의 빈 누름틀 둘을 뒤의 것부터 하나씩 고르고, 고른 누름틀에 친 글이
//! 그 누름틀에 든다. 셀·각주·머리말 안에서는 그 영역의 누름틀을 먼저 고르고, 다음에 셀이면 그 표를,
//! 각주·머리말이면 그 조판 부호 앞의 본문 개체를 고른다. 본문에서는 표 안으로 들어가지 않는다.

use rhwp::document_core::queries::{BookmarkHeaderFooter, BookmarkPlace};
use rhwp::document_core::DocumentCore;
use serde_json::Value;

fn blank(text: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    core
}

fn json(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap()
}

fn body(para: usize) -> BookmarkPlace {
    BookmarkPlace {
        section: 0,
        para,
        cell_path: vec![],
        header_footer: None,
    }
}

fn nested(para: usize, path: Vec<(usize, usize, usize)>) -> BookmarkPlace {
    BookmarkPlace {
        cell_path: path,
        ..body(para)
    }
}

fn find(core: &DocumentCore, place: &BookmarkPlace, offset: usize, from: Option<usize>) -> Value {
    json(
        &core
            .find_nearest_control_at_native(place, offset, from, false)
            .unwrap(),
    )
}

fn field(core: &mut DocumentCore, para: usize, offset: usize, name: &str) -> usize {
    core.insert_click_here_field_at(0, para, offset, &format!("{name}안내"), "", name, true)
        .unwrap();
    let list = json(&core.get_field_list_json());
    let found = list
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == name)
        .unwrap();
    found["controlIndex"].as_u64().unwrap() as usize
}

fn values(core: &DocumentCore) -> Vec<(String, String, u64, u64)> {
    json(&core.get_field_list_json())
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["name"].as_str().unwrap().to_string(),
                f["value"].as_str().unwrap().to_string(),
                f["startPos"].as_u64().unwrap(),
                f["endPos"].as_u64().unwrap(),
            )
        })
        .collect()
}

#[test]
fn same_place_click_here_fields_have_distinct_addresses_and_take_typing_one_by_one() {
    // "앞 뒤끝"의 글자 자리 2에 빈 누름틀 가·나를 차례로 넣으면 원시 슬롯은 가 시작·가 끝·나 시작·나 끝이다.
    let mut core = blank("앞 뒤끝");
    let a = field(&mut core, 0, 2, "가");
    let b = field(&mut core, 0, 2, "나");
    assert_ne!(a, b);
    // 문단 끝에서 F11: 나, 그 앞 형제 가, 그다음은 없다.
    let first = find(&core, &body(0), 4, None);
    assert_eq!(first["type"], "field");
    assert_eq!(first["ci"].as_u64(), Some(b as u64));
    assert_eq!(
        (first["start"].as_u64(), first["end"].as_u64()),
        (Some(2), Some(2))
    );
    let second = find(&core, &body(0), 4, Some(b));
    assert_eq!(second["ci"].as_u64(), Some(a as u64));
    assert_eq!(find(&core, &body(0), 4, Some(a))["type"], "none");

    // 고른 나를 활성으로 두면 그 자리 입력은 나에 든다.
    assert!(core.set_active_field_at(&body(0), b).unwrap());
    assert!(!core.set_active_field_at(&body(0), 0).unwrap());
    core.insert_text_native(0, 0, 2, "값").unwrap();
    let expected = vec![
        ("가".into(), "".into(), 26, 29),
        ("나".into(), "값".into(), 45, 46),
    ];
    assert_eq!(values(&core), expected);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(values(&DocumentCore::from_bytes(&bytes).unwrap()), expected);
    }
}

#[test]
fn caret_in_table_cell_picks_cell_field_then_its_table() {
    let mut core = blank("앞");
    let body_field = field(&mut core, 0, 0, "본문");
    let table = json(&core.create_table_native(0, 0, 1, 1, 1).unwrap());
    let (tp, tc) = (
        table["paraIdx"].as_u64().unwrap() as usize,
        table["controlIdx"].as_u64().unwrap() as usize,
    );
    core.insert_text_in_cell_native(0, tp, tc, 0, 0, 0, "셀 ")
        .unwrap();
    core.insert_click_here_field_at_by_path(0, tp, &[(tc, 0, 0)], 2, "셀안내", "", "셀", true)
        .unwrap();
    let cell = nested(tp, vec![(tc, 0, 0)]);

    let found = find(&core, &cell, 3, None);
    assert_eq!(found["type"], "field");
    assert_eq!(found["areas"], serde_json::json!(["table"]));
    assert_eq!(
        found["place"],
        serde_json::json!({"section":0,"para":tp,"cellPath":[[tc,0,0]]})
    );
    assert_eq!(
        (found["start"].as_u64(), found["end"].as_u64()),
        (Some(2), Some(2))
    );
    let in_cell = found["ci"].as_u64().unwrap() as usize;

    // 셀을 다 훑으면 그 표를 고른다.
    let host = find(&core, &cell, 0, Some(in_cell));
    assert_eq!(host["type"], "table");
    assert_eq!(host["ci"].as_u64(), Some(tc as u64));
    assert_eq!(
        host["place"],
        serde_json::json!({"section":0,"para":tp,"cellPath":[]})
    );
    // 표에서는 표 앞 본문 개체로 간다.
    let before = find(&core, &body(tp), 0, Some(tc));
    assert_eq!(before["ci"].as_u64(), Some(body_field as u64));

    // 표 뒤 본문에서는 셀 안으로 들어가지 않고 표를 고른다.
    let last = core.document().sections[0].paragraphs.len() - 1;
    let after = find(&core, &body(last), usize::MAX, None);
    assert_eq!(after["type"], "table");
}

#[test]
fn caret_in_header_picks_header_field_then_body_before_the_header() {
    let mut core = blank("본문");
    core.add_bookmark_native(0, 0, 0, "처음").unwrap();
    let header = json(&core.create_header_footer_native(0, true, 0).unwrap());
    let (hp, hc) = (
        header["paraIndex"].as_u64().unwrap() as usize,
        header["controlIndex"].as_u64().unwrap() as usize,
    );
    core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "머리 ")
        .unwrap();
    core.insert_click_here_field_at_by_path(0, hp, &[(hc, 0, 0)], 3, "머리안내", "", "머리", true)
        .unwrap();
    let place = BookmarkPlace {
        header_footer: Some(BookmarkHeaderFooter {
            is_header: true,
            apply_to: 0,
        }),
        ..body(0)
    };

    let found = find(&core, &place, 100, None);
    assert_eq!(found["type"], "field");
    assert_eq!(
        found["place"],
        serde_json::json!({"section":0,"para":0,"cellPath":[],"headerFooter":{"isHeader":true,"applyTo":0}})
    );
    assert_eq!(found["areas"], serde_json::json!([]));
    let in_header = found["ci"].as_u64().unwrap() as usize;
    let next = find(&core, &place, 0, Some(in_header));
    assert_eq!(next["type"], "bookmark");
    assert_eq!(
        next["place"],
        serde_json::json!({"section":0,"para":hp,"cellPath":[]})
    );
}
