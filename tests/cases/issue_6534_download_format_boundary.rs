//! Issue #6534: OOXML·불완전 ZIP을 HWPX로 확정하지 않는 공개 format 경계.

use std::io::{Cursor, Write};

use rhwp::parser::{detect_format, parse_document, FileFormat, ParseError};
use zip::write::SimpleFileOptions;

fn make_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut output);
        let options = SimpleFileOptions::default();
        for (name, contents) in entries {
            writer.start_file(*name, options).expect("ZIP entry start");
            writer.write_all(contents).expect("ZIP entry write");
        }
        writer.finish().expect("ZIP finish");
    }
    output.into_inner()
}

#[test]
fn issue6534_detect_format_does_not_promote_xlsx_zip_to_hwpx() {
    let data = make_zip(&[
        ("[Content_Types].xml", b"<Types/>"),
        ("xl/workbook.xml", b"<workbook/>"),
    ]);

    assert_eq!(detect_format(&data), FileFormat::Unknown);
    let error = parse_document(&data).expect_err("XLSX must not enter the HWPX parser");
    assert!(
        matches!(error, ParseError::UnsupportedFormat { code, .. } if code == "UNSUPPORTED_FILE_FORMAT"),
        "XLSX ZIP must fail at format detection instead of HWPX MissingFile: {error}"
    );
}

#[test]
fn issue6534_detect_format_requires_both_hwpx_package_entries() {
    let only_content = make_zip(&[("Contents/content.hpf", b"<package/>")]);
    let only_header = make_zip(&[("Contents/header.xml", b"<head/>")]);

    assert_eq!(detect_format(&only_content), FileFormat::Unknown);
    assert_eq!(detect_format(&only_header), FileFormat::Unknown);
}
