//! #7751: 독립 한컴 Print의 물리 종이와 바탕쪽 그림 프레임 계약.
//!
//! 입력 SHA-256: 40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f.
//! 기준: pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf
//! SHA-256: 3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593.
//! 96dpi, 좌상단 CSS px. 0206db466의 전383쪽 raw 최저 Native93.07066%,
//! fresh WASM92.98362%; 130쪽 Native98.69152%, WASM98.45828%와 직접 판독.
//! 종이 밖으로 나가는 그림의 포함 관계는 보정 전 FAIL / 보정 후 PASS다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn bounds(b: BoundingBox) -> [f64; 4] {
    [b.x, b.y, b.width, b.height]
}
fn visit<'a>(node: &'a LayerNode, out: &mut Vec<(&'a LayerNode, &'a PaintOp)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for c in children {
                visit(c, out)
            }
        }
        LayerNodeKind::ClipRect { child, .. } => visit(child, out),
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                out.push((node, op))
            }
        }
    }
}
fn source_nodes<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for c in &node.children {
        source_nodes(c, out)
    }
}
#[test]
fn master_raster_print_frame_preserves_paper_border_and_unrelated_source_frames() {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/2025 행정업무운영 편람(최종).hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let semantic = core.build_page_render_tree(129).unwrap();
    let source = core
        .build_page_layer_tree_with_profile(129, RenderProfile::Screen)
        .unwrap();
    let printed = core
        .build_page_layer_tree_with_profile(129, RenderProfile::Print)
        .unwrap();
    let mut screen_ops = vec![];
    visit(&source.root, &mut screen_ops);
    let mut print_ops = vec![];
    visit(&printed.root, &mut print_ops);
    let mut src = vec![];
    source_nodes(&semantic.root, &mut src);
    let full = |image: &rhwp::renderer::render_tree::ImageNode| {
        image.bin_data_id == 355
            && image
                .crop
                .zip(image.original_size_hu)
                .is_some_and(|(c, s)| c == (0, 0, s.0 as i32, s.1 as i32))
    };
    let original: Vec<_> = screen_ops
        .iter()
        .filter_map(|(_, op)| match op {
            PaintOp::Image { bbox, image, .. } if full(image) => Some(*bbox),
            _ => None,
        })
        .collect();
    assert_eq!(original.len(), 1);
    assert!(
        (original[0].width - semantic.root.bbox.width).abs() < 1e-6,
        "원본의 전체 종이 폭을 차지하는 raster 그림을 검사한다"
    );
    let found: Vec<_> = print_ops
        .iter()
        .filter_map(|(leaf, op)| match op {
            PaintOp::Image { bbox, image, .. } if full(image) => Some((*leaf, *bbox)),
            _ => None,
        })
        .collect();
    assert_eq!(found.len(), 1);
    let (leaf, frame) = found[0];
    assert_eq!(
        bounds(leaf.bounds),
        bounds(frame),
        "소비자가 같은 목적 프레임을 사용한다"
    );
    // 독립 PDF130쪽 /Im8은 x0,너비555pt(740px)다. 잘린 종이 밖에 테두리를 남기지 않는다.
    assert!(
        frame.x + frame.width <= printed.page_width + 1e-6,
        "master raster escapes the physical print paper: right={}, paper={}",
        frame.x + frame.width,
        printed.page_width
    );
    assert!(
        (frame.width - printed.page_width).abs() < 1e-6,
        "원본의 종이 폭 관계를 유지한다"
    );
    // 독립 PDF130쪽 /Im8의 unclipped cm 프레임(72pt→96dpi)이다.
    // 그림은 종이 높이보다 길어도 된다. 종이 폭의 포함 관계와 별도로 검사한다.
    for (actual, expected) in bounds(frame)
        .into_iter()
        .zip([0.0, 0.0, 740.0, 1014.301333])
    {
        assert!(
            (actual - expected).abs() <= 2.0,
            "130쪽 바탕쪽 그림: 실제{actual}, 독립 Print{expected}, 허용±2px"
        );
    }
    let mut body = 0;
    let mut title = 0;
    for (_, op) in &print_ops {
        match op {
            PaintOp::Image { bbox, image, .. } if image.bin_data_id == 376 => {
                let old = src
                    .iter()
                    .find(|n| matches!(&n.node_type,RenderNodeType::Image(i) if i.bin_data_id==376))
                    .unwrap();
                assert_eq!(
                    bounds(*bbox),
                    bounds(old.bbox),
                    "본문 WMF의 좌표계는 유지한다"
                );
                body += 1;
            }
            PaintOp::TextRun { bbox, run, .. } if run.text == "2025 " => {
                let old = src
                    .iter()
                    .find(|n| matches!(&n.node_type,RenderNodeType::TextRun(t) if t.text=="2025 "))
                    .unwrap();
                assert_eq!(
                    bounds(*bbox),
                    bounds(old.bbox),
                    "동적 바탕쪽 글자는 축소하지 않는다"
                );
                title += 1;
            }
            _ => {}
        }
    }
    assert_eq!(body, 1);
    assert_eq!(title, 1);
}

#[test]
fn handbook_print_media_box_keeps_logical_layout_in_both_orientations() {
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/2025 행정업무운영 편람(최종).hwp"),
    )
    .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(true);
    // 독립 Print PDF: 309 portrait555×754pt,74 landscape754×555pt.
    for (page, width_pt, height_pt, source_width_hu, source_height_hu) in [
        (0, 555.0, 754.0, 55559.0, 75402.0),
        (374, 754.0, 555.0, 75402.0, 55559.0),
    ] {
        let source = core.build_page_render_tree(page).unwrap();
        let screen = rhwp::paint::LayerBuilder::new(RenderProfile::Screen).build(&source);
        assert!((screen.page_width - source_width_hu / 75.0).abs() < 1e-6);
        assert!((screen.page_height - source_height_hu / 75.0).abs() < 1e-6);
        let printed = rhwp::paint::LayerBuilder::new(RenderProfile::Print).build(&source);
        let a = printed.root.bounds;
        let b = screen.root.bounds;
        assert_eq!(
            [a.x, a.y, a.width, a.height],
            [b.x, b.y, b.width, b.height],
            "배치와 객체 좌표는 원본 논리 종이에 남긴다"
        );
        assert!(
            (printed.page_width * 0.75 - width_pt).abs() < 1e-6,
            "Print MediaBox 너비: page={}, actual={}pt expected={}pt",
            page + 1,
            printed.page_width * 0.75,
            width_pt
        );
        assert!(
            (printed.page_height * 0.75 - height_pt).abs() < 1e-6,
            "Print MediaBox 높이"
        );
    }
}
