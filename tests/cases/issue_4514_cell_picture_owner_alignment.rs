//! 셀의 가운데 정렬 TAC 그림은 가용 글줄 상자의 가운데를 소유한다.
//! 실제 원본과 독립 Print의 5쪽 경계를 Native/fresh WASM으로 검증한 뒤 추가한다.
//! 절대 좌표를 고정하지 않고 동일 셀의 owner 줄과 그림의 정렬·포함 관계를 검사한다.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::style::Alignment;
use serde_json::Value;

const SAMPLE: &str = "samples/issue4514/sample1-repro.hwp";
const HOST_PARAGRAPH: usize = 75;

struct TemporaryTree(PathBuf);

impl TemporaryTree {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system timestamp")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rhwp-issue4514-tree-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create owned temporary directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryTree {
    fn drop(&mut self) {
        let _ = fs::remove_file(self.0.join("render_tree_005.json"));
        let _ = fs::remove_dir(&self.0);
    }
}

fn nodes(node: &Value) -> Vec<&Value> {
    let mut result = vec![node];
    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            result.extend(nodes(child));
        }
    }
    result
}

fn box_value(node: &Value, name: &str) -> f64 {
    node["bbox"][name].as_f64().expect("render-tree bbox")
}

#[test]
fn centered_cell_picture_preserves_its_owner_line_alignment() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let source = DocumentCore::from_bytes(&fs::read(&input).expect("read original"))
        .expect("parse original");
    let paragraph = &source.document().sections[0].paragraphs[HOST_PARAGRAPH];
    let Control::Table(table) = &paragraph.controls[0] else {
        panic!("original diagram table owner");
    };
    let cell = &table.cells[0];
    let owner = &cell.paragraphs[0];
    assert_eq!(
        source.document().doc_info.para_shapes[owner.para_shape_id as usize].alignment,
        Alignment::Center,
        "independent source paragraph requests centered inline content"
    );
    let Control::Picture(picture) = &owner.controls[0] else {
        panic!("original inline diagram");
    };
    assert!(picture.common.treat_as_char);
    assert_eq!(picture.common.margin.left, picture.common.margin.right);

    // Test-only override executes this same contract against a frozen pre-fix CLI.
    let binary = std::env::var_os("RHWP_4514_RENDER_BIN")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_rhwp").into());
    let output = TemporaryTree::new();
    let result = Command::new(binary)
        .arg("export-render-tree")
        .arg(&input)
        .args(["--page", "4", "--compat", "2022", "--output"])
        .arg(output.path())
        .output()
        .expect("export original page");
    assert!(
        result.status.success(),
        "render export failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let tree: Value = serde_json::from_slice(
        &fs::read(output.path().join("render_tree_005.json")).expect("read page tree"),
    )
    .expect("parse page tree");
    let table_node = nodes(&tree)
        .into_iter()
        .find(|node| node["type"] == "Table" && node["pi"].as_u64() == Some(HOST_PARAGRAPH as u64))
        .expect("diagram table stays on its Print page");
    let cell_node = table_node["children"]
        .as_array()
        .expect("table children")
        .iter()
        .find(|node| node["type"] == "Cell" && node["row"] == 0 && node["col"] == 0)
        .expect("diagram owner cell");
    let contents = nodes(cell_node);
    let line = contents
        .iter()
        .find(|node| node["type"] == "TextLine" && node["pi"] == 0)
        .expect("source owner line");
    let pictures: Vec<_> = contents
        .iter()
        .filter(|node| {
            node["type"] == "Image" && node["pi"].as_u64() == Some(HOST_PARAGRAPH as u64)
        })
        .collect();
    assert_eq!(
        pictures.len(),
        1,
        "diagram is neither omitted nor duplicated"
    );
    let image = pictures[0];
    let line_width = box_value(line, "w");
    assert!(line_width > box_value(image, "w"));
    let center_error = ((box_value(image, "x") + box_value(image, "w") / 2.0)
        - (box_value(line, "x") + line_width / 2.0))
        .abs()
        / line_width;
    // CLI simplified bbox fields round to one decimal place; propagate that
    // serialization precision into this dimensionless alignment relation.
    let serialization_quantum = 10_f64.powi(-1);
    let precision = 2.0 * serialization_quantum / line_width;
    assert!(
        center_error <= precision,
        "centered diagram must share the owner line center; normalized error={center_error}"
    );
    assert!(box_value(image, "x") >= box_value(cell_node, "x"));
    assert!(
        box_value(image, "x") + box_value(image, "w")
            <= box_value(cell_node, "x") + box_value(cell_node, "w")
    );
}
