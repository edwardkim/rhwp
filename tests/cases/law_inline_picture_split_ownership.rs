//! 관계형·고정형: 그림 문단의 내용 소유·순서·겹침과 독립 Print 쪽 수.
//! 주석 갱신: 2026-10-30T16:20:11+09:00; 직전 파일 커밋: 2026-10-30T15:31:23+09:00 (`77fbef6da4a83d0671c5e9ed0a8b1bb3b4777788`).
//! 2026-10-30 중간 측정 raw(96 DPI, PDFium, 2px 이웃): 2쪽 Native 98.86% / WASM 98.87%,
//! 3쪽 Native 99.80% / WASM 99.81%; 전 6쪽 최저 Native 97.47% / WASM 97.46%.
//! 근거: mydocs/pr/assets/issue_8371/law-split-20261030/README.md. 최종 head 재측정 필요.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

fn walk<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    out.push(node);
    for child in node["children"].as_array().into_iter().flatten() {
        walk(child, out);
    }
}

#[test]
fn text_before_inline_picture_stays_on_current_page_and_picture_moves_once() {
    let input =
        include_bytes!("../../mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx");
    let mut doc = HwpDocument::from_bytes(input).unwrap();
    doc.set_hangul2024_compat(false);
    assert_eq!(doc.page_count(), 6, "독립 한컴 Print의 전체 쪽 수");
    let pages: Vec<Value> = (0..doc.page_count())
        .map(|page| serde_json::from_str(&doc.get_page_render_tree(page).unwrap()).unwrap())
        .collect();
    let mut matching_text_pages = Vec::new();
    let mut owner_image_pages = Vec::new();
    for (page, root) in pages.iter().enumerate() {
        let mut nodes = Vec::new();
        walk(root, &mut nodes);
        let text = nodes
            .iter()
            .filter(|n| n["type"] == "TextRun" && n["pi"] == 59)
            .filter_map(|n| n["text"].as_str())
            .collect::<String>();
        if text.contains("합산배제 대상 전통사찰보존지") {
            matching_text_pages.push(page);
        }
        owner_image_pages.extend(
            nodes
                .iter()
                .filter(|n| n["type"] == "Image" && n["pi"] == 59)
                .map(|n| (page, *n)),
        );
    }
    assert_eq!(
        matching_text_pages,
        [1],
        "그림 앞 본문은 이전 쪽에 한 번만 남는다"
    );
    assert_eq!(owner_image_pages.len(), 1, "그림 누락·중복");
    assert_eq!(owner_image_pages[0].0, 2, "그림은 다음 쪽에 이어진다");
    let image = owner_image_pages[0].1;
    let mut nodes = Vec::new();
    walk(&pages[2], &mut nodes);
    let following = nodes
        .iter()
        .find(|n| n["type"] == "TextRun" && n["pi"] == 60)
        .expect("그림 뒤 본문을 보존한다");
    let bottom = image["bbox"]["y"].as_f64().unwrap() + image["bbox"]["h"].as_f64().unwrap();
    assert!(
        bottom <= following["bbox"]["y"].as_f64().unwrap(),
        "그림과 후속 본문은 겹치지 않는다"
    );
}
