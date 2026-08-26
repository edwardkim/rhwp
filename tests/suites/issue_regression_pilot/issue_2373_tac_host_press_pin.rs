//! Issue #2373 — 보도자료(hwpx, 한글 저장) 모순 TAC 표 문단의 +1쪽 회귀 핀.
//!
//! `samples/issue2373/156689818_kftc_press.hwpx` — 공정위 보도자료, 한글 4쪽.
//! p1→p2 로 분할되는 모순 TAC 표(treat_as_char + TopAndBottom, pi=15) 뒤로
//! 본문이 이어 흐른다. 종전 #2352 host 줄박스 가산(host CS 크기 font_size
//! box)은 이 문서군을 +1쪽으로 회귀시켰고(10k r16 실측 군집), #2441 의
//! host sb+sa 정확 모델(sb=0 → 무가산)이 이를 해소했다 — 한글 재저장 오라클
//! 실측에서 한글 fresh 스텝은 저장 ladder 와 일치(d=0, 무가산).
//!
//! 핀 채집 계보: PR #2454(step-trust 시도, 오라클·샘플 채집)에서 승계.
//! step-trust 본체는 devel(#2441) 대비 10k FIXED 0/REGRESSED 9 로 기각 —
//! 본 핀은 font_size 근사류 재도입 회귀를 막는 가드로만 유지한다.

use std::fs;
use std::path::Path;

fn page_count_of(rel: &str) -> u32 {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.page_count()
}

#[test]
fn press_156689818_page_count_pin() {
    let pages = page_count_of("samples/issue2373/156689818_kftc_press.hwpx");
    assert_eq!(
        pages, 4,
        "issue2373 156689818 핀 4쪽 (한글 2022 정답지 4쪽 정합). \
         5p 면 모순 TAC host 과대 가산(#2352 계열) 회귀 — 분할 표 꼬리가 \
         페이지를 단독 점유하고 후속 본문이 밀렸는지 확인할 것. 실측 {}p.",
        pages
    );
}
