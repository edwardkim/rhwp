//! 쪽보다 긴 중첩 표 한 행을 쪽 경계의 글줄 사이에서 나눈다.
//!
//! `samples/inner-table-01.hwp`의 바깥 표 마지막 칸에 든 12×2 안쪽 표(둘 다 쪽 경계에서 "나눔")의
//! ‘SFR-CLOUD-001’ 칸 뒤에 `줄 1`…`줄 70` 문단을 더하면 그 행이 쪽보다 길어진다. 한/글 12.30 Mac은
//! 그 행을 통째로 다음 쪽에 넘기지 않고 2쪽 바닥에서 글줄 사이로 나누고, 넘친 줄을 3·4쪽에 잇는다
//! (인쇄 대조 2026-11-21: 1~21줄이 2쪽, 22~58줄이 3쪽, 59~70줄과 나머지 행이 4쪽).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const BODY_TOP: f64 = 132.3;
const BODY_BOTTOM: f64 = 1009.1;

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
fn drawn_y(doc: &HwpDocument, page: u32, text: &str) -> Option<f64> {
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
    walk(
        &serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap(),
        text,
    )
}

/// 줄마다 캐럿이 본문 안에 있고 그 자리에 그 줄이 그려지며, 쪽은 뒤로만 간다.
/// 행은 둘째 쪽(쪽 번호 1)에서 시작하고(통째로 넘기지 않는다) `first_break` 줄부터 셋째 쪽이며
/// (한/글 인쇄의 첫 나눔 자리), 마지막 줄은 끝 쪽에 있다.
fn assert_split(doc: &HwpDocument, lines: usize, first_break: usize, pages: u32) {
    let mut previous = 0;
    for para in 1..=lines {
        let (page, y, h) = caret(doc, para);
        assert!(
            y >= BODY_TOP - 0.5 && y + h <= BODY_BOTTOM + 0.5,
            "줄 {para}의 캐럿이 {page}쪽 본문 밖 y {y:.1}에 있다"
        );
        assert!(page >= previous, "줄 {para}가 앞 쪽 {page}로 돌아갔다");
        previous = page;
        let drawn = drawn_y(doc, page as u32, &format!("줄 {para}"))
            .unwrap_or_else(|| panic!("줄 {para}가 {page}쪽에 그려지지 않았다"));
        assert!(
            (drawn - y).abs() < h,
            "줄 {para}: 그림 y {drawn:.1}과 캐럿 y {y:.1}이 다르다"
        );
    }
    assert_eq!(
        caret(doc, 1).0,
        1,
        "행이 둘째 쪽에서 시작하지 않고 통째로 넘어갔다"
    );
    assert_eq!(caret(doc, first_break - 1).0, 1);
    assert_eq!(
        caret(doc, first_break).0,
        2,
        "줄 {first_break}부터 셋째 쪽이어야 한다"
    );
    assert_eq!(doc.page_count(), pages, "한/글 인쇄와 쪽 수가 다르다");
    assert_eq!(caret(doc, lines).0, u64::from(pages) - 1);
}

/// 한/글 12.30 인쇄: 70줄은 22줄부터 셋째 쪽이고 4쪽, 30줄은 22줄부터 셋째 쪽이고 3쪽이다.
const HANCOM: [(usize, usize, u32); 2] = [(70, 22, 4), (30, 22, 3)];

#[test]
fn row_longer_than_page_splits_at_lines() {
    assert_split(&grown(70), 70, 22, 4);
}

/// 다음 쪽에 통째로 들어갈 행도 한/글처럼 통째로 넘기지 않고 쪽 바닥에서 나눈다.
#[test]
fn row_that_fits_next_page_still_splits_at_page_bottom() {
    assert_split(&grown(30), 30, 22, 3);
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
    for (lines, first_break, pages) in HANCOM {
        let saved = grown(lines).export_hwp().unwrap();
        assert_split(
            &HwpDocument::from_bytes(&saved).unwrap(),
            lines,
            first_break,
            pages,
        );
    }
}

#[test]
fn saved_hwpx_reopens_split_where_hancom_breaks() {
    for (lines, first_break, pages) in HANCOM {
        let saved = grown(lines).export_hwpx().unwrap();
        assert_split(
            &HwpDocument::from_bytes(&saved).unwrap(),
            lines,
            first_break,
            pages,
        );
    }
}

/// HWPX 저장본에서 `줄 n` 칸 문단 첫 줄의 vertpos.
fn saved_vpos(hwpx: &[u8], line: usize) -> i64 {
    use std::io::Read;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).unwrap();
    let mut xml = String::new();
    zip.by_name("Contents/section0.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let rest = &xml[xml.find(&format!("<hp:t>줄 {line}</hp:t>")).unwrap()..];
    let rest = &rest[rest.find("vertpos=\"").unwrap() + 9..];
    rest[..rest.find('"').unwrap()].parse().unwrap()
}

/// 저장본은 한/글처럼 나뉜 쪽의 첫 칸 줄을 원점(vpos 0)으로 되감고 그 뒤 줄을 거기서 잇는다
/// (한/글 12.30이 저장한 30줄 HWPX: `줄 21` 36960, `줄 22` 0, `줄 23` 1760).
#[test]
fn saved_rows_restart_at_page_break_like_hancom() {
    let hancom = [(21, 36960), (22, 0), (23, 1760)];
    let saved = grown(30).export_hwpx().unwrap();
    for (line, vpos) in hancom {
        assert_eq!(saved_vpos(&saved, line), vpos, "줄 {line}");
    }
    // 다시 연 HWP를 다시 저장해도 되감김이 그대로 남는다.
    let reopened = HwpDocument::from_bytes(&grown(30).export_hwp().unwrap()).unwrap();
    let saved = reopened.export_hwpx().unwrap();
    for (line, vpos) in hancom {
        assert_eq!(saved_vpos(&saved, line), vpos, "다시 저장한 줄 {line}");
    }
}
