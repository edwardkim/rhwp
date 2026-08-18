//! HWPX 글자모양 직렬화 표 — `hh:charPr` / PARA_CHAR_SHAPE 의 단일 출처.
//!
//! `header.rs` 의 `write_char_pr` 과 `char_shapes.rs` 가 이 모듈의 문자열 표를
//! 그대로 쓴다. 파서 역매핑과 어긋나면 왕복이 깨지므로 표를 여기 한곳에만 둔다.

pub mod attr_bits;
pub mod emphasis;
pub mod encoding_matrix;
pub mod hwp5_layout;
pub mod lang;
pub mod line_shape;
pub mod outline;
pub mod same_id_corpus;
pub mod shadow;
pub mod shape_catalog;
pub mod underline;

pub use attr_bits::*;
pub use emphasis::*;
pub use encoding_matrix::*;
pub use hwp5_layout::*;
pub use lang::*;
pub use line_shape::*;
pub use outline::*;
pub use same_id_corpus::*;
pub use shadow::*;
pub use shape_catalog::*;
pub use underline::*;
