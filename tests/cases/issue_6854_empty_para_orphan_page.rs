//! [#6854] 잉크 없는 빈 문단 하나가 쪽을 통째로 차지해 **꼬리말만 있는 빈 쪽**이 생기고,
//! 그 뒤의 모든 쪽이 한 칸씩 밀린다.
//!
//! `70833`(전기안전관리법 시행규칙 규제영향분석서) `pi=84` 는 글자도 컨트롤도 없는 한 줄
//! 문단인데, 쪽 예산을 **2.3px**(한 줄 미만) 넘겨 다음 쪽으로 밀린다. 바로 다음 문단이
//! **문서가 선언한 쪽나누기**(`ColumnBreakType::Page`)라 그 쪽에는 더 들어올 것이 없다 —
//! 결과가 꼬리말 `- 14 -` 만 있는 14쪽이다.
//!
//! ```text
//!                     쪽수   본문 없는 쪽   engine 2020 정본
//!   70833  수정 전     19       14쪽            18쪽
//!          수정 후     18       없음            18쪽
//!   22037757 수정 전   16       10쪽            15쪽
//!          수정 후     15       없음            15쪽
//! ```
//!
//! `Task #1537` 이 같은 모양의 고아 쪽을 이미 막고 있었지만 **"글자 있는 여러 줄 문단"**
//! 만 인정했다. 빈 문단은 하단 여백으로 흘려도 **그려지는 것이 없으므로** 그 완화가
//! 걱정하던 bleed 가 성립하지 않는다.
//!
//! ⚠ 잉크 없는 경우에는 **선언된** 쪽나누기만 인정하고, 초과 상한("한 줄 미만")도 그대로
//! 둔다. 둘 다 코퍼스 10,000건 A/B 로 지탱을 확인했다 — 추론된 경계까지 넓히면 쪽 이득
//! 없는 넘침이 7건 늘고, 상한을 없애면 `1342000-202200027` 이 용지 밖 5 → 7 로
//! 악화한다(`samples/issue6854/README.md`).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 양성 ① — 규제영향분석서(HWP5, 2020 저장). engine 2020 정본 18쪽.
const SAMPLE_HWP: &str = "samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp";
/// 양성 ② — 인사 규칙 별표(HWPX, 2022 저장). engine 2020 정본 15쪽.
const SAMPLE_HWPX: &str = "samples/issue6854/22037757-chuncheon-personnel-rule-annex13.hwpx";

fn open(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}")))
        .unwrap_or_else(|e| panic!("open {sample}: {e}"))
}

/// SVG 는 글자 단위 `<text>` 로 방출된다 — 순서대로 이어 붙인다.
fn svg_text(svg: &str) -> String {
    let mut out = String::new();
    for cap in svg.split("</text>") {
        if let Some(i) = cap.rfind('>') {
            out.push_str(&cap[i + 1..]);
        }
    }
    out
}

/// 본문 글자가 없는 쪽(0 기준).
///
/// 공백과 `-` 를 걷어낸 뒤 아무것도 안 남거나(쪽번호가 없는 문서) 남는 것이 숫자뿐이면
/// (`- 14 -`) 그 쪽에는 본문이 없다.
fn pages_without_body_text(core: &DocumentCore) -> Vec<u32> {
    let page_count = u32::try_from(core.page_count()).expect("page count fits u32");
    (0..page_count)
        .filter(|page| {
            let svg = core
                .render_page_svg_native(*page)
                .unwrap_or_else(|e| panic!("{}쪽 svg: {e}", page + 1));
            let text: String = svg_text(&svg)
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '-')
                .collect();
            text.chars().all(|c| c.is_ascii_digit()) && text.chars().count() <= 3
        })
        .collect()
}

#[test]
fn issue_6854_hwp_sample_has_no_body_less_page() {
    let core = open(SAMPLE_HWP);
    let orphans = pages_without_body_text(&core);
    assert!(
        orphans.is_empty(),
        "잉크 없는 빈 문단이 쪽을 통째로 가지면 안 된다 — 수정 전 0기준 13(14쪽)이 \
         꼬리말 `- 14 -` 만 담았다. 실측 {orphans:?}"
    );
}

#[test]
fn issue_6854_hwp_sample_matches_the_2020_oracle_page_count() {
    let core = open(SAMPLE_HWP);
    assert_eq!(
        core.page_count(),
        18,
        "engine 2020 정본과 같은 쪽수여야 한다 — 고아 쪽이 살아 있으면 19쪽 \
         (pdf/70833-electrical-safety-rule-regulatory-analysis-2020.pdf)"
    );
}

#[test]
fn issue_6854_hwpx_sample_has_no_body_less_page() {
    let core = open(SAMPLE_HWPX);
    let orphans = pages_without_body_text(&core);
    assert!(
        orphans.is_empty(),
        "표 별표 문서에서도 같은 고아 쪽이 생기면 안 된다 — 수정 전 0기준 9(10쪽)가 \
         완전히 비었다(이 문서는 쪽번호도 없다). 실측 {orphans:?}"
    );
}

#[test]
fn issue_6854_hwpx_sample_matches_the_2020_oracle_page_count() {
    let core = open(SAMPLE_HWPX);
    assert_eq!(
        core.page_count(),
        15,
        "engine 2020 정본과 같은 쪽수여야 한다 — 고아 쪽이 살아 있으면 16쪽 \
         (pdf/22037757-chuncheon-personnel-rule-annex13-2020.pdf)"
    );
}
