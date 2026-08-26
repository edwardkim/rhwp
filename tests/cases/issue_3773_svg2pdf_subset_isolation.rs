//! #3773: svg2pdf `SubsetError` 를 페이지 단위로 격리하고 경고로 강등한다.
//!
//! 한 페이지의 서브셋 실패가 문서 전체 PDF 변환을 죽이면 안 된다. 첫 실패는
//! `embed_text=false`(글리프 path)로 재시도하고, 그래도 실패하면 그 페이지만 건너뛴다.

use rhwp::renderer::pdf::{
    classify_svg2pdf_page_error, svg2pdf_invalid_image_error, svg2pdf_subset_error_stub,
    svg2pdf_to_chunk, svgs_to_pdf, svgs_to_pdf_with_to_chunk, PdfExportOptions,
    Svg2pdfPageIsolation,
};

fn page_svg(label: &str, width: u32, height: u32) -> String {
    format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}">"#,
            r#"<text x="8" y="24">{label}</text>"#,
            r#"</svg>"#,
        ),
        width = width,
        height = height,
        label = label
    )
}

fn is_isolated_page(width: f32) -> bool {
    (width - 200.0).abs() < 0.5
}

fn pdf_page_count(pdf: &[u8]) -> usize {
    let text = String::from_utf8_lossy(pdf);
    text.match_indices("/Count ")
        .filter_map(|(idx, _)| {
            text[idx + "/Count ".len()..]
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .and_then(|n| n.parse::<usize>().ok())
        })
        .max()
        .unwrap_or(0)
}

#[test]
fn issue_3773_subset_error_is_classified_as_isolated_warning() {
    let subset = svg2pdf_subset_error_stub();
    assert_eq!(
        classify_svg2pdf_page_error(&subset, true, false),
        Svg2pdfPageIsolation::RetryWithoutEmbedText
    );
    assert_eq!(
        classify_svg2pdf_page_error(&subset, false, true),
        Svg2pdfPageIsolation::SkipPage
    );
    assert_eq!(
        classify_svg2pdf_page_error(&svg2pdf_invalid_image_error(), true, false),
        Svg2pdfPageIsolation::Fatal
    );
}

#[test]
fn issue_3773_subset_error_retries_without_embed_text() {
    let pages = [
        page_svg("good-1", 120, 80),
        page_svg("subset-fail", 200, 80),
        page_svg("good-3", 120, 80),
    ];
    let pdf =
        svgs_to_pdf_with_to_chunk(&pages, &PdfExportOptions::default(), |tree, embed_text| {
            if is_isolated_page(tree.size().width()) && embed_text {
                return Err(svg2pdf_subset_error_stub());
            }
            svg2pdf_to_chunk(tree, embed_text)
        })
        .expect("한 페이지 SubsetError 가 전체 PDF 를 죽이면 안 된다");

    assert!(pdf.starts_with(b"%PDF-"), "PDF 헤더가 없다");
    assert_eq!(
        pdf_page_count(&pdf),
        3,
        "재시도 성공 페이지를 포함해야 한다"
    );
}

#[test]
fn issue_3773_subset_error_skips_only_the_failing_page() {
    let pages = [
        page_svg("good-1", 120, 80),
        page_svg("subset-fail", 200, 80),
        page_svg("good-3", 120, 80),
    ];
    let pdf =
        svgs_to_pdf_with_to_chunk(&pages, &PdfExportOptions::default(), |tree, embed_text| {
            if is_isolated_page(tree.size().width()) {
                return Err(svg2pdf_subset_error_stub());
            }
            svg2pdf_to_chunk(tree, embed_text)
        })
        .expect("실패한 페이지만 건너뛰고 나머지를 유지해야 한다");

    assert!(pdf.starts_with(b"%PDF-"), "PDF 헤더가 없다");
    assert_eq!(pdf_page_count(&pdf), 2, "SubsetError 페이지만 빠져야 한다");
}

#[test]
fn issue_3773_non_subset_error_still_aborts_document() {
    let pages = [page_svg("good-1", 120, 80), page_svg("bad-image", 200, 80)];
    let err =
        svgs_to_pdf_with_to_chunk(&pages, &PdfExportOptions::default(), |tree, embed_text| {
            if is_isolated_page(tree.size().width()) {
                return Err(svg2pdf_invalid_image_error());
            }
            svg2pdf_to_chunk(tree, embed_text)
        })
        .expect_err("SubsetError 가 아닌 변환 실패는 문서 전체 오류여야 한다");
    assert!(
        err.contains("SVG→chunk 변환 실패"),
        "예상과 다른 오류: {err}"
    );
}

#[test]
fn issue_3773_two_good_pages_still_export() {
    let pages = [page_svg("ok-1", 120, 80), page_svg("ok-2", 160, 90)];
    let pdf = svgs_to_pdf(&pages).expect("정상 페이지 PDF 변환이 실패했다");
    assert!(pdf.starts_with(b"%PDF-"));
    assert_eq!(pdf_page_count(&pdf), 2);
}
