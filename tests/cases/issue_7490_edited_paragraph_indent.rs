//! [#7490] 편집한 문단에도 들여쓰기·내어쓰기가 그려져야 한다.
//!
//! 렌더러는 저장 줄의 `TAG_INDENTATION`(bit 20)이 꺼져 있으면 그 줄에 문단
//! `indent` 를 얹지 않는다(#6190). 편집 재조판이 이 비트를 비운 채 줄을 발행하면
//! 문단 모양에 저장된 들여쓰기·내어쓰기가 화면에서 사라진다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::paragraph::{LineSeg, ParaMeta};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const TEXT: &str = "1) 가나다라마바사아자차카타파하 가나다라마바사아자차카타파하 \
가나다라마바사아자차카타파하 가나다라마바사아자차카타파하 가나다라마바사아자차카타파하";

/// 새 문서 첫 문단에 두 줄 넘게 입력하고 문단 모양 `indent` 를 준 뒤의 줄별 시작 x.
fn typed_line_starts(indent: i32) -> Vec<f64> {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank document");
    doc.insert_text_native(0, 0, 0, TEXT).expect("insert text");
    doc.apply_para_format_native(0, 0, &format!(r#"{{"indent":{indent}}}"#))
        .expect("apply indent");
    let starts = line_starts(&doc, 0);
    assert!(starts.len() >= 2, "두 줄 넘게 입력해야 한다: {starts:?}");
    starts
}

#[test]
fn hanging_indent_moves_following_lines_of_typed_paragraph() {
    let flat = typed_line_starts(0);
    let hanging = typed_line_starts(-3000);
    assert!(
        (hanging[0] - flat[0]).abs() < 0.5,
        "내어쓰기는 첫 줄을 옮기지 않는다: {hanging:?}"
    );
    assert!(
        hanging[1] - flat[1] > 5.0,
        "내어쓰기는 둘째 줄부터 오른쪽으로 민다 — 없음 {flat:?}, 내어쓰기 {hanging:?}"
    );
}

#[test]
fn first_line_indent_moves_first_line_of_typed_paragraph() {
    let flat = typed_line_starts(0);
    let indented = typed_line_starts(3000);
    assert!(
        indented[0] - flat[0] > 5.0,
        "들여쓰기는 첫 줄을 오른쪽으로 민다 — 없음 {flat:?}, 들여쓰기 {indented:?}"
    );
    assert!(
        (indented[1] - flat[1]).abs() < 0.5,
        "들여쓰기는 둘째 줄을 옮기지 않는다: {indented:?}"
    );
}

#[test]
fn editing_keeps_stored_hanging_indent() {
    // biz_plan.hwp 문단 51: indent=-7284, 저장 줄 tag 0x60000 / 0x160000.
    const PARA: usize = 51;
    let mut doc = open("samples/biz_plan.hwp");
    let before = line_starts(&doc, PARA);
    assert!(
        before.len() >= 2 && before[1] - before[0] > 5.0,
        "원본 내어쓰기: {before:?}"
    );

    // 문단 끝(67자 뒤)에 한 글자를 넣는다.
    doc.insert_text_native(0, PARA, 67, "가")
        .expect("insert text");
    let after = line_starts(&doc, PARA);
    assert!(
        after.len() == before.len() && after.iter().zip(&before).all(|(a, b)| (a - b).abs() < 0.5),
        "글자를 넣어도 내어쓰기는 그대로다 — 편집 전 {before:?}, 편집 후 {after:?}"
    );
}

#[test]
fn editing_keeps_hancom_record_of_unindented_line() {
    // #6190 표본: 문단 3~7 은 indent=20445 인데 한글이 bit 20 을 꺼 둔 가운데 정렬 문단이다.
    // 편집으로 비트를 새로 켜면 줄이 indent/2(68.1px) 밀리고, 문단 7 이 호스트하는 표가
    // 용지 밖으로 나간다.
    const HEADING: usize = 4; // `경 력 사 항`
    const TABLE_HOST: usize = 7;
    const BODY_RIGHT_PX: f64 = 699.2;
    const BODY_WIDTH_PX: f64 = 604.7;
    let mut doc = open("samples/issue6190/center_align_first_line_indent.hwp");
    let before = line_starts(&doc, HEADING);

    doc.insert_text_native(0, HEADING, 7, "가")
        .expect("insert heading");
    let after = line_starts(&doc, HEADING);
    assert!(
        after.len() == 1 && after[0] <= before[0] + 0.5,
        "글자를 넣어도 한글이 들여쓰지 않은 줄은 밀리지 않는다 — 편집 전 {before:?}, 편집 후 {after:?}"
    );

    doc.insert_text_native(0, TABLE_HOST, 0, "가")
        .expect("insert table host");
    let mut tables = Vec::new();
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("render tree");
        collect_tables(&tree.root, &mut tables);
    }
    let escaping: Vec<_> = tables
        .iter()
        .filter(|(x, w)| *w <= BODY_WIDTH_PX + 1.0 && x + w > BODY_RIGHT_PX + 1.0)
        .map(|(x, w)| format!("x={x:.1} 우변={:.1}", x + w))
        .collect();
    assert!(
        escaping.is_empty(),
        "표 호스트 문단을 편집해도 본문(우단 {BODY_RIGHT_PX})에 들어가는 표가 밀려나지 않는다: {escaping:?}"
    );
}

#[test]
fn applying_indent_to_hancom_paragraph_draws_it() {
    // #6190 표본의 `학 력 사 항`(indent 0, 가운데 정렬, bit 20 꺼짐)에 새로 들여쓰기를 준다.
    const HEADING: usize = 1;
    let mut doc = open("samples/issue6190/center_align_first_line_indent.hwp");
    let before = line_starts(&doc, HEADING);

    doc.apply_para_format_native(0, HEADING, r#"{"indent":20445}"#)
        .expect("apply indent");
    let after = line_starts(&doc, HEADING);
    assert!(
        after.len() == 1 && after[0] - before[0] > 60.0,
        "새로 준 들여쓰기는 그려진다(가운데 정렬이라 indent/2 = 68.1px) — \
         적용 전 {before:?}, 적용 후 {after:?}"
    );
}

#[test]
fn editing_list_paragraph_keeps_hancom_continuation_bits() {
    // tac-img-02.hwp 문단 37: 글머리표, indent 0, 저장 줄 tag 0x260000 / 0x160000.
    // 한글은 문단 머리 문단의 둘째 줄부터 bit 20 을 켠다.
    const PARA: usize = 37;
    let mut doc = open("samples/tac-img-02.hwp");
    assert_eq!(indentation_bits(&doc, PARA), [false, true], "원본 기록");

    doc.insert_text_native(0, PARA, 50, "가").expect("insert");
    doc.delete_text_native(0, PARA, 50, 1).expect("delete");
    assert_eq!(
        indentation_bits(&doc, PARA),
        [false, true],
        "편집해도 문단 머리 문단의 둘째 줄 기록은 남는다"
    );
}

#[test]
fn indent_on_empty_paragraph_moves_caret_before_typing() {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank document");
    let flat = caret_x(&doc);
    doc.apply_para_format_native(0, 0, r#"{"indent":3000}"#)
        .expect("apply indent");
    let indented = caret_x(&doc);
    assert!(
        indented - flat > 5.0,
        "빈 문단에 준 들여쓰기는 입력 전 캐럿에 반영된다 — 없음 {flat}, 들여쓰기 {indented}"
    );

    doc.insert_text_native(0, 0, 0, "a").expect("insert");
    let typed = caret_x(&doc);
    assert!(
        (typed - indented).abs() < 0.5,
        "첫 글자를 넣어도 캐럿 시작이 튀지 않는다 — 입력 전 {indented}, 입력 후 {typed}"
    );
}

#[test]
fn merge_undo_keeps_indent_of_restored_paragraph() {
    // 병합 undo 는 앞 문단(들여쓰기 0)의 첫 줄 기록을 물려받은 새 문단에 원래 문단
    // 모양(들여쓰기 3000)을 되돌린다.
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank document");
    doc.insert_text_native(0, 0, 0, "가나다")
        .expect("insert first");
    doc.split_paragraph_native(0, 0, 3, None).expect("enter");
    doc.insert_text_native(0, 1, 0, TEXT)
        .expect("insert second");
    doc.apply_para_format_native(0, 1, r#"{"indent":3000}"#)
        .expect("apply indent");
    let before = line_starts(&doc, 1);

    let merged = doc.merge_paragraph_native(0, 1).expect("backspace merge");
    let merged: serde_json::Value = serde_json::from_str(&merged).expect("merge JSON");
    let meta: ParaMeta =
        serde_json::from_value(merged["removedParaMeta"].clone()).expect("removed meta");
    doc.split_paragraph_native(0, 0, 3, Some(meta))
        .expect("undo merge");
    let after = line_starts(&doc, 1);
    assert!(
        after.len() == before.len() && after.iter().zip(&before).all(|(a, b)| (a - b).abs() < 0.5),
        "병합을 되돌리면 들여쓰기도 돌아온다 — 병합 전 {before:?}, 되돌린 뒤 {after:?}"
    );
}

#[test]
fn pasting_into_blank_paragraph_draws_pasted_indent() {
    // 143E433F503322BD33.hwp 문단 9: indent 2000, 저장 줄 bit 20 [켜짐, 꺼짐…].
    // 빈 문단에 붙여넣으면 대상이 이 문단 모양을 물려받는다.
    const PARA: usize = 9;
    let source = open("samples/143E433F503322BD33.hwp");
    let mut foreign = source.document().clone();
    let para = foreign.sections[0].paragraphs[PARA].clone();
    foreign.sections.truncate(1);
    foreign.sections[0].paragraphs = vec![para];

    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank document");
    doc.paste_foreign_document_native(0, 0, 0, foreign)
        .expect("paste");
    let starts = line_starts(&doc, 0);
    assert!(
        starts.len() >= 2 && starts[0] - starts[1] > 5.0,
        "붙여넣은 문단의 첫 줄은 들여쓴다: {starts:?}"
    );
}

#[test]
fn applying_indent_in_cell_marks_first_line() {
    // 새 표의 셀 줄은 bit 20 이 모두 꺼져 있다. 셀 재조판은 이 기록을 읽으므로 문단
    // 모양을 바꿀 때 기록도 새 들여쓰기에 맞춰야 한다.
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("blank document");
    let table = doc
        .create_table_native(0, 0, 0, 1, 1)
        .expect("create table");
    let table: serde_json::Value = serde_json::from_str(&table).expect("table JSON");
    let para = table["paraIdx"].as_u64().expect("paraIdx") as usize;
    let control = table["controlIdx"].as_u64().expect("controlIdx") as usize;
    doc.insert_text_in_cell_native(0, para, control, 0, 0, 0, TEXT)
        .expect("insert cell text");
    doc.apply_para_format_in_cell_native(0, para, control, 0, 0, r#"{"indent":3000}"#)
        .expect("apply cell indent");

    let Control::Table(table) = &doc.document().sections[0].paragraphs[para].controls[control]
    else {
        panic!("표가 아니다");
    };
    let bits: Vec<bool> = table.cells[0].paragraphs[0]
        .line_segs
        .iter()
        .map(|line| line.tag & LineSeg::TAG_INDENTATION != 0)
        .collect();
    assert!(
        bits.len() >= 2 && bits[0] && !bits[1..].iter().any(|&bit| bit),
        "셀 문단의 들여쓰기는 첫 줄에만 적용된다: {bits:?}"
    );
}

fn open(sample: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    HwpDocument::from_bytes(&std::fs::read(path).expect("read sample")).expect("open sample")
}

/// 구역 0 문단 `para` 의 줄마다 `TAG_INDENTATION`(bit 20) 여부.
fn indentation_bits(doc: &HwpDocument, para: usize) -> Vec<bool> {
    doc.document().sections[0].paragraphs[para]
        .line_segs
        .iter()
        .map(|line| line.tag & LineSeg::TAG_INDENTATION != 0)
        .collect()
}

/// 구역 0 문단 0 의 첫 글자 앞 캐럿 x.
fn caret_x(doc: &HwpDocument) -> f64 {
    let json = doc.get_cursor_rect_native(0, 0, 0).expect("cursor rect");
    serde_json::from_str::<serde_json::Value>(&json).expect("cursor JSON")["x"]
        .as_f64()
        .expect("cursor x")
}

/// 모든 표의 `(x, width)`.
fn collect_tables(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push((node.bbox.x, node.bbox.width));
    }
    for child in &node.children {
        collect_tables(child, out);
    }
}

/// 구역 0 문단 `para` 의 줄마다 첫 글자 run 의 x.
fn line_starts(doc: &HwpDocument, para: usize) -> Vec<f64> {
    let mut starts = Vec::new();
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("render tree");
        collect_line_starts(&tree.root, para, &mut starts);
    }
    starts
}

fn collect_line_starts(node: &RenderNode, para: usize, out: &mut Vec<f64>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.section_index == Some(0) && line.para_index == Some(para) {
            if let Some(x) = first_run_x(node) {
                out.push(x);
            }
        }
        return;
    }
    for child in &node.children {
        collect_line_starts(child, para, out);
    }
}

fn first_run_x(node: &RenderNode) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.cell_context.is_none() && !run.text.trim().is_empty() {
            return Some(node.bbox.x);
        }
    }
    node.children.iter().find_map(first_run_x)
}
