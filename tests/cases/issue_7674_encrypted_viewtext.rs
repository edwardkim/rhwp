//! #7674: 비밀번호 보호와 ViewText 선택은 독립된 계약이다.
//! 실제 비공개 문서 대신 서로 다른 본문을 가진 CFB를 메모리에서 구성한다.
use rhwp::model::document::{Document, HwpVersion, Section};
use rhwp::model::paragraph::Paragraph;
use rhwp::parser::{self, cfb_reader::CfbReader, crypto::CryptoError, ParseError};
use std::io::{Cursor, Read, Write};

const PASSWORD: &[u8] = b"public-test-input-7674";
type Compound = cfb::CompoundFile<Cursor<Vec<u8>>>;

fn document(texts: &[&str], compressed: bool) -> Document {
    let mut doc = Document::default();
    doc.header.version = HwpVersion {
        major: 5,
        minor: 0,
        build: 1,
        revision: 7,
    };
    doc.header.compressed = compressed;
    doc.header.flags = 0x4000 | u32::from(compressed);
    doc.doc_properties.section_count = texts.len() as u16;
    doc.sections = texts
        .iter()
        .map(|text| Section {
            paragraphs: vec![Paragraph {
                text: (*text).into(),
                char_count: text.encode_utf16().count() as u32 + 1,
                ..Default::default()
            }],
            ..Default::default()
        })
        .collect();
    doc
}

fn stream(cfb: &mut Compound, path: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    cfb.open_stream(path)
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    bytes
}

fn write_stream(cfb: &mut Compound, path: &str, bytes: &[u8]) {
    cfb.create_storage_all(std::path::Path::new(path).parent().unwrap())
        .unwrap();
    cfb.create_stream(path).unwrap().write_all(bytes).unwrap();
}

fn fixture(compressed: bool, encrypted: bool, body1: bool) -> Vec<u8> {
    let encode = |doc: &Document| {
        if encrypted {
            rhwp::serializer::serialize_hwp_with_password(doc, PASSWORD).unwrap()
        } else {
            rhwp::serializer::serialize_document(doc).unwrap()
        }
    };
    let mut body = Compound::open(Cursor::new(encode(&document(
        &["old zero", "old one"],
        compressed,
    ))))
    .unwrap();
    let mut view = Compound::open(Cursor::new(encode(&document(
        &["visible zero", "visible one"],
        compressed,
    ))))
    .unwrap();
    for i in 0..2 {
        let raw = stream(&mut view, &format!("/BodyText/Section{i}"));
        write_stream(&mut body, &format!("/ViewText/Section{i}"), &raw);
    }
    if !body1 {
        body.remove_stream("/BodyText/Section1").unwrap();
    }
    body.into_inner().into_inner()
}

// 미사용 FAT slot에 기존 chain 참조를 중복시켜 본문 변경 없이 strict open을 거부시킨다.
fn force_lenient(mut bytes: Vec<u8>) -> Vec<u8> {
    let sector_size = 1usize << u16::from_le_bytes(bytes[30..32].try_into().unwrap());
    let sectors = (bytes.len() - 512) / sector_size;
    let fat_count = u32::from_le_bytes(bytes[44..48].try_into().unwrap()) as usize;
    let fat_ids: Vec<usize> = (0..fat_count)
        .map(|i| u32::from_le_bytes(bytes[76 + 4 * i..80 + 4 * i].try_into().unwrap()) as usize)
        .collect();
    let offset = |id: usize| {
        512 + fat_ids[id / (sector_size / 4)] * sector_size + (id % (sector_size / 4)) * 4
    };
    let target = (0..sectors)
        .map(|id| u32::from_le_bytes(bytes[offset(id)..offset(id) + 4].try_into().unwrap()))
        .find(|id| (*id as usize) < sectors)
        .expect("live FAT link");
    let padding = offset(sectors);
    bytes[padding..padding + 4].copy_from_slice(&target.to_le_bytes());
    assert!(
        CfbReader::open(&bytes).is_err(),
        "must exercise lenient open"
    );
    bytes
}

fn texts(doc: &Document) -> Vec<String> {
    doc.sections
        .iter()
        .map(|s| s.paragraphs.iter().map(|p| p.text.as_str()).collect())
        .collect()
}

fn assert_viewtext_roundtrip(body1: bool) {
    for compressed in [false, true] {
        for lenient in [false, true] {
            let bytes = fixture(compressed, true, body1);
            assert_eq!(CfbReader::open(&bytes).unwrap().section_count(), 2);
            let bytes = if lenient { force_lenient(bytes) } else { bytes };
            let doc = parser::parse_document_with_password(&bytes, PASSWORD).unwrap();
            assert_eq!(
                texts(&doc),
                ["visible zero", "visible one"],
                "compressed={compressed}, body1={body1}, lenient={lenient}"
            );
            let saved = rhwp::serializer::serialize_document(&doc).unwrap();
            assert_eq!(texts(&parser::parse_document(&saved).unwrap()), texts(&doc));
        }
    }
}

#[test]
fn encrypted_viewtext_opens_sections_missing_from_bodytext() {
    assert_viewtext_roundtrip(false);
}

#[test]
fn encrypted_viewtext_wins_when_both_storages_have_the_section() {
    assert_viewtext_roundtrip(true);
}

#[test]
fn plaintext_viewtext_policy_matches_password_policy() {
    let bytes = fixture(true, false, false);
    assert_eq!(
        texts(&parser::parse_document(&bytes).unwrap()),
        ["visible zero", "visible one"]
    );
}

#[test]
fn viewtext_does_not_bypass_password_validation() {
    for lenient in [false, true] {
        let bytes = fixture(true, true, false);
        let bytes = if lenient { force_lenient(bytes) } else { bytes };
        assert!(matches!(
            parser::parse_document(&bytes),
            Err(ParseError::EncryptedDocument)
        ));
        assert!(matches!(
            parser::parse_document_with_password(&bytes, b"incorrect"),
            Err(ParseError::CryptoError(CryptoError::WrongPassword))
        ));
    }
}

#[test]
fn absent_or_stub_viewtext_preserves_bodytext() {
    for compressed in [false, true] {
        for stub in [None, Some(Vec::new()), Some(vec![0xff; 16])] {
            let mut cfb = Compound::open(Cursor::new(fixture(compressed, true, true))).unwrap();
            for i in 0..2 {
                let path = format!("/ViewText/Section{i}");
                match &stub {
                    None => cfb.remove_stream(&path).unwrap(),
                    Some(raw) => write_stream(
                        &mut cfb,
                        &path,
                        &rhwp::password_crypto::encrypt_hwp5_stream(raw, PASSWORD),
                    ),
                }
            }
            let bytes = cfb.into_inner().into_inner();
            for lenient in [false, true] {
                let bytes = if lenient {
                    force_lenient(bytes.clone())
                } else {
                    bytes.clone()
                };
                assert_eq!(
                    texts(&parser::parse_document_with_password(&bytes, PASSWORD).unwrap()),
                    ["old zero", "old one"]
                );
            }
        }
    }
}

#[test]
fn viewtext_output_limit_is_not_hidden_by_bodytext_fallback() {
    // 현재 문서 열기의 단일 출력 예산(256 MiB)을 넘는 작은 압축 스트림.
    // 반복 입력은 chunk로 쓰므로 fixture 생성 자체에는 거대 평문 할당이 필요 없다.
    let mut deflate = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::fast());
    let chunk = [0u8; 64 * 1024];
    for _ in 0..4097 {
        deflate.write_all(&chunk).unwrap();
    }
    let ciphertext =
        rhwp::password_crypto::encrypt_hwp5_stream(&deflate.finish().unwrap(), PASSWORD);
    let mut cfb = Compound::open(Cursor::new(fixture(true, true, true))).unwrap();
    write_stream(&mut cfb, "/ViewText/Section0", &ciphertext);
    let bytes = cfb.into_inner().into_inner();
    for lenient in [false, true] {
        let bytes = if lenient {
            force_lenient(bytes.clone())
        } else {
            bytes.clone()
        };
        assert!(matches!(
            parser::parse_document_with_password(&bytes, PASSWORD),
            Err(ParseError::CryptoError(
                CryptoError::DecompressedStreamLimitExceeded { .. }
            ))
        ));
    }
}
