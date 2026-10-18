//! PDF content stream parsing and execution.
//!
//! This module handles parsing and processing of PDF content streams,
//! which contain sequences of operators that define page appearance.
//!
//! Phase 4

pub mod graphics_state;
pub(crate) mod operator_copy;
pub mod operators;
pub mod parser;

pub use graphics_state::{GraphicsState, GraphicsStateStack, Matrix};
pub use operators::{Operator, TextElement};
pub use parser::parse_and_execute_text_only;
pub use parser::parse_content_stream;
pub use parser::parse_content_stream_images_only;
pub use parser::parse_content_stream_paths_only;
#[allow(deprecated)]
pub use parser::parse_content_stream_text_only;
