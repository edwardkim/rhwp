//! 한컴 Print와 양쪽 raw100을 확인한 셀의 물리 폭·내용 소유를 보호한다.
//! 원시 선언 600 HU는 직렬화 값이고, 실제 칸에는 좌우 안 여백도 들어간다.
//! 독립 괘선 중심은 PDF 좌표를 96 DPI로 환산한 값이며 허용 오차는 ±2px다.

#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::Command, time::SystemTime};

const ASSETS: &str = "mydocs/pr/assets/pr_7833_minimum_cell_width";

fn number(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().expect("실제 경계 좌표")
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print {expected}, 실제 {actual}; 허용 ±2 CSSpx"
    );
}

fn tables<'a>(node: &'a Value, result: &mut Vec<&'a Value>) {
    if node["type"] == "Table" {
        result.push(node);
    }
    for child in node["children"].as_array().into_iter().flatten() {
        tables(child, result);
    }
}

fn leaf_owners<'a>(node: &'a Value, owner: Option<&'a Value>, result: &mut Vec<&'a Value>) {
    let owner = if node["type"] == "Cell" {
        Some(node)
    } else {
        owner
    };
    if node["type"] == "Table" {
        let mut descendants = Vec::new();
        tables(node, &mut descendants);
        if descendants.len() == 1 {
            result.push(owner.expect("인라인 표는 실제 원본 셀에 속한다"));
        }
    }
    for child in node["children"].as_array().into_iter().flatten() {
        leaf_owners(child, owner, result);
    }
}

fn text(node: &Value, result: &mut String) {
    if node["type"] == "TextRun" {
        result.push_str(node["text"].as_str().expect("원문 텍스트"));
    }
    for child in node["children"].as_array().into_iter().flatten() {
        text(child, result);
    }
}

fn counts(text: &str) -> BTreeMap<char, usize> {
    let mut result = BTreeMap::new();
    for character in text.chars() {
        *result.entry(character).or_default() += 1;
    }
    result
}

#[test]
fn printed_inline_cells_keep_padding_content_and_independent_grid_edges() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let oracle: Value = serde_json::from_slice(
        &fs::read(root.join(ASSETS).join("independent-width-oracle.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(oracle["dpi"], 96);
    for kind in ["input", "pdf"] {
        let bytes = fs::read(root.join(oracle[kind].as_str().unwrap())).unwrap();
        let hash: String = Sha256::digest(bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(hash, oracle[format!("{kind}_sha256")].as_str().unwrap());
    }
    let output = root.join("output/test-runs").join(format!(
        "inline-physical-width-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&output).unwrap();
    let rendered = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-render-tree")
        .arg(root.join(oracle["input"].as_str().unwrap()))
        .args(["--compat", "2022", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        rendered.status.success(),
        "실제 CLI 실패: {}",
        String::from_utf8_lossy(&rendered.stderr)
    );
    let pages: Vec<_> = fs::read_dir(&output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    assert_eq!(pages.len(), 1, "독립 Print 한 쪽의 내용 보존");
    let page: Value = serde_json::from_slice(&fs::read(&pages[0]).unwrap()).unwrap();
    fs::remove_dir_all(output).unwrap();

    let mut all_tables = Vec::new();
    tables(&page, &mut all_tables);
    assert_eq!(
        all_tables.len(),
        oracle["table_count"].as_u64().unwrap() as usize
    );
    let leaves: Vec<_> = all_tables
        .iter()
        .copied()
        .filter(|table| {
            let mut descendants = Vec::new();
            for child in table["children"].as_array().unwrap() {
                tables(child, &mut descendants);
            }
            descendants.is_empty()
        })
        .collect();
    assert_eq!(
        leaves.len(),
        oracle["leaf_table_count"].as_u64().unwrap() as usize
    );
    let mut owners = Vec::new();
    leaf_owners(&page, None, &mut owners);
    assert_eq!(owners.len(), leaves.len());
    for (index, (leaf, owner)) in leaves.iter().zip(&owners).enumerate() {
        assert_eq!(leaf["pi"], if index < 2 { 0 } else { 2 }, "원문 문단 소속");
        assert_eq!(leaf["ci"], index % 2, "문단 안 개체 순서");
        assert_eq!(owner["row"], 0);
        assert_eq!(owner["col"], 0, "두 번째 형제 셀로 이동하지 않음");
        for (origin, extent) in [("x", "w"), ("y", "h")] {
            assert!(number(leaf, origin) >= number(owner, origin) - 2.0);
            assert!(
                number(leaf, origin) + number(leaf, extent)
                    <= number(owner, origin) + number(owner, extent) + 2.0
            );
        }
    }
    let edges: Vec<_> = oracle["x_edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .collect();
    let padding = oracle["source_padding_hu"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_f64().unwrap())
        .sum::<f64>()
        * 96.0
        / 7200.0;
    for table in &leaves {
        close(number(table, "w"), edges[2] - edges[0]);
        close(number(table, "x"), edges[0]);
        let cells: Vec<_> = table["children"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|node| node["type"] == "Cell")
            .collect();
        assert_eq!(cells.len(), 2);
        for (index, cell) in cells.iter().enumerate() {
            close(number(cell, "x"), edges[index]);
            close(number(cell, "w"), edges[index + 1] - edges[index]);
            assert!(
                number(cell, "w") > padding,
                "칸이 안 여백 뒤의 내용 공간을 소유해야 한다"
            );
            close(number(cell, "y"), number(table, "y"));
        }
        close(
            number(cells[0], "x") + number(cells[0], "w"),
            number(cells[1], "x"),
        );
        close(
            number(cells[1], "x") + number(cells[1], "w"),
            number(table, "x") + number(table, "w"),
        );
    }
    for pair in leaves.windows(2) {
        assert!(
            number(pair[0], "y") + number(pair[0], "h") < number(pair[1], "y"),
            "원문 순서와 비겹침"
        );
    }
    let mut painted = String::new();
    text(&page, &mut painted);
    assert_eq!(
        counts(&painted),
        counts(&format!("머리가🦦나다라앞뒤끝말{}", "중간".repeat(6))),
        "텍스트 누락·중복 없음"
    );
}
