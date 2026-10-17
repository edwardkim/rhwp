//! #7688: public output must paint the four original document arrowheads.
//! Hancom Print measures medium heads at ~15.35px at 96dpi, independently of
//! shaft length. Visual prerequisite and controlled size/width inputs: stage5.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

struct OutputDir(std::path::PathBuf);
impl OutputDir {
    fn new(kind: &str, profile: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "rhwp-7688-arrows-{}-{kind}-{profile}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn binary() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}
fn sample() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/group-drawing-02.hwp")
}

#[test]
fn original_svg_heads_keep_intrinsic_size_and_tip_ownership() {
    for profile in ["screen", "print"] {
        let dir = OutputDir::new("svg", profile);
        let output = Command::new(binary())
            .arg("export-svg")
            .arg(sample())
            .args(["--profile", profile, "--json", "-o"])
            .arg(&dir.0)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(manifest["pageCount"], 1);
        let svg = std::fs::read_to_string(manifest["pages"][0]["path"].as_str().unwrap()).unwrap();
        let document = roxmltree::Document::parse(&svg).unwrap();
        let lines: Vec<_> = document
            .descendants()
            .filter(|n| n.has_tag_name("line") && n.attribute("marker-start").is_some())
            .collect();
        assert_eq!(lines.len(), 4, "all four arrows retain their source line");
        for line in lines {
            let reference = line.attribute("marker-start").unwrap();
            let id = reference
                .strip_prefix("url(#")
                .unwrap()
                .strip_suffix(')')
                .unwrap();
            let marker = document
                .descendants()
                .find(|n| n.has_tag_name("marker") && n.attribute("id") == Some(id))
                .unwrap();
            let width: f64 = marker.attribute("markerWidth").unwrap().parse().unwrap();
            let height: f64 = marker.attribute("markerHeight").unwrap().parse().unwrap();
            assert!(
                (14.0..17.0).contains(&width) && (14.0..17.0).contains(&height),
                "{profile}: medium head {width}x{height} lost Hancom's ~15.35px intrinsic size"
            );
            assert_eq!(marker.attribute("refX"), Some("0"));
            assert_eq!(marker.attribute("refY"), Some("0"));
            assert_eq!(marker.attribute("markerUnits"), Some("userSpaceOnUse"));
        }
    }
}

#[cfg(feature = "native-skia")]
#[test]
fn native_png_paints_heads_off_the_shaft_in_both_profiles() {
    use rhwp::document_core::DocumentCore;
    use rhwp::paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile};
    use rhwp::renderer::ArrowStyle;
    fn tips(node: &LayerNode, output: &mut Vec<(f64, f64)>) {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => children.iter().for_each(|n| tips(n, output)),
            LayerNodeKind::ClipRect { child, .. } => tips(child, output),
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::Line { line, .. } = op {
                        if line.style.start_arrow == ArrowStyle::Arrow {
                            assert!(line.x2 > line.x1 && (line.y2 - line.y1).abs() < 0.01);
                            output.push((line.x1, line.y1));
                        }
                    }
                }
            }
        }
    }
    let core = DocumentCore::from_bytes(&std::fs::read(sample()).unwrap()).unwrap();
    for (profile, mode) in [
        ("screen", RenderProfile::Screen),
        ("print", RenderProfile::Print),
    ] {
        let tree = core.build_page_layer_tree_with_profile(0, mode).unwrap();
        let mut anchors = Vec::new();
        tips(&tree.root, &mut anchors);
        assert_eq!(anchors.len(), 4);
        let dir = OutputDir::new("png", profile);
        let output = Command::new(binary())
            .arg("export-png")
            .arg(sample())
            .args(["--profile", profile, "--dpi", "96", "-o"])
            .arg(&dir.0)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let png = std::fs::read_dir(&dir.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|x| x == "png"))
            .unwrap();
        let image = image::open(png).unwrap().to_rgba8();
        for (x, y) in anchors {
            // Intrinsic head band from the independent Print triangle: inside
            // its base, outside the thin shaft; unrelated text/frames excluded.
            let mut ink = 0;
            for dx in 5..13 {
                for dy in [-5, -4, -3, 3, 4, 5] {
                    let p = image.get_pixel(
                        (x.round() as i32 + dx) as u32,
                        (y.round() as i32 + dy) as u32,
                    );
                    ink += usize::from(p[0] < 96 && p[1] < 96 && p[2] < 96);
                }
            }
            assert!(ink > 12, "{profile}: native PNG omitted head at source endpoint ({x}, {y}): {ink} head-band pixels");
        }
    }
}
