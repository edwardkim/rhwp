//! #6660의 두 그림을 한컴 PDF 좌표에 직접 대조한다.
//!
//! 한컴 2022 PDF의 841 x 1190pt 용지를 원본 HWP 용지 높이
//! 111685HU / 75로 균일 확대했다. bbox JSON의 반올림된 쪽 높이로
//! 다시 배율을 계산하거나 허용 오차를 넓히지 않는다.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

struct RenderOutput(PathBuf);

impl RenderOutput {
    fn new() -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("rhwp-6660-oracle-{}-{stamp}", std::process::id()));
        std::fs::create_dir(&path).expect("render output directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for RenderOutput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn collect_picture_y(node: &Value, pi: u64, width: f64, found: &mut Vec<f64>) {
    if node["type"] == "Image"
        && node["pi"].as_u64() == Some(pi)
        && node["bbox"]["w"]
            .as_f64()
            .is_some_and(|actual| (actual - width).abs() < 0.2)
    {
        found.push(node["bbox"]["y"].as_f64().expect("picture y"));
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect_picture_y(child, pi, width, found);
        }
    }
}

#[test]
fn both_reported_pictures_are_within_one_pixel_of_hancom() {
    let output = RenderOutput::new();
    let bin =
        std::env::var_os("CARGO_BIN_EXE_rhwp").unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
    let result = Command::new(bin)
        .arg("export-render-tree")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/samples/exam_science.hwp"
        ))
        .arg("--output")
        .arg(output.path())
        .output()
        .expect("run render tree export");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let pages = std::fs::read_dir(output.path())
        .expect("rendered pages")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .count();
    assert_eq!(
        pages, 4,
        "#6660 보정이 원본 4쪽의 페이지 나눔을 바꾸면 안 된다"
    );

    let mut failures = Vec::new();
    for (page, pi, width, oracle_y) in [(1, 28, 75.2, 1085.0663), (4, 109, 59.5, 1011.5182)] {
        let path = output.path().join(format!("render_tree_{page:03}.json"));
        let tree: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let mut found = Vec::new();
        collect_picture_y(&tree, pi, width, &mut found);
        assert_eq!(
            found.len(),
            1,
            "{page}쪽 대상 그림은 유일해야 한다: {found:?}"
        );
        let dy = found[0] - oracle_y;
        if dy.abs() >= 1.0 {
            failures.push(format!(
                "{page}쪽 pi={pi}: rhwp={}, Hancom={oracle_y}, dy={dy:.4}px",
                found[0]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "#6660 완료 기준 위반:\n{}",
        failures.join("\n")
    );
}
