//! [#5941 f8c784235 축] 저장 tail 넘침 게이트가 누적 드리프트를 못 넘겨 꼬리
//! 한 줄이 단독 쪽으로 밀리던 회귀(+8쪽) 가드.
//!
//! 1490000-201600081(비정규직 목표관리 로드맵, 304쪽급 대형 HWP5) — f8c784235
//! 가 고정 20px 꼬리 허용치(TAIL_BREAK_OVERFLOW_TOLERANCE_PX)를 저장 bounds
//! 게이트(saved_tail_overflow_to_fit)로 바꾸면서, 흐름이 누적 드리프트로 저장
//! tail 을 이미 지나친(cur > bottom) 형상에서 overlap 조건이 깨져 게이트가
//! 죽었다 — p61 실측: 저장 853.9..868.5(body 876.9 안), 흐름 878.3(top+24.4).
//! 수정: 흐름이 **이미 body 를 넘긴 상태**(cur > body — body 안이면 일반 fit
//! 소관, task1725 242쪽 핀이 그 반증)에서 tail 상단 드리프트 허용(#5822 상수,
//! 64px) 안이면 지나쳤어도 source 증거로 신뢰(top==0 쪽-시작 센티널 제외,
//! #6027). 수정 후 이 문서는 305쪽(수정 전 devel 311, r39 312, r37 304,
//! 한글 2024 302 — `cur > body` 만 요구하던 판은 305).
//!
//! **[#5941 잔존]** 그 "cur ≤ body 형상"을 열었다. `cur > body` 를 통째로 요구하는 대신
//! **흐름이 body 바닥에 앉은 형상만**(`body − cur <= BODY_BOTTOM_SEAT_PX`) 거부한다.
//! 이 문서는 **305 → 304**(r37 과 같고 한글 302 에 한 걸음 더 가깝다).
//! 같은 구간의 `1490000-200600032` 도 82 → 81(r37·한글 79).

use rhwp::document_core::DocumentCore;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5941/1490000-201600081_roadmap_research.hwp";

#[test]
fn issue_5941_drift_past_saved_tail_still_grants_overflow() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(path).expect("read #5941 fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse #5941 fixture");

    assert_eq!(
        core.page_count(),
        304,
        "1490000-201600081 은 이 게이트로 304쪽이다 (r37 과 같고 한글 302, 게이트 사각 시 311~312 — \
         `cur > body` 만 요구하던 판은 305)"
    );
}
