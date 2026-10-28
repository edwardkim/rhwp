//! #7518 조각 검사의 공통 입력·실제 렌더·소유 판독 도우미.
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
pub(super) const SAMPLE: &str = "samples/issue7500/picture_band_cell_spaces.hwp";

pub(super) fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned())
}

pub(super) struct TempDir(pub(super) PathBuf);
impl TempDir {
    pub(super) fn new() -> Self {
        // 같은 시계 tick의 병렬 검사도 서로 다른 출력 경로를 사용한다.
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "rhwp-7518-{}-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub(super) fn rendered_pages(input: &Path) -> Vec<Value> {
    let dir = TempDir::new();
    let output = Command::new(rhwp_bin())
        .args(["export-render-tree"])
        .arg(input)
        .arg("-o")
        .arg(&dir.0)
        .output()
        .expect("run actual renderer");
    assert!(output.status.success(), "{output:?}");
    let mut paths: Vec<_> = std::fs::read_dir(&dir.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap())
        .collect()
}

pub(super) fn original_pages() -> Vec<Value> {
    rendered_pages(&Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE))
}

pub(super) fn regulatory_page39() -> (rhwp::model::document::Document, Value) {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/76076_regulatory_analysis.hwp");
    let source = rhwp::parser::parse_hwp(&std::fs::read(&input).unwrap()).unwrap();
    (source, rendered_pages(&input)[38].clone())
}

pub(super) fn source_table(para: &rhwp::model::paragraph::Paragraph) -> &rhwp::model::table::Table {
    match para.controls.as_slice() {
        [rhwp::model::control::Control::Table(table)] => table,
        _ => panic!("one source table"),
    }
}

pub(super) fn body_node<'a>(page: &'a Value, kind: &str, pi: usize) -> &'a Value {
    nodes(page)
        .into_iter()
        .find(|node| node["type"] == kind && node["pi"] == pi)
        .expect("source paragraph belongs to page 39")
}

pub(super) fn nodes(node: &Value) -> Vec<&Value> {
    let mut all = vec![node];
    if let Some(children) = node["children"].as_array() {
        for child in children {
            all.extend(nodes(child));
        }
    }
    all
}

pub(super) fn cell_owned_nodes(node: &Value) -> Vec<&Value> {
    if node["type"] == "Table" {
        return Vec::new();
    }
    let mut all = vec![node];
    if let Some(children) = node["children"].as_array() {
        for child in children {
            all.extend(cell_owned_nodes(child));
        }
    }
    all
}

pub(super) fn outer(page: &Value) -> &Value {
    nodes(page)
        .into_iter()
        .find(|node| node["type"] == "Table" && node["pi"] == 3)
        .expect("original outer table")
}

pub(super) fn cell(table: &Value, row: u32, col: u32) -> &Value {
    table["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["type"] == "Cell" && node["row"] == row && node["col"] == col)
        .expect("outer cell")
}

pub(super) fn coord(node: &Value, key: &str) -> f64 {
    node["bbox"][key].as_f64().unwrap()
}

pub(super) fn bottom(node: &Value) -> f64 {
    coord(node, "y") + coord(node, "h")
}

pub(super) fn nested(owner: &Value) -> &Value {
    nodes(owner)
        .into_iter()
        .find(|node| node["type"] == "Table")
        .expect("nested table")
}

pub(super) fn first_line(owner: &Value) -> &Value {
    nodes(owner)
        .into_iter()
        .find(|node| node["type"] == "TextLine" && node["pi"] == 8)
        .expect("remaining timeline line")
}

pub(super) fn assert_terminal_nested_contents(input: &Path, pages: &[Value]) {
    use std::collections::BTreeMap;
    let source = rhwp::parser::parse_hwp(&std::fs::read(input).unwrap()).unwrap();
    let table = source_table(&source.sections[0].paragraphs[3]);
    let owner = table
        .cells
        .iter()
        .find(|cell| cell.row == 5 && cell.col == 1)
        .expect("source owner cell");
    let child = owner
        .paragraphs
        .iter()
        .flat_map(|para| &para.controls)
        .find_map(|control| match control {
            rhwp::model::control::Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("source nested table");
    let normalize = |text: String| {
        text.chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    };
    let expected: BTreeMap<_, _> = child
        .cells
        .iter()
        .map(|cell| {
            let text = cell
                .paragraphs
                .iter()
                .map(|para| para.text.as_str())
                .collect();
            ((u64::from(cell.row), u64::from(cell.col)), normalize(text))
        })
        .collect();
    let mut actual = BTreeMap::<(u64, u64), String>::new();
    for page in pages {
        let owner = cell(outer(page), 5, 1);
        for table in nodes(owner).into_iter().filter(|n| n["type"] == "Table") {
            for cell in table["children"].as_array().unwrap() {
                if cell["type"] != "Cell" {
                    continue;
                }
                let id = (cell["row"].as_u64().unwrap(), cell["col"].as_u64().unwrap());
                let text = actual.entry(id).or_default();
                for run in nodes(cell).into_iter().filter(|n| n["type"] == "TextRun") {
                    text.push_str(run["text"].as_str().unwrap());
                }
            }
        }
    }
    let actual: BTreeMap<_, _> = actual
        .into_iter()
        .map(|(id, text)| (id, normalize(text)))
        .collect();
    assert_eq!(
        actual, expected,
        "nested cell ownership/content must survive exactly once"
    );
}

pub(super) fn assert_recursive_reflow_child(input: &str, expected_final_child_rows: usize) {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pr7518_review_page_budget/valid_orientation")
        .join(input);
    let pages = rendered_pages(&input);
    assert_recursive_child_page_owners(&pages);
    let source = rhwp::parser::parse_hwp(&std::fs::read(&input).unwrap()).unwrap();
    let source_outer = source_table(&source.sections[0].paragraphs[3]);
    let source_owner = source_outer
        .cells
        .iter()
        .find(|cell| cell.row == 5 && cell.col == 1)
        .unwrap();
    let source_child = source_table(source_owner.paragraphs.last().unwrap());
    let normalize = |text: &str| text.split_whitespace().collect::<String>();
    let expected_following: std::collections::BTreeMap<_, _> = source_outer
        .cells
        .iter()
        .filter(|cell| cell.row == 7)
        .map(|cell| {
            (
                u64::from(cell.col),
                normalize(
                    &cell
                        .paragraphs
                        .iter()
                        .map(|p| p.text.as_str())
                        .collect::<String>(),
                ),
            )
        })
        .collect();
    let mut first = true;
    let mut fragments = 0;
    let mut source_units = std::collections::BTreeSet::new();
    let mut final_child_rows = 0;
    let mut following_content = std::collections::BTreeMap::<u64, String>::new();
    for page in &pages {
        let all = nodes(page);
        let column = all.iter().find(|n| n["type"] == "Column").unwrap();
        for owner in outer(page)["children"].as_array().unwrap() {
            if owner["type"] != "Cell" {
                continue;
            }
            if owner["row"] == 7 {
                let text: String = cell_owned_nodes(owner)
                    .into_iter()
                    .filter(|node| node["type"] == "TextRun")
                    .filter_map(|node| node["text"].as_str())
                    .collect();
                following_content
                    .entry(owner["col"].as_u64().unwrap())
                    .or_default()
                    .push_str(&normalize(&text));
            }
            if owner["row"] != 5 || owner["col"] != 1 {
                continue;
            }
            assert!(
                bottom(owner) <= bottom(column) + 0.2,
                "paint expanded the accepted row budget"
            );
            for child in nodes(owner).into_iter().filter(|n| n["type"] == "Table") {
                fragments += 1;
                // The PDF repeats the declared outer top at each recursive
                // fragment. Only the paragraph offset belongs to its first.
                let lead = (f64::from(source_child.outer_margin_top)
                    + if first {
                        f64::from(source_child.common.vertical_offset)
                    } else {
                        0.0
                    })
                    / 75.0;
                assert!(
                    (coord(child, "y") - coord(owner, "y") - 141.0 / 75.0 - lead).abs() < 0.2,
                    "recursive paint added the paragraph offset twice"
                );
                assert!(
                    (coord(child, "x") - coord(owner, "x") - (141.0 + 283.0 + 341.0) / 75.0).abs()
                        < 0.2
                );
                // Padding is measured from the child cell frame. Table's paint
                // bbox includes half of the border stroke beyond that frame.
                let child_frame_bottom = child["children"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|node| node["type"] == "Cell")
                    .map(bottom)
                    .reduce(f64::max)
                    .expect("recursive child frame");
                assert!(child_frame_bottom <= bottom(owner) - 141.0 / 75.0 + 0.2);
                assert!(
                    bottom(child) <= bottom(owner) + 0.2,
                    "child stroke leaves its owner"
                );
                first = false;
                for node in nodes(child) {
                    if node["type"] != "TextRun" {
                        continue;
                    }
                    let Some(text) = node["text"].as_str() else {
                        continue;
                    };
                    if text.starts_with("UNIT ") {
                        assert!(source_units.insert(text.to_owned()), "source unit replayed");
                    }
                    if text == "FINAL CHILD ROW" {
                        final_child_rows += 1;
                    }
                }
            }
        }
    }
    assert!(fragments >= 2, "exercise a cut inside the child row");
    let expected: std::collections::BTreeSet<_> = (0..65).map(|i| format!("UNIT {i:03}")).collect();
    assert_eq!(
        source_units, expected,
        "every child paragraph must survive once"
    );
    assert_eq!(final_child_rows, expected_final_child_rows);
    assert_eq!(
        following_content, expected_following,
        "following row content must survive exactly once across its physical fragments"
    );
}

pub(super) fn assert_recursive_child_page_owners(pages: &[Value]) {
    // Exact-input Hancom 2020 PDFs in valid_orientation and valid_generated:
    // p4 owns 000..017, p5 018..036, p6 037..055, p7 056..064.
    // p8 contains the following outer row, not another child unit/blank page.
    assert_eq!(pages.len(), 8);
    for (index, page) in pages.iter().enumerate() {
        let actual: Vec<_> = nodes(page)
            .into_iter()
            .filter(|node| node["type"] == "TextRun")
            .filter_map(|node| node["text"].as_str())
            .filter(|text| text.starts_with("UNIT "))
            .map(str::to_owned)
            .collect();
        let range = match index {
            3 => 0..18,
            4 => 18..37,
            5 => 37..56,
            6 => 56..65,
            _ => 0..0,
        };
        let expected: Vec<_> = range.map(|unit| format!("UNIT {unit:03}")).collect();
        assert_eq!(
            actual,
            expected,
            "Hancom child owners on page {}",
            index + 1
        );
    }
    let last = pages.last().unwrap();
    assert!(cell_owned_nodes(cell(outer(last), 7, 1))
        .into_iter()
        .any(|node| node["type"] == "TextRun"
            && node["text"]
                .as_str()
                .is_some_and(|text| !text.trim().is_empty())));
}

pub(super) fn assert_independent_nested_print_edges() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pr7518_review_page_budget/valid_orientation/nested-split.hwp");
    let pages = rendered_pages(&input);
    // 원본 그대로 한컴 2020 Print, 96 DPI의 독립 경계다. 관련 전쪽 양쪽 raw 90%와
    // 이 두 쪽 Native/fresh WASM raw 98% 이상·직접 판독 증거를 먼저 고정한다.
    let fourth = cell(outer(&pages[3]), 5, 1);
    let fourth_child = nested(fourth);
    let fifth = cell(outer(&pages[4]), 5, 1);
    let fifth_child = nested(fifth);
    for (actual, expected, label) in [
        (coord(fourth, "y"), 58.4040, "p4 부모 시작"),
        (bottom(fourth), 437.5507, "p4 부모 종료"),
        (coord(fourth_child, "y"), 79.9467, "p4 자식 시작"),
        (bottom(fourth_child), 431.8053, "p4 자식 종료"),
        (coord(fifth_child, "y"), 63.9893, "p5 자식 시작"),
        (bottom(fifth_child), 183.6693, "p5 자식 종료"),
    ] {
        assert!(
            (actual - expected).abs() <= 2.0,
            "{label}: {actual} / {expected}"
        );
    }
}
