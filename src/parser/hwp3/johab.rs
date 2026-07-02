//! 조합형 텍스트 변환 로직
//!
//! `johab_map.rs`의 테이블을 활용하여 실제 조합형 텍스트를 유니코드(UTF-8) 문자로
//! 디코딩하는 함수(`decode_johab`)를 제공한다.

use crate::parser::hwp3::johab_map;

pub fn decode_johab(ch: u16) -> char {
    if ch < 0x80 {
        return ch as u8 as char;
    } else if ch >= 0x8000 {
        // 조합형 한글 (상위 비트 1)
        let cho_idx = (ch >> 10) & 0x1F;
        let jung_idx = (ch >> 5) & 0x1F;
        let jong_idx = ch & 0x1F;

        let cho_map: [i32; 32] = [
            -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ];
        let jung_map: [i32; 32] = [
            -1, -1, -1, 0, 1, 2, 3, 4, -1, -1, 5, 6, 7, 8, 9, 10, -1, -1, 11, 12, 13, 14, 15, 16,
            -1, -1, 17, 18, 19, 20, -1, -1,
        ];
        let jong_map: [i32; 32] = [
            -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, -1, 17, 18, 19, 20, 21,
            22, 23, 24, 25, 26, 27, -1, -1,
        ];

        let cho = cho_map[cho_idx as usize];
        let jung = jung_map[jung_idx as usize];
        let mut jong = jong_map[jong_idx as usize];

        if cho != -1 && jung != -1 {
            if jong == -1 {
                jong = 0;
            }
            let uni_val = 0xAC00 + (cho * 21 * 28) + (jung * 28) + jong;
            if let Some(c) = std::char::from_u32(uni_val as u32) {
                return c;
            }
        }

        // 한자 및 기호 영역 (이진 탐색)
        if let Ok(idx) = johab_map::JOHAB_SYMBOLS.binary_search_by_key(&ch, |&(k, _)| k) {
            return johab_map::JOHAB_SYMBOLS[idx].1;
        }
    } else if ch >= 0x0080 {
        // [Task #741 Stage 5] HWP3 사적 graphic char 영역 (0x0080~0x7FFF).
        // 표준 KSSM 조합형 영역 (0x8000+) 외 한컴 사적 인코딩.
        // 매핑은 hwp3-sample10.hwp ↔ hwp3-sample10-hwp5.hwp cross-ref 로 도출.
        // Target: HWP5 변환본 IR 정합 (PUA 보존).
        if let Some(c) = decode_hwp3_extra(ch) {
            return c;
        }
    }

    // 매핑되지 않은 값
    '?'
}

/// HWP3 사적 graphic char (0x0080~0x7FFF 영역) → Unicode 매핑.
///
/// 한컴 변환본 (HWP3 → HWP5) 의 IR 과 정합. PUA (Private Use Area) 영역도
/// 변환본 정합 위해 그대로 보존.
///
/// 매핑 출처: hwp3-sample10.hwp ↔ hwp3-sample10-hwp5.hwp paragraph 별 cross-ref.
fn decode_hwp3_extra(ch: u16) -> Option<char> {
    // [Task #877 Stage 3] 로마숫자 대문자 Ⅰ~Ⅹ: 0x3590~0x3599 → U+2160~U+2169.
    // sample16 (hwp3-sample16.hwp) 의 cross-ref 로 도출. 한컴 HWP5 변환본의
    // paragraph 26/31/36/44 ("Ⅰ. 사업개요", "Ⅱ. 제안 일반사항", "Ⅲ ...", "Ⅳ ...") 정합.
    if (0x3590..=0x3599).contains(&ch) {
        return char::from_u32(0x2160 + (ch - 0x3590) as u32);
    }
    // HWP3 사적 원문자 계열. 연속 코드가 ①~⑩에 대응한다.
    if (0x36E7..=0x36F0).contains(&ch) {
        return char::from_u32(0x2460 + (ch - 0x36E7) as u32);
    }
    let codepoint: u32 = match ch {
        0x0081 => 0x201C,  // 왼쪽 큰따옴표
        0x0082 => 0x201D,  // 오른쪽 큰따옴표
        0x301E => 0xF0811, // 한컴 PUA - 관계도 가지 선문자
        0x301C => 0xF080F, // 한컴 PUA - 굵은 가로선 (94.5% 발생)
        0x3024 => 0xF0817, // 한컴 PUA - 관계도 하단 가지 선문자
        0x3027 => 0xF081A, // 한컴 PUA - 관계도 가로 선문자
        0x3404 => 0x2024,  // 한 점 리더
        0x3446 => 0x2192,  // 오른쪽 화살표
        0x35E1 => 0x2500,  // 상자 그리기 가로선
        0x303D => 0xF0827, // 한컴 PUA
        0x3479 => 0x25B7,  // ▷ WHITE RIGHT-POINTING TRIANGLE
        0x347A => 0x25B6,  // ▶ BLACK RIGHT-POINTING TRIANGLE
        0x3441 => 0x25A0,  // ■ BLACK SQUARE
        // [Task #1105] sample16 글머리 prefix.
        // HWP3 0x3366 은 한컴 HWP5 변환본에서 U+F03C5 로 보존되고, 렌더러가
        // 이를 한컴오피스 표시값인 □(U+25A1)로 확장한다. 여기서 ○로 직접
        // 낮추면 HWP3 원본만 정답지와 다른 bullet 로 보이므로 PUA를 보존한다.
        0x3366 => 0xF03C5,
        _ => return None,
    };
    char::from_u32(codepoint)
}
