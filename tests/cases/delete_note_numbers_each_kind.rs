#![cfg(not(target_arch = "wasm32"))]
//! 각주와 미주는 번호를 따로 매긴다. `deleteFootnote`로 주석을 지운 뒤에도 각주는 각주끼리,
//! 미주는 미주끼리 이어지고, 미주는 구역의 미주 시작 번호에서 시작한다. HWP·HWPX로 저장해
//! 다시 열어도 같다.
use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

/// 첫 문단의 (각주 번호, 미주 번호)
fn numbers(doc: &HwpDocument) -> (Vec<u16>, Vec<u16>) {
    let mut footnotes = Vec::new();
    let mut endnotes = Vec::new();
    for ctrl in &doc.document().sections[0].paragraphs[0].controls {
        match ctrl {
            Control::Footnote(note) => footnotes.push(note.number),
            Control::Endnote(note) => endnotes.push(note.number),
            _ => {}
        }
    }
    (footnotes, endnotes)
}

fn assert_numbers(doc: &HwpDocument, footnotes: &[u16], endnotes: &[u16]) {
    let expected = (footnotes.to_vec(), endnotes.to_vec());
    assert_eq!(numbers(doc), expected, "편집한 문서의 (각주, 미주) 번호");
    for (format, bytes) in [
        ("HWP", doc.export_hwp().unwrap()),
        ("HWPX", doc.export_hwpx().unwrap()),
    ] {
        let reopened = HwpDocument::new(&bytes).unwrap();
        assert_eq!(
            numbers(&reopened),
            expected,
            "{format}로 저장해 다시 연 문서의 (각주, 미주) 번호"
        );
    }
}

/// `가나다라마`의 글자 사이에 `kinds` 차례대로 주석을 넣는다. `F`는 각주, `E`는 미주다.
fn doc_with_notes(kinds: &str) -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.insert_text(0, 0, 0, "가나다라마").unwrap();
    for (offset, kind) in (1..).zip(kinds.chars()) {
        match kind {
            'F' => doc.insert_footnote(0, 0, offset).unwrap(),
            _ => doc.insert_endnote(0, 0, offset).unwrap(),
        };
    }
    doc
}

/// 첫 문단에서 `nth`번째 각주(`F`)나 미주(`E`)의 컨트롤 번호
fn note_control(doc: &HwpDocument, kind: char, nth: usize) -> u32 {
    doc.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .enumerate()
        .filter(|(_, ctrl)| match kind {
            'F' => matches!(ctrl, Control::Footnote(_)),
            _ => matches!(ctrl, Control::Endnote(_)),
        })
        .nth(nth)
        .map(|(index, _)| index as u32)
        .unwrap()
}

#[test]
fn deleting_the_first_footnote_makes_the_next_footnote_1() {
    let mut doc = doc_with_notes("FEF");
    assert_numbers(&doc, &[1, 2], &[1]);
    doc.delete_footnote(0, 0, note_control(&doc, 'F', 0))
        .unwrap();
    assert_numbers(&doc, &[1], &[1]);
}

#[test]
fn deleting_one_kind_leaves_the_other_kind_numbers() {
    let mut doc = doc_with_notes("FEFE");
    doc.delete_footnote(0, 0, note_control(&doc, 'F', 1))
        .unwrap();
    assert_numbers(&doc, &[1], &[1, 2]);
    doc.delete_footnote(0, 0, note_control(&doc, 'E', 0))
        .unwrap();
    assert_numbers(&doc, &[1], &[1]);
}

#[test]
fn deleting_a_footnote_keeps_the_endnote_start_number() {
    let mut doc = doc_with_notes("");
    doc.apply_endnote_shape(0, r#"{"startNumber":3}"#).unwrap();
    doc.insert_endnote(0, 0, 1).unwrap();
    doc.insert_footnote(0, 0, 2).unwrap();
    doc.delete_footnote(0, 0, note_control(&doc, 'F', 0))
        .unwrap();
    assert_numbers(&doc, &[], &[3]);
}
