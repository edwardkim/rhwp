//! 관계형: 음수 배분 간격의 원점 이동을 글리프 축소로 재사용하지 않는다.
//! 주석 갱신: 2026-11-05 11:40 (+0900). 직전 파일 변경 6097253468 (2026-10-24 15:11:35 +0900).
//! 실제 같은 글리프를 두 간격으로 그려 모양을 비교하며 golden/hash를 고정하지 않는다.
//! 원본 근거: 독립 Print `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf` 20쪽
//! '2. 개인정보 보호 관련 자료의 관리'(H2mjsM 14.04pt=18.72CSSpx). 한글 원점 전진은
//! 18.56/18.72CSSpx 가 번갈아 나오는 원점 양자화(평균 18.66)로 글자 자체는 압축되지 않고,
//! 내부 공백 전진이 7.52~7.68CSSpx 로 앞머리 공백 9.28CSSpx 보다 좁다(PyMuPDF rawdict 원점,
//! 2026-11-05 측정). #8389 이후 rhwp 도 이 줄의 넘침을 내부 공백에 배정해 배분 간격이 0 이다.
//! 이전 판의 '다음 원점 전진 18.56 = 글리프 폭과 다른 전진' 해석은 양자화 오독이라 정정했다(#8613).
//! 측정 이력: C72 Native/fresh WASM 전66쪽 최저93.77937%, 쪽수 일치 확인 후 추가했다.
//! 렌더러 계약 검사는 원본 글줄 스타일에 음수 배분 간격을 명시해 넣고, 그리는 프로그램은
//! 설치 글꼴이 아니라 저장소 OFL `ttfs/opensource/NotoSansKR-Regular.ttf` 로 공급한다.
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
    // 독립 Print 처럼 이 줄의 넘침은 글자가 아니라 내부 공백이 흡수한다.
    assert_eq!(
        source.style.extra_char_spacing, 0.0,
        "독립 Print 의 한글 전진은 글자 크기와 같고 내부 공백만 좁아진다"
    );
    assert_eq!(source.style.letter_spacing, 0.0);
    // 렌더러 계약용 음수 배분 간격(#8389 이전 이 줄의 값과 같은 크기).
    let negative_distribution = -0.394;
    let render = |extra| {
        let mut run = source.clone();
        run.text = "개".into();
        run.style.font_family = "Noto Sans KR".into();
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
            .try_with_font_data(&[
                include_bytes!("../../ttfs/opensource/NotoSansKR-Regular.ttf").to_vec(),
            ])
            .unwrap()
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
    let spaced = render(negative_distribution);
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
