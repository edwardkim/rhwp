//! 관계형: 수식이 든 셀 문단의 논리 캐럿(글자와 글자처럼 취급한 개체가 한 칸씩) 복사·붙이기.
//! 입력은 메모리에서 생성한 API 계약이며 독립 Print의 실물 px 좌표나 golden을 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use serde_json::Value;

fn ok(text: &str) -> Value {
    let value: Value = serde_json::from_str(text).unwrap();
    assert_eq!(value["ok"], true, "{value}");
    value
}

/// 1×2 표의 첫 칸에 `가[수식]나`, 둘째 칸에 `다라`를 둔다. 표 문단 번호와 표 컨트롤 번호를 준다.
fn fixture() -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나").unwrap();
    ok(&core
        .insert_equation_native(0, 0, 1, "a over b", 1000, 0)
        .unwrap());
    ok(&core.copy_selection_logical_native(0, 0, 0, 0, 3).unwrap());
    ok(&core.split_paragraph_native(0, 0, 2, None).unwrap());
    let table = ok(&core.create_table_native(0, 1, 0, 1, 2).unwrap());
    let para = table["paraIdx"].as_u64().unwrap() as usize;
    let ctrl = table["controlIdx"].as_u64().unwrap() as usize;
    ok(&core
        .paste_internal_in_cell_by_path_native(0, para, &[(ctrl, 0, 0)], 0)
        .unwrap());
    ok(&core
        .insert_text_in_cell_native(0, para, ctrl, 1, 0, 0, "다라")
        .unwrap());
    assert_eq!(cell(&core, para, ctrl, 0), "가[eq]나");
    assert_eq!(cell(&core, para, ctrl, 1), "다라");
    (core, para, ctrl)
}

/// 셀 첫 문단의 글 사이에 수식은 `[eq]`, 그 밖의 개체는 `[ob]`로 끼워 읽는다.
fn cell(core: &DocumentCore, para: usize, ctrl: usize, index: usize) -> String {
    let Control::Table(table) = &core.document().sections[0].paragraphs[para].controls[ctrl] else {
        panic!("표가 아닙니다");
    };
    marked(&table.cells[index].paragraphs[0])
}

fn marked(paragraph: &Paragraph) -> String {
    let positions = paragraph.control_text_positions();
    let chars: Vec<char> = paragraph.text.chars().collect();
    let mut out = String::new();
    for at in 0..=chars.len() {
        for (control, &position) in paragraph.controls.iter().zip(&positions) {
            if position == at {
                out.push_str(match control {
                    Control::Equation(_) => "[eq]",
                    _ => "[ob]",
                });
            }
        }
        if let Some(character) = chars.get(at) {
            out.push(*character);
        }
    }
    out
}

#[test]
fn logical_cell_copy_splits_at_both_sides_of_inline_equation() {
    // 가(0) [수식](1) 나(2) 끝(3). 한/글 12.30도 수식 뒤에서 시작한 셀 선택은 수식을 담지 않는다.
    for (from, to, expected) in [
        (0, 1, "가다라"),
        (1, 2, "[eq]다라"),
        (2, 3, "나다라"),
        (0, 2, "가[eq]다라"),
        (1, 3, "[eq]나다라"),
        (0, 3, "가[eq]나다라"),
    ] {
        let (mut core, para, ctrl) = fixture();
        ok(&core
            .copy_selection_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 0, from, 0, to)
            .unwrap());
        ok(&core
            .paste_internal_in_cell_by_path_logical_native(0, para, &[(ctrl, 1, 0)], 0)
            .unwrap());
        assert_eq!(cell(&core, para, ctrl, 1), expected, "{from}..{to}");
        assert_eq!(cell(&core, para, ctrl, 0), "가[eq]나");
    }
}

#[test]
fn logical_cell_paste_lands_after_inline_equation() {
    // 글 조각·개체 조각·HTML 모두 수식 바로 뒤(논리 2)에 붙인다.
    let (mut core, para, ctrl) = fixture();
    ok(&core
        .copy_selection_in_cell_by_path_logical_native(0, para, &[(ctrl, 1, 0)], 0, 0, 0, 1)
        .unwrap());
    let result = ok(&core
        .paste_internal_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 2)
        .unwrap());
    assert_eq!(cell(&core, para, ctrl, 0), "가[eq]다나");
    // 돌려주는 charOffset은 다른 셀 붙이기처럼 글자 자리다.
    assert_eq!(result["charOffset"], 2);

    let (mut core, para, ctrl) = fixture();
    ok(&core
        .copy_selection_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 0, 1, 0, 2)
        .unwrap());
    ok(&core
        .paste_internal_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 2)
        .unwrap());
    assert_eq!(cell(&core, para, ctrl, 0), "가[eq][eq]나");

    let (mut core, para, ctrl) = fixture();
    ok(&core
        .paste_html_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 2, "<p>마</p>")
        .unwrap());
    assert_eq!(cell(&core, para, ctrl, 0), "가[eq]마나");
    ok(&core
        .paste_html_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 1, "<p>바</p>")
        .unwrap());
    assert_eq!(cell(&core, para, ctrl, 0), "가바[eq]마나");
}

#[test]
fn logical_cell_clipboard_rejects_offsets_past_paragraph() {
    let (mut core, para, ctrl) = fixture();
    assert!(core
        .copy_selection_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 0, 0, 0, 4)
        .is_err());
    ok(&core
        .copy_selection_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 0, 0, 0, 1)
        .unwrap());
    assert!(core
        .paste_internal_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 4)
        .is_err());
    assert!(core
        .paste_html_in_cell_by_path_logical_native(0, para, &[(ctrl, 0, 0)], 4, "<p>마</p>")
        .is_err());
    assert_eq!(cell(&core, para, ctrl, 0), "가[eq]나");
}
