//! #1749 사회기여 표 HWPX의 첫 합성 문단 간격 소유.
//! 독립 Print와 C12 Native/fresh WASM 전체 5쪽의 최저 일치율은 97.98406%다.
//! 4·5쪽 직접 판독 후 추가하며, 실제 셀의 절대 위치를 고정하지 않는다.
//! 원본 문단 모양의 앞 간격과 셀 안 여백으로 줄·셀 사이 관계를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, table::Table};
use serde_json::Value;
use std::{path::PathBuf, process::Command};

const SAMPLE: &str = "samples/task1749/saved_bounds_cumulative_page_break.hwpx";

struct TempDir(PathBuf);
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> (DocumentCore, Vec<Value>) {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&input).unwrap()).unwrap();
    let output = TempDir(std::env::temp_dir().join(format!(
        "rhwp-1749-spacing-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )));
    std::fs::create_dir(&output.0).unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let run = Command::new(binary)
        .arg("export-render-tree")
        .arg(input)
        .args(["--compat", "2022", "-o"])
        .arg(&output.0)
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "actual renderer: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let mut paths: Vec<_> = std::fs::read_dir(&output.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    paths.sort();
    let pages: Vec<Value> = paths
        .iter()
        .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap())
        .collect();
    assert_eq!(pages.len(), 5, "독립 Print의 전체 쪽 소속을 보존한다");
    (core, pages)
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

fn source_table(core: &DocumentCore, para_index: usize) -> &Table {
    core.document().sections[0].paragraphs[para_index]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .unwrap()
}

fn rendered_cell(page: &Value, para_index: usize) -> &Value {
    let table = nodes(page)
        .into_iter()
        .find(|node| node["type"] == "Table" && node["pi"] == para_index)
        .expect("원본 문단이 소유한 표");
    table["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["type"] == "Cell" && node["row"] == 2 && node["col"] == 0)
        .expect("본문 셀")
}

fn coord(node: &Value, axis: &str) -> f64 {
    node["bbox"][axis].as_f64().unwrap()
}

fn stars(node: &Value) -> usize {
    nodes(node)
        .into_iter()
        .filter(|child| child["type"] == "TextRun")
        .map(|child| child["text"].as_str().unwrap().matches('*').count())
        .sum()
}

fn star_lines(cell: &Value) -> Vec<&Value> {
    cell["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|node| node["type"] == "TextLine" && stars(node) > 0)
        .collect()
}

fn source_before(core: &DocumentCore, para_index: usize) -> f64 {
    let table = source_table(core, para_index);
    let cell = table
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 0)
        .unwrap();
    let para = &cell.paragraphs[0];
    assert!(para.controls.is_empty());
    assert!(para.line_segs.iter().all(|seg| seg.tag & 0x8000_0000 != 0));
    let stored = core.document().doc_info.para_shapes[para.para_shape_id as usize].spacing_before;
    assert!(stored > 0 && stored % 2 == 0);
    // HWP ParaShape 간격은 실제 HWPUNIT의 두 배로 저장된다.
    // 이번 원본의 HWPX namespace case prev=500과 독립 Print로 확인한 계약이다.
    rhwp::renderer::hwpunit_to_px(stored / 2, rhwp::renderer::DEFAULT_DPI)
}

#[test]
fn centered_synthetic_stack_owns_first_paragraph_before_space_once() {
    let (core, pages) = fixture();
    let cell = rendered_cell(&pages[3], 50);
    let lines = star_lines(cell);
    let first = lines.first().unwrap();
    let last = lines.last().unwrap();
    assert!(lines
        .windows(2)
        .all(|pair| coord(pair[0], "y") < coord(pair[1], "y")));
    let source = source_table(&core, 50)
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 0)
        .unwrap();
    let expected_stars: usize = source
        .paragraphs
        .iter()
        .map(|para| para.text.matches('*').count())
        .sum();
    assert_eq!(stars(cell), expected_stars, "완성 셀의 내용 보존");
    let top = coord(first, "y") - coord(cell, "y");
    let bottom = coord(cell, "y") + coord(cell, "h") - coord(last, "y") - coord(last, "h");
    assert!(top >= 0.0 && bottom >= 0.0, "글줄은 셀 안에 포함된다");
    let before = source_before(&core, 50);
    let tolerance = coord(first, "h") / 10.0;
    assert!(
        (top - bottom - before).abs() <= tolerance,
        "가운데 정렬의 상하 간격 차는 원본 앞 간격 한 번이다: top={top}, bottom={bottom}, before={before}"
    );
}

#[test]
fn opening_rowbreak_fragment_keeps_source_first_paragraph_before_space() {
    let (core, pages) = fixture();
    let cell = rendered_cell(&pages[3], 52);
    let lines = star_lines(cell);
    let first = lines.first().unwrap();
    let table = source_table(&core, 52);
    let source = table
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 0)
        .unwrap();
    assert_eq!(
        stars(first),
        source.paragraphs[0].text.matches('*').count(),
        "첫 source 문단은 앞 조각에 속한다"
    );
    let before = source_before(&core, 52);
    let padding = source.effective_padding(&table.padding);
    let top_padding =
        rhwp::renderer::hwpunit_to_px(i32::from(padding.top), rhwp::renderer::DEFAULT_DPI);
    let top = coord(first, "y") - coord(cell, "y");
    let tolerance = coord(first, "h") / 10.0;
    assert!(
        (top - top_padding - before).abs() <= tolerance,
        "첫 조각의 줄 원점은 안 여백과 원본 앞 간격을 한 번 소유한다: top={top}, padding={top_padding}, before={before}"
    );
    let last = lines.last().unwrap();
    assert!(
        coord(last, "y") + coord(last, "h") <= coord(cell, "y") + coord(cell, "h"),
        "뒤 조각으로 넘어갈 내용을 여기서 중복하거나 셀 밖으로 밀지 않는다"
    );
}
