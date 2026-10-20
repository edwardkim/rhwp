//! Issue #2158 — HWPX lineseg 재계산이 저장 쪽-상대 vpos 리셋(쪽나눔 인코딩)을
//! 파괴하던 회귀를 막는 페이지 수 핀.
//!
//! HWPX 원본 XML은 HWP5와 동일하게 쪽-상대 vertpos를 저장하며, 직전 문단 대비
//! 급감(예: sample16 pi87 70412 → pi88 568=sb)이 쪽나눔을 인코딩한다. HWPX 로딩은
//! lineSegArray 부재 문단이 있으면 구역 전체 vpos를 재계산하는데, 이때 원본
//! lineseg 문단의 리셋 신호가 누적 좌표로 덮여(568→208008) typeset의 vpos-reset
//! 쪽나눔(#321/#1921)이 무력화 — 동일 문서가 HWP5로는 64쪽(한글 정합), HWPX로는
//! 63쪽이 되던 결함.
//!
//! 과거 COM 쪽수와 달리 같은 원본의 독립 전체 Print는46쪽이다.
//! stage2869 전쪽 Native/fresh WASM 검증과 stage2903 직접 판독을 근거로 한다.

use std::fs;
use std::path::Path;

fn load(rel: &str) -> rhwp::document_core::DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    rhwp::document_core::DocumentCore::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e))
}

#[test]
fn onsaemiro_hwpx_page_count_matches_hangul() {
    let doc = load("samples/[2027] 온새미로 1 본교재.hwpx");
    assert_eq!(
        doc.page_count(),
        46,
        "같은 원본의 독립 전체 Print46쪽. 과거47쪽 핀으로 정상 출력을 회귀로 판정하지 않는다."
    );
    let layout: serde_json::Value =
        serde_json::from_str(&doc.get_page_text_layout_native(5).expect("독립 Print6쪽")).unwrap();
    let runs = layout["runs"].as_array().unwrap();
    let original = &doc.document().sections[1].paragraphs[32];
    assert!(original
        .text
        .starts_with("수많은 SF 영화나 소설이 유토피아"));
    let mut next = 0;
    let mut text = String::new();
    for run in runs
        .iter()
        .filter(|run| run["secIdx"] == 1 && run["paraIdx"] == 32)
    {
        assert_eq!(run["charStart"].as_u64(), Some(next), "지문 순서·누락·중복");
        let value = run["text"].as_str().unwrap();
        text.push_str(value);
        next += value.encode_utf16().count() as u64;
    }
    assert_eq!(
        text, original.text,
        "쪽수와 함께 원문 지문의 쪽 소속·전체 내용 보존"
    );
}
