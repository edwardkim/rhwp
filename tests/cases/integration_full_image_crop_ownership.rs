//! 관계형: 원본 전체 그림은 불필요한 자르기 viewport 없이 소유되어야 한다.
//! 주석 갱신: 2026-11-01 01:25 KST. 새 파일이므로 직전 파일 변경 커밋은 없다.
//! 측정: 2026-11-01 KST, 한컴 2020 Print, 96 DPI, 색상 경계 보정 전 2px 실루엣.
//! `exam-social-insert.hwp` 2쪽 Native 99.999505711504%, WASM 99.999529296581%;
//! 전 4쪽 최저 Native 99.717936122913%, WASM 99.708907814252%.
//! `exam-social-insert-delete.hwp` 2쪽 Native 99.999505606423%, WASM 99.999529163606%;
//! 전 4쪽 최저 Native 99.717936122913%, WASM 99.708907814252%.
//! 입력·PDF·쪽별 TSV: `mydocs/pr/assets/integration_20261031_full_image_crop/README.md`.
//! 설치 글꼴에 의존하지 않고 SVG의 그림과 viewport 소유 관계만 검사한다.

use rhwp::DocumentCore;

#[test]
fn full_source_image_is_not_wrapped_as_a_crop() {
    for relative in [
        "samples/pr8040/exam-social-insert.hwp",
        "samples/pr8040/exam-social-insert-delete.hwp",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let bytes = std::fs::read(&path).expect("원본 입력");
        let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
        let svg = core.render_page_svg_native(1).expect("둘째 쪽 렌더");
        let document = roxmltree::Document::parse(&svg).expect("SVG 파싱");
        let images: Vec<_> = document
            .descendants()
            .filter(|node| node.has_tag_name("image"))
            .collect();
        assert!(!images.is_empty(), "원본 그림 보존: {relative}");
        for image in images {
            let svg_ancestors = image
                .ancestors()
                .filter(|node| node.has_tag_name("svg"))
                .count();
            assert_eq!(
                svg_ancestors, 1,
                "전체 원본 그림을 자르기 창으로 감싸면 안 된다: {relative}"
            );
        }
    }
}
