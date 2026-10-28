#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;

// 입력에는 목록 문단의 저장 LineSeg가 없다. MCP2024 Print의 전체4쪽에서
// B13은 1/2쪽, B40은 3/4쪽에 나뉘고 B26은 2쪽에 전부 남는다.
// 위치의 픽셀값 대신 최종 페이지 항목의 줄 소유·내용 순서·완전성을 검사한다.
#[test]
fn reflowed_list_lines_keep_their_print_page_ownership() {
    let core = DocumentCore::from_bytes(
        &std::fs::read("samples/issue7418/list_marker_head_synthetic.hwpx")
            .expect("목록 원본 읽기"),
    )
    .expect("목록 원본 로드");
    let pages = core.dump_page_items_json(None);
    let pages = pages.as_array().expect("페이지 항목 목록");
    assert_eq!(pages.len(), 4, "독립 Print의 전체 쪽 수");
    let mut seen = Vec::new();
    for (page, entry) in pages.iter().enumerate() {
        for column in entry["columns"].as_array().expect("단 목록") {
            for item in column["items"].as_array().expect("단 항목") {
                let pi = item["paraIndex"].as_u64().expect("문단 소유") as usize;
                if pi == 0 {
                    continue;
                }
                let count = core.document().sections[0].paragraphs[pi].line_segs.len();
                let (start, end) = match item["kind"].as_str().expect("항목 종류") {
                    "fullParagraph" => (0, count),
                    "partialParagraph" => (
                        item["startLine"].as_u64().unwrap() as usize,
                        item["endLine"].as_u64().unwrap() as usize,
                    ),
                    other => panic!("본문에 예상하지 않은 항목: {other}"),
                };
                for line in start..end {
                    seen.push((pi, line, page));
                }
            }
        }
    }
    let mut expected = Vec::new();
    for pi in 1..=43 {
        for line in 0..3 {
            let page = match (pi, line) {
                (1..=13, _) | (14, 0) => 0,
                (14, 1..=2) | (15..=27, _) => 1,
                (28..=40, _) | (41, 0..=1) => 2,
                (41, 2) | (42..=43, _) => 3,
                _ => unreachable!(),
            };
            expected.push((pi, line, page));
        }
    }
    assert_eq!(
        seen, expected,
        "Print와 같은 줄 소유, 원문 순서, 누락·중복 없음"
    );
}
