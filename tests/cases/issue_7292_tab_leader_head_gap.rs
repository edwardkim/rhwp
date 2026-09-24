//! [#7292] 차례 점끌기가 앞 글자에 붙어 시작해 정본보다 점이 3~4개 더 찍힌다.
//!
//! # 무엇이 깨져 있었나
//!
//! `extract_tab_leaders_with_extended` 는 리더의 시작을 **탭 문자 위치 그대로**
//! (`start_x: before_x`) 두었다. 곧 앞 글자의 advance 끝에 바로 붙는다. 뒤쪽만
//! `0.25em` 을 비웠다.
//!
//! 한/글은 **앞에도 같은 0.25em 을 비운다.** 두 정본에서 같은 비율이 나온다
//! (`pdftotext -bbox` 의 advance 상자 기준, 앞 글자 `xMax` → 첫 점 `xMin`).
//!
//! ```text
//!   문서                               앞 여백     px(96dpi)
//!   pdf/KTX-2022.pdf 2쪽 (목차)        3.70pt      4.93
//!   pdf/aift-2022.pdf 4쪽 (목차)       3.24pt      4.32
//!   1170000-200500003 … (한/글 2020)   2.88pt      3.84
//! ```
//!
//! 그 결과 같은 줄에 점이 3~4개 더 들어갔다(정본 63·92개 ↔ 우리 66·96개).
//!
//! # 이 검사
//!
//! `samples/KTX.hwp` 목차 쪽의 **첫 점끌기 시작 x** 를 정본 좌표로 잠근다.
//! 정본 `pdf/KTX-2022.pdf` 2쪽에서 `사업 개요` 의 advance 끝이 160.6pt, 첫 점이
//! 164.3pt = **219.07px** 다. 수정 전에는 214.17px(−4.9px)였다.
//!
//! 뒤 여백은 잠그지 않는다 — 점은 탭 정지점에서 끝나고 쪽번호가 오른쪽 정렬되므로
//! 그 사이 간격은 번호 폭에 따라 달라진다(정본 실측 5.76pt ↔ 9.2pt).

#![cfg(not(target_arch = "wasm32"))]

/// 정본 `pdf/KTX-2022.pdf` 2쪽의 첫 점끌기 시작 x (96dpi px).
const ORACLE_FIRST_LEADER_X_PX: f64 = 219.07;

/// 수정 전 값 — 이 검사가 무엇을 막는지 남긴다.
const BEFORE_FIX_X_PX: f64 = 214.17;

fn first_leader_start_x(svg: &str) -> f64 {
    for line in svg.lines() {
        if !line.contains("stroke-dasharray") || !line.contains("<line") {
            continue;
        }
        let Some(rest) = line.split("x1=\"").nth(1) else {
            continue;
        };
        let Some(value) = rest.split('"').next() else {
            continue;
        };
        if let Ok(x) = value.parse::<f64>() {
            return x;
        }
    }
    panic!("점끌기 선(stroke-dasharray)을 찾지 못했다");
}

#[test]
fn toc_leader_starts_one_quarter_em_after_the_text() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/KTX.hwp");
    let bytes = std::fs::read(&path).expect("KTX.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("문서 로드");
    let svg = doc.render_page_svg_native(1).expect("목차 쪽 SVG");

    let x = first_leader_start_x(&svg);
    assert!(
        (x - ORACLE_FIRST_LEADER_X_PX).abs() <= 0.6,
        "첫 점끌기 시작 {x:.2}px — 정본(pdf/KTX-2022.pdf 2쪽) {ORACLE_FIRST_LEADER_X_PX:.2}px \
         (수정 전 {BEFORE_FIX_X_PX:.2}px = 앞 글자에 바로 붙음)"
    );
}
