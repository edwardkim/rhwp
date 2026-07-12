//! Issue #2099: hwpspec 문서의 `글` 아래아 표시 누락.
//!
//! U+F53A는 `samples/hwpspec.hwp`와 `samples/pua-test.pdf`에서 실제 옛한글
//! ``으로 보이는 PUA 코드포인트다. 공개 폰트 환경에서도 아래아가 보이도록
//! 렌더 경로에서는 KS X 1026-1 자모 시퀀스 `ᄒᆞᆫ`으로 확장해야 한다.

use rhwp::renderer::composer::expand_pua_render_text;
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

fn read_sample(rel: &str) -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn render_sample_page(rel: &str, page: u32) -> String {
    let bytes = read_sample(rel);
    let doc = HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"));
    doc.render_page_svg_native(page)
        .unwrap_or_else(|e| panic!("render {rel} page {page}: {e}"))
}

#[test]
fn issue_2099_f53a_expands_to_old_hangul_han() {
    assert_eq!(
        expand_pua_render_text("\u{F53A}글"),
        "\u{1112}\u{119E}\u{11AB}글",
        "U+F53A는 렌더 경로에서 `ᄒᆞᆫ` 자모 시퀀스로 확장되어야 함",
    );
}

#[test]
fn issue_2099_hwpspec_svg_does_not_emit_raw_f53a() {
    let svg = render_sample_page("samples/hwpspec.hwp", 0);
    assert!(
        !svg.contains('\u{F53A}'),
        "hwpspec SVG에 raw U+F53A가 남으면 공개 폰트 환경에서 아래아가 깨질 수 있음",
    );
    assert!(
        svg.contains("\u{1112}\u{119E}\u{11AB}"),
        "hwpspec SVG는 U+F53A를 `ᄒᆞᆫ`으로 확장해야 함",
    );
}

#[test]
fn issue_2099_pua_test_svg_matches_visible_f53a_baseline() {
    let svg = render_sample_page("samples/pua-test.hwp", 0);
    assert!(
        !svg.contains('\u{F53A}'),
        "pua-test SVG에도 raw U+F53A를 그대로 출력하면 안 됨",
    );
    assert!(
        svg.contains("\u{1112}\u{119E}\u{11AB}"),
        "pua-test의 U+F53A 기준 글리프도 `ᄒᆞᆫ`으로 확장되어야 함",
    );
}

#[test]
fn issue_2099_revision13_page1_prioritizes_old_hangul_font() {
    let svg = render_sample_page("samples/한글문서파일형식_5.0_revision1.3.hwp", 0);
    assert!(
        !svg.contains('\u{F53A}'),
        "사용자 재현 68쪽 배포본 SVG에도 raw U+F53A가 남으면 안 됨",
    );
    let needle = "\u{1112}\u{119E}\u{11AB}</text>";
    let text_end = svg
        .find(needle)
        .expect("사용자 재현 68쪽 배포본 1쪽 제목에 `ᄒᆞᆫ` 클러스터가 있어야 함");
    let text_start = svg[..text_end]
        .rfind("<text ")
        .expect("`ᄒᆞᆫ` 클러스터는 독립 SVG text node로 출력되어야 함");
    let text_node = &svg[text_start..text_end];
    assert!(
        text_node.contains("font-family=\"&apos;Source Han Serif K Old Hangul&apos;,"),
        "`ᄒᆞᆫ` 클러스터는 일반 고딕 폰트보다 옛한글 전용 폰트를 우선해야 함: {text_node}",
    );
}
