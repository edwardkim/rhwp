//! SVG snapshot regression tests for HWPX rendering.
//!
//! Pure-Rust replacement for `tools/verify_hwpx.py`, which requires Windows
//! + Hancom Office + pyhwpx and cannot run in CI. This harness invokes
//!   `rhwp::wasm_api::HwpDocument::render_page_svg_native()` directly so the
//!   same SVG the CLI produces is diffed against committed golden files.
//!
//! # Updating goldens
//!
//! When rhwp's rendering intentionally changes, regenerate goldens:
//!
//! ```sh
//! UPDATE_GOLDEN=1 cargo test --test svg_snapshot
//! ```
//!
//! Commit the resulting `tests/golden_svg/**/*.svg` files alongside the
//! source change and mention the intentional diff in the PR body.
//!
//! # Determinism
//!
//! These tests assume:
//! - `render_page_svg_native` output is deterministic for a fixed input
//!   (no timestamps, no random IDs, no host-font-dependent glyph IDs).
//! - The native entry point embeds only document-owned font bytes and emits
//!   their `@font-face` rules in sorted family-name order. Host font files are
//!   never read by this path, so host state cannot leak into the snapshot.
//!
//! If a flake is observed, the first debugging step is to diff two
//! back-to-back runs on the same machine. Host-specific variance
//! indicates a real determinism bug — worth its own issue.

use std::fs;
use std::path::{Path, PathBuf};

/// Generate an SVG for a specific page and compare against the committed
/// golden. Set `UPDATE_GOLDEN=1` to regenerate.
fn check_snapshot(hwpx_relpath: &str, page: u32, golden_name: &str) {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwpx_path = Path::new(repo_root).join(hwpx_relpath);
    let bytes =
        fs::read(&hwpx_path).unwrap_or_else(|e| panic!("read {}: {}", hwpx_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", hwpx_relpath, e));

    let actual = doc
        .render_page_svg_native(page)
        .unwrap_or_else(|e| panic!("render {} p.{}: {}", hwpx_relpath, page, e));

    let golden_path = PathBuf::from(repo_root)
        .join("tests/golden_svg")
        .join(format!("{golden_name}.svg"));

    if std::env::var("UPDATE_GOLDEN").as_deref() == Ok("1") {
        if let Some(parent) = golden_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&golden_path, &actual).unwrap();
        eprintln!("UPDATED {}", golden_path.display());
        return;
    }

    let expected = fs::read_to_string(&golden_path).unwrap_or_else(|e| {
        panic!(
            "missing golden {}: {}. Run `UPDATE_GOLDEN=1 cargo test --test svg_snapshot` to create.",
            golden_path.display(),
            e
        )
    });

    if actual != expected {
        // Write the actual output next to the golden for local inspection
        // without polluting the committed tree.
        let actual_path = golden_path.with_extension("actual.svg");
        let _ = fs::write(&actual_path, &actual);
        panic!(
            "SVG snapshot mismatch for {}.\n  expected: {}\n  actual:   {}\n\
             Inspect the diff; if intentional, rerun with UPDATE_GOLDEN=1.",
            golden_name,
            golden_path.display(),
            actual_path.display()
        );
    }
}

#[test]
fn form_002_page_0() {
    // [Task #993] 골든 갱신 — 컷 모델이 분할 표의 큰 셀을 vpos 리셋(429.3px)에서
    // 분할. 한컴 2022 PDF(pdf/hwpx/form-002-2022.pdf) 대조 결과 분할 콘텐츠
    // 경계가 일치(페이지 1 끝 "…주사제형화 기술 개발", 페이지 2 시작
    // "ㅇ PFC 나노산소운반체…"). 기존 px 모델은 분할 셀 박스를 콘텐츠보다
    // 17.5px 길게(페이지 하단까지) 그렸으나 한컴은 콘텐츠 끝까지만 그린다.
    check_snapshot("samples/hwpx/form-002.hwpx", 0, "form-002/page-0");
}

#[test]
fn table_text_page_0() {
    check_snapshot("samples/hwpx/table-text.hwpx", 0, "table-text/page-0");
}

/// Issue #157: 비-TAC wrap=위아래 표 out-of-flow 배치 — 표가 텍스트와 중첩되지 않음
#[test]
fn issue_157_page_1() {
    use serde_json::Value;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpx/issue_157.hwpx");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&fs::read(path).expect("원문 읽기"))
        .expect("원문 열기");
    assert_eq!(doc.page_count(), 2, "독립 한컴 PDF의 전체 쪽수");
    let tree: Value = serde_json::from_str(&doc.get_page_render_tree(1).expect("2쪽 렌더 트리"))
        .expect("렌더 트리 JSON");

    fn collect<'a>(value: &'a Value, output: &mut Vec<&'a Value>) {
        output.push(value);
        if let Some(children) = value["children"].as_array() {
            for child in children {
                collect(child, output);
            }
        }
    }
    let mut nodes = Vec::new();
    collect(&tree, &mut nodes);
    let table = |pi| {
        nodes
            .iter()
            .copied()
            .find(|node| node["type"] == "Table" && node["pi"] == pi)
            .unwrap_or_else(|| panic!("2쪽 원문 표 문단 {pi} 누락"))
    };
    let text = |phrase: &str| {
        nodes
            .iter()
            .copied()
            .find(|node| {
                node["type"] == "TextRun"
                    && node["text"]
                        .as_str()
                        .is_some_and(|content| content.contains(phrase))
            })
            .unwrap_or_else(|| panic!("2쪽 문장 {phrase} 누락"))
    };
    let top = |node: &Value| node["bbox"]["y"].as_f64().expect("상단");
    let bottom = |node: &Value| top(node) + node["bbox"]["h"].as_f64().expect("높이");

    let attendance = table(7);
    assert!(
        bottom(text("바랍니다.)")) <= top(attendance),
        "참석장 표가 앞 문장을 덮으면 안 된다"
    );
    assert!(
        bottom(attendance) <= top(text("(대리참석 위임)")),
        "참석장 표가 뒤 문장을 덮으면 안 된다"
    );

    let delegation = table(25);
    assert!(
        bottom(text("기타 정기주주총회 참석")) <= top(delegation),
        "위임인 표가 앞 문장을 덮으면 안 된다"
    );
    assert!(
        bottom(delegation) <= top(text("2026")),
        "위임인 표가 뒤 문장을 덮으면 안 된다"
    );
}

/// Issue #267: KTX.hwp 목차 페이지 — right tab 장제목/소제목 페이지 번호 정렬
#[test]
fn issue_267_ktx_toc_page() {
    check_snapshot("samples/KTX.hwp", 1, "issue-267/ktx-toc-page");
}

/// Issue #147: aift.hwp 4페이지 — MEMO 컨트롤이 바탕쪽으로 오분류되어 렌더링되는 버그
#[test]
fn issue_147_aift_page3() {
    check_snapshot("samples/aift.hwp", 3, "issue-147/aift-page3");
}

/// Issue #617: 시험지 보기 셀의 여백과 17쪽 표시 상자의 저장 줄 위치를 검증한다.
#[test]
fn issue_617_exam_kor_page5() {
    use serde_json::Value;

    fn collect<'a>(value: &'a Value, output: &mut Vec<&'a Value>) {
        output.push(value);
        if let Some(children) = value["children"].as_array() {
            for child in children {
                collect(child, output);
            }
        }
    }

    fn has_text(value: &Value, expected: &str) -> bool {
        value["text"]
            .as_str()
            .is_some_and(|text| text.contains(expected))
            || value["children"]
                .as_array()
                .is_some_and(|children| children.iter().any(|child| has_text(child, expected)))
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/exam_kor.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&fs::read(path).expect("원문 읽기"))
        .expect("원문 열기");
    assert_eq!(doc.page_count(), 20, "독립 한컴 PDF의 전체 쪽수");

    let page6: Value = serde_json::from_str(&doc.get_page_render_tree(5).expect("6쪽 렌더 트리"))
        .expect("6쪽 JSON");
    let mut nodes6 = Vec::new();
    collect(&page6, &mut nodes6);
    let body_cell = nodes6
        .iter()
        .find(|node| {
            node["type"] == "Cell"
                && node["row"] == 2
                && node["col"] == 0
                && has_text(node, "사용한다.")
        })
        .expect("16번 보기 본문 셀");
    let mut cell_nodes = Vec::new();
    collect(body_cell, &mut cell_nodes);
    let line = cell_nodes
        .iter()
        .find(|node| node["type"] == "TextLine" && has_text(node, "사용한다."))
        .expect("보기 본문 둘째 줄");
    let left = line["bbox"]["x"].as_f64().unwrap() - body_cell["bbox"]["x"].as_f64().unwrap();
    let right = body_cell["bbox"]["x"].as_f64().unwrap() + body_cell["bbox"]["w"].as_f64().unwrap()
        - line["bbox"]["x"].as_f64().unwrap()
        - line["bbox"]["w"].as_f64().unwrap();
    let cell_width = body_cell["bbox"]["w"].as_f64().unwrap();
    assert!(
        left > 0.0 && right > 0.0 && left / cell_width > 0.01 && right / cell_width > 0.01,
        "보기 문단이 셀의 내부 여백에 있어야 함: 좌우 비율 {:.3}/{:.3}",
        left / cell_width,
        right / cell_width
    );

    let page17: Value =
        serde_json::from_str(&doc.get_page_render_tree(16).expect("17쪽 렌더 트리"))
            .expect("17쪽 JSON");
    let mut nodes17 = Vec::new();
    collect(&page17, &mut nodes17);
    let label_box = nodes17
        .iter()
        .find(|node| node["type"] == "Rect" && has_text(node, "홀수형"))
        .expect("홀수형 사각형");
    let mut box_nodes = Vec::new();
    collect(label_box, &mut box_nodes);
    let label = box_nodes
        .iter()
        .find(|node| node["type"] == "TextRun" && node["text"] == "홀수형")
        .expect("홀수형 글줄");
    let top_gap = label["bbox"]["y"].as_f64().unwrap() - label_box["bbox"]["y"].as_f64().unwrap();
    let bottom_gap = label_box["bbox"]["y"].as_f64().unwrap()
        + label_box["bbox"]["h"].as_f64().unwrap()
        - label["bbox"]["y"].as_f64().unwrap()
        - label["bbox"]["h"].as_f64().unwrap();
    let box_height = label_box["bbox"]["h"].as_f64().unwrap();
    assert!(
        top_gap > 0.0 && bottom_gap > 0.0 && (top_gap - bottom_gap).abs() / box_height <= 0.05,
        "홀수형 글자가 사각형 중앙에 있어야 함: 위아래 차이 비율 {:.3}",
        (top_gap - bottom_gap).abs() / box_height
    );
}

/// Determinism probe: render the same page twice in one process and assert
/// byte-for-byte equality. If this ever fails, the snapshot tests above
/// are unreliable regardless of golden correctness.
#[test]
fn render_is_deterministic_within_process() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let bytes =
        fs::read(Path::new(repo_root).join("samples/hwpx/form-002.hwpx")).expect("sample present");

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    let a = doc.render_page_svg_native(0).expect("render #1");
    let b = doc.render_page_svg_native(0).expect("render #2");
    assert_eq!(a, b, "render_page_svg_native must be deterministic");
}
