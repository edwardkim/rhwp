//! 원래 HWPX와 독립 한컴 Print로 셀 캐시·암묵적 글자모양·부호 있는 전진을 보호한다.
//! 양쪽 raw 98% 이상·확대 판독 증거와 독립 좌표는 아래 ASSETS에 보존한다.

#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::Command, time::SystemTime};

const ASSETS: &str = "mydocs/pr/assets/hwpx_cell_frame_print/independent-oracle.json";

struct Output(std::path::PathBuf);

impl Output {
    fn new(label: &str) -> Self {
        Self(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("output/test-runs")
                .join(format!(
                    "cell-frame-{label}-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                )),
        )
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn export(input: &Path, output: &Path, kind: &str) -> String {
    fs::create_dir_all(output).unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let mut cmd = Command::new(binary);
    cmd.arg(format!("export-{kind}"))
        .arg(input)
        .args(["--compat", "2022"]);
    if kind == "svg" {
        cmd.args(["--profile", "print"]);
    }
    let result = cmd.arg("-o").arg(output).output().unwrap();
    assert!(
        result.status.success(),
        "실제 CLI 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let extension = if kind == "svg" { "svg" } else { "json" };
    let files: Vec<_> = fs::read_dir(output)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|v| v == extension))
        .collect();
    assert_eq!(files.len(), 1, "독립 Print와 같은 한 쪽");
    fs::read_to_string(&files[0]).unwrap()
}

fn oracle() -> Value {
    serde_json::from_slice(&fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(ASSETS)).unwrap())
        .unwrap()
}

fn check_sources(case: &Value) {
    for kind in ["input", "pdf"] {
        let data =
            fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(case[kind].as_str().unwrap()))
                .unwrap();
        let sha: String = Sha256::digest(data)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            sha,
            case[format!("{kind}_sha256")].as_str().unwrap(),
            "검증한 원본 바이트 유지"
        );
    }
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print {expected}, 실제 {actual}; 허용 ±2 CSSpx"
    );
}

fn coordinate(node: &Value, axis: &str) -> f64 {
    node["bbox"][axis].as_f64().unwrap()
}

fn nodes<'a>(
    node: &'a Value,
    owner: Option<&'a Value>,
    result: &mut Vec<(&'a Value, Option<&'a Value>)>,
) {
    let owner = if node["type"] == "Cell" {
        Some(node)
    } else {
        owner
    };
    result.push((node, owner));
    for child in node["children"].as_array().into_iter().flatten() {
        nodes(child, owner, result);
    }
}

fn own_paragraphs(node: &Value, result: &mut BTreeMap<u64, String>) {
    if node["type"] == "TextRun" && !node["text"].as_str().unwrap().is_empty() {
        result
            .entry(node["pi"].as_u64().unwrap())
            .or_default()
            .push_str(node["text"].as_str().unwrap());
    }
    for child in node["children"].as_array().into_iter().flatten() {
        if child["type"] != "Cell" {
            own_paragraphs(child, result);
        }
    }
}

fn svg_text<'a>(doc: &'a roxmltree::Document<'a>, text: &str) -> roxmltree::Node<'a, 'a> {
    let found: Vec<_> = doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "text")
        .filter(|n| {
            n.descendants()
                .filter(|c| c.is_text())
                .filter_map(|c| c.text())
                .collect::<String>()
                == text
        })
        .collect();
    assert_eq!(found.len(), 1, "원문 {text:?}의 누락·중복 없음");
    found[0]
}

fn svg_number(node: roxmltree::Node<'_, '_>, key: &str) -> f64 {
    node.attribute(key).unwrap().parse().unwrap()
}

#[test]
fn original_cached_cells_keep_nearest_owner_source_order_and_print_baseline() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let oracle = oracle();
    assert_eq!(oracle["dpi"], 96);
    for case in oracle["cases"].as_array().unwrap() {
        check_sources(case);
        let output = Output::new(case["key"].as_str().unwrap());
        let input = root.join(case["input"].as_str().unwrap());
        let page: Value =
            serde_json::from_str(&export(&input, &output.0.join("tree"), "render-tree")).unwrap();
        let mut all = Vec::new();
        nodes(&page, None, &mut all);
        for (kind, key) in [("Table", "table_count"), ("Image", "image_count")] {
            assert_eq!(
                all.iter().filter(|(n, _)| n["type"] == kind).count(),
                case[key].as_u64().unwrap() as usize,
                "원본 {kind} 개체의 누락·중복 없음"
            );
        }
        let ga: Vec<_> = all
            .iter()
            .filter(|(n, _)| n["type"] == "TextRun" && n["text"] == "가")
            .collect();
        assert_eq!(ga.len(), 1);
        let (first, owner) = ga[0];
        let owner = owner.expect("첫 글자는 가장 가까운 원본 셀에 속한다");
        assert_eq!(owner["row"], 0);
        assert_eq!(owner["col"], 0);
        assert_eq!(first["pi"], 0);
        assert!(coordinate(first, "x") >= coordinate(owner, "x"));
        assert!(
            coordinate(first, "x") + coordinate(first, "w")
                <= coordinate(owner, "x") + coordinate(owner, "w")
        );
        assert!(coordinate(first, "y") >= coordinate(owner, "y"));
        assert!(
            coordinate(first, "y") - coordinate(owner, "y") < coordinate(first, "h"),
            "본문에서 복사한 첫 원점을 셀 안 여백으로 재사용하지 않는다"
        );
        assert!(
            coordinate(first, "y") + coordinate(first, "h")
                <= coordinate(owner, "y") + coordinate(owner, "h")
        );
        let mut paragraphs = BTreeMap::new();
        own_paragraphs(owner, &mut paragraphs);
        let expected: BTreeMap<_, _> = case["cell_paragraphs"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, text)| (i as u64, text.as_str().unwrap().to_owned()))
            .collect();
        assert_eq!(
            paragraphs, expected,
            "같은 셀의 문단 순서·절단 뒤 내용 보존"
        );
        let svg = export(&input, &output.0.join("svg"), "svg");
        let doc = roxmltree::Document::parse(&svg).unwrap();
        let first = svg_text(&doc, "가");
        for (axis, index) in [("x", 0), ("y", 1)] {
            close(
                svg_number(first, axis),
                case["first_ga"]["origin_pt"][index].as_f64().unwrap() * 96.0 / 72.0,
            );
        }
    }
}

#[test]
fn implicit_style_inline_tables_keep_print_baseline_and_signed_advance() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let oracle = oracle();
    let mut tail_origins = Vec::new();
    for case in oracle["minimal_cases"].as_array().unwrap() {
        check_sources(case);
        let key = case["key"].as_str().unwrap();
        let input = root.join(case["input"].as_str().unwrap());
        let output = Output::new(key);
        let page: Value =
            serde_json::from_str(&export(&input, &output.0.join("tree"), "render-tree")).unwrap();
        let svg = export(&input, &output.0.join("svg"), "svg");
        let doc = roxmltree::Document::parse(&svg).unwrap();
        let mut all = Vec::new();
        nodes(&page, None, &mut all);
        let tables: Vec<_> = all.iter().filter(|(n, _)| n["type"] == "Table").collect();
        assert_eq!(tables.len(), 1);
        let table = tables[0].0;
        assert_eq!(table["pi"], 0);
        close(
            coordinate(table, "h"),
            case["source_table_height_hu"].as_f64().unwrap() * 96.0 / 7200.0,
        );
        let word = if key.starts_with("pr8031") {
            "BODY"
        } else {
            "NEXT"
        };
        let body: Vec<_> = all
            .iter()
            .filter(|(n, _)| n["type"] == "TextRun" && n["text"] == word)
            .collect();
        assert_eq!(body.len(), 1);
        assert!(
            body[0].1.is_none(),
            "후속 본문을 빈 표 셀로 이동하지 않는다"
        );
        assert_eq!(body[0].0["pi"], 1);
        assert!(coordinate(body[0].0, "y") > coordinate(table, "y"));
        assert!(
            coordinate(body[0].0, "y") < coordinate(table, "y") + coordinate(table, "h"),
            "0% 줄간격의 음수 전진 감소량과 표 물리 점유를 구별한다"
        );
        // SVG는 글자별 노드로 출력할 수 있다. 단어 노드 하나라는 구현 가정을 하지 않는다.
        let painted = svg_text(&doc, &word[..1]);
        let glyphs: Vec<_> = word
            .chars()
            .map(|c| svg_text(&doc, &c.to_string()))
            .collect();
        for pair in glyphs.windows(2) {
            assert!(svg_number(pair[0], "x") < svg_number(pair[1], "x"));
            assert_eq!(svg_number(pair[0], "y"), svg_number(pair[1], "y"));
        }
        let reference = case["spans"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["text"] == word)
            .unwrap();
        for (axis, index) in [("x", 0), ("y", 1)] {
            close(
                svg_number(painted, axis),
                reference["origin"][index].as_f64().unwrap() * 96.0 / 72.0,
            );
        }
        assert!(painted.attribute("font-family").unwrap().contains("Gulim"));
        close(
            svg_number(painted, "font-size"),
            reference["size"].as_f64().unwrap() * 96.0 / 72.0,
        );
        let text: String = all
            .iter()
            .filter(|(n, _)| n["type"] == "TextRun")
            .map(|(n, _)| n["text"].as_str().unwrap())
            .collect();
        if word == "BODY" {
            assert_eq!(text, "BODY- 1 -", "본문과 해소한 쪽 번호를 각각 한 번 보존");
            let footer = svg_text(&doc, "1");
            assert!(svg_number(footer, "y") > svg_number(painted, "y"));
        } else {
            assert_eq!(text, "NEXT");
            tail_origins.push((svg_number(painted, "x"), svg_number(painted, "y")));
        }
    }
    assert_eq!(tail_origins.len(), 2);
    assert_eq!(
        tail_origins[0], tail_origins[1],
        "비가시 컨트롤 꼬리는 본문 원점을 바꾸지 않는다"
    );
}
