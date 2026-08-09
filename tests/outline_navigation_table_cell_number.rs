//! 개요 탐색(`getOutlineNavigation`) 회귀 가드 — 표 셀 번호 문단이 지나간 뒤의 번호.
//!
//! PR #4093 리뷰 지적: 탐색 질의가 최상위 `section.paragraphs` 만 훑으면 표 셀의
//! `NUMBER` 문단이 렌더러 번호 상태에는 반영되고 질의 상태에는 반영되지 않는다.
//! `앞 개요 1. → 표 셀 번호 2. → 뒤 개요 3.` 문서에서 렌더러는 `3.` 을 그리는데
//! 질의는 `2.` 를 돌려줬다.
//!
//! 이 테스트는 같은 fixture 한 개로 두 경로를 맞대 본다 — 렌더된 SVG 텍스트(화면에
//! 실제로 그려진 번호)와 탐색 질의의 번호가 같아야 한다. fixture 는
//! `scripts/generate_outline_navigation_fixture.py` 가 만든다.

use rhwp::wasm_api::HwpDocument;

const FIXTURE: &str = "samples/pr4093/outline_navigation_table_cell_number.hwpx";
const DEMO_FIXTURE: &str = "samples/pr4093/outline_navigation_panel_demo.hwpx";

/// SVG `<text>` 내용만 이어 붙이고 공백을 지운 문자열.
fn page_text(doc: &HwpDocument, page: u32) -> String {
    let svg = doc.render_page_svg(page).unwrap();
    let mut out = String::new();
    let mut rest = svg.as_str();
    while let Some(start) = rest.find("<text") {
        let Some(open_end) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + open_end + 1..];
        let Some(close) = after.find("</text>") else {
            break;
        };
        out.push_str(&after[..close]);
        rest = &after[close..];
    }
    out.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 개요 항목의 쪽 번호만.
fn entries_pages(doc: &HwpDocument) -> Vec<u32> {
    let json: serde_json::Value =
        serde_json::from_str(&doc.get_outline_navigation().unwrap()).expect("개요 탐색 JSON");
    json["outline"]
        .as_array()
        .expect("outline 배열")
        .iter()
        .map(|item| item["page"].as_u64().unwrap_or_default() as u32)
        .collect()
}

fn outline_entries(doc: &HwpDocument) -> Vec<(String, String, u64)> {
    let json: serde_json::Value =
        serde_json::from_str(&doc.get_outline_navigation().unwrap()).expect("개요 탐색 JSON");
    json["outline"]
        .as_array()
        .expect("outline 배열")
        .iter()
        .map(|item| {
            (
                item["number"].as_str().unwrap_or_default().to_owned(),
                item["title"].as_str().unwrap_or_default().to_owned(),
                item["level"].as_u64().unwrap_or_default(),
            )
        })
        .collect()
}

#[test]
fn outline_numbers_match_rendered_numbers_across_table_cell_number() {
    let bytes = std::fs::read(FIXTURE).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();

    let entries = outline_entries(&doc);

    // 개요 속성이 붙은 문단만 — 표 셀 문단과 `1. 일반 본문`(텍스트만 번호) 은 빠진다.
    assert_eq!(
        entries.len(),
        3,
        "개요 문단 3개만 나와야 한다 (실제: {entries:?})"
    );
    assert_eq!(
        entries,
        vec![
            ("1.".to_owned(), "개요".to_owned(), 1),
            ("가.".to_owned(), "목적".to_owned(), 2),
            // 표 셀의 NUMBER 문단이 카운터를 2 로 밀어낸 뒤라 3. 이다.
            ("3.".to_owned(), "요구사항".to_owned(), 1),
        ],
    );

    // 화면 대조 — 렌더러가 그린 번호와 같은지. 표 셀 문단을 건너뛰는 구현은 여기서
    // 질의만 2. 가 되어 두 값이 어긋난다.
    let rendered = page_text(&doc, 0);
    assert!(
        rendered.contains("3.요구사항"),
        "렌더된 뒤 개요가 3. 이 아니다: {rendered}"
    );
    assert!(
        !rendered.contains("2.요구사항"),
        "렌더된 뒤 개요가 2. 로 그려졌다: {rendered}"
    );
    assert!(
        rendered.contains("1.개요"),
        "렌더된 앞 개요가 1. 이 아니다: {rendered}"
    );
}

/// 패널 데모 fixture — 3수준 계층 15개가 3쪽에 걸쳐 있고, 가운데에 같은 표 셀 경계가 있다.
///
/// 실제 문서 모양에서 패널이 보여줄 값을 통째로 고정한다. 손으로 눌러 볼 때 기대치를
/// 이 목록으로 확인할 수 있고(`samples/pr4093/README.md`), 수준·쪽 계산이 틀어지면
/// 여기서 잡힌다.
#[test]
fn panel_demo_outline_matches_rendered_document() {
    let bytes = std::fs::read(DEMO_FIXTURE).unwrap();
    let doc = HwpDocument::from_bytes(&bytes).unwrap();

    assert_eq!(doc.page_count(), 3, "데모 fixture 는 3쪽이어야 한다");

    let entries = outline_entries(&doc);
    let shape: Vec<(&str, &str, u64)> = entries
        .iter()
        .map(|(number, title, level)| (number.as_str(), title.as_str(), *level))
        .collect();

    assert_eq!(
        shape,
        vec![
            ("1.", "총칙", 1),
            ("가.", "목적", 2),
            ("1)", "배경", 3),
            ("2)", "적용 범위", 3),
            ("나.", "용어 정의", 2),
            ("2.", "본문 규정", 1),
            ("가.", "요구사항", 2),
            ("1)", "기능 요구", 3),
            ("2)", "비기능 요구", 3),
            ("나.", "제약 조건", 2),
            ("3.", "표가 낀 구간", 1),
            ("가.", "표 뒤 하위 개요", 2),
            // 표 셀의 NUMBER 문단이 4 를 가져갔으므로 다음 최상위 개요는 5. 다.
            ("5.", "부칙", 1),
            ("가.", "시행일", 2),
            ("나.", "경과 조치", 2),
        ],
    );

    // 이동 대상 쪽도 함께 고정한다 — 패널에서 누르면 이 쪽으로 스크롤한다.
    let pages: Vec<u32> = entries_pages(&doc);
    assert_eq!(pages, vec![1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3]);

    // 화면 대조 — 표 셀 경계를 지난 뒤 개요가 렌더러와 같은 5. 인지.
    let page3 = page_text(&doc, 2);
    assert!(
        page3.contains("5.부칙"),
        "렌더된 부칙이 5. 가 아니다: {page3}"
    );
    assert!(
        !page3.contains("4.부칙"),
        "렌더된 부칙이 4. 로 그려졌다: {page3}"
    );
}
