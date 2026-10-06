//! [#7092] 고정폭 표의 작은따옴표 `‘`·`’` 를 적힌 전각 폭으로 잰다.
//!
//! ## 무엇이 문제였나
//!
//! `measure_char_width_embedded` 의 `is_narrow_unicode_punct` 분기는 전각으로 적힌 값을
//! face 와 무관하게 `em × 0.3` 으로 눌러 왔다. 같은 분기의 `·`(U+00B7)는 `#630` 에서 이미
//! 고정폭 표를 예외로 두었는데(`is_monospace_metric`), 따옴표에는 그 예외가 없었다.
//!
//! ## 정본 실측 — 같은 줄 한글 전진폭 대비 비율
//!
//! 저장소 한컴 정본 623개(앞 6쪽·연속 리더 제외)에서 고정폭 face 의 따옴표는 전부 전각이다.
//!
//! ```text
//!   GulimChe   ‘ n=10 1.011   ’ n=6  1.000
//!   BatangChe  ‘ n=12 1.000   ’ n=8  1.000
//!   DotumChe   ‘ n=12 1.000   ’ n=12 1.000      관측 34건 전부 0.98 이상
//!   글꼴 파일  gulim.ttc·batang.ttc 의 GulimChe·BatangChe quoteleft·quoteright = 1024/1024
//! ```
//!
//! 이 시험이 쓰는 두 표본의 같은 문맥 대조는 아래와 같다.
//!
//! ```text
//!   samples/hwpx/issue_157.hwpx 2쪽 `‘주주총회 소집통지서’`
//!     pdf/hwpx/issue_157-hwpx-2020.pdf     ‘ 1.012 · ’ 1.000
//!   samples/task2097/1730000_selection_report.hwp
//!     pdf/task2097/1730000_selection_report-hwp-2020.pdf   ‘ 1.000
//!   수정 전 rhwp 는 둘 다 0.299 였다.
//! ```
//!
//! ## 반례
//!
//! 비고정폭 face 는 이 변경의 범위 밖이다. 같은 이름이 TrueType/HFT 두 realization 으로
//! 갈리고(휴먼명조 `‘` 정본 1.010 ↔ 0.24) 기호 슬롯 선택 축이 따로 있다. 아래 둘째 시험이
//! 그 face 의 종전 폭 유지를 잠근다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 굴림체 앵커. 정본 `pdf/hwpx/issue_157-hwpx-2020.pdf`.
const SAMPLE_ISSUE_157: &str = "samples/hwpx/issue_157.hwpx";
/// 돋움체 앵커. 정본 `pdf/task2097/1730000_selection_report-hwp-2020.pdf`.
const SAMPLE_1730000: &str = "samples/task2097/1730000_selection_report.hwp";
/// 비고정폭(휴먼명조) 반례. 정본 `pdf/issue7235/156467175_press_release_header_logo_p1-2020.pdf`.
const SAMPLE_156467175: &str = "samples/issue7235/156467175_press_release_header_logo_p1.hwp";

/// 한 문서에서 `‘`·`’` 의 (글꼴, 글자크기 대비 전진폭) 을 모은다.
fn quote_advances(sample: &str, max_pages: u32) -> Vec<(String, f64)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let mut out = Vec::new();
    for page in 0..core.page_count().min(max_pages) {
        let Ok(raw) = core.get_page_text_layout_native(page) else {
            continue;
        };
        let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON");
        for run in layout["runs"].as_array().into_iter().flatten() {
            let text: Vec<char> = run["text"].as_str().unwrap_or_default().chars().collect();
            let font_size = run["fontSize"].as_f64().unwrap_or(0.0);
            let family = run["fontFamily"].as_str().unwrap_or_default().to_string();
            let char_x: Vec<f64> = run["charX"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_f64)
                .collect();
            if font_size <= 0.0 || char_x.len() < text.len() + 1 {
                continue;
            }
            for (i, &c) in text.iter().enumerate() {
                if c != '\u{2018}' && c != '\u{2019}' {
                    continue;
                }
                out.push((family.clone(), (char_x[i + 1] - char_x[i]) / font_size));
            }
        }
    }
    out
}

/// 한 글꼴의 따옴표 전진폭을 오름차순으로 모은다.
fn advances_for(font: &str, samples: &[(&str, u32)]) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    for (sample, pages) in samples {
        out.extend(
            quote_advances(sample, *pages)
                .into_iter()
                .filter(|(f, _)| f == font)
                .map(|(_, adv)| adv),
        );
    }
    out.sort_by(|a, b| a.partial_cmp(b).expect("유한값"));
    out
}

/// 고정폭 표의 따옴표는 전각으로 전진한다 — 수정 전에는 전건 0.300 em 이었다.
#[test]
fn monospace_quotes_advance_full_width() {
    for (font, samples) in [
        ("굴림체", &[(SAMPLE_ISSUE_157, 2u32)][..]),
        ("돋움체", &[(SAMPLE_1730000, 4u32)][..]),
    ] {
        let advances = advances_for(font, samples);
        assert!(
            !advances.is_empty(),
            "{font} 따옴표가 하나도 없다 — 검사 대상이 0건이면 통과 증거가 아니다"
        );
        let narrow: Vec<_> = advances.iter().filter(|adv| **adv < 0.9).collect();
        assert!(
            narrow.is_empty(),
            "{font} 따옴표는 정본대로 전각이어야 한다(수정 전 0.300). 좁은 것: {narrow:?} · 전체 {advances:?}"
        );
    }
}

/// 반례 — 비고정폭 face 는 종전 폭을 유지한다.
///
/// `156467175` 의 `(이하‘대구염색공단’)` 은 정본이 1.010 em 이지만, 같은 이름의 Type3
/// 실현은 0.24 이고 기호 슬롯 선택 축이 남아 있어 이 변경이 손대지 않는다. 폭을 함께
/// 넓히면 `issue6284` 14쪽 표가 본문 바닥을 +51.9px 넘는 것을 실측으로 확인했다.
#[test]
fn proportional_font_quotes_keep_their_previous_width() {
    let advances = advances_for("휴먼명조", &[(SAMPLE_156467175, 1)]);
    assert!(
        !advances.is_empty(),
        "휴먼명조 따옴표가 하나도 없다 — 반례 표본 전제가 깨졌다"
    );
    let widened: Vec<_> = advances.iter().filter(|adv| **adv > 0.5).collect();
    assert!(
        widened.is_empty(),
        "휴먼명조 따옴표는 이 변경의 범위 밖이라 종전 0.300 em 이어야 한다. 넓어진 것: {widened:?}"
    );
}
