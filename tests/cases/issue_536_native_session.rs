//! Native host 수명·입력·raw frame 계약. 문서 조판의 시각 정답을 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::native_host::{NativeDocumentSession, NativeHostError};

#[test]
fn issue_536_native_sessions_are_independent_and_reopen_core_bytes() {
    let first = NativeDocumentSession::new_blank().unwrap();
    let bytes = first.document().export_hwpx_native().unwrap();
    let second = NativeDocumentSession::open_bytes(&bytes, None).unwrap();
    drop(first);
    assert!(second.document().page_count() > 0);
    assert_eq!(second.revision(), 0);
    assert!(second.page_size(0).unwrap().width > 0.0);
    assert!(second.page_size(second.document().page_count()).is_err());
    assert!(matches!(
        second.check_revision(1),
        Err(NativeHostError::StaleRevision { .. })
    ));
}

#[test]
fn issue_536_native_session_rejects_invalid_inputs_before_render() {
    assert!(NativeDocumentSession::open_bytes(&[], None).is_err());
    assert!(NativeDocumentSession::open_bytes(b"not a document", None).is_err());
    let session = NativeDocumentSession::new_blank().unwrap();
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY, 65.0] {
        assert!(matches!(
            session.render_page(0, scale),
            Err(NativeHostError::InvalidArgument(_))
        ));
    }
}

#[cfg(not(feature = "native-skia"))]
#[test]
fn issue_536_native_session_without_skia_reports_capability_error() {
    let session = NativeDocumentSession::new_blank().unwrap();
    assert!(matches!(
        session.render_page(0, 1.0),
        Err(NativeHostError::FeatureDisabled)
    ));
}

#[cfg(feature = "native-skia")]
#[test]
fn issue_536_native_session_returns_owned_repeatable_raw_pixels() {
    let session = NativeDocumentSession::new_blank().unwrap();
    let first = session.render_page(0, 0.25).unwrap();
    let second = session.render_page(0, 0.25).unwrap();
    assert_eq!(first.pixels, second.pixels);
    drop(session);
    first.pixels.validate().unwrap();
    assert_eq!(first.revision, 0);
    assert!(first.pixels.bytes.chunks_exact(4).all(|p| p[3] == 255));
}
