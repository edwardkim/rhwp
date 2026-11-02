//! 관계형·고정형: 편집한 미주 번호를 한/글 저장본과 대조하고, 한/글이 저장한 번호를 기대값으로 검사한다.
//! 주석 갱신: 2026-11-04T19:09:06+09:00; 새 파일이므로 직전 파일 변경 커밋은 없다.
//! 편집 명령이 저장하는 번호와 모든 쪽에 그린 `N)`의 순서를 보는 계약이며 쪽의 px 좌표나 일치율을 주장하지 않는다.
//! 문서를 여는 조판은 바꾸지 않아 시각 raw 측정은 비해당이다. 기준은 한/글 12.30(Mac) 저장본 세 개다.
//!
//! 미주 번호는 구역마다 미주 모양의 번호 매기기를 따른다(한/글 12.30 Mac).
//!
//! - 앞 구역에 이어서: 앞 구역의 번호를 잇는다.
//! - 현재 구역부터 새로 시작: 미주가 없는 구역이라도 그 구역의 시작 번호에서 다시 센다.
//! - 미주 번호의 새 번호로 시작: 다음 미주 번호를 그 값으로 정한다.
//!
//! 고정 문서는 한/글이 저장한 두 구역 문서다(구역마다 미주 둘, 문서 끝 배치).
//! `restart-5`는 구역 0이 5부터 새로 시작, `continue`는 같은 문서에서 구역 0을 앞 구역에
//! 이어서로 바꾼 것(시작 번호 5는 남지만 쓰지 않는다), `new-number-20`은 `restart-5`에서
//! 구역 0의 미주를 지우고 구역 1 첫머리에 미주 번호 20의 새 번호로 시작을 넣은 것이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const DIR: &str = "tests/fixtures/endnote_numbering";

fn open(name: &str) -> HwpDocument {
    let bytes = std::fs::read(format!("{DIR}/{name}")).unwrap();
    HwpDocument::from_bytes(&bytes).unwrap()
}

fn collect(paragraphs: &[Paragraph], out: &mut Vec<u16>) {
    for para in paragraphs {
        for ctrl in &para.controls {
            match ctrl {
                Control::Endnote(note) => out.push(note.number),
                Control::Table(table) => {
                    for cell in &table.cells {
                        collect(&cell.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}

/// 구역마다 미주의 저장 번호.
fn numbers(doc: &HwpDocument) -> Vec<Vec<u16>> {
    doc.document()
        .sections
        .iter()
        .map(|section| {
            let mut out = Vec::new();
            collect(&section.paragraphs, &mut out);
            out
        })
        .collect()
}

/// 모든 쪽에 그린 `N)` 표시를 차례로 모은다(본문 미주 번호와 문서 끝 미주 내용 번호).
fn drawn(doc: &HwpDocument) -> String {
    let text: String = (0..doc.page_count())
        .map(|page| {
            let svg = doc.render_page_svg_native(page).unwrap();
            svg.split("<text")
                .skip(1)
                .filter_map(|t| Some(t.split_once('>')?.1.split_once("</text>")?.0.to_string()))
                .collect::<String>()
        })
        .collect();
    let mut marks = Vec::new();
    let mut digits = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        if ch == ')' && !digits.is_empty() {
            marks.push(format!("{digits})"));
        }
        digits.clear();
    }
    marks.join(" ")
}

fn shape(doc: &HwpDocument, section: usize) -> (String, u64) {
    let v: Value = serde_json::from_str(&doc.get_endnote_shape_native(section).unwrap()).unwrap();
    (
        v["numbering"].as_str().unwrap().to_string(),
        v["startNumber"].as_u64().unwrap(),
    )
}

fn reopened(doc: &HwpDocument) -> [HwpDocument; 2] {
    [
        HwpDocument::from_bytes(&doc.export_hwp().unwrap()).unwrap(),
        HwpDocument::from_bytes(&doc.export_hwpx().unwrap()).unwrap(),
    ]
}

fn first_endnote(doc: &HwpDocument, section: usize) -> (usize, usize) {
    for (pi, para) in doc.document().sections[section]
        .paragraphs
        .iter()
        .enumerate()
    {
        if let Some(ci) = para
            .controls
            .iter()
            .position(|c| matches!(c, Control::Endnote(_)))
        {
            return (pi, ci);
        }
    }
    panic!("구역 {section}에 미주가 없습니다");
}

fn delete_first_endnote(doc: &mut HwpDocument, section: usize) {
    let (para, control) = first_endnote(doc, section);
    doc.delete_footnote_native(section, para, control).unwrap();
}

#[test]
fn hancom_files_open_with_their_numbers() {
    assert_eq!(numbers(&open("restart-5.hwpx")), [vec![5, 6], vec![7, 8]]);
    assert_eq!(numbers(&open("continue.hwpx")), [vec![1, 2], vec![3, 4]]);
    assert_eq!(numbers(&open("new-number-20.hwpx")), [vec![], vec![20, 21]]);
}

#[test]
fn continue_after_restart_numbers_like_hancom() {
    let mut doc = open("restart-5.hwpx");
    doc.apply_endnote_shape_native(0, r#"{"numbering":"continue"}"#)
        .unwrap();
    let hancom = open("continue.hwpx");
    assert_eq!(numbers(&doc), numbers(&hancom));
    assert_eq!(drawn(&doc), "1) 2) 3) 4) 1) 2) 3) 4)");
    assert_eq!(drawn(&doc), drawn(&hancom));
    // 한/글처럼 시작 번호 5는 남긴다.
    assert_eq!(shape(&doc, 0), ("continue".to_string(), 5));
    for re in reopened(&doc) {
        assert_eq!(numbers(&re), [vec![1, 2], vec![3, 4]]);
        assert_eq!(shape(&re, 0), ("continue".to_string(), 5));
    }
}

#[test]
fn restart_after_continue_numbers_like_hancom() {
    let mut doc = open("continue.hwpx");
    doc.apply_endnote_shape_native(0, r#"{"numbering":"restartSection"}"#)
        .unwrap();
    assert_eq!(numbers(&doc), [vec![5, 6], vec![7, 8]]);
    assert_eq!(drawn(&doc), drawn(&open("restart-5.hwpx")));
    for re in reopened(&doc) {
        assert_eq!(numbers(&re), [vec![5, 6], vec![7, 8]]);
        assert_eq!(shape(&re, 0), ("restartSection".to_string(), 5));
    }
}

#[test]
fn inserted_endnote_continues_previous_section() {
    let mut doc = open("restart-5.hwpx");
    let inserted: Value =
        serde_json::from_str(&doc.insert_endnote_native(1, 0, 0).unwrap()).unwrap();
    assert_eq!(inserted["endnoteNumber"], 7);
    assert_eq!(numbers(&doc), [vec![5, 6], vec![7, 8, 9]]);
    for re in reopened(&doc) {
        assert_eq!(numbers(&re), [vec![5, 6], vec![7, 8, 9]]);
    }

    // 앞 구역에 넣은 미주는 이어 세는 뒤 구역의 번호를 민다.
    let inserted: Value =
        serde_json::from_str(&doc.insert_endnote_native(0, 0, 0).unwrap()).unwrap();
    assert_eq!(inserted["endnoteNumber"], 5);
    assert_eq!(numbers(&doc), [vec![5, 6, 7], vec![8, 9, 10]]);
}

#[test]
fn deleted_endnote_renumbers_following_sections() {
    let mut doc = open("restart-5.hwpx");
    delete_first_endnote(&mut doc, 0);
    // 한/글 저장본: 구역 0은 5), 이어 세는 구역 1은 6) 7).
    assert_eq!(numbers(&doc), [vec![5], vec![6, 7]]);
    assert_eq!(drawn(&doc), "5) 6) 7) 5) 6) 7)");
    for re in reopened(&doc) {
        assert_eq!(numbers(&re), [vec![5], vec![6, 7]]);
    }

    // 미주가 없어진 구역도 시작 번호로 되돌리므로, 이어 세는 구역은 5부터다.
    delete_first_endnote(&mut doc, 0);
    assert_eq!(numbers(&doc), [vec![], vec![5, 6]]);
}

#[test]
fn new_number_control_sets_next_endnote() {
    let mut doc = open("new-number-20.hwpx");
    doc.apply_endnote_shape_native(0, "{}").unwrap();
    assert_eq!(numbers(&doc), [vec![], vec![20, 21]]);

    delete_first_endnote(&mut doc, 1);
    assert_eq!(numbers(&doc), [vec![], vec![20]]);
}
