//! Issue #1624: footer(발신명의) Page+Bottom over-push — #1611 vpos 동기화의 +1쪽 부작용.
//!
//! #1611 은 footer 의 stored vpos 가 흐름 cur_h 보다 위면 `sync_h = max(cur_h, vpos)` 로
//! 끌어올려 page-fit 을 판정했다. 그러나 본문이 짧은데 footer 의 stored vpos 가 page-bottom
//! 앵커/누적 노이즈로 매우 큰(예: 987~1628px, 본문 cur_h 28~602px) 경우, vpos 동기화가 본문
//! 직후에 들어갈 footer 를 spurious 하게 다음 쪽으로 밀어 한글보다 1쪽 많게 렌더한다(+1쪽).
//!
//! 대표 케이스 `36395270`(서울시 opengov 결재문서): 한글 2쪽인데 #1611 후 rhwp 3쪽(over-push).
//!
//! 정정(typeset.rs): vpos 가 흐름을 plausibly 따를 때(`vpos <= cur_h + block_height`)만 동기화.
//! gap 이 footer 한 개 높이를 초과하면 vpos 를 무시하고 흐름 위치에 배치 → 한글 2쪽 일치.

use std::fs;
use std::path::Path;

#[test]
fn issue_1624_footer_overpush_matches_hangul_page_count() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwpx_path = Path::new(repo_root).join("samples/hwpx/opengov/36395270_footer_overpush.hwpx");
    let bytes =
        fs::read(&hwpx_path).unwrap_or_else(|e| panic!("read {}: {}", hwpx_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse 36395270_footer_overpush.hwpx");

    // 한글 정답지 2쪽 — footer 의 disconnected stored vpos 로 인한 spurious over-push 가 없어야 함.
    assert_eq!(
        doc.page_count(),
        2,
        "footer over-push: stored vpos 동기화로 한글(2쪽)보다 많게 렌더됨 (#1624)"
    );
}
