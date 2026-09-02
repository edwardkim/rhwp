//! [Issue #5699 H1] 저장 사다리가 자리차지 표 밴드를 계상하지 않아 본문이 표를
//! 관통해 그려지던 결함(코호트 77문서 중 지배 가족 40문서)의 회귀 가드.
//!
//! 근인: 기계생성(자치법규 등) 사다리는 자리차지 표의 매핑 줄 th 를 한 줄
//! 높이(예: 1000HU=13px)로 기록해, 397px 표가 흐름에 13px 만 계상됐다. 후속
//! 문단들이 표 페인트 대역 안에 배치·페인트되어 서식이 읽을 수 없게 된다.
//!
//! 수정: 저장 th 기반 계상이 **선언 표높이와 실측 둘 다**의 1/4 미만이고 선언·실측이
//! 정합(2배 이내)이면 사다리 자기모순으로 보고 실측 높이로 교정 + 후속 저장 vpos
//! 후방 스냅 금지(`ladder_band_floor`/`min_flow_floor`). 한글 2022 오라클 실측
//! (영월군 20099369): 한글은 이 문서를 재조판해 **4쪽**, 표 아래 깨끗한 배치.
//!
//! 게이트 계약: HWPX 컨테이너 제외(36397752 하자검사조서 — 한글 1쪽 유지 실측,
//! 발동 시 +1 회귀), 직파싱 HWP3 는 tac=true 모순 조합만(tac=false TopAndBottom 은
//! 겹침이 한글 정본인 #4533 하동군 계열).
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5699/20099369_yeongwol_forms.hwp";

fn load() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"))
}

/// 한글 2022 오라클 쪽수(4쪽) 정합 — 종전에는 사다리 추종으로 2쪽에 전부 압착.
#[test]
fn issue_5699_yeongwol_page_count_matches_hangul_oracle() {
    let doc = load();
    assert_eq!(
        doc.page_count(),
        4,
        "영월군 별지 서식은 한글 2022 재조판 기준 4쪽이어야 한다 (#5699 H1 회귀)"
    );
}

/// p1 에서 계약조건 본문(문단 pi=4..)이 표 페인트 대역 아래에서 시작해야 한다.
/// 종전에는 표(약 y183..580) 안 y213 부터 겹쳐 그려졌다.
#[test]
fn issue_5699_yeongwol_body_starts_below_table_band() {
    let doc = load();
    let svg = doc
        .render_page_svg_native(0)
        .unwrap_or_else(|e| panic!("render p1: {e}"));
    // SVG 는 글자 단위 `<text transform="translate(x,y) ...">글자</text>` 방출이다.
    // '건' 은 p1 에서 "임대차계약조건" 제목에만 나오는 글자 — 그 y 로 위치를 판정한다.
    let mut ys = Vec::new();
    let mut rest = svg.as_str();
    while let Some(i) = rest.find(">건<") {
        let head = &rest[..i];
        if let Some(t) = head.rfind("translate(") {
            let coords = &head[t + "translate(".len()..];
            if let Some(y) = coords
                .split(')')
                .next()
                .and_then(|xy| xy.split(',').nth(1))
                .and_then(|v| v.trim().parse::<f64>().ok())
            {
                ys.push(y);
            }
        }
        rest = &rest[i + ">건<".len()..];
    }
    assert!(!ys.is_empty(), "p1 에 계약조건 제목('건')이 없다");
    let min_y = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(
        min_y > 570.0,
        "계약조건 제목이 표 대역(y~183..580) 아래(>570)에 있어야 한다 — 실측 y={min_y:.1} (#5699 H1 회귀)"
    );
}
