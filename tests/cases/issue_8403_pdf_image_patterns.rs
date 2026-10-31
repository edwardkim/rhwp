//! 관계형·고정형(독립 원본 속성) 회귀: PDF 이미지 패턴, 저장 재열기와 내용 보존.
//! 주석 갱신: 2026-10-31T14:10:29+09:00. 새 파일이므로 직전 파일 변경 커밋은 없음.
//! 측정: 2026-10-31, 96dpi print, 색상 경계 보정 전 2px raw 실루엣.
//! 제품 source 19c6b60f97, 최적화 WASM SHA-256 23de6d71ed04f0372f9346112caf7a027f9068a1d736029a7d117f9e61e0190e.
//! 문자 획/편집 PDF 각 7쪽: Native/fresh WASM 전쪽 각각 100.00%; CanvasKit 획 7쪽 100.00%.
//! 이미지 패턴 PDF 5쪽: Native/fresh WASM 각각 [100,99.98,100,100,100], 최저 99.98%.
//! 패턴 HWP 재열기 WASM 전쪽 최저 98.91%; 문자 획 양식·편집 저장은 양쪽 전쪽 100.00%.
//! 독립 기준은 생성기의 PDF 연산자와 Poppler/MuPDF 출력이다. 제품 PNG를 기준으로 쓰지 않는다.
//! 증거: mydocs/tech/investigations/issue-8403/validation/README.md 및 전쪽 TSV/PNG.
//! 절대 px 좌표를 고정하지 않는다. 이미지 위상·채움/획·q/Q·글자 편집·저장 보존 관계를 검사한다.
#![cfg(feature = "pdf-import")]
use rhwp::DocumentCore;
const PDF: &[u8] = include_bytes!("../../pdf/issue8403/image-patterns.pdf");
fn image_boxes(node: &rhwp::paint::layer_tree::LayerNode, result: &mut Vec<(f64, f64)>) {
    use rhwp::paint::{layer_tree::LayerNodeKind, paint_op::PaintOp};
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                image_boxes(child, result);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            image_boxes(child, result)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Image { bbox, .. } = op {
                    result.push((bbox.x, bbox.width));
                }
            }
        }
    }
}
fn verify(core: &DocumentCore) {
    assert_eq!(
        core.page_count(),
        5,
        "independent source PDF has five pages; every page retains its images"
    );
    for page in 0..core.page_count() {
        let svg = core.render_page_svg_native(page).unwrap();
        let xml = roxmltree::Document::parse(&svg).unwrap();
        assert!(
            xml.descendants().any(|n| n.has_tag_name("image")),
            "source pattern must become pictures on page {page}"
        );
        assert!(
            !xml.descendants()
                .filter(|n| n.has_tag_name("path"))
                .any(|n| n.attribute("fill") == Some("#000000")),
            "unsupported pattern must not retain stale black fill"
        );
    }
    // Both page-5 fills share one pattern phase despite a later 100pt CTM translation.
    // SVG image definitions can omit x; the paint instances own placement.
    let tree = core
        .build_page_layer_tree_with_profile(4, rhwp::paint::profile::RenderProfile::Print)
        .unwrap();
    let mut images = Vec::new();
    image_boxes(&tree.root, &mut images);
    let full_width = images.iter().map(|v| v.1).fold(0., f64::max);
    let mut xs: Vec<_> = images
        .iter()
        .filter(|v| (v.1 - full_width).abs() < 0.01)
        .map(|v| v.0)
        .collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup_by(|a, b| (*a - *b).abs() < 0.01);
    assert!(xs.len() >= 4, "full cells from both source fill regions");
    let period = xs[1] - xs[0];
    assert!(
        (period / full_width - 1.5).abs() < 0.02,
        "source XStep/image width is 30/20"
    );
    for x in xs.iter().skip(2) {
        let cells = (x - xs[0]) / period;
        assert!(
            (cells - cells.round()).abs() < 0.01,
            "paint-time CTM reset the shared pattern phase"
        );
    }
    let pixels: Vec<_> = core
        .document()
        .bin_data_content
        .iter()
        .filter_map(|bin| image::load_from_memory(&bin.data.load()).ok())
        .map(|im| im.to_rgba8())
        .collect();
    assert!(pixels
        .iter()
        .flat_map(|im| im.pixels())
        .any(|p| p[0] == 255 && p[1] == 0 && p[2] == 0 && p[3] > 0));
    assert!(pixels
        .iter()
        .flat_map(|im| im.pixels())
        .any(|p| p[0] == 0 && p[1] == 255 && p[2] == 0 && p[3] > 0));
    assert!(pixels
        .iter()
        .flat_map(|im| im.pixels())
        .any(|p| p[0] == 0 && p[1] == 0 && p[2] == 255 && p[3] > 0));
    assert!(
        pixels.iter().flat_map(|im| im.pixels()).any(|p| p[0] == 255
            && p[1] == 0
            && p[2] == 0
            && p[3] > 0
            && p[3] <= 128),
        "SMask and outer alpha must survive"
    );
}
#[test]
fn coloured_image_patterns_and_normal_paint_survive_both_saved_formats() {
    let core = DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    verify(&core);
    for saved in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let opened = DocumentCore::from_bytes(&saved).unwrap();
        assert_eq!(opened.page_count(), core.page_count());
        verify(&opened);
    }
}
#[test]
fn malformed_patterns_are_rejected_and_do_not_poison_next_import() {
    for (bytes, code) in [
        (
            include_bytes!("../../pdf/issue8403/zero-step.pdf").as_slice(),
            "PDF_PATTERN_UNSUPPORTED",
        ),
        (
            include_bytes!("../../pdf/issue8403/huge-tiles.pdf").as_slice(),
            "PDF_RESOURCE_LIMIT",
        ),
        (
            include_bytes!("../../pdf/issue8403/unsupported-paint.pdf").as_slice(),
            "PDF_PATTERN_UNSUPPORTED",
        ),
        (
            include_bytes!("../../pdf/issue8403/cyclic-pattern.pdf").as_slice(),
            "PDF_PATTERN_UNSUPPORTED",
        ),
        (
            include_bytes!("../../pdf/issue8403/uncoloured-pattern.pdf").as_slice(),
            "PDF_PATTERN_UNSUPPORTED",
        ),
        (
            include_bytes!("../../pdf/issue8403/shading-pattern.pdf").as_slice(),
            "PDF_PATTERN_UNSUPPORTED",
        ),
        (
            include_bytes!("../../pdf/issue8403/singular-pattern.pdf").as_slice(),
            "PDF_PATTERN_UNSUPPORTED",
        ),
    ] {
        let e = match DocumentCore::from_pdf_bytes(bytes, &Default::default()) {
            Ok(_) => panic!("invalid pattern accepted"),
            Err(e) => e,
        };
        assert!(e.to_string().contains(code), "{e}");
        verify(&DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap());
    }
}
