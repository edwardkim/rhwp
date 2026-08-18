//! [Issue #4056] HWPX 내보내기 전후로 쪽수가 달라진다 (issue-505-equations.hwp, 4쪽 → 1쪽).
//!
//! HWP5 는 한 BodyText 섹션에 구역(secd)을 여럿 담을 수 있다(issue-505: 수식 4개가 각각 자기
//! 구역 = 4쪽). 종전 HWPX 직렬화기는 `write_section`/`render_runs` 에서 **첫 구역만** secPr
//! 템플릿으로 방출하고 뒤 구역의 SectionDef 를 통째로 드롭해, 뒤 3개 구역의 쪽나눔이 사라져
//! 재파싱 쪽수가 4 → 1 로 붕괴했다(`--verify-pages` 실패).
//!
//! 수정: HWPX 는 한 section0.xml 안에 secPr 를 여럿 둘 수 있으므로(한글 원본 실증:
//! issue2019 10개), 뒤 구역마다 `<hp:secPr>` 를 방출한다. 재파싱 시 구역 4개가 살아 4쪽이 된다.
//!
//! 계약: `samples/issue-505-equations.hwp` 를 HWPX 로 왕복하면 쪽수가 원본과 같아야 한다(4쪽).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};

const SAMPLE: &str = "samples/issue-505-equations.hwp";

#[test]
fn hwpx_roundtrip_preserves_multi_secd_page_count() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let data = std::fs::read(Path::new(repo_root).join(SAMPLE))
        .unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));

    let diff = roundtrip_geom(&data, Via::Hwpx)
        .unwrap_or_else(|e| panic!("roundtrip_geom({SAMPLE}): {e:?}"));

    assert_eq!(
        diff.page_count_a, 4,
        "원본은 구역 4개로 4쪽이어야 한다: {}",
        diff.page_count_a
    );
    assert_eq!(
        diff.page_count_b, diff.page_count_a,
        "HWPX 왕복 쪽수가 원본과 달라졌다 (A={} B={}) — 뒤 구역(secd)이 드롭돼 쪽나눔 소실 (#4056 회귀)",
        diff.page_count_a, diff.page_count_b
    );
}
