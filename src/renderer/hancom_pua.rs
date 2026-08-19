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
    // `img-start-001.hwp` p1 본문 bullet. 한글 2022 PDF 실측: 16갈래 물방울-꽃잎
    // 방사 asterisk — 공개 글꼴의 16각 asterisk 로 의미 보존한다.
    (0xF0090, "✺"),
    // `복학원서.hwp` 서명란. Hancom PDF: `(인)`.
    (0xF012B, "(인)"),
    // `hwp3-sample11`(HWP3→HWP5 변환본) p23 NVRAM 바이트 라벨 ⓪..⑨.
    // Hancom PDF `pdf/hwp3-sample11-2020.pdf` p23 실측: 라벨 줄이
    // ⓪ ① ② ③ ④ ⑤ ⑥ ⑦ ⑧ ⑨ ⓐ ⓑ 로 이어지고, 문서 본문은 같은 자리에
    // F0288 F0289 F028A ③(리터럴) F028C F028D F028E F028F F0290 F0291 ⓐ ⓑ 다.
    // 같은 쪽 아래 "Host-ID = ①+ⓒ+ⓓ+ⓔ" 줄이 F0289=① 을 한 번 더 확인해 준다.
    // F028B(=③)는 이 문서가 리터럴 ③ 을 써서 근거가 없으므로 넣지 않는다.
    (0xF0288, "⓪"),
    (0xF0289, "①"),
    (0xF028A, "②"),
    (0xF028C, "④"),
    (0xF028D, "⑤"),
    (0xF028E, "⑥"),
    (0xF028F, "⑦"),
    (0xF0290, "⑧"),
    (0xF0291, "⑨"),
    // 네모 숫자(U+F02B1~)는 이 표에 넣지 않는다 — 렌더는 원문 유지가 계약이다
    // (캡스톤 F-1: 원-안 글리프 매핑이 한컴 사각-안 정답지와 발산해 되돌림,
    // issue_3385/3385b 가 잠근다). 한글 2022 오라클 실측(1..7 = 네모 1..7,
    // U+F02C5 = 네모 12)은 텍스트 표면 표(composer::text_surface_replacement)에 반영.
    // `3191107`(육아기 근로시간 단축 신청서, 코퍼스 00447 문서) 안내문 3줄 bullet.
    // 한글 2022 PDF 실측: 작은 빈 마름모.
    (0xF02EC, "◇"),
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
    // 2025 행정업무운영 편람 p43 "(예시: ①, F03A8, F03A7 등)" 줄. 한글 2022 PDF
    // 실측: 둥근 네모 안 + / − — 쉼표 위치로 문서 순서(F03A8 가 둘째, F03A7 이
    // 셋째)를 확정했다. squared plus/minus 로 의미 보존.
    (0xF03A7, "⊟"),
    (0xF03A8, "⊞"),
    // HWP3→HWP5 변환본 `sample16-hwp5`의 빈 체크박스 bullet.
    (0xF03C5, "□"),
    // `36300012`(성동구 결재문서) p3 항목 bullet. 한글 2022 PDF 실측: 빈 네모
    // (둥근 모서리) — F03C5 와 같은 계열의 빈 네모로 보존한다.
    (0xF03DA, "□"),
    // `HWP3/HWP5/HWPX-password-123456` 공통 머리말. Hancom PDF: 한글과컴퓨터.
    (0xF03EF, "한"),
    (0xF03F0, "글"),
    (0xF03F1, "과"),
    (0xF03F2, "컴"),
    (0xF03F3, "퓨"),
    (0xF03F4, "터"),
    // `3191107`(육아기 근로시간 단축 신청서) 표 셀 제목 bullet("신 청 인" 등 4곳).
    // 한글 2022 PDF 실측: 빈 네모. (PDF 텍스트 층은 이 기호를 U+FFFF 로 재부호화
    // 하므로 텍스트 추출로는 못 찾고, 셀 앵커 좌표 절단으로 확정했다.)
    (0xF03FF, "□"),
    // 罫線(괘선) 조각 — `hwp3-sample11` 이 텍스트 다이어그램의 세로 묶음에 쓴다.
    // Hancom PDF `pdf/hwp3-sample11-2020.pdf` 실측:
    //   p6  `SUN OS 4.1.1 ━F0808 / F0810 / F0810 / 4.1.4 ━F080E` → ┐ │ │ ┘
    //   p22 `━━━F0807━━━` 와 그 아래 `F080C━>` → ┬ 와 └
    //   p129 세 줄 연속 F0806 / F0810 / F080C → ┌ │ └
    // `SO-SUEOP.hwpx`(Hancom PDF `pdf/SO-SUEOP-2024.pdf`)도 같은 묶음을 쓴다.
    (0xF0806, "┌"),
    (0xF0807, "┬"),
    (0xF0808, "┐"),
    (0xF080C, "└"),
    (0xF080E, "┘"),
    (0xF0810, "│"),
    // `복학원서.hwp` 마지막 안내줄의 점선 괘선 조각(2연속). 한글 2022 PDF 실측:
    // 작은 사각 점 ~9개가 이어진 대시 조각 — 4중 점선 괘선으로 보존한다.
    (0xF081C, "┈"),
    // `36382936`(지방행정의 달인 선발계획) p6 등 구분선 채움 문자(문서당 863회).
    // 한글 2022 PDF 실측: 위 가는 선 + 아래 굵은 선의 이중 가로 괘선 조각 —
    // 반복되어 이중 괘선을 이룬다. 이중 가로선으로 보존한다.
    (0xF0832, "═"),
    // 2025 행정업무운영 편람 p138~140 절차 항목 bullet. 한글 2022 PDF 실측:
    // 짧고 굵은 가로 막대 — 굵은 가로선으로 보존한다.
    (0xF0848, "━"),
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
            0xF0090, 0xF012B, 0xF02EC, 0xF02FB, 0xF02FC, 0xF031C, 0xF03A0, 0xF03A7, 0xF03A8,
            0xF03C5, 0xF03DA, 0xF03EF, 0xF03F4, 0xF03FF, 0xF081C, 0xF0832, 0xF0848,
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
        // 네모 숫자 가족(F02B1~)은 렌더 원문 유지 계약이라 이 표에 없어야 한다.
        assert_eq!(
            verified_hancom_pua_display('\u{F02B1}'),
            None,
            "네모 숫자는 렌더 원문 유지(캡스톤 F-1) — 이 표에 넣으면 안 됨",
        );
    }
}
