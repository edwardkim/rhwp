//! 본문 표와 머리말 표가 같은 (문단, 컨트롤) 번호를 가질 때, 본문 칸의 캐럿·선택·칸 상자 조회는
//! 본문 표에서 찾고 머리말 조회는 머리말 표에서 찾는다. 머리말 표의 칸 글은 머리말 문단 번호를
//! 부모 문단으로 가져 렌더 트리에서 같은 번호의 본문 표 칸과 주소가 겹친다. 전에는 쪽 트리에서
//! 먼저 나오는 머리말 표를 잡아 본문 칸 캐럿을 머리말에 그렸다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;

fn json(text: &str) -> serde_json::Value {
    serde_json::from_str(text).unwrap()
}

#[derive(Debug, Clone, Copy)]
struct Area {
    y: f64,
    bottom: f64,
}

fn boxes(text: &str) -> Vec<Area> {
    json(text)
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            let y = b["y"].as_f64().unwrap();
            Area {
                y,
                bottom: y + b["h"].as_f64().unwrap(),
            }
        })
        .collect()
}

fn inside(y: f64, area: Area) {
    assert!(
        y >= area.y - 0.5 && y <= area.bottom + 0.5,
        "{y} 은 {area:?} 안"
    );
}

fn path_json(path: &[(usize, usize, usize)]) -> String {
    serde_json::to_string(
        &path
            .iter()
            .map(|&(c, cell, p)| {
                serde_json::json!({"controlIndex": c, "cellIndex": cell, "cellParaIndex": p})
            })
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

/// 본문 "첫 줄" 뒤 1×2 표와, 같은 문단·컨트롤 번호의 머리말 1×2 표. 두 표의 둘째 칸에는 1×1 표가 있다.
/// (본문 표 문단, 표 컨트롤, 머리말을 품은 본문 문단, 머리말 컨트롤)을 돌려준다.
fn doc_with_same_address_tables() -> (HwpDocument, usize, usize, usize, usize) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.insert_text_native(0, 0, 0, "첫 줄").unwrap();
    let created = json(&doc.create_table_native(0, 0, 3, 1, 2).unwrap());
    let para = created["paraIdx"].as_u64().unwrap() as usize;
    let control = created["controlIdx"].as_u64().unwrap_or(0) as usize;
    doc.insert_text_in_cell_native(0, para, control, 0, 0, 0, "bp")
        .unwrap();
    doc.create_table_in_cell_by_path_native(
        0,
        para,
        &[(control, 1, 0)],
        0,
        1,
        1,
        None,
        None,
        false,
    )
    .unwrap();

    doc.create_header_footer_native(0, true, 0).unwrap();
    let info = json(&doc.get_header_footer_native(0, true, 0).unwrap());
    let host = info["paraIndex"].as_u64().unwrap() as usize;
    let hf = info["controlIndex"].as_u64().unwrap() as usize;
    for _ in 0..para {
        doc.split_paragraph_in_header_footer_native(0, true, 0, 0, 0, None)
            .unwrap();
    }
    let header = json(
        &doc.create_table_in_cell_by_path_native(
            0,
            host,
            &[(hf, 0, para)],
            0,
            1,
            2,
            None,
            None,
            false,
        )
        .unwrap(),
    );
    // 머리말 표도 같은 문단·컨트롤 번호다.
    assert_eq!(
        header["cellPath"][1]["controlIndex"].as_u64(),
        Some(control as u64)
    );
    let cell = |index: usize| vec![(hf, 0, para), (control, index, 0)];
    doc.insert_text_in_cell_by_path(0, host, &cell(0), 0, "hp")
        .unwrap();
    doc.create_table_in_cell_by_path_native(0, host, &cell(1), 0, 1, 1, None, None, false)
        .unwrap();
    (doc, para, control, host, hf)
}

/// 머리말 경로 캐럿 줄의 (위, 아래). 머리말 경로 조회는 그 머리말 노드 안에서만 찾는다.
fn header_caret(doc: &HwpDocument, para: usize, control: usize, host: usize, hf: usize) -> Area {
    let caret = json(
        &doc.get_cursor_rect_by_path(
            0,
            host as u32,
            &path_json(&[(hf, 0, para), (control, 0, 0)]),
            1,
        )
        .unwrap(),
    );
    let y = caret["y"].as_f64().unwrap();
    Area {
        y,
        bottom: y + caret["height"].as_f64().unwrap(),
    }
}

#[test]
fn body_cell_caret_selection_and_boxes_stay_in_body_table() {
    let (doc, para, control, host, hf) = doc_with_same_address_tables();
    let header = header_caret(&doc, para, control, host, hf);
    let body = boxes(
        &doc.get_table_cell_bboxes(0, para as u32, control as u32, None)
            .unwrap(),
    );
    assert_eq!(body.len(), 2);
    // 본문 표 칸은 머리말 표 아래에 있다.
    for cell in &body {
        assert!(
            cell.y >= header.bottom,
            "본문 칸 {cell:?} / 머리말 {header:?}"
        );
    }

    let caret = json(
        &doc.get_cursor_rect_in_cell_native(0, para, control, 0, 0, 1)
            .unwrap(),
    );
    inside(caret["y"].as_f64().unwrap(), body[0]);

    let selection = json(
        &doc.get_selection_rects_in_cell(0, para as u32, control as u32, 0, 0, 0, 0, 2)
            .unwrap(),
    );
    let selection = selection.as_array().unwrap();
    assert!(!selection.is_empty());
    for rect in selection {
        inside(rect["y"].as_f64().unwrap(), body[0]);
    }

    // 둘째 칸의 1×1 표 칸(깊이 2)도 본문 표에서 찾는다.
    let nested = json(
        &doc.get_cursor_rect_by_path(0, para as u32, &path_json(&[(control, 1, 0), (0, 0, 0)]), 0)
            .unwrap(),
    );
    inside(nested["y"].as_f64().unwrap(), body[1]);
    let nested_boxes = boxes(
        &doc.get_table_cell_bboxes_by_path(
            0,
            para as u32,
            &path_json(&[(control, 1, 0), (0, 0, 0)]),
        )
        .unwrap(),
    );
    assert_eq!(nested_boxes.len(), 1);
    inside(nested_boxes[0].y, body[1]);
}

#[test]
fn header_cell_caret_stays_in_header_table() {
    let (doc, para, control, host, hf) = doc_with_same_address_tables();
    let header = header_caret(&doc, para, control, host, hf);
    let body = boxes(
        &doc.get_table_cell_bboxes(0, para as u32, control as u32, None)
            .unwrap(),
    );
    // 머리말 칸 캐럿은 본문 표 위, 머리말에 있다.
    assert!(
        header.bottom <= body[0].y,
        "머리말 {header:?} / 본문 {body:?}"
    );
}
