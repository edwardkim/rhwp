//! 계산 빈 호스트의 표 바깥 상자와 뒤 문단을 독립 Print에 따라 보호한다.
//! 같은 네 원본의 전쪽 Native210/fresh057 raw100%와 직접 판독을 먼저 확인했다.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::{fs, path::Path, process::Command, time::SystemTime};

const ASSETS: &str = "mydocs/pr/assets/computed_table_outer_box_20261025";
const EXPECTED: &str =
    include_str!("../../mydocs/pr/assets/computed_table_outer_box_20261025/expected.json");

fn normalized(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn body_nodes<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    out.push(node);
    // 셀 글줄을 본문 뒤 문단으로 세지 않는다.
    if node["type"] != "Table" {
        for child in node["children"].as_array().into_iter().flatten() {
            body_nodes(child, out);
        }
    }
}

fn tree(stem: &str) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let parent = root.join("output/test-runs");
    fs::create_dir_all(&parent).unwrap();
    let output_dir = parent.join(format!(
        "outer-box-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&output_dir).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg("export-render-tree")
        .arg(
            root.join(ASSETS)
                .join("inputs")
                .join(format!("{stem}.hwpx")),
        )
        .args(["--compat", "2022", "--output"])
        .arg(&output_dir)
        .output();
    let pages: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("render_tree_")
                && path.extension().is_some_and(|ext| ext == "json")
        })
        .collect();
    let bytes = fs::read(output_dir.join("render_tree_001.json"));
    fs::remove_dir_all(&output_dir).unwrap();
    let result = result.expect("같은 Cargo head의 CLI 실행");
    assert!(
        result.status.success(),
        "{stem}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(pages.len(), 1, "{stem}: 독립 Print와 전체 쪽수 일치");
    serde_json::from_slice(&bytes.unwrap()).unwrap()
}

fn bbox(node: &Value, name: &str) -> f64 {
    node["bbox"][name].as_f64().unwrap()
}

#[test]
fn completed_outer_box_pages_preserve_successor_ownership_and_print_origins() {
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let cases = expected.as_object().unwrap();
    assert_eq!(cases.len(), 4, "빈 문단·본문과 두 바깥 여백의 대조군");
    for (stem, oracle) in cases {
        let tree = tree(stem);
        let mut nodes = Vec::new();
        body_nodes(&tree, &mut nodes);
        let tables: Vec<_> = nodes.iter().filter(|n| n["type"] == "Table").collect();
        assert_eq!(tables.len(), 1, "{stem}: 표 누락·중복·불필요한 분할");
        let table = tables[0];
        assert_eq!(table["pi"], 0, "원래 앵커 문단 소유");
        assert_eq!(table["rows"], 2);
        assert_eq!(table["cols"], 2);
        let bottom = bbox(table, "y") + bbox(table, "h");
        let outer_bottom = oracle["outer_bottom_hu"].as_f64().unwrap() * 96.0 / 7200.0;
        let successors: Vec<_> = nodes
            .iter()
            .filter(|n| n["type"] == "TextLine" && n["pi"].as_u64().is_some_and(|pi| pi > 0))
            .collect();
        assert_eq!(successors.len(), 2, "빈 줄도 독립 줄 상자를 소유한다");
        assert_eq!(successors[0]["pi"], 1);
        assert_eq!(successors[1]["pi"], 2);
        for line in &successors {
            // JSON의 y·h·뒤 y 각각 0.05px 반올림 오차만 포함한다.
            assert!(
                bbox(line, "y") + 0.15 >= bottom + outer_bottom,
                "{stem}: 표 바깥 아래 여백을 침범한 뒤 문단"
            );
        }
        assert!(bbox(successors[1], "y") >= bbox(successors[0], "y") + bbox(successors[0], "h"));
        let text: String = nodes
            .iter()
            .filter(|n| n["type"] == "TextRun")
            .map(|n| normalized(n["text"].as_str().unwrap()))
            .collect();
        assert_eq!(
            text,
            oracle["source_text"].as_str().unwrap(),
            "{stem}: 내용 순서·누락·중복"
        );

        // 양쪽 raw100%·직접 판독을 마친 96DPI 종이 원점의 독립 PDF 좌표다.
        // CLI JSON의 반올림 오차도 ±2px 안에서 소비한다.
        for (actual, field, rounding) in [
            (bbox(table, "x"), "table_left", 0.05),
            (bbox(table, "x") + bbox(table, "w"), "table_right", 0.10),
            (bbox(table, "y"), "table_top", 0.05),
            (bottom, "table_bottom", 0.10),
            (bbox(successors[1], "y"), "last_body_top", 0.05),
        ] {
            assert!(
                (actual - oracle[field].as_f64().unwrap()).abs() + rounding <= 2.0,
                "{stem}: 독립 Print의 {field}, actual={actual}"
            );
        }
    }
}
