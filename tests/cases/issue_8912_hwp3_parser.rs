//! 관계형·고정형: HWP3 문자 보존·저장 왕복과 독립 한컴 저장 코드의 golden 계약.
//! 주석 갱신: 2026-11-14T19:19:39+09:00; 직전 파일 커밋:
//! 2026-11-14T13:53:44+09:00, 5c68397caf09a289e137e573b8282cc73c097460.
//! 2026-11-14 측정 raw(96dpi, 2px 실루엣): 정상 합성 specimen 전 1쪽
//! Native 98.56783% / fresh WASM 98.55072%; 전쪽 최저도 각각 같다.
//! 근거: mydocs/pr/assets/pr9070/README.md. 수동 한컴 2020 Print와 확대 판독 완료.
//! 절단 입력은 파서·보안 계약이며 위 정상 specimen의 시각 수치를 재인용하지 않는다.
//! [#8912] HWP3 조합형 특수기호/괘선 복원 및 `ch == 29` 상호 참조 절단 방어 계약.
//!
//! src 쪽 단위 테스트는 unit-test-tier 정책(source-side 총량 동결)에 따라
//! 공개 API(`decode_johab`, `parse_hwp3`) 기반 통합 테스트로 이관했다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::{Control, FieldType};
use rhwp::parser::hwp3::johab::decode_johab;
use rhwp::parser::hwp3::parse_hwp3;

#[test]
fn decode_hwp3_all_line_chars_preserves_known_pua() {
    let expected: Vec<char> = [
        // 독립 한컴 변환본의 실제 hp:t 저장값. 제품 매핑 상수를 재사용하지 않는다.
        "\u{F0806}\u{F0807}\u{F0808}\u{F0809}\u{F080A}\u{F080B}\u{F080C}\u{F080D}\u{F080E}\u{F080F}\u{F0810}",
        "\u{F0811}\u{F0812}\u{F0813}\u{F0814}\u{F0815}\u{F0816}\u{F0817}\u{F0818}\u{F0819}\u{F081A}\u{F081B}",
        "\u{F0308}\u{F0309}\u{F030A}\u{F030B}\u{F030C}\u{F030D}\u{F030E}\u{F030F}\u{F0310}\u{F081C}\u{F081D}",
        "\u{F081E}\u{F081F}\u{F0820}\u{F0821}\u{F0822}\u{F0823}\u{F0824}\u{F0825}\u{F0826}\u{F0827}\u{F0828}",
        "\u{F0829}\u{F082A}\u{F082B}\u{F082C}\u{F082D}\u{F082E}\u{F082F}\u{F0830}\u{F0831}\u{F0832}\u{F0833}",
        "\u{F0834}\u{F0835}\u{F0836}\u{F0837}\u{F0838}\u{F0839}\u{F083A}\u{F083B}\u{F083C}\u{F083D}\u{F083E}",
        "\u{F083F}\u{F0840}\u{F0841}\u{F0842}\u{F0843}\u{F0844}\u{F0845}\u{F0846}\u{F0847}\u{F0848}\u{F0849}",
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
fn hwp3_specimen_keeps_line_characters_and_paragraph_order_after_both_saves() {
    let source = include_bytes!("../../samples/pr9070/hwp3-linechar-specimen.hwp");
    let mut core = DocumentCore::from_bytes(source).unwrap();
    let text = |core: &DocumentCore| -> Vec<String> {
        core.document()
            .sections
            .iter()
            .flat_map(|s| &s.paragraphs)
            .map(|p| p.text.clone())
            .collect()
    };
    let before = text(&core);
    assert!(before.first().unwrap().starts_with("HWP3 line characters:"));
    let lines: Vec<_> = before.iter().filter(|p| p.starts_with("set ")).collect();
    assert_eq!(lines.len(), 7);
    for (row, paragraph) in lines.iter().enumerate() {
        let expected: String = (0..11)
            .map(|column| decode_johab(0x3013 + (row * 11 + column) as u16))
            .collect();
        assert!(paragraph.ends_with(&expected), "row {row}: {paragraph:?}");
    }
    for _ in 0..2 {
        for saved in [
            core.export_hwpx_native().unwrap(),
            core.export_hwp_native().unwrap(),
        ] {
            let reloaded = DocumentCore::from_bytes(&saved).unwrap();
            assert_eq!(text(&reloaded), before, "문자 소실·중복·순서 변경");
        }
        core = DocumentCore::from_bytes(&core.export_hwpx_native().unwrap()).unwrap();
    }
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
