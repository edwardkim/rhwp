//! #8174: 외부 Rust 앱의 표준 오류 전파와 도메인 오류 downcast 계약.

use rhwp::{DocumentCore, HwpError};
use std::error::Error;

fn open_core(bytes: &[u8]) -> Result<DocumentCore, Box<dyn Error + Send + Sync>> {
    Ok(DocumentCore::from_bytes(bytes)?)
}

#[test]
fn core_failure_propagates_as_a_standard_error_without_losing_its_type() {
    let error = open_core(b"not an HWP/HWPX document")
        .err()
        .expect("invalid input must fail through the public core API");
    let domain_error = error
        .downcast::<HwpError>()
        .expect("hosts must retain the original domain error type");
    assert!(matches!(*domain_error, HwpError::InvalidFile(_)));
}
