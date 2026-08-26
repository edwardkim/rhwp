//! [Issue #6030] TAC 다문단 셀의 마지막 줄 trailing 줄간격이 **음수**(압축 줄간격
//! 70%)일 때 측정이 이를 합산해 행을 과소 측정, 마지막 선택지 줄의 descender 가
//! 행 괘선에 깎인다 (`samples/issue6030/2386771_agritech_review_form.hwp` 1쪽
//! 심사서식 셀 10곳, 초과 0.9~2.7px).
//!
//! 기전: 평가항목 셀은 1줄 문단 5개, 저장 ls=-332HU(=lh 1100 의 70% 압축).
//! [Task #874/#1086] TAC 다문단 예외는 셀 마지막 줄에도 trailing ls 를 포함하는데
//! 음수면 마지막 글리프 박스(14.67px)를 10.24px 로 압축해 행이 4.4px 부족해진다.
//! 페인트는 마지막 줄을 lh 그대로 그리므로 측정/페인트 발산 = clip 깎임.
//! 한글 2022 정본(2020 저작 문서, COM PDF producer=Hancom PDF) 실측: 평가항목 행
//! 피치 64.5pt(rhwp 결함 61.2pt), 1쪽 사다리 21개 전 괘선이 수정 후 ≤1.0pt 일치.
//!
//! 수정: include_trailing_ls 로 포함되는 **셀 마지막 줄** 의 trailing 만 0 으로
//! 클램프 (음수 한정 — 양수 trailing 포함 회계인 KTX TOC 핀은 불변).
//!
//! 결함 상태에서는 행 피치 81.6px(61.2pt)·표 1쪽 하단 978.5px 로 두 밴드가 실패.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6030/2386771_agritech_review_form.hwp";

#[test]
fn issue_6030_negative_trailing_ls_grows_row_to_content() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(core.page_count(), 2, "한글 정본은 2쪽이다");

    let svg = core.render_page_svg_native(0).expect("page 1 svg");
    let rules = full_width_rule_ys(&svg);
    assert_eq!(
        rules.len(),
        21,
        "1쪽 전폭 괘선 사다리는 21개여야 한다: {rules:?}"
    );

    // 평가항목(기술성) 행 피치 — 한글 64.5pt=86.0px. 결함 시 81.6px(61.2pt).
    let eval_row_pitch = rules[3] - rules[2];
    assert!(
        (85.2..=86.9).contains(&eval_row_pitch),
        "평가항목 행 피치가 한글 정본(86.0px=64.5pt) 근방이어야 한다 (결함 시 81.6px): {eval_row_pitch:.1}"
    );

    // 1쪽 표 하단 괘선 — 한글 762.7pt=1016.9px (rhwp 정합 1018.2px). 결함 시 978.5px.
    let bottom = *rules.last().expect("괘선");
    assert!(
        (1013.0..=1023.0).contains(&bottom),
        "1쪽 표 하단 괘선이 한글 정본(1017px) 근방이어야 한다 (결함 시 978.5px): {bottom:.1}"
    );
}

/// 전폭 가로 괘선(`y1 == y2`, 길이 400px 초과)의 y 를 오름차순으로 모은다.
/// 이중 괘선은 3.3px(2.5pt) 이내로 묶어 한 경계로 센다.
fn full_width_rule_ys(svg: &str) -> Vec<f64> {
    let mut ys: Vec<f64> = Vec::new();
    for chunk in svg.split("<line ").skip(1) {
        let Some(end) = chunk.find('>') else {
            continue;
        };
        let head = &chunk[..end];
        let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
            attr(head, "x1"),
            attr(head, "y1"),
            attr(head, "x2"),
            attr(head, "y2"),
        ) else {
            continue;
        };
        if (y1 - y2).abs() > 0.01 || (x2 - x1).abs() < 400.0 {
            continue;
        }
        ys.push(y1);
    }
    ys.sort_by(|a, b| a.partial_cmp(b).expect("좌표는 유한값"));

    let mut merged: Vec<f64> = Vec::new();
    for y in ys {
        let is_new_boundary = match merged.last() {
            None => true,
            Some(last) => (y - last).abs() > 3.3,
        };
        if is_new_boundary {
            merged.push(y);
        }
    }
    merged
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
