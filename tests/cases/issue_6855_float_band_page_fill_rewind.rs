//! [#6855] 자리차지 밴드가 채운 쪽을 "빈 쪽"으로 읽어, 저장 되감김이 지시한 쪽 경계를
//! 놓치고 다음 줄을 **용지 밖**에 그린다 — 글자 11자가 출력에서 사라진다.
//!
//! `1613000-202200037`(항공교통관제사 CBTA 체계 도입 연구) **182쪽**(0 기준 `181`):
//!
//! ```text
//!   용지 1122.5px · 본문 75.6 .. 1046.9
//!
//!   pi=0  문단        vpos  3,000    y=  115.6
//!   pi=1  29×3 표(자리차지 TopAndBottom)  y= 259.8 .. 1036.2   ← 밴드 184.3..960.6
//!   pi=2  빈 문단     vpos  7,069    ← 되감김 (10,052 → 7,069)
//!   pi=3  "과목 2: 인적 요소"  vpos 13,669
//! ```
//!
//! `#3837` 의 되감김 쪽-경계 규칙은 **"이 쪽이 이미 찼는가"**(`>= 90%`)를 함께 묻는데,
//! 그 양을 흐름 계상(`current_height`)으로만 쟀다. 자리차지 표는 흐름에 host 줄만
//! 계상하므로 이 쪽의 계상은 **118.0px** 에 머문다 — 표가 960.6px 까지 그려 놨는데도.
//! 그래서 관문이 열리지 않았고, 한/글이 쪽을 끊은 자리에서 계속 담아 `pi=3` 이
//! `y = 1168.0`, 곧 **용지 45.5px 아래**에서 시작했다.
//!
//! ```text
//!                  pi=3 위치            문서 용지 밖(off-canvas)
//!   수정 전    182쪽 y=1168.0 (용지 밖)          5건
//!   수정 후    183쪽 y=  75.6 (본문 상단)        4건
//!   정본(2020)  같은 글이 쪽 위쪽 본문 안 (y=257.6)
//! ```
//!
//! ⚠ 같은 양을 `stored_vpos_rewind_overflow_break` 까지 넓히면 안 된다 — 코퍼스
//! 10,000건 실측에서 `1480000-201600147` 의 **글자 겹침이 33 → 37** 로 는다.
//! 좁은 판은 코퍼스 전체에서 **이 문서 1건만** 바꾼다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp";

/// 이 문서의 용지 높이(px, 96dpi).
const PAPER_HEIGHT_PX: f64 = 1122.5;
/// 결함이 드러나던 쪽(0 기준) — 29×3 자리차지 표가 밴드로 채운 쪽.
const BAND_PAGE: u32 = 181;
/// 그 다음 쪽 — 한/글이 되감김으로 시작한 쪽.
const NEXT_PAGE: u32 = 182;
/// 용지 밖으로 밀려 사라지던 글.
const LOST_TEXT: &str = "과목2:인적요소";

fn open_sample() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("read sample")).expect("open sample")
}

fn page_svg(core: &DocumentCore, page: u32) -> String {
    core.render_page_svg_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 svg: {e}", page + 1))
}

/// SVG 는 글자 단위 `<text>` 로 방출된다 — 공백을 걷어내고 이어 붙인다.
fn page_text(core: &DocumentCore, page: u32) -> String {
    let svg = page_svg(core, page);
    let mut out = String::new();
    for cap in svg.split("</text>") {
        if let Some(i) = cap.rfind('>') {
            out.push_str(&cap[i + 1..]);
        }
    }
    out.chars().filter(|c| !c.is_whitespace()).collect()
}

/// 그 쪽 모든 `<text …>` 의 baseline `y`.
fn text_baselines(svg: &str) -> Vec<f64> {
    let mut out = Vec::new();
    for cap in svg.split("<text ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let Some(ys) = head.find("y=\"") else {
            continue;
        };
        let s = ys + 3;
        if let Some(e) = head[s..].find('"') {
            if let Ok(y) = head[s..s + e].parse::<f64>() {
                out.push(y);
            }
        }
    }
    out
}

#[test]
fn issue_6855_band_page_paints_nothing_below_the_paper() {
    let core = open_sample();
    let svg = page_svg(&core, BAND_PAGE);
    let baselines = text_baselines(&svg);
    assert!(
        !baselines.is_empty(),
        "{}쪽에 글자가 있어야 한다",
        BAND_PAGE + 1
    );
    let max_y = baselines.iter().copied().fold(f64::MIN, f64::max);
    assert!(
        max_y <= PAPER_HEIGHT_PX,
        "{}쪽 최대 글자 baseline({max_y:.1})이 용지({PAPER_HEIGHT_PX}) 안이어야 한다 \
         — 결함 시 `과목 2: 인적 요소` 가 1184.0 에 그려져 출력에서 사라졌다",
        BAND_PAGE + 1
    );
}

#[test]
fn issue_6855_rewound_line_starts_the_next_page() {
    let core = open_sample();
    let band_page = page_text(&core, BAND_PAGE);
    assert!(
        !band_page.contains(LOST_TEXT),
        "밴드가 채운 {}쪽에 되감긴 글이 남아 있으면 안 된다",
        BAND_PAGE + 1
    );
    let next_page = page_text(&core, NEXT_PAGE);
    assert!(
        next_page.contains(LOST_TEXT),
        "되감긴 글은 다음 쪽으로 넘어가야 한다 — engine 2020 정본도 그 글을 쪽 위쪽 \
         본문 안에 둔다\n--- {}쪽 앞부분 ---\n{}",
        NEXT_PAGE + 1,
        next_page.chars().take(60).collect::<String>()
    );
}
