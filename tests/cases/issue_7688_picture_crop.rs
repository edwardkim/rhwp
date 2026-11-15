//! 관계형·고정형: crop 원본 행 보존과 그림·소유 테두리의 실제 paint 순서.
//! 주석 갱신: 2026-11-15T11:18:45+09:00 (Asia/Seoul); 직전 파일 커밋: 2026-10-09T20:28:16+09:00 (21ed64071f81f35f37618717693bd30f727392c9).
//! 2026-11-15 측정: pic-crop-01 HWP 전 1쪽 Native 100.00% / 최적화 fresh WASM Canvas 100.00%.
//! 같은 1쪽이 양쪽 전쪽 최저이며 Print / compat2022 / 96DPI / PDFium / raw2px 실루엣이다.
//! 독립 pdf/pic-crop-01-2022.pdf와 같은 배율에서 두 crop 창과 네 변의 테두리를 직접 판독했다.
//! 근거: mydocs/pr/assets/jinho-von-choi-20261114/protected-environment-review.md.
//! 고정 쪽수·원본 선택 행은 기존 독립 Print 근거를 보존하고 새 paint 검사는 좌표를 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

struct OutputDir(std::path::PathBuf);
impl OutputDir {
    fn new(profile: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("rhwp-7688-crop-{}-{profile}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn right_bottom_crop_keeps_the_selected_source_rows() {
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/pic-crop-01.hwp");
    for profile in ["screen", "print"] {
        let output_dir = OutputDir::new(profile);
        let output = Command::new(&binary)
            .arg("export-svg")
            .arg(&sample)
            .args(["--profile", profile, "--json", "-o"])
            .arg(&output_dir.0)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(manifest["pageCount"], 1);
        let svg = std::fs::read_to_string(manifest["pages"][0]["path"].as_str().unwrap()).unwrap();
        let document = roxmltree::Document::parse(&svg).unwrap();
        let selections: Vec<(f64, f64)> = document
            .descendants()
            .filter(|node| node.has_tag_name("image"))
            .map(|node| {
                let viewport = node
                    .ancestors()
                    .find(|parent| parent.has_tag_name("svg"))
                    .unwrap();
                if viewport.parent_element().is_none() {
                    return (639.0, 70.0);
                }
                assert_eq!(node.attribute("width"), Some("639"));
                assert_eq!(node.attribute("height"), Some("70"));
                let window: Vec<f64> = viewport
                    .attribute("viewBox")
                    .unwrap()
                    .split_whitespace()
                    .map(|value| value.parse().unwrap())
                    .collect();
                assert_eq!(window.len(), 4);
                assert_eq!((window[0], window[1]), (0.0, 0.0));
                (window[2], window[3])
            })
            .collect();
        assert_eq!(selections.len(), 2, "picture count/order must survive");
        let (first, second) = (selections[0], selections[1]);
        assert!((first.1 - 70.0).abs() < 0.01);
        assert!(first.0 > 639.0 * 0.99 && first.0 <= 639.0);
        assert!((second.0 - 639.0).abs() < 0.01);
        assert!(
            second.1 > 57.0 && second.1 < 60.0,
            "{profile}: bottom crop lost: selected {} rows instead of PDF's ~58",
            second.1
        );
        assert!(second.1 < first.1 * 0.9);
    }
}

#[test]
fn picture_frames_follow_their_own_cropped_image_in_real_print_paint() {
    use rhwp::{
        paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
        renderer::render_tree::{RenderNode, RenderNodeType},
        DocumentCore,
    };
    fn collect_images(node: &RenderNode, images: &mut Vec<u32>) {
        if matches!(node.node_type, RenderNodeType::Image(_)) {
            images.push(node.id);
        }
        for child in &node.children {
            collect_images(child, images);
        }
    }
    fn collect_ops<'a>(node: &'a LayerNode, ops: &mut Vec<(Option<u32>, &'a PaintOp)>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    collect_ops(child, ops);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                collect_ops(child, ops);
            }
            LayerNodeKind::Leaf { ops: leaf } => {
                ops.extend(leaf.iter().map(|op| (node.source_node_id, op)));
            }
        }
    }
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/pic-crop-01.hwp"))
            .unwrap();
    let mut core = DocumentCore::from_bytes(&bytes).unwrap();
    core.set_hangul2024_compat(false);
    let render = core.build_page_render_tree(0).unwrap();
    let mut owners = Vec::new();
    collect_images(&render.root, &mut owners);
    assert_eq!(owners.len(), 2, "원본 crop 그림 두 개를 빠짐없이 배치한다");
    let layer = core
        .build_page_layer_tree_with_profile(0, RenderProfile::Print)
        .unwrap();
    let mut paint = Vec::new();
    collect_ops(&layer.root, &mut paint);
    for owner in owners {
        let images: Vec<_> = paint
            .iter()
            .enumerate()
            .filter(|(_, (id, op))| *id == Some(owner) && matches!(op, PaintOp::Image { .. }))
            .map(|(index, _)| index)
            .collect();
        let frames: Vec<_> = paint
            .iter()
            .enumerate()
            .filter_map(|(index, (_, op))| match op {
                PaintOp::Rectangle { rect, .. }
                    if rect.picture_border && rect.image_frame_owner == Some(owner) =>
                {
                    Some(index)
                }
                _ => None,
            })
            .collect();
        assert_eq!(images.len(), 1, "소유 그림의 실제 paint 누락·중복");
        assert_eq!(frames.len(), 1, "소유 테두리의 실제 paint 누락·중복");
        assert!(
            images[0] < frames[0],
            "그림을 먼저 그린 뒤 자기 테두리를 그려 네 변의 가시성을 보존한다"
        );
    }
}
