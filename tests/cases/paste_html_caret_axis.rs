//! 본문 HTML 붙이기는 캐럿의 논리 위치를 받고 돌려준다. 논리 위치는 글자와 각주·미주·글자처럼
//! 취급한 개체를 한 칸씩 센다(`getLogicalLength`·`navigateNextEditable` 과 같은 축).
//! 셀 붙이기는 셀 캐럿처럼 글자 위치를 받는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

/// 글자 사이에 각주·수식을 `{fn}`·`{eq}` 로 적는다. 구역·단 정의는 뺀다.
fn tokens(para: &Paragraph) -> String {
    let positions = para.control_text_positions();
    let chars: Vec<char> = para.text.chars().collect();
    let mut out = String::new();
    for i in 0..=chars.len() {
        for (control, _) in para
            .controls
            .iter()
            .zip(&positions)
            .filter(|(_, &at)| at == i)
        {
            match control {
                Control::Footnote(_) => out.push_str("{fn}"),
                Control::Equation(_) => out.push_str("{eq}"),
                _ => {}
            }
        }
        out.extend(chars.get(i));
    }
    out
}

/// HWP·HWPX 로 저장해 다시 연 두 문서.
fn reopened(core: &DocumentCore) -> [DocumentCore; 2] {
    [
        DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap(),
        DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap(),
    ]
}

fn body(core: &DocumentCore) -> Vec<String> {
    core.document().sections[0]
        .paragraphs
        .iter()
        .map(tokens)
        .collect()
}

fn note_doc(text: &str, footnote_at: usize) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, text).unwrap();
    core.insert_footnote_native(0, 0, footnote_at).unwrap();
    core
}

/// 붙인 뒤 문서와 저장본이 모두 `expected` 이고, 돌려준 캐럿이 `caret_after` 다.
fn assert_paste(mut core: DocumentCore, caret: usize, expected: &[&str], caret_after: usize) {
    let result = core
        .paste_html_native(
            0,
            0,
            caret,
            "<!--StartFragment--><p>XY</p><!--EndFragment-->",
        )
        .unwrap();
    assert!(
        result.contains(&format!("\"paraIdx\":0,\"charOffset\":{caret_after}}}")),
        "캐럿 {caret}: {result}"
    );
    assert_eq!(body(&core), expected, "캐럿 {caret}");
    for saved in reopened(&core) {
        assert_eq!(body(&saved), expected, "캐럿 {caret} 저장본");
    }
}

#[test]
fn single_paragraph_paste_lands_at_the_logical_caret_around_a_footnote() {
    // 가나[각주]다라 — 논리 2 는 각주 앞, 3 은 각주 바로 뒤, 5 는 문단 끝이다.
    assert_paste(note_doc("가나다라", 2), 2, &["가나XY{fn}다라"], 4);
    assert_paste(note_doc("가나다라", 2), 3, &["가나{fn}XY다라"], 5);
    assert_paste(note_doc("가나다라", 2), 5, &["가나{fn}다라XY"], 7);
    // 가나다라[각주] — 끝 각주의 앞(4)과 뒤(5).
    assert_paste(note_doc("가나다라", 4), 4, &["가나다라XY{fn}"], 6);
    assert_paste(note_doc("가나다라", 4), 5, &["가나다라{fn}XY"], 7);
}

#[test]
fn single_paragraph_paste_lands_after_an_inline_equation() {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가나다라").unwrap();
    core.insert_equation_native(0, 0, 2, "x+y", 1000, 0)
        .unwrap();
    assert_eq!(body(&core), ["가나{eq}다라"]);
    assert_paste(core, 3, &["가나{eq}XY다라"], 5);
}

#[test]
fn multi_paragraph_paste_splits_at_the_logical_caret_after_a_footnote() {
    let mut core = note_doc("가나다라", 2);
    let result = core.paste_html_native(0, 0, 3, "<p>X</p><p>Y</p>").unwrap();
    assert!(
        result.contains("\"paraIdx\":1,\"charOffset\":1}"),
        "{result}"
    );
    let expected = ["가나{fn}X", "Y다라"];
    assert_eq!(body(&core), expected);
    for saved in reopened(&core) {
        assert_eq!(body(&saved), expected);
    }
}

#[test]
fn cell_paste_of_paragraphs_splits_at_the_text_caret_after_an_inline_equation() {
    // 앞[표] / 가나[수식]다라 — 수식이 든 문단을 표 셀에 붙여 셀 문단을 만든다.
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞").unwrap();
    core.split_paragraph_native(0, 0, 1, None).unwrap();
    core.insert_text_native(0, 1, 0, "가나다라").unwrap();
    core.insert_equation_native(0, 1, 2, "x+y", 1000, 0)
        .unwrap();
    core.create_table_ex_native(0, 0, 1, 1, 1, true, None, None)
        .unwrap();
    let table = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Table(_)))
        .unwrap();
    core.copy_selection_native(0, 1, 0, 1, 4).unwrap();
    core.paste_internal_in_cell_native(0, 0, table, 0, 0, 0)
        .unwrap();
    let cell = |core: &DocumentCore| -> Vec<String> {
        let Control::Table(table) = core.document().sections[0].paragraphs[0]
            .controls
            .iter()
            .find(|control| matches!(control, Control::Table(_)))
            .unwrap()
        else {
            unreachable!()
        };
        table.cells[0].paragraphs.iter().map(tokens).collect()
    };
    assert_eq!(cell(&core), ["가나{eq}다라"]);

    // 셀 캐럿은 글자 위치다 — 3 은 다 뒤, 라 앞.
    let result = core
        .paste_html_in_cell_native(0, 0, table, 0, 0, 3, "<p>X</p><p>Y</p>")
        .unwrap();
    assert!(
        result.contains("\"cellParaIdx\":1,\"charOffset\":1}"),
        "{result}"
    );
    let expected = ["가나{eq}다X", "Y라"];
    assert_eq!(cell(&core), expected);
    for saved in reopened(&core) {
        assert_eq!(cell(&saved), expected);
    }
}

#[test]
fn cell_paste_of_paragraphs_counts_a_char_overlap_before_the_text_caret() {
    // 표 셀 문단 "가나[글자겹침]다라". 글자겹침은 나누기 축에서 한 칸이다.
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "앞").unwrap();
    core.create_table_ex_native(0, 0, 1, 1, 1, true, None, None)
        .unwrap();
    let table = core.document().sections[0].paragraphs[0]
        .controls
        .iter()
        .position(|control| matches!(control, Control::Table(_)))
        .unwrap();
    fn cell_paragraphs(core: &mut DocumentCore, table: usize) -> &mut Vec<Paragraph> {
        let Control::Table(table) =
            &mut core.document_mut().sections[0].paragraphs[0].controls[table]
        else {
            unreachable!()
        };
        &mut table.cells[0].paragraphs
    }
    let para = &mut cell_paragraphs(&mut core, table)[0];
    para.text = "가나다라".into();
    para.char_offsets = vec![0, 1, 10, 11];
    para.controls = vec![Control::CharOverlap(Default::default())];
    para.ctrl_data_records = vec![None];
    para.char_count = 13;
    para.has_para_text = true;

    // 셀 캐럿 3 은 다 뒤, 라 앞이다.
    let result = core
        .paste_html_in_cell_native(0, 0, table, 0, 0, 3, "<p>X</p><p>Y</p>")
        .unwrap();
    assert!(
        result.contains("\"cellParaIdx\":1,\"charOffset\":1}"),
        "{result}"
    );
    let texts: Vec<String> = cell_paragraphs(&mut core, table)
        .iter()
        .map(|para| para.text.clone())
        .collect();
    assert_eq!(texts, ["가나다X", "Y라"]);
}
