use rhwp::parser::{self, cfb_reader, header, ParseError};
use rhwp::serializer::serialize_hwp_with_password;

const PASSWORD: &[u8] = b"stage4-password";

fn sample_document() -> rhwp::model::document::Document {
    let bytes = std::fs::read("samples/143E433F503322BD33.hwp").expect("HWP5 sample");
    parser::parse_document(&bytes).expect("sample parse")
}

#[test]
fn hwp5_password_save_requires_the_new_password_and_roundtrips() {
    let source = sample_document();
    let encrypted = serialize_hwp_with_password(&source, PASSWORD).expect("password save");

    let mut cfb = cfb_reader::CfbReader::open(&encrypted).expect("encrypted CFB");
    let file_header = header::parse_file_header(&cfb.read_file_header().expect("FileHeader"))
        .expect("header parse");
    assert!(file_header.flags.encrypted);
    assert_eq!(
        file_header.encrypt_version,
        rhwp::password_crypto::HWP5_ENCRYPT_VERSION
    );
    assert!(matches!(
        parser::parse_document(&encrypted),
        Err(ParseError::EncryptedDocument)
    ));
    assert!(matches!(
        parser::parse_document_with_password(&encrypted, b"wrong-password"),
        Err(ParseError::CryptoError(
            rhwp::parser::crypto::CryptoError::WrongPassword
        ))
    ));

    let reopened =
        parser::parse_document_with_password(&encrypted, PASSWORD).expect("password open");
    assert_eq!(reopened.sections.len(), source.sections.len());
    assert_eq!(
        reopened
            .sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum::<usize>(),
        source
            .sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum::<usize>()
    );
}
