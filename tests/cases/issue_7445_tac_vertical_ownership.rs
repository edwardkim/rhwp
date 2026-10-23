//! 가로 펜만 인증된 원본 TAC 표가 캡션의 저장 세로 줄을 덮어쓰지 않는다.
//! 기존 정상 논문 대조군의 전 7쪽 Native/fresh WASM과 독립 2020 Print를 대조한다.
#![cfg(not(target_arch = "wasm32"))]
use serde_json::Value;
use std::{path::Path, process::Command};

fn walk<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
    out.push(node);
    if let Some(children) = node["children"].as_array() {
        for child in children {
            walk(child, out);
        }
    }
}
fn text(node: &Value) -> String {
    node["children"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|n| n["text"].as_str())
        .collect()
}
fn top(node: &Value) -> f64 {
    node["bbox"]["y"].as_f64().unwrap()
}
fn bottom(node: &Value) -> f64 {
    top(node) + node["bbox"]["h"].as_f64().unwrap()
}

#[test]
fn horizontal_tac_pen_preserves_the_independent_caption_row() {
    struct Directory(std::path::PathBuf);
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Directory(
        std::env::temp_dir().join(format!("rhwp-tac-vertical-{}-{stamp}", std::process::id())),
    );
    std::fs::create_dir(&dir.0).unwrap();
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(bin)
        .arg("export-render-tree")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/treatise sample.hwp"))
        .args(["--compat", "2022", "-o"])
        .arg(&dir.0)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut files: Vec<_> = std::fs::read_dir(&dir.0)
        .unwrap()
        .map(|f| f.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    assert_eq!(files.len(), 7, "동일 원본의 독립 Print는 7쪽이다");
    let mut found = Vec::new();
    for (page, file) in files.iter().enumerate() {
        let tree: Value = serde_json::from_slice(&std::fs::read(file).unwrap()).unwrap();
        let mut nodes = Vec::new();
        walk(&tree, &mut nodes);
        let captions: Vec<_> = nodes
            .iter()
            .copied()
            .filter(|n| {
                n["type"] == "TextLine"
                    && text(n).split_whitespace().collect::<String>() == "표1.CGA생성시간"
            })
            .collect();
        for caption in captions {
            found.push(page);
            let preceding: Vec<_> = nodes
                .iter()
                .copied()
                .filter(|n| n["type"] == "TextLine" && text(n).trim() == "타낸다.")
                .collect();
            let tables: Vec<_> = nodes
                .iter()
                .copied()
                .filter(|n| n["type"] == "Table" && n["pi"] == 60 && n["ci"] == 0)
                .collect();
            assert_eq!(preceding.len(), 1, "바로 앞 본문 끝을 누락·재생하지 않는다");
            assert_eq!(tables.len(), 1, "원본 표의 소유와 개수를 보존한다");
            assert!(
                bottom(preceding[0]) < top(caption) && bottom(caption) < top(tables[0]),
                "본문 끝, 위 캡션, 소유 표의 물리 순서를 보존한다"
            );
            // 독립 Print 5쪽의 한글 캡션 잉크 상단 379.2364196777344pt.
            // 전체 7쪽이 양쪽 90% 이상이고 이 쪽이 양쪽 98% 이상인 후에 추가한다.
            let expected_top = 379.2364196777344 * 96.0 / 72.0;
            assert!(
                (top(caption) - expected_top).abs() <= 2.0,
                "독립 캡션 상단 ±2 CSSpx: actual={}, expected={expected_top}",
                top(caption)
            );
        }
    }
    assert_eq!(
        found,
        vec![4],
        "해당 캡션은 독립 Print의 5쪽에 한 번 속한다"
    );
}
