//! [Issue #6032] 직전 쪽 말미 anchor 의 자리차지 표가 다음 쪽으로 흘러넘친 뒤의
//! 빈-host 자리차지 표가 한글보다 6.1pt 아래에 그려져 하단 괘선이 다음 문단
//! 글줄을 관통한다 (`samples/issue6032/2912695_civil_petition_form.hwp` 2쪽
//! "- 신청서 작성요령 -" 1×1 표).
//!
//! 저장 사다리: pi=3(7×6 표 host)이 1쪽 말미 vpos=65212 에 남고 표만 2쪽으로
//! 흘러넘치며, pi=4(1×1 표의 빈 host)는 vpos=57794 로 **되감긴다**(쪽 경계 신호,
//! 2쪽 page-relative 좌표). 한글 정본 실측: 표 상단 666.0pt(=888.0px),
//! 다음 문단 "(용지규격…)"은 저장 vpos 64553(=969.2px) — 사다리 델타 6759HU 는
//! v_off(595)+outer_top(141)+선언높이(5882)+outer_bottom(141)과 정확히 일치한다.
//!
//! rhwp 흐름은 넘친 표 뒤에 host 줄 간격까지 계상해 anchor 가 +9.0px 표류했고
//! (888.09 vs 저장 879.06), 그 위에 v_off 7.9px 가 다시 얹혀 표가 896.0px 에
//! 그려졌다 — 하단 괘선 974.4px 가 다음 글줄(966.5px)을 관통. 수정은 ① 되감긴
//! 저장 anchor 로 host y 스냅 ② 스냅 문단에 한해 물리 사다리 여분(v_off+outer)
//! 흐름 계상(#5870 게이트 확장) — 결과 표 887.0px·다음 줄 969.2px 로 정합.
//!
//! 결함 상태에서는 표 상단 896.0px·하단 974.4px 로 두 밴드 어서션이 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6032/2912695_civil_petition_form.hwp";

#[test]
fn issue_6032_rewound_empty_anchor_table_snaps_to_saved_vpos() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    assert_eq!(core.page_count(), 2, "한글 정본은 2쪽이다");

    let svg = core.render_page_svg_native(1).expect("page 2 svg");
    let rules: Vec<f64> = horizontal_rule_ys(&svg)
        .into_iter()
        .filter(|y| *y > 880.0)
        .collect();
    assert_eq!(
        rules.len(),
        2,
        "2쪽 하단 1×1 표 괘선은 상·하 2개여야 한다: {rules:?}"
    );

    // 표 상단 — 한글 실측 666.0pt=888.0px (저장 anchor 879.06 + v_off 7.93).
    // 결함 시 896.0px (흐름 표류 +9.0px 위에 v_off 재가산).
    let top = rules[0];
    assert!(
        (883.0..=891.0).contains(&top),
        "1×1 표 상단 괘선이 한글 정본(888.0px) 근방이어야 한다 (결함 시 896.0): {top:.1}"
    );

    // 표 하단 — 정합 965.4px. 다음 문단 "(용지규격…)" 줄 상단(저장 vpos 969.2px)
    // 위에 있어야 한다. 결함 시 974.4px 로 글줄을 관통한다.
    let bottom = rules[1];
    assert!(
        (960.0..=968.0).contains(&bottom),
        "1×1 표 하단 괘선이 다음 글줄(969.2px) 위여야 한다 (결함 시 974.4): {bottom:.1}"
    );
}

/// 표 괘선 중 가로선(`y1 == y2`, 길이 100px 초과)의 y 를 오름차순으로 모은다.
/// 이중 괘선은 3px 이내로 묶어 한 경계로 센다.
fn horizontal_rule_ys(svg: &str) -> Vec<f64> {
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
        if (y1 - y2).abs() > 0.01 || (x2 - x1).abs() < 100.0 {
            continue;
        }
        ys.push(y1);
    }
    ys.sort_by(|a, b| a.partial_cmp(b).expect("좌표는 유한값"));

    let mut merged: Vec<f64> = Vec::new();
    for y in ys {
        let is_new_boundary = match merged.last() {
            None => true,
            Some(last) => (y - last).abs() > 3.0,
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
