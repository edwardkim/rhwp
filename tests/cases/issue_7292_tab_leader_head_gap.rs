//! #7292: 첫 점의 실제 중심을 독립 Print와 비교해 앞 글자와의 간격을 보호한다.
//! pdf/KTX-2022.pdf p2: 첫 U+00B7의 origin=164.28pt, size=15pt.
//! T3 Encoding code00→HFT1, CharProc xref24의 윤곽 중심은 192/1000em이다.
//! 따라서 첫 점 중심은 (164.28 + 15*0.192)*96/72 = 222.88px이다.
//! 이전 검사는 glyph advance 원점 219.07px를 SVG dash 선의 x1과 비교했다.
//! 개별 원을 그리는 출력에서는 실제 점 중심을 확인하며, 선 태그 존재로 대신하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

mod svg_tab_leaders {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/common/svg_tab_leaders.rs"
    ));
}

#[test]
fn toc_first_dot_center_preserves_independent_head_gap() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs/pr/assets/issue7445/ktx-toc27/KTX.hwp");
    let bytes = std::fs::read(&path).expect("KTX.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("문서 로드");
    let svg = doc.render_page_svg_native(1).expect("목차 쪽 SVG");
    let leader = svg_tab_leaders::dot_leaders(&svg)
        .into_iter()
        .min_by(|a, b| a.y.total_cmp(&b.y))
        .expect("실제 점끌기");
    let first_center = leader.start + leader.diameter / 2.0;
    assert!(
        (first_center - 222.88).abs() <= 0.6,
        "첫 점 중심 {first_center:.2}px — 독립 Print의 실제 glyph 중심 222.88px"
    );
}
