//! 검사 유형: 관계형. 준비된 WMF와 같은 배치의 PNG가 동일 Print 프레임을 소비하는지 대조한다.
//! 주석 갱신: 2026-11-09T09:39:40+09:00. 신규 검사.
//! 공개 합성 원본의 형식·좌표계 계약이며 실물 페이지의 고정 px 핀이 아니다.
//! 같은 배치의 PNG를 독립 대조군으로 사용하며 글꼴 설치에 의존하지 않는다.
use rhwp::{
    paint::{LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImageKind},
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};
fn collect(node: &LayerNode, out: &mut Vec<(u32, BoundingBox, bool)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for c in children {
                collect(c, out)
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            collect(child, out)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Image { bbox, resolved, .. } = op {
                    let prepared = resolved.as_ref().is_some_and(|p| {
                        p.kind == ResolvedImageKind::OutputDevice && p.mime == "image/png"
                    });
                    out.push((node.source_node_id.unwrap(), *bbox, prepared))
                }
            }
        }
    }
}
fn replace(node: &mut RenderNode, ids: &mut Vec<u32>) {
    if let RenderNodeType::Image(image) = &mut node.node_type {
        ids.push(node.id);
        image.data = Some(
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/issue8183/scan-source-gray-200.png"
            ))
            .to_vec(),
        );
        image.crop = None;
    }
    for c in &mut node.children {
        replace(c, ids)
    }
}
#[test]
fn prepared_metafile_surface_and_raster_share_the_print_frame() {
    for fixture in [
        "metafile-aspect-explicit-pen",
        "metafile-equal-axis-controls",
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("pdf/issue7642-metafile-palette-visibility-20261017")
            .join(format!("{fixture}.hwpx"));
        let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
        core.set_hangul2024_compat(true);
        let source = core.build_page_render_tree(0).unwrap();
        let mut control = source.clone();
        let mut ids = vec![];
        replace(&mut control.root, &mut ids);
        assert!(!ids.is_empty());
        for profile in [RenderProfile::Screen, RenderProfile::Print] {
            let mut actual = vec![];
            let mut expected = vec![];
            let mut a = LayerBuilder::new(profile)
                .with_hancom2024_print_bitmap(true)
                .with_metafile_device_pixel_width(Some(0.16));
            let mut b = LayerBuilder::new(profile)
                .with_hancom2024_print_bitmap(true)
                .with_metafile_device_pixel_width(Some(0.16));
            collect(&a.build(&source).root, &mut actual);
            collect(&b.build(&control).root, &mut expected);
            actual.sort_by_key(|v| v.0);
            expected.sort_by_key(|v| v.0);
            assert_eq!(actual.len(), ids.len());
            assert_eq!(actual.len(), expected.len());
            for (a, b) in actual.iter().zip(&expected) {
                assert_eq!(a.0, b.0);
                if profile == RenderProfile::Print {
                    assert!(a.2, "WMF를 실제 출력 장치 PNG로 준비해야 한다");
                }
                for (x, y) in [a.1.x, a.1.y, a.1.width, a.1.height]
                    .into_iter()
                    .zip([b.1.x, b.1.y, b.1.width, b.1.height])
                {
                    assert!(
                        (x - y).abs() < 1e-9,
                        "{fixture} {profile:?} owner={} WMF={:?} PNG={:?}",
                        a.0,
                        a.1,
                        b.1
                    );
                }
            }
        }
    }
}
