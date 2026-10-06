//! [#7092] 실측 검증 face 의 가운뎃점 `·` 은 글꼴이 지닌 전각 폭으로 전진한다.
//!
//! # 무엇이 깨져 있었나
//!
//! `·`(U+00B7)이 `em` 폭으로 적힌 메트릭을 `.notdef` 상자로 오인해 `0.3em` 으로 덮어썼다.
//! 그 가정은 **글리프가 없는 글꼴**에만 맞는다. `HY헤드라인M` 은 글꼴 파일이 이 글자를
//! 실제로 갖고 있다.
//!
//! ```text
//!   ttfs/hwp/H2HDRM.TTF   upm=1024
//!     U+00B7 periodcentered  advance=1024 = 1.000 em
//!     윤곽선 bbox (439, 297, 586, 435)      ← 빈 .notdef 상자가 아니다
//! ```
//!
//! # 독립 기대값 — 저장소 한/글 정본 25개 문서
//!
//! `pdf/**` 의 한컴 PDF 에서 이 face 의 `·` 전진폭(같은 줄 다음 글자와의 차 ÷ 글자크기)을
//! 재면 좁은 갈래가 한 건도 없다.
//!
//! ```text
//!   k-water-rfp 5종 · mel-001 3종 · aift-2022 · pr_6528_issue6181_p5 …
//!   n≈50   중앙값 1.000   범위 0.89 ~ 1.06   (0.2~0.4em 구간 0건)
//! ```
//!
//! 이 시험은 `samples/mel-001.hwp` 로 잠근다 — 정본 `pdf/mel-001-hwp-2020.pdf` 가
//! 이 문서의 `·` 를 1.000em 으로 그린다.
//!
//! # 반례
//!
//! `휴먼명조` 는 정본이 전각인데도 **아직 좁힌다** — 메트릭 표의 Latin-1 구간이 `0`/`em`
//! 두 값뿐이라(`latin1_table_is_uninformative`) 표를 근거로 쓸 수 없기 때문이다. 표를
//! 고치는 다른 갈래이므로, 여기서는 그 face 가 움직이지 않는 것까지 함께 잠근다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// `HY헤드라인M` 앵커. 정본 `pdf/mel-001-hwp-2020.pdf`.
const SAMPLE_MEL: &str = "samples/mel-001.hwp";
/// `휴먼명조` 반례. 같은 문서의 정본 `pdf/k-water-rfp-hwp-2020.pdf`.
const SAMPLE_HUMAN: &str = "samples/k-water-rfp.hwp";

/// 한 문서에서 `·` 의 (글꼴, 글자크기 대비 전진폭) 을 모은다.
fn middle_dot_advances(sample: &str, max_pages: u32) -> Vec<(String, f64)> {
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
                if c != '\u{00B7}' {
                    continue;
                }
                out.push((family.clone(), (char_x[i + 1] - char_x[i]) / font_size));
            }
        }
    }
    out
}

fn advances_for(sample: &str, pages: u32, font: &str) -> Vec<f64> {
    let mut out: Vec<f64> = middle_dot_advances(sample, pages)
        .into_iter()
        .filter(|(f, _)| f.split(',').next().unwrap_or(f).trim_matches('\'') == font)
        .map(|(_, adv)| adv)
        .collect();
    out.sort_by(|a, b| a.partial_cmp(b).expect("유한값"));
    out
}

/// 검증 face 의 `·` 는 글꼴이 지닌 전각 폭으로 전진한다.
#[test]
fn verified_face_middle_dot_keeps_the_written_full_width() {
    let seen = middle_dot_advances(SAMPLE_MEL, 24);
    let advances = advances_for(SAMPLE_MEL, 24, "HY헤드라인M");
    assert!(
        !advances.is_empty(),
        "mel-001 에서 HY헤드라인M 가운뎃점을 찾지 못했다 — 관측: {seen:?}"
    );
    for adv in &advances {
        assert!(
            (adv - 1.0).abs() < 0.05,
            "HY헤드라인M `·` 은 정본대로 1.000em 이어야 한다(수정 전 0.300) — got {adv:.3}, 전체 {advances:?}"
        );
    }
}

/// 반례 — 메트릭 표가 폭 정보를 담지 못한 face 는 종전 폭 그대로다.
#[test]
fn uninformative_metric_face_middle_dot_stays_narrow() {
    let advances = advances_for(SAMPLE_HUMAN, 24, "휴먼명조");
    assert!(
        !advances.is_empty(),
        "반례 문서에서 휴먼명조 가운뎃점을 찾지 못했다"
    );
    // 좁힘은 `0.3em` 오버레이지만 실제 전진에는 자간·장평이 섞여 0.27~0.33 으로 관측된다.
    // 이 반례가 잠그는 것은 "전각으로 넓어지지 않는다"이다.
    for adv in &advances {
        assert!(
            *adv < 0.5,
            "휴먼명조 `·` 은 이 변경의 범위 밖이라 좁은 채로 있어야 한다 — got {adv:.3}, 전체 {advances:?}"
        );
    }
}
