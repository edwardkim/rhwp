//! 문서를 변경하는 CLI command 어댑터.
//!
//! 출력 전용 어댑터와 달리 문서 상태를 바꾸고 직렬화하므로 CQRS 경계를 분리한다.

pub(crate) mod batch_convert;
pub(crate) mod batch_fill;
pub(crate) mod caption_validation;
pub(crate) mod conversion;
pub(crate) mod edit;
pub(crate) mod generation;
pub(crate) mod internal_validation;
pub(crate) mod tabular_import;
