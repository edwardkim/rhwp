//! 한컴 수식 스크립트 파싱 및 렌더링
//!
//! 수식 스크립트(버전 6.0)를 토큰화하고 AST로 변환한 뒤 SVG로 렌더링한다.
//! 참조: openhwp/docs/hwpx/appendix-i-formula.md

pub mod ast;
#[cfg(target_arch = "wasm32")]
pub mod canvas_render;
pub mod layout;
pub mod parser;
pub mod svg_render;
pub mod symbols;
pub mod tokenizer;

use crate::model::control::Equation;

/// 수식 스크립트를 조판하여 자연 크기(natural extent)를 반환한다.
///
/// `font_size_units` 단위로 결과가 나온다(레이아웃 엔진은 font_size 에 선형
/// 비례하므로, font_size 를 HWPUNIT 로 넣으면 폭/높이도 HWPUNIT, px 로 넣으면 px).
/// 반환값: `(width, height)`.
fn equation_natural_extent(script: &str, font_size_units: f64) -> (f64, f64) {
    let (w, h, _ascent) = equation_natural_metrics(script, font_size_units);
    (w, h)
}

/// 수식 스크립트를 조판하여 자연 크기 + baseline(ascent) 을 반환한다.
///
/// `font_size_units` 단위로 결과가 나온다(레이아웃 엔진은 font_size 에 선형 비례).
/// 반환값: `(width, height, ascent)` — `ascent` 는 박스 상단에서 baseline 까지의
/// 거리(= `LayoutBox.baseline`). 줄높이/baseline 보정에서 ascent/descent 분리가
/// 필요하므로 폭·높이와 함께 노출한다.
fn equation_natural_metrics(script: &str, font_size_units: f64) -> (f64, f64, f64) {
    let tokens = tokenizer::tokenize(script);
    let ast = parser::EqParser::new(tokens).parse();
    let lb = layout::EqLayout::new(font_size_units).layout(&ast);
    (lb.width, lb.height, lb.baseline)
}

/// 줄 배치(겹침 방지)에 필요한 수식의 자연 ascent/descent(px)를 반환한다.
///
/// `equation_effective_size_hwpunit` 은 줄 **높이**(advance/세로 예약 총량)만 주는데,
/// TALL 한 분수(분자/분모 다단)는 baseline **아래**(descent)로 깊게 내려가므로 총
/// 높이만 키워서는 부족하다 — baseline 을 ascent 위치에 맞추고 그 **아래로** descent
/// 만큼을 따로 예약해야 다음 문단이 분모(예: `dx^4`) 위에 겹치지 않는다.
///
/// 렌더(paragraph_layout)는 수식을 `font_size_px` 로 다시 조판한 `layout_box` 로
/// 그 자리에 그리므로(`eq_y = line_top + line_baseline - layout_box.baseline`),
/// 여기서도 **동일한 px 폰트 크기**로 조판해 ascent/descent 가 정확히 일치하게 한다.
///
/// 저장 bbox 가 양수여도(한컴이 측정한 advance) baseline 정보는 없으므로 ascent 는
/// 항상 조판값을 쓰되, descent 가 저장 높이를 넘으면(다단 분수) 저장 높이 대신 조판
/// descent 를 예약한다. 반환값: `(ascent_px, descent_px)`, 둘 다 `>= 0`.
pub fn equation_natural_ascent_descent_px(eq: &Equation, dpi: f64) -> (f64, f64) {
    let font_units = if eq.font_size > 0 {
        eq.font_size as f64
    } else {
        1000.0
    };
    let font_px = crate::renderer::hwpunit_to_px(font_units as i32, dpi);
    let (_w, h, ascent) = equation_natural_metrics(&eq.script, font_px);
    let descent = (h - ascent).max(0.0);
    (ascent.max(0.0), descent)
}

/// 한컴이 저장한 수식 컨트롤의 폭/높이가 0(작성 툴이 bbox 미계산)일 때,
/// 스크립트를 조판하여 자연 크기를 HWPUNIT 로 보충한다.
///
/// 한컴 뷰어는 size=0×0 수식을 조판 시점에 스스로 재측정하여 advance/줄높이를
/// 확보한다. rhwp_core 도 동일하게: 저장값이 양수면 그대로 신뢰하고, 0/음수면
/// 조판 자연 크기로 대체한다. EQEDIT 의 글자 크기(`font_size`, HWPUNIT)를
/// 레이아웃 단위로 사용하면 결과가 HWPUNIT 으로 나온다.
///
/// 반환값: `(width_hwpunit, height_hwpunit)` — 둘 다 `>= 0`.
pub fn equation_effective_size_hwpunit(eq: &Equation) -> (i32, i32) {
    let stored_w = eq.common.width as i32;
    let stored_h = eq.common.height as i32;
    if stored_w > 0 && stored_h > 0 {
        return (stored_w, stored_h);
    }
    // EQEDIT font_size 가 0/비정상이면 한컴 기본 수식 글자크기(1000 HWPUNIT ≈ 10pt) 사용.
    let font_units = if eq.font_size > 0 {
        eq.font_size as f64
    } else {
        1000.0
    };
    let (nat_w, nat_h) = equation_natural_extent(&eq.script, font_units);
    let width = if stored_w > 0 {
        stored_w
    } else {
        nat_w.ceil().max(0.0) as i32
    };
    let height = if stored_h > 0 {
        stored_h
    } else {
        nat_h.ceil().max(0.0) as i32
    };
    (width, height)
}

#[cfg(test)]
mod size_tests {
    use super::*;
    use crate::model::control::Equation;

    fn eq(script: &str, w: u32, h: u32, fs: u32) -> Equation {
        let mut e = Equation {
            script: script.to_string(),
            font_size: fs,
            ..Default::default()
        };
        e.common.width = w;
        e.common.height = h;
        e
    }

    #[test]
    fn stored_size_is_trusted_when_positive() {
        let e = eq("a over b", 5000, 3000, 1000);
        assert_eq!(equation_effective_size_hwpunit(&e), (5000, 3000));
    }

    #[test]
    fn zero_size_falls_back_to_natural_extent() {
        // 작성 툴이 bbox=0×0 으로 둔 수식 — advance/줄높이가 사라지면 안 됨.
        let e = eq("x = {-b +- sqrt {b^2 - 4ac}} over {2a}", 0, 0, 1000);
        let (w, h) = equation_effective_size_hwpunit(&e);
        assert!(w > 0, "0 폭 수식은 조판 자연폭으로 보충되어야 함: {w}");
        assert!(h > 0, "0 높이 수식은 조판 자연높이로 보충되어야 함: {h}");
        // 분수는 한 줄 텍스트(1000 HWPUNIT)보다 확실히 높아야 함.
        assert!(h > 1000, "분수 수식 높이({h})는 단일 줄보다 커야 함");
    }

    #[test]
    fn font_size_scales_natural_extent() {
        let small = equation_effective_size_hwpunit(&eq("a over b", 0, 0, 1000));
        let big = equation_effective_size_hwpunit(&eq("a over b", 0, 0, 2000));
        assert!(big.0 > small.0 && big.1 > small.1, "글자 크기에 비례해야 함");
    }

    #[test]
    fn ascent_descent_sum_matches_total_height() {
        // ascent + descent 는 자연 높이와 일치해야 한다(줄 예약의 단일 출처).
        let e = eq("a over b", 0, 0, 1000);
        let dpi = 96.0;
        let (ascent, descent) = equation_natural_ascent_descent_px(&e, dpi);
        let font_px = crate::renderer::hwpunit_to_px(1000, dpi);
        let (_w, h, _bl) = equation_natural_metrics(&e.script, font_px);
        assert!((ascent + descent - h).abs() < 1e-6, "ascent+descent == height");
        assert!(ascent > 0.0 && descent > 0.0);
    }

    #[test]
    fn tall_block_fraction_reserves_descent_below_baseline() {
        // engineering_formulas 의 EI d^4w/dx^4 블록 수식: 분모 dx^4 가 baseline
        // 아래(descent)로 깊게 내려간다. 이 descent 가 충분히 크지 않으면 줄높이
        // 보정이 다음 문단("3. 레이놀즈 수")과의 겹침을 못 막는다.
        let dpi = 96.0;
        // 한 줄 텍스트 baseline ~ 0.8em, descent ~ 0.2em.
        let one_line = eq("a", 0, 0, 1000);
        let (_a1, text_like_descent) = equation_natural_ascent_descent_px(&one_line, dpi);

        let block = eq("EI {d ^4 w} over {d x ^4} = q (x)", 0, 0, 1000);
        let (ascent, descent) = equation_natural_ascent_descent_px(&block, dpi);
        assert!(ascent > 0.0, "ascent 가 있어야 함");
        // 분수 descent 는 단일 줄 텍스트 descent 보다 훨씬 커야 한다(예약 부족 = 겹침).
        assert!(
            descent > text_like_descent * 1.5,
            "TALL 분수 descent({descent})는 텍스트 descent({text_like_descent})보다 커야 함",
        );
    }
}
