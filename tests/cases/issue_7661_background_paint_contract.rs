//! PR #7661: 독립 한컴 Print로 확인한 글줄별 문단 배경·셀 레이어 계약.
//! fixture 생성·전쪽 Native/fresh WASM 증거는 pr_7661_review.md에 연결한다.
//! 소유는 입력의 문단·줄 순서로 고르고, 포함·레이어 순서를 검사한다.
#![cfg(not(target_arch = "wasm32"))]
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

struct OutputDir(PathBuf);
impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn tree(name: &str, page: u32) -> Value {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root
        .join("tests/fixtures/issue7661_backgrounds")
        .join(format!("{name}.hwpx"));
    tree_path(&path, page)
}
fn tree_path(path: &Path, page: u32) -> Value {
    let dir = OutputDir(std::env::temp_dir().join(format!(
            "rhwp-7661-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(bin)
        .arg("export-render-tree")
        .arg(path)
        .args(["-p", &page.to_string(), "--compat", "2022", "-o"])
        .arg(&dir.0)
        .output()
        .expect("run renderer");
    assert!(
        result.status.success(),
        "renderer failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(
        &std::fs::read(dir.0.join(format!("render_tree_{:03}.json", page + 1))).unwrap(),
    )
    .unwrap()
}
fn children(n: &Value) -> &[Value] {
    n["children"].as_array().map_or(&[], Vec::as_slice)
}
fn ty(n: &Value) -> &str {
    n["type"].as_str().unwrap_or("")
}
fn find<'a>(n: &'a Value, p: &impl Fn(&Value) -> bool) -> Option<&'a Value> {
    if p(n) {
        return Some(n);
    }
    children(n).iter().find_map(|n| find(n, p))
}
fn column(root: &Value) -> &Value {
    find(root, &|n| ty(n) == "Column").expect("body column")
}
fn text(n: &Value) -> String {
    let mut s = n["text"].as_str().unwrap_or("").to_owned();
    for c in children(n) {
        s.push_str(&text(c));
    }
    s
}
fn rect(n: &Value) -> (f64, f64, f64, f64) {
    let b = &n["bbox"];
    let (x, y, w, h) = (
        b["x"].as_f64().unwrap(),
        b["y"].as_f64().unwrap(),
        b["w"].as_f64().unwrap(),
        b["h"].as_f64().unwrap(),
    );
    (x, y, x + w, y + h)
}
fn contains(outer: &Value, inner: &Value) {
    let (a, b, c, d) = rect(outer);
    let (x, y, r, q) = rect(inner); // JSON is rounded to 0.1px per component.
    assert!(
        a <= x + 0.2 && b <= y + 0.2 && c + 0.2 >= r && d + 0.2 >= q,
        "background must contain its source line: {:?} {:?}",
        rect(outer),
        rect(inner)
    );
}
fn bands(col: &Value) -> Vec<&Value> {
    children(col)
        .iter()
        .filter(|n| matches!(ty(n), "Rect" | "Image"))
        .collect()
}
fn source_lines(col: &Value, pi: u64) -> Vec<&Value> {
    children(col)
        .iter()
        .filter(|n| ty(n) == "TextLine" && n["pi"].as_u64() == Some(pi))
        .collect()
}

#[test]
fn inline_plan_keeps_one_background_per_owned_line_including_blank_line() {
    let root = tree("flow-band-ownership", 0);
    let col = column(&root);
    let fills = bands(col);
    // Input paragraphs: same-line pair, explicitly separated pair, text/space/text.
    assert_eq!(
        fills.iter().map(|n| ty(n)).collect::<Vec<_>>(),
        ["Rect", "Image", "Image", "Rect", "Rect", "Rect"]
    );
    let lines: Vec<_> = (1..=3).flat_map(|pi| source_lines(col, pi)).collect();
    assert_eq!(lines.len(), 6);
    for (fill, line) in fills.iter().zip(&lines) {
        contains(fill, line);
    }
    assert!(
        !text(lines[4]).is_empty() && text(lines[4]).chars().all(char::is_whitespace),
        "middle line owns real spaces"
    );
    assert!(
        rect(lines[4]).3 > rect(lines[4]).1,
        "blank line occupies height"
    );
    assert!(rect(lines[3]).3 <= rect(lines[4]).1 + 0.2);
    assert!(rect(lines[4]).3 <= rect(lines[5]).1 + 0.2);
    for pi in [1, 2] {
        let tables: Vec<_> = children(col)
            .iter()
            .filter(|n| ty(n) == "Table" && n["pi"].as_u64() == Some(pi))
            .collect();
        assert_eq!(tables.len(), 2);
        assert!(text(tables[0]).contains("Table A"));
        assert!(text(tables[1]).contains("Table B"));
    }
    let first = source_lines(col, 1);
    assert_eq!(first.len(), 1);
    let next = source_lines(col, 2);
    assert_eq!(next.len(), 2);
    assert!(rect(next[0]).3 <= rect(next[1]).1 + 0.2);
}
#[test]
fn connected_borders_do_not_merge_fill_ownership_or_copy_fill_to_none() {
    let root = tree("connected-backgrounds", 0);
    let col = column(&root);
    let fills = bands(col);
    assert_eq!(
        fills.iter().map(|n| ty(n)).collect::<Vec<_>>(),
        ["Rect", "Rect", "Image"]
    );
    for (i, fill) in fills.iter().enumerate() {
        let lines = source_lines(col, (i + 1) as u64);
        assert_eq!(lines.len(), 1);
        contains(fill, lines[0]);
    }
    let none = source_lines(col, 4);
    assert_eq!(none.len(), 1);
    assert!(text(none[0]).contains("No background"));
}
#[test]
fn continuation_preserves_all_lines_and_restarts_background_each_line() {
    let mut contents = Vec::new();
    let mut count = 0;
    let mut tail_pages = Vec::new();
    for page in 0..3 {
        let root = tree("continued-paragraph", page);
        let col = column(&root);
        let fills = bands(col);
        let lines = source_lines(col, 1);
        assert_eq!(fills.len(), lines.len());
        let tail = source_lines(col, 2);
        if let Some(next) = tail.first() {
            assert_eq!(tail.len(), 1);
            assert_eq!(text(next).trim(), "After the continued paragraph.");
            assert!(rect(lines.last().expect("continued lines")).3 <= rect(next).1 + 0.2);
            tail_pages.push(page);
        }
        for (fill, line) in fills.iter().zip(lines) {
            contains(fill, line);
            contents.push(text(line));
            count += 1;
        }
    }
    assert_eq!(
        count, 85,
        "independent Print contains all 85 input lines across three pages"
    );
    assert_eq!(
        tail_pages,
        [2],
        "following paragraph survives once after the last line"
    );
    for (i, line) in contents.iter().enumerate() {
        assert_eq!(line.trim(), format!("Continuation line {:03}.", i + 1));
    }
}
#[test]
fn cell_background_stays_below_paragraph_image_and_text() {
    let root = tree("cell-background-layers", 0);
    let cell = find(&root, &|n| {
        ty(n) == "Cell" && text(n).contains("Cell paragraph image over gradient.")
    })
    .expect("source cell");
    let nodes = children(cell);
    let gradient = nodes
        .iter()
        .position(|n| ty(n) == "Rect")
        .expect("cell gradient");
    let image = nodes
        .iter()
        .position(|n| ty(n) == "Image")
        .expect("paragraph image");
    let line = nodes
        .iter()
        .position(|n| ty(n) == "TextLine" && text(n).contains("Cell paragraph"))
        .expect("paragraph text");
    assert!(
        gradient < image && image < line,
        "cell / paragraph fill / text layer order"
    );
    contains(&nodes[gradient], &nodes[image]);
    contains(&nodes[image], &nodes[line]);
}

#[test]
fn carried_tac_line_keeps_its_full_source_gap_at_the_page_origin_reset() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue7235/156086935_none_image_fill.hwp");
    let core =
        rhwp::document_core::DocumentCore::from_bytes(&std::fs::read(&path).unwrap()).unwrap();
    let paragraphs = &core.document().sections[0].paragraphs;
    let root = tree_path(&path, 5);
    let col = column(&root);
    for label in ["사례 1", "사례 2", "사례 3"] {
        // Print page 6 has this three-case group. Earlier source pages reuse the
        // same case labels, so resolve the owner on the independently known page.
        let hosts: Vec<_> = children(col)
            .iter()
            .filter(|n| ty(n) == "Table" && text(n).contains(label))
            .collect();
        assert_eq!(hosts.len(), 1, "one case header on Print page 6: {label}");
        let table_node = hosts[0];
        let pi = table_node["pi"].as_u64().expect("source paragraph") as usize;
        let ci = table_node["ci"].as_u64().expect("source control") as usize;
        let para = &paragraphs[pi];
        let rhwp::model::control::Control::Table(table) = &para.controls[ci] else {
            panic!("rendered case must retain its source table owner")
        };
        assert!(table
            .cells
            .iter()
            .flat_map(|cell| &cell.paragraphs)
            .any(|p| p.text.contains(label)));
        let [seg] = para.line_segs.as_slice() else {
            panic!("single source object row")
        };
        assert_eq!(
            i64::from(seg.line_height),
            i64::from(table.common.height)
                + i64::from(table.outer_margin_top)
                + i64::from(table.outer_margin_bottom)
        );
        let next = source_lines(col, (pi + 1) as u64);
        assert_eq!(next.len(), 1, "following real blank line survives");
        let actual = rect(next[0]).1 - rect(table_node).3;
        let expected =
            f64::from(seg.line_spacing + i32::from(table.outer_margin_bottom)) * 96.0 / 7200.0;
        assert!((actual-expected).abs()<0.2,"{label}: source line spacing and outer bottom margin must both survive ({actual} vs {expected})");
    }
}
