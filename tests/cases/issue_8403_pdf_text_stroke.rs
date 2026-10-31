//! 관계형·고정형(독립 원본 속성) 회귀: PDF 패턴·문자 획, 편집과 저장 재열기 보존.
//! 주석 갱신: 2026-10-31T16:33:18+09:00. 직전 파일 변경: 2aceae6e46, 2026-10-31T14:29:28+09:00.
//! 측정: 2026-10-31, 96dpi print, 색상 경계 보정 전 2px raw 실루엣.
//! 제품 source 6d95c45f08, 최적화 WASM SHA-256 17942ab2a5243aa9be2aea558c7449519b3607ca20368c1c94eee41cb1c52f69.
//! 문자 획/편집 PDF 각 7쪽: Native/fresh WASM 전쪽 각각 100.00%; CanvasKit 획 7쪽 100.00%.
//! 이미지 패턴 PDF 5쪽: Native/fresh WASM 각각 [100,99.98,100,100,100], 최저 99.98%.
//! 패턴 HWP/HWPX 재열기와 CanvasKit도 전쪽 최저 99.98%; 문자 획·편집 저장은 양쪽 전쪽 100.00%.
//! 패턴 3쪽은 양쪽 raw 100.00%와 직접 색상 판독 완료. 반쪽 타일의 원본 열 선택을 추가 보호한다.
//! 독립 기준은 생성기의 PDF 연산자와 Poppler/MuPDF 출력이다. 제품 PNG를 기준으로 쓰지 않는다.
//! 증거: mydocs/tech/investigations/issue-8403/validation/README.md 및 전쪽 TSV/PNG.
//! 절대 페이지 px를 고정하지 않는다. 소유한 타일 내부의 원본 색상·선택 폭과 내용 보존을 검사한다.
#![cfg(feature = "pdf-import")]
use rhwp::DocumentCore;
const PDF: &[u8] = include_bytes!("../../pdf/issue8403/text-stroke.pdf");
fn assert_painted_and_plain_text(core: &DocumentCore, word: &str) {
    assert_eq!(
        core.page_count(),
        7,
        "independent source PDF has seven pages; content is checked on each"
    );
    for page in 0..core.page_count() {
        let svg = core.render_page_svg_native(page).unwrap();
        let xml = roxmltree::Document::parse(&svg).unwrap();
        let mut painted = String::new();
        let mut plain = String::new();
        for n in xml
            .descendants()
            .filter(|n| n.is_text() && n.ancestors().any(|a| a.has_tag_name("text")))
        {
            let stroked = n
                .ancestors()
                .any(|a| a.attribute("paint-order") == Some("fill stroke"));
            if stroked {
                let source_color = match page {
                    1 | 2 => "#0000cc",
                    3 => "#006600",
                    _ => "#000000",
                };
                assert_eq!(
                    n.ancestors()
                        .find_map(|a| a.attribute("stroke"))
                        .unwrap()
                        .to_ascii_lowercase(),
                    source_color
                );
                let fill = n.ancestors().find_map(|a| a.attribute("fill")).unwrap();
                assert_eq!(
                    fill == "none",
                    matches!(page, 1 | 5),
                    "source Tr controls fill, independent of font weight"
                );
                if page == 2 {
                    let fill_alpha: f64 = n
                        .ancestors()
                        .find_map(|a| a.attribute("fill-opacity"))
                        .unwrap()
                        .parse()
                        .unwrap();
                    let stroke_alpha: f64 = n
                        .ancestors()
                        .find_map(|a| a.attribute("stroke-opacity"))
                        .unwrap()
                        .parse()
                        .unwrap();
                    assert!((fill_alpha - 0.5).abs() <= 1.0 / 255.0);
                    assert!((stroke_alpha - 0.75).abs() <= 1.0 / 255.0);
                }
                painted.push_str(n.text().unwrap());
            } else {
                plain.push_str(n.text().unwrap());
            }
        }
        assert_eq!(
            painted
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>(),
            word,
            "stroke glyph content page {page}"
        );
        assert_eq!(
            plain
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect::<String>(),
            word,
            "q/Q plain control page {page}"
        );
    }
}
#[test]
fn pdf_text_stroke_remains_editable_paint_through_both_save_formats() {
    let core = DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    assert_painted_and_plain_text(&core, "Stroke한글");
    // The fixture's embedded OpenType name ID 1, not its PDF subset label.
    assert!(core
        .document()
        .doc_info
        .font_faces
        .iter()
        .flatten()
        .any(|font| font.name == "Noto Sans KR"));
    for saved in [
        rhwp::serializer::cfb_writer::serialize_hwp_with_report(core.document()).unwrap(),
        rhwp::serializer::hwpx::serialize_hwpx_with_report(core.document()).unwrap(),
    ] {
        use rhwp::serializer::content_loss::{ContentLossCode, ContentLossReason};
        assert!(saved
            .content_loss()
            .losses()
            .iter()
            .any(|loss| loss.code == ContentLossCode::MetadataReduced
                && loss.reason == ContentLossReason::UnsupportedByOutputFormat));
        assert_painted_and_plain_text(
            &DocumentCore::from_bytes(saved.bytes()).unwrap(),
            "Stroke한글",
        );
    }
    let paints: Vec<_> = core
        .document()
        .doc_info
        .char_shapes
        .iter()
        .filter_map(|s| s.text_paint)
        .collect();
    assert!(paints
        .iter()
        .any(|p| p.fill && p.width_mpt == 1715 && p.stroke_color == 0));
    assert!(paints
        .iter()
        .any(|p| !p.fill && p.width_mpt == 1200 && p.stroke_color == 0xcc0000));
    assert!(paints.iter().any(|p| p.fill
        && p.width_mpt == 1100
        && p.fill_alpha == 128
        && p.stroke_alpha == 191
        && p.line_join == 1
        && p.miter_limit_milli == 3000));
    assert!(paints.iter().any(|p| !p.fill && p.width_mpt == 0));
    assert!(paints.iter().any(|p| p.fill && p.width_mpt == 2058));
    assert!(
        core.document()
            .doc_info
            .char_shapes
            .iter()
            .filter(|s| s.text_paint.is_some())
            .all(|s| !s.bold),
        "Tr is not a request for a bold font"
    );
    for saved in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = DocumentCore::from_bytes(&saved).unwrap();
        assert_eq!(reopened.page_count(), core.page_count());
        assert_eq!(
            reopened
                .document()
                .doc_info
                .char_shapes
                .iter()
                .filter_map(|s| s.text_paint)
                .collect::<Vec<_>>(),
            paints
        );
        assert_painted_and_plain_text(&reopened, "Stroke한글");
    }
}
#[test]
fn unsupported_text_paint_is_an_explicit_error_and_next_request_recovers() {
    for bytes in [
        include_bytes!("../../pdf/issue8403/dashed-text.pdf").as_slice(),
        include_bytes!("../../pdf/issue8403/anisotropic-text.pdf").as_slice(),
        include_bytes!("../../pdf/issue8403/clipped-text-stroke.pdf").as_slice(),
        include_bytes!("../../pdf/issue8403/pattern-text.pdf").as_slice(),
    ] {
        let error = match DocumentCore::from_pdf_bytes(bytes, &Default::default()) {
            Ok(_) => panic!("unsupported paint accepted"),
            Err(e) => e,
        };
        assert!(
            error.to_string().contains("PDF_TEXT_UNSUPPORTED"),
            "{error}"
        );
        let restored = DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
        assert_painted_and_plain_text(&restored, "Stroke한글");
    }
}

#[test]
fn edited_glyphs_repaint_the_stroke_instead_of_retaining_a_frozen_overlay() {
    let mut core = DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    core.replace_all_native("Stroke", "ktroke", true).unwrap();
    assert_painted_and_plain_text(&core, "ktroke한글");
    for saved in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_painted_and_plain_text(&DocumentCore::from_bytes(&saved).unwrap(), "ktroke한글");
    }
}
