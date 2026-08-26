//! 문서를 변경하지 않지만 파일·stdout 산출물을 만드는 CLI output 어댑터.
//!
//! 읽기 전용 query와 달리 파일 시스템 부작용을 가질 수 있으므로 별도 경계에 둔다.

pub(crate) mod doclang;
pub(crate) mod pdf;
pub(crate) mod preview;
pub(crate) mod raster;
pub(crate) mod tabular;
pub(crate) mod text;
pub(crate) mod vector;

pub(crate) fn allows_implicit_sibling_resources(format: rhwp::parser::FileFormat) -> bool {
    // HML sibling paths are untrusted input and require an explicit resolver policy.
    !matches!(format, rhwp::parser::FileFormat::Hml)
}
