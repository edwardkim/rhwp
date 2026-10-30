//! [#8315] 기호 슬롯 글자는 기호 슬롯 글꼴의 실제 전진폭으로 잰다.
//!
//! 입력 `samples/issue8136/lang-slot-ranges.hwpx`(#8136): 한글 맑은 고딕 · 영문 Arial · 기호 궁서,
//! 문단마다 `가X가`. 정본 `pdf/issue8136/lang-slot-ranges-2024.pdf`(한/글 2024 Print)에서 X 의
//! 전진폭을 같은 줄 첫 `가` 의 전진폭으로 나눈 비율을 기대값으로 둔다(절대 좌표가 아니라 관계).
//!
//! 두 갈래를 함께 본다.
//! - 폭 DB 밖 글자(그리스·키릴·첨자·통화·Letterlike·⅓·㈀·㎀): 확인한 hmtx 보충표
//!   (`font_metric_symbol_slot_chars.rs`)가 없으면 반각·전각 폴백을 받았다.
//! - `‥ …`: DB 의 전각 값(궁서 1024/1024)을 `U+2018–2027` 반각 규칙이 em/2 로 눌렀다.
//!
//! 한/글은 기호 슬롯 글꼴만 바꾼 합성 입력에서 TrueType 9종·HFT 2종 모두 이 글자들을 글꼴
//! hmtx 그대로(장치 격자 배율만 곱해) 전진시켰다 — 측정 기록은 #8315 PR 본문.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue8136/lang-slot-ranges.hwpx";

/// 문단 `XXXX 가X가` 마다 X 전진폭 / 첫 `가` 전진폭.
fn advance_ratios() -> BTreeMap<u32, f64> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let mut out = BTreeMap::new();
    for page in 0..core.page_count() {
        let raw = core
            .get_page_text_layout_native(page)
            .expect("text-layout JSON");
        let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON 해석");
        // 줄마다 (글자, 원점 x) 를 run 의 charX 로 펼친다.
        let mut lines: BTreeMap<i64, Vec<(f64, char)>> = BTreeMap::new();
        for run in layout["runs"].as_array().into_iter().flatten() {
            let x0 = run["x"].as_f64().unwrap_or_default();
            let y = run["y"].as_f64().unwrap_or_default().round() as i64;
            let xs = run["charX"].as_array().cloned().unwrap_or_default();
            for (i, c) in run["text"].as_str().unwrap_or_default().chars().enumerate() {
                let dx = xs.get(i).and_then(|v| v.as_f64()).unwrap_or_default();
                lines.entry(y).or_default().push((x0 + dx, c));
            }
        }
        for mut chars in lines.into_values() {
            chars.sort_by(|a, b| a.0.total_cmp(&b.0));
            let text: String = chars.iter().map(|c| c.1).collect();
            let Some(code) = text.get(..4).and_then(|h| u32::from_str_radix(h, 16).ok()) else {
                continue;
            };
            let Some(i) = chars
                .iter()
                .skip(4)
                .position(|c| c.1 == '가')
                .map(|i| i + 4)
            else {
                continue;
            };
            if i + 2 < chars.len() {
                let hangul = chars[i + 1].0 - chars[i].0;
                let symbol = chars[i + 2].0 - chars[i + 1].0;
                out.insert(code, symbol / hangul);
            }
        }
    }
    out
}

fn assert_ratios(cases: &[(u32, f64)]) {
    let got = advance_ratios();
    for &(code, want) in cases {
        let r = *got.get(&code).unwrap_or_else(|| {
            panic!("{code:04X} 문단을 찾지 못했다 — 검사 대상 0건은 통과 증거가 아니다")
        });
        assert!(
            (r - want).abs() <= 0.03,
            "{code:04X} {} 전진폭 비율 {r:.3} ≠ 한/글 Print {want:.3}",
            char::from_u32(code).unwrap()
        );
    }
}

#[test]
fn symbol_slot_characters_outside_the_metric_db_use_the_font_advance() {
    // (코드, 한/글 Print 의 X/가 전진폭 비율)
    assert_ratios(&[
        (0x0391, 0.711),
        (0x03C9, 0.783),
        (0x0401, 0.711),
        (0x0410, 0.711),
        (0x0451, 0.627),
        (0x2074, 1.012),
        (0x2081, 1.012),
        (0x20A9, 0.738),
        (0x20AC, 0.702),
        (0x2116, 0.964),
        (0x2122, 1.012),
        (0x2153, 1.012),
        (0x215B, 0.488),
        (0x3200, 1.012),
        (0x3260, 1.012),
        (0x3380, 1.012),
    ]);
}

#[test]
fn fullwidth_leaders_in_the_symbol_slot_are_not_halved() {
    assert_ratios(&[(0x2025, 1.012), (0x2026, 1.012)]);
}

#[test]
fn symbol_slot_characters_already_in_the_metric_db_stay_as_measured() {
    // 대조군: 종전에도 정본과 맞던 글자 — 보충표·반각 규칙 변경이 건드리지 않는다.
    assert_ratios(&[
        (0x2013, 0.506),
        (0x2014, 1.012),
        (0x2020, 0.566),
        (0x2022, 0.386),
        (0x2032, 0.337),
        (0x2033, 0.530),
        (0x203B, 0.988),
        (0x2044, 0.422),
    ]);
}
