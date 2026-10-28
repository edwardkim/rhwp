//! Public destination contract; original spelling and unsafe payload rejection.
use rhwp::model::hyperlink::*;

#[test]
fn destinations_preserve_original_command_spelling() {
    for (uri, kind) in [
        (
            "https://example.com/한글?q=a;b#목표",
            LinkDestinationKind::Web,
        ),
        ("mailto:user@example.com", LinkDestinationKind::Mail),
        ("#목표", LinkDestinationKind::DocumentAnchor),
        ("../서류/신청서.pdf", LinkDestinationKind::RelativeFile),
        (r"C:\서류\신청서.hwp", LinkDestinationKind::LocalFile),
        ("file:///home/user/문서.hwp", LinkDestinationKind::LocalFile),
        (
            "rhwp-app:open/document",
            LinkDestinationKind::RegisteredApplication,
        ),
    ] {
        assert_eq!(classify_destination(uri), Some(kind));
        assert_eq!(command_uri(&destination_command(uri).unwrap()), uri);
    }
}

#[test]
fn executable_payloads_and_control_bytes_are_not_destinations() {
    for uri in [
        "javascript:alert(1)",
        "VBScript:run",
        "data:text/html,test",
        "",
        "#",
        "../bad\nfile",
        "https://user:secret@example.com",
        "https://",
    ] {
        assert!(classify_destination(uri).is_none(), "{uri:?}");
        assert!(destination_command(uri).is_err());
    }
}
