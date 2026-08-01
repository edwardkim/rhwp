//! Issue #3751: 쪽을 넘나드는 문단이 안 쪼개져 쪽 밖으로 나간다.
//!
//! ## 근인
//!
//! 조판은 적합 판정용 높이(`height_for_fit`)를 저장 사다리의 span 으로 상한
//! 클램프한다. 그 가드가 **첫 줄과 마지막 줄만** 비교했다.
//!
//! ```rust
//! let has_progressing_vpos =
//!     para.line_segs.len() <= 1 || last.vertical_pos > first.vertical_pos;
//! ```
//!
//! 쪽을 넘나드는 문단은 vpos 가 0 으로 돌아갔다가 다시 오른다. 끝점만 보면 "증가" 라
//! 가드를 통과하는데, 그 사이 리셋 때문에 span 은 무의미해진다.
//!
//! ```text
//! 1170000 입법역량 pi=1265
//!   ls[0] vpos=48000 … ls[6] 62400 … ls[7] vpos=0 … ls[33] 62400
//!   span = 62400 + 1200 − 48000 = 15600 HU = 208px    (실제 34줄 1088px)
//!
//! DIAG_ADV pi=1264 adv=512.0  total=512.0  h4f=496.0   ← 정상 (h4f ≈ total)
//! DIAG_ADV pi=1265 adv=1088.0 total=1088.0 h4f=208.0   ← 판정만 208
//! ```
//!
//! 잔여 236.9px 에 208px 이면 "들어간다" 이므로 통째로 얹고 실제로는 1088px 전진해
//! 863.9px 을 넘긴다. 나간 글자는 `export-text` 에도 SVG `<text>` 에도 남아 있어
//! 텍스트/IR diff 로는 잡히지 않는다([[out-of-page-glyph-blind-spot]] 계열).
//!
//! ## 계약
//!
//! 사다리 **전체가 단조 증가**일 때만 span 을 쓴다. 표본은 앞 20줄로 쪽을 채운 뒤
//! 34줄짜리 문단을 두고, 그 문단의 사다리를 9번째 줄에서 0 으로 리셋해 뒀다 —
//! 끝점만 보면 증가지만 중간이 끊긴 형태다.
//!
//! ```text
//! 수정 전  쪽 밖 글자 528  최대 초과 553.6px
//! 수정 후  쪽 밖 글자  30  최대 초과   9.6px
//! ```
//!
//! 잔여 9.6px 은 이 기전과 별개다(분할 경계 걸침). 0 을 요구하면 다른 축의 결함까지
//! 이 테스트가 지므로 두 상태 사이의 값으로 계약한다.
#![cfg(not(target_arch = "wasm32"))]

const SAMPLE: &str = "samples/issue3751/vpos_reset_midparagraph_fit.hwpx";

/// 수정 전 최대 초과 553.6px, 수정 후 9.6px — 그 사이.
const MAX_OVERFLOW_PX: f64 = 60.0;

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// SVG 한 쪽에서 (뷰박스 높이, 쪽 높이를 넘은 글자의 y) 를 뽑는다.
fn out_of_page(svg: &str) -> (f64, Vec<f64>) {
    let height = svg
        .split("height=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let mut over = Vec::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(rest) = chunk.split(" y=\"").nth(1) else {
            continue;
        };
        let Some(y) = rest.split('"').next().and_then(|s| s.parse::<f64>().ok()) else {
            continue;
        };
        if y > height + MAX_OVERFLOW_PX {
            over.push(y);
        }
    }
    (height, over)
}

/// 사다리가 중간에서 리셋되는 문단은 쪽 경계에서 쪼개진다.
#[test]
fn paragraph_with_midway_vpos_reset_is_split_at_the_page_boundary() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("파싱");
    let pages = doc.page_count();
    assert!(
        pages >= 2,
        "쪽이 {pages}개다 — 긴 문단이 쪽을 넘어야 이 경로를 탄다. 표본 확인"
    );

    let mut offenders = Vec::new();
    for p in 0..pages {
        let svg = doc.render_page_svg(p).expect("SVG 렌더");
        let (h, over) = out_of_page(&svg);
        assert!(h > 0.0, "{}쪽 뷰박스 높이를 읽지 못했다", p + 1);
        for y in over {
            offenders.push(format!("{}쪽 y={y:.1} (초과 {:.1}px)", p + 1, y - h));
        }
    }

    assert!(
        offenders.is_empty(),
        "쪽 높이를 {MAX_OVERFLOW_PX}px 넘게 벗어난 글자가 {}개 있다 (수정 전 실측 528개·최대 553.6px).\n  {}\n\
         적합 판정이 저장 사다리 span 을 끝점만 보고 쓰면, 쪽을 넘나들며 리셋되는 문단이 \
         실제 높이의 1/5 로 보여 안 쪼개진다.",
        offenders.len(),
        offenders
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// 표본이 계약을 시험하는 형태인지 못박는다 — 사다리가 중간에서 리셋되지만
/// 첫↔끝만 보면 증가여야 한다(그래야 예전 가드를 통과한다).
#[test]
fn the_fixture_ladder_resets_midway_yet_looks_progressing_at_the_endpoints() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::parser::parse_document(&bytes).expect("파싱");
    let section = doc.sections.first().expect("구역");
    let target = section
        .paragraphs
        .iter()
        .max_by_key(|p| p.line_segs.len())
        .expect("문단");

    assert!(
        target.line_segs.len() >= 30,
        "가장 긴 문단의 줄이 {}개다 — 34줄 형태가 유지돼야 한다",
        target.line_segs.len()
    );
    let first = target.line_segs.first().unwrap().vertical_pos;
    let last = target.line_segs.last().unwrap().vertical_pos;
    assert!(
        last > first,
        "끝점이 증가하지 않으면 예전 가드가 이미 걸러 이 표본이 무의미하다 \
         (first={first}, last={last})"
    );
    let has_reset = target
        .line_segs
        .windows(2)
        .any(|w| w[1].vertical_pos < w[0].vertical_pos);
    assert!(
        has_reset,
        "사다리가 중간에서 리셋되지 않는다 — 표본이 계약을 시험하지 못한다"
    );
}
