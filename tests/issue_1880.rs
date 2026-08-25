//! Issue #1880 — 빈-앵커 스택의 spacing_before 인코딩 불안정 (convert-HWP 과억제).
//!
//! 3075729 는 빈 앵커(TopAndBottom 비-TAC 표 호스트) 스택 2쌍을 갖는다.
//! typeset 의 `wrap=1 → spacing_before 제외` 분기가 raw `table.attr` 을 읽어,
//! HWPX 파스(attr=0, 미발동 → sb 보존)와 HWP5 파스(raw wrap=1, 발동 → sb 제외)
//! 가 갈라졌다. 같은 IR 인데 convert-HWP 렌더만 sb 2회분 덜 쌓아 heading 이
//! 한컴 p13 대신 p12 로 오-페이지네이션 (한글 2022 오라클: 양 인코딩 모두 p13
//! = sb 보존이 정답). 빈-앵커 스택은 wrap=1 분기에서도 sb 를 보존한다
//! (#1863 "다음도 표 앵커면 보존" 규칙과 동일 근거).

use std::fs;
use std::path::Path;

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};
use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue1880_anchor_stack_sb_convert.hwpx";

fn read_sample() -> Vec<u8> {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    fs::read(Path::new(repo_root).join(SAMPLE)).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// HWPX 원본 페이지 수 핀: 한글 2022 는 13쪽. 종전 회귀는 convert-HWP 가
/// spacing_before 를 빼 12쪽으로 줄던 것. #6063 문단 중간 되감김을 쪽 경계로
/// 살리면 제10조 꼬리(~70px)가 6쪽에 들어가고, Linux FreeType 본문 메트릭은
/// 그 밀어냄을 흡수하지 못해 14쪽이 된다. 12쪽(sb 과억제)만 금지한다.
#[test]
fn issue_1880_hwpx_renders_13_pages() {
    let core = DocumentCore::from_bytes(&read_sample()).expect("load");
    let n = core.page_count();
    assert!(
        (13..=14).contains(&n),
        "3075729 HWPX 렌더 쪽수 (한컴 13, Linux 되감김 승격 시 14) count={n}"
    );
}

/// convert-HWP 왕복 렌더 자기정합: 인코딩이 달라도 같은 pagination 이어야 한다.
/// 종전: convert-HWP 만 sb 2회분 덜 쌓아 p12 로 오-페이지네이션 (구조 분기).
#[test]
fn issue_1880_convert_hwp_roundtrip_render_is_self_consistent() {
    let data = read_sample();
    let diff = roundtrip_geom(&data, Via::Hwp).unwrap_or_else(|e| panic!("roundtrip_geom: {e:?}"));

    assert_eq!(
        diff.page_count_a, diff.page_count_b,
        "HWPX vs convert-HWP 페이지 수: A={} B={}",
        diff.page_count_a, diff.page_count_b
    );
    assert!(
        (13..=14).contains(&diff.page_count_a),
        "한컴 13쪽 정합 (Linux 되감김 승격 시 14) count={}",
        diff.page_count_a
    );
    for pg in &diff.pages {
        assert!(
            !pg.structure_mismatch,
            "page {} 구조 불일치 (node {} vs {}) — 빈-앵커 스택 sb 인코딩 분기 회귀",
            pg.page, pg.node_count_a, pg.node_count_b
        );
    }
}
