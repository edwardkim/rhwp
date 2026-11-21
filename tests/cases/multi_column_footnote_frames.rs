//! 다단 쪽의 단 각주 위치(각 단 아래·오른쪽 단 아래)와 각주 영역 편집 조회.
//!
//! 고정 문서는 rhwp로 만든 2·3단 문서이고, 한/글 12.30 Mac이 열어 PDF로 인쇄한 좌표와 맞댔다
//! (2026-11-18). 각 단 아래면 단마다 그 단의 각주를 단 폭으로 단 아래에 그리고 구분선도 단마다
//! 긋는다. 오른쪽 단 아래면 모든 각주를 오른쪽 단 폭 영역 하나에 그린다.
//! - `c2-eachColumn.hwp`: 2단, 1단 각주 1개, 2단 각주 2개. 한/글 각주 글 왼쪽 113.4·412.0px.
//! - `c3u-eachColumn.hwp`: 단 너비 8000·14000·20000HU의 3단, 단마다 각주. 113.4·238.6·436.6px.
//! - `c2-rightColumn.hwp`: `c2-eachColumn`을 오른쪽 단 아래로 바꾼 것. 세 각주 모두 412.0px.
//!
//! 한 쪽에 각주 영역이 여럿이므로 누르기·캐럿·선택 조회는 그 각주가 있는 영역을 써야 한다.
//! 각주를 고치면 그 각주를 그리는 영역 폭으로 줄을 나누고, 단 각주 위치를 바꾸면 새 폭으로 다시 나눈다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const DIR: &str = "tests/fixtures/footnote_columns";

fn open(name: &str) -> HwpDocument {
    HwpDocument::from_bytes(&std::fs::read(format!("{DIR}/{name}")).unwrap()).unwrap()
}

fn json(text: String) -> Value {
    serde_json::from_str(&text).unwrap()
}

#[derive(Debug, Clone, Copy)]
struct Area {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Area {
    fn contains_x(&self, x: f64) -> bool {
        x >= self.x - 0.5 && x <= self.x + self.w + 0.5
    }
}

/// 쪽의 각주 영역 상자와 그 안 글줄 수.
fn areas(doc: &HwpDocument, page: u32) -> Vec<(Area, usize)> {
    fn lines(node: &Value) -> usize {
        usize::from(node["type"] == "TextLine")
            + node["children"]
                .as_array()
                .into_iter()
                .flatten()
                .map(lines)
                .sum::<usize>()
    }
    let tree = json(doc.get_page_render_tree(page).unwrap());
    tree["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|node| node["type"] == "FootnoteArea")
        .map(|node| {
            let b = &node["bbox"];
            let area = Area {
                x: b["x"].as_f64().unwrap(),
                y: b["y"].as_f64().unwrap(),
                w: b["w"].as_f64().unwrap(),
                h: b["h"].as_f64().unwrap(),
            };
            (area, lines(node))
        })
        .collect()
}

#[test]
fn each_column_notes_draw_under_their_own_column() {
    for (name, lefts) in [
        ("c2-eachColumn.hwp", &[113.4, 412.0][..]),
        ("c3u-eachColumn.hwp", &[113.4, 238.6, 436.6][..]),
        ("c2-rightColumn.hwp", &[412.0][..]),
    ] {
        let doc = open(name);
        let found = areas(&doc, 0);
        assert_eq!(found.len(), lefts.len(), "{name}: 각주 영역 수");
        for ((area, _), left) in found.iter().zip(lefts) {
            assert!(
                (area.x - left).abs() < 0.5,
                "{name}: 영역 왼쪽 {area:?} != {left}"
            );
            // 모든 영역이 본문 아래 끝에 붙는다.
            assert!((area.y + area.h - found[0].0.y - found[0].0.h).abs() < 0.5);
        }
    }
}

#[test]
fn hit_test_uses_the_clicked_column_area() {
    let doc = open("c2-eachColumn.hwp");
    let found = areas(&doc, 0);
    for (index, (area, _)) in found.iter().enumerate() {
        let (x, y) = (area.x + 30.0, area.y + area.h - 6.0);
        let hit = json(doc.hit_test_in_footnote_native(0, x, y).unwrap());
        assert_eq!(hit["hit"], true);
        // 1단 영역은 각주 0, 2단 영역의 맨 아래 줄은 각주 2다.
        assert_eq!(hit["footnoteIndex"], index * 2, "영역 {index}: {hit}");
        let caret_x = hit["cursorRect"]["x"].as_f64().unwrap();
        assert!(area.contains_x(caret_x), "영역 {index}: 캐럿 {caret_x}");
    }
}

#[test]
fn caret_and_selection_follow_the_note_column() {
    for name in [
        "c2-eachColumn.hwp",
        "c3u-eachColumn.hwp",
        "c2-rightColumn.hwp",
    ] {
        let doc = open(name);
        let found = areas(&doc, 0);
        let notes = if name.starts_with("c3u") { 4 } else { 3 };
        for index in 0..notes {
            let caret = json(
                doc.get_cursor_rect_in_footnote_native(0, index, 0, 3)
                    .unwrap(),
            );
            let x = caret["x"].as_f64().unwrap();
            let y = caret["y"].as_f64().unwrap();
            let area = found
                .iter()
                .map(|(area, _)| area)
                .find(|area| area.contains_x(x) && y >= area.y && y <= area.y + area.h)
                .unwrap_or_else(|| panic!("{name}: 각주 {index} 캐럿 {caret}이 영역 밖이다"));
            let rects = json(
                doc.get_selection_rects_in_footnote_native(0, index, 0, 0, 0, 3)
                    .unwrap(),
            );
            let rects = rects.as_array().unwrap();
            assert!(!rects.is_empty(), "{name}: 각주 {index} 선택 상자가 없다");
            for rect in rects {
                assert!(
                    area.contains_x(rect["x"].as_f64().unwrap()),
                    "{name}: {rect}"
                );
            }
            // 캐럿 자리를 다시 누르면 같은 각주다.
            let hit = json(doc.hit_test_in_footnote_native(0, x, y + 4.0).unwrap());
            assert_eq!(
                hit["footnoteIndex"], index,
                "{name}: 각주 {index} 누르기 {hit}"
            );
        }
    }
}

#[test]
fn edited_note_wraps_at_column_width() {
    // 단 폭 영역의 각주는 글자를 좁히지 않고 단 폭으로 줄을 나눈다. 전단으로 바꾸면 본문 폭으로 다시 나눈다.
    let mut doc = open("c2-eachColumn.hwp");
    let control = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, rhwp::model::control::Control::Footnote(_)))
        .unwrap();
    let at = json(doc.get_note_edit_info_native(0, 0, control).unwrap())["charOffset"]
        .as_u64()
        .unwrap() as usize;
    doc.insert_text_in_footnote_native(0, 0, control, 0, at, &"넘치는 각주 ".repeat(12))
        .unwrap();
    let (column, column_lines) = areas(&doc, 0)[0];
    doc.apply_footnote_shape_native(0, r#"{"placement":"mergedColumn"}"#)
        .unwrap();
    let (body, body_lines) = areas(&doc, 0)[0];
    assert!(body.w > column.w * 2.0);
    // 2단 영역은 본문 폭의 반이 안 되므로 같은 각주가 거의 두 배 줄을 쓴다.
    assert!(
        column_lines as f64 >= (body_lines - 2) as f64 * 1.8,
        "단 폭 {column_lines}줄, 본문 폭 {body_lines}줄"
    );
    let tree = json(doc.get_page_render_tree(0).unwrap());
    assert!(tree.to_string().contains("넘치는"));
}

#[test]
fn column_and_margin_changes_refold_existing_notes() {
    // 단 수·쪽 여백을 바꾸면 각주 영역 폭이 바뀐다. 한/글 12.30 Mac은 이미 있는 각주도 곧바로 새 폭으로
    // 다시 나눈다(2단·3단·1단으로 바꿔 확인). 옛 폭의 저장 줄을 그대로 두면 글자를 좁히거나 넓혀 그린다.
    let mut doc = open("c2-eachColumn.hwp");
    let control = doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|c| matches!(c, rhwp::model::control::Control::Footnote(_)))
        .unwrap();
    let at = json(doc.get_note_edit_info_native(0, 0, control).unwrap())["charOffset"]
        .as_u64()
        .unwrap() as usize;
    doc.insert_text_in_footnote_native(0, 0, control, 0, at, &"넘치는 각주 ".repeat(20))
        .unwrap();
    let note_lines =
        |doc: &HwpDocument| match &doc.document().sections[0].paragraphs[0].controls[control] {
            rhwp::model::control::Control::Footnote(note) => note.paragraphs[0].line_segs.len(),
            _ => unreachable!(),
        };
    // 본문 폭 줄 수의 두 배 가까이 되어야 반 폭으로 다시 나눈 것이다.
    let refolded_narrow =
        |narrow: usize, wide: usize| narrow > wide && narrow as f64 >= (wide - 1) as f64 * 1.8;
    doc.set_column_def_native(0, 1, 0, true, 0).unwrap();
    let body_lines = note_lines(&doc);
    assert!(body_lines >= 3, "본문 폭 {body_lines}줄");
    // 본문 폭 영역의 글줄 수가 저장 줄 수와 같다.
    assert_eq!(areas(&doc, 0)[0].1, body_lines);
    doc.set_column_def_native(0, 2, 0, true, 2268).unwrap();
    let column_lines = note_lines(&doc);
    assert!(
        refolded_narrow(column_lines, body_lines),
        "2단으로 바꾼 뒤 {column_lines}줄, 1단 {body_lines}줄"
    );
    doc.set_column_def_native(0, 1, 0, true, 0).unwrap();
    assert_eq!(note_lines(&doc), body_lines, "다시 1단");
    let mut page = json(doc.get_page_def_native(0).unwrap());
    let (left, right) = (
        page["marginLeft"].as_u64().unwrap(),
        page["marginRight"].as_u64().unwrap(),
    );
    let width = page["width"].as_u64().unwrap();
    // 본문 폭을 반으로 줄이면 1단 각주 줄도 두 배 가까이 는다.
    let half = (width - left - right) / 4;
    page["marginLeft"] = (left + half).into();
    page["marginRight"] = (right + half).into();
    doc.set_page_def_native(0, &page.to_string()).unwrap();
    let narrow_lines = note_lines(&doc);
    assert!(
        refolded_narrow(narrow_lines, body_lines),
        "여백을 늘린 뒤 {narrow_lines}줄, 본문 폭 {body_lines}줄"
    );
}
