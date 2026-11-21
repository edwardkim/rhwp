//! 관계형 검사. 합성 문서의 캐럿 및 표 시작 쪽에서 감추기 컨트롤을 찾고 해당 쪽의 머리말과 활성 필드 소유를 검사한다.
//! 주석 갱신 2026-11-21T15:21:50+09:00. 직전 파일 변경 710f46dcfe9d94de1060f418f6c06806c48bec5e 2026-11-20T13:29:45+09:00.
//! 실물 인쇄 좌표·픽셀을 대상으로 하는 검사는 없다. 실행 근거는 `mydocs/pr/pr_9215_review.md`에 있다.
//!
//! 쪽 감추기를 쪽 단위로 고치기: 앞 쪽에서 이어지는 문단 가운데와 표 셀 안의 감추기.
//!
//! 한/글 12.30 Mac(2026-11-17) 쪽 > 감추기는 앞 쪽에서 이어지는 문단만 있는 쪽에서도 캐럿 자리에
//! 감추기 조판 부호를 넣고, 그 부호가 놓인 줄의 쪽에만 적용한다. 같은 쪽에서 다시 설정하면 그 쪽의
//! 부호를 새 캐럿 자리로 옮겨 하나만 남기고, 모두 끄면 지운다. 나뉜 표의 둘째 쪽 셀에서 설정하면 부호는
//! 그 셀에 들지만 표가 처음 놓인 쪽을 감추고, 창은 그 셀이 놓인 쪽에서 그 값을 읽는다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::model::control::{Control, PageHide};
use rhwp::model::paragraph::Paragraph;
use rhwp::wasm_api::HwpDocument;

const HIDE: PageHide = PageHide {
    hide_header: true,
    hide_footer: false,
    hide_master_page: false,
    hide_border: false,
    hide_fill: false,
    hide_page_num: true,
};

fn blank_with_header() -> HwpDocument {
    let mut doc = HwpDocument::create_empty();
    doc.create_blank_document_native().unwrap();
    doc.create_header_footer_native(0, true, 0).unwrap();
    doc.insert_text_in_header_footer_native(0, true, 0, 0, 0, "HEADERMARK")
        .unwrap();
    doc
}

fn hides(doc: &HwpDocument, page: u32) -> serde_json::Value {
    serde_json::from_str(&doc.get_page_hides_on_page_native(page).unwrap()).unwrap()
}

fn header_drawn(doc: &HwpDocument, page: u32) -> bool {
    let svg = doc.render_page_svg_native(page).unwrap();
    let text: String = svg
        .split("<text")
        .skip(1)
        .filter_map(|t| t.split_once('>').and_then(|(_, r)| r.split_once("</text>")))
        .map(|(t, _)| t)
        .collect();
    text.contains("HEADERMARK")
}

fn page_hide_count(paragraphs: &[Paragraph]) -> usize {
    paragraphs
        .iter()
        .flat_map(|p| &p.controls)
        .map(|c| match c {
            Control::PageHide(_) => 1,
            Control::Table(t) => t.cells.iter().map(|c| page_hide_count(&c.paragraphs)).sum(),
            _ => 0,
        })
        .sum()
}

/// HWP·HWPX 각각 저장 → 다시 열기 → 다시 저장 → 다시 열기.
fn reopened(doc: &HwpDocument) -> Vec<HwpDocument> {
    let hwp = |d: &HwpDocument| HwpDocument::from_bytes(&d.export_hwp_native().unwrap()).unwrap();
    let hwpx = |d: &HwpDocument| HwpDocument::from_bytes(&d.export_hwpx_native().unwrap()).unwrap();
    vec![hwp(&hwp(doc)), hwpx(&hwpx(doc))]
}

#[test]
fn continuation_page_hide_goes_at_caret_and_hides_only_that_page() {
    let mut doc = blank_with_header();
    let words: Vec<String> = (0..400)
        .map(|i| format!("word{i} continues here."))
        .collect();
    let text = words.join(" ");
    doc.insert_text_native(0, 0, 0, &text).unwrap();
    let pages = doc.page_count();
    assert!(pages >= 3, "문단이 셋째 쪽까지 이어진다: {pages}");
    // 둘째 쪽 가운데 줄의 글자 자리: 문단 시작 쪽(0)이 아닌 쪽에 캐럿을 둔다.
    let offset = text.find("word250").unwrap();
    let rect: serde_json::Value =
        serde_json::from_str(&doc.get_cursor_rect_native(0, 0, offset).unwrap()).unwrap();
    let page = rect["pageIndex"].as_u64().unwrap() as u32;
    assert!(page > 0);
    assert_eq!(hides(&doc, page)["hides"].as_array().unwrap().len(), 0);

    doc.set_page_hide_on_page_native(page, 0, &[], offset, HIDE)
        .unwrap();
    let found = hides(&doc, page);
    assert_eq!(found["hides"].as_array().unwrap().len(), 1, "{found}");
    assert_eq!(found["hides"][0]["hidePageNum"], true);
    assert_eq!(
        doc.document().sections[0].paragraphs[0].text,
        text,
        "글은 그대로다"
    );
    for p in 0..pages {
        assert_eq!(header_drawn(&doc, p), p != page, "쪽 {p}");
    }

    // 같은 쪽의 다른 자리에서 다시 고치면 부호를 그 자리로 옮겨 하나만 남긴다.
    let only_header = PageHide {
        hide_page_num: false,
        ..HIDE
    };
    let later = offset + 30;
    doc.set_page_hide_on_page_native(page, 0, &[], later, only_header)
        .unwrap();
    assert_eq!(page_hide_count(&doc.document().sections[0].paragraphs), 1);
    let para = &doc.document().sections[0].paragraphs[0];
    // 8유닛 조판 부호 자리가 새 캐럿 글자 바로 앞에 있다.
    assert_eq!(
        para.char_offsets[later],
        para.char_offsets[later - 1] + 1 + 8,
        "부호가 캐럿 자리로 옮겨졌다"
    );
    assert_eq!(hides(&doc, page)["hides"][0]["hidePageNum"], false);

    for again in reopened(&doc) {
        assert_eq!(again.document().sections[0].paragraphs[0].text, text);
        let found = hides(&again, page);
        assert_eq!(found["hides"].as_array().unwrap().len(), 1, "{found}");
        assert!(!header_drawn(&again, page));
        assert!(header_drawn(&again, 0));
    }

    // 모두 끄면 그 부호를 지우고 글자 자리를 되돌린다.
    doc.set_page_hide_on_page_native(page, 0, &[], 0, PageHide::default())
        .unwrap();
    assert_eq!(page_hide_count(&doc.document().sections[0].paragraphs), 0);
    assert_eq!(doc.document().sections[0].paragraphs[0].text, text);
    assert!(header_drawn(&doc, page));
}

#[test]
fn cell_page_hide_on_a_split_table_hides_the_table_start_page() {
    let mut doc = blank_with_header();
    doc.insert_text_native(0, 0, 0, "before").unwrap();
    let table: serde_json::Value =
        serde_json::from_str(&doc.create_table_native(0, 0, 6, 80, 2).unwrap()).unwrap();
    let para = table["paraIdx"].as_u64().unwrap() as usize;
    let ctrl = table["controlIdx"].as_u64().unwrap() as usize;
    // 한컴 새 표처럼 쪽 경계에서 셀 단위로 나눈다(HWPX CELL = RowBreak).
    doc.set_table_properties_native(0, para, ctrl, r#"{"pageBreak":2}"#)
        .unwrap();
    assert!(doc.page_count() >= 2, "표가 둘째 쪽으로 이어진다");
    let last = 159;
    doc.insert_text_in_cell_native(0, para, ctrl, last, 0, 0, "cell")
        .unwrap();
    let rect: serde_json::Value = serde_json::from_str(
        &doc.get_cursor_rect_in_cell_native(0, para, ctrl, last, 0, 2)
            .unwrap(),
    )
    .unwrap();
    let page = rect["pageIndex"].as_u64().unwrap() as u32;
    assert!(page > 0, "{rect}");
    doc.set_page_hide_on_page_native(page, para, &[(ctrl, last, 0)], 2, HIDE)
        .unwrap();
    // 둘째 쪽 칸 안의 중첩 표 칸에서 고친 감추기도 그 쪽에서 찾는다.
    let nested: serde_json::Value = serde_json::from_str(
        &doc.create_table_in_cell_by_path_native(
            0,
            para,
            &[(ctrl, last - 1, 0)],
            0,
            1,
            1,
            None,
            None,
            false,
        )
        .unwrap(),
    )
    .unwrap();
    let path: Vec<(usize, usize, usize)> = nested["cellPath"]
        .as_array()
        .unwrap()
        .iter()
        .map(|step| {
            let n = |k: &str| step[k].as_u64().unwrap() as usize;
            (n("controlIndex"), n("cellIndex"), n("cellParaIndex"))
        })
        .collect();
    assert_eq!(path.len(), 2, "{nested}");
    doc.set_page_hide_on_page_native(page, para, &path, 0, HIDE)
        .unwrap();
    let found = hides(&doc, page);
    assert_eq!(found["hides"].as_array().unwrap().len(), 1, "{found}");
    assert_eq!(found["hides"][0]["cellPath"].as_array().unwrap().len(), 2);
    let mut docs = reopened(&doc);
    docs.push(doc);
    for again in &docs {
        let found = hides(again, page);
        assert_eq!(
            found["hides"][0]["cellPath"][0]["cellIndex"],
            last - 1,
            "{found}"
        );
        assert_eq!(hides(again, 0)["hides"].as_array().unwrap().len(), 0);
        assert!(!header_drawn(again, 0), "표 첫 쪽을 감춘다");
        assert!(header_drawn(again, page));
    }
}

#[test]
fn page_hide_at_the_first_character_of_a_page_stays_on_that_page() {
    let mut doc = blank_with_header();
    let text = "long body text. ".repeat(600);
    doc.insert_text_native(0, 0, 0, &text).unwrap();
    let page_of = |doc: &HwpDocument, offset: usize| {
        let rect: serde_json::Value =
            serde_json::from_str(&doc.get_cursor_rect_native(0, 0, offset).unwrap()).unwrap();
        rect["pageIndex"].as_u64().unwrap() as u32
    };
    // 둘째 쪽 첫 글자 자리: 캐럿이 둘째 쪽 첫 줄 첫머리에 있다.
    let first = (0..text.len()).find(|&o| page_of(&doc, o) == 1).unwrap();
    doc.set_page_hide_on_page_native(1, 0, &[], first, HIDE)
        .unwrap();
    assert_eq!(hides(&doc, 1)["hides"].as_array().unwrap().len(), 1);
    assert_eq!(hides(&doc, 0)["hides"].as_array().unwrap().len(), 0);
    assert!(header_drawn(&doc, 0));
    assert!(!header_drawn(&doc, 1));
    for again in reopened(&doc) {
        assert_eq!(hides(&again, 1)["hides"].as_array().unwrap().len(), 1);
        assert!(header_drawn(&again, 0));
    }
}

/// 같은 문단의 감추기 조판 부호를 넣거나 빼거나 옮겨도 활성 누름틀 끝 입력은 그 누름틀에 든다.
#[test]
fn page_hide_on_page_keeps_typing_in_the_active_field() {
    for (before, hide) in [(false, true), (true, false), (true, true)] {
        let mut doc = HwpDocument::create_empty();
        doc.create_blank_document_native().unwrap();
        doc.insert_text_native(0, 0, 0, "앞  뒤").unwrap();
        doc.insert_click_here_field_at(0, 0, 2, "안내문", "메모", "남을 필드", true)
            .unwrap();
        doc.set_field_value_by_name("남을 필드", "AB").unwrap();
        if before {
            doc.set_page_hide_on_page_native(0, 0, &[], 0, HIDE)
                .unwrap();
        }
        let fields: Vec<serde_json::Value> =
            serde_json::from_str(&doc.get_field_list_json()).unwrap();
        let end = fields[0]["endCharIdx"].as_u64().unwrap() as usize;
        assert!(doc.set_active_field(0, 0, end));
        let ph = PageHide {
            hide_header: hide,
            ..Default::default()
        };
        doc.set_page_hide_on_page_native(0, 0, &[], 0, ph).unwrap();
        doc.insert_text_native(0, 0, end, "C").unwrap();
        let field: serde_json::Value =
            serde_json::from_str(&doc.get_field_value_by_name("남을 필드").unwrap()).unwrap();
        assert_eq!(field["value"], "ABC", "앞 감추기 {before}, 감추기 {hide}");
    }
}
