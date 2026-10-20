//! 원 face·굵기·같은 계열의 문자 fallback을 Native와 Canvas가 공유한다.

use super::drop_noto_sans_kr_extralight;

/// CSS generic fallback 반환 (serif 또는 sans-serif)
///
/// 폰트 이름에 명조/바탕/궁서 등 세리프 계열 키워드가 포함되면 "serif",
/// 그 외에는 "sans-serif"를 반환한다.
pub fn generic_fallback(font_family: &str) -> &'static str {
    // 원래 돋움·굴림의 획 보완은 현대 시스템 고딕의 선택과 구분한다.
    // 실제 요청 face/설치 별칭은 호출자의 체인 앞에 유지한다. 부재할 때만
    // 기존 한컴 돋움 밀도 측정에 근거한 가벼운 대체 face를 우선한다.
    if matches!(font_family, "돋움" | "Dotum" | "굴림" | "Gulim") {
        return "'Noto Sans KR ExtraLight','Noto Sans KR','Malgun Gothic','맑은 고딕','Apple SD Gothic Neo','HCR Dotum','함초롬돋움','Pretendard','HCR Batang','함초롬바탕','HCR Batang Ext','함초롬바탕 확장','HCR Batang ExtB','함초롬바탕 확장B','RHWP Old Hangul',sans-serif";
    }
    // 일반 고딕은 플랫폼 face 다음에 같은 계열의 함초롬돋움을 둔다.
    // 원 face의 일부 문자가 없을 때 범용 Noto보다 먼저 그 문자를 공급한다.
    // #7336 Print의 반각 가운뎃점은 실제 HCRDotum 프로그램을 사용한다.
    // 명조·고정폭과 KoPub Light의 선행 전용 face는 별도 계약을 유지한다.
    // Task #727 (F-1): 확장 함초롬 family는 chain 후반에 둔다.
    // 한컴 자체 PUA (사각 안 숫자 U+F02B1~F02C4 등) 글리프는 표준 한글 폰트
    // (Malgun Gothic, Noto Sans KR 등) 에 없어 .notdef tofu 가 나온다.
    //
    // [#4086] 어느 family 가 그 글리프를 갖는지 cmap 실측 (한글 2022 동봉 8종):
    //
    //   HCR Batang / HCR Dotum (일반)  59,330 자 — 한글·CJK 통합/확장A·BMP PUA
    //                                   **U+F02B1~F02C4 20 자 전부 보유**
    //   HCR Batang ExtB (확장B)        42,799 자 — CJK 확장B(U+20000~) 전담
    //                                   원문자 대역 **미보유**
    //   HCR Batang Ext / Dotum Ext     3,535 자 — 평면 1 희귀 스크립트 전담
    //                                   (U+10000~), PUA 무관
    //
    // 즉 PUA 원문자의 소재는 **일반**이고, 확장·확장B 는 그 목적과 무관하다
    // (원 주석이 확장B 를 PUA 보유자로 적었던 것은 사실과 반대다). 세 family 의
    // 담당 대역은 서로 겹치지 않으므로(일반∩확장 0 자, 일반∩확장B 2 자 —
    // U+0020·U+00A0 뿐) 나열 순서는 무엇이 매칭되는지를 바꾸지 않는다. 목적을
    // 드러내도록 일반 → 확장 → 확장B 순으로 적는다.
    //
    // 한글 본문 영역은 1순위 폰트가 글리프를 가지면 chain 우선순위에 의해 1순위를
    // 쓰므로 영향 0. 글리프 부재 시에만 함초롬이 매칭된다.
    if font_family.is_empty() {
        // 일반 고딕: 플랫폼 face → 같은 계열 함초롬 → 범용 Noto → 확장 대역.
        // Task #1224: 시스템 고딕(맑은고딕/Apple) 부재 환경(Linux/CI)에서 폴백되는
        // 'Noto Sans KR'(CJK Regular)의 획이 한컴 돋움보다 +43% 두꺼워 본문이 과도하게
        // 굵게 렌더됨. 한컴 돋움 획 두께(페이지 밀도 0.265)에 근접한
        // 'Noto Sans KR ExtraLight'(rsvg 페이지 밀도 0.277)를 무거운 Noto 직전에 삽입 —
        // 시스템 고딕 렌더는 무영향, Noto 폴백만 가볍게 교체.
        return "'Malgun Gothic','맑은 고딕','Apple SD Gothic Neo','HCR Dotum','함초롬돋움','Noto Sans KR ExtraLight','Noto Sans KR','Pretendard','HCR Batang','함초롬바탕','HCR Batang Ext','함초롬바탕 확장','HCR Batang ExtB','함초롬바탕 확장B','RHWP Old Hangul',sans-serif";
    }
    // 고정폭 키워드
    let lower = font_family.to_ascii_lowercase();
    if (font_family.contains("KoPub돋움체") || lower.contains("kopub dotum"))
        && (font_family.contains("Light") || lower.contains("light"))
    {
        return "'Noto Sans KR ExtraLight','Malgun Gothic','맑은 고딕','Apple SD Gothic Neo','HCR Dotum','함초롬돋움','Noto Sans KR','Pretendard','HCR Batang','함초롬바탕','HCR Batang Ext','함초롬바탕 확장','HCR Batang ExtB','함초롬바탕 확장B','RHWP Old Hangul',sans-serif";
    }
    // BatangChe is fixed-width serif. Its stored glyph advances are replayed
    // independently of the paint fallback; choosing a sans coding face here
    // discards the source's serif appearance without preserving layout better.
    if font_family.contains("바탕체")
        || lower.contains("batangche")
        || lower.contains("kopub batang")
    {
        return "'Haansoft Batang','Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR','HCR Batang','함초롬바탕','HCR Batang Ext','함초롬바탕 확장','HCR Batang ExtB','함초롬바탕 확장B','RHWP Old Hangul',serif";
    }
    if font_family.contains("굴림체")
        || lower.contains("gulimche")
        || lower.contains("coding")
        || lower.contains("courier")
        || lower.contains("mono")
    {
        // Monospace: Windows → 오픈소스 → generic
        return "'GulimChe','굴림체','D2Coding','Noto Sans Mono',monospace";
    }
    // 세리프 키워드 (한글)
    if font_family.contains("바탕") || font_family.contains("명조") || font_family.contains("궁서")
    {
        // Serif: Windows → macOS(Bold 보유 우선) → macOS 기본 → Android → 오픈소스 → 한컴 → 리눅스 시스템 → generic
        // Nanum Myeongjo 는 macOS 10.9+ 기본 설치이며 Bold variant 보유.
        // AppleMyungjo 보다 앞에 두어야 macOS Chrome 에서 CJK 글리프 bold 매칭 성공.
        // 'RHWP Old Hangul' (Task #528): @font-face unicode-range 가 옛한글
        // 영역 (U+1100-11FF, U+A960-A97F, U+D7B0-D7FF) 만 매칭하므로 일반 한글에 영향 없음.
        return "'Haansoft Batang','Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR','HCR Batang','함초롬바탕','HCR Batang Ext','함초롬바탕 확장','HCR Batang ExtB','함초롬바탕 확장B','RHWP Old Hangul',serif";
    }
    // 세리프 키워드 (영문) — "serif" 포함하되 "sans" 부분 문자열을 가진 폰트명 전체 제외
    if lower.contains("times")
        || lower.contains("hymjre")
        || lower.contains("palatino")
        || lower.contains("georgia")
        || lower.contains("batang")
        || lower.contains("gungsuh")
        || (lower.contains("serif") && !lower.contains("sans"))
    {
        return "'Haansoft Batang','Batang','바탕','Nanum Myeongjo','AppleMyungjo','Noto Serif KR','Noto Serif CJK KR','HCR Batang','함초롬바탕','HCR Batang Ext','함초롬바탕 확장','HCR Batang ExtB','함초롬바탕 확장B','RHWP Old Hangul',serif";
    }
    // 일반 고딕: 플랫폼 face → 같은 계열 함초롬 → 범용 Noto → 확장 대역.
    // 'RHWP Old Hangul' (Task #528): unicode-range 옛한글 자모 영역 한정
    // 'Noto Sans KR ExtraLight' (Task #1224): 무거운 Noto CJK Regular 폴백 직전에 삽입해
    // 한컴 돋움 획 두께에 근접시킴. 시스템 고딕 우선 → 부재 시에만 ExtraLight 매칭.
    "'Malgun Gothic','맑은 고딕','Apple SD Gothic Neo','HCR Dotum','함초롬돋움','Noto Sans KR ExtraLight','Noto Sans KR','Pretendard','HCR Batang','함초롬바탕','HCR Batang Ext','함초롬바탕 확장','HCR Batang ExtB','함초롬바탕 확장B','RHWP Old Hangul',sans-serif"
}

/// `generic_fallback` 에 굵기 힌트를 얹는다. bold 는 ExtraLight 를 제거한다 (#3772).
pub fn generic_fallback_for_weight(font_family: &str, bold: bool) -> String {
    let chain = generic_fallback(font_family);
    if bold {
        drop_noto_sans_kr_extralight(chain)
    } else {
        chain.to_string()
    }
}
