//! [#8079] 줄 끝 공백은 줄 맞춤(오버플로우 압축) 판정에 넣지 않는다.
//!
//! ## 무엇이 문제였나
//!
//! 저장 줄의 가시 본문이 줄 폭 안에 드는데도, 문단 끝에 작성한 말미 공백까지 더한 폭이 줄 폭을
//! 넘으면 rhwp 는 그 줄의 모든 글자 간격을 음수로 줄였다. 자동 줄바꿈 줄의 구분 공백은 이미
//! 빼고 있었지만(#7642) 문단 끝·명시 개행 줄의 말미 공백은 그대로 셌다.
//!
//! ## 정본 실측
//!
//! ```text
//!   samples/issue6793/1611000-201000141-small-air-transport-study.hwp 5쪽
//!   pdf/1611000-201000141-small-air-transport-study-2020.pdf (한컴 2020 Print)
//!     `◦ 해외 소형 항공운송사업 운영 실태 및 발전 가능성 조사 ` 줄(문단 끝 공백)의 글자 원점이 압축 없는 자연 전진이다.
//!     수정 전 rhwp 는 글자마다 약 0.18px 씩 당겨 줄 끝으로 갈수록 왼쪽으로 밀렸다.
//!   samples/hwpx_sample2.hwp 2쪽 `… 충족해야 합니다. ` (같은 규칙, 이 문서는 90% 미만 쪽이 있어 검사 fixture 로 쓰지 않음)
//!     저장 줄 sw 52660HU = 702.13px, rhwp 가시 본문 702.28px + 말미 공백 6.89px.
//!     한컴 Print 의 한글 전진은 압축 없는 0.94em(자간 −6%).
//! ```
//!
//! ## 적용하지 않는 경우
//!
//! 밑줄 친 말미 공백과 마지막 개체 앞 공백은 가시 내용이라 그대로 센다
//! (`trailing_space_width_after_last_inline_object` 의 `stop_on_underline` · 개체 위치).

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6793/1611000-201000141-small-air-transport-study.hwp";
/// 0-based 쪽 번호 (5쪽).
const PAGE: u32 = 4;
const NEEDLE: &str = "해외 소형 항공운송사업 운영 실태 및 발전 가능성 조사";

struct Run {
    text: Vec<char>,
    char_x: Vec<f64>,
    font_size: f64,
    ratio: f64,
    letter_spacing: f64,
}

fn page_runs() -> Vec<Run> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let raw = core.get_page_text_layout_native(PAGE).expect("text-layout");
    let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON");
    layout["runs"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|run| Run {
            text: run["text"].as_str().unwrap_or_default().chars().collect(),
            char_x: run["charX"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_f64)
                .collect(),
            font_size: run["fontSize"].as_f64().unwrap_or(0.0),
            ratio: run["ratio"].as_f64().unwrap_or(1.0),
            letter_spacing: run["letterSpacing"].as_f64().unwrap_or(0.0),
        })
        .collect()
}

#[test]
fn paragraph_end_trailing_space_does_not_compress_the_fitting_line() {
    let runs = page_runs();
    let run = runs
        .iter()
        .find(|run| run.text.iter().collect::<String>().starts_with(NEEDLE))
        .unwrap_or_else(|| panic!("`{NEEDLE}` 로 시작하는 run 이 5쪽에 있어야 한다"));
    assert_eq!(
        run.text.last(),
        Some(&' '),
        "문단 끝 작성 공백으로 끝나는 줄이다"
    );
    assert!(run.char_x.len() > run.text.len(), "charX 는 글자 수 + 1");
    // 완성형 한글은 1em 전진이고 자간은 글자 전진 비례(#2279)라 전각에서는 장평 × (fs + 자간px)다.
    let natural = run.ratio * (run.font_size + run.letter_spacing);
    let hangul: Vec<f64> = (0..run.text.len() - 1)
        .filter(|&i| ('\u{AC00}'..='\u{D7A3}').contains(&run.text[i]))
        .map(|i| run.char_x[i + 1] - run.char_x[i])
        .collect();
    assert!(hangul.len() >= 15, "한글 전진 표본이 충분해야 한다");
    let measured = hangul.iter().sum::<f64>() / hangul.len() as f64;
    assert!(
        (measured - natural).abs() < 0.05,
        "한글 평균 전진 {measured:.3}px — 압축 없는 자연 전진 {natural:.3}px 이어야 한다"
    );
}
