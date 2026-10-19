//! 메타파일 host font는 저장 문서가 아닌 paint 세션 상태다.
//! 실물 배치나 렌더링 일치율을 고정하는 검사가 아니다.

use rhwp::document_core::DocumentCore;

const FONT: &[u8] = include_bytes!("../fixtures/fonts/RHWPHostFixture-Regular.ttf");

#[test]
fn metafile_font_registration_and_clear_preserve_document_and_saved_bytes() {
    let mut core = DocumentCore::new_empty();
    let document_before = format!("{:?}", core.document());
    let saved_before = rhwp::serializer::serialize_document(core.document()).unwrap();
    let info_before = core.get_document_info();

    // 등록 중복과 해제 뒤의 재등록은 실제 세션 상태가 바뀌었음을 확인한다.
    for _ in 0..2 {
        assert!(core
            .register_metafile_font_native("RHWP Host Fixture", FONT, 0)
            .unwrap());
        assert!(!core
            .register_metafile_font_native("RHWP Host Fixture", FONT, 0)
            .unwrap());
        assert_eq!(format!("{:?}", core.document()), document_before);
        assert_eq!(
            rhwp::serializer::serialize_document(core.document()).unwrap(),
            saved_before,
            "paint 프로그램 등록은 저장 IR과 원시 스트림을 바꾸지 않는다"
        );
        core.clear_metafile_fonts_native();
        assert_eq!(format!("{:?}", core.document()), document_before);
        assert_eq!(
            rhwp::serializer::serialize_document(core.document()).unwrap(),
            saved_before,
            "paint 세션 해제는 저장 내용을 바꾸지 않는다"
        );
        assert_eq!(core.get_document_info(), info_before);
    }
}
