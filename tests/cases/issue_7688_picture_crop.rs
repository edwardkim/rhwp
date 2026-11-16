//! 관계형·고정형: crop 원본 행 보존과 그림·소유 테두리의 실제 paint 순서·인쇄 pen 계약.
//! 주석 갱신: 2026-11-15T15:29:44+09:00 (Asia/Seoul); 직전 파일 커밋: 2026-11-15T11:42:52+09:00 (7fbc8ba62b23edd490391c1968bd7ebb87f0c5fe).
//! 2026-11-15 90d7515dee 측정: pic-crop-01 HWP 전 1쪽 Native 100.00% / 최적화 fresh WASM Canvas 100.00%.
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

#[test]
fn print_picture_pen_is_independent_of_bitmap_color_generation() {
    use rhwp::{
        paint::{LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile},
        renderer::{
            render_tree::{RenderNode, RenderNodeType},
            StrokeDash,
        },
        DocumentCore,
    };
    fn pens(node: &LayerNode, found: &mut Vec<Option<[f64; 2]>>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    pens(child, found);
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                pens(child, found)
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::Rectangle { rect, .. } = op {
                        if rect.picture_border {
                            found.push(rect.print_picture_pen);
                        }
                    }
                }
            }
        }
    }
    fn verify(layer: &LayerNode, print: bool) {
        let mut found = Vec::new();
        pens(layer, &mut found);
        assert_eq!(found.len(), 2, "원문 두 그림 테두리의 누락·중복 방지");
        for pen in found {
            if print {
                // 독립 2022 PDF compound path: band 0.480pt, outline 0.120pt.
                // 96dpi 좌표에서 0.64 / 0.16이며 원본 48HU의 띠를 보존한다.
                let [band, outline] = pen.expect("색상 세대와 별개인 실제 Print 그림 pen");
                assert!((band - 0.64).abs() < 1e-9);
                assert!((outline - 0.16).abs() < 1e-9);
            } else {
                assert_eq!(pen, None, "Screen/HQ에 Print 기준 장치를 부여하지 않는다");
            }
        }
    }
    fn alter(node: &mut RenderNode, mode: u8) {
        if let RenderNodeType::Rectangle(rect) = &mut node.node_type {
            if rect.picture_border {
                match mode {
                    0 => rect.style.stroke_dash = StrokeDash::Dash,
                    1 => rect.image_frame_owner = None,
                    2 => rect.transform.rotation = 10.0,
                    _ => unreachable!(),
                }
            }
        }
        for child in &mut node.children {
            alter(child, mode);
        }
    }
    let bytes =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/pic-crop-01.hwp"))
            .unwrap();
    for generation in [false, true] {
        let mut core = DocumentCore::from_bytes(&bytes).unwrap();
        core.set_hangul2024_compat(generation);
        for profile in [
            RenderProfile::Print,
            RenderProfile::Screen,
            RenderProfile::HighQuality,
        ] {
            let layer = core.build_page_layer_tree_with_profile(0, profile).unwrap();
            verify(&layer.root, profile == RenderProfile::Print);
        }
        for mode in 0..3 {
            let mut render = core.build_page_render_tree(0).unwrap();
            alter(&mut render.root, mode);
            let layer = LayerBuilder::new(RenderProfile::Print).build(&render);
            verify(&layer.root, false);
        }
    }
}
