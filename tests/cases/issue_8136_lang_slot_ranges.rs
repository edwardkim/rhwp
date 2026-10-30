//! [#8136] 한/글이 앞 글자 언어와 무관하게 기호·영문 슬롯 글꼴로 그리는 글자.
//!
//! 입력 `samples/issue8136/lang-slot-ranges.hwpx` 는 `samples/lang_slot/gen_lang_slot_probe.py` 로
//! 만든 합성 문서다. 글자 모양 하나에 일곱 슬롯마다 다른 설치 글꼴을 준다 — 한글 맑은 고딕 ·
//! 영문 Arial · 한자 바탕 · 일본어 Times New Roman · 기타 돋움 · 기호 궁서 · 사용자 HY헤드라인M.
//! 문단마다 `가X가` 를 적고, 저장 줄은 없다.
//!
//! 정본 `pdf/issue8136/lang-slot-ranges-2024.pdf`(한/글 2024 13.0.0.564 Print)에서 X 의 글꼴:
//! - 궁서(기호 슬롯): 그리스·키릴, – — † • ‥ … ‰ ′ ″ ※ ‾ ⁄ ⁴ ₁ ₩ € ℃ № ™ ⅓ ⅛ ㈀ ㉠ ㉿ ㎀ ㏊
//! - Arial(영문 슬롯): § ± ´ ¶ ¼ ¿ ‚ „
//! - 어느 슬롯 글꼴에도 글리프가 없어 시스템 대체로 그려진 ‐ ‖ ⁰ ₀ ก 는 슬롯을 가를 수 없다.
//!   이 글자들은 분류를 바꾸지 않았으므로, 앞뒤 한글을 따라 한글 슬롯(맑은 고딕)에 남는지 본다.
//!
//! 근거 측정은 같은 생성기로 후보 블록 908자를 찍고, 기호·영문 슬롯 글꼴만 바꾼 입력으로
//! 글꼴이 슬롯을 따라가는지 대조했다(#8136 PR 본문).

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue8136/lang-slot-ranges.hwpx";

/// 문단 `XXXX 가X가` 의 X(코드 다음 첫 `가` 뒤 글자)를 담은 run 의 첫 글꼴 이름.
fn probe_faces() -> BTreeMap<u32, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core =
        DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드");
    let mut out = BTreeMap::new();
    for page in 0..core.page_count() {
        let raw = core
            .get_page_text_layout_native(page)
            .expect("text-layout JSON");
        let layout: serde_json::Value = serde_json::from_str(&raw).expect("text-layout JSON 해석");
        // 줄마다 run 을 이어 `XXXX 가X가` 를 복원하고, 각 글자의 run 글꼴을 함께 둔다.
        let mut lines: BTreeMap<i64, Vec<(f64, char, String)>> = BTreeMap::new();
        for run in layout["runs"].as_array().into_iter().flatten() {
            let family = run["fontFamily"].as_str().unwrap_or_default();
            let primary = family
                .split(',')
                .next()
                .unwrap_or_default()
                .trim()
                .trim_matches(['\'', '"'])
                .to_string();
            let y = run["y"].as_f64().unwrap_or_default().round() as i64;
            let x = run["x"].as_f64().unwrap_or_default();
            for (i, c) in run["text"].as_str().unwrap_or_default().chars().enumerate() {
                lines
                    .entry(y)
                    .or_default()
                    .push((x + i as f64 * 1e-3, c, primary.clone()));
            }
        }
        for mut chars in lines.into_values() {
            chars.sort_by(|a, b| a.0.total_cmp(&b.0));
            let text: String = chars.iter().map(|c| c.1).collect();
            let Some(code) = text.get(..4).and_then(|h| u32::from_str_radix(h, 16).ok()) else {
                continue;
            };
            if let Some(i) = chars.iter().skip(4).position(|c| c.1 == '가') {
                if let Some(x) = chars.get(4 + i + 1) {
                    out.insert(code, x.2.clone());
                }
            }
        }
    }
    out
}

fn assert_faces(faces: &BTreeMap<u32, String>, codes: &[u32], want: &str) {
    for code in codes {
        let got = faces.get(code).unwrap_or_else(|| {
            panic!("{code:04X} 문단을 찾지 못했다 — 검사 대상 0건은 통과 증거가 아니다")
        });
        assert_eq!(got, want, "{code:04X} {}", char::from_u32(*code).unwrap());
    }
}

#[test]
fn symbol_slot_characters_use_the_symbol_slot_face() {
    let faces = probe_faces();
    assert_faces(
        &faces,
        &[
            0x0391, 0x03C9, 0x0401, 0x0410, 0x0451, 0x2013, 0x2014, 0x2020, 0x2022, 0x2025, 0x2026,
            0x2030, 0x2032, 0x2033, 0x203B, 0x203E, 0x2044, 0x2074, 0x2081, 0x20A9, 0x20AC, 0x2103,
            0x2116, 0x2122, 0x2153, 0x215B, 0x3200, 0x3260, 0x327F, 0x3380, 0x33CA,
        ],
        "궁서",
    );
}

#[test]
fn latin1_symbols_and_low_quotes_use_the_latin_slot_face() {
    let faces = probe_faces();
    assert_faces(
        &faces,
        &[
            0x00A7, 0x00B1, 0x00B4, 0x00B6, 0x00BC, 0x00BF, 0x201A, 0x201E,
        ],
        "Arial",
    );
}

#[test]
fn undecided_characters_and_controls_keep_their_slot() {
    let faces = probe_faces();
    // 한/글도 시스템 대체로 그려 슬롯을 가를 수 없는 글자 — 분류를 옮기지 않았다.
    assert_faces(
        &faces,
        &[0x2010, 0x2016, 0x2070, 0x2080, 0x0E01],
        "맑은 고딕",
    );
    // 대조군: 한글 호환 자모는 한글 슬롯, 화살표·전각 느낌표는 원래 기호 슬롯.
    assert_faces(&faces, &[0x3131], "맑은 고딕");
    assert_faces(&faces, &[0x2192, 0xFF01], "궁서");
}
