//! Issue #1355: 해설(미주) 문제-제목 앞 세로 여백 이중계상 회귀 가드.
//!
//! 직전 미주 문단이 "수식 전용(보이는 텍스트 없음)" tail 이고, 제목의 saved LINE_SEG
//! vpos 가 직전 bottom 보다 크게 점프(원본 단/쪽 경계)하면 vpos_adjust 가 "미주 사이"
//! gap 을 한 번 더 더해 제목 앞 여백이 약 2배가 되던 결함(p18 문30 → 문24 답안 본문
//! 초과, p19 문28 드리프트)의 회귀 가드.
//!
//! 재현 문서: `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`
//! 정정(layout.rs): 직전 문단이 textless(수식전용) + flow_advance≈gap + saved-vpos 점프
//! 큰 경우에만 제목을 흐름 위치로 되돌려 gap 1회만 남김. 직전이 텍스트 문단이거나
//! saved-vpos 점프가 작은 일반 순차 미주(2022_oct q19/q29, 2022_sep q15)는 제외.
//!
//! 가드: p18(0-idx 17) 좌측 컬럼 첫 미주 제목(문30)의 baseline y 가 이중계상 시의
//! 위치(+약 26px)로 회귀하지 않는지 절대 bound 로 추적.

use std::fs;
use std::path::Path;

fn load_doc(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

/// 페이지 SVG 에서 좌측 컬럼(x < x_max)의 '문' 글자 text baseline y 들을 오름차순 반환.
fn left_column_question_title_ys(svg: &str, x_max: f64) -> Vec<f64> {
    let mut ys = Vec::new();
    let mut rest = svg;
    while let Some(o) = rest.find("<text") {
        let end = rest[o..]
            .find("</text>")
            .map(|g| o + g + 7)
            .unwrap_or(rest.len());
        let tag = &rest[o..end];
        let plain: String = {
            let inner_start = tag.find('>').map(|g| g + 1).unwrap_or(0);
            let inner = &tag[inner_start..tag.len().saturating_sub(7)];
            let mut s = String::new();
            let mut skip = false;
            for ch in inner.chars() {
                match ch {
                    '<' => skip = true,
                    '>' => skip = false,
                    _ if !skip => s.push(ch),
                    _ => {}
                }
            }
            s
        };
        if plain.contains('문') {
            if let (Some(x), Some(y)) = (attr_f(tag, " x=\""), attr_f(tag, " y=\"")) {
                if x < x_max {
                    ys.push(y);
                }
            }
        }
        rest = &rest[end..];
    }
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys
}

fn attr_f(tag: &str, key: &str) -> Option<f64> {
    let i = tag.find(key)? + key.len();
    let r = &tag[i..];
    let e = r.find('"')?;
    r[..e].parse::<f64>().ok()
}

/// p18(0-idx 17) 좌측 첫 미주 제목(문30) baseline y.
/// 이중계상 회귀 시 약 +26px(≈ 1줄) 아래로 이동한다. 정정 위치 ~336px 기준,
/// 회귀(약 ≥362px)를 잡도록 상한 350px.
#[test]
fn endnote_first_title_gap_not_doubled_p18() {
    let doc = load_doc("samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp");
    let svg = doc.render_page_svg_native(17).expect("render p18");
    let ys = left_column_question_title_ys(&svg, 380.0);
    let first = *ys
        .first()
        .unwrap_or_else(|| panic!("p18 좌측 미주 제목('문') 미검출: {ys:?}"));
    assert!(
        first < 350.0,
        "p18 첫 미주 제목 y={first:.1} ≥ 350 — 제목 앞 gap 이중계상 회귀 의심 (정정 위치 ~336)"
    );
}
