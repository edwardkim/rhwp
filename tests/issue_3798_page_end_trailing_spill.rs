//! Issue #3798: 쪽 끝 문단이 말미 줄간격만큼 쪽을 넘겨 얹힌다.
//!
//! ## 근인
//!
//! 쪽 마지막 문단의 적합 판정은 말미 줄간격을 뺀 높이(`height_for_fit`)로 한다
//! [Task #359]. 한글도 쪽 하단을 조금 넘겨 채우므로 방향은 맞다. 그런데 그 트림에
//! **한도가 없다.**
//!
//! ```rust
//! fn paragraph_page_end_fit_height(total, height_for_fit, require_full) -> f64 {
//!     if require_full { total.max(height_for_fit) } else { height_for_fit }
//! }
//! ```
//!
//! 그래서 rhwp 는 말미 줄간격 **전량**(모집단 실측 최대 33.7px)을 쪽 밖으로
//! 흘려보내고, 한글이 다음 쪽으로 넘긴 문단을 현재 쪽에 얹는다.
//!
//! ```text
//! PI_MISMATCH n=1 코호트 66건 (쪽 총수는 맞는데 문단 하나만 어긋난 문서)
//!   그 문단을 들여보낸 경로:  h4f-trim 25 · 분할/이동 24 · plain 17
//!   경계의 slack 중앙값 9.7px — 면도날이다
//! ```
//!
//! 후보 수정은 트림에 **한도**를 두는 것이었다(#2137 의 TAC 개체 40px 스필 한도와 같은
//! 관용구). 접었다 — 한글 정답지 테스트가 초록인 창(12~16px)에서 순이득이 음수다.
//! 측정·반증은 `mydocs/report/task3798/README.md`.
//!
//! ## 계약
//!
//! 표본은 앞 28줄(각 32.0px, 누적 896.0px)로 쪽을 채운 뒤, 줄 16.0px 에 말미 간격
//! 40.0px 을 단 문단을 둔다. 본문은 933.6px 이다.
//!
//! ```text
//! 현행     트림 후 16.0 -> 896.0 + 16.0 = 912.0 <= 933.6  이라 1쪽에 얹힌다
//!          실제 소비는 896.0 + 56.0 = 952.0 으로 18.4px 넘친다
//! 한도 C   적합 높이 56.0-C -> 952.0-C > 933.6  즉 C < 18.4 여야 2쪽으로 간다
//! ```
//!
//! 그래서 이 표본은 한도 18px 이하만 시험한다. 한글 정답지 테스트가 요구하는 하한이
//! 12px 이므로 시험 가능한 창은 12~18px 이고, 그 안에서 이득이 사라진다.
#![cfg(not(target_arch = "wasm32"))]

const SAMPLE: &str = "samples/issue3798/page_end_trailing_spill.hwpx";
/// 표본 경계 문단의 말미 줄간격(px). 한도보다 훨씬 커야 계약을 시험한다.
const FIXTURE_TRAILING_SPACING_PX: f64 = 40.0;
/// 경계 문단을 알아보는 글월.
const BOUNDARY: &str = "경계문단";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// 그 쪽의 글자를 이어 붙인다. SVG 는 글자마다 `<text>` 를 내므로 통째로 훑으면
/// 여러 글자짜리 글월을 찾지 못한다.
fn page_text(doc: &rhwp::wasm_api::HwpDocument, page: u32) -> String {
    let svg = doc.render_page_svg(page).expect("SVG 렌더");
    let mut out = String::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(rest) = chunk.split_once('>') else {
            continue;
        };
        if let Some((body, _)) = rest.1.split_once("</text>") {
            out.push_str(body);
        }
    }
    out
}

/// 그 쪽에 이 글월이 있는가.
fn page_contains(doc: &rhwp::wasm_api::HwpDocument, page: u32, needle: &str) -> bool {
    page_text(doc, page).contains(needle)
}

/// 말미 줄간격이 한도보다 크면 그 문단은 다음 쪽에서 시작한다.
///
/// 이 결함은 **글자가 종이 밖으로 나가지 않는다** — 넘치는 것은 글리프가 없는 말미
/// 간격이라 쪽 밖 글자 판별로는 침묵한다. 드러나는 곳은 배치뿐이고, 그래서 한글
/// 오라클에서도 쪽수는 맞는데 문단만 어긋나는(PI_MISMATCH) 모양으로 나타난다.
/// **미해결 — 수정을 제출하지 않았다.** 상수 한도 형태는 안전한 창(12~16px) 안에서
/// 순이득이 음수였다(문단 배치 +2 · 쪽수 −1). 측정과 반증은
/// `mydocs/report/task3798/README.md` 에 있고, 실험 패치는 같은 자리의
/// `trimcap_experiment.patch` 다. 이 계약은 다음에 이 축을 여는 사람이 표본을 다시
/// 만들지 않도록 남긴다.
#[ignore = "#3798 미해결 — 한도 형태는 순이득 음수로 접었다 (mydocs/report/task3798/)"]
#[test]
fn a_paragraph_whose_trailing_spacing_exceeds_the_spill_limit_moves_to_the_next_page() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("파싱");
    assert!(doc.page_count() >= 2, "쪽이 {}개다", doc.page_count());

    assert!(
        page_contains(&doc, 0, "앞채움28"),
        "앞 채움 28번째 줄이 1쪽에 없다 — 표본이 쪽을 계획대로 채우지 못했다. \n         이 문단까지가 1쪽 몫이다(누적 896.0px, 본문 933.6px)."
    );
    assert!(
        !page_contains(&doc, 0, BOUNDARY),
        "경계 문단이 1쪽에 얹혔다. 트림 후 높이 16.0 으로 재면 \n         896.0 + 16.0 = 912.0 <= 933.6 이라 들어가지만, 실제 소비는 \n         896.0 + 56.0 = 952.0 으로 본문을 18.4px 넘긴다. \n         말미 줄간격 트림에 한도가 없어서 생긴다."
    );
    assert!(
        page_contains(&doc, 1, BOUNDARY),
        "경계 문단이 2쪽에도 없다 — 표본이나 계약이 어긋났다"
    );
}

/// 표본이 계약을 시험하는 형태인지 못박는다 — 말미 줄간격이 한도보다 커야 한다.
#[test]
fn the_fixture_has_a_trailing_spacing_larger_than_the_spill_limit() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::parser::parse_document(&bytes).expect("파싱");
    let section = doc.sections.first().expect("구역");

    // 1 px = 75 HWPUNIT (96 dpi)
    let widest = section
        .paragraphs
        .iter()
        .filter_map(|p| p.line_segs.last().map(|s| s.line_spacing as f64 / 75.0))
        .fold(0.0_f64, f64::max);

    assert!(
        (widest - FIXTURE_TRAILING_SPACING_PX).abs() < 1.0,
        "표본의 최대 말미 줄간격이 {widest:.1}px 다 — \n         {FIXTURE_TRAILING_SPACING_PX:.1}px 이어야 한다. 시험하려는 한도(12~18px)보다 \n         충분히 크지 않으면 트림 한도를 시험하지 못하고 수정 전에도 통과한다."
    );
}
