//! 검사 유형: 고정형 — 각 함수가 독립 한컴2022 Print의 표 위·아래를 ±2 CSSpx로 고정한다.
//! 69쪽 함수는 별도로 관계형 조건(독립 빈 줄의 소유·유일성·다음 글줄과의 순서)도 검사한다.
//! 주석 갱신: 2026-11-14T21:50:49+09:00.
//! hwpctl_API_v2.4.hwp 전105쪽 Native/fresh WASM 최저99.55%, 대상13·65·69쪽은
//! 두 경로 모두100.00%다(closing-span-clean 생산자). 같은 쪽 review·overlay를 판독했다.
//! 기준 PDF SHA256: 1d289727dd40ed35e48135bf16df06fe4cd080d967441ff464fb0e0b205fae74.
//! PDF 파일명의2020과 실제 Creator Hwp2022를 구분한다. 기대값은 PDF 도형 경계(pt×96/72)다.
//! 원본 글꼴의 설치나 잉크 폭 대신 저장 표의 물리 점유를 검사하며, 최소 글꼴에서도 동일해야 한다.

use serde_json::Value;
use std::{path::PathBuf, process::Command};

struct TemporaryTree(PathBuf);

impl TemporaryTree {
    fn new(page: usize) -> Self {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "rhwp-7731-frame-{}-{page}-{suffix}",
            std::process::id()
        ));
        // 기존 디렉터리를 재사용하지 않아 이 검사 소유의 경로만 정리한다.
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TemporaryTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn table_frame(page: usize, paragraph: u64) -> (Value, f64, f64) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let temporary = TemporaryTree::new(page);
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(binary)
        .arg("export-render-tree")
        .arg(root.join("mydocs/pr/assets/issue7445/hwpctl_API_v2.4.hwp"))
        .args(["-p", &(page - 1).to_string(), "--compat", "2022", "-o"])
        .arg(temporary.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "트리 산출 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files: Vec<_> = std::fs::read_dir(temporary.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    assert_eq!(files.len(), 1, "선택한 한 쪽의 트리만 생성해야 한다");
    let tree: Value = serde_json::from_slice(&std::fs::read(&files[0]).unwrap()).unwrap();
    fn find(node: &Value, paragraph: u64, out: &mut Vec<(f64, f64)>) {
        if node["type"] == "Table" && node["pi"].as_u64() == Some(paragraph) {
            let b = &node["bbox"];
            let top = b["y"].as_f64().unwrap();
            out.push((top, top + b["h"].as_f64().unwrap()));
        }
        if let Some(children) = node["children"].as_array() {
            for child in children {
                find(child, paragraph, out)
            }
        }
    }
    let mut matches = Vec::new();
    find(&tree, paragraph, &mut matches);
    assert_eq!(matches.len(), 1, "{page}쪽 문단{paragraph} 표의 누락·중복");
    let (top, bottom) = matches[0];
    (tree, top, bottom)
}

fn check(page: usize, paragraph: u64, top_pt: f64, bottom_pt: f64) -> Value {
    let (tree, top, bottom) = table_frame(page, paragraph);
    for (edge, actual, point) in [("위", top, top_pt), ("아래", bottom, bottom_pt)] {
        let expected = point * 96.0 / 72.0;
        assert!(
            (actual - expected).abs() <= 2.0,
            "{page}쪽 문단{paragraph} 표 {edge} 경계: {actual} / 독립 Print {expected}"
        );
    }
    tree
}

#[test]
fn first_continuation_table_keeps_its_independent_print_frame() {
    check(13, 176, 102.00799560546875, 130.7769775390625);
}
#[test]
fn empty_square_host_uses_its_source_opening_and_closing_frames() {
    check(65, 1578, 440.87799072265625, 668.509033203125);
}
#[test]
fn independent_tac_tail_preserves_the_following_table_frame() {
    let tree = check(69, 1658, 564.822021484375, 721.490966796875);
    fn lines(node: &Value, paragraph: u64, out: &mut Vec<(f64, f64, String)>) {
        if node["type"] == "TextLine" {
            let children = node["children"].as_array().unwrap();
            if children
                .iter()
                .any(|child| child["pi"].as_u64() == Some(paragraph))
            {
                let text: String = children
                    .iter()
                    .filter_map(|child| child["text"].as_str())
                    .collect();
                out.push((
                    node["bbox"]["y"].as_f64().unwrap(),
                    node["bbox"]["h"].as_f64().unwrap(),
                    text,
                ));
            }
        }
        if let Some(children) = node["children"].as_array() {
            for child in children {
                lines(child, paragraph, out)
            }
        }
    }
    let mut blank = Vec::new();
    lines(&tree, 1644, &mut blank);
    let blank: Vec<_> = blank
        .iter()
        .filter(|(_, _, text)| text.chars().all(char::is_whitespace))
        .collect();
    assert_eq!(
        blank.len(),
        1,
        "표 host 뒤 독립 빈 글줄이 같은 쪽에 한 번 있어야 한다"
    );
    let mut next = Vec::new();
    lines(&tree, 1645, &mut next);
    assert_eq!(next.len(), 1, "후속 reverse 설명 글줄의 누락·중복");
    assert!(next[0].2.starts_with("reverse"));
    assert!(
        next[0].0 >= blank[0].0 + blank[0].1,
        "후속 설명이 독립 빈 줄의 공간을 삼키면 안 된다"
    );
}
