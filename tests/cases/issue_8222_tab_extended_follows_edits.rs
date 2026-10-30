//! 관계형: 탭 확장 정보의 소유·순서 보존.
//! 주석 갱신: 2026-10-31T02:04:18+09:00; 직전 파일 커밋: 2026-10-31T01:36:40+09:00 (b8c0ef2e4ba1b827db13b9b86bf355f7e27f5e4f).
//! 합성 Paragraph 및 실제 HWP의 DocumentCore 편집·저장 왕복에서 탭 확장 word의 소유·순서를 검사한다. px 좌표를 고정하지 않는다.
//! 실제 원본·편집본 별도 검증: 2026-10-31, 96 DPI PDFium, 2px raw 실루엣, 각 1쪽.
//! 원본 Native 98.06% / fresh WASM 97.91%; 삽입·삭제 98.05% / 97.89%; 분할 98.10% / 98.00%; 재병합 98.06% / 97.91%.
//! 5개 문서 전쪽 최저 Native 98.05% / fresh WASM 97.89%. 정식 증거: mydocs/pr/assets/chrisryugj_20261031/README.md.
//! 근거: mydocs/pr/pr_8224_review.md.
//!
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

#[test]
fn document_core_edits_keep_serialized_tab_ownership() {
    use rhwp::DocumentCore;

    let mut core =
        DocumentCore::from_bytes(include_bytes!("../../samples/lseg-05-tab.hwp")).unwrap();
    let original = core.document().sections[0].paragraphs[2].clone();
    assert_eq!(
        original.tab_extended.len(),
        2,
        "source must contain both tabs"
    );

    core.insert_text_native(0, 2, 0, "새\t").unwrap();
    let inserted = rhwp::parse_document(&core.export_hwp_with_adapter().unwrap()).unwrap();
    let tabs = &inserted.sections[0].paragraphs[2].tab_extended;
    assert_eq!(tabs[0], TAB_EXT_PLACEHOLDER);
    assert_eq!(&tabs[1..], original.tab_extended.as_slice());

    core.delete_text_native(0, 2, 0, 2).unwrap();
    let split = original.text.chars().position(|c| c == '\t').unwrap() + 1;
    core.split_paragraph_native(0, 2, split, None).unwrap();
    let separated = rhwp::parse_document(&core.export_hwp_with_adapter().unwrap()).unwrap();
    let paragraphs = &separated.sections[0].paragraphs;
    assert_eq!(paragraphs[2].tab_extended, original.tab_extended[..1]);
    assert_eq!(paragraphs[3].tab_extended, original.tab_extended[1..]);
    assert_eq!(
        format!("{}{}", paragraphs[2].text, paragraphs[3].text),
        original.text
    );

    core.merge_paragraph_native(0, 3).unwrap();
    let restored = rhwp::parse_document(&core.export_hwp_with_adapter().unwrap()).unwrap();
    assert_eq!(restored.sections[0].paragraphs[2].text, original.text);
    assert_eq!(
        restored.sections[0].paragraphs[2].tab_extended,
        original.tab_extended
    );
}
