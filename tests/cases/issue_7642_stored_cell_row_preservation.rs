//! 유효한 저장 셀의 어절 꼬리는 새 측정에서 한 줄에 들어가도 합치지 않는다.
//! 독립 Print 122쪽과 Native/fresh WASM 전쪽 검증 근거는 fixture README에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{document_core::DocumentCore, model::control::Control};
use serde_json::Value;
use std::{path::PathBuf, process::Command};

struct TemporaryTree(PathBuf);

impl Drop for TemporaryTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
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

fn bounds(node: &Value) -> [f64; 4] {
    let bbox = &node["bbox"];
    let x = bbox["x"].as_f64().unwrap();
    let y = bbox["y"].as_f64().unwrap();
    [
        x,
        y,
        x + bbox["w"].as_f64().unwrap(),
        y + bbox["h"].as_f64().unwrap(),
    ]
}

#[test]
fn stored_cell_tail_keeps_two_rows_ownership_and_print_boundary() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input = root.join("samples/hwp-3.0-HWPML.hwp");
    let bytes = std::fs::read(&input).unwrap();
    let source = rhwp::parser::parse_document(&bytes).unwrap();
    let source_table = source.sections[3].paragraphs[215]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("원본 TABLE 속성 설명 표");
    let source_cell = source_table
        .cells
        .iter()
        .find(|cell| cell.row == 5 && cell.col == 4)
        .expect("RepeatHeader의 기본값 셀");
    assert_eq!(source_cell.paragraphs.len(), 1);
    let paragraph = &source_cell.paragraphs[0];
    assert_eq!(paragraph.text, "true");
    assert_eq!(
        paragraph
            .line_segs
            .iter()
            .map(|line| line.text_start)
            .collect::<Vec<_>>(),
        [0, 3],
        "독립 Print의 tru/e에 대응하는 저장 줄 소속"
    );
    assert_eq!(
        DocumentCore::from_bytes(&bytes).unwrap().page_count(),
        122,
        "같은 원본의 독립 Print 전체 쪽수"
    );

    let temporary = TemporaryTree(std::env::temp_dir().join(format!(
        "rhwp-7642-stored-cell-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )));
    std::fs::create_dir(&temporary.0).unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let output = Command::new(binary)
        .arg("export-render-tree")
        .arg(&input)
        .args(["--compat", "2022", "--page", "92", "-o"])
        .arg(&temporary.0)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let files: Vec<_> = std::fs::read_dir(&temporary.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    assert_eq!(files.len(), 1, "지정한 93쪽의 render tree 한 개");
    let page: Value = serde_json::from_slice(&std::fs::read(&files[0]).unwrap()).unwrap();
    let all = nodes(&page);
    let table = all
        .iter()
        .find(|node| node["type"] == "Table" && node["pi"] == 215 && node["ci"] == 0)
        .unwrap();
    let cell = table["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["type"] == "Cell" && node["row"] == 5 && node["col"] == 4)
        .unwrap();
    let lines: Vec<_> = cell["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|node| node["type"] == "TextLine")
        .collect();
    let text: Vec<String> = lines
        .iter()
        .map(|line| {
            nodes(line)
                .into_iter()
                .filter(|node| node["type"] == "TextRun")
                .map(|node| {
                    assert_eq!(node["pi"], 0, "글자가 원래 셀 문단에 속한다");
                    node["text"].as_str().unwrap()
                })
                .collect()
        })
        .collect();
    assert_eq!(text, ["tru", "e"], "줄 경계와 원문 순서·누락·중복");
    assert_eq!(text.concat(), paragraph.text);
    let cell_bounds = bounds(cell);
    for line in &lines {
        let line_bounds = bounds(line);
        assert!(line_bounds[0] >= cell_bounds[0]);
        assert!(line_bounds[1] >= cell_bounds[1]);
        assert!(line_bounds[2] <= cell_bounds[2] + 1e-6);
        assert!(line_bounds[3] <= cell_bounds[3] + 1e-6);
    }
    assert!(bounds(lines[0])[3] <= bounds(lines[1])[1]);
    let following = all
        .iter()
        .find(|node| node["type"] == "Table" && node["pi"] == 216)
        .unwrap();
    assert!(
        bounds(table)[3] < bounds(following)[1],
        "뒤 표와 겹치지 않는다"
    );

    // 93쪽 양쪽 raw가 99%대이고 직접 판독을 마친 경계만 ±2px로 보호한다.
    // 이 값은 renderer 계산식이 아니라 독립 PDF의 선분 교점(@96dpi)이다.
    let independent_print = [626.543, 286.407, 665.403, 317.732];
    for (actual, expected) in cell_bounds.into_iter().zip(independent_print) {
        assert!(
            (actual - expected).abs() <= 2.0,
            "93쪽 독립 Print 셀 경계: {actual} != {expected} ±2px"
        );
    }
}
