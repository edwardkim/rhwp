//! 관계형·고정형 회귀 — 중첩 표의 줄·캐럿 소속과 독립 Print의 쪽 경계를 확인한다.
//! 주석 갱신 2026-11-21T20:06:00+09:00.
//! 직전 통합 파일 변경 143f804aef0e44db5e7a87f4ab55645bb57e79b9 2026-11-21T12:29:46+09:00.
//! 표 70줄을 편집한 직후 전 4쪽 최저 Native 98.29830% / fresh WASM 98.26549%. 96 DPI, 2px raw.
//! 30줄 편집 직후 전 3쪽 최저 Native 98.2508% / fresh WASM 98.2153%. 두 형식 재열기 최저 98.0549% / 97.9423%.
//! 2쪽의 1~21줄, 3쪽의 22~58줄, 4쪽의 59~70줄을 같은 편집 상태의 한컴 2020 Print와 대조했다.
//! 근거: pdf/pr9273-nested-row-split/nested-70-hwp-2020.pdf 및 mydocs/pr/pr_9273_review.md.
//!
//! `samples/inner-table-01.hwp`의 바깥 표 마지막 칸에 든 12×2 안쪽 표(둘 다 쪽 경계에서 "나눔")의
//! ‘SFR-CLOUD-001’ 칸 뒤에 `줄 1`…`줄 70` 문단을 더하면 그 행이 쪽보다 길어진다. 한/글 12.30 Mac은
//! 그 행을 통째로 다음 쪽에 넘기지 않고 2쪽 바닥에서 글줄 사이로 나누고, 넘친 줄을 3·4쪽에 잇는다
//! (인쇄 대조 2026-11-21: 1~21줄이 2쪽, 22~58줄이 3쪽, 59~70줄과 나머지 행이 4쪽).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn path(para: usize) -> String {
    format!(
        r#"[{{"controlIndex":2,"cellIndex":13,"cellParaIndex":4}},{{"controlIndex":0,"cellIndex":2,"cellParaIndex":{para}}}]"#
    )
}

/// ‘SFR-CLOUD-001’ 칸 끝에 `줄 1`…`줄 n` 문단을 더한다.
fn grown(lines: usize) -> HwpDocument {
    let mut doc =
        HwpDocument::from_bytes(&std::fs::read("samples/inner-table-01.hwp").unwrap()).unwrap();
    for i in 0..lines {
        let len = doc
            .get_cell_paragraph_length_by_path(0, 0, &path(i))
            .unwrap();
        doc.split_paragraph_in_cell_by_path_api(0, 0, &path(i), len, None)
            .unwrap();
        doc.insert_text_in_cell_by_path_api(0, 0, &path(i + 1), 0, &format!("줄 {}", i + 1))
            .unwrap();
    }
    doc
}

/// 문단 캐럿의 (쪽, y, 높이).
fn caret(doc: &HwpDocument, para: usize) -> (u64, f64, f64) {
    let rect: Value =
        serde_json::from_str(&doc.get_cursor_rect_by_path(0, 0, &path(para), 0).unwrap()).unwrap();
    (
        rect["pageIndex"].as_u64().unwrap(),
        rect["y"].as_f64().unwrap(),
        rect["height"].as_f64().unwrap(),
    )
}

/// 쪽 그림에서 글이 `text`인 글줄의 y.
fn drawn_y(tree: &Value, text: &str) -> Option<f64> {
    fn joined(node: &Value) -> String {
        let own = if node["type"] == "TextRun" {
            node["text"].as_str().unwrap_or_default().to_string()
        } else {
            String::new()
        };
        node["children"]
            .as_array()
            .into_iter()
            .flatten()
            .fold(own, |all, child| all + &joined(child))
    }
    fn walk(node: &Value, text: &str) -> Option<f64> {
        if node["type"] == "TextLine" && joined(node).trim() == text {
            return node["bbox"]["y"].as_f64();
        }
        node["children"]
            .as_array()?
            .iter()
            .find_map(|child| walk(child, text))
    }
    walk(tree, text)
}

/// 한컴 Print의 줄 소속으로 세 조각의 경계를 검사한다. 캐럿·그리기 모두
/// 실제 본문 상자 안에서 같은 줄을 가리켜야 한다.
fn assert_split(doc: &HwpDocument, lines: usize, pages: u32) {
    assert_eq!(doc.page_count(), pages);
    let trees: Vec<Value> = (0..doc.page_count())
        .map(|page| serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap())
        .collect();
    for para in 1..=lines {
        let (page, y, h) = caret(doc, para);
        let expected = match para {
            1..=21 => 1,
            22..=58 => 2,
            _ => 3,
        };
        assert_eq!(page, expected, "줄 {para}의 인쇄 쪽이 달라졌다");
        let tree = &trees[page as usize];
        let body = tree["children"]
            .as_array()
            .unwrap()
            .iter()
            .find(|node| node["type"] == "Body")
            .unwrap();
        let top = body["bbox"]["y"].as_f64().unwrap();
        let bottom = top + body["bbox"]["h"].as_f64().unwrap();
        assert!(
            y >= top - 0.5 && y + h <= bottom + 0.5,
            "줄 {para}의 캐럿이 {page}쪽 본문 밖 y {y:.1}에 있다"
        );
        let drawn = drawn_y(tree, &format!("줄 {para}"))
            .unwrap_or_else(|| panic!("줄 {para}가 {page}쪽에 그려지지 않았다"));
        assert!(
            (drawn - y).abs() < h,
            "줄 {para}: 그림 y {drawn:.1}과 캐럿 y {y:.1}이 다르다"
        );
    }
}

#[test]
fn row_longer_than_page_splits_at_lines() {
    assert_split(&grown(70), 70, 4);
}

/// 다음 쪽에 통째로 들어갈 행도 한/글처럼 통째로 넘기지 않고 쪽 바닥에서 나눈다.
#[test]
fn row_that_fits_next_page_still_splits_at_page_bottom() {
    assert_split(&grown(30), 30, 3);
}

/// 다른 칸에 글자를 쳐도 쪽에서 나뉜 앞 행(바깥 표 ‘사업개요’ 행)의 이어진 조각은
/// 선언 높이를 지킨다.
#[test]
fn typing_keeps_earlier_split_row_height() {
    let continued = |doc: &HwpDocument| {
        fn walk(node: &Value, depth: usize) -> Option<f64> {
            let depth = depth + usize::from(node["type"] == "Table");
            if node["type"] == "Cell" && depth == 1 && node["row"] == 6 {
                return node["bbox"]["h"].as_f64();
            }
            node["children"]
                .as_array()?
                .iter()
                .find_map(|child| walk(child, depth))
        }
        walk(
            &serde_json::from_str(&doc.get_page_render_tree(1).unwrap()).unwrap(),
            0,
        )
        .unwrap()
    };
    let mut doc =
        HwpDocument::from_bytes(&std::fs::read("samples/inner-table-01.hwp").unwrap()).unwrap();
    let opened = continued(&doc);
    doc.insert_text_in_cell_by_path_api(0, 0, &path(0), 0, "x")
        .unwrap();
    let typed = continued(&doc);
    assert!(
        (typed - opened).abs() < 0.5,
        "글자 하나를 치자 이어진 조각이 {opened:.1}에서 {typed:.1}로 바뀌었다"
    );
}

#[test]
fn click_on_continued_part_reaches_that_line() {
    let doc = grown(70);
    let (page, y, h) = caret(&doc, 65);
    assert_eq!(page, 3, "줄 65는 넷째 쪽에 이어진다");
    let hit: Value =
        serde_json::from_str(&doc.hit_test(page as u32, 260.0, y + h / 2.0).unwrap()).unwrap();
    let cell_path = hit["cellPath"]
        .as_array()
        .expect("이어진 칸을 눌렀는데 셀 경로가 없다");
    assert_eq!(cell_path.last().unwrap()["cellParaIndex"], 65, "{hit}");
}

#[test]
fn saved_hwp_reopens_split_where_hancom_breaks() {
    for (lines, pages) in [(30, 3), (70, 4)] {
        let saved = grown(lines).export_hwp().unwrap();
        assert_split(&HwpDocument::from_bytes(&saved).unwrap(), lines, pages);
    }
}

#[test]
fn saved_hwpx_reopens_split_where_hancom_breaks() {
    for (lines, pages) in [(30, 3), (70, 4)] {
        let saved = grown(lines).export_hwpx().unwrap();
        assert_split(&HwpDocument::from_bytes(&saved).unwrap(), lines, pages);
    }
}

/// 양 형식의 실제 저장 바이트를 다시 파싱해 칸 자체의 저장 줄을 읽는다.
fn assert_saved_page_origins(bytes: &[u8], lines: usize) {
    use rhwp::model::control::Control;
    let document = rhwp::parser::parse_document(bytes).unwrap();
    let Control::Table(outer) = &document.sections[0].paragraphs[0].controls[2] else {
        panic!("바깥 표")
    };
    let Control::Table(inner) = &outer.cells[13].paragraphs[4].controls[0] else {
        panic!("안쪽 표")
    };
    let paragraphs = &inner.cells[2].paragraphs;
    for (line, expected) in [
        (21, 36960),
        (22, 0),
        (23, 1760),
        (58, 63360),
        (59, 0),
        (60, 1760),
    ] {
        if line <= lines {
            assert_eq!(paragraphs[line].text, format!("줄 {line}"));
            assert_eq!(
                paragraphs[line].line_segs[0].vertical_pos, expected,
                "줄 {line}"
            );
        }
    }
}

#[test]
fn saved_rows_restart_at_page_break_like_hancom() {
    for lines in [30, 70] {
        let live = grown(lines);
        for saved in [live.export_hwp().unwrap(), live.export_hwpx().unwrap()] {
            assert_saved_page_origins(&saved, lines);
            let reopened = HwpDocument::from_bytes(&saved).unwrap();
            assert_saved_page_origins(&reopened.export_hwpx().unwrap(), lines);
        }
    }
}
