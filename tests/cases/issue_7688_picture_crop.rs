//! #7688: real picture selection survives all public SVG output profiles.
//! Independent Hancom Print PDF: first banner shows 70 source rows; second ~58.
//! Native/fresh WASM visual prerequisite and before/after proof: stage4 report.
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
