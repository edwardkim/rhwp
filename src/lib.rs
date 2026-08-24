//! rhwp — Rust HWP 뷰어/에디터
//!
//! 본 제품은 한글과컴퓨터의 한글 문서 파일(.hwp) 공개 문서를 참고하여 개발하였습니다.

use wasm_bindgen::prelude::*;

/// 기존 함수 시그니처를 유지한 채 그 몸통만 Subsecond jump table로 우회한다.
/// 호출부는 벤더를 모르고, production에서는 원래 함수 직접 호출로 완전히 접힌다.
macro_rules! hot_call {
    ($target:path $(, $arg:expr)* $(,)?) => {{
        #[cfg(feature = "subsecond-dev")]
        {
            if crate::subsecond_dev::patch_epoch() == 0 {
                $target($($arg),*)
            } else {
                let mut hot = subsecond::HotFn::current($target);
                hot.call(($($arg,)*))
            }
        }
        #[cfg(not(feature = "subsecond-dev"))]
        {
            $target($($arg),*)
        }
    }};
}
pub(crate) use hot_call;

pub mod agent;
pub mod agent_seal;
pub mod capabilities_schema;
pub mod diagnostics;
pub mod docdiff;
pub mod doclang;
pub mod document_core;
pub mod emf;
pub mod eps;
pub mod error;
pub use rhwp_contracts::ir_schema;
pub mod model;
pub mod ole_chart;
pub use rhwp_contracts::ontology;
pub use rhwp_ooxml_chart as ooxml_chart;
pub mod paint;
pub mod parser;
pub use rhwp_password_crypto as password_crypto;
pub mod plan_schema;
pub mod pq_sign;
pub use rhwp_contracts::provenance;
pub mod rag;
pub mod render_backend;
pub mod renderer;
pub mod scaffold;
pub mod schema_registry;
pub mod security_trailer;
pub mod serializer;
pub mod service;
/// 핫패치 벤더(Dioxus subsecond) 어댑터. **rhwp 의 API 가 아니다** (#4580).
///
/// `pub` 인 이유는 `tools/rhwp-subsecond` 가 `link_wasm_exports()` 를 불러 wasm export 를 살려
/// 둬야 하기 때문이지, 이 안의 함수들을 밖에서 쓰라는 뜻이 아니다. 격리 자체는 feature 가 이미
/// 지킨다 — `subsecond-dev` 없이는 모듈이 존재하지 않으므로 릴리스 표면에는 처음부터 나오지
/// 않는다. `#[doc(hidden)]` 은 그 사실을 문서에도 적는 것이다.
#[doc(hidden)]
#[cfg(feature = "subsecond-dev")]
pub mod subsecond_dev;
pub mod wasm_api;
pub mod wmf;

pub use document_core::DocumentCore;
pub use error::HwpError;
pub use model::event::DocumentEvent;
pub use parser::{parse_document, parse_document_with_password, DocumentParser};
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
        fn add_one(value: u32) -> u32 {
            value + 1
        }
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(crate::hot_call!(add_one, 41), 42);
    }
}
