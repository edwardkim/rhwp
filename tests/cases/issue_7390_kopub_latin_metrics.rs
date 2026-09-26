//! #7390: KoPub 계열의 라틴 전진폭을 **글꼴 실측 표**로 잰다.
//!
//! `kopub_char_width` 는 KoPub face 를 전용 표로 처리하면서 한글 전각만 실측값이었고
//! **ASCII 는 전부 `font_size * 0.5`** 였다. KoPub 은 비례 글꼴이라 `i` 232 · `N`/`H` 718
//! 처럼 3배 넘게 갈린다(1000em 기준). 그래서 영문이 섞인 줄이 계통적으로 좁게 잡혔다.
//!
//! # 기대값
//!
//! 한컴 정본 `pdf/issue2006/1790387_prep_final_report-hwp2020-20260814.pdf` 는
//! **KoPub 이 설치된 환경**에서 인쇄돼 `KoPubDotumLight`/`Bold` 서브셋을 내장한다.
//! `mutool draw -F stext` 로 같은 글자열 줄의 첫 글자 `x0` 부터 마지막 글자 `x1` 까지를
//! 96dpi px 로 잰 값이 아래 `oracle_ink_px` 다.
//!
//! ```text
//!   쪽    정본 잉크폭   수정 전 점유폭    수정 후 점유폭
//!   94      412.55      364.40 (-48.2)   414.20 (+1.7)
//!  108      570.67      540.60 (-30.1)   574.10 (+3.4)
//! ```
//!
//! rhwp 쪽은 run 의 **점유폭**이라 마지막 글자 전진폭까지 포함해 정본 잉크폭보다 조금 크다.
//! 그래서 등식이 아니라 `±10px` 구간으로 본다. 수정 전 값은 30~48px 좁아 구간 밖이다.
//!
//! # 공백은 바꾸지 않았다
//!
//! 글꼴 `hmtx` 의 공백은 KoPubDotum 290 · KoPubBatang 312 지만 **한/글은 그 값을 쓰지
//! 않는다.** 같은 정본을 세 방법으로 재면 모두 반각을 가리킨다 — 연속 공백 쌍 n=359 과
//! 4개 이상 덩어리 n=59 가 둘 다 0.4767 em(그려진 폭), 183줄 최소제곱이 장평 0.9510 ·
//! 자연 공백 0.5045 em. 고정값별 잔차 중앙은 `0.290 → 0.669` · `0.436 → 0.314` ·
//! `0.484 → 0.249` · `0.500 → 0.269` 로, 글꼴 값 0.290 이 가장 나쁘다.
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const SAMPLE: &str = "samples/issue2006/1790387_prep_final_report.hwpx";
/// 정본 PDF 의 쪽 너비(595.0pt)를 96dpi px 로 환산한 값.
const ORACLE_PAGE_PX: f64 = 595.0 * 96.0 / 72.0;
const TOLERANCE_PX: f64 = 10.0;

fn collect(node: &RenderNode, out: &mut Vec<(String, f64, f64, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((run.text.clone(), node.bbox.x, node.bbox.width, node.bbox.y));
    }
    for child in &node.children {
        collect(child, out);
    }
}

/// 같은 baseline 의 run 을 모아 글자열이 `prefix` 로 시작하는 줄의 점유폭과 쪽 너비를 낸다.
fn line_span(doc: &DocumentCore, page: u32, prefix: &str) -> Option<(f64, f64)> {
    let tree = doc.build_page_render_tree(page - 1).ok()?;
    let mut runs = Vec::new();
    collect(&tree.root, &mut runs);
    let mut baselines: Vec<f64> = runs.iter().map(|r| r.3).collect();
    baselines.sort_by(|a, b| a.partial_cmp(b).unwrap());
    baselines.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    for baseline in baselines {
        let mut line: Vec<&(String, f64, f64, f64)> = runs
            .iter()
            .filter(|r| (r.3 - baseline).abs() < 0.5)
            .collect();
        line.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let joined: String = line
            .iter()
            .flat_map(|r| r.0.chars())
            .filter(|c| !c.is_whitespace())
            .collect();
        if joined.starts_with(prefix) {
            let left = line.iter().fold(f64::MAX, |a, r| a.min(r.1));
            let right = line.iter().fold(f64::MIN, |a, r| a.max(r.1 + r.2));
            return Some((right - left, tree.root.bbox.width));
        }
    }
    None
}

#[test]
fn kopub_latin_line_width_matches_hancom_in_absolute_px() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc =
        DocumentCore::from_bytes(&std::fs::read(path).expect("공개 회귀 문서")).expect("문서 파싱");
    for (page, prefix, oracle_ink_px) in [
        (
            94u32,
            "MSM,menwhohavesexwithmen;TGW/TGM,transwo",
            412.55_f64,
        ),
        (108, "withmenthroughthenetworkscale-upmethodin", 570.67),
    ] {
        let (span, page_px) =
            line_span(&doc, page, prefix).unwrap_or_else(|| panic!("{page}쪽의 `{prefix}` 줄"));
        let expected = oracle_ink_px * page_px / ORACLE_PAGE_PX;
        assert!(
            (span - expected).abs() <= TOLERANCE_PX,
            "{page}쪽 `{prefix}` 줄 점유폭 {span:.2}px 이 정본 {expected:.2}px 에서 \
             {:.2}px 벗어났다(허용 {TOLERANCE_PX}px). KoPub ASCII 를 일률 0.5em 으로 재면 \
             줄이 30~48px 좁아진다.",
            span - expected,
        );
    }
}

/// 공백은 글꼴 표가 아니라 반각을 쓴다 — 이 변경이 공백까지 건드리지 않았는지 지킨다.
///
/// 글꼴 `hmtx` 의 KoPubDotum 공백은 290/1000 이다. 그 값을 쓰면 공백마다 0.21 em 이 빠져
/// 아래 줄이 크게 좁아진다. 정본은 반각을 가리킨다(본 파일 머리말의 세 측정).
///
/// 이 검사는 **수정 전에도 실패한다**(540.64px) — 다만 원인이 다르다. 수정 전에는 ASCII 가
/// 일률 0.5em 이라 좁고, 이 검사가 막으려는 것은 앞으로 공백을 글꼴 값 0.290 으로 바꾸는
/// 변경이다. 그래서 위 검사의 중복이 아니라 공백 축 전용 하한이다.
#[test]
fn kopub_space_stays_half_width() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc =
        DocumentCore::from_bytes(&std::fs::read(path).expect("공개 회귀 문서")).expect("문서 파싱");
    let (span, _) =
        line_span(&doc, 108, "withmenthroughthenetworkscale-upmethodin").expect("108쪽 대상 줄");
    // 이 줄의 공백은 9개다. 글꼴 값 0.290 을 쓰면 9 × 0.21 em ≈ 25px 가 빠진다.
    assert!(
        span > 555.0,
        "108쪽 줄 점유폭 {span:.2}px 이 555px 이하다. 공백에 글꼴 표값(0.290 em)을 쓰면 \
         이 구간 아래로 떨어진다 — 한/글은 반각으로 전진시킨다.",
    );
}
