//! The password package prepass runs before HwpxReader's XML budget.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{parse_document_with_password, password_crypto, DocumentCore};
use std::io::{Cursor, Read, Write};

const PASSWORD: &[u8] = b"private-local-contract";

fn evidence(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("RHWP_PRIVATE_SECURITY_OUTPUT") {
        let directory = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(name), bytes).unwrap();
    }
}

fn options() -> zip::write::SimpleFileOptions {
    zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(1))
}

fn normal_package() -> Vec<u8> {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "normal password document")
        .unwrap();
    let package =
        password_crypto::encrypt_hwpx_package(&core.export_hwpx_native().unwrap(), PASSWORD)
            .unwrap();
    evidence("normal-password.hwpx", &package);
    package
}

fn write_large_xml(writer: &mut zip::ZipWriter<Cursor<Vec<u8>>>, path: &str) {
    writer.start_file(path, options()).unwrap();
    let chunk = vec![b' '; 1024 * 1024];
    for _ in 0..64 {
        writer.write_all(&chunk).unwrap();
    }
}

fn assert_budget_rejection(result: Result<Option<Vec<u8>>, password_crypto::PasswordCryptoError>) {
    match result {
        Err(error) => assert!(error.to_string().contains("XML read budget"), "{error}"),
        Ok(_) => panic!("password prepass accepted more than 512MiB of XML output"),
    }
}

#[test]
fn normal_password_document_still_opens() {
    let doc = parse_document_with_password(&normal_package(), PASSWORD).unwrap();
    assert!(doc.sections[0]
        .paragraphs
        .iter()
        .any(|p| p.text.contains("normal password document")));
}

#[test]
fn password_prepass_bounds_unprotected_xml_members_before_the_reader() {
    let encrypted = normal_package();
    let mut source = zip::ZipArchive::new(Cursor::new(encrypted)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut input = source.by_index(index).unwrap();
        if input.is_dir() {
            continue;
        }
        writer.start_file(input.name(), options()).unwrap();
        std::io::copy(&mut input, &mut writer).unwrap();
    }
    for entry in 0..9 {
        write_large_xml(&mut writer, &format!("Aux/extra{entry}.xml"));
    }
    let package = writer.finish().unwrap().into_inner();
    evidence("password-outer-xml-budget.hwpx", &package);
    assert_budget_rejection(password_crypto::decrypt_hwpx_package(&package, PASSWORD));
    // The public document-open caller must propagate the same resource failure.
    match parse_document_with_password(&package, PASSWORD) {
        Err(error) => assert!(error.to_string().contains("XML read budget"), "{error}"),
        Ok(_) => panic!("document open hid the password XML budget failure"),
    }
}

#[test]
fn password_prepass_bounds_inner_decrypted_xml_output() {
    // A small compressed/encrypted member expands to 64MiB. Replicate the valid
    // crypto metadata under distinct paths; no checksum or password guessing.
    let mut plain = zip::ZipWriter::new(Cursor::new(Vec::new()));
    plain
        .start_file("META-INF/manifest.xml", options())
        .unwrap();
    plain.write_all(b"<manifest/>").unwrap();
    write_large_xml(&mut plain, "Contents/section0.xml");
    let encrypted =
        password_crypto::encrypt_hwpx_package(&plain.finish().unwrap().into_inner(), PASSWORD)
            .unwrap();
    let mut source = zip::ZipArchive::new(Cursor::new(encrypted)).unwrap();
    let mut manifest = String::new();
    source
        .by_name("META-INF/manifest.xml")
        .unwrap()
        .read_to_string(&mut manifest)
        .unwrap();
    let parsed = roxmltree::Document::parse(&manifest).unwrap();
    let entry = parsed
        .root_element()
        .children()
        .find(|node| node.is_element())
        .unwrap();
    let entry_text = &manifest[entry.range()];
    let mut ciphertext = Vec::new();
    source
        .by_name("Contents/section0.xml")
        .unwrap()
        .read_to_end(&mut ciphertext)
        .unwrap();
    assert!(ciphertext.len() < 1024 * 1024);
    let mut repeated_manifest = String::from(
        r#"<odf:manifest xmlns:odf="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0">"#,
    );
    for index in 0..9 {
        repeated_manifest.push_str(&entry_text.replace(
            "Contents/section0.xml",
            &format!("Contents/section{index}.xml"),
        ));
    }
    repeated_manifest.push_str("</odf:manifest>");
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file("META-INF/manifest.xml", options())
        .unwrap();
    writer.write_all(repeated_manifest.as_bytes()).unwrap();
    for index in 0..9 {
        writer
            .start_file(format!("Contents/section{index}.xml"), options())
            .unwrap();
        writer.write_all(&ciphertext).unwrap();
    }
    assert_budget_rejection(password_crypto::decrypt_hwpx_package(
        &writer.finish().unwrap().into_inner(),
        PASSWORD,
    ));
}
