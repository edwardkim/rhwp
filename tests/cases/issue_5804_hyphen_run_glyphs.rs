//! [Issue #5804] 개정문 하이픈 표기(`- - - - -`)를 끊김 없는 실선으로 바꿔 그려
//! HWPX 95건에서 63,782자 중 27자만 남았다 (소실률 99.96%).
//!
//! 근인: Task #352 dash leader — 3연속 이상 '-' 클러스터를 글리프 대신 단일
//! `<line>` 으로 통합(svg/web_canvas/skia 3벡엔드 복제). 한글 2022 는 하이픈
//! 낱글자를 띄엄띄엄 그린다(81240 4쪽 실측: 글리프, 라인 아님).
//!
//! 수정: 3벡엔드의 글리프→선 치환을 제거. 슬랙 분배(문단 justify 가 dash run 에
//! 여백을 싣는 측정 경로)는 그대로라 글리프가 한글처럼 퍼져 그려진다.
//!
//! 픽스처는 원본 HWPX(63쪽)의 구역0 문단 14..17 절단 축소본(45KB) — 신구조문
//! 대비표 하이픈 19줄이 1쪽에 들어 있다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5804/hyphen_run_dash_notation.hwpx";

#[test]
fn issue_5804_hyphen_runs_render_as_glyphs_not_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let svg = core.render_page_svg_native(0).expect("page 1 svg");

    // 하이픈 낱글자가 글리프로 남아야 한다. 결함 시 dash leader 가 삼켜 0~2자.
    let hyphen_glyphs = svg.matches(">-</text>").count();
    assert!(
        hyphen_glyphs >= 200,
        "하이픈 글리프가 대비표만큼 나와야 한다 (결함 시 ~0): {hyphen_glyphs}"
    );

    // dash leader 실선이 더는 없어야 한다. 그 경로만 stroke-width 를 소수 4자리로
    // 방출했다 (예: stroke-width="1.2133") — 괘선/테두리는 다른 포맷을 쓴다.
    let dash_leader_lines = svg
        .split("<line ")
        .skip(1)
        .filter(|head| {
            head.split_once('>')
                .map_or(*head, |(h, _)| h)
                .contains("stroke-width=\"1.2133\"")
        })
        .count();
    assert_eq!(
        dash_leader_lines, 0,
        "하이픈 run 자리에 dash leader 실선이 남아 있다"
    );
}
