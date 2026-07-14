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

/// 수식 스크립트와 BaseUnit에서 레이아웃이 소비할 intrinsic HWPUNIT 크기를 계산한다.
pub fn intrinsic_size_hwp(script: &str, font_size: u32) -> (u32, u32) {
    let font_size_px = super::hwpunit_to_px(font_size.max(1) as i32, super::DEFAULT_DPI);
    let tokens = tokenizer::tokenize(script);
    let ast = parser::EqParser::new(tokens).parse();
    let layout = layout::EqLayout::new(font_size_px).layout(&ast);
    (
        super::px_to_hwpunit(layout.width, super::DEFAULT_DPI).max(1) as u32,
        super::px_to_hwpunit(layout.height, super::DEFAULT_DPI).max(1) as u32,
    )
}
