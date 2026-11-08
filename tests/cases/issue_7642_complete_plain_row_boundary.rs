//! 완결 줄의 정확한 예산 경계와 1 HWPUNIT 여유를 독립 Print로 구분한다.
//! 두 입력 모두 전쪽 Native/fresh WASM raw 100%다. 출처와 ±2px 선행 증거는
//! tests/fixtures/issue7642_complete_plain_row_boundary/README.md를 따른다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const FIXTURES: &str = "tests/fixtures/issue7642_complete_plain_row_boundary";

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rhwp-7642-complete-row-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn pages(input: &Path) -> Vec<Value> {
    let output = TempDir::new();
    // 수정 전/후의 불변 CLI 실행 파일에도 같은 소유·포함 검사를 적용한다.
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(binary)
        .arg("export-render-tree")
        .arg(input)
        .args(["--compat", "2022", "-o"])
        .arg(&output.0)
        .output()
        .expect("최종 렌더 트리 내보내기");
    assert!(result.status.success(), "{result:?}");
    let mut files: Vec<_> = std::fs::read_dir(&output.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap())
        .collect()
}

fn edge(node: &Value, name: &str) -> f64 {
    node["bbox"][name].as_f64().unwrap()
}

fn contained(inner: &Value, outer: &Value) {
    // CLI JSON의 좌표는 소수 첫째 자리로 직렬화된다. 두 경계의 반올림만 허용한다.
    const ROUNDING: f64 = 0.11;
    for (start, size) in [("x", "w"), ("y", "h")] {
        assert!(edge(inner, start) + ROUNDING >= edge(outer, start));
        assert!(
            edge(inner, start) + edge(inner, size)
                <= edge(outer, start) + edge(outer, size) + ROUNDING,
            "내용은 소속 셀/본문 안에 있어야 한다: {inner:?}, {outer:?}"
        );
    }
}

fn collect(
    node: &Value,
    page: usize,
    body: Option<&Value>,
    cell: Option<&Value>,
    found: &mut Vec<(usize, usize, usize)>,
    followers: &mut Vec<(usize, f64)>,
) {
    let body = if node["type"] == "Body" {
        Some(node)
    } else {
        body
    };
    let cell = if node["type"] == "Cell" {
        contained(node, body.expect("셀의 본문 소유"));
        Some(node)
    } else {
        cell
    };
    if node["type"] == "TextLine" {
        if let Some(cell) = cell {
            contained(node, cell);
        }
    }
    if node["type"] == "TextRun" {
        if node["text"] == "AFTER TABLE FOLLOWER" {
            assert!(cell.is_none(), "표 뒤 본문을 셀 내용으로 흡수하지 않는다");
            contained(node, body.expect("후속 본문의 소유"));
            followers.push((page, edge(node, "y")));
        } else {
            let cell = cell.expect("행 표식은 표 셀에 속한다");
            contained(node, cell);
            let row = cell["row"].as_u64().unwrap() as usize;
            let col = cell["col"].as_u64().unwrap() as usize;
            assert_eq!(node["text"], format!("ROW {row} CELL {col}"));
            found.push((row, col, page));
        }
    }
    if let Some(children) = node["children"].as_array() {
        for child in children {
            collect(child, page, body, cell, found, followers);
        }
    }
}

fn target_baseline(node: &RenderNode) -> Option<(f64, f64)> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.display_or_text() == "ROW 4 CELL 0" {
            return Some((node.bbox.x, node.bbox.y + run.baseline));
        }
    }
    node.children.iter().find_map(target_baseline)
}

fn table_bottom(node: &Value) -> Option<f64> {
    let own = (node["type"] == "Table").then(|| edge(node, "y") + edge(node, "h"));
    node["children"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(table_bottom)
        .chain(own)
        .reduce(f64::max)
}

#[test]
fn complete_plain_row_keeps_print_ownership_at_exact_and_one_unit_budgets() {
    // 원본을 바꾸지 않고 Print PDF의 전쪽 텍스트·글자 행렬에서 읽은 기대값이다.
    for (stem, row_pages, baseline) in [
        ("minimum-28500", [1, 2, 3, 4, 6, 7], 51.339_029_947_916_664),
        ("headroom-1", [1, 2, 3, 4, 5, 7], 344.499_267_578_125),
    ] {
        let input = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(FIXTURES)
            .join(format!("{stem}.hwpx"));
        let rendered = pages(&input);
        assert_eq!(rendered.len(), 8, "{stem}: 독립 Print 전쪽 수");
        let mut found = Vec::new();
        let mut followers = Vec::new();
        for (page, tree) in rendered.iter().enumerate() {
            collect(tree, page + 1, None, None, &mut found, &mut followers);
        }
        let expected: Vec<_> = row_pages
            .iter()
            .enumerate()
            .flat_map(|(row, page)| [(row, 0, *page), (row, 1, *page)])
            .collect();
        assert_eq!(
            found, expected,
            "{stem}: 내용 순서·셀 소유·누락·중복·쪽 소유"
        );
        assert_eq!(followers.len(), 1, "후속 본문의 누락·중복 방지");
        assert_eq!(followers[0].0, 8, "독립 Print의 후속 본문 쪽 소유");
        assert!(found.iter().all(|(_, _, page)| *page < followers[0].0));
        assert!(
            table_bottom(&rendered[followers[0].0 - 1]).expect("표의 끝 조각")
                <= followers[0].1 + 0.11,
            "표의 끝 조각과 후속 본문이 겹치지 않는다"
        );

        // 완료 페이지 두 backend raw 100% 및 확대 판독 후의 중요한 시작 기준선.
        // DPI=96, CSS px, 페이지 왼쪽 위 원점. 원본 계약의 관계 검사도 위에서 유지한다.
        let core = DocumentCore::from_bytes(&std::fs::read(&input).unwrap()).unwrap();
        let tree = core
            .build_page_render_tree(row_pages[4] as u32 - 1)
            .unwrap();
        let (x, y) = target_baseline(&tree.root).expect("다섯째 행의 첫 셀 글자");
        assert!((x - 66.683_858_235_677_08).abs() <= 2.0, "{stem}: x={x}");
        assert!(
            (y - baseline).abs() <= 2.0,
            "{stem}: 기준선={y}, PDF={baseline}"
        );
    }
}
