//! 음수 배분 간격의 원점 이동을 글리프 축소로 재사용하지 않는다.
//! 독립 Print의 개인정보20쪽: H2mjsM '개' 폭18.72CSSpx,
//! 다음 원점 전진18.56CSSpx. 글리프 폭과 전진폭이 다르다.
//! C72 Native/fresh WASM 전66쪽 최저93.77937%, 쪽수 일치 확인 후 추가했다.
//! 실제 같은 글리프를 두 간격으로 그려 모양을 비교하며 golden/hash를 고정하지 않는다.
#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::{
    paint::{LayerNode, PageLayerTree, PaintOp},
    renderer::{
        font_environment::FontEnvironment,
        layer_renderer::RasterRenderOptions,
        render_tree::{BoundingBox, RenderNode, RenderNodeType, TextRunNode},
        skia::SkiaLayerRenderer,
    },
    DocumentCore,
};

fn source_run(node: &RenderNode) -> Option<&TextRunNode> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == "개인정보 보호 관련 자료의 관리" {
            return Some(run);
        }
    }
    node.children.iter().find_map(source_run)
}

#[test]
fn negative_distribution_keeps_the_native_glyph_shape() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/task2097/75544_pii_bunseok.hwpx"))
            .expect("정식 개인정보 원본"),
    )
    .unwrap();
    let environment = FontEnvironment::from_json(include_str!(
        "../../mydocs/pr/assets/issue7445/resolution-20261005/pii5846-reference-font-environment.json"
    ))
    .unwrap();
    core.set_font_environment(Some(environment)).unwrap();
    let tree = core.build_page_render_tree(19).unwrap();
    let source = source_run(&tree.root).expect("원문 표219/셀2/문단9의 글줄");
    assert!(source.style.extra_char_spacing < 0.0);
    assert_eq!(source.style.letter_spacing, 0.0);
    let render = |extra| {
        let mut run = source.clone();
        run.text = "개".into();
        run.style.extra_char_spacing = extra;
        run.style.underline = rhwp::model::style::UnderlineType::None;
        run.layout_positions = None;
        let tree = PageLayerTree::new(
            60.0,
            48.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 60.0, 48.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(8.0, 8.0, 40.0, 32.0),
                    run,
                )],
            ),
        );
        SkiaLayerRenderer::new()
            .render_pixels_with_options(
                &tree,
                RasterRenderOptions {
                    scale: 4.0,
                    ..Default::default()
                },
            )
            .unwrap()
    };
    let natural = render(0.0);
    let spaced = render(source.style.extra_char_spacing);
    assert!(natural.bytes.chunks_exact(4).any(|pixel| pixel[3] > 0));
    let changed = natural
        .bytes
        .chunks_exact(4)
        .zip(spaced.bytes.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        changed, 0,
        "독립 Print처럼 다음 원점을 좁혀도 실제 글리프의 자연 모양은 유지해야 한다"
    );
}
