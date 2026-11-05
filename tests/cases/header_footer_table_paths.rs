//! 관계형 회귀: 머리말·꼬리말 셀 경로의 편집·원본 속성·내용 보존. 원본 HWPUNIT 폭 계약은 절대 배치 핀이 아니다.
//! 주석 갱신: 2026-11-06T08:25:01+09:00 (Asia/Seoul).
//! 직전 파일 변경: 5172f2d55c73b2ed33d4d3657d81e769bc9004c2, 작성 2026-11-06T08:13:58+09:00, 커밋 2026-11-06T08:13:58+09:00.
//! 측정일: 2026-11-06 KST. 제품 source 86709ac16711820e61baa3b87e2c4ddaac23573e.
//! Print 프로필·96 DPI·PDFium·색상 경계 보정 전 2px raw. 독립 PDF와 전체 쪽수가 같다.
//! 대응 실문서·API 저장본의 전쪽 결과이며 입력 개입·합성 분기의 전수 시각 측정으로 주장하지 않는다.
//! header-table.hwp (api-8571-header-table-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-table.hwpx (api-8571-header-table-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! non-inline-header-table.hwp (api-8571-non-inline-header-table-hwp): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! non-inline-header-table.hwpx (api-8571-non-inline-header-table-hwpx): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-tac-explicit-break.hwpx (header-tac-explicit-break): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-tac-mixed-text.hwpx (header-tac-mixed-text): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-tac-pair-fit.hwpx (header-tac-pair-fit): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-tac-pair-wrap.hwpx (header-tac-pair-wrap): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-stale.hwpx (api-8571-height-contract-header-stale): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-near.hwpx (api-8571-height-contract-header-near): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! header-grown.hwpx (api-8571-height-contract-header-grown): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! footer-stale.hwpx (api-8571-height-contract-footer-stale): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! footer-near-complete.hwpx (api-8571-height-contract-footer-near-complete): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! footer-grown.hwpx (api-8571-height-contract-footer-grown): 전 1쪽, Native 최저 100.0%, fresh WASM 최저 100.0%.
//! 원본·PDF SHA, 명령·원본별 전쪽 TSV·정밀 raw·대표 PNG:
//! mydocs/pr/assets/semanticist21_20261105/validation-r6/README.md 및 measurements.md.
//! 원본 형식 계약:
//! 관계형 및 고정형 회귀: 셀 경로의 머리말·꼬리말 뿌리로 머리말·꼬리말 문단에 표를 만들고,
//! 그 셀을 고치고, 머리말 문단에 붙이고, 표를 지운다.
//! 주석 갱신: 2026-11-03T13:40:00+09:00 (Asia/Seoul). 절대 px 위치를 고정하지 않는다.
//!
//! 셀 경로의 첫 마디가 머리말·꼬리말 컨트롤이면 글상자처럼 셀 0 하나로 지나간다:
//! `[(머리말 컨트롤, 0, 머리말 문단), (표 컨트롤, 셀, 셀 문단)]`. 뿌리 문단은 머리말 컨트롤이 든
//! 본문 문단이다(`getHeaderFooter`의 `paraIndex`·`controlIndex`).
//!
//! 기대값은 한/글 12.30(Mac)이 만든 문서에서 정했다. 머리말의 `hp,hq`를 골라 표 > 표 > 문자열을
//! 표로(쉼표)를 실행해 HWPX로 저장하자 머리말 안에 1×2 표가 생겼다. 표는 `vertRelTo="PARA"`,
//! `horzRelTo="COLUMN"`이고, 폭은 머리말 글 영역 42,520 − 바깥 여백 566 = 41,954(칸마다 20,977)였다.
//! 한/글의 그 표는 글자처럼 취급하지 않는 표다. 칸 경로 표 만들기는 지금 글자처럼 취급한 표만
//! 만들므로 여기서는 자리·폭과 경로만 본다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::{HorzRelTo, TextWrap, VertRelTo};
use rhwp::model::table::Table;
use rhwp::wasm_api::HwpDocument;

type Path = Vec<(usize, usize, usize)>;

/// 빈 문서에 양쪽 머리말을 만들고 머리말 컨트롤의 (본문 문단, 컨트롤)을 돌려준다.
fn doc_with_header() -> (HwpDocument, usize, usize) {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document().unwrap();
    doc.create_header_footer_native(0, true, 0).unwrap();
    let info: serde_json::Value =
        serde_json::from_str(&doc.get_header_footer_native(0, true, 0).unwrap()).unwrap();
    let host = info["paraIndex"].as_u64().unwrap() as usize;
    let control = info["controlIndex"].as_u64().unwrap() as usize;
    (doc, host, control)
}

fn header_paragraphs(doc: &HwpDocument, host: usize, control: usize) -> &[Paragraph] {
    match &doc.document().sections[0].paragraphs[host].controls[control] {
        Control::Header(header) => &header.paragraphs,
        _ => panic!("머리말이 아니다"),
    }
}

fn header_table(doc: &HwpDocument, host: usize, control: usize, para: usize) -> &Table {
    match header_paragraphs(doc, host, control)[para].controls.first() {
        Some(Control::Table(t)) => t,
        _ => panic!("머리말 문단 {para} 에 표가 없다"),
    }
}

fn cell(hf: usize, para: usize, cell: usize, cell_para: usize) -> Path {
    vec![(hf, 0, para), (0, cell, cell_para)]
}

/// 머리말 첫 문단에 1×2 표를 만들고 칸에 `hp`·`hq`를 넣는다.
fn doc_with_header_table() -> (HwpDocument, usize, usize) {
    let (mut doc, host, hf) = doc_with_header();
    doc.create_table_in_cell_by_path_native(0, host, &[(hf, 0, 0)], 0, 1, 2, None, None, true)
        .unwrap();
    doc.insert_text_in_cell_by_path(0, host, &cell(hf, 0, 0, 0), 0, "hp")
        .unwrap();
    doc.insert_text_in_cell_by_path(0, host, &cell(hf, 0, 1, 0), 0, "hq")
        .unwrap();
    (doc, host, hf)
}

fn cell_text(doc: &HwpDocument, path: &[(usize, usize, usize)], host: usize) -> String {
    doc.get_text_in_cell_by_path(0, host, path, 0, usize::MAX / 2)
        .unwrap()
}

#[test]
fn create_table_in_header_paragraph_matches_hancom() {
    let (mut doc, host, hf) = doc_with_header();
    let page = &doc.document().sections[0].section_def.page_def;
    let text_width = page.width - page.margin_left - page.margin_right;
    assert_eq!(text_width, 42_520, "한/글 새 문서와 같은 머리말 글 영역 폭");

    let json = doc
        .create_table_in_cell_by_path_native(0, host, &[(hf, 0, 0)], 0, 1, 2, None, None, true)
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        v["cellPath"],
        serde_json::json!([
            {"controlIndex": hf, "cellIndex": 0, "cellParaIndex": 0},
            {"controlIndex": 0, "cellIndex": 0, "cellParaIndex": 0}
        ]),
        "캐럿이 갈 머리말 표 첫 칸까지의 경로"
    );

    let paragraphs = header_paragraphs(&doc, host, hf);
    assert_eq!(paragraphs.len(), 2, "표 문단 + 뒤 빈 문단");
    let table = header_table(&doc, host, hf, 0);
    assert_eq!(table.common.text_wrap, TextWrap::TopAndBottom);
    assert_eq!(table.common.vert_rel_to, VertRelTo::Para);
    assert_eq!(table.common.horz_rel_to, HorzRelTo::Column);
    assert_eq!(table.common.width, 41_954, "한/글과 같은 표 폭");
    assert!(table.cells.iter().all(|c| c.width == 20_977), "칸 폭");
    assert_eq!(
        doc.document().sections[0].paragraphs.len(),
        1,
        "본문 문단은 그대로"
    );
}

#[test]
fn header_table_cells_are_edited_by_path() {
    let (mut doc, host, hf) = doc_with_header_table();
    assert_eq!(cell_text(&doc, &cell(hf, 0, 0, 0), host), "hp");
    assert_eq!(cell_text(&doc, &cell(hf, 0, 1, 0), host), "hq");

    doc.delete_text_in_cell_by_path(0, host, &cell(hf, 0, 0, 0), 1, 1)
        .unwrap();
    doc.split_paragraph_in_cell_by_path(0, host, &cell(hf, 0, 1, 0), 1, None)
        .unwrap();
    assert_eq!(cell_text(&doc, &cell(hf, 0, 0, 0), host), "h");
    assert_eq!(cell_text(&doc, &cell(hf, 0, 1, 0), host), "h");
    assert_eq!(cell_text(&doc, &cell(hf, 0, 1, 1), host), "q");
    doc.merge_paragraph_in_cell_by_path(0, host, &cell(hf, 0, 1, 1))
        .unwrap();
    assert_eq!(cell_text(&doc, &cell(hf, 0, 1, 0), host), "hq");

    // 본문 문단 0 에는 표가 없다 — 머리말 표 경로가 본문으로 새지 않는다.
    assert!(doc
        .insert_text_in_cell_by_path(0, host, &[(0, 0, 0)], 0, "x")
        .is_err());
    // 머리말 마디의 셀 번호는 0 뿐이다.
    assert!(doc
        .insert_text_in_cell_by_path(0, host, &[(hf, 1, 0), (0, 0, 0)], 0, "x")
        .is_err());
}

#[test]
fn header_paragraph_takes_pasted_cells_and_loses_its_table() {
    let (mut doc, host, hf) = doc_with_header_table();
    // 둘째 칸 글을 머리말의 표 뒤 빈 문단에 붙이고, 표를 지운다(표를 문자열로).
    doc.copy_selection_in_cell_by_path_native(0, host, &cell(hf, 0, 1, 0), 0, 0, 0, 2)
        .unwrap();
    let pasted: serde_json::Value = serde_json::from_str(
        &doc.paste_internal_in_cell_by_path_native(0, host, &[(hf, 0, 1)], 0)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(pasted["ok"], true);
    doc.delete_cell_table_control_by_path_native(
        0,
        host,
        &format!(r#"[{{"controlIndex":{hf},"cellIndex":0,"cellParaIndex":0}}]"#),
        0,
    )
    .unwrap();

    let texts: Vec<String> = header_paragraphs(&doc, host, hf)
        .iter()
        .map(|p| p.text.clone())
        .collect();
    assert_eq!(texts, vec!["", "hq"]);
    assert!(header_paragraphs(&doc, host, hf)
        .iter()
        .all(|p| p.controls.is_empty()));
}

#[test]
fn header_table_is_drawn_in_the_header_and_survives_save() {
    use serde_json::Value;

    fn texts(node: &Value, out: &mut Vec<String>) {
        if node["type"] == "TextRun" {
            if let Some(t) = node["text"].as_str() {
                out.push(t.to_string());
            }
        }
        for child in node["children"].as_array().into_iter().flatten() {
            texts(child, out);
        }
    }

    let (doc, host, hf) = doc_with_header_table();
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let header = tree["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Header")
        .expect("머리말 노드");
    let mut found = Vec::new();
    texts(header, &mut found);
    assert!(
        found.contains(&"hp".to_string()) && found.contains(&"hq".to_string()),
        "머리말 표 칸 글: {found:?}"
    );

    for (label, bytes) in [
        ("hwp", doc.export_hwp_native().unwrap()),
        ("hwpx", doc.export_hwpx_native().unwrap()),
    ] {
        let reloaded =
            HwpDocument::new(&bytes).unwrap_or_else(|_| panic!("{label} 다시 읽기 실패"));
        let table = header_table(&reloaded, host, hf, 0);
        assert_eq!(table.common.width, 41_954, "{label}: 표 폭");
        let cells: Vec<&str> = table
            .cells
            .iter()
            .map(|c| c.paragraphs[0].text.as_str())
            .collect();
        assert_eq!(cells, vec!["hp", "hq"], "{label}: 칸 글");
    }
}

/// 머리말 노드 안 표 칸(줄·칸 번호)의 상자 (x, y, w, h).
fn header_cell_box(doc: &HwpDocument, col: u64) -> (f64, f64, f64, f64) {
    use serde_json::Value;
    fn find<'a>(node: &'a Value, col: u64) -> Option<&'a Value> {
        if node["type"] == "Cell" && node["row"] == 0 && node["col"] == col {
            return Some(node);
        }
        node["children"]
            .as_array()?
            .iter()
            .find_map(|n| find(n, col))
    }
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let header = tree["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Header")
        .expect("머리말 노드");
    let cell = find(header, col).expect("머리말 표 칸");
    let b = |k: &str| cell["bbox"][k].as_f64().unwrap();
    (b("x"), b("y"), b("w"), b("h"))
}

#[test]
fn header_table_cell_is_hit_and_has_a_caret() {
    let (doc, host, hf) = doc_with_header_table();
    let (x, y, w, h) = header_cell_box(&doc, 1);
    // 둘째 칸의 글(hq) 오른쪽 빈 곳을 누르면 그 칸 글 끝이다.
    let hit: serde_json::Value = serde_json::from_str(
        &doc.hit_test_in_header_footer_target_native(0, 0, true, 0, x + w - 2.0, y + h / 2.0)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(hit["hit"], true);
    assert_eq!(hit["parentParaIndex"], host as u64);
    assert_eq!(
        hit["cellPath"],
        serde_json::json!([
            {"controlIndex": hf, "cellIndex": 0, "cellParaIndex": 0},
            {"controlIndex": 0, "cellIndex": 1, "cellParaIndex": 0}
        ])
    );
    assert_eq!(
        (hit["paraIndex"].as_u64(), hit["charOffset"].as_u64()),
        (Some(0), Some(2))
    );

    let path = format!(
        r#"[{{"controlIndex":{hf},"cellIndex":0,"cellParaIndex":0}},{{"controlIndex":0,"cellIndex":1,"cellParaIndex":0}}]"#
    );
    for rect in [
        doc.get_cursor_rect_by_path(0, host as u32, &path, 1)
            .unwrap(),
        doc.get_cursor_rect_by_path_near(0, host as u32, &path, 1, 0)
            .unwrap(),
    ] {
        let rect: serde_json::Value = serde_json::from_str(&rect).unwrap();
        let (cx, cy) = (rect["x"].as_f64().unwrap(), rect["y"].as_f64().unwrap());
        assert_eq!(rect["pageIndex"], 0);
        assert!(
            cx > x && cx < x + w && cy >= y && cy < y + h,
            "캐럿이 그 칸 안: {rect}"
        );
    }

    // 표 밖 머리말 글은 지금처럼 머리말 문단 자리다.
    let (first_x, first_y, ..) = header_cell_box(&doc, 0);
    let outside: serde_json::Value = serde_json::from_str(
        &doc.hit_test_in_header_footer_target_native(0, 0, true, 0, first_x - 30.0, first_y - 30.0)
            .unwrap(),
    )
    .unwrap();
    assert!(outside.get("cellPath").is_none(), "표 밖: {outside}");
}

#[test]
fn header_paragraph_shape_is_set_by_path() {
    let (mut doc, host, hf) = doc_with_header();
    let shape = doc.document().doc_info.para_shapes.len() as u16 - 1;
    assert_ne!(header_paragraphs(&doc, host, hf)[0].para_shape_id, shape);
    doc.set_cell_para_shape_id_by_path(0, host, &[(hf, 0, 0)], shape)
        .unwrap();
    assert_eq!(header_paragraphs(&doc, host, hf)[0].para_shape_id, shape);
    // 문단 위 간격이 다른 모양도 셀 첫 문단 원점 없이 머리말 문단에 준다.
    let before = header_paragraphs(&doc, host, hf)[0].para_shape_id;
    doc.apply_para_format_in_cell_by_path(0, host, &[(hf, 0, 0)], r#"{"spacingBefore":1000}"#)
        .unwrap();
    let after = header_paragraphs(&doc, host, hf)[0].para_shape_id;
    assert_ne!(after, before);
    assert_eq!(
        doc.document().doc_info.para_shapes[after as usize].spacing_before,
        1000
    );
}
