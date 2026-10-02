//! Issue #1100: 시험지의 문항 소속과 머리말·바탕쪽 자동번호 보존.
//!
//! 원본: samples/hwpx/exam_social.hwpx.
//! 독립 기준: pdf/exam_social-hwpx-revalidated-2020.pdf (정상 한컴 출력 4쪽).
//! 좌표는 조판 개선 때 바뀔 수 있으므로 고정 픽셀 대신 실제 쪽·영역·내용을 검사한다.
//! 배치 품질은 같은 원본의 전체 Native/fresh WASM Visual Sweep으로 별도 확인한다.

use std::fs;
use std::path::Path;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

fn page_tree(doc: &rhwp::wasm_api::HwpDocument, page: u32) -> serde_json::Value {
    serde_json::from_str(&doc.get_page_render_tree(page).expect("쪽 렌더 트리"))
        .expect("렌더 트리 JSON")
}

fn descendants(node: &serde_json::Value) -> Vec<&serde_json::Value> {
    let mut pending = vec![node];
    let mut nodes = Vec::new();
    while let Some(node) = pending.pop() {
        nodes.push(node);
        if let Some(children) = node["children"].as_array() {
            pending.extend(children.iter().rev());
        }
    }
    nodes
}

fn displayed_text(node: &serde_json::Value) -> &str {
    node["displayText"]
        .as_str()
        .or_else(|| node["text"].as_str())
        .unwrap_or_default()
}

fn region<'a>(tree: &'a serde_json::Value, kind: &str) -> &'a serde_json::Value {
    tree["children"]
        .as_array()
        .expect("쪽 영역")
        .iter()
        .find(|node| node["type"] == kind)
        .expect("필수 쪽 영역")
}

#[test]
fn issue_1100_hwpx_four_pages_preserve_question_paragraph_ownership() {
    let doc = load_doc("samples/hwpx/exam_social.hwpx");
    assert_eq!(doc.page_count(), 4, "한컴 기준과 같은 전체 4쪽");
    for page in 0..4 {
        let tree = page_tree(&doc, page);
        let body = region(&tree, "Body");
        // 칼럼의 본문 문단만 읽는다. 표 셀의 보기 번호를 문항 번호로 세지 않는다.
        let mut questions = Vec::new();
        for column in body["children"].as_array().expect("본문 칼럼") {
            for node in column["children"].as_array().expect("칼럼 항목") {
                if node["type"] != "TextLine" {
                    continue;
                }
                let Some(first) = node["children"].as_array().and_then(|runs| runs.first()) else {
                    continue;
                };
                if let Some(number) = displayed_text(first)
                    .trim()
                    .strip_suffix('.')
                    .and_then(|text| text.parse::<u32>().ok())
                {
                    questions.push(number);
                }
            }
        }
        assert_eq!(
            questions,
            (page * 5 + 1..=page * 5 + 5).collect::<Vec<_>>(),
            "각 문항은 정상 PDF와 같은 쪽에 한 번씩 있어야 한다: {}쪽",
            page + 1
        );
        let body_text = descendants(body)
            .into_iter()
            .filter(|node| node["type"] == "TextRun")
            .map(displayed_text)
            .collect::<String>();
        assert_eq!(
            body_text.contains("시기별 계층 구성 비율"),
            page == 1,
            "10번 문항의 자료는 2쪽 본문에 있어야 한다"
        );
        if page == 3 {
            // 원본 HANGUL_JAMO와 한컴 PDF의 16/18번 보기는 두 번 모두 ㄱ~ㄹ이다.
            for marker in ["ㄱ. ", "ㄴ. ", "ㄷ. ", "ㄹ. "] {
                assert_eq!(
                    descendants(body)
                        .into_iter()
                        .filter(|node| node["type"] == "TextRun" && displayed_text(node) == marker)
                        .count(),
                    2,
                    "4쪽 보기의 자모 번호를 숫자로 바꾸지 않는다: {marker}"
                );
            }
        }
    }
}

#[test]
fn issue_1100_hwpx_even_header_page_auto_number_replaces_one_placeholder_only() {
    let doc = load_doc("samples/hwpx/exam_social.hwpx");
    assert_eq!(doc.page_count(), 4);
    for page in [1, 2, 3] {
        let tree = page_tree(&doc, page);
        let header = region(&tree, "Header");
        let number = (page + 1).to_string();
        let expected = if page % 2 == 1 {
            format!("{number}\u{2007}")
        } else {
            number.clone()
        };
        let placeholders: Vec<_> = descendants(header)
            .into_iter()
            .filter(|node| {
                node["type"] == "TextRun" && node["displayText"].as_str() == Some(expected.as_str())
            })
            .collect();
        assert_eq!(
            placeholders.len(),
            1,
            "머리말의 현재 쪽번호는 한 번만 치환: {}쪽",
            page + 1
        );
        let raw = placeholders[0]["text"].as_str().expect("모델 자리표시자");
        assert_eq!(
            raw,
            if page % 2 == 1 { " \u{2007}" } else { " " },
            "표시 치환은 원모델의 자리표시자와 뒤 공백을 보존한다"
        );
        let text = descendants(header)
            .into_iter()
            .filter(|node| node["type"] == "TextRun")
            .map(displayed_text)
            .collect::<String>();
        assert_eq!(
            text.matches(&number).count(),
            1,
            "뒤 공백에 쪽번호를 중복 출력하지 않는다"
        );
        if page % 2 == 1 {
            assert!(
                text.contains("(사회·문화)"),
                "음수 문단 위치의 과목 글상자도 머리말에 보존"
            );
        }
    }
}

#[test]
fn issue_1100_hwpx_master_page_footer_page_number_is_preserved() {
    let doc = load_doc("samples/hwpx/exam_social.hwpx");
    assert_eq!(doc.page_count(), 4);
    for page in 0..4 {
        let tree = page_tree(&doc, page);
        let master = region(&tree, "MasterPage");
        let number = (page + 1).to_string();
        let nodes = descendants(master);
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node["type"] == "TextRun"
                    && node["displayText"].as_str() == Some(number.as_str()))
                .count(),
            1,
            "바탕쪽 꼬리말의 현재 쪽번호는 한 번만 표시: {}쪽",
            page + 1
        );
        assert!(
            nodes
                .iter()
                .any(|node| node["type"] == "TextRun" && displayed_text(node) == "32"),
            "원본 꼬리말의 인쇄 묶음 번호도 보존"
        );
    }
}
