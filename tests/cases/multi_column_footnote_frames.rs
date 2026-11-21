//! 관계형 검사. 본문 각주 번호의 폭, 단별 줄 소속과 각주 간격, 편집 캐럿을 확인한다.
//! 근거 갱신 2026-11-21 18:19 KST. 직전 커밋 3afac61dc22b, 커밋 시각 2026-11-21 11:06:38 KST.
//! 2026-11-21에 한컴 2020 독립 Print와 96 DPI, 2px raw 실루엣을 비교했다.
//! c2-eachColumn, c2-rightColumn, c3u-eachColumn은 각각 1쪽이며 세 원본 모두
//! Native와 fresh WASM이 100.00%다. 두 단 원본의 마지막 각주는 Print에서 양쪽 단의
//! 기준선이 같고, 각주가 두 개인 단에만 원본의 주석 사이 간격이 한 번 들어간다.
//! 독립 표본은 pdf/pr9234-footnote-columns, 쪽별 값과 확대 판독은
//! mydocs/pr/assets/semanticist21-20261121/footnotes 및 validation/final/feature-pages.tsv에 있다.
//!
//! 너비가 다른 세 단의 원본은 각주 번호가 포함된 첫 두 단의 저장 줄이 실제 폭을 넘는다.
//! 처음 열기·본문 편집·두 저장 형식 재열기에서 같은 줄에 참조와 본문을 배치해야 한다.
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
    for (name, owners) in [
        ("c2-eachColumn.hwp", &[0, 1][..]),
        ("c3u-eachColumn.hwp", &[0, 1, 2][..]),
        ("c2-rightColumn.hwp", &[1][..]),
    ] {
        let doc = open(name);
        let tree = json(doc.get_page_render_tree(0).unwrap());
        let body = tree["children"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["type"] == "Body")
            .unwrap();
        let columns = body["children"].as_array().unwrap();
        let found = areas(&doc, 0);
        assert_eq!(found.len(), owners.len(), "{name}");
        for ((area, _), owner) in found.iter().zip(owners) {
            let column = &columns[*owner]["bbox"];
            assert!(
                (area.x - column["x"].as_f64().unwrap()).abs() < 0.5,
                "{name}: {owner}"
            );
            assert!(
                (area.w - column["w"].as_f64().unwrap()).abs() < 0.5,
                "{name}: {owner}"
            );
            // 모든 영역이 본문 아래 끝에 붙는다.
            assert!((area.y + area.h - found[0].0.y - found[0].0.h).abs() < 0.5);
        }
    }
}

#[test]
fn each_column_places_the_final_note_line_at_the_bottom_of_its_frame() {
    let doc = open("c2-eachColumn.hwp");
    let tree = json(doc.get_page_render_tree(0).unwrap());
    let frames: Vec<_> = tree["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|node| node["type"] == "FootnoteArea")
        .collect();
    assert_eq!(frames.len(), 2);
    let lines = |frame: &Value| -> Vec<(f64, f64)> {
        frame["children"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|node| node["type"] == "TextLine")
            .map(|node| {
                let bounds = &node["bbox"];
                (bounds["y"].as_f64().unwrap(), bounds["h"].as_f64().unwrap())
            })
            .collect()
    };
    let left = lines(frames[0]);
    let right = lines(frames[1]);
    assert_eq!((left.len(), right.len()), (1, 2));
    // 한컴 Print에서 1번·3번의 기준선이 같고, 2번과 3번 사이에는 원본의 주석 간격이 있다.
    assert!((left[0].0 - right[1].0).abs() < 0.2);
    assert!(right[0].0 + right[0].1 < right[1].0);
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

/// 한컴 Print의 세 단 본문을 줄 단위로 읽어 참조 번호의 위치까지 대조한다.
fn assert_original_body(doc: &HwpDocument) {
    let tree = json(doc.get_page_render_tree(0).unwrap());
    let body = tree["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Body")
        .unwrap();
    let columns = body["children"].as_array().unwrap();
    let expected: &[&[&str]] = &[
        &["1단 #본문입니", "다. 각주를 단", "문장입니다"],
        &["2단 #본문입니다.# 각주를", "단 문장입니다"],
        &["3단 #본문입니다. 각주를 단 문장입니", "다..."],
    ];
    assert_eq!(doc.page_count(), 1);
    assert_eq!(columns.len(), expected.len());
    for (column, printed) in columns.iter().zip(expected) {
        assert_eq!(column["type"], "Column");
        let lines: Vec<_> = column["children"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["type"] == "TextLine")
            .map(|line| {
                let mut text = String::new();
                for run in line["children"].as_array().unwrap() {
                    match run["type"].as_str() {
                        Some("TextRun") => text.push_str(run["text"].as_str().unwrap()),
                        Some("FnMarker") => text.push('#'),
                        _ => {}
                    }
                }
                text.trim_end().to_owned()
            })
            .collect();
        assert_eq!(lines.as_slice(), *printed);
    }
}

#[test]
fn original_body_wraps_references_on_open_edit_and_reopen() {
    let mut doc = open("c3u-eachColumn.hwp");
    assert_original_body(&doc);
    for para in 0..3 {
        let end = doc.document().sections[0].paragraphs[para]
            .text
            .chars()
            .count();
        doc.insert_text_native(0, para, end, "X").unwrap();
        doc.delete_text_native(0, para, end, 1).unwrap();
    }
    assert_original_body(&doc);
    for bytes in [doc.export_hwp().unwrap(), doc.export_hwpx().unwrap()] {
        assert_original_body(&HwpDocument::from_bytes(&bytes).unwrap());
    }
}
