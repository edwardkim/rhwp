//! [#6844] 런 **안**에 오른쪽 탭이 있는 런의 bbox 가 문자 위치와 어긋나, 글리프가 자기
//! 상자 **밖**에 그려진다.
//!
//! `pending_right_tab_render` 는 런이 탭으로 **끝날 때**만 선다 — 정렬 대상 블록이 다음
//! 런에 있는 교차-run 형상이다. 그런데 한컴 목차는 `"\t8"` 처럼 탭과 쪽번호를 **한 런**에
//! 담기도 한다. 그런 런은 `charX` 가 이미 오른쪽 정렬된 자리를 담는데 **bbox 폭만
//! `estimate_text_width` 값(탭 스톱까지)으로 남았다.**
//!
//! `30269`(제도개선 권고안) 4쪽 목차 실측:
//!
//! ```text
//!   런 "\t8"   x=595.4   charX [0.0, 88.5, 98.3]
//!     수정 전  w=78.0  → 상자 673.4 에서 끝나는데 글자는 693.7 까지 간다
//!     수정 후  w=98.3  → 상자와 글자가 693.7 에서 함께 끝난다
//!
//!   형제 줄(교차-run 경로)  "1"·"2"·"8"·"12"·"15"·"18" 모두 오른쪽 끝 693.7
//! ```
//!
//! ⚠ **출력은 불변이다** — 글리프 위치는 이미 옳았고 상자만 짧았다. 이 문서 4쪽 SVG 는
//! 수정 전후 바이트가 같다. 잉크가 아니라 **렌더 트리 계약** 결함이라, hit-test·선택·클립
//! 처럼 상자를 쓰는 소비자만 영향을 받는다. `#6800` 과 같은 부류다.
//!
//! ⚠ 문자 위치 자체는 옮기지 않는다. 교차-run 경로의 `effective_pos` 변환을 그대로 가져와
//! 재정렬해 봤더니 코퍼스에서 34건이 움직였고, 그중 `3142535`(별지 5 징수결정액통지서)의
//! 글자가 `x=671.4 → 1000.1` 로 **용지(793.7) 밖**으로 나갔다 — 기각했다.
//!
//! 코퍼스 10,000건 A/B: `layout-anomaly` 다섯 지표 변화 **0건**. 대상 형상은 2,556건
//! (223문서)이고 그중 **998건이 글리프가 상자 밖**이었다(최대 571.7px).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6844/30269-anticorruption-recommendation-toc.hwp";
/// 목차가 있는 쪽(0 기준).
const TOC_PAGE: u32 = 3;

fn toc_runs() -> Vec<serde_json::Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let layout: serde_json::Value = serde_json::from_str(
        &core
            .get_page_text_layout_native(TOC_PAGE)
            .expect("공개 text-layout"),
    )
    .expect("text-layout JSON");
    layout["runs"].as_array().expect("runs").clone()
}

fn run_text(run: &serde_json::Value) -> &str {
    run["text"].as_str().unwrap_or_default()
}

#[test]
fn issue_6844_intra_run_right_tab_bbox_reaches_the_glyphs() {
    let runs = toc_runs();
    let mut checked = 0usize;
    for run in &runs {
        let text = run_text(run);
        // 탭 뒤에 가시문자가 남는 런만 본다 — 교차-run 경로가 닿지 않는 형상.
        let Some((_, after)) = text.rsplit_once('\t') else {
            continue;
        };
        if after.trim().is_empty() {
            continue;
        }
        let width = run["w"].as_f64().expect("bbox 폭");
        let ink_end = run["charX"]
            .as_array()
            .expect("charX")
            .last()
            .and_then(serde_json::Value::as_f64)
            .expect("마지막 문자 경계");
        checked += 1;
        assert!(
            width + 0.2 >= ink_end,
            "글리프가 자기 bbox 밖에 있으면 안 된다 — 런 {text:?} 폭 {width:.1} \
             < 잉크 끝 {ink_end:.1}"
        );
    }
    assert!(
        checked >= 1,
        "이 쪽에는 탭 뒤 가시문자가 있는 런이 있어야 한다 (확인 {checked})"
    );
}

#[test]
fn issue_6844_toc_page_numbers_share_one_right_edge() {
    // 목차 각 줄은 쪽번호로 끝난다. 교차-run 경로로 정렬된 줄(`"1"`·`"12"` …)과 런 안에서
    // 정렬된 줄(`"\t8"`)이 **같은 오른쪽 끝**을 공유해야 한다.
    let runs = toc_runs();
    let mut right_edges: Vec<f64> = runs
        .iter()
        .filter(|run| {
            let tail = run_text(run).trim_end();
            !tail.is_empty() && tail.chars().all(|ch| ch.is_ascii_digit() || ch == '\t')
        })
        .map(|run| run["x"].as_f64().expect("런 x") + run["w"].as_f64().expect("런 폭"))
        .collect();
    right_edges.sort_by(|a, b| a.partial_cmp(b).expect("유한값"));
    assert!(
        right_edges.len() >= 6,
        "목차 쪽번호 런을 6개 이상 찾아야 한다 (실측 {})",
        right_edges.len()
    );
    let spread = right_edges.last().expect("최댓값") - right_edges[0];
    assert!(
        spread <= 1.0,
        "목차 쪽번호 런의 오른쪽 끝이 한 자리에 모여야 한다 — 결함 시 `\\t8` 줄만 \
         20.3px 짧았다. 실측 {right_edges:?}"
    );
}
