//! SVG snapshot regression tests for HWPX rendering.
//!
//! Pure-Rust replacement for `tools/verify_hwpx.py`, which requires Windows
//! + Hancom Office + pyhwpx and cannot run in CI. This harness invokes
//! `rhwp::wasm_api::HwpDocument::render_page_svg_native()` directly so the
//! same SVG the CLI produces is diffed against committed golden files.
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
//! - Font embedding is OFF (`FontEmbedMode::None` via the native entry
//!   point) so host system fonts cannot leak into the snapshot.
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
    let bytes = fs::read(&hwpx_path)
        .unwrap_or_else(|e| panic!("read {}: {}", hwpx_path.display(), e));

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
    check_snapshot("samples/hwpx/form-002.hwpx", 0, "form-002/page-0");
}

#[test]
fn table_text_page_0() {
    check_snapshot("samples/hwpx/table-text.hwpx", 0, "table-text/page-0");
}

/// Issue #157: 비-TAC wrap=위아래 표 out-of-flow 배치 — 표가 텍스트와 중첩되지 않음
#[test]
fn issue_157_page_1() {
    check_snapshot("samples/hwpx/issue_157.hwpx", 1, "issue-157/page-1");
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

/// Issue #617: exam_kor.hwp 6페이지 — 16번 보기 박스 셀 padding 이
/// shrink 휴리스틱으로 0까지 깎이는 회귀를 잠가둔다. 다중 줄 셀에서
/// HWP 가 분배한 line_segs 를 신뢰하고 padding 을 보존하는 동작을 검증.
#[test]
fn issue_617_exam_kor_page5() {
    check_snapshot("samples/exam_kor.hwp", 5, "issue-617/exam-kor-page5");
}

/// Issue #677: 복학원서.hwp 1페이지 — 다음 두 결함 영역의 회귀 차단
///   1. PartialParagraph y 누적 결함 (인라인 TAC 표 라인)
///      `layout.rs::PageItem::PartialParagraph` 가 y_offset 을
///      LineSeg.vpos 정합 위치로 리셋하는 동작을 잠가둔다.
///   2. U+F081C HWP PUA 채움 문자 폭 (0)
///      `text_measurement.rs::char_width` 의 5 사이트 모두 채움 문자
///      폭을 0 으로 처리하는 동작을 잠가둔다.
///   3. 한컴 워터마크 모드 표준 프리셋 (brightness=+70, contrast=-50)
///      `svg.rs::render_image` 의 워터마크 게이트 동작을 잠가둔다.
#[test]
fn issue_677_bokhakwonseo_page1() {
    check_snapshot("samples/복학원서.hwp", 0, "issue-677/bokhakwonseo-page1");
}

/// Determinism probe: render the same page twice in one process and assert
/// byte-for-byte equality. If this ever fails, the snapshot tests above
/// are unreliable regardless of golden correctness.
#[test]
fn render_is_deterministic_within_process() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let bytes = fs::read(Path::new(repo_root).join("samples/hwpx/form-002.hwpx"))
        .expect("sample present");

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse");
    let a = doc.render_page_svg_native(0).expect("render #1");
    let b = doc.render_page_svg_native(0).expect("render #2");
    assert_eq!(a, b, "render_page_svg_native must be deterministic");
}
