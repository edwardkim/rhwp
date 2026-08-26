//! `ColorRef` 도메인 규칙.
//!
//! HWP 계열은 Windows `COLORREF` 규약을 따른다 — 하위 3바이트가 `0x00BBGGRR` 이고
//! **상위 바이트가 0이 아니면 "색 없음/자동"**(`CLR_INVALID`/`CLR_DEFAULT`)이다.
//! 이 규칙이 파서·렌더러·판정기에 흩어져 조금씩 다르게 재구현되면서
//! #3546·#3557·#4155 가 반복됐다. 정의는 여기 하나뿐이다.

use super::ColorRef;

/// "색 없음/자동" sentinel.
///
/// 추정이 아니라 실측 정본이다 — HWPX `shadeColor="none"` 이 파싱되는 값
/// (`parser/hwpx/utils.rs` `parse_color_str`), 한/글이 HML 에 쓰는 `4294967295`
/// (`samples/hml/*.hml` 2/2), 한컴산 HWP5 코퍼스 380건의 `CHAR_SHAPE` 음영색
/// `0xffffffff` × 22,189 (검정 `0x00000000` 은 전수에서 0건, #4155 실측).
pub const NONE: ColorRef = 0xFFFF_FFFF;

/// 확정된 불투명 RGB 만 돌려준다. 상위 바이트가 0이 아니면 "색 없음/자동"이다.
///
/// 모르는 색으로는 아무것도 단정하지 않는다 — `renderer::style_resolver` 의 채우기
/// 해소가 쓰는 판정과 같다.
pub fn opaque_rgb(color: ColorRef) -> Option<ColorRef> {
    ((color >> 24) == 0).then_some(color & 0x00FF_FFFF)
}

/// 글자 음영이 **실제로 그려지는** 색인지. `None` 이면 그리지 않는다.
///
/// 렌더 백엔드 5곳(`renderer::svg`·`web_canvas`·`html`·`skia::text_replay`·
/// `canvaskit_policy`)이 각자 재구현하던 판정의 정본이다. 판정기가 렌더러보다 더 많은
/// 것을 음영으로 세면 그 차이가 곧 오탐이다.
///
/// [`NONE`] 외에 두 값을 더 "없음"으로 본다.
/// - 흰색 `0x00FFFFFF` — 종이색이라 칠해도 안 보인다. HWP5 파서의 음영색 폴백이었다.
/// - 검정 `0x00000000` — 여러 파서·빌더가 "미지정" 자리에 남기던 값이다. 검정 글자는
///   문서의 압도적 다수이므로 이를 음영으로 믿으면 정상 문서가 통째로 오탐된다
///   (은닉 텍스트 판정 실측: 351개 표본 중 HWP3 문서 17개에서 31,907건 오탐).
///
/// **저장 축에는 쓰지 않는다.** 라이터는 "그려지는가"가 아니라 "값을 보존하는가"를
/// 묻는 다른 질문이다 — 흰색 음영은 한컴산 코퍼스에 49건 실재하고 그대로 저장돼야 한다.
pub fn char_shade(color: ColorRef) -> Option<ColorRef> {
    let rgb = opaque_rgb(color)?;
    (rgb != 0x00FF_FFFF && rgb != 0x0000_0000).then_some(rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_no_shade_sentinel_is_recognized() {
        assert_eq!(char_shade(NONE), None, "한컴/HWPX \"none\"/한글 HML");
        assert_eq!(char_shade(0x00FF_FFFF), None, "흰색 = 종이색");
        assert_eq!(
            char_shade(0x0000_0000),
            None,
            "미지정 잔재 — 31,907건 오탐원"
        );
        assert_eq!(char_shade(0x00D8_D8D8), Some(0x00D8_D8D8), "실제 음영");
    }

    #[test]
    fn opaque_rgb_rejects_any_nonzero_high_byte() {
        assert_eq!(opaque_rgb(0x0012_3456), Some(0x0012_3456));
        assert_eq!(opaque_rgb(0xFF00_0000), None);
        assert_eq!(opaque_rgb(NONE), None);
    }
}
