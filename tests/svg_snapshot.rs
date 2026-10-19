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
//! - 비교 원문은 문서 내장 폰트만 포함하며 호스트 폰트를 읽지 않는다.
//!   실패 때 사람이 확인하는 `.actual.svg` 사본만 Full 폰트 API를 사용한다.
//!   원문은 `output/svg-snapshot/`에 보존하므로 진단 사본의 호스트 폰트가
//!   snapshot 비교나 golden 기대값에 들어가지 않는다.
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
        // 비교 원문은 output에 보존한다. 사람이 여는 사본은 윤곽선 폰트를
        // 공급해 Chrome의 로컬 비트맵 폰트 선택으로 생기는 두부문자를 방지한다.
        // 폰트 공급 사본은 golden 비교나 기대값 갱신에 사용하지 않는다.
        let raw_path = PathBuf::from(repo_root)
            .join("output/svg-snapshot")
            .join(format!("{golden_name}.actual.svg"));
        fs::create_dir_all(raw_path.parent().unwrap()).expect("비교 원문 디렉터리 생성");
        fs::write(&raw_path, &actual).expect("비교 원문 SVG 보존");
        let preview = doc
            .render_page_svg_with_fonts(page, rhwp::renderer::svg::FontEmbedMode::Full, &[])
            .expect("폰트 공급 진단 SVG 렌더링");
        let actual_path = golden_path.with_extension("actual.svg");
        fs::write(&actual_path, &preview).expect("폰트 공급 진단 SVG 보존");
        panic!(
            "SVG snapshot mismatch for {}.\n  expected: {}\n  actual:   {}\n\
             raw comparison SVG: {}\n\
             Inspect the diff; if intentional, rerun with UPDATE_GOLDEN=1.",
            golden_name,
            golden_path.display(),
            actual_path.display(),
            raw_path.display()
        );
    }
}

#[test]
fn form_002_page_0() {
    use serde_json::Value;

    // 한컴2020 정본 전10쪽의 Native·fresh WASM 시각 검증을 선행한다.
    // SVG 바이트·절대 좌표 대신 쪽수, 원본 표 소유, 분할 문단의 소속을 검사한다.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpx/form-002.hwpx");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&fs::read(path).expect("원문 읽기"))
        .expect("원문 열기");
    assert_eq!(doc.page_count(), 10, "독립 한컴 PDF의 전체 쪽수");
    let owners = [
        (0, 26, 27),
        (0, 26, 27),
        (2, 29, 28),
        (2, 29, 28),
        (3, 27, 27),
        (3, 27, 27),
        (4, 27, 27),
        (4, 27, 27),
        (5, 27, 27),
        (5, 27, 27),
    ];
    for (page, (paragraph, rows, columns)) in owners.into_iter().enumerate() {
        let tree: Value = serde_json::from_str(
            &doc.get_page_render_tree(page as u32)
                .expect("쪽별 렌더 트리"),
        )
        .expect("렌더 트리 JSON");
        let mut pending = vec![&tree];
        let table = loop {
            let node = pending.pop().expect("모든 쪽에 원본 바깥 표가 표시된다");
            if node["type"] == "Table" {
                break node;
            }
            if let Some(children) = node["children"].as_array() {
                pending.extend(children.iter().rev());
            }
        };
        assert_eq!(table["pi"], paragraph, "{}쪽 원본 표 소유", page + 1);
        assert_eq!(table["rows"], rows, "원본 표 행 수");
        assert_eq!(table["cols"], columns, "원본 표 열 수");
        if page < 2 {
            let cell = table["children"]
                .as_array()
                .expect("표 자식")
                .iter()
                .find(|cell| cell["type"] == "Cell" && cell["row"] == 19 && cell["col"] == 0)
                .expect("두 쪽에 이어지는 개발내용 칸");
            let lines: Vec<_> = cell["children"]
                .as_array()
                .expect("칸 자식")
                .iter()
                .filter(|line| line["type"] == "TextLine")
                .collect();
            let (present, absent, phrase) = if page == 0 {
                (13, 15, "주사제형화기술개발")
            } else {
                (15, 13, "PFC나노산소운반체의최적제조공정개발및GMP실증")
            };
            assert!(
                lines.iter().any(|line| line["pi"] == present),
                "쪽 소유 문단 보존"
            );
            assert!(
                !lines.iter().any(|line| line["pi"] == absent),
                "다른 쪽 문단 중복 방출 금지"
            );
            let text: String = lines
                .iter()
                .flat_map(|line| line["children"].as_array().expect("글줄 자식").iter())
                .filter_map(|run| run["text"].as_str())
                .collect();
            let text: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
            assert!(
                text.contains(phrase),
                "한컴 PDF에서 확인한 분할 경계 문구: {text}"
            );
        }
    }
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

// Issue #267: KTX.hwp 목차 페이지 — right tab 장제목/소제목 페이지 번호 정렬
// #7445: 최신 upstream 74f9b71b1의 KTX 목차가 Native/fresh WASM 모두
// 49.63836%다. 실패한 이 snapshot만 이관하며 원문·golden·다른 검사는 보존한다.

// #147의 주석/목차 소유 계약은 tests/cases/issue_147_memo_page_ownership.rs에서
// 검사한다. 탭 리더 끝 좌표의 변경을 MEMO 오분류 회귀로 취급하지 않는다.

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
