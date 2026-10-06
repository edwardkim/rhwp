//! [Issue #7017] 검증된 한컴 PUA 점선 `U+F081C` 를 **측정만** 0폭으로 봐서 가운데 정렬
//! 절취선 줄이 208.8px 우측으로 밀려 용지 밖으로 나간다
//! (`samples/issue7017/2528375-science-invention-contest-receipt.hwp` 1쪽).
//!
//! 기전: 같은 코드 포인트에 정책이 셋이었다.
//!
//! ```text
//!   text_measurement.rs   폭 0 (`U+FFFC` 오브젝트 자리표시자와 같은 갈래)
//!   expand_pua_display_text   `continue` — 안 그린다
//!   hancom_pua.rs         (0xF081C, "┈") — 반각 점선으로 그린다
//! ```
//!
//! 실제 렌더는 셋째를 타 글자마다 `0.5em × 장평` 만큼 전진하는데, 정렬에 쓰인 줄 폭은
//! PUA 60자를 전부 0으로 센 84.0px 이었다. 가운데 정렬이 그 폭으로 잡은 시작 x 는
//! `75.6 + (642.5 − 84.0)/2 = 354.85` 이고, 밀림량 `(501.6 − 84.0)/2 = 208.8px` 가
//! 관측값과 소수점까지 맞는다. `hancom_pua` 모듈 계약("paint **및 폭 측정**에만
//! 투영한다")이 지켜지지 않은 상태였다.
//!
//! 수정: `U+F081C` 를 0폭 분기에서 뺀다. 다른 `hancom_pua` 괘선 조각(`F0806`
//! `F0807` `F0810` …)과 같이 폴백 0.5em 으로 떨어지고, 그 값이 `┈` 의 렌더 전진폭과
//! 같다.
//!
//! 정답지 — 한컴 2020 (`lastSavedWith` = hancom-office-2010 8.5.8.1677 → §3.5.1 2020
//! 버킷, 새로 생성한 PDF):
//!
//! ```text
//!            줄 시작 x   마지막 글리프 우단 x
//!   한컴 2020   145.4px        647.7px
//!   결함        354.85px       856.5px      ← 용지 우단 793.3px 를 63.2px 넘는다
//! ```
//!
//! 같은 수정이 `#677` 복학원서의 한/글 정본 정합도 함께 올린다 —
//! `tests/svg_snapshot.rs` 의 `issue_677_bokhakwonseo_page1` 주석 참조.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue7017/2528375-science-invention-contest-receipt.hwp";

/// 용지 우단(px). 절취선이 이 밖으로 나가면 잘려 보이지 않는다.
const PAPER_RIGHT_PX: f64 = 793.3;

#[test]
fn issue_7017_centered_pua_dotted_line_stays_on_the_paper() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let svg = core.render_page_svg_native(0).expect("page 1 svg");
    let dotted = glyph_xs(&svg, '┈');
    assert!(
        dotted.len() >= 50,
        "절취선 점선 글리프가 50개 이상이어야 한다: {}",
        dotted.len()
    );

    let first = dotted[0];
    let last = dotted[dotted.len() - 1];
    assert!(
        last < PAPER_RIGHT_PX,
        "절취선 마지막 글리프가 용지 안(< {PAPER_RIGHT_PX})이어야 한다 \
         (결함 시 856.5): {last:.1}"
    );
    // 한컴 2020 정본: 줄 시작 145.4px.
    assert!(
        (first - 145.4).abs() < 2.0,
        "절취선 시작 x 가 정본 145.4px 근처여야 한다 (결함 시 354.85): {first:.1}"
    );
}

/// SVG 에서 주어진 글자를 담은 `<text>` 들의 x 좌표(오름차순).
fn glyph_xs(svg: &str, needle: char) -> Vec<f64> {
    let mut xs: Vec<f64> = Vec::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(tag_end) = chunk.find('>') else {
            continue;
        };
        let Some(close) = chunk[tag_end + 1..].find("</text>") else {
            continue;
        };
        if !chunk[tag_end + 1..tag_end + 1 + close].contains(needle) {
            continue;
        }
        if let Some(x) = attr(&chunk[..tag_end], "x") {
            xs.push(x);
        }
    }
    xs.sort_by(|a, b| a.partial_cmp(b).expect("finite x"));
    xs
}

fn attr(head: &str, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = head.find(&needle)? + needle.len();
    let rest = &head[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}
