//! HWP3/HWPX 공통 암호 모듈의 실제 fixture 저장 계약.

#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;
use std::path::Path;

use rhwp::parse_document_with_password;
use rhwp::parser::{parse_document, ParseError};
use rhwp::password_crypto::{
    decrypt_hwp3_password_document, decrypt_hwpx_package, encrypt_hwp3_password_document,
    encrypt_hwpx_package, is_hwp3_password_protected, PasswordCryptoError,
};
use rhwp::serializer::serialize_hwpx_with_password;
use zip::{CompressionMethod, ZipArchive};

const PASSWORD: &[u8] = b"123456";

fn fixture(relative: &str) -> Vec<u8> {
    std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .expect("password fixture를 읽어야 함")
}

fn document_shape(document: &rhwp::model::document::Document) -> (usize, usize, usize) {
    (
        document.sections.len(),
        document
            .sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum(),
        document.bin_data_content.len(),
    )
}

#[test]
fn hwp3_common_module_reencrypts_the_actual_password_fixture() {
    let source = fixture("samples/HWP3-password-123456.hwp");
    let plain = decrypt_hwp3_password_document(&source, PASSWORD).expect("HWP3 복호화");
    let encrypted = encrypt_hwp3_password_document(&plain, PASSWORD).expect("HWP3 암호화");

    assert!(is_hwp3_password_protected(&encrypted).expect("암호 flag"));
    assert!(matches!(
        parse_document(&encrypted),
        Err(ParseError::EncryptedDocument)
    ));
    assert_eq!(
        decrypt_hwp3_password_document(&encrypted, PASSWORD).expect("HWP3 재복호화"),
        plain
    );
    assert!(matches!(
        decrypt_hwp3_password_document(&encrypted, b"wrong-password"),
        Err(PasswordCryptoError::WrongPasswordOrCorruptPayload)
    ));
    assert_eq!(
        document_shape(&parse_document_with_password(&encrypted, PASSWORD).expect("HWP3 재열기")),
        document_shape(&parse_document_with_password(&source, PASSWORD).expect("원본 HWP3 열기"))
    );
}

#[test]
fn hwpx_common_module_and_serializer_write_password_packages() {
    let plain = fixture("samples/HWP5-nopassword-123456.hwpx");
    let encrypted = encrypt_hwpx_package(&plain, PASSWORD).expect("HWPX 암호화");
    let mut archive = ZipArchive::new(std::io::Cursor::new(&encrypted)).expect("암호 HWPX ZIP");
    let mut manifest = String::new();
    archive
        .by_name("META-INF/manifest.xml")
        .expect("manifest")
        .read_to_string(&mut manifest)
        .expect("manifest 읽기");
    assert!(manifest.contains("aes256-cbc"));
    assert!(manifest.contains("Contents/header.xml"));
    assert_eq!(
        archive
            .by_name("Contents/header.xml")
            .expect("header")
            .compression(),
        CompressionMethod::Stored
    );
    assert_eq!(
        archive
            .by_name("Contents/section0.xml")
            .expect("section")
            .compression(),
        CompressionMethod::Stored
    );
    drop(archive);

    assert!(matches!(
        parse_document(&encrypted),
        Err(ParseError::EncryptedDocument)
    ));
    let original = parse_document(&plain).expect("평문 HWPX 열기");
    let reopened = parse_document_with_password(&encrypted, PASSWORD).expect("암호 HWPX 열기");
    assert_eq!(document_shape(&reopened), document_shape(&original));
    assert!(matches!(
        decrypt_hwpx_package(&encrypted, b"wrong-password"),
        Err(PasswordCryptoError::WrongPasswordOrCorruptPayload)
    ));

    let serialized = serialize_hwpx_with_password(&original, PASSWORD).expect("HWPX 암호 저장");
    let reparsed =
        parse_document_with_password(&serialized, PASSWORD).expect("serializer HWPX 재열기");
    assert_eq!(document_shape(&reparsed), document_shape(&original));
}
