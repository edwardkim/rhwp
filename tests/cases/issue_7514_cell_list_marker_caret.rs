//! [#7514] 셀 안 번호·글머리표 문단의 캐럿과 선택 영역이 번호 쪽으로 어긋났다.
//!
//! 번호·글머리표는 문서 글자가 아닌 `char_start: None` TextRun 이다. 셀 캐럿·선택
//! 질의가 이 run 의 시작을 0으로 보아, 위치 0부터 번호 글자 수까지 번호 위에 놓였다.

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn json(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|e| panic!("JSON {text}: {e}"))
}

fn assert_close(actual: &Value, expected: f64, label: &str) {
    let actual = actual
        .as_f64()
        .unwrap_or_else(|| panic!("{label}: 값 없음"));
    assert!(
        (actual - expected).abs() <= 0.1,
        "{label}: {actual:.1}, 기대 {expected:.1}"
    );
}

struct ListCell {
    doc: HwpDocument,
    parent_para: usize,
    control: usize,
    body_para: usize,
}

/// 1×2 표 첫 셀과 표 뒤 본문 문단에 `한한첫`을 넣고 같은 머리 모양을 준다.
fn list_cell(head: &str) -> ListCell {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().expect("빈 문서");
    let id = match head {
        "Bullet" => doc.ensure_default_bullet("●"),
        _ => doc.ensure_default_numbering(),
    };
    let created = json(&doc.create_table_native(0, 0, 0, 1, 2).expect("1×2 표"));
    let parent_para = created["paraIdx"].as_u64().expect("paraIdx") as usize;
    let control = created["controlIdx"].as_u64().expect("controlIdx") as usize;
    let body_para = parent_para + 1;
    let props = format!(r#"{{"headType":"{head}","paraLevel":0,"numberingId":{id}}}"#);
    doc.insert_text_in_cell_native(0, parent_para, control, 0, 0, 0, "한한첫")
        .expect("셀 글 입력");
    doc.apply_para_format_in_cell_native(0, parent_para, control, 0, 0, &props)
        .expect("셀 머리 모양");
    doc.insert_text_native(0, body_para, 0, "한한첫")
        .expect("본문 글 입력");
    doc.apply_para_format_native(0, body_para, &props)
        .expect("본문 머리 모양");
    ListCell {
        doc,
        parent_para,
        control,
        body_para,
    }
}

/// `한한첫` 각 글자 경계의 x. 번호 run 이 같은 줄에 그려졌는지도 확인한다.
fn glyph_edges(doc: &HwpDocument, in_cell: bool, para: u64) -> Vec<f64> {
    let layout = json(&doc.get_page_text_layout_native(0).expect("텍스트 배치"));
    let runs: Vec<&Value> = layout["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .filter(|run| run.get("cellIdx").is_some() == in_cell)
        .filter(|run| !in_cell || run["cellIdx"].as_u64() == Some(0))
        .collect();
    let text = runs
        .iter()
        .find(|run| run["text"] == "한한첫" && run["paraIdx"].as_u64() == Some(para))
        .unwrap_or_else(|| panic!("본문 run 없음: {runs:?}"));
    let marker = runs
        .iter()
        .find(|run| run.get("charStart").is_none() && run["text"] != "")
        .unwrap_or_else(|| panic!("번호 run 없음: {runs:?}"));
    assert!((marker["y"].as_f64().unwrap() - text["y"].as_f64().unwrap()).abs() <= 1.0);
    assert!(marker["x"].as_f64().unwrap() < text["x"].as_f64().unwrap());
    let x = text["x"].as_f64().unwrap();
    text["charX"]
        .as_array()
        .expect("charX")
        .iter()
        .map(|dx| x + dx.as_f64().unwrap())
        .collect()
}

fn assert_selection(rects: &str, from: f64, to: f64, label: &str) {
    let rects = json(rects);
    let rects = rects.as_array().expect("선택 사각형");
    assert_eq!(rects.len(), 1, "{label}: {rects:?}");
    assert_close(&rects[0]["x"], from, &format!("{label} x"));
    assert_close(&rects[0]["width"], to - from, &format!("{label} 폭"));
}

#[test]
fn cell_list_caret_and_selection_follow_text_not_marker() {
    for head in ["Number", "Bullet"] {
        let ListCell {
            doc,
            parent_para,
            control,
            ..
        } = list_cell(head);
        let edges = glyph_edges(&doc, true, 0);
        let path = format!(r#"[{{"controlIndex":{control},"cellIndex":0,"cellParaIndex":0}}]"#);

        for (offset, &x) in edges.iter().enumerate() {
            let rect = json(
                &doc.get_cursor_rect_in_cell_native(0, parent_para, control, 0, 0, offset)
                    .expect("셀 캐럿"),
            );
            assert_close(
                &rect["x"],
                x,
                &format!("{head} getCursorRectInCell({offset})"),
            );
            let rect = json(
                &doc.get_cursor_rect_by_path(0, parent_para as u32, &path, offset as u32)
                    .expect("경로 캐럿"),
            );
            assert_close(
                &rect["x"],
                x,
                &format!("{head} getCursorRectByPath({offset})"),
            );
        }

        let (pp, ci) = (parent_para as u32, control as u32);
        for (start, end) in [(0, 1), (1, 3)] {
            let label = format!("{head} 셀 선택 {start}..{end}");
            let rects = doc
                .get_selection_rects_in_cell(0, pp, ci, 0, 0, start, 0, end)
                .expect("셀 선택");
            assert_selection(&rects, edges[start as usize], edges[end as usize], &label);
            let rects = doc
                .get_selection_rects_in_cell_by_path(0, pp, &path, 0, start, 0, end)
                .expect("경로 셀 선택");
            assert_selection(&rects, edges[start as usize], edges[end as usize], &label);
        }
    }
}

#[test]
fn body_list_selection_inside_first_line_follows_text_not_marker() {
    for head in ["Number", "Bullet"] {
        let ListCell { doc, body_para, .. } = list_cell(head);
        let edges = glyph_edges(&doc, false, body_para as u64);
        let para = body_para as u32;
        let rects = doc
            .get_selection_rects(0, para, 1, para, 2)
            .expect("본문 선택");
        assert_selection(
            &rects,
            edges[1],
            edges[2],
            &format!("{head} 본문 선택 1..2"),
        );
    }
}
