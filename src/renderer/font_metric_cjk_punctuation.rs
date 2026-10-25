//! 확인한 실제 프로그램의 기호·CJK 구두점 cmap. 미수집 메트릭과 glyph 결측을 구분한다.
//! 글꼴 바이트는 포함하지 않는다. 세 KoPub 굵기와 HCR 정체/굵기의 coverage가 각각 같다.
// HANBatang.ttf; SHA-256 c5d544769d832477d48b7318222b9c3c2eced25455f7ba9acb23060dcecac104
// HANBatangB.ttf; SHA-256 0d355f9766ffd1c7ec7648260611ddea9e12ea3c610423e552d52b921fc0f29f
// KoPub Batang Bold.ttf; SHA-256 94de26a11aa25619526ac8183fde4f46e74e22c527acb42aea2e93f5313bbc5b
// KoPub Batang Light.ttf; SHA-256 b7438d61595fe9584706ab2cb3c9aa2acdc486d61c8299374a30b3e77bc2045f
// KoPub Batang Medium.ttf; SHA-256 7a798592beb0d35bb557b010c2794cbcc988e89fcbcee0eb5edbb35161141ab4
// KoPub Dotum Bold.ttf; SHA-256 967efe18063dd971c63dde3e8fcb247e0fcf16486a24ba2e284a716e44b90275
// KoPub Dotum Light.ttf; SHA-256 018a0f18613a98d446696ea3d36194e288d5d8004aa58327789c830f6179e23d
// KoPub Dotum Medium.ttf; SHA-256 d0adfdcb40b7e73389128cc0222b53a91e5aa0e653f89967485a1f8b46124eb7
// KoPubWorld Batang Bold.ttf; SHA-256 72122f815cad001c97269bb99337647336b5597d2c3f749a49fb879c9e6ae9ff
// KoPubWorld Batang Light.ttf; SHA-256 e3ee21a86b6a6728c567a95aaebd8883480f27ce4f230207b0d7266b5cb3fb18
// KoPubWorld Batang Medium.ttf; SHA-256 f2c31ffd195ea0fe74c9e720592930ed528c52ee7876baffaeff24d42c133d6d
// KoPubWorld Dotum Bold.ttf; SHA-256 896a4952ab3c28108c095c638548989a82295cf644bcced1d4c53f5616a90db6
// KoPubWorld Dotum Light.ttf; SHA-256 069494cce21a4222c88e537f256b6f46fee209375aba769f82431b2d382bc84f
// KoPubWorld Dotum Medium.ttf; SHA-256 6269624bd0c5ae8746a8e731b8087f056088af930a7bbc0efb76902bf732a293

pub(super) fn primary_has_glyph(family: &str, _bold: bool, italic: bool, ch: char) -> Option<bool> {
    let cjk_offset = (ch as u32)
        .checked_sub(0x3000)
        .filter(|offset| *offset < 64);
    if ch != '\u{2024}' && cjk_offset.is_none() {
        return None;
    }
    if italic {
        return None;
    }
    let primary = family
        .split(',')
        .next()?
        .trim()
        .trim_matches('\'')
        .trim_matches('"');
    let name = primary.to_ascii_lowercase().replace(' ', "");
    let base = ["light", "medium", "bold"]
        .iter()
        .find_map(|suffix| name.strip_suffix(suffix))
        .unwrap_or(&name);
    // 위에 기록한 열두 원본 프로그램 모두 U+2024 cmap이 없다.
    // 독립 편람 Print는 선언 체인의 HBATANG으로 공급한다. 부분 폭 표의
    // 미수집을 결측으로 추정하거나 U+2027의 실제 KoPub hit를 바꾸지 않는다.
    if ch == '\u{2024}'
        && matches!(
            base,
            "kopubbatang"
                | "kopubdotum"
                | "kopub바탕체"
                | "kopub돋움체"
                | "kopubworldbatang"
                | "kopubworlddotum"
                | "kopubworld바탕체"
                | "kopubworld돋움체"
        )
    {
        return Some(false);
    }
    let offset = cjk_offset?;
    let coverage: u64 = match base {
        "kopubbatang" | "kopubdotum" | "kopub바탕체" | "kopub돋움체" => 0x00000000003bff0f,
        "kopubworldbatang" | "kopubworlddotum" | "kopubworld바탕체" | "kopubworld돋움체" => {
            0x0000c000003bff0f
        }
        "hcrbatang" | "함초롬바탕" => 0xffffffffffffffff,
        _ => return None,
    };
    Some(coverage & (1_u64 << offset) != 0)
}
