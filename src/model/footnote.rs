//! 각주/미주 (Footnote, Endnote, FootnoteShape)

use super::paragraph::Paragraph;
use super::*;

/// 각주 ('fn  ' 컨트롤)
///
/// [Task #1050] HWP5 CTRL_FOOTNOTE payload 전체 보존 (size=20):
/// number(UInt4) + beforeDecorationLetter(WChar) + afterDecorationLetter(WChar)
/// + numberShape(UInt4) + instanceId(UInt4, optional).
///
/// LIST_HEADER for Footnote (size=16): paraCount(SInt4) + property(UInt4) + 8 byte zero padding.
///
/// 참조: `hwplib::ControlFootnote` + `CtrlHeaderFootnote`.
#[derive(Debug, Default, Clone)]
pub struct Footnote {
    /// 각주 번호
    pub number: u16,
    /// 문단 리스트
    pub paragraphs: Vec<Paragraph>,
    /// 앞 장식 문자 (WChar, default 0 = 없음)
    pub before_decoration_letter: u16,
    /// 뒤 장식 문자 (WChar, default 0x0029 = ')')
    pub after_decoration_letter: u16,
    /// 번호 모양 (UInt4, default 0 = Digit)
    pub number_shape: u32,
    /// 문서 내 고유 식별자 (UInt4, optional)
    pub instance_id: u32,
    /// LIST_HEADER property (UInt4, default 0)
    pub list_header_property: u32,
}

/// 미주 ('en  ' 컨트롤) — [Task #1050] Footnote 와 동일 구조
#[derive(Debug, Default, Clone)]
pub struct Endnote {
    /// 미주 번호
    pub number: u16,
    /// 문단 리스트
    pub paragraphs: Vec<Paragraph>,
    /// 앞 장식 문자 (WChar)
    pub before_decoration_letter: u16,
    /// 뒤 장식 문자 (WChar)
    pub after_decoration_letter: u16,
    /// 번호 모양 (UInt4)
    pub number_shape: u32,
    /// 문서 내 고유 식별자 (UInt4)
    pub instance_id: u32,
    /// LIST_HEADER property (UInt4)
    pub list_header_property: u32,
}

/// 각주/미주 모양 (HWPTAG_FOOTNOTE_SHAPE)
#[derive(Debug, Clone, Default)]
pub struct FootnoteShape {
    /// 속성 비트 플래그
    pub attr: u32,
    /// 번호 모양
    pub number_format: NumberFormat,
    /// 사용자 기호
    pub user_char: char,
    /// 앞 장식 문자
    pub prefix_char: char,
    /// 뒤 장식 문자
    pub suffix_char: char,
    /// 시작 번호
    pub start_number: u16,
    /// 구분선 길이. 한컴 미주 기본값 noteLine="14692344"(전폭 sentinel)이
    /// i16 범위를 넘으므로 i32로 보존한다(i16 절단 시 0x2ff8=12280 → 짧은 구분선 버그).
    pub separator_length: i32,
    /// HWPX 원본 슬롯: 구분선 위 여백.
    pub separator_margin_top: HwpUnit16,
    /// HWP5 원본 슬롯: 구분선 위 여백. HWPX 경로의 과거 매핑값 보존에도 사용될 수 있다.
    pub separator_margin_bottom: HwpUnit16,
    /// HWP5/HWPX 원본 슬롯: 한컴 UI의 "구분선 아래" 값.
    pub note_spacing: HwpUnit16,
    /// 구분선 종류
    pub separator_line_type: u8,
    /// 구분선 굵기
    pub separator_line_width: u8,
    /// 구분선 색상
    pub separator_color: ColorRef,
    /// 번호 매기기 방식
    pub numbering: FootnoteNumbering,
    /// 배치 방법 (각주: 단 배치, 미주: 문서/구역 끝)
    pub placement: FootnotePlacement,
    /// 주석 내용 중 번호 코드의 모양을 위첨자로 출력할지 여부.
    pub number_code_superscript: bool,
    /// 텍스트에 이어 바로 출력할지 여부.
    pub print_inline_after_text: bool,
    /// HWP5 미문서화 2바이트. 한컴 UI의 "주석 사이" 값으로 사용된다.
    pub raw_unknown: u16,
}

impl FootnoteShape {
    /// HWP5 `attr` 비트에서 각주/미주 모양 semantic 필드를 갱신한다.
    pub fn apply_attr_fields_from_raw(&mut self) {
        self.number_format = Self::number_format_from_attr_code(self.attr & 0xff);
        self.placement = match (self.attr >> 8) & 0x03 {
            1 => FootnotePlacement::BelowText,
            2 => FootnotePlacement::RightColumn,
            _ => FootnotePlacement::EachColumn,
        };
        self.numbering = match (self.attr >> 10) & 0x03 {
            1 => FootnoteNumbering::RestartSection,
            2 => FootnoteNumbering::RestartPage,
            _ => FootnoteNumbering::Continue,
        };
        self.number_code_superscript = (self.attr & (1 << 12)) != 0;
        self.print_inline_after_text = (self.attr & (1 << 13)) != 0;
    }

    /// 각주/미주 모양 semantic 필드를 HWP5 `attr` 비트로 인코딩한다.
    pub fn encode_attr(&self) -> u32 {
        let number_format = Self::number_format_attr_code(self.number_format);
        let placement = match self.placement {
            FootnotePlacement::BelowText => 1,
            FootnotePlacement::RightColumn => 2,
            FootnotePlacement::EachColumn => 0,
        };
        let numbering = match self.numbering {
            FootnoteNumbering::RestartSection => 1,
            FootnoteNumbering::RestartPage => 2,
            FootnoteNumbering::Continue => 0,
        };

        let mut attr = self.attr & !0x3fff;
        attr |= number_format;
        attr |= (placement & 0x03) << 8;
        attr |= (numbering & 0x03) << 10;
        if self.number_code_superscript {
            attr |= 1 << 12;
        }
        if self.print_inline_after_text {
            attr |= 1 << 13;
        }
        attr
    }

    /// 표 134의 번호 모양 코드를 모델 값으로 변환한다.
    pub fn number_format_from_attr_code(code: u32) -> NumberFormat {
        match code & 0xff {
            0 => NumberFormat::Digit,
            1 => NumberFormat::CircledDigit,
            2 => NumberFormat::UpperRoman,
            3 => NumberFormat::LowerRoman,
            4 => NumberFormat::UpperAlpha,
            5 => NumberFormat::LowerAlpha,
            6 => NumberFormat::CircledUpperAlpha,
            7 => NumberFormat::CircledLowerAlpha,
            8 => NumberFormat::HangulSyllable,
            9 => NumberFormat::CircledHangulSyllable,
            10 => NumberFormat::HangulJamo,
            11 => NumberFormat::CircledHangulJamo,
            12 => NumberFormat::HangulDigit,
            13 => NumberFormat::HanjaDigit,
            14 => NumberFormat::CircledHanjaDigit,
            15 => NumberFormat::HanjaGapEul,
            16 => NumberFormat::HanjaGapEulHanja,
            0x80 | 17 => NumberFormat::FourSymbol,
            0x81 | 18 => NumberFormat::UserChar,
            _ => NumberFormat::Digit,
        }
    }

    /// 표 134의 번호 모양 코드로 변환한다.
    pub fn number_format_attr_code(format: NumberFormat) -> u32 {
        match format {
            NumberFormat::Digit => 0,
            NumberFormat::CircledDigit => 1,
            NumberFormat::UpperRoman => 2,
            NumberFormat::LowerRoman => 3,
            NumberFormat::UpperAlpha => 4,
            NumberFormat::LowerAlpha => 5,
            NumberFormat::CircledUpperAlpha => 6,
            NumberFormat::CircledLowerAlpha => 7,
            NumberFormat::HangulSyllable => 8,
            NumberFormat::CircledHangulSyllable => 9,
            NumberFormat::HangulJamo => 10,
            NumberFormat::CircledHangulJamo => 11,
            NumberFormat::HangulDigit => 12,
            NumberFormat::HanjaDigit => 13,
            NumberFormat::CircledHanjaDigit => 14,
            NumberFormat::HanjaGapEul => 15,
            NumberFormat::HanjaGapEulHanja => 16,
            NumberFormat::FourSymbol => 0x80,
            NumberFormat::UserChar => 0x81,
        }
    }

    /// HWPX/API 번호 모양 이름을 모델 값으로 변환한다.
    pub fn number_format_from_name(value: &str, fallback: NumberFormat) -> NumberFormat {
        match value {
            "digit" | "DIGIT" => NumberFormat::Digit,
            "circledDigit" | "CIRCLED_DIGIT" => NumberFormat::CircledDigit,
            "upperRoman" | "ROMAN_CAPITAL" => NumberFormat::UpperRoman,
            "lowerRoman" | "ROMAN_SMALL" => NumberFormat::LowerRoman,
            "upperAlpha" | "LATIN_CAPITAL" => NumberFormat::UpperAlpha,
            "lowerAlpha" | "LATIN_SMALL" => NumberFormat::LowerAlpha,
            "circledUpperAlpha" | "CIRCLED_LATIN_CAPITAL" => NumberFormat::CircledUpperAlpha,
            "circledLowerAlpha" | "CIRCLED_LATIN_SMALL" => NumberFormat::CircledLowerAlpha,
            "hangulSyllable" | "HANGUL_SYLLABLE" => NumberFormat::HangulSyllable,
            "circledHangulSyllable" | "CIRCLED_HANGUL_SYLLABLE" => {
                NumberFormat::CircledHangulSyllable
            }
            "hangulJamo" | "HANGUL_JAMO" => NumberFormat::HangulJamo,
            "circledHangulJamo" | "CIRCLED_HANGUL_JAMO" => NumberFormat::CircledHangulJamo,
            "hangulDigit" | "HANGUL_PHONETIC" => NumberFormat::HangulDigit,
            "hanjaDigit" | "IDEOGRAPH" => NumberFormat::HanjaDigit,
            "circledHanjaDigit" | "CIRCLED_IDEOGRAPH" => NumberFormat::CircledHanjaDigit,
            "hanjaGapEul" | "DECAGON_CIRCLE" => NumberFormat::HanjaGapEul,
            "hanjaGapEulHanja" | "DECAGON_CIRCLE_HANJA" => NumberFormat::HanjaGapEulHanja,
            "fourSymbol" | "SYMBOL" => NumberFormat::FourSymbol,
            "userChar" | "USER_CHAR" => NumberFormat::UserChar,
            _ => fallback,
        }
    }

    /// 한컴 UI "구분선 위": 본문과 주석 구분선 사이의 간격.
    pub fn separator_above_margin_hu(&self) -> HwpUnit16 {
        let hwpx_above = self.separator_margin_top.max(0);
        if hwpx_above != 0 {
            hwpx_above
        } else {
            self.separator_margin_bottom.max(0)
        }
    }

    /// 한컴 UI "구분선 아래": 주석 구분선과 첫 주석 내용 사이의 간격.
    pub fn separator_below_margin_hu(&self) -> HwpUnit16 {
        self.note_spacing.max(0)
    }

    /// 한컴 UI "각주/미주 사이": 앞 번호 주석 내용과 다음 번호 주석 내용 사이의 간격.
    pub fn between_notes_margin_hu(&self) -> u16 {
        self.raw_unknown
    }
}

/// 번호 형식
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum NumberFormat {
    #[default]
    Digit, // 1, 2, 3
    CircledDigit,          // ①, ②, ③
    UpperRoman,            // I, II, III
    LowerRoman,            // i, ii, iii
    UpperAlpha,            // A, B, C
    LowerAlpha,            // a, b, c
    CircledUpperAlpha,     // Ⓐ, Ⓑ, Ⓒ
    CircledLowerAlpha,     // ⓐ, ⓑ, ⓒ
    HangulSyllable,        // 가, 나, 다
    CircledHangulSyllable, // ㉮, ㉯, ㉰
    HangulJamo,            // ㄱ, ㄴ, ㄷ
    CircledHangulJamo,     // ㉠, ㉡, ㉢
    HangulDigit,           // 일, 이, 삼
    HanjaDigit,            // 一, 二, 三
    CircledHanjaDigit,     // 동그라미 一, 二, 三
    HanjaGapEul,           // 갑, 을, 병 ...
    HanjaGapEulHanja,      // 甲, 乙, 丙 ...
    FourSymbol,            // 4가지 문자 반복
    UserChar,              // 사용자 지정 문자 반복
}

/// 번호 매기기 방식
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FootnoteNumbering {
    #[default]
    /// 앞 구역에 이어서
    Continue,
    /// 현재 구역부터 새로 시작
    RestartSection,
    /// 쪽마다 새로 시작 (각주 전용)
    RestartPage,
}

/// 배치 방법
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FootnotePlacement {
    #[default]
    /// 각 단마다 따로 배열 / 문서의 마지막
    EachColumn,
    /// 통단으로 배열 / 구역의 마지막
    BelowText,
    /// 가장 오른쪽 단에 배열
    RightColumn,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footnote_default() {
        let note = Footnote::default();
        assert_eq!(note.number, 0);
        assert!(note.paragraphs.is_empty());
    }

    #[test]
    fn test_footnote_shape_default() {
        let shape = FootnoteShape::default();
        assert_eq!(shape.number_format, NumberFormat::Digit);
        assert_eq!(shape.numbering, FootnoteNumbering::Continue);
    }

    #[test]
    fn test_footnote_shape_attr_bits_follow_table_134() {
        let mut shape = FootnoteShape {
            attr: 0x81 | (1 << 8) | (2 << 10) | (1 << 12) | (1 << 13),
            ..Default::default()
        };

        shape.apply_attr_fields_from_raw();

        assert_eq!(shape.number_format, NumberFormat::UserChar);
        assert_eq!(shape.placement, FootnotePlacement::BelowText);
        assert_eq!(shape.numbering, FootnoteNumbering::RestartPage);
        assert!(shape.number_code_superscript);
        assert!(shape.print_inline_after_text);
        assert_eq!(
            shape.encode_attr() & 0x3fff,
            0x81 | (1 << 8) | (2 << 10) | (1 << 12) | (1 << 13)
        );
    }
}
