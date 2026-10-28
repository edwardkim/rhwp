//! #6699: 셀 안 그림과 뒤 문자열을 정렬할 때 마지막 자간을 중복 점유하지 않는다.
use rhwp::model::style::Alignment;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/table-in-tbox.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("공개 회귀 문서")).expect("HWP 파싱")
}

fn collect<'a>(node: &'a RenderNode, nodes: &mut Vec<&'a RenderNode>) {
    nodes.push(node);
    for child in &node.children {
        collect(child, nodes);
    }
}

fn logo_and_text(core: &mut DocumentCore) -> (f64, f64, f64) {
    let tree = core.build_page_render_tree(0).expect("첫 쪽 렌더 트리");
    let mut nodes = Vec::new();
    collect(&tree.root, &mut nodes);
    let images: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::Image(image) if image.bin_data_id == 8))
        .collect();
    assert_eq!(images.len(), 1, "큰 로고는 한 번만 그린다");
    let texts: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text == "충남중부권지사장"))
        .collect();
    assert_eq!(texts.len(), 1, "로고 뒤 문자열은 한 번만 그린다");
    (images[0].bbox.x, texts[0].bbox.x, images[0].bbox.width)
}

#[test]
fn logo_and_first_glyph_match_existing_hancom_pdf() {
    let mut core = core();
    let page_height = core.build_page_render_tree(0).unwrap().root.bbox.height;
    // pdf/table-in-tbox-hwp-2020.pdf, MediaBox 595x841pt.
    // 문서 높이에 맞춰 균일 배율을 적용한다. 문자열 앞 공백을 첫 한글로 오인하지 않는다.
    let scale = page_height / 841.0;
    let (logo_x, text_x, logo_width) = logo_and_text(&mut core);
    assert!(
        (logo_x - 194.654998779 * scale).abs() < 1.0,
        "로고 x={logo_x}"
    );
    assert!(
        (text_x - 250.679992676 * scale).abs() < 1.0,
        "첫 글자 x={text_x}"
    );
    assert!(
        (text_x - logo_x - logo_width - 16.0).abs() < 0.1,
        "원본 공백 유지"
    );
}

#[test]
fn positive_tracking_only_occupies_the_seven_internal_gaps() {
    for (alignment, factor) in [
        (Alignment::Left, 0.0),
        (Alignment::Center, 0.5),
        (Alignment::Right, 1.0),
    ] {
        let mut core = core();
        let mut document = core.document().clone();
        document.doc_info.para_shapes[18].alignment = alignment;
        document.doc_info.char_shapes[20].spacings = [0; 7];
        core.set_document(document.clone());
        let (base_logo, base_text, base_width) = logo_and_text(&mut core);
        for spacing in [10, 20] {
            document.doc_info.char_shapes[20].spacings = [spacing; 7];
            core.set_document(document.clone());
            let (logo_x, text_x, logo_width) = logo_and_text(&mut core);
            let shift = -7.0 * (1600.0 / 75.0) * f64::from(spacing) / 100.0 * factor;
            // 기존 run 폭은 정수 반올림된다. 1px 이동 결함보다 작은 허용치를 유지한다.
            assert!(
                (logo_x - base_logo - shift).abs() < 0.25,
                "{alignment:?}/{spacing}: 로고"
            );
            assert!(
                (text_x - base_text - shift).abs() < 0.25,
                "{alignment:?}/{spacing}: 문자"
            );
            assert!((logo_width - base_width).abs() < 0.01);
            assert!((text_x - logo_x - base_text + base_logo).abs() < 0.01);
        }
    }
}
