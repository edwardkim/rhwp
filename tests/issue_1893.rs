//! Issue #1893 — 빈 누름틀(CLICK_HERE) 서식 HWPX 라운드트립 렌더 위치 자기정합.
//!
//! 해양경찰청 범죄수사규칙 별지 서식 계열(20k 라운드트립 위치 검사에서 표면화)은
//! DocumentCore 로드(안내문 clear) → export_hwpx_native → 재파스 경로에서 렌더가
//! 최대 752px 갈라졌다. 3중 결함 체인:
//!   1) clear_initial_field_texts 가 text/field_ranges 만 고치고
//!      char_offsets/char_count/char_shapes 를 stale 로 방치,
//!   2) 직렬화기가 문단 끝 0-length 필드의 fieldEnd 를 fieldBegin 앞에 방출,
//!   3) 슬롯 루프가 같은 갭의 fieldEnd 몫 8유닛을 다음 fieldBegin 으로 소진해
//!      재파스 LIFO 페어링이 교차(fr(0,0)+(50,50) → fr(0,50)+(0,0)).
//!
//! 권위: pdf/issue1893_clickhere_field_roundtrip-2022.pdf (한글 2022, pyhwpx) —
//! 한컴도 초기 누름틀 안내문을 렌더하지 않으며 1쪽 서식이다.

use std::fs;
use std::path::Path;

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};

const SAMPLE: &str = "samples/issue1893_clickhere_field_roundtrip.hwpx";

#[test]
fn issue_1893_clickhere_form_roundtrip_render_is_self_consistent() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(SAMPLE);
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));

    let diff = roundtrip_geom(&data, Via::Hwpx)
        .unwrap_or_else(|e| panic!("roundtrip_geom({SAMPLE}): {e:?}"));

    assert_eq!(
        diff.page_count_a, diff.page_count_b,
        "{SAMPLE} 라운드트립 페이지 수 불일치: A={} B={}",
        diff.page_count_a, diff.page_count_b
    );
    assert_eq!(diff.page_count_a, 1, "{SAMPLE} 는 1쪽 서식이어야 함");
    assert!(
        diff.max_disp <= 1.0,
        "{SAMPLE} 라운드트립 렌더 변위 {:.2}px > 1.0px (빈 누름틀 필드 왕복 회귀)",
        diff.max_disp
    );
    for pg in &diff.pages {
        assert!(
            !pg.structure_mismatch,
            "{SAMPLE} page {} 구조 불일치 (node {} vs {}) — 필드 페어링/placeholder 왕복 회귀",
            pg.page, pg.node_count_a, pg.node_count_b
        );
    }
}
