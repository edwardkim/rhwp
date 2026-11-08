//! HWP3의 부호 있는 문단 들여쓰기를 공통 줄별 속성으로 옮기는 합성 파싱 계약.
//! 실물 문서의 시각 일치나 절대 화면 좌표를 고정하는 검사가 아니다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::paragraph::LineSeg;

fn document_bytes(indent: i16, stored_lines: bool) -> Vec<u8> {
    let mut bytes = b"HWP Document File V3.00 \x1a\x01\x02\x03\x04\x05".to_vec();
    bytes.extend_from_slice(&[0; 128 + 1008]);
    for _ in 0..7 {
        bytes.extend_from_slice(&1u16.to_le_bytes());
        let mut name = [0; 40];
        name[..4].copy_from_slice(&[0xB9, 0xD9, 0xC5, 0xC1]); // 바탕
        bytes.extend_from_slice(&name);
    }
    bytes.extend_from_slice(&0u16.to_le_bytes()); // 스타일 없음
    let text = if stored_lines {
        "ABCDEFGHIJKL".to_string()
    } else {
        "ABCD ".repeat(20)
    };
    bytes.push(0); // 이전 문단 모양을 상속하지 않음
    bytes.extend_from_slice(&(text.len() as u16 + 1).to_le_bytes());
    bytes.extend_from_slice(&(if stored_lines { 3u16 } else { 0 }).to_le_bytes());
    bytes.extend_from_slice(&[0; 7]);
    let mut char_shape = [0; 31];
    char_shape[..2].copy_from_slice(&250u16.to_le_bytes());
    char_shape[9..16].fill(100);
    bytes.extend_from_slice(&char_shape);
    let mut para_shape = [0; 187];
    para_shape[..2].copy_from_slice(&1250u16.to_le_bytes());
    para_shape[2..4].copy_from_slice(&250u16.to_le_bytes());
    para_shape[4..6].copy_from_slice(&indent.to_le_bytes());
    para_shape[6..8].copy_from_slice(&160u16.to_le_bytes());
    para_shape[172] = 1; // 한 단
    bytes.extend_from_slice(&para_shape);
    if stored_lines {
        for index in 0..3u16 {
            for value in [index * 4, 0, 250, 100 + index * 400, 0, 0, 0] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    for character in text.encode_utf16().chain(std::iter::once(13)) {
        bytes.extend_from_slice(&character.to_le_bytes());
    }
    bytes.extend_from_slice(&[0; 43]); // 문단 목록 끝
    bytes
}

#[test]
fn hwp3_signed_indentation_preserves_each_lines_source_role() {
    for stored_lines in [true, false] {
        for indent in [-625, 0, 625] {
            let document = rhwp::parser::hwp3::parse_hwp3(&document_bytes(indent, stored_lines))
                .expect("합성 HWP3 파싱");
            let paragraph = &document.sections[0].paragraphs[0];
            assert!(paragraph.line_segs.len() >= 3, "여러 줄 대조군");
            let expected_text = if stored_lines {
                "ABCDEFGHIJKL".to_string()
            } else {
                "ABCD ".repeat(20)
            };
            assert_eq!(paragraph.text, expected_text, "본문 보존");
            for (index, line) in paragraph.line_segs.iter().enumerate() {
                let expected = (indent > 0 && index == 0) || (indent < 0 && index > 0);
                assert_eq!(
                    line.tag & LineSeg::TAG_INDENTATION != 0,
                    expected,
                    "들여쓰기={indent}, 저장 줄={stored_lines}, 줄={index}"
                );
                assert_eq!(
                    line.tag & LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    LineSeg::TAG_SINGLE_SEGMENT_LINE,
                    "기존 단일 세그먼트 속성 보존"
                );
                if index > 0 {
                    assert!(line.text_start > paragraph.line_segs[index - 1].text_start);
                }
            }
        }
    }
}
