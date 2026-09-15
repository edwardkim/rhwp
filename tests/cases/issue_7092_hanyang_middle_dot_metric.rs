//! [#7092] 한양중고딕의 `·`(U+00B7) 메트릭이 남의 글꼴 값을 빌려 쓴다.
//!
//! ## 무엇이 문제인가
//!
//! `font_metrics_overlays.rs` 의 한양 항목은 자기 Basic Latin(`0x0020-0x007E`)과 한글만
//! 실측하고, `0x00A0-0x00FF` 구간은 **HY 항목에서 빌려 쓴다.**
//!
//! ```text
//!   HanyangJungGothic  0x00A0-0x00FF  ->  FONT_267_LATIN_1 = HYGothic-Medium
//!                                          (윈도우 H2GTRM.TTF, U+00B7 = 1024/1024 = 전각)
//! ```
//!
//! 한/글은 한양 계열을 **자기 글꼴**로 그린다 — 정본 PDF 에서 그 글자는 TrueType 자원이
//! 아니라 Type3 자원으로 나오고, 그 `/W` 는 전각이 아니라 **0.381 em** 이다.
//!
//! ## 정본 실측 — 문서 둘이 독립적으로 같은 값
//!
//! 두 문서 모두 `·` 런의 글꼴을 rhwp 가 `한양중고딕` 으로 풀고, 같은 줄이 정본에서
//! Type3 로 그려진다.
//!
//! ```text
//!   samples/issue1891/76076_regulatory_analysis-2024.pdf
//!     p18  '가피하게덮개·울을개방하고'   /W 0.381  전진 0.391
//!     p18  '로덮개·울등을설치하'         /W 0.381  전진 0.380
//!   pdf/80168_regulatory_analysis-2022.pdf (2024·hwp-2024 판본도 동일)
//!     Type3 n=54                        /W 0.381  전진 중앙 0.386
//! ```
//!
//! 같은 문서의 `휴먼명조` 는 정본에서 TrueType(`INPILL+휴먼명조`)으로 나가고 `/W` 가
//! 1.001 em 이다 — 빌려 온 전각값이 그쪽에는 맞는다. 그래서 이 정정은 **U+00B7 한 글자,
//! 한양중고딕 한 글꼴**로 한정한다.
//!
//! ## 무엇이 걸려 있었나
//!
//! 빌려 온 전각값 때문에 `text_measurement` 의 `.notdef` 좁힘(0.300 em)이 발동해 왔다.
//! 0.300 은 정본 0.381 과 0.08 em 어긋나고, 그 오차가 쌓여 `76076` p18 의 조각 경계를
//! 21.8px 여유 위에서 흔든다(`issue_3820_p18_p19` 계약). 메트릭을 제 값으로 주면 좁힘이
//! 애초에 발동하지 않는다(390 < 1024).

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 정본 `samples/issue1891/76076_regulatory_analysis-2024.pdf` 가 이 문서의 정답지다.
const SAMPLE_76076: &str = "samples/76076_regulatory_analysis.hwp";
/// 정본 `pdf/80168_regulatory_analysis-2022.pdf` (2024·hwp-2024 판본 동일).
const SAMPLE_80168: &str = "samples/80168_regulatory_analysis.hwp";
/// 한양신명조 앵커. `·` 38개가 **전부** 이 글꼴이고, 정본
/// `samples/21868765_별표2_보건소_분장사무.pdf` 의 Type3 `·` 도 정확히 38개다 — 1:1.
const SAMPLE_21868765: &str = "samples/21868765_별표2_보건소_분장사무.hwp";
/// 같은 글꼴의 둘째 앵커. 정본 `pdf/task2097/21298295_byeolpyo5_disaster-hwp-2020.pdf`.
const SAMPLE_21298295: &str = "samples/task2097/21298295_byeolpyo5_disaster.hwp";

/// 한 문서에서 `·` 의 (글꼴, 글자크기 대비 전진폭) 을 모은다. 연속 `·`(목차 점 채움)은 뺀다.
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
                let leader = (i > 0 && text[i - 1] == '\u{00B7}')
                    || (i + 1 < text.len() && text[i + 1] == '\u{00B7}');
                if leader {
                    continue;
                }
                out.push((family.clone(), (char_x[i + 1] - char_x[i]) / font_size));
            }
        }
    }
    out
}

/// 한 글꼴의 `·` 전진폭을 오름차순으로 모은다.
fn advances_for(font: &str, samples: &[(&str, u32)]) -> Vec<f64> {
    let mut out: Vec<f64> = Vec::new();
    for (sample, pages) in samples {
        out.extend(
            middle_dot_advances(sample, *pages)
                .into_iter()
                .filter(|(f, _)| f == font)
                .map(|(_, adv)| adv),
        );
    }
    out.sort_by(|a, b| a.partial_cmp(b).expect("유한값"));
    out
}

/// 한양중고딕의 `·` 는 자기 메트릭(390/1024 = 0.3809 em)대로 전진한다.
///
/// 수정 전에는 빌려 온 전각 메트릭 때문에 `.notdef` 좁힘이 걸려 **전건 0.300 em** 이었다.
///
/// 자간·양쪽정렬 분배·HWP px 양자화가 개별 값을 흔들어(관측 0.3295~0.4082) 낱개를 좁게
/// 잠그면 부서진다. 그래서 **중앙값**으로 본체를 잠그고, 낱개는 좁힘 값(0.300)과 갈리는
/// 선에서만 잠근다. 두 단언 모두 수정 전에 깨진다.
#[test]
fn hanyang_junggothic_middle_dot_matches_its_own_metric() {
    let hanyang = advances_for("한양중고딕", &[(SAMPLE_76076, 24), (SAMPLE_80168, 24)]);
    assert!(
        hanyang.len() >= 5,
        "한양중고딕 `·` 를 5개 이상 봐야 한다 — 검사 대상이 0건이면 통과 증거가 아니다. got {}",
        hanyang.len()
    );
    let median = hanyang[hanyang.len() / 2];
    assert!(
        (0.36..=0.40).contains(&median),
        "한양중고딕 `·` 전진 중앙값은 자기 메트릭 0.3809 em 이어야 한다(수정 전 0.300). got {median:.4} · 전체 {hanyang:?}"
    );
    let off: Vec<_> = hanyang
        .iter()
        .filter(|adv| !(0.31..=0.45).contains(*adv))
        .collect();
    assert!(
        off.is_empty(),
        "좁힘 값(0.300)과 갈리는 선을 벗어난 것: {off:?} (전체 {}개)",
        hanyang.len()
    );
}

/// 한양중고딕 **말고는** 아무것도 바꾸지 않는다 — 좁힘이 걸리던 글꼴은 그대로다.
///
/// 같은 문서의 `휴먼명조` 는 정본에서 TrueType 으로 1.001 em 이지만, 그 축(#7092 본체)은
/// 별개 판정이 필요해 이 변경의 범위 밖이다. 여기서는 **종전 값(0.300 em)이 유지되는지**만
/// 잠가, 이 정정이 한 글꼴을 넘지 않았음을 확인한다.
#[test]
fn other_fonts_keep_their_previous_middle_dot_width() {
    let human = advances_for("휴먼명조", &[(SAMPLE_80168, 24)]);
    assert!(
        !human.is_empty(),
        "휴먼명조 `·` 가 하나도 없다 — 표본 전제가 깨졌다"
    );
    let moved: Vec<_> = human
        .iter()
        .filter(|adv| !(0.26..=0.34).contains(*adv))
        .collect();
    assert!(
        moved.is_empty(),
        "휴먼명조 `·` 는 이 변경의 범위 밖이라 종전 0.300 em 이어야 한다. 움직인 것: {moved:?}"
    );
}

/// 한양신명조의 `·` 는 자기 메트릭(393/1024 = 0.3838 em)대로 전진한다.
///
/// 한양중고딕과 **같은 갈래, 다른 값**이다. 이 글꼴은 `0x00A0-0x00FF` 를
/// `FONT_276_LATIN_1`(= `HYSinMyeongJo-Medium`, 윈도우 `H2MJSM.TTF`)에서 빌려 왔고 그
/// 값은 전각이다. 수정 전에는 그 전각값 때문에 `.notdef` 좁힘이 걸려 0.300 em 이었다.
///
/// ```text
///   samples/21868765_별표2_보건소_분장사무.pdf         Type3 n=38  /W 0.3842
///   pdf/task2097/21298295_byeolpyo5_disaster-hwp-2020.pdf
///                                                   Type3 n=19  /W 0.3840
/// ```
#[test]
fn hanyang_sinmyeongjo_middle_dot_matches_its_own_metric() {
    let hanyang = advances_for(
        "한양신명조",
        &[(SAMPLE_21868765, 24), (SAMPLE_21298295, 24)],
    );
    assert!(
        hanyang.len() >= 20,
        "한양신명조 `·` 를 20개 이상 봐야 한다 — 검사 대상이 0건이면 통과 증거가 아니다. got {}",
        hanyang.len()
    );
    let median = hanyang[hanyang.len() / 2];
    assert!(
        (0.36..=0.41).contains(&median),
        "한양신명조 `·` 전진 중앙값은 자기 메트릭 0.3838 em 이어야 한다(수정 전 0.300). got {median:.4} · 전체 {hanyang:?}"
    );
    let off: Vec<_> = hanyang
        .iter()
        .filter(|adv| !(0.31..=0.45).contains(*adv))
        .collect();
    assert!(
        off.is_empty(),
        "좁힘 값(0.300)과 갈리는 선을 벗어난 것: {off:?} (전체 {}개)",
        hanyang.len()
    );
}

/// 한양견명조·한양견고딕은 **손대지 않는다** — 저장소에 근거가 없다.
///
/// `samples/` 1,082건 전수에서 `·` 런을 그 두 글꼴로 푸는 문서가 **0개**다(한양중고딕 51
/// 문서 637회 · 한양신명조 16문서 310회). 값을 정할 실측도 없고, 바꿔도 보이는 곳이 없다.
/// 그 둘이 `·` 를 담은 문서가 표본에 들어오면 그때 같은 자로 재야 한다.
#[test]
fn kyun_families_have_no_anchored_evidence_in_the_corpus() {
    let mut seen = 0usize;
    for sample in [SAMPLE_76076, SAMPLE_80168, SAMPLE_21868765, SAMPLE_21298295] {
        for (font, _) in middle_dot_advances(sample, 24) {
            if font == "한양견명조" || font == "한양견고딕" {
                seen += 1;
            }
        }
    }
    assert_eq!(
        seen, 0,
        "견 계열이 `·` 를 담은 표본이 생겼다 — 정본으로 값을 재서 이 시험과 오버레이를 함께 갱신해라"
    );
}
