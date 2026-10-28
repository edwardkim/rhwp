//! 존재하지 않는 그림 주소는 문서 변경과 panic 없이 오류로 반환한다.

use rhwp::DocumentCore;

#[test]
fn invalid_header_footer_picture_addresses_return_errors_and_keep_document() {
    let mut core = DocumentCore::new_empty();
    let original = format!("{:?}", core.document());
    for section in [0, usize::MAX] {
        assert!(core
            .set_header_footer_picture_properties_native(section, 0, 0, 0, 0, "{}")
            .is_err());
        assert_eq!(format!("{:?}", core.document()), original);
    }
    core.create_blank_document_native().unwrap();
    let original = format!("{:?}", core.document());
    for paragraph in [0, usize::MAX] {
        assert!(core
            .set_header_footer_picture_properties_native(0, paragraph, 0, 0, 0, "{}")
            .is_err());
        assert_eq!(format!("{:?}", core.document()), original);
    }
}
