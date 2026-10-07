//! #6660의 두 그림이 원래 쪽과 표 셀 안에 남는지 확인한다.
//!
//! 정본: `pdf/exam_science-2020.pdf`(MCP engine2020, 원문272×394mm 용지,4쪽).
//! 실제 그림의 세부 좌표와 전쪽 시각 일치율은 같은 원본·정본의 Visual Sweep에서 확인한다.

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
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("output/test")
            .join(format!("rhwp-6660-oracle-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&path).expect("render output directory");
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

fn collect_pictures<'a>(
    node: &'a Value,
    pi: u64,
    table: Option<&'a Value>,
    cell: Option<&'a Value>,
    found: &mut Vec<(&'a Value, &'a Value, &'a Value)>,
) {
    let table = if node["type"] == "Table" && node["pi"].as_u64() == Some(pi) {
        Some(node)
    } else {
        table
    };
    let cell = if node["type"] == "Cell" {
        Some(node)
    } else {
        cell
    };
    if node["type"] == "Image" && node["pi"].as_u64() == Some(pi) {
        found.push((
            node,
            table.expect("그림 소유 표"),
            cell.expect("그림 소유 셀"),
        ));
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect_pictures(child, pi, table, cell, found);
        }
    }
}

fn inside(inner: &Value, outer: &Value) -> bool {
    let read = |node: &Value, key: &str| node["bbox"][key].as_f64().expect("상자 좌표");
    let (ix, iy, iw, ih) = (
        read(inner, "x"),
        read(inner, "y"),
        read(inner, "w"),
        read(inner, "h"),
    );
    let (ox, oy, ow, oh) = (
        read(outer, "x"),
        read(outer, "y"),
        read(outer, "w"),
        read(outer, "h"),
    );
    ix >= ox - 0.5 && iy >= oy - 0.5 && ix + iw <= ox + ow + 0.5 && iy + ih <= oy + oh + 0.5
}

#[test]
fn both_reported_pictures_stay_inside_their_cells_on_their_pages() {
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

    for (page, pi, count) in [(1, 28, 3), (4, 109, 2)] {
        let path = output.path().join(format!("render_tree_{page:03}.json"));
        let tree: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let mut found = Vec::new();
        collect_pictures(&tree, pi, None, None, &mut found);
        assert_eq!(
            found.len(),
            count,
            "{page}쪽 문단 {pi}의 그림이 누락되거나 중복되었다: {found:?}"
        );
        for (image, table, cell) in found {
            assert!(
                inside(image, cell),
                "{page}쪽 문단 {pi} 그림이 셀 밖에 있다"
            );
            assert!(
                inside(cell, table),
                "{page}쪽 문단 {pi} 셀이 소유 표 밖에 있다"
            );
        }
    }
}
