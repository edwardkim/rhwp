//! #8153: WMF Print의 전체 원본·물리 crop·소유와 독립 PDF ±2px 경계를 보호한다.
//! Native/fresh DEV WASM 36bae7c0ed의 두 원본 130쪽 raw는 각각 99.96446/99.96402다.
//! 96dpi·입력/PDF SHA·전체 Native/핵심 WASM TSV·확대 판독과 수정 전후 근거는
//! `mydocs/plans/issue_8153_wmf_print_raster_20261027.md`에 연결한다.
//! PDF 130쪽의 전체 이미지 matrix와 별도 clip에서 읽은 독립 96dpi 경계다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{
        ClipKind, LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImageKind,
        ResolvedImagePayload,
    },
    renderer::render_tree::{BoundingBox, ImageNode},
    DocumentCore,
};
struct Figure<'a> {
    bounds: &'a BoundingBox,
    image: &'a ImageNode,
    payload: &'a ResolvedImagePayload,
    crop: Option<&'a BoundingBox>,
    body: bool,
    owner_preserved: bool,
}
fn collect<'a>(
    node: &'a LayerNode,
    container: Option<&'a LayerNode>,
    body: bool,
    figures: &mut Vec<Figure<'a>>,
) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect(child, container, body, figures)
            }
        }
        LayerNodeKind::ClipRect {
            clip_kind, child, ..
        } => collect(
            child,
            if *clip_kind == ClipKind::Generic {
                Some(node)
            } else {
                container
            },
            body || *clip_kind == ClipKind::Body,
            figures,
        ),
        LayerNodeKind::ClipPath { child, .. } => collect(child, container, body, figures),
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Image {
                    bbox,
                    image,
                    resolved,
                } = op
                {
                    if image.bin_data_id == 376 {
                        let crop = container.and_then(|n| {
                            if let LayerNodeKind::ClipRect { clip, .. } = &n.kind {
                                Some(clip)
                            } else {
                                None
                            }
                        });
                        figures.push(Figure {
                            bounds: bbox,
                            image,
                            payload: resolved.as_deref().expect("실제 Print 장치 자원"),
                            crop,
                            body,
                            owner_preserved: container
                                .is_some_and(|parent| parent.source_node_id == node.source_node_id)
                                && node.source_node_id.is_some(),
                        });
                    }
                }
            }
        }
    }
}
fn near(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print {label}: {actual} vs {expected}"
    )
}
fn verify(extension: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let before = core.export_hwp_native().unwrap();
    let tree = core
        .build_page_layer_tree_with_profile(129, RenderProfile::Print)
        .unwrap();
    let mut figures = Vec::new();
    collect(&tree.root, None, false, &mut figures);
    assert_eq!(figures.len(), 1, "그림 누락·중복 금지");
    let figure = &figures[0];
    assert!(
        figure.body && figure.owner_preserved,
        "본문·그림 원본 소유를 물리 clip과 함께 보존한다"
    );
    assert!(figure.image.section_index.is_some() && figure.image.para_index.is_some());
    assert!(
        figure.image.crop.is_none() && figure.image.content_inset.is_none(),
        "물리 clip 뒤 원본 창을 다시 자르지 않는다"
    );
    assert_eq!(figure.image.brightness, -5, "원본 명암 메타정보 보존");
    assert_eq!(figure.payload.kind, ResolvedImageKind::OutputDevice);
    assert!(figure.payload.suppress_effects);
    assert_eq!(figure.payload.mime, "image/png");
    let clip = figure.crop.expect("원본 crop의 물리 clip");
    let full = figure.bounds;
    assert!(
        clip.y > full.y && clip.y + clip.height <= full.y + full.height,
        "위쪽 crop과 나머지 원본 포함 관계"
    );
    assert!(
        clip.x >= full.x - 1e-8 && clip.x + clip.width <= full.x + full.width + 1e-8,
        "가로 내용 포함 관계"
    );
    // 독립 PDF의 359.611×268.294pt image matrix와 116.154pt 원점을 환산했다.
    for (value, expected, label) in [
        (full.x, 154.872, "전체 원본 왼쪽"),
        (full.y, 460.5946666666667, "전체 원본 위"),
        (full.x + full.width, 634.3533333333334, "전체 원본 오른쪽"),
        (full.y + full.height, 818.32, "전체 원본 아래"),
        (clip.x, 154.872, "crop 왼쪽"),
        (clip.y, 463.31466666666665, "crop 위"),
        (clip.x + clip.width, 634.3546666666666, "crop 오른쪽"),
        (clip.y + clip.height, 818.16, "crop 아래"),
    ] {
        near(value, expected, label)
    }
    assert_eq!(
        before,
        core.export_hwp_native().unwrap(),
        "출력 자원은 원본 문서·순서·데이터를 변경하지 않는다"
    );
}
#[test]
fn hwp_wmf_print_keeps_full_source_physical_crop_and_owner() {
    verify("hwp")
}
#[test]
fn hwpx_wmf_print_keeps_full_source_physical_crop_and_owner() {
    verify("hwpx")
}
