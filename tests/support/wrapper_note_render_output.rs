// 실제 Native 인쇄 경로의 SVG를 검사하고 검사 전용 임시 출력만 정리한다.

use serde_json::Value;
use std::{fs, path::Path, process::Command};

struct OutputDir(std::path::PathBuf);

impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn pages(input: &str) -> Vec<String> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("시각")
        .as_nanos();
    let output = OutputDir(std::env::temp_dir().join(format!(
        "rhwp-wrapper-note-relations-{}-{nonce}",
        std::process::id()
    )));
    fs::create_dir(&output.0).expect("검사 전용 출력 디렉터리");
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(binary)
        .arg("export-svg")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join(input))
        .args(["--profile", "print", "--compat", "2022", "--json", "-o"])
        .arg(&output.0)
        .output()
        .expect("실제 Native 인쇄 출력");
    assert!(
        result.status.success(),
        "{input}: 인쇄 출력 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let manifest: Value = serde_json::from_slice(&result.stdout).expect("실제 출력 목록");
    let pages = manifest["pages"].as_array().expect("전체 쪽의 출력 목록");
    assert_eq!(
        manifest["pageCount"].as_u64().unwrap() as usize,
        pages.len()
    );
    assert_eq!(
        manifest["renderedCount"].as_u64().unwrap() as usize,
        pages.len()
    );
    pages
        .iter()
        .map(|page| fs::read_to_string(page["path"].as_str().unwrap()).expect("목록의 실제 SVG"))
        .collect()
}

pub fn text(document: &roxmltree::Document<'_>) -> String {
    document
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .flat_map(|node| {
            node.descendants()
                .filter(|child| child.is_text())
                .filter_map(|child| child.text())
        })
        .collect::<String>()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}
