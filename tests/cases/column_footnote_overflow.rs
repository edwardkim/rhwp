//! 다단 쪽에서 단 각주 영역을 넘치는 각주를 잇는 자리.
//!
//! 고정 문서는 rhwp로 만든 2단·너비가 다른 3단(8000·14000·20000HU) 문서다. 한/글 12.30 Mac이
//! 열어 PDF로 인쇄한 글줄과 맞댔다(2026-11-20).
//! - 각 단 아래: 단 영역을 넘친 각주는 같은 쪽 다음 단 각주 영역에 먼저 잇고, 그 단 각주는
//!   뒤에 붙는다. 다음 단 본문은 그 위에 남는다.
//! - 오른쪽 단 아래: 앞 단 각주는 오른쪽 단 높이를 다 쓸 수 있다. 오른쪽 단을 다 채우면
//!   오른쪽 단 본문은 다음 쪽으로 가고, 남은 각주와 뒤 번호 각주는 다음 쪽 오른쪽 단에 잇는다.
//!
//! - `overflow-e2`·`overflow-r2`: 2단. 1단 각주(가1~가345)가 단을 넘치고, 단 나눔 뒤 2단 문단에 각주 하나.
//! - `overflow-e2-tail`: 뒤 문단 없이 1단 각주만 넘치는 2단.
//! - `overflow-e2-flow`·`overflow-r2-flow`: 같은 각주 뒤에 단 나눔 없이 긴 본문(흐름1~흐름300).
//! - `overflow-r3u`: 너비가 다른 3단 오른쪽 단 아래. 1단 각주(나1~)가 넘치고 2·3단 문단에 각주 하나씩.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const DIR: &str = "tests/fixtures/footnote_columns";

fn open(name: &str) -> HwpDocument {
    HwpDocument::from_bytes(&std::fs::read(format!("{DIR}/{name}")).unwrap()).unwrap()
}

fn text(node: &Value) -> String {
    let own = if node["type"] == "TextRun" {
        node["text"].as_str().unwrap_or_default().to_string()
    } else {
        String::new()
    };
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .fold(own, |all, child| all + &text(child))
}

fn lines(node: &Value) -> usize {
    usize::from(node["type"] == "TextLine")
        + node["children"]
            .as_array()
            .into_iter()
            .flatten()
            .map(lines)
            .sum::<usize>()
}

/// 쪽의 본문 단 글과 각주 영역 글. 둘 다 (왼쪽 x, 공백 정리한 글, 글줄 수).
struct Page {
    bodies: Vec<(f64, String)>,
    notes: Vec<(f64, String, usize)>,
}

impl Page {
    fn body_at(&self, x: f64) -> Option<&str> {
        self.bodies
            .iter()
            .find(|(left, _)| (left - x).abs() < 1.0)
            .map(|(_, body)| body.as_str())
    }

    fn notes_at(&self, x: f64) -> Option<(&str, usize)> {
        self.notes
            .iter()
            .find(|(left, _, _)| (left - x).abs() < 1.0)
            .map(|(_, note, count)| (note.as_str(), *count))
    }
}

fn page(doc: &HwpDocument, index: u32) -> Page {
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(index).unwrap()).unwrap();
    let squash = |node: &Value| text(node).split_whitespace().collect::<Vec<_>>().join(" ");
    let left = |node: &Value| node["bbox"]["x"].as_f64().unwrap();
    let mut bodies = Vec::new();
    let mut notes = Vec::new();
    for node in tree["children"].as_array().unwrap() {
        if node["type"] == "Body" {
            for column in node["children"].as_array().into_iter().flatten() {
                let body = squash(column);
                if !body.is_empty() {
                    bodies.push((left(column), body));
                }
            }
        } else if node["type"] == "FootnoteArea" {
            notes.push((left(node), squash(node), lines(node)));
        }
    }
    Page { bodies, notes }
}

#[test]
fn each_column_overflow_continues_in_next_column_on_same_page() {
    let doc = open("overflow-e2.hwp");
    assert_eq!(
        doc.page_count(),
        1,
        "넘친 각주는 다음 쪽이 아니라 2단에 잇는다"
    );
    let first = page(&doc, 0);
    let (left, _) = first.notes_at(113.4).expect("1단 각주 영역");
    assert!(left.ends_with("가304"), "1단 영역 끝: {left}");
    let (right, count) = first.notes_at(412.0).expect("2단 각주 영역");
    assert!(
        right.starts_with("가305") && right.ends_with("둘째 단 각주"),
        "2단 영역은 이어받은 조각 뒤에 2단 각주: {right}"
    );
    assert_eq!(count, 8, "이어받은 7줄과 2단 각주 1줄");
    assert!(first
        .body_at(412.0)
        .is_some_and(|body| body.starts_with("2단")));

    // 뒤 문단이 없어도 같은 쪽 2단에 잇는다.
    let tail = open("overflow-e2-tail.hwp");
    assert_eq!(tail.page_count(), 1);
    let (rest, _) = page(&tail, 0).notes_at(412.0).expect("2단 각주 영역");
    assert!(
        rest.starts_with("가305") && rest.ends_with("가345"),
        "{rest}"
    );
}

#[test]
fn each_column_overflow_keeps_next_column_body_above_continuation() {
    let doc = open("overflow-e2-flow.hwp");
    assert_eq!(doc.page_count(), 2);
    let first = page(&doc, 0);
    let (right, _) = first.notes_at(412.0).expect("2단 각주 영역");
    assert!(
        right.starts_with("가305") && right.ends_with("둘째 단 각주"),
        "2단 영역: {right}"
    );
    let body = first.body_at(412.0).expect("2단 본문");
    assert!(
        body.starts_with("흐름1 ") && body.ends_with("흐름180"),
        "2단 본문: {body}"
    );
    let second = page(&doc, 1);
    assert!(second.notes.is_empty(), "2쪽에는 각주가 없다");
    assert!(second
        .body_at(113.4)
        .is_some_and(|body| body.starts_with("흐름181")));
}

#[test]
fn right_column_overflow_takes_whole_right_column_and_pushes_its_body() {
    for name in ["overflow-r2.hwp", "overflow-r2-flow.hwp"] {
        let doc = open(name);
        assert_eq!(doc.page_count(), 2, "{name}");
        let first = page(&doc, 0);
        assert_eq!(first.notes.len(), 1, "{name}");
        let (notes, count) = first.notes_at(412.0).expect("오른쪽 단 각주 영역");
        assert!(notes.ends_with("가310"), "{name}: 오른쪽 단 끝 {notes}");
        assert_eq!(count, 49, "{name}: 오른쪽 단 한 단을 다 쓴다");
        assert!(
            first.body_at(412.0).is_none(),
            "{name}: 각주가 채운 오른쪽 단에는 본문이 없다"
        );
        let second = page(&doc, 1);
        let (rest, _) = second.notes_at(412.0).expect("2쪽 오른쪽 단 각주 영역");
        assert!(
            rest.starts_with("가311") && rest.ends_with("둘째 단 각주"),
            "{name}: {rest}"
        );
    }
    let pushed = page(&open("overflow-r2.hwp"), 1);
    assert!(pushed
        .body_at(113.4)
        .is_some_and(|body| body.starts_with("2단")));
    let flow = open("overflow-r2-flow.hwp");
    assert!(page(&flow, 0)
        .body_at(113.4)
        .is_some_and(|body| body.ends_with("흐름210")));
    assert!(page(&flow, 1)
        .body_at(113.4)
        .is_some_and(|body| body.starts_with("흐름211")));
}

#[test]
fn right_column_overflow_queues_later_notes_behind_continuation() {
    let doc = open("overflow-r3u.hwp");
    assert_eq!(doc.page_count(), 2);
    let first = page(&doc, 0);
    assert_eq!(first.notes.len(), 1);
    let (notes, _) = first.notes_at(436.6).expect("오른쪽 단 각주 영역");
    assert!(
        notes.starts_with("나1 ") && !notes.contains("2단 각주"),
        "{notes}"
    );
    assert!(
        first.body_at(436.6).is_none(),
        "오른쪽 단 본문은 다음 쪽으로"
    );
    let second = page(&doc, 1);
    let (rest, _) = second.notes_at(436.6).expect("2쪽 오른쪽 단 각주 영역");
    let (two, three) = (rest.find("2단 각주"), rest.find("3단 각주"));
    assert!(
        !rest.starts_with("나1 ") && two.is_some() && two < three,
        "이어받은 조각 뒤에 2·3단 각주: {rest}"
    );
}
