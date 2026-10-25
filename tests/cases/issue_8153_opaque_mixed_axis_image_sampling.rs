// #8153: 독립 2024 Print의 그림 틀 모서리와 닫힘 선을 실제 Native 출력으로 보호한다.
// 96dpi CSS px, 잉크 임계값 232, 독립 PDF 점의 허용 오차 ±2px다.
// 근거와 Native/fresh WASM TSV는 issue_8153_mixed_axis_bitmap_sampling_20261026.md 참조.
// tests/cases의 정식 suite에서 native-skia feature로 실행한다.

#[cfg(feature = "native-skia")]
use rhwp::{
    document_core::queries::{png_export::NativePngExporter, rendering::PngExportOptions},
    model::style::ImageFillMode,
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    renderer::{
        layer_renderer::RasterRenderOptions, raster_pixels::RasterPixels, render_tree::BoundingBox,
        skia::SkiaLayerRenderer,
    },
    DocumentCore,
};

#[cfg(feature = "native-skia")]
fn frame_parts(node: &LayerNode, parts: &mut Vec<(u32, BoundingBox)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                frame_parts(child, parts);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            frame_parts(child, parts);
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Image {
                    bbox,
                    image,
                    resolved: Some(payload),
                } = op
                {
                    if matches!(
                        image.fill_mode,
                        Some(ImageFillMode::FitToSize | ImageFillMode::Total)
                    ) && payload.mime == "image/png"
                    {
                        parts.push((
                            node.source_node_id.expect("원본 그림 조각의 소유"),
                            image.paint_bbox(bbox),
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(feature = "native-skia")]
fn has_ink_near(raster: &RasterPixels, x: usize, y: usize) -> bool {
    let opaque = raster.opaque_rgba_on_white().unwrap();
    assert!(x >= 2 && x + 2 < raster.width as usize);
    assert!(y >= 2 && y + 2 < raster.height as usize);
    (y - 2..=y + 2).any(|yy| {
        (x - 2..=x + 2).any(|xx| {
            let at = (yy * raster.width as usize + xx) * 4;
            opaque[at..at + 3].iter().any(|channel| *channel < 232)
        })
    })
}

#[cfg(feature = "native-skia")]
fn check(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    // 그림 틀 자체의 검사는 독점 글꼴의 복사·설치·고정 호스트 경로에 의존하지 않는다.
    let exporter = NativePngExporter::new(RenderProfile::Print, PngExportOptions::default());
    exporter.prepare_document(&mut core).unwrap();
    assert_eq!(core.page_count(), 383, "독립 Print 쪽수와 그림 소유 쪽");
    let tree = core
        .build_page_layer_tree_with_profile(238, RenderProfile::Print)
        .unwrap();
    let mut parts = Vec::new();
    frame_parts(&tree.root, &mut parts);
    assert_eq!(parts.len(), 8, "독립 Print의 그림 틀 여덟 조각 보존");
    let owners: std::collections::BTreeSet<_> = parts.iter().map(|part| part.0).collect();
    assert_eq!(owners.len(), 8, "조각 누락·중복 없는 서로 다른 원본 소유");
    let horizontal: Vec<_> = parts
        .iter()
        .filter(|(_, b)| b.width > 2.0 * b.height)
        .collect();
    assert_eq!(horizontal.len(), 2, "위·아래 가로 띠");
    let vertical: Vec<_> = parts
        .iter()
        .filter(|(_, b)| b.height > 2.0 * b.width)
        .collect();
    assert_eq!(vertical.len(), 2, "왼쪽·오른쪽 세로 띠");
    assert_eq!(
        parts.len() - horizontal.len() - vertical.len(),
        4,
        "네 모서리"
    );
    let raster = SkiaLayerRenderer::new()
        .render_pixels_with_options(
            &tree,
            RasterRenderOptions {
                scale: 1.0,
                dpi: Some(96.0),
                ..Default::default()
            },
        )
        .unwrap();
    // 원본 누락·소유 검사를 유지하면서 독립 Print의 모서리와 닫힘 선을 확인한다.
    for (x, y) in [(626, 623), (300, 886)] {
        assert!(
            has_ink_near(&raster, x, y),
            "{extension}: 독립 Print의 239쪽 ({x},{y}) 그림 테두리에서 ±2px 안의 잉크 보존"
        );
    }
}

#[cfg(feature = "native-skia")]
#[test]
fn hwp_original_frame_corner_and_bottom_ink_are_preserved() {
    check("hwp");
}

#[cfg(feature = "native-skia")]
#[test]
fn hwpx_original_frame_corner_and_bottom_ink_are_preserved() {
    check("hwpx");
}
