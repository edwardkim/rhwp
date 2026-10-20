//! 독립 한컴 Print 전쪽 Native/fresh WASM 100% 뒤 고정한 빈 부동 개체 호스트 계약.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::{path::Path, process::Command};

fn walk<'a>(node: &'a Value, found: &mut Vec<&'a Value>) {
    found.push(node);
    for child in node["children"].as_array().into_iter().flatten() {
        walk(child, found);
    }
}

fn check(kind: &str, following: usize) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join(format!(
        "tests/fixtures/issue_7642/empty-square/empty-square-{kind}-follower-hancom-2020.hwp"
    ));
    let source = rhwp::parser::parse_document(&std::fs::read(&path).unwrap()).unwrap();
    let paragraphs = &source.sections[0].paragraphs;
    let host = &paragraphs[0].line_segs[0];
    let next = &paragraphs[following].line_segs[0];
    // 한컴 정상 저장본의 원점·줄 높이가 독립 기대값이다. 구현 helper는 호출하지 않는다.
    let expected = f64::from(next.vertical_pos - host.vertical_pos) / f64::from(next.line_height);
    let dir = std::env::temp_dir().join(format!(
        "rhwp-empty-square-{}-{}-{kind}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let output = Command::new(binary)
        .args([
            "export-render-tree",
            path.to_str().unwrap(),
            "-o",
            dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    assert_eq!(files.len(), 1, "독립 Print는 전체 한 쪽");
    let tree: Value = serde_json::from_slice(&std::fs::read(&files[0]).unwrap()).unwrap();
    std::fs::remove_dir_all(dir).unwrap();
    let mut nodes = Vec::new();
    walk(&tree, &mut nodes);
    let column: Vec<_> = nodes.iter().filter(|n| n["type"] == "Column").collect();
    assert_eq!(column.len(), 1);
    let lines: Vec<_> = nodes
        .iter()
        .filter(|n| n["type"] == "TextLine" && n["pi"] == following)
        .collect();
    assert_eq!(lines.len(), 1, "후속 문단의 소유·누락·중복");
    let body_top = column[0]["bbox"]["y"].as_f64().unwrap();
    let next_top = lines[0]["bbox"]["y"].as_f64().unwrap();
    let next_height = lines[0]["bbox"]["h"].as_f64().unwrap();
    let actual = (next_top - body_top) / next_height;
    // JSON bbox 반올림을 줄 높이에 대한 무차원 비율로 허용한다.
    assert!(
        (actual - expected).abs() < 0.01,
        "빈 호스트 줄을 소비한 뒤 본문 원점: 비율 {actual} != {expected}"
    );
    let content: Vec<_> = nodes
        .iter()
        .filter(|n| n["type"] == "TextRun" && n["text"] == "후속본문")
        .collect();
    assert_eq!(content.len(), 1, "본문 내용 누락·중복");
    assert_eq!(content[0]["pi"], following);
    let pictures: Vec<_> = nodes.iter().filter(|n| n["type"] == "Image").collect();
    assert_eq!(pictures.len(), 1, "부동 개체 누락·중복");
    assert_eq!(pictures[0]["pi"], 0, "부동 그림의 문단 소유");
    if kind == "empty" {
        let host_lines: Vec<_> = nodes
            .iter()
            .filter(|n| n["type"] == "TextLine" && n["pi"] == 0)
            .collect();
        let middle_lines: Vec<_> = nodes
            .iter()
            .filter(|n| n["type"] == "TextLine" && n["pi"] == 1)
            .collect();
        assert_eq!(host_lines.len(), 1);
        assert_eq!(middle_lines.len(), 1);
        let bottom = |line: &Value| {
            line["bbox"]["y"].as_f64().unwrap() + line["bbox"]["h"].as_f64().unwrap()
        };
        assert!(bottom(host_lines[0]) <= middle_lines[0]["bbox"]["y"].as_f64().unwrap());
        assert!(bottom(middle_lines[0]) <= next_top, "빈 줄 순서·겹침");
    }
}

#[test]
fn empty_follower_preserves_host_line_and_visible_content_origin() {
    check("empty", 2);
}

#[test]
fn visible_follower_keeps_existing_stored_advance_as_control() {
    check("text", 1);
}
