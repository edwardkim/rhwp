//! 관계형: 원본 전체 GIF를 불필요한 자르기 viewport 없이 보존한다.
//! 주석 갱신: 2026-11-01T03:58:43+09:00; 새 파일이므로 직전 파일 변경 커밋은 없다.
//! 2026-11-01 측정 raw(96dpi, 2px): 3쪽 Native·fresh WASM 모두 100.00%;
//! 전 6쪽 최저 Native 98.86%, fresh WASM 98.87%.
//! 원본·한컴 Print·전쪽 TSV·3쪽 비교: mydocs/pr/assets/integration_20261031_gif_crop/README.md.
//! 시스템 글꼴의 설치 여부와 무관하게 원본 GIF와 SVG viewport의 소유 관계를 검사한다.

use rhwp::DocumentCore;

#[test]
fn full_gif_is_not_wrapped_as_a_cropped_image() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("mydocs/pr/assets/legal_hwpx_cellsz_host_line/input/jongbu_law.hwpx");
    let bytes = std::fs::read(path).expect("원본 HWPX");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    let svg = core.render_page_svg_native(2).expect("셋째 쪽 렌더");
    let document = roxmltree::Document::parse(&svg).expect("SVG 파싱");
    let gifs: Vec<_> = document
        .descendants()
        .filter(|node| {
            node.has_tag_name("image")
                && node
                    .attribute("href")
                    .is_some_and(|href| href.starts_with("data:image/gif;base64,"))
        })
        .collect();
    assert_eq!(gifs.len(), 1, "원본 GIF가 빠지거나 중복되면 안 된다");
    let svg_ancestors = gifs[0]
        .ancestors()
        .filter(|node| node.has_tag_name("svg"))
        .count();
    assert_eq!(
        svg_ancestors, 1,
        "전체 원본 GIF를 자르기 창으로 감싸면 안 된다"
    );
}
