//! Issue #2293: skia RawSvg 조각 래스터의 텍스트 소실 — fontdb 폰트 해석 축.
//!
//! 원인: `svg_fontdb()` 가 ①프로젝트 `ttfs/` 미로딩 ②generic 폴백을 존재
//! 확인 없이 특정 배포판 폰트명에 하드 고정 — 매칭 실패 시 resvg 가
//! 텍스트를 드롭했다 (조각에는 `<text>` 12개 정상 포함, #2292 검증 중 발견).
//!
//! 수정: PDF 경로(`renderer/pdf.rs::create_fontdb`)와 동일 규약으로
//! `ttfs/`(재귀)·WSL 윈도우 폰트 로딩 + generic 폴백을 존재 확인 체인으로.
//!
//! 검증: 차트 PNG 의 제목·축 라벨 영역에 잉크가 존재해야 한다 (수정 전
//! FAILED). 환경별 폰트 차이를 허용하기 위해 잉크 존재만 단언하고 특정
//! 글리프 픽셀 비교는 하지 않는다. #2292 표적(기하)과 축 분리.
#![cfg(feature = "native-skia")]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/chart/라인/표식이있는꺽은선형.hwp";

fn find_chart_bbox(node: &RenderNode) -> Option<(f64, f64, f64, f64)> {
    if let RenderNodeType::RawSvg(_) = &node.node_type {
        return Some((node.bbox.x, node.bbox.y, node.bbox.width, node.bbox.height));
    }
    node.children.iter().find_map(find_chart_bbox)
}

fn ink_in_region(img: &image::RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32) -> usize {
    let mut ink = 0;
    for y in y0..y1.min(img.height()) {
        for x in x0..x1.min(img.width()) {
            let p = img.get_pixel(x, y);
            if p.0[3] > 8 && (p.0[0] < 200 || p.0[1] < 200 || p.0[2] < 200) {
                ink += 1;
            }
        }
    }
    ink
}

/// 차트 텍스트(제목 상단 밴드 + 카테고리 라벨 하단 밴드)가 래스터에 그려진다.
#[test]
fn chart_png_renders_text_labels() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = fs::read(repo.join(SAMPLE)).expect("read sample");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");

    let tree = core.build_page_render_tree(0).expect("tree");
    let (bx, by, bw, bh) = find_chart_bbox(&tree.root).expect("RawSvg(차트) 부재");

    let png = core.render_page_png_native(0).expect("png");
    let img = image::load_from_memory(&png).expect("decode").to_rgba8();
    let scale = img.width() as f64 / tree.root.bbox.width;
    let px = |v: f64| (v * scale).round() as u32;

    // 제목 밴드: bbox 상단 12% (조각 실측 — "차트 제목" y≈150/bbox y=132..382).
    // 기하(플롯 프레임 상단선)가 걸치지 않도록 좌우 25% 인셋의 중앙부만 본다.
    let title = ink_in_region(
        &img,
        px(bx + bw * 0.25),
        px(by + bh * 0.02),
        px(bx + bw * 0.75),
        px(by + bh * 0.12),
    );

    // 카테고리 라벨 밴드: 플롯 하단과 bbox 사이 (조각 실측 — "항목 N"
    // y≈374/bbox 하단 382). 하단 6% 밴드.
    let category = ink_in_region(
        &img,
        px(bx + bw * 0.1),
        px(by + bh * 0.94),
        px(bx + bw * 0.9),
        px(by + bh),
    );

    assert!(
        title > 0,
        "차트 제목 밴드 잉크 0 — 조각 래스터 텍스트 소실(#2293)"
    );
    assert!(
        category > 0,
        "카테고리 라벨 밴드 잉크 0 (title={title}) — 조각 래스터 텍스트 소실(#2293)"
    );
}

// #7445: 원본 PNG 산점도의 전302쪽 수용 후 추가한 Print 축소 경계.
// 독립 PDF·Native/fresh WASM 전302쪽 및153쪽 raw98.66795%와 review/overlay는
// mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/bitmap-print-20261018/ 참조.
#[test]
fn reduced_scatter_plot_preserves_every_interior_grid_band() {
    use rhwp::paint::{LayerNode, LayerNodeKind, PageLayerTree, PaintOp, RenderProfile};
    use rhwp::renderer::{
        layer_renderer::{LayerRasterRenderer, RasterRenderOptions},
        skia::SkiaLayerRenderer,
    };

    fn image_op(node: &LayerNode) -> Option<&PaintOp> {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children.iter().find_map(image_op),
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                image_op(child)
            }
            LayerNodeKind::Leaf { ops } => {
                ops.iter().find(|op| matches!(op, PaintOp::Image { .. }))
            }
        }
    }

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5941/1490000-201600081_roadmap_research.hwp");
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    // 이 원본의 독립 기준은 한컴2020 Print다. 2024 호환 모드로 대체하지 않는다.
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 302);
    let layer = core
        .build_page_layer_tree_with_profile(152, RenderProfile::Print)
        .unwrap();
    let op = image_op(&layer.root)
        .expect("독립 Print153쪽 산점도")
        .clone();
    let PaintOp::Image {
        bbox,
        image,
        resolved,
    } = &op
    else {
        unreachable!()
    };
    let bytes = resolved
        .as_deref()
        .map(|p| p.data.as_slice())
        .or(image.data.as_deref())
        .expect("원본 그림 픽셀");
    let source = image::load_from_memory(bytes).unwrap().to_rgb8();
    // PDF153쪽의 실제 이미지 사각형(pt)을96dpi CSS px로 환산했다.
    // 그림과 내부 격자를 같은 개체로 식별한 뒤 네 변을 함께 보호한다.
    let [x, y, width, height] = [121.374664, 146.560018, 558.740000, 430.089315];
    let reference_edges = [x, y, x + width, y + height];
    for (actual, reference) in [bbox.x, bbox.y, bbox.x + bbox.width, bbox.y + bbox.height]
        .into_iter()
        .zip(reference_edges)
    {
        assert!(
            (actual - reference).abs() <= 2.0,
            "독립 Print 그림 경계: {actual} vs {reference}"
        );
    }
    let (width, height) = source.dimensions();
    let (first, end) = (width / 4, width * 3 / 4);
    let gray = |p: &image::Rgb<u8>| {
        let [r, g, b] = p.0;
        r.abs_diff(g) <= 2 && g.abs_diff(b) <= 2 && r < 240
    };
    // 원본의 중앙 절반을 가로지르는 회색 줄만 추출한다. 축 바깥 그림
    // 테두리·점/문자·색 선은 제외하며 y 좌표 목록을 기대값으로 복사하지 않는다.
    let rows: Vec<_> = (1..height - 1)
        .filter(|&y| {
            (first..end)
                .filter(|&x| gray(source.get_pixel(x, y)))
                .count()
                * 10
                >= (end - first) as usize * 9
        })
        .collect();
    assert_eq!(
        rows.len(),
        7,
        "축 테두리2개와 내부 격자5개가 같은 그림에 속한다"
    );
    let scene = PageLayerTree::with_profile(
        layer.page_width,
        layer.page_height,
        LayerNode::leaf(*bbox, None, vec![op.clone()]),
        RenderProfile::Print,
    );
    let png = SkiaLayerRenderer::new()
        .render_png_with_options(
            &scene,
            RasterRenderOptions {
                transparent: false,
                ..Default::default()
            },
        )
        .unwrap();
    let output = image::load_from_memory(&png).unwrap().to_rgb8();
    let sx = bbox.width / f64::from(width);
    let sy = bbox.height / f64::from(height);
    // 첫/끝 줄은 축 테두리다. 내부의 반복 격자를 모두 검사한다.
    // 독립 Print를 pdftoppm96dpi로 읽은 같은5개 밴드의 명암5/95분위수.
    // JPEG/축소 위상 때문에 네 번째 줄만 더 옅다. 모든 줄에232를 적용하면
    // 기준 PDF 자체도198/272열(72.8%)만 통과하므로 잘못된 기대값이다.
    // 나머지 네 줄의 상한은 기존232보다 엄격하다. 검은 선으로 덮어서
    // 통과하지 못하도록 중앙값의 하한도 독립 PDF의5분위수로 검사한다.
    let reference_bands = [(215, 218), (215, 218), (215, 221), (224, 238), (215, 222)];
    for (&y, (dark_limit, light_limit)) in rows
        .iter()
        .skip(1)
        .take(rows.len() - 2)
        .zip(reference_bands)
    {
        let center = (bbox.y + (f64::from(y) + 0.5) * sy - 0.5).round() as u32;
        let left = (bbox.x + f64::from(first) * sx).round() as u32;
        let right = (bbox.x + f64::from(end) * sx).round() as u32;
        let mut preserved = 0;
        let mut sampled = 0;
        let mut levels = Vec::new();
        for x in left..right {
            let source_x = ((f64::from(x) + 0.5 - bbox.x) / sx).floor() as u32;
            let original = source.get_pixel(source_x.min(width - 1), y);
            if !gray(original) {
                continue;
            }
            // 원본 회색 행이 있는 열에서 ±2CSSpx의 RGB 최대 성분 최솟값을
            // 읽는다. JPEG의 색 번짐도 독립 PDF와 같은 조건으로 평가한다.
            let level = (center.saturating_sub(2)..=center + 2)
                .map(|row| output.get_pixel(x, row).0.into_iter().max().unwrap())
                .min()
                .unwrap_or(255);
            sampled += 1;
            preserved += usize::from(level <= light_limit);
            levels.push(level);
        }
        assert!(
            sampled > 0 && preserved * 10 >= sampled * 9,
            "원본 grid 행{y}: 신호를 보존한 열{preserved}/{sampled}; 원본 격자선의 연속 표시 유지"
        );
        levels.sort_unstable();
        assert!(
            levels[levels.len() / 2] >= dark_limit,
            "원본 grid 행{y}: 독립 Print보다 검게 덮지 않는다"
        );
    }
}
