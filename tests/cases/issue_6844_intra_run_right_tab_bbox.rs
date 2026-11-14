//! #6844 탭 뒤 글리프가 자기 bbox를 벗어나던 API 기하 결함을 검사한다.
//!
//! 검사 종류: 관계형. 첫 검사는 같은 런의 탭 뒤 문자 경계를 bbox가 포함하는지,
//! 두 번째는 각 목차 숫자 런의 bbox 끝이 자기 charX 끝과 일치하는지 검사한다.
//! 0.2는 소수 한 자리 JSON 경계 두 개의 반올림 오차이며 실물의 절대 px 핀이 아니다.
//! 주석 갱신: 2026-11-15 03:17 KST.
//! 시각 근거: 같은 원본 권고안 4쪽 Native/fresh WASM 각각 99.96%,
//! 전 22쪽 최저 98.69/98.68%, head `353989d9f2`. 같은 쪽 review/overlay 직접 판독을 마쳤다.
//! 기준 PDF: pdf/30269-anticorruption-recommendation-toc-2020.pdf (기존 한컴 Print).
//!
//! 원 해결 c1a1a5dcf3의 예시는 "\t8"의 bbox 폭 78.0과 실제 문자 끝 98.3의 불일치다.
//! 이 이탈을 거절하는 첫 검사는 유지한다. 모든 숫자의 우단이 동일하다는 보조 가정은
//! 서로 다른 스타일·저장 탭 전진을 혼동하므로 각 런의 소유 경계 관계로 교정했다.
//! 저장 전진 보정 후 기존 우단 편차는 1.3px지만 실제 출력은 99.86→99.96%로 개선됐다.
//! 첫 검사는 짧은 bbox를, 두 번째는 짧거나 지나치게 넓은 bbox를 모두 거절한다.
//! 특정 쪽번호의 glyph 원점을 옮기거나 원래 1px 문턱만 늘려 통과시키지 않는다.
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
fn issue_6844_toc_page_number_bboxes_match_their_own_character_ends() {
    let runs = toc_runs();
    let numbers: Vec<_> = runs
        .iter()
        .filter(|run| {
            let tail = run_text(run).trim_end();
            !tail.is_empty() && tail.chars().all(|ch| ch.is_ascii_digit() || ch == '\t')
        })
        .collect();
    assert!(
        numbers.len() >= 6,
        "목차 쪽번호 런을 6개 이상 찾아야 한다 (실측 {})",
        numbers.len()
    );
    for run in numbers {
        let width = run["w"].as_f64().expect("bbox 폭");
        let end = run["charX"]
            .as_array()
            .expect("charX")
            .last()
            .and_then(serde_json::Value::as_f64)
            .expect("마지막 문자 경계");
        assert!(
            (width - end).abs() <= 0.2,
            "쪽번호 {:?}의 bbox 끝은 자기 문자 끝과 같아야 한다: 폭={width}, 문자 끝={end}",
            run_text(run)
        );
    }
}
