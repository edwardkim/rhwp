//! [#8079] 굵은 글자는 Bold 면으로 그리되 전진폭은 같은 family 보통(Regular) 면으로 잡는다.
//!
//! ## 무엇이 문제였나
//!
//! rhwp 는 별도 Bold 면이 있는 TrueType 의 굵은 글자를 Bold 면의 전진폭으로 쟀다. 맑은 고딕
//! 숫자는 Bold 1187/2048(0.580em) 이고 보통 1128/2048(0.551em) 이라, 저장 글줄 없는 문단을
//! 재조판하면 한/글보다 줄이 일찍 꺾였다(80168 133쪽 셀 첫 문단 `④ 사업` / 한/글 `④ 사업시`).
//!
//! ## 정본 실측
//!
//! 한컴 PDF 는 임베드 Bold 글꼴의 `/W`(숫자 579/1000)를 그대로 둔 채 TJ 조정으로 다음 글자를
//! 당겨 보통 면 전진폭에 놓는다 — `[<5>29.000<4>]` → 579 − 29 = 550.
//!
//! ```text
//!   pdf/ 한컴 정본 205개, TJ 안 굵은 글자의 (/W − 조정)
//!     MalgunGothicBold   보통 폭 쪽 26,893자 · Bold 폭 쪽 357자
//!     Arial-BoldMT · TimesNewRomanPS-BoldMT   Bold 폭 쪽 0자
//!   pdf/hwpx/2026_oss_rst.pdf 1쪽 굵은 맑은 고딕 `- 제출 기한 : ~2026. 8. 27.(목) 18:00`
//!     `~` 원점 250.26px → `18:00` 끝 422.32px = 172.06px
//!     수정 전 rhwp 181.85px (Bold 폭) / 보통 폭이면 171.8px
//! ```
//!
//! ## 적용하지 않는 경우
//!
//! 선언 face 가 HFT 프로그램이면 한/글은 그 프로그램의 굵은 전진폭(Type3)을 쓴다. rhwp 가 그
//! 값을 모르는 상태에서 표시 대체 TTF 의 보통 폭으로 바꾸면 정본에서 더 멀어진다
//! (`issue5169` 1쪽 가운데 정렬 제목, HFT `HCI Poppy` → 표시 Palatino Linotype: 보통 폭으로
//! 바꾸면 둘째 줄 시작이 정본 140.6px 에서 143.8px 로 밀렸다). 마지막 시험이 그 경계를 잠근다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 정본 `pdf/hwpx/2026_oss_rst.pdf` (HWP·HWPX 두 저장본이 같은 정본을 쓴다).
const SAMPLE_OSS_HWP: &str = "samples/2026_oss_rst.hwp";
const SAMPLE_OSS_HWPX: &str = "samples/hwpx/2026_oss_rst.hwpx";
/// 반례(HFT 선언 굵은 영문). 정본 `pdf/issue7674/issue5169_viewtext_changetracking-hwpx-2020.pdf`.
const SAMPLE_ISSUE_5169: &str = "samples/issue5169_viewtext_changetracking.hwp";

/// 맑은 고딕 보통 면 숫자 전진폭 — `malgun.ttf` hmtx 1128 / unitsPerEm 2048.
const MALGUN_REGULAR_DIGIT_EM: f64 = 1128.0 / 2048.0;
/// 맑은 고딕 Bold 면 숫자 전진폭 — `malgunbd.ttf` hmtx 1187 / 2048. 이 값이면 수정 전 측정이다.
const MALGUN_BOLD_DIGIT_EM: f64 = 1187.0 / 2048.0;

struct Run {
    text: Vec<char>,
    x: f64,
    y: f64,
    char_x: Vec<f64>,
    font_size: f64,
    family: String,
    bold: bool,
}

fn page_runs(sample: &str, page: u32) -> Vec<Run> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let raw = core.get_page_text_layout_native(page).expect("text-layout");
    let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON");
    layout["runs"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|run| Run {
            text: run["text"].as_str().unwrap_or_default().chars().collect(),
            x: run["x"].as_f64().unwrap_or(0.0),
            y: run["y"].as_f64().unwrap_or(0.0),
            char_x: run["charX"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_f64)
                .collect(),
            font_size: run["fontSize"].as_f64().unwrap_or(0.0),
            family: run["fontFamily"].as_str().unwrap_or_default().to_string(),
            bold: run["bold"].as_bool().unwrap_or(false),
        })
        .collect()
}

fn primary_family(run: &Run) -> &str {
    run.family
        .split(',')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches('\'')
}

/// 굵은 맑은 고딕 run 중 `needle` 을 담은 것.
fn bold_malgun_run<'a>(runs: &'a [Run], needle: &str) -> &'a Run {
    runs.iter()
        .find(|run| {
            run.bold
                && matches!(primary_family(run), "맑은 고딕" | "Malgun Gothic")
                && run.text.iter().collect::<String>().contains(needle)
        })
        .unwrap_or_else(|| panic!("굵은 맑은 고딕 run `{needle}` 이 1쪽에 있어야 한다"))
}

fn digit_advances_em(run: &Run) -> Vec<(char, f64)> {
    assert!(run.char_x.len() > run.text.len(), "charX 는 글자 수 + 1");
    run.text
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_ascii_digit())
        // 다음 글자가 같은 run 안에 있어 전진폭이 정해진 숫자만 본다.
        .filter(|(i, _)| i + 1 < run.text.len())
        .map(|(i, &c)| (c, (run.char_x[i + 1] - run.char_x[i]) / run.font_size))
        .collect()
}

#[test]
fn bold_malgun_digits_advance_by_the_regular_face() {
    for sample in [SAMPLE_OSS_HWP, SAMPLE_OSS_HWPX] {
        let runs = page_runs(sample, 0);
        let run = bold_malgun_run(&runs, "2026");
        let digits = digit_advances_em(run);
        // `2026.` · `8.` · `27.` — 같은 run 안에서 다음 글자가 있는 숫자 7개.
        assert!(digits.len() >= 7, "{sample}: 숫자 표본 {}", digits.len());
        for (c, em) in digits {
            // charX 는 0.1px 로 반올림돼 있다 — 16px 에서 ±0.006em.
            assert!(
                (em - MALGUN_REGULAR_DIGIT_EM).abs() < 0.007,
                "{sample}: 굵은 `{c}` 전진폭 {em:.4}em — 보통 면 {MALGUN_REGULAR_DIGIT_EM:.4}em 이어야 한다 (Bold 면 {MALGUN_BOLD_DIGIT_EM:.4})"
            );
        }
    }
}

#[test]
fn bold_malgun_line_span_matches_the_hancom_print() {
    // 정본: `~` 원점 250.26px, `18:00` 끝 422.32px (pdftotext -bbox-layout, 96dpi 환산).
    const HANCOM_SPAN_PX: f64 = 422.32 - 250.26;
    for sample in [SAMPLE_OSS_HWP, SAMPLE_OSS_HWPX] {
        let runs = page_runs(sample, 0);
        let start = bold_malgun_run(&runs, "~");
        let end = bold_malgun_run(&runs, "18:00");
        let tilde = start.text.iter().position(|&c| c == '~').expect("`~`");
        let last = end.text.len() - 1;
        assert_eq!(
            end.text[last], '0',
            "{sample}: run 끝은 `18:00` 의 마지막 `0`"
        );
        let span = (end.x + end.char_x[last + 1]) - (start.x + start.char_x[tilde]);
        assert!(
            (span - HANCOM_SPAN_PX).abs() < 0.8,
            "{sample}: `~` → `18:00` 끝 {span:.2}px, 정본 {HANCOM_SPAN_PX:.2}px"
        );
    }
}

#[test]
fn bold_hangul_and_regular_digits_keep_their_advances() {
    let runs = page_runs(SAMPLE_OSS_HWP, 0);
    // 같은 굵은 run 의 한글은 두 면 모두 1em 이다.
    let run = bold_malgun_run(&runs, "제출");
    let i = run.text.iter().position(|&c| c == '제').expect("`제`");
    let em = (run.char_x[i + 1] - run.char_x[i]) / run.font_size;
    assert!((em - 1.0).abs() < 0.007, "굵은 한글 `제` {em:.4}em");
    // 보통 굵기 맑은 고딕 숫자는 종전과 같은 보통 면 폭이다.
    let regular = runs
        .iter()
        .find(|run| {
            !run.bold
                && matches!(primary_family(run), "맑은 고딕" | "Malgun Gothic")
                && digit_advances_em(run).len() >= 2
        })
        .expect("보통 굵기 맑은 고딕 숫자 run");
    for (c, em) in digit_advances_em(regular) {
        assert!(
            (em - MALGUN_REGULAR_DIGIT_EM).abs() < 0.007,
            "보통 `{c}` {em:.4}em"
        );
    }
}

#[test]
fn hft_declared_bold_latin_keeps_the_bold_display_advance() {
    // `issue5169` 1쪽 제목은 영문 슬롯이 HFT `HCI Poppy`(alt_type 2) 인 굵은 글자다.
    // 정본 둘째 줄 `Vision) 수립에 대한 한국의 기여` 는 140.6px → 652.9px (512.3px).
    // 보통 폭으로 바꾸면 이 줄이 6.6px 좁아져 가운데 정렬 시작이 3.3px 밀린다.
    const HANCOM_SECOND_LINE_PX: f64 = 652.9 - 140.6;
    let runs = page_runs(SAMPLE_ISSUE_5169, 0);
    let vision = runs
        .iter()
        .find(|run| run.bold && run.text.iter().collect::<String>().starts_with("Vision"))
        .expect("`Vision` run");
    // 같은 줄 = 굵은 run 중 상자 윗변이 `Vision` run 과 한 줄 높이의 절반 안.
    let line_runs: Vec<&Run> = runs
        .iter()
        .filter(|run| run.bold && (run.y - vision.y).abs() < vision.font_size * 0.5)
        .collect();
    assert!(line_runs.len() >= 2, "둘째 줄은 영문·한글 run 이 섞여 있다");
    let first_x = vision.x + vision.char_x[0];
    // 정본 줄 끝은 마지막 보이는 글자 끝이다(뒤 공백 제외).
    let end_x = line_runs
        .iter()
        .filter_map(|run| {
            let last = run.text.iter().rposition(|c| !c.is_whitespace())?;
            Some(run.x + run.char_x[last + 1])
        })
        .fold(f64::MIN, f64::max);
    let span = end_x - first_x;
    assert!(
        (span - HANCOM_SECOND_LINE_PX).abs() < 2.0,
        "HFT 굵은 제목 둘째 줄 {span:.2}px, 정본 {HANCOM_SECOND_LINE_PX:.2}px"
    );
}
