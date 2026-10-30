//! 관계형·고정형: 그림 문단의 내용 소유·순서·겹침과 독립 Print 쪽 수.
//! 주석 갱신: 2026-10-31T00:16:01+09:00; 직전 파일 커밋: 2026-10-30T23:08:46+09:00 (`8f3d099de5b8569ae44405858d8210588ed3898e`).
//! 2026-10-30 측정 raw(96 DPI, PDFium, 2px 이웃): 1쪽 Native 99.97% / WASM 99.90%,
//! 6쪽 Native 99.98% / WASM 100.00%, 2쪽 Native 98.86% / WASM 98.19%,
//! 3쪽 Native 99.80% / WASM 99.82%; 전 6쪽 최저 Native 98.86% / WASM 98.19%.
//! 근거: mydocs/pr/assets/issue_8371/law-split-20261030/README.md.
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
fn paired_angle_opener_stays_with_the_following_line() {
    let input =
        include_bytes!("../../mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx");
    let mut doc = HwpDocument::from_bytes(input).unwrap();
    doc.set_hangul2024_compat(false);
    let page: Value = serde_json::from_str(&doc.get_page_render_tree(0).unwrap()).unwrap();
    let mut nodes = Vec::new();
    walk(&page, &mut nodes);
    let lines: Vec<String> = nodes
        .iter()
        .filter(|node| node["type"] == "TextLine" && node["pi"] == 10)
        .map(|line| {
            line["children"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|node| node["text"].as_str())
                .collect()
        })
        .collect();
    assert_eq!(lines.len(), 2, "원문의 문단과 내용 소유를 유지한다");
    assert!(!lines[0].trim_end().ends_with('<'));
    assert!(lines[1].trim_start().starts_with("<개정"));
}

#[test]
fn missing_paragraph_shape_uses_hancom_default_without_changing_explicit_shapes() {
    let input =
        include_bytes!("../../mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx");
    let core = rhwp::DocumentCore::from_bytes(input).unwrap();
    let document = core.document();
    let paragraphs = &document.sections[0].paragraphs;
    let missing = &paragraphs[156];
    let explicit = &paragraphs[155];
    assert!(missing.text.is_empty());
    assert_eq!(
        document.doc_info.para_shapes[missing.para_shape_id as usize].line_spacing,
        160
    );
    assert_eq!(
        document.doc_info.para_shapes[explicit.para_shape_id as usize].line_spacing,
        130
    );

    let mut doc = HwpDocument::from_bytes(input).unwrap();
    doc.set_hangul2024_compat(false);
    let page: Value = serde_json::from_str(&doc.get_page_render_tree(5).unwrap()).unwrap();
    let mut nodes = Vec::new();
    walk(&page, &mut nodes);
    let line = |paragraph| {
        nodes
            .iter()
            .find(|node| node["type"] == "TextLine" && node["pi"] == paragraph)
            .expect("빈 문단과 후속 문장은 같은 쪽에 각각 한 번 배치된다")
    };
    let empty = line(156);
    let following = line(157);
    let top = empty["bbox"]["y"].as_f64().unwrap();
    let height = empty["bbox"]["h"].as_f64().unwrap();
    let next_top = following["bbox"]["y"].as_f64().unwrap();
    assert!(
        (next_top - top - height * 1.6).abs() < height / 100.0,
        "참조를 생략한 빈 문단은 한컴 기본 줄 간격을 소비한다"
    );
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
