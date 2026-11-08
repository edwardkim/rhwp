//! 기존 두 혼합 컷 입력의 바깥 프레임·이어받기·종료를 실제 CLI에서 보호한다.
//! 96dpi 독립 Print 좌표와 전체 Native/fresh WASM 자격은 같은 이름의 oracle에 기록한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rhwp-7518-tail-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn rendered_pages(input: &Path) -> Vec<Value> {
    let dir = TempDir::new();
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(bin)
        .arg("export-render-tree")
        .arg(input)
        .args(["--compat", "2022", "-o"])
        .arg(&dir.0)
        .output()
        .unwrap();
    assert!(result.status.success(), "{result:?}");
    let mut files: Vec<_> = std::fs::read_dir(&dir.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap())
        .collect()
}

fn nodes(root: &Value) -> Vec<&Value> {
    let mut result = vec![root];
    if let Some(children) = root["children"].as_array() {
        for child in children {
            result.extend(nodes(child));
        }
    }
    result
}

fn oracle(root: &Path, family: &str) -> Value {
    let document: Value = serde_json::from_slice(
        &std::fs::read(
            root.join("mydocs/pr/assets/pr7518_mixed_cut_20261027/independent-frame-oracle.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(document["dpi"], 96);
    assert_eq!(document["status"], "qualified");
    let case = document["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["family"] == family)
        .unwrap()
        .clone();
    for kind in ["input", "pdf"] {
        let bytes = std::fs::read(root.join(case[kind].as_str().unwrap())).unwrap();
        let key = format!("{kind}_sha256");
        let digest: String = Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(digest, case[key.as_str()].as_str().unwrap());
    }
    for backend in ["native", "wasm"] {
        assert!(case["qualification"][backend]["raw_min"].as_f64().unwrap() >= 98.0);
    }
    case
}

fn coord(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().unwrap()
}

fn outer(page: &Value) -> &Value {
    let owners: Vec<_> = nodes(page)
        .into_iter()
        .filter(|n| n["type"] == "Table" && n["pi"] == 3)
        .collect();
    assert_eq!(owners.len(), 1, "원본 바깥 표의 쪽별 소유 하나");
    owners[0]
}

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print 좌표 {expected}, 실제 {actual}; 허용 ±2 CSSpx"
    );
}

#[test]
fn following_overlay_owns_its_outer_margin_before_and_after_the_first_cut() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case = oracle(root, "valid_generated");
    let input = root.join(case["input"].as_str().unwrap());
    let source = rhwp::parser::parse_hwp(&std::fs::read(&input).unwrap()).unwrap();
    let host = &source.sections[0].paragraphs[3];
    let table = host
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .unwrap();
    let margin = f64::from(table.outer_margin_top) / 75.0;
    assert!(margin > 0.0 && !host.line_segs.is_empty());
    let pages = rendered_pages(&input);
    assert_eq!(pages.len(), 8, "원본 독립 Print 쪽수");
    for position in case["positions"].as_array().unwrap() {
        let index = position["page"].as_u64().unwrap() as usize - 1;
        let print_top = position["outer_top_csspx"].as_f64().unwrap();
        let column = nodes(&pages[index])
            .into_iter()
            .find(|n| n["type"] == "Column")
            .unwrap();
        let frame = outer(&pages[index]);
        let anchor = coord(column, "y")
            + if index == 0 {
                (f64::from(host.line_segs[0].vertical_pos)
                    + f64::from(table.common.vertical_offset))
                    / 75.0
            } else {
                0.0
            };
        let owned = coord(frame, "y") - anchor;
        assert!(
            (owned / margin - 1.0).abs() < 0.08,
            "선행 저장 줄과 이어받기 모두 원본 바깥여백을 소유해야 한다: {owned}/{margin}"
        );
        near(coord(frame, "y"), print_top);
        let cells: Vec<_> = frame["children"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| n["type"] == "Cell")
            .collect();
        assert!(!cells.is_empty());
        assert!(cells
            .iter()
            .all(|cell| coord(cell, "y") >= coord(frame, "y") - 0.1));
        assert!(coord(frame, "y") > coord(column, "y"));
    }
}

#[test]
fn mixed_stream_print_slice_preserves_owned_lines_and_the_final_physical_frame() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case = oracle(root, "valid_orientation");
    let input = root.join(case["input"].as_str().unwrap());
    let pages = rendered_pages(&input);
    assert_eq!(pages.len(), 8, "원본 독립 Print 쪽수");
    let frame = outer(&pages[3]);
    let owner = frame["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["type"] == "Cell" && n["row"] == 5 && n["col"] == 1)
        .unwrap();
    let child = nodes(owner)
        .into_iter()
        .find(|n| n["type"] == "Table")
        .unwrap();
    let units: Vec<_> = nodes(child)
        .into_iter()
        .filter(|n| n["type"] == "TextRun")
        .filter(|n| {
            n["text"]
                .as_str()
                .is_some_and(|text| text.starts_with("UNIT "))
        })
        .collect();
    assert_eq!(units.len(), 18, "Print 쪽4의 혼합 소스 컷은 UNIT000~017");
    for (index, unit) in units.iter().enumerate() {
        assert_eq!(unit["text"], format!("UNIT {index:03}"));
        assert!(coord(unit, "y") >= coord(child, "y"));
        assert!(coord(unit, "y") + coord(unit, "h") <= coord(child, "y") + coord(child, "h") + 0.1);
        if index > 0 {
            assert!(coord(unit, "y") > coord(units[index - 1], "y"));
        }
    }
    let slice = &case["positions"][0];
    near(
        coord(child, "x"),
        slice["nested_left_csspx"].as_f64().unwrap(),
    );
    near(
        coord(child, "y"),
        slice["nested_top_csspx"].as_f64().unwrap(),
    );
    near(
        coord(child, "y") + coord(child, "h"),
        slice["nested_bottom_csspx"].as_f64().unwrap(),
    );
    let final_frame = outer(&pages[7]);
    let tail = &case["positions"][1];
    near(
        coord(final_frame, "y"),
        tail["outer_top_csspx"].as_f64().unwrap(),
    );
    near(
        coord(final_frame, "y") + coord(final_frame, "h"),
        tail["outer_bottom_csspx"].as_f64().unwrap(),
    );
    assert!(
        nodes(final_frame)
            .into_iter()
            .filter(|n| n["type"] == "TextRun")
            .all(|n| !n["text"].as_str().unwrap_or("").starts_with("UNIT ")),
        "소비한 혼합 유닛을 최종 표 조각에 중복 배치하지 않는다"
    );
}
