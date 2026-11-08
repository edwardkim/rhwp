//! 독립 Print의 TAC 셀 그리드와 외부 자손의 부모 물리 프레임을 보호한다.
//! 두 원본의 Native229/fresh065 전쪽 raw100%와 직접 판독 후 추가한다.
//! PDF 벡터 괘선 중심 좌표는 96 DPI의 CSS px이며 ±2px와 관계 검사를 함께 쓴다.

#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::Command, time::SystemTime};

const ASSETS: &str = "mydocs/pr/assets/pr_7833_cell_axis_diagnostic_20261025";

fn bbox(node: &Value, component: &str) -> f64 {
    node["bbox"][component].as_f64().unwrap()
}

fn close(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "{label}: 독립 PDF {expected}, 실제 {actual}, 허용 ±2 CSSpx"
    );
}

fn collect<'a>(node: &'a Value, kind: &str, out: &mut Vec<&'a Value>) {
    if node["type"] == kind {
        out.push(node);
    }
    for child in node["children"].as_array().into_iter().flatten() {
        collect(child, kind, out);
    }
}

fn characters(text: &str) -> BTreeMap<char, usize> {
    let mut counts = BTreeMap::new();
    for character in text.chars() {
        *counts.entry(character).or_default() += 1;
    }
    counts
}

fn verify(key: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let oracle: Value = serde_json::from_slice(
        &fs::read(root.join(ASSETS).join("independent-frame-oracle.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(oracle["dpi"], 96);
    let case = oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["key"] == key)
        .unwrap();
    let input = root.join(case["input"].as_str().unwrap());
    assert_eq!(
        Sha256::digest(fs::read(&input).unwrap())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        case["inputSha256"].as_str().unwrap(),
        "독립 PDF와 대응한 원본 바이트"
    );
    let pdf = root.join(case["pdf"].as_str().unwrap());
    let pdf_hash: String = Sha256::digest(fs::read(pdf).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(pdf_hash, case["pdfSha256"].as_str().unwrap());
    let output = root.join("output/test-runs").join(format!(
        "inline-grid-{key}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&output).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-render-tree")
        .arg(input)
        .args(["--compat", "2022", "--output"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{key}: CLI 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files: Vec<_> = fs::read_dir(&output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    assert_eq!(files.len(), 1, "독립 Print 한 쪽과 전체 내용 보존");
    let page: Value = serde_json::from_slice(&fs::read(&files[0]).unwrap()).unwrap();
    fs::remove_dir_all(output).unwrap();
    let mut tables = Vec::new();
    collect(&page, "Table", &mut tables);
    let outer = tables[0];
    let mut ancestors = vec![outer];
    while let Some(child) = ancestors.last().unwrap()["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|child| child["type"] == "Table")
    {
        ancestors.push(child);
    }
    let heights = case["ancestorHeights"].as_array().unwrap();
    assert_eq!(ancestors.len(), heights.len(), "원본 조상 표 소속");
    for (table, height) in ancestors.iter().zip(heights) {
        close(
            bbox(table, "x"),
            case["x"].as_f64().unwrap(),
            "공동 가로 원점",
        );
        close(
            bbox(table, "y"),
            case["y"].as_f64().unwrap(),
            "공동 세로 원점",
        );
        close(
            bbox(table, "w"),
            case["width"].as_f64().unwrap(),
            "셀 그리드 폭",
        );
        close(bbox(table, "h"), height.as_f64().unwrap(), "부모 물리 높이");
    }
    let inner = *ancestors.last().unwrap();
    let mut inline: Vec<_> = tables
        .into_iter()
        .filter(|table| table["pi"] != 1)
        .collect();
    inline.sort_by(|left, right| bbox(left, "y").total_cmp(&bbox(right, "y")));
    let boxes = case["inlineTables"].as_array().unwrap();
    assert_eq!(inline.len(), boxes.len(), "인라인 표 누락·중복 없음");
    for (table, expected) in inline.iter().zip(boxes) {
        close(
            bbox(table, "x"),
            case["inlineTableX"].as_f64().unwrap(),
            "인라인 원점",
        );
        close(
            bbox(table, "y"),
            expected["y"].as_f64().unwrap(),
            "인라인 줄 소속",
        );
        close(
            bbox(table, "h"),
            expected["height"].as_f64().unwrap(),
            "인라인 셀 높이",
        );
        close(
            bbox(table, "w"),
            bbox(inner, "w"),
            "같은 셀 그리드의 물리 폭",
        );
        assert!(bbox(table, "y") >= bbox(inner, "y"));
        assert!(bbox(table, "y") + bbox(table, "h") <= bbox(inner, "y") + bbox(inner, "h"));
    }
    if ancestors.len() > 1 {
        assert!(bbox(outer, "y") + bbox(outer, "h") < bbox(inner, "y") + bbox(inner, "h"));
        assert!(inline
            .iter()
            .any(|table| bbox(table, "y") > bbox(outer, "y") + bbox(outer, "h")));
    }
    let mut runs = Vec::new();
    collect(&page, "TextRun", &mut runs);
    let text: String = runs.iter().filter_map(|run| run["text"].as_str()).collect();
    let mut remaining = text.as_str();
    for expected in ["머리", "가", "🦦", "나", "다라", "앞", "뒤", "끝", "말"] {
        let position = remaining.find(expected).expect("원본 본문의 내용 순서");
        remaining = &remaining[position + expected.len()..];
    }
    assert_eq!(
        characters(&text),
        characters(case["expectedText"].as_str().unwrap()),
        "원본 글자 보존·무중복"
    );
    let mut lines = Vec::new();
    collect(inner, "TextLine", &mut lines);
    let line_y = |expected: &str| {
        let line = lines
            .iter()
            .find(|line| {
                let mut content = Vec::new();
                collect(line, "TextRun", &mut content);
                content
                    .iter()
                    .filter_map(|run| run["text"].as_str())
                    .collect::<String>()
                    == expected
            })
            .expect("인라인 표 앞뒤 본문의 줄 소속");
        bbox(line, "y")
    };
    assert!(line_y("앞") < bbox(inline[2], "y"));
    assert!(bbox(inline[2], "y") + bbox(inline[2], "h") < line_y("뒤끝"));
    assert!(line_y("뒤끝") < bbox(inline[3], "y"));
    assert!(bbox(inline[3], "y") + bbox(inline[3], "h") < line_y("말"));
}

#[test]
fn verified_inline_tables_use_cell_grid_instead_of_stale_object_size() {
    verify("flat");
}

#[test]
fn verified_external_ancestors_keep_physical_frames_and_descendant_content() {
    verify("nested");
}
