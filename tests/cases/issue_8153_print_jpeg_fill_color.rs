//! 관계형·고정형: 전체 JPEG 채움의 원본 RGB·출력 자원·프로필 경계를 보호한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! HWP 261쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 261쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 독립 Print 자원과 시각 증거는 tests/fixtures/issue8153/README.md 참조.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::style::ImageFillMode;
use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImageKind};
use rhwp::DocumentCore;

fn images<'a>(node: &'a LayerNode, found: &mut Vec<(Option<u32>, &'a PaintOp)>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                images(child, found);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            images(child, found)
        }
        LayerNodeKind::Leaf { ops } => {
            found.extend(ops.iter().filter_map(|op| {
                match op {
                    PaintOp::Image { image, .. }
                        if matches!(
                            image.fill_mode,
                            Some(ImageFillMode::FitToSize | ImageFillMode::Total)
                        ) && image
                            .data
                            .as_deref()
                            .is_some_and(|bytes| bytes.starts_with(&[0xff, 0xd8])) =>
                    {
                        Some((node.source_node_id, op))
                    }
                    _ => None,
                }
            }));
        }
    }
}

fn verify(extension: &str) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root.join(format!("samples/2025 행정업무운영 편람(최종).{extension}"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(input).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 383, "독립 Print 전체 쪽수");
    let expected = [
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/issue8153/handbook-p261-top-print.jpeg"
        ))
        .as_slice(),
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/issue8153/handbook-p261-bottom-print.jpeg"
        ))
        .as_slice(),
    ];
    let mut sources = Vec::new();
    for profile in [
        RenderProfile::Print,
        RenderProfile::Screen,
        RenderProfile::Print,
    ] {
        let tree = core
            .build_page_layer_tree_with_profile(260, profile)
            .unwrap();
        let mut targets = Vec::new();
        images(&tree.root, &mut targets);
        assert_eq!(targets.len(), 2, "원본 전체 채움의 누락·중복 금지");
        let owners: std::collections::HashSet<_> = targets
            .iter()
            .map(|(owner, _)| owner.expect("원본 소유"))
            .collect();
        assert_eq!(owners.len(), 2, "서로 다른 원본 채움의 소유");
        let mut previous_bottom = None;
        for (i, (_, op)) in targets.iter().enumerate() {
            let PaintOp::Image {
                bbox,
                image,
                resolved,
            } = op
            else {
                unreachable!()
            };
            let source = image.data.as_deref().unwrap();
            if sources.len() < 2 {
                sources.push(source.to_vec());
            }
            assert_eq!(source, sources[i], "Print가 원본 JPEG를 덮어쓰지 않는다");
            assert_eq!(image.opacity, 1.0);
            assert!(image.crop.is_none() && !image.transform.has_transform());
            if let Some(bottom) = previous_bottom {
                assert!(bottom < bbox.y, "앞뒤 채움의 순서·비겹침");
            }
            previous_bottom = Some(bbox.y + bbox.height);
            if profile == RenderProfile::Screen {
                assert_eq!(
                    resolved.as_deref().map_or(source, |p| p.data.as_slice()),
                    source,
                    "Screen은 원본 색과 JPEG 바이트를 보존한다"
                );
                continue;
            }
            let output = resolved.as_deref().expect("배치별 Print 전달");
            assert_eq!(output.mime, "image/jpeg", "독립 Print의 JPEG 채움 전달");
            assert_eq!(output.kind, ResolvedImageKind::OutputDevice);
            let original_size = image::load_from_memory(source)
                .unwrap()
                .to_rgb8()
                .dimensions();
            let oracle = image::load_from_memory(expected[i]).unwrap().to_rgb8();
            let actual = image::load_from_memory(&output.data).unwrap().to_rgb8();
            assert_eq!(original_size, oracle.dimensions(), "같은 원본 전체 창");
            assert_eq!(actual.dimensions(), oracle.dimensions());
            assert_eq!(
                actual.as_raw(),
                oracle.as_raw(),
                "원본 RGB를 재압축한 독립 Print와 전픽셀 일치; Picture RGB555를 섞지 않는다"
            );
        }
    }
}

#[test]
fn hwp_jpeg_fill_keeps_independent_print_rgb_and_source_profile_boundary() {
    verify("hwp");
}

#[test]
fn hwpx_jpeg_fill_keeps_independent_print_rgb_and_source_profile_boundary() {
    verify("hwpx");
}
