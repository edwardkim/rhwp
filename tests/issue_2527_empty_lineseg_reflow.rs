//! Issue #2527: 빈 lineseg 배열 HWPX — as-is 정상 / `reflowLinesegs()` 좌표 붕괴 재현 fixture.
//!
//! PR #2528(모달 비표시 완화) 검토의 후속 권고("실제 문제 fixture 와 as-is/auto-fix
//! 비교 회귀 테스트 보존")에 따른 재현 자산이다. `samples/issue2527_empty_linesegs.hwpx`
//! 는 텍스트 문단 5개(본문 4 — 3줄 래핑 장문 포함 — + 표 셀 1) 전부의
//! linesegarray 가 비어있고 HWP5-기원 마커(`META-INF/rhwp-hwp5-origin`)를 가진
//! 비표준 HWPX 다 — 이 계열만 로드 시 자동 합성(#1380)을 건너뛰어
//! `reflowLinesegs()` 온디맨드 경로가 실제로 발동한다 (#2527 검증 경고 형상).
//!
//! 주: 이 fixture 는 native 경로에서 #2527 의 좌표 붕괴를 재현하지 못한다
//! (현 devel GREEN). 붕괴는 studio/WASM 측 폰트 준비 전 measureText 의존이
//! 유력해(이슈의 "헤드리스 CanvasKit 재현" 단서) 원본 문서 확보 시 보강한다.

use std::fs;
use std::path::Path;

const FIXTURE: &str = "samples/issue2527_empty_linesegs.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(FIXTURE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {FIXTURE}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {FIXTURE}: {e:?}"))
}

fn attr_f64(attrs: &str, name: &str) -> Option<f64> {
    let key = format!("{name}=\"");
    let start = attrs.find(&key)? + key.len();
    let end = attrs[start..].find('"')? + start;
    attrs[start..end].parse().ok()
}

/// SVG 에서 `marker` 문자열의 각 글자에 해당하는 per-char `<text>` x 좌표를
/// 문서 내 등장 순서대로 수집한다.
fn glyph_xs(svg: &str, marker: &str) -> Vec<f64> {
    let wanted: Vec<String> = marker.chars().map(|c| c.to_string()).collect();
    let mut xs = Vec::new();
    let mut wi = 0;
    let mut search_from = 0;
    while wi < wanted.len() {
        let Some(rel) = svg[search_from..].find("<text ") else {
            break;
        };
        let tag_start = search_from + rel;
        search_from = tag_start + 6;
        let Some(close_rel) = svg[tag_start..].find('>') else {
            break;
        };
        let attrs = &svg[tag_start..tag_start + close_rel];
        let content_start = tag_start + close_rel + 1;
        let Some(end_rel) = svg[content_start..].find("</text>") else {
            break;
        };
        let text = &svg[content_start..content_start + end_rel];
        if text == wanted[wi] {
            if let Some(x) = attr_f64(attrs, "x") {
                xs.push(x);
                wi += 1;
            }
        } else if wi > 0 && text == wanted[0] {
            // 부분 일치 후 어긋남 — 처음부터 재시도
            xs.clear();
            xs.push(attr_f64(attrs, "x").unwrap_or(0.0));
            wi = 1;
        }
    }
    assert_eq!(
        wi,
        wanted.len(),
        "marker {marker:?} 글리프 시퀀스를 SVG 에서 찾지 못함 (found {wi}/{})",
        wanted.len()
    );
    xs
}

/// 인접 글리프 간 advance 최소값 (px).
fn min_advance(xs: &[f64]) -> f64 {
    xs.windows(2)
        .map(|w| w[1] - w[0])
        .fold(f64::INFINITY, f64::min)
}

const MARKER: &str = "긴문단래핑검증";

/// 빈 lineseg 검증 경고가 본문 4 + 표 셀 1 = 5건 잡히는지 고정.
#[test]
fn issue_2527_fixture_reports_empty_lineseg_warnings() {
    let doc = load_doc();
    let json = doc.get_validation_warnings();
    assert!(
        json.contains(r#""lineseg 배열이 비어있음":5"#),
        "빈 lineseg 경고 5건이어야 함: {json}"
    );
    assert!(
        json.contains(r#""cell":{"#),
        "표 셀 경고가 포함돼야 함: {json}"
    );
}

/// as-is(그대로 보기, reflow 미적용) 경로: 렌더 시점 레이아웃으로 글리프가
/// 겹치지 않고 단조 증가 배치돼야 한다 — #2527 '그대로 보기 = 정상 판독' 계약.
#[test]
fn issue_2527_as_is_render_has_no_glyph_overlap() {
    let doc = load_doc();
    let svg = doc.render_page_svg_native(0).expect("as-is render page 0");
    let xs = glyph_xs(&svg, MARKER);
    let adv = min_advance(&xs);
    assert!(
        adv > 3.0,
        "as-is 글리프 최소 advance {adv:.2}px — 겹침 없이 배치돼야 함: {xs:?}"
    );
}

/// auto-fix(`reflowLinesegs()`) 경로: 빈 lineseg 문단 재배치 후에도 as-is 와
/// 동등한 비겹침 배치여야 한다 — #2527 의 catastrophic overlap 회귀 가드.
///
/// 현재 defect 가 재현되면(글리프 수렴) 이 테스트가 RED 로 근본 수정 필요성을
/// 표면화하고, 수정 후에는 GREEN 핀으로 남는다.
#[test]
fn issue_2527_reflow_must_not_collapse_glyphs() {
    let mut doc = load_doc();
    let reflowed = doc.reflow_linesegs();
    assert!(
        reflowed >= 5,
        "빈 lineseg 문단들이 reflow 돼야 함: {reflowed}"
    );
    let svg = doc
        .render_page_svg_native(0)
        .expect("post-reflow render page 0");
    let xs = glyph_xs(&svg, MARKER);
    let adv = min_advance(&xs);
    assert!(
        adv > 3.0,
        "reflow 후 글리프 최소 advance {adv:.2}px — #2527 좌표 붕괴(수렴) 재발: {xs:?}"
    );
}
