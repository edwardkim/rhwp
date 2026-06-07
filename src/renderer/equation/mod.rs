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
    let tokens = tokenizer::tokenize(script);
    let ast = parser::EqParser::new(tokens).parse();
    let lb = layout::EqLayout::new(font_size_units).layout(&ast);
    (lb.width, lb.height)
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
}
