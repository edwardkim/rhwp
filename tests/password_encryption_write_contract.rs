//! 관계형·고정형: 검사 대상 — 암호 저장·복호화와 원본 대비 재열기.
//! 주석 갱신: 2026-11-01T05:59:30+09:00; 직전 파일 커밋: 2026-08-01T15:42:15+09:00 (`9dd0965dc037987cbac32af8dbd33b60f7e1b6eb`).
//! 입력·기준: `samples/143E433F503322BD33.hwp`와 HWP5 암호 저장 API.
//! 모델·API·직렬화 계약을 검사하며 화면 시각 일치율은 측정하지 않는다.
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
