//! 고정형(원본 비트맵 개수) 계약: 하위 SVG 그림의 PDF 이미지 자원 보존.
//! 주석 갱신: 2026-11-03T13:46:00+09:00. 직전 파일 변경: 2026-09-02T20:07:32+09:00,
//! 7946a457d924f7bb0f36aabd93f163449bd03d30 (#6612 → #6618).
//! Native/fresh WASM raw 측정: 비해당. PDF 이미지 사전 개수를 검사하며 쪽수·배치를 고정하지 않는다.
//! 원본 WMF 13장의 비트맵 21개를 보호한다. 문자용 Color Emoji 비트맵은 이 계약의 대상이 아니다.
//! 시스템 글꼴과 무관하게 저장소의 공개 Noto Sans KR을 명시한다. 원본 파일·이미지는 변경하지 않는다.
//! 진단·수정 전후 실행: mydocs/pr/assets/pr8187/issue6612-font-diagnostic.json.
//!
//! usvg 의 기본 하위 SVG 로더는 SVG 규격대로 참조된 SVG 안의 `<image>` 를 전부 버린다.
//! rhwp 의 WMF 변환은 비트맵을 `<svg viewBox><image href="data:image/png"/></svg>` 로 감싸므로
//! 그대로 두면 그림 자리가 빈칸이 된다. PDF 경계에서 하위 SVG 를 이미지 유지한 채 파싱한다.

use base64::Engine;
use rhwp::renderer::pdf::{svgs_to_pdf, PdfExportOptions};
use rhwp::DocumentCore;

/// 2×1 PNG(왼쪽 빨강, 오른쪽 파랑).
const PNG_2X1_BASE64: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAIAAAABCAIAAAB7QOjdAAAAD0lEQVR4nGP4z8DAwPAfAAcAAf9+CLHQAAAAAElFTkSuQmCC";

/// svg2pdf 가 래스터 그림마다 쓰는 XObject 사전 항목.
const IMAGE_XOBJECT_MARKER: &str = "/Subtype /Image";

fn count_image_xobjects(pdf: &[u8]) -> usize {
    String::from_utf8_lossy(pdf)
        .matches(IMAGE_XOBJECT_MARKER)
        .count()
}

/// rhwp 의 WMF→SVG 래퍼와 같은 꼴: viewBox 만 있는 하위 SVG 안에 비트맵 `<image>` 하나.
fn page_with_sub_svg_bitmap() -> String {
    let sub_svg = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 2">"#,
            r#"<image href="data:image/png;base64,{png}" width="4" height="2" x="0" y="0"/>"#,
            r#"</svg>"#,
        ),
        png = PNG_2X1_BASE64
    );
    let sub_svg_b64 = base64::engine::general_purpose::STANDARD.encode(sub_svg.as_bytes());
    format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100">"#,
            r#"<image href="data:image/svg+xml;base64,{sub}" x="20" y="10" width="120" height="60" preserveAspectRatio="none"/>"#,
            r#"</svg>"#,
        ),
        sub = sub_svg_b64
    )
}

#[test]
fn issue_6612_sub_svg_bitmap_reaches_pdf() {
    let pdf = svgs_to_pdf(&[page_with_sub_svg_bitmap()]).expect("PDF 변환이 실패했다");
    assert!(pdf.starts_with(b"%PDF-"), "PDF 헤더가 없다");
    assert_eq!(
        count_image_xobjects(&pdf),
        1,
        "하위 SVG 안의 비트맵이 이미지 XObject 로 남아야 한다"
    );
}

#[test]
fn issue_6612_sample14_wmf_pictures_reach_pdf() {
    // hwp3-sample14-hwp5: 그림 13장이 전부 비트맵 WMF 이고, 몇 장은 비트맵 타일을 여러 개 품어
    // 이미지 XObject 는 21개다(PyMuPDF `get_image_info` 배치 수와 같다). 수정 전에는 0개였다.
    let mut core = DocumentCore::from_bytes(include_bytes!("../../samples/hwp3-sample14-hwp5.hwp"))
        .expect("샘플 문서를 열지 못했다");
    // 이미지 보존 검사에 글자용 비트맵이 섞이지 않도록 문서의 글꼴을 명시한다.
    // PdfExportOptions의 generic fallback만 바꾸면 원본의 미설치 family 선택이 남는다.
    let mut source = core.document().clone();
    for font in source.doc_info.font_faces.iter_mut().flatten() {
        font.name = "Noto Sans KR".into();
    }
    core.set_document(source);
    let options = PdfExportOptions {
        fallback_serif: "Noto Sans KR".into(),
        fallback_sans: "Noto Sans KR".into(),
        fallback_mono: "Noto Sans KR".into(),
        font_paths: vec![std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ttfs/opensource")],
        ..PdfExportOptions::default()
    };
    let pages: Vec<u32> = (0..core.page_count()).collect();
    let pdf = core
        .render_pages_pdf_native_with_options(&pages, &options)
        .expect("PDF 변환이 실패했다");
    assert_eq!(
        count_image_xobjects(&pdf),
        21,
        "WMF 그림 13장의 비트맵 타일 21개가 모두 이미지 XObject 로 남아야 한다"
    );
}
