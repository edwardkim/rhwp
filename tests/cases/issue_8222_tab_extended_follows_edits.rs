//! [#8222] 글을 넣고 지우고 나누고 합쳐도 `tab_extended` 는 문단의 탭 글자와 차례로 짝이 맞는다.
//!
//! 직렬화는 탭마다 `tab_extended` 를 차례로 꺼내 쓴다. 짝이 밀리면 기존 탭의 저장 폭(한/글이 쓴
//! 전진폭)이 다른 탭으로 옮겨 가 한/글에서 탭 자리가 틀어진다. 새로 생긴 탭은 자리표
//! (`TAB_EXT_PLACEHOLDER`)여서 조판이 `TabDef` 로 다시 잰다.

use rhwp::model::paragraph::{Paragraph, TAB_EXT_PLACEHOLDER};

const W1: [u16; 7] = [2500, 0, 0x0100, 0, 0, 0, 9];
const W2: [u16; 7] = [4000, 0, 0x0100, 0, 0, 0, 9];

/// 탭 두 개("A\tB\tC")에 저장 폭이 붙은 문단 (탭 하나는 스트림 8 유닛)
fn para_with_tabs() -> Paragraph {
    Paragraph {
        text: "A\tB\tC".to_string(),
        char_count: 20,
        char_offsets: vec![0, 1, 9, 10, 18],
        tab_extended: vec![W1, W2],
        has_para_text: true,
        ..Default::default()
    }
}

#[test]
fn inserted_tab_does_not_take_existing_tab_widths() {
    let mut para = para_with_tabs();
    para.insert_text_at(0, "X\t");
    assert_eq!(para.text, "X\tA\tB\tC");
    assert_eq!(para.tab_extended, vec![TAB_EXT_PLACEHOLDER, W1, W2]);

    let mut para = para_with_tabs();
    para.insert_text_at(3, "\t");
    assert_eq!(para.tab_extended, vec![W1, TAB_EXT_PLACEHOLDER, W2]);
}

#[test]
fn deleted_tab_drops_its_width() {
    let mut para = para_with_tabs();
    para.delete_text_at(1, 1);
    assert_eq!(para.text, "AB\tC");
    assert_eq!(para.tab_extended, vec![W2]);

    let mut para = para_with_tabs();
    para.delete_text_at(2, 1);
    assert_eq!(para.tab_extended, vec![W1, W2], "탭이 없는 범위는 그대로");
}

#[test]
fn split_moves_later_tab_widths_to_new_paragraph() {
    let mut para = para_with_tabs();
    let next = para.split_at(3);
    assert_eq!((para.text.as_str(), next.text.as_str()), ("A\tB", "\tC"));
    assert_eq!(para.tab_extended, vec![W1]);
    assert_eq!(next.tab_extended, vec![W2]);
}

#[test]
fn merge_joins_tab_widths_in_order() {
    let mut first = para_with_tabs();
    let w3 = [1200, 0, 0x0100, 0, 0, 0, 9];
    let second = Paragraph {
        text: "D\tE".to_string(),
        char_count: 11,
        char_offsets: vec![0, 1, 9],
        tab_extended: vec![w3],
        has_para_text: true,
        ..Default::default()
    };
    first.merge_from(&second);
    assert_eq!(first.text, "A\tB\tCD\tE");
    assert_eq!(first.tab_extended, vec![W1, W2, w3]);
}

#[test]
fn paragraph_without_stored_tab_data_stays_without() {
    // 확장 데이터가 없는 문단(새 문서 등)은 계속 없다 — 직렬화가 탭마다 자리표를 쓴다.
    let mut para = Paragraph {
        text: "A".to_string(),
        char_count: 2,
        char_offsets: vec![0],
        has_para_text: true,
        ..Default::default()
    };
    para.insert_text_at(1, "\tB");
    assert!(para.tab_extended.is_empty());
}
