//! [#8912] HWP3 조합형 특수기호/괘선 복원 및 `ch == 29` 상호 참조 절단 방어 계약.
//!
//! src 쪽 단위 테스트는 unit-test-tier 정책(source-side 총량 동결)에 따라
//! 공개 API(`decode_johab`, `parse_hwp3`) 기반 통합 테스트로 이관했다.

use rhwp::model::control::{Control, FieldType};
use rhwp::parser::hwp3::johab::decode_johab;
use rhwp::parser::hwp3::parse_hwp3;

#[test]
fn decode_hwp3_all_line_chars_preserves_known_pua() {
    let expected: Vec<char> = [
        "\u{F0806}\u{F0807}\u{F0808}├┼┤\u{F080C}┴\u{F080E}\u{F080F}\u{F0810}",
        "\u{F0811}┳┓┣╋┫\u{F0817}┻┛\u{F081A}┃",
        "╔╦╗╠╬╣╚╩╝═║",
        "┍┯┑┝┿┥┕┷┙\u{F0827}│",
        "┎┰┒┠╂┨┖┸┚\u{F0832}┃",
        "╒╤╕╞╪╡╘╧╛═│",
        "╓╥╖╟╫╢╙╨╜─║",
    ]
    .iter()
    .flat_map(|row| row.chars())
    .collect();
    assert_eq!(expected.len(), 77);
    for (code, expected) in (0x3013..=0x305F).zip(expected) {
        assert_ne!(decode_johab(code), '?', "0x{code:04X}");
        assert_eq!(decode_johab(code), expected, "0x{code:04X}");
    }
    assert_eq!(decode_johab(0x3012), '?');
    assert_eq!(decode_johab(0x3060), '?');
    assert_eq!(decode_johab(0x3062), '“');
}

#[test]
fn decode_hwp3_issue_8912_symbols() {
    for (code, expected) in [
        (0x2051, '±'),
        (0x2053, '×'),
        (0x048A, '˚'),
        (0x223C, '∼'),
        (0x3151, '‧'),
        (0x2F66, '▶'),
        (0x2191, '↑'),
    ] {
        assert_ne!(decode_johab(code), '?', "0x{code:04X}");
        assert_eq!(decode_johab(code), expected, "0x{code:04X}");
    }
    assert_eq!(decode_johab(0x340D), '～');
}

#[test]
fn cross_reference_truncated_input_stops_before_creating_field() {
    let mut doc_prefix = Vec::from(&b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05"[..]);
    doc_prefix.resize(30 + 128 + 1008, 0);
    for _ in 0..7 {
        doc_prefix.extend_from_slice(&1u16.to_le_bytes());
        let mut name = [0u8; 40];
        name[..4].copy_from_slice(&[0xB9, 0xD9, 0xC5, 0xC1]);
        doc_prefix.extend_from_slice(&name);
    }
    doc_prefix.extend_from_slice(&0u16.to_le_bytes());
    doc_prefix.push(0);
    doc_prefix.extend_from_slice(&5u16.to_le_bytes());
    doc_prefix.extend_from_slice(&1u16.to_le_bytes());
    doc_prefix.extend_from_slice(&[0u8; 7 + 31]);
    let mut ps = [0u8; 187];
    ps[6..8].copy_from_slice(&160u16.to_le_bytes());
    ps[172] = 1;
    doc_prefix.extend_from_slice(&ps);
    doc_prefix.extend_from_slice(&[0u8; 14]);
    doc_prefix.extend_from_slice(&29u16.to_le_bytes());

    let mut xref = Vec::new();
    xref.extend_from_slice(&46u32.to_le_bytes());
    xref.extend_from_slice(&29u16.to_le_bytes());
    let mut payload = [0u8; 46];
    payload[1..7].copy_from_slice(b"TARGET");
    xref.extend_from_slice(&payload);

    for len in 0..=xref.len() {
        let mut bytes = doc_prefix.clone();
        bytes.extend_from_slice(&xref[..len]);
        if len == xref.len() {
            bytes.extend_from_slice(&13u16.to_le_bytes());
            bytes.extend_from_slice(&[0u8; 43]);
        }
        if len < xref.len() {
            if let Ok(doc) = parse_hwp3(&bytes) {
                let refs: Vec<_> = doc
                    .sections
                    .iter()
                    .flat_map(|s| &s.paragraphs)
                    .flat_map(|p| &p.controls)
                    .filter_map(|c| match c {
                        Control::Field(f) if f.field_type == FieldType::CrossRef => Some(f),
                        _ => None,
                    })
                    .collect();
                assert!(refs.is_empty(), "잘린 입력 길이 {len}");
            }
        } else {
            let doc = parse_hwp3(&bytes).unwrap();
            let refs: Vec<_> = doc
                .sections
                .iter()
                .flat_map(|s| &s.paragraphs)
                .flat_map(|p| &p.controls)
                .filter_map(|c| match c {
                    Control::Field(f) if f.field_type == FieldType::CrossRef => Some(f),
                    _ => None,
                })
                .collect();
            assert_eq!(refs.len(), 1);
            assert_eq!(refs[0].command, "Target:TARGET");
        }
    }
}
