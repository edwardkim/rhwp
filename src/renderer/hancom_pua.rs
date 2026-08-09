//! 한컴 전용 PUA 기호의 **검증된 표시 대체 표**.
//!
//! 이 표는 `pua_oldhangul`과 의도적으로 분리한다. 후자는 공개된
//! HanyangPuaTableProject의 옛한글(BMP PUA) 매핑이고, 이 모듈은 HWP 97의 HNC
//! 기호와 전용 HFT 글꼴에만 존재하는 glyph를 Hancom PDF 대조로 확인해 넣는
//! 좁은 호환 표다.
//!
//! PUA는 글꼴별 사적 영역이므로, 코드 포인트 범위만으로 의미를 추정해서는 안 된다.
//! 새 항목은 반드시 실제 문서·Hancom PDF·회귀 테스트를 함께 남긴 뒤 추가한다.

/// 한컴 PDF 대조로 의미가 확정된 PUA 코드 포인트와 공개 글꼴용 표시 문자열.
///
/// 코드 포인트 오름차순으로 유지해 이진 탐색한다. 원문 IR은 이 표로 바꾸지 않고,
/// SVG/Canvas/HTML paint 및 폭 측정에만 투영한다.
static VERIFIED_HANCOM_PUA_DISPLAY: &[(u32, &str)] = &[
    // `복학원서.hwp` 서명란. Hancom PDF: `(인)`.
    (0xF012B, "(인)"),
    // `pau-004.hwp`/한컴 문자표와 `issue2007_nested_cell_pagination_42065.hwp` 중첩 표 글머리표.
    // HCR Dotum/Hancom PDF: small right-pointing triangle. 공개 글꼴에서 raw PUA는 두부가 된다.
    (0xF02FB, "▸"),
    // 2025 행정업무운영 편람 p15 callout bullet. Hancom PDF: right pointer.
    (0xF02FC, "►"),
    // 2025 행정업무운영 편람 p08 TOC bullet. Hancom PDF: filled square.
    (0xF031C, "■"),
    // `HWP5-nopassword-123456.{hwp,hwpx}` 하이퍼텍스트 안내 문장. Hancom
    // PDF의 Enter-key pictogram을 공개 글꼴의 줄바꿈 화살표로 의미 보존한다.
    (0xF03A0, "↵"),
    // HWP3→HWP5 변환본 `sample16-hwp5`의 빈 체크박스 bullet.
    (0xF03C5, "□"),
    // `HWP3/HWP5/HWPX-password-123456` 공통 머리말. Hancom PDF: 한글과컴퓨터.
    (0xF03EF, "한"),
    (0xF03F0, "글"),
    (0xF03F1, "과"),
    (0xF03F2, "컴"),
    (0xF03F3, "퓨"),
    (0xF03F4, "터"),
];

/// 검증된 한컴 기호에 대한 공개 글꼴 표시 대체값.
///
/// 미등록 PUA는 `None`으로 남긴다. 잘못된 의미를 지어내는 일반 범위 매핑보다,
/// 검증 대상을 발견·등록하는 편이 문서 충실도에 안전하다.
pub(crate) fn verified_hancom_pua_display(ch: char) -> Option<&'static str> {
    let code_point = ch as u32;
    VERIFIED_HANCOM_PUA_DISPLAY
        .binary_search_by_key(&code_point, |(code, _)| *code)
        .ok()
        .map(|index| VERIFIED_HANCOM_PUA_DISPLAY[index].1)
}

#[cfg(test)]
mod tests {
    use super::verified_hancom_pua_display;

    #[test]
    fn verified_table_is_sorted_and_does_not_guess_unknown_pua() {
        for code_point in [
            0xF012B, 0xF02FB, 0xF02FC, 0xF031C, 0xF03A0, 0xF03C5, 0xF03EF, 0xF03F4,
        ] {
            assert!(
                verified_hancom_pua_display(char::from_u32(code_point).unwrap()).is_some(),
                "검증된 U+{code_point:05X}가 표에서 누락됨"
            );
        }
        assert_eq!(
            verified_hancom_pua_display('\u{F03E0}'),
            None,
            "인접 PUA를 근거 없이 같은 기호군으로 추정하면 안 됨",
        );
    }
}
