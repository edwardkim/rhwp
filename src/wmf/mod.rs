#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::enum_variant_names,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    clippy::wildcard_imports,
    non_camel_case_types,
    non_snake_case,
    unexpected_cfgs,
    dead_code,
    unused_imports,
    unused_variables
)]

// tracing 스텁 매크로 (converter/parser 모듈보다 먼저 정의해야 하위 모듈에서 사용 가능)
#[allow(unused_macros)]
macro_rules! debug {
    ($($arg:tt)+) => {};
}
#[allow(unused_macros)]
macro_rules! info {
    ($($arg:tt)+) => {};
}
#[allow(unused_macros)]
macro_rules! warn {
    ($($arg:tt)+) => {};
}
#[allow(unused_macros)]
macro_rules! error {
    ($($arg:tt)+) => {};
}

pub mod converter;
pub mod parser;

mod imports {
    pub use std::{
        borrow::ToOwned,
        boxed::Box,
        collections::{BTreeMap, BTreeSet, VecDeque},
        string::{String, ToString},
        vec::Vec,
    };
}

pub use embedded_io::Read;
