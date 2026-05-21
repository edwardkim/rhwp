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
    /// 구분선 길이
    pub separator_length: HwpUnit16,
    /// 구분선 위 여백
    pub separator_margin_top: HwpUnit16,
    /// 구분선 아래 여백
    pub separator_margin_bottom: HwpUnit16,
    /// 주석 사이 여백
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
    /// 미문서화 2바이트 (라운드트립 보존용)
    pub raw_unknown: u16,
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
}
