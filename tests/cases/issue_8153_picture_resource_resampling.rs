//! #8153: 선언 크기의 두 축 재표본화·이진 coverage·원본 소유·독립 Print 경계.
//! Native 0fcbd1e42e / 표준 최적화 fresh WASM 70e060edad의 두 원본 144쪽 raw 100%·미대응 0.
//! 같은 런의 두 원본 전쪽 최저는 Native 99.96446%, WASM 99.96402%다.
//! 원본/PDF SHA·96dpi·TSV·확대 판독·독립 정수 경계 대조는
//! mydocs/plans/issue_8153_picture_device_stage_20261027.md에 연결한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    paint::{
        ClipKind, LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImageKind,
        ResolvedImagePayload,
    },
    renderer::render_tree::{BoundingBox, ImageNode},
    DocumentCore,
};
struct Picture<'a> {
    bounds: &'a BoundingBox,
    image: &'a ImageNode,
    payload: &'a ResolvedImagePayload,
    body: bool,
    owner: Option<u32>,
}
fn collect<'a>(node: &'a LayerNode, body: bool, images: &mut Vec<Picture<'a>>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect(child, body, images)
            }
        }
        LayerNodeKind::ClipRect {
            clip_kind, child, ..
        } => collect(child, body || *clip_kind == ClipKind::Body, images),
        LayerNodeKind::ClipPath { child, .. } => collect(child, body, images),
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Image {
                    bbox,
                    image,
                    resolved,
                } = op
                {
                    if [389, 390].contains(&image.bin_data_id) {
                        images.push(Picture {
                            bounds: bbox,
                            image,
                            payload: resolved.as_deref().expect("Print 자원"),
                            body,
                            owner: node.source_node_id,
                        });
                    }
                }
            }
        }
    }
}
fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2.,
        "독립 Print 96dpi 경계: {actual} vs {expected}"
    )
}
fn verify(extension: &str) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    let original = core.export_hwp_native().unwrap();
    let tree = core
        .build_page_layer_tree_with_profile(143, RenderProfile::Print)
        .unwrap();
    let mut pictures = Vec::new();
    collect(&tree.root, false, &mut pictures);
    assert_eq!(pictures.len(), 2, "두 원본 그림 누락·중복 금지");
    pictures.sort_by_key(|p| p.image.bin_data_id);
    // 독립 Print PDF의 이미지 자원 크기와 표시 사각형에서 읽은 기대값이다.
    for (picture, source_size, device_size, expected) in [
        (
            &pictures[0],
            (860, 769),
            (1214, 607),
            [
                133.13599650065103,
                722.9693196614584,
                232.86799112955728,
                116.46931966145833,
            ],
        ),
        (
            &pictures[1],
            (937, 771),
            (1181, 604),
            [
                392.5360107421875,
                723.1293131510416,
                226.6346435546875,
                115.98933919270833,
            ],
        ),
    ] {
        assert!(
            picture.body && picture.owner.is_some(),
            "본문과 원래 그림 소유 보존"
        );
        assert!(picture.image.section_index.is_some() && picture.image.para_index.is_some());
        let source = picture.image.data.as_deref().unwrap();
        assert!(
            source.starts_with(&[0xff, 0xd8]),
            "원본 JPEG를 편집하지 않는다"
        );
        let decoded = image::load_from_memory(source).unwrap();
        assert_eq!((decoded.width(), decoded.height()), source_size);
        assert_eq!(picture.payload.kind, ResolvedImageKind::OutputDevice);
        assert_eq!(picture.payload.mime, "image/png");
        let device = image::load_from_memory(&picture.payload.data)
            .unwrap()
            .into_rgba8();
        assert_eq!(device.dimensions(), device_size);
        assert!(
            device_size.0 > source_size.0 && device_size.1 < source_size.1,
            "한 축 축소와 다른 축 확대를 함께 보존"
        );
        assert!(
            picture
                .payload
                .data
                .windows(6)
                .any(|bytes| bytes == b"rhWP\x04\x01"),
            "공통 별도 평면 표본 계약"
        );
        assert!(device.pixels().all(|p| p[3] == 0 || p[3] == 255));
        assert!(
            device.pixels().any(|p| p[3] == 0) && device.pixels().any(|p| p[3] == 255),
            "투명 경계를 흰 배경으로 덮지 않는다"
        );
        // 독립 Print의 별도 1비트 마스크에서 읽은 자원 픽셀 표본이다.
        // 왼쪽·오른쪽 투명 경계와 바로 옆 불투명 픽셀을 함께 보호한다.
        let width = device_size.0;
        let height = device_size.1;
        let right_top = if picture.image.bin_data_id == 389 {
            0
        } else {
            255
        };
        for (x, y, alpha) in [
            (0, 0, 255),
            (0, 3, 0),
            (1, 3, 255),
            (width - 1, 2, right_top),
            (width - 2, 2, 255),
            (width - 1, height - 1, 0),
            (width - 2, height - 1, 255),
            (width / 2, height - 1, 255),
        ] {
            assert_eq!(
                device.get_pixel(x, y)[3],
                alpha,
                "독립 Print coverage ({x},{y})"
            );
        }
        let bbox = picture.bounds;
        for (actual, expected) in [bbox.x, bbox.y, bbox.width, bbox.height]
            .into_iter()
            .zip(expected)
        {
            near(actual, expected)
        }
    }
    assert_ne!(
        pictures[0].owner, pictures[1].owner,
        "두 그림의 원본 소유를 합치지 않는다"
    );
    assert!(
        pictures[0].bounds.x + pictures[0].bounds.width < pictures[1].bounds.x,
        "앞뒤 그림의 순서와 분리 보존"
    );
    assert!(
        pictures
            .iter()
            .all(|p| p.bounds.y >= 0. && p.bounds.y + p.bounds.height <= tree.page_height),
        "그림의 페이지 포함"
    );
    assert_eq!(
        core.export_hwp_native().unwrap(),
        original,
        "Print 재표본화가 원본 속성·바이트를 바꾸지 않는다"
    );
}
#[test]
fn hwp_picture_resource_preserves_both_axes_and_owner() {
    verify("hwp")
}
#[test]
fn hwpx_picture_resource_preserves_both_axes_and_owner() {
    verify("hwpx")
}
