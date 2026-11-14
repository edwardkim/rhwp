//! 관계형 — [#8973] 재조판된 빈 블록 표 host의 비양수 오프셋은 글줄을 중복 가산하지 않는다.
//! 주석 갱신: 2026-11-14 13:38:40 KST.
//!
//! 원본: samples/issue8969/36400485-owned-tac-lines.hwpx, 표 pi=54의 23행 2열.
//! 독립 Print: pdf/issue8969/36400485-2020.pdf(한컴 2020, 11쪽).
//! Native/fresh WASM(96 DPI·compat2022) 전11쪽 최저 raw 각각95.64%, 검사 대상10쪽 각각100.00%.
//! 측정 producer: 613d992a61cd + 공통 host 합집합 보정, source SHA 1c2fec4b97a0ed065714c5416e863dc94f3c6aa1a219ec89ea593bd20b4fc8bc.
//! 증거: mydocs/pr/assets/planet6897_integration_20261114/row8973/.
//! 고정 px 좌표를 사용하지 않는다. 원본의 음수 오프셋 하나만 0으로 바꾼 대조군과
//! 실제 CLI가 그린 행·뒤 행의 상자, 내용 순서·포함 관계를 비교한다.
//! 운영체제와 설치 글꼴에 의존해 건너뛰지 않으며 최소 글꼴 환경에서 실행한다.
#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SAMPLE: &str = "samples/issue8969/36400485-owned-tac-lines.hwpx";

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn zero_offset_control(bytes: &[u8]) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(bytes)).expect("원본 HWPX ZIP");
    let mut target = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut changed = false;
    for index in 0..source.len() {
        let mut entry = source.by_index(index).expect("원본 ZIP 항목");
        let name = entry.name().expect("ZIP 항목 이름").into_owned();
        let options =
            zip::write::SimpleFileOptions::default().compression_method(entry.compression());
        if entry.is_dir() {
            target.add_directory(name, options).expect("디렉터리 보존");
            continue;
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("원본 항목 읽기");
        if name == "Contents/section0.xml" {
            let text = String::from_utf8(data).expect("HWPX XML UTF-8");
            let needle = "vertOffset=\"4294961674\"";
            assert_eq!(text.matches(needle).count(), 1, "대조할 음수 오프셋 하나");
            data = text.replacen(needle, "vertOffset=\"0\"", 1).into_bytes();
            changed = true;
        }
        target.start_file(name, options).expect("원본 항목 보존");
        target.write_all(&data).expect("대조 항목 쓰기");
    }
    assert!(changed, "문단 내용·표 속성을 유지하고 오프셋만 변경");
    target.finish().expect("대조 HWPX 마감").into_inner()
}

fn render_tree(input: &Path, output: &Path) -> Value {
    let binary = std::env::var("RHWP_ISSUE8973_TEST_CLI")
        .or_else(|_| std::env::var("CARGO_BIN_EXE_rhwp"))
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    let result = Command::new(&binary)
        .args(["export-render-tree"])
        .arg(input)
        .arg("-o")
        .arg(output)
        .args(["-p", "9", "--compat", "2022"])
        .output()
        .expect("실제 CLI 실행");
    assert!(
        result.status.success(),
        "CLI={binary}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&std::fs::read(output.join("render_tree_010.json")).expect("10쪽 트리"))
        .expect("실제 트리 JSON")
}

fn cell(node: &Value, row: u64, col: u64, in_parent: bool) -> Option<&Value> {
    let in_parent = in_parent || (node["type"] == "Table" && node["pi"] == 54 && node["ci"] == 0);
    if in_parent && node["type"] == "Cell" && node["row"] == row && node["col"] == col {
        return Some(node);
    }
    node["children"]
        .as_array()?
        .iter()
        .find_map(|child| cell(child, row, col, in_parent))
}

fn text(node: &Value) -> String {
    let mut result = node["text"].as_str().unwrap_or("").to_owned();
    if let Some(children) = node["children"].as_array() {
        for child in children {
            result.push_str(&text(child));
        }
    }
    result
}

fn assert_content_inside_row(node: &Value, top: f64, bottom: f64) {
    if node["type"] == "TextLine" || node["type"] == "Table" {
        let y = node["bbox"]["y"].as_f64().expect("내용 위끝");
        let h = node["bbox"]["h"].as_f64().expect("내용 높이");
        // 공개 JSON의 소수 첫째 자리 표시 오차만 허용하는 포함 관계다.
        assert!(
            y + 0.2 >= top && y + h <= bottom + 0.2,
            "내용이 제 행 밖으로 벗어난다"
        );
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            assert_content_inside_row(child, top, bottom);
        }
    }
}

#[test]
fn negative_offset_empty_block_host_keeps_the_same_row_as_zero_offset() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("현재 시각")
        .as_nanos();
    let scratch = Scratch(
        root.join("output")
            .join(format!("issue8973-test-{}-{nonce}", std::process::id())),
    );
    std::fs::create_dir_all(&scratch.0).expect("검사 전용 경로");
    let original = root.join(SAMPLE);
    let bytes = std::fs::read(&original).expect("원본 fixture");
    let control = scratch.0.join("zero-offset.hwpx");
    std::fs::write(&control, zero_offset_control(&bytes)).expect("대조 사본");
    let before = render_tree(&original, &scratch.0.join("negative"));
    let after = render_tree(&control, &scratch.0.join("zero"));
    let negative = cell(&before, 23, 2, false).expect("원본 중첩 표를 품은 칸");
    let zero = cell(&after, 23, 2, false).expect("대조 중첩 표를 품은 칸");
    assert_eq!(
        negative["bbox"], zero["bbox"],
        "같은 흐름 상자에 host 글줄을 별도로 가산하지 않는다"
    );
    assert_eq!(text(negative), text(zero), "본문 순서·내용을 보존한다");
    assert!(text(negative).contains("정보공개심의회"));
    assert!(text(negative).contains("근거자료"));
    let top = negative["bbox"]["y"].as_f64().expect("행 위끝");
    let bottom = top + negative["bbox"]["h"].as_f64().expect("행 높이");
    assert_content_inside_row(negative, top, bottom);
    assert_eq!(
        cell(&before, 24, 2, false).expect("원본 뒤 행")["bbox"],
        cell(&after, 24, 2, false).expect("대조 뒤 행")["bbox"],
        "불필요한 host 높이가 뒤 행을 밀지 않는다"
    );
}
