//! [#8099] KoPub 계열의 한자·전각 기호는 한글과 다른 전진폭(900/1000em, 낫표 450)이다.
//!
//! 수정 전 rhwp 는 KoPub돋움체·바탕체의 한자·○①→ 같은 전각 기호를 한글 전진폭
//! (돋움 0.872 · 바탕 0.94)으로 쟀고, 「」『』·× 는 KoPub 표 밖이라 공통 폴백으로 갔다.
//! 글꼴(`cmap`+`hmtx`)은 한자·기호·× 900, 낫표 450 이다.
//!
//! # 기대값
//!
//! 한컴 정본 `pdf/2025 행정업무운영 편람(최종)-2020-kopub.pdf`(Hwp 2020 11.0.0.9136,
//! KoPub 서브셋 내장, 383쪽)에서 KoPub 글자의 원점 차이(전진)를 **같은 줄·같은 글꼴·같은
//! 크기 한글 전진의 중앙값**으로 나눈 비다. 장평·자간이 양쪽에 같이 걸리므로 글꼴 비(900/872
//! = 1.032 · 900/936 = 0.962 · 450/872 = 0.516 · 450/936 = 0.481) 근처에 모인다.
//!
//! 기준을 쪽이 아니라 줄로 잡는 이유: 한 쪽에 장평·자간이 다른 문단이 섞이면 쪽 중앙값이 두
//! 무리 사이에 걸린다. 288쪽은 한/글이 KoPub바탕 Light 를 두 크기(한글 13.92 · 14.40px)로 내고
//! rhwp 는 한 크기로 묶어, 쪽 중앙값이 0.1px 흔들려도 비가 허용치를 넘나들었다. 57쪽 `×` 도
//! 쪽 기준으로는 다른 자간의 한글과 섞여 0.889 였고, 같은 줄 기준으로는 글꼴 비 0.962 에 맞는다.
//! 아래 정본 비는 같은 PDF 에서 같은 줄 기준으로 다시 잰 값이다(57쪽 외에는 쪽 기준과 같다).
//!
//! ```text
//!   쪽    글꼴    글자            n    정본 비
//!   38    바탕    한자             6    0.9667
//!   38    돋움    한자             8    1.0345
//!   60    바탕    ○①→ 등        350    0.9651
//!   177   돋움    ○①→ 등         25    1.0307
//!   205   돋움    「」『』          6    0.5217
//!   288   바탕    「」『』          8    0.4778
//!   57    바탕    × (U+00D7)       4    0.9638   `(×)` — ○·→ 와 같은 0.804em
//! ```
//!
//! rhwp 도 같은 줄·같은 글꼴·같은 크기의 한글 대비 비를 잰다. 절대 px 이 아니라 비라서 줄
//! 위치·쪽 배치와 무관하게 전진폭 규칙만 본다. 수정 전에는 첫 사례(38쪽 바탕 한자)부터 1.0070 이다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const TOLERANCE: f64 = 0.015;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Class {
    Hanja,
    Symbol,
    Quote,
    Times,
}

fn class(c: char) -> Option<Class> {
    match c {
        '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}' => Some(Class::Hanja),
        '「' | '」' | '『' | '』' => Some(Class::Quote),
        '×' => Some(Class::Times),
        '○' | '□' | '◇' | '◆' | '■' | '●' | '①' | '②' | '③' | '④' | '⑤' | '→' | 'ㆍ' | '［'
        | '］' | '（' | '）' | '：' => Some(Class::Symbol),
        _ => None,
    }
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).expect("유한값"));
    values[values.len() / 2]
}

/// 쪽(1-based)의 KoPub run 에서 (돋움 여부, 글자 종류) 별로 «글자 전진 / 같은 줄·글꼴·크기
/// 한글 전진 중앙값» 을 모은다. 한자는 언어 슬롯이 달라 한글과 다른 run 이 되므로, run 이
/// 아니라 같은 줄(run 의 y)·같은 글꼴 이름·글자 크기에서 한글 기준을 잡는다.
fn ratios(core: &DocumentCore, page: u32) -> BTreeMap<(bool, Class), Vec<f64>> {
    let raw = core
        .get_page_text_layout_native(page - 1)
        .expect("text-layout");
    let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON");
    // (글꼴 첫 이름, 글자 크기 ×100, 줄 y) → (한글 전진들, (돋움 여부, 종류, 전진)들)
    type Samples = (Vec<f64>, Vec<(bool, Class, f64)>);
    let mut by_face: BTreeMap<(String, i64, i64), Samples> = BTreeMap::new();
    for run in layout["runs"].as_array().into_iter().flatten() {
        let family = run["fontFamily"].as_str().unwrap_or_default();
        let primary = family
            .split(',')
            .next()
            .unwrap_or_default()
            .trim()
            .trim_matches(|c| c == '\'' || c == '"')
            .to_string();
        if !primary.contains("KoPub") {
            continue;
        }
        let dotum = primary.contains("돋움") || primary.contains("Dotum");
        let size = (run["fontSize"].as_f64().unwrap_or(0.0) * 100.0).round() as i64;
        let text: Vec<char> = run["text"].as_str().unwrap_or_default().chars().collect();
        let xs: Vec<f64> = run["charX"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(serde_json::Value::as_f64)
            .collect();
        if size <= 0 || xs.len() < text.len() + 1 {
            continue;
        }
        let line = run["y"].as_f64().unwrap_or(0.0).round() as i64;
        let entry = by_face.entry((primary, size, line)).or_default();
        for (i, &c) in text.iter().enumerate() {
            let advance = xs[i + 1] - xs[i];
            if ('\u{AC00}'..='\u{D7A3}').contains(&c) {
                entry.0.push(advance);
            } else if let Some(k) = class(c) {
                entry.1.push((dotum, k, advance));
            }
        }
    }
    let mut out: BTreeMap<(bool, Class), Vec<f64>> = BTreeMap::new();
    for (hangul, samples) in by_face.into_values() {
        if hangul.is_empty() {
            continue;
        }
        let base = median(hangul);
        for (dotum, k, advance) in samples {
            out.entry((dotum, k)).or_default().push(advance / base);
        }
    }
    out
}

/// 독립 온새미로 Print 36쪽의 연속 원점은 ㉠ 215.6800, ∼ 227.0405,
/// ㉤ 238.4009px다. 같은 KoPubWorld 프로그램의 두 글리프는 900/1000em이며,
/// 앞 원문자의 폭만 바로잡고 ∼를 반각으로 남기면 이후 본문이 당겨진다.
#[test]
fn kopub_world_similarity_and_circled_hangul_keep_equal_advances() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/[2027] 온새미로 1 본교재.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("정식 원본")).expect("로드");
    let raw = core.get_page_text_layout_native(35).expect("text-layout");
    let layout: serde_json::Value = serde_json::from_str(&raw).expect("JSON");
    let mut advances = BTreeMap::<char, Vec<f64>>::new();
    for run in layout["runs"].as_array().expect("runs") {
        if !run["fontFamily"]
            .as_str()
            .unwrap_or_default()
            .contains("KoPubWorld")
        {
            continue;
        }
        let text: Vec<char> = run["text"].as_str().expect("text").chars().collect();
        let xs = run["charX"].as_array().expect("charX");
        let size = run["fontSize"].as_f64().expect("fontSize");
        let ratio = run["ratio"].as_f64().expect("ratio");
        assert!(xs.len() > text.len() && size > 0.0 && ratio > 0.0);
        for (i, c) in text.into_iter().enumerate() {
            if matches!(c, '㉠' | '㉤' | '∼') {
                let advance = xs[i + 1].as_f64().expect("x") - xs[i].as_f64().expect("x");
                advances
                    .entry(c)
                    .or_default()
                    .push(advance / (size * ratio));
            }
        }
    }
    let circle = median(advances.remove(&'㉠').expect("㉠ 실측"));
    let similarity = median(advances.remove(&'∼').expect("∼ 실측"));
    println!("㉠={circle:.4}em, ∼={similarity:.4}em");
    assert!((circle - 0.9).abs() < 0.015, "원문자 전진폭 {circle}");
    assert!(
        (similarity / circle - 1.0).abs() < 0.015,
        "독립 Print는 같은 전진폭이나 ∼/㉠={}",
        similarity / circle
    );
}
