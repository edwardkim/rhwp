//! rhwp — Rust HWP 뷰어/에디터
//!
//! 본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.

use wasm_bindgen::prelude::*;

pub mod document_core;
pub mod error;
pub mod model;
pub mod paint;
pub mod parser;
pub mod renderer;
pub mod serializer;
pub mod wasm_api;
pub mod wmf;
pub mod emf;
pub mod ooxml_chart;

pub use document_core::DocumentCore;
pub use error::HwpError;
pub use model::event::DocumentEvent;
pub use parser::{parse_document, DocumentParser};
pub use serializer::{serialize_document, DocumentSerializer};

/// WASM panic hook 초기화 (한 번만 실행)
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
    }
}
