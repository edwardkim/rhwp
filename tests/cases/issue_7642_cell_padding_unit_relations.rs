//! 저장 줄 없는 실물 셀의 줄바꿈·안여백·문단 소유를 독립 한컴 Print와 대조한다.
//! 시각 선행 증거와 기준 PDF는 task_m100_7642_stage3844.md에 연결한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const INPUT: &str = "samples/issue6776/78494-virtual-convergence-industry-decree.hwpx";
const FONT_ENVIRONMENT: &str = r#"{"id":"independent-legal-print-true-type","substitutions":{"한양중고딕":"HY중고딕","한양신명조":"HY신명조"},"trueTypeFaces":["HY중고딕","HY신명조"]}"#;

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}
fn collect<'a>(node: &'a Value, kind: &str, out: &mut Vec<&'a Value>) {
    if node["type"] == kind {
        out.push(node);
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect(child, kind, out);
        }
    }
}
fn edge(node: &Value, name: &str) -> f64 {
    node["bbox"][name].as_f64().unwrap()
}

#[test]
fn issue7642_reflow_cell_preserves_source_padding_pdf_wraps_and_paragraph_owners() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root.join(INPUT);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&input).expect("원본 읽기")).expect("원본 파싱");
    let tables: Vec<_> = core.document().sections[0].paragraphs[256]
        .controls
        .iter()
        .filter_map(|c| match c {
            Control::Table(t) => Some(t),
            _ => None,
        })
        .collect();
    assert_eq!(tables.len(), 1, "원본 문단의 표 소유");
    let source_table = tables[0];
    let source = source_table.cell_at(1, 2).expect("원본 대안 내용 칸");
    assert_eq!(source.paragraphs.len(), 4);
    assert!(
        source.paragraphs.iter().all(|p| p.line_segs.is_empty()),
        "저장 줄이 없는 재조판 사례"
    );
    let pad = source.effective_padding(&source_table.padding);
    let left_padding = f64::from(pad.left) * 96.0 / 7200.0;
    let right_padding = f64::from(pad.right) * 96.0 / 7200.0;
    assert!(left_padding > 0.0 && right_padding > 0.0);
    let scratch = Scratch(std::env::temp_dir().join(format!(
            "rhwp-7642-cell-padding-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    std::fs::create_dir(&scratch.0).expect("임시 출력 폴더");
    let environment = scratch.0.join("font-environment.json");
    std::fs::write(&environment, FONT_ENVIRONMENT).unwrap();
    // 같은 검사를 보존한 수정 전 CLI에도 실행해 결함 검출을 확인한다.
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(binary)
        .arg("export-render-tree")
        .arg(&input)
        .args(["--compat", "2022", "--page", "28", "--font-environment"])
        .arg(&environment)
        .arg("-o")
        .arg(&scratch.0)
        .output()
        .expect("실제 배치 출력");
    assert!(result.status.success(), "배치 출력 실패: {result:?}");
    let files: Vec<_> = std::fs::read_dir(&scratch.0)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.extension().is_some_and(|s| s == "json")
                && p.file_name().unwrap() != "font-environment.json"
        })
        .collect();
    assert_eq!(files.len(), 1, "요청 쪽의 실제 출력 누락·중복");
    let tree: Value = serde_json::from_slice(&std::fs::read(&files[0]).unwrap()).unwrap();
    let mut rendered_tables = Vec::new();
    collect(&tree, "Table", &mut rendered_tables);
    let owners: Vec<_> = rendered_tables
        .into_iter()
        .filter(|n| n["pi"] == 256)
        .collect();
    assert_eq!(owners.len(), 1, "원본 문단의 표는 이 쪽에 한 번 존재한다");
    let cells: Vec<_> = owners[0]["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|n| n["type"] == "Cell" && n["row"] == 1 && n["col"] == 2)
        .collect();
    assert_eq!(cells.len(), 1, "대안 내용 칸의 소유·중복");
    let cell = cells[0];
    let mut lines = Vec::new();
    collect(cell, "TextLine", &mut lines);
    let expected = [
        (0, "지역별 가상융합산업지원센터(이하 “지원센터”라"),
        (0, "한다)로 지정받으려는 자는 다음 각 호의 요건을"),
        (0, "모두 갖추어야 한다."),
        (1, "1. 가상융합산업 진흥을 위한 업무를 전담하여"),
        (1, "수행할 2명 이상의 상시 근로자로 구성되는 별"),
        (1, "도의 조직을 갖출 것"),
        (2, "2. 지원센터의 장은 가상융합산업에 관한 분야에서"),
        (2, "3년 이상 실무에 종사한 사람으로서 가상융합산"),
        (2, "업에 관한 전문지식과 경험을 갖춘 사람일 것"),
        (3, "3. 가상융합산업 진흥을 위한 사업의 수행에 필"),
        (3, "요한 시설 및 장비를 보유하고 있을 것"),
    ];
    assert_eq!(lines.len(), expected.len(), "독립 Print의 11개 줄");
    // bbox JSON은 0.1px 단위다. 두 상자의 반올림 오차만 허용하며 절대 위치를 고정하지 않는다.
    let serialization_error = 0.101;
    assert!(
        ((edge(lines[0], "x") - edge(cell, "x")) - left_padding).abs() <= serialization_error,
        "정확히 들어가는 내용 때문에 원본 안여백을 축소하면 안 된다"
    );
    let mut rendered_text = String::new();
    for (line, (owner, text)) in lines.iter().zip(expected) {
        assert_eq!(line["pi"], owner, "문단 소유와 줄 순서");
        let mut runs = Vec::new();
        collect(line, "TextRun", &mut runs);
        let actual: String = runs
            .iter()
            .map(|r| {
                assert_eq!(r["pi"], owner);
                r["text"].as_str().unwrap()
            })
            .collect();
        assert_eq!(compact(&actual), compact(text), "독립 Print의 줄 끝");
        rendered_text.push_str(&actual);
        assert!(edge(line, "x") >= edge(cell, "x") + left_padding - serialization_error);
        assert!(
            edge(line, "x") + edge(line, "w")
                <= edge(cell, "x") + edge(cell, "w") - right_padding + serialization_error
        );
        assert!(edge(line, "y") >= edge(cell, "y") - serialization_error);
        assert!(
            edge(line, "y") + edge(line, "h")
                <= edge(cell, "y") + edge(cell, "h") + serialization_error
        );
    }
    for pair in lines.windows(2) {
        assert!(
            edge(pair[0], "y") + edge(pair[0], "h") <= edge(pair[1], "y") + serialization_error,
            "줄 순서·비겹침"
        );
    }
    let source_text: String = source.paragraphs.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(
        compact(&rendered_text),
        compact(&source_text),
        "원본 내용 누락·중복 없이 보존"
    );
}
