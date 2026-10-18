//! 뒤 구역만 다시 조판해도 앞 구역에서 물려받은 머리말이 처음 조판과 같다.
//!
//! `paginate` 는 구역 간 머리말/꼬리말 carry 를 두 규칙으로 만들었다. dirty 구역은 구역에
//! 정의된 머리말 컨트롤로 만들고, dirty 가 아닌 구역은 쪽의 `active_header` 로 만들었다.
//! 쪽에 한 번도 쓰이지 않은 정의(홀수 쪽 하나뿐인 구역의 Even 머리말)는 뒤 구역만 다시
//! 조판할 때 빠졌다. `exam_social.hwp` 2구역(2~4쪽)에서 글자 하나를 넣었다 지우면
//! 2·4쪽 머리말 "2 (사회·문화)"·"4 (사회·문화)" 가 "2"·"4" 로 바뀌었다.

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/exam_social.hwp";

/// 쪽마다 쪽 정보와 첫 네 줄(머리말 포함).
fn page_heads(d: &HwpDocument) -> Vec<String> {
    (0..d.page_count())
        .map(|p| {
            let text = d.extract_page_text_native(p).unwrap();
            format!(
                "{}\n{}",
                d.get_page_info_native(p).unwrap(),
                text.lines().take(4).collect::<Vec<_>>().join("\n")
            )
        })
        .collect()
}

#[test]
fn editing_later_section_keeps_inherited_even_header() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(doc.document().sections.len(), 2, "1구역 1쪽, 2구역 2~4쪽");
    let before = page_heads(&doc);
    for p in [1, 3] {
        assert!(
            before[p].contains("(사회·문화)"),
            "{}쪽 머리말이 전제다: {}",
            p + 1,
            before[p]
        );
    }
    let pi = doc.document().sections[1]
        .paragraphs
        .iter()
        .position(|p| p.text.chars().count() > 4)
        .expect("2구역 본문 문단");
    doc.insert_text_native(1, pi, 1, "가").unwrap();
    doc.delete_text_native(1, pi, 1, 1).unwrap();
    assert_eq!(page_heads(&doc), before);
}
