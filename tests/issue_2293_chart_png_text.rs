//! Issue #2293: skia RawSvg 조각 래스터의 텍스트 소실 — fontdb 폰트 해석 축.
//!
//! 원인: `svg_fontdb()` 가 ①프로젝트 `ttfs/` 미로딩 ②generic 폴백을 존재
//! 확인 없이 특정 배포판 폰트명에 하드 고정 — 매칭 실패 시 resvg 가
//! 텍스트를 드롭했다 (조각에는 `<text>` 12개 정상 포함, #2292 검증 중 발견).
//!
//! 수정: PDF 경로(`renderer/pdf.rs::create_fontdb`)와 동일 규약으로
//! `ttfs/`(재귀)·WSL 윈도우 폰트 로딩 + generic 폴백을 존재 확인 체인으로.
//!
//! 검증: 제목·눈금·카테고리·범례를 각각 제외한 대조와 최종 PNG를 비교해
//! 각 원본 텍스트의 가시 출력 기여를 검사한다. 고정 좌표·bbox 비율 영역·
//! 특정 글리프 농도는 사용하지 않는다. #2292 표적(기하)과 축을 분리한다.
#![cfg(feature = "native-skia")]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/chart/라인/표식이있는꺽은선형.hwp";

/// 원본 SVG의 특정 라벨만 제외한 대조를 만든다. 위치·글꼴·다른 paint는 유지한다.
fn remove_svg_label(node: &mut rhwp::paint::LayerNode, label: &str) -> usize {
    use rhwp::paint::{LayerNodeKind, PaintOp};

    match &mut node.kind {
        LayerNodeKind::Group { children, .. } => children
            .iter_mut()
            .map(|child| remove_svg_label(child, label))
            .sum(),
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            remove_svg_label(child, label)
        }
        LayerNodeKind::Leaf { ops } => {
            let mut removed = 0;
            for op in ops {
                let PaintOp::RawSvg { raw, .. } = op else {
                    continue;
                };
                let prefix = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">"#;
                let xml = format!("{prefix}{}</svg>", raw.svg);
                let document = roxmltree::Document::parse(&xml).expect("차트 SVG");
                let ranges: Vec<_> = document
                    .descendants()
                    .filter(|node| {
                        node.is_element()
                            && node.tag_name().name() == "text"
                            && node.text() == Some(label)
                    })
                    .map(|node| node.range())
                    .collect();
                removed += ranges.len();
                for range in ranges.into_iter().rev() {
                    raw.svg
                        .replace_range(range.start - prefix.len()..range.end - prefix.len(), "");
                }
            }
            removed
        }
    }
}

/// 각 원본 라벨과 최종 가시 출력의 관계를 검사한다. 좌표·영역·획 농도는 고정하지 않는다.
#[test]
fn chart_png_renders_text_labels() {
    use rhwp::renderer::{layer_renderer::LayerRasterRenderer, skia::SkiaLayerRenderer};

    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let core = DocumentCore::from_bytes(&fs::read(repo.join(SAMPLE)).expect("read sample"))
        .expect("parse");
    assert_eq!(core.page_count(), 1, "원본 차트는 한 쪽에 속한다");
    let scene = core.build_page_layer_tree(0).expect("layer tree");
    let original = image::load_from_memory(&core.render_page_png_native(0).expect("png"))
        .expect("decode")
        .to_rgba8();
    let renderer = SkiaLayerRenderer::new();

    // 원본 HWP와 독립 Print의 제목·눈금·카테고리·범례다. SVG는 이 소비 단계의
    // 입력이며 조판 위치의 독립 정답지가 아니다. 위치는 별도의 전쪽 Sweep으로 확인한다.
    for label in [
        "차트 제목",
        "0",
        "2",
        "4",
        "6",
        "항목 1",
        "항목 2",
        "항목 3",
        "항목 4",
        "계열 1",
        "계열 2",
        "계열 3",
    ] {
        let mut without_label = scene.clone();
        assert_eq!(
            remove_svg_label(&mut without_label.root, label),
            1,
            "원본 라벨은 SVG에 한 번만 존재한다: {label}"
        );
        let control =
            image::load_from_memory(&renderer.render_png(&without_label).expect("라벨 제외 대조"))
                .expect("decode control")
                .to_rgba8();
        assert_eq!(original.dimensions(), control.dimensions());
        // 같은 출력 경로에서 해당 글자만 제외했을 때 표시가 달라져야 한다.
        // 격자·다른 라벨의 잉크로 소실된 글자의 존재를 대신할 수 없다.
        let contribution = original
            .pixels()
            .zip(control.pixels())
            .filter(|(original, control)| original != control)
            .count();
        assert!(
            contribution > 0,
            "차트 라벨이 가시 출력에 기여하지 않는다: {label} — SVG→Skia 텍스트 소실(#2293)"
        );
        eprintln!("#2293 {label}: 가시 출력 기여 {contribution}");
    }
}

// #7445: 원본 산점도의 전 302쪽 수용 후 추가한 Print 축소 경계.
// 독립 PDF·Native/fresh WASM 전302쪽 및153쪽 raw98.66795%와 review/overlay는
// mydocs/pr/assets/issue7445/completed-20261009/regression-policy-20261015/bitmap-print-20261018/ 참조.
// 현재 PDFium 전쪽 재검증과 기존 핀의 정정은 Stage75 기록을 따른다.
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
    // 원본의 7개 선은 편집 원본에서 식별한다. Print JPEG는 이미 압축된
    // 출력 payload이므로 색 번짐을 원본의 행 개수로 잘못 세지 않는다.
    // 아래 실제 paint는 resolved를 포함한 op 전체를 계속 사용한다.
    let bytes = image.data.as_deref().expect("편집 원본 그림 픽셀");
    let source = image::load_from_memory(bytes).unwrap().to_rgb8();
    fn descendants<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        out.push(node);
        for child in &node.children {
            descendants(child, out);
        }
    }
    let tree = core.build_page_render_tree(152).unwrap();
    let mut nodes = Vec::new();
    descendants(&tree.root, &mut nodes);
    let owners: Vec<_> = nodes
        .iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Image(owner) if owner.bin_data_id == image.bin_data_id)
        })
        .collect();
    assert_eq!(
        owners.len(),
        1,
        "산점도는 같은 쪽의 원본 그림 한 개에 속한다"
    );
    let RenderNodeType::Image(owner) = &owners[0].node_type else {
        unreachable!()
    };
    // 이 쪽은 현재 양 backend 모두 90% 이상이지만 98% 미만이다.
    // 과거 절대 px 네 변 대신 원본 개체의 쪽·문단·컨트롤 소유와 포함을 보호한다.
    // Print는 종이·장치 축으로 투영하므로 layout bbox와의 수치 등식은 요구하지 않는다.
    assert_eq!(
        (owner.section_index, owner.para_index, owner.control_index),
        (image.section_index, image.para_index, image.control_index),
        "질의와 paint는 같은 원본 그림 소유자를 유지한다"
    );
    assert!(bbox.x >= 0.0 && bbox.y >= 0.0 && bbox.width > 0.0 && bbox.height > 0.0);
    assert!(bbox.x + bbox.width <= layer.page_width);
    assert!(bbox.y + bbox.height <= layer.page_height);
    let text = |needle: &str| {
        nodes
            .iter()
            .find(|node| {
                matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle))
            })
            .unwrap_or_else(|| panic!("같은 쪽의 원문을 보존한다: {needle}"))
            .bbox
    };
    let source_label = text("OECD Indicators of Employment Protection");
    let caption_start = text("무기계약 근로자 해고");
    let caption_end = text("지수의 관계 산포도");
    assert!(bbox.y + bbox.height <= source_label.y);
    assert!(source_label.y + source_label.height <= caption_start.y);
    assert!(caption_start.y + caption_start.height <= caption_end.y);
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
    // 같은 독립 Print PDF의 다섯 밴드를 현재 정본 PDFium 153.0.7999.0,
    // pdfium_smooth_rgb_v1, 96 dpi로 읽은 명암 5/95분위수다. 이전
    // pdftoppm의 범위는 이 PDFium 기준 그림 자체도 거부하므로 재사용하지 않는다.
    // JPEG/축소 위상의 차이를 밴드별 독립 관측값으로 구분한다. 열의 90%에서
    // 표시를 유지하고 검게 덮은 선도 거부하도록 중앙값의 하한을 함께 검사한다.
    // 측정 원본·명령·전쪽 자격은 Stage75 검증 기록을 따른다.
    let reference_bands = [(223, 224), (226, 229), (233, 235), (219, 226), (230, 235)];
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
