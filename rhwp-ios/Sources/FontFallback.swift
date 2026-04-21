// 폰트 폴백 매핑 — HWP 폰트명 → Apple 플랫폼 시스템 폰트
// 참조: mydocs/tech/font_fallback_strategy.md

import CoreText

/// HWP 폰트명을 Apple 플랫폼에서 사용 가능한 폰트명으로 변환한다.
func resolveIOSFont(hwpFontFamily: String, bold: Bool, italic: Bool) -> CTFont {
    let iosName = mapHWPFontToIOS(hwpFontFamily)
    let size: CGFloat = 1.0 // 크기는 호출 측에서 설정
    var traits = CTFontSymbolicTraits()
    if bold { traits.insert(.boldTrait) }
    if italic { traits.insert(.italicTrait) }

    if let font = CTFontCreateWithName(iosName as CFString, size, nil) as CTFont? {
        if let withTraits = CTFontCreateCopyWithSymbolicTraits(font, size, nil, traits, [.boldTrait, .italicTrait]) {
            return withTraits
        }
        return font
    }
    return CTFontCreateWithName("AppleSDGothicNeo-Regular" as CFString, size, nil)
}

/// HWP 폰트명 → Apple 플랫폼 폰트명 매핑
func mapHWPFontToIOS(_ hwpFont: String) -> String {
    // 정확한 매핑
    switch hwpFont {
    // Serif 계열 (바탕/명조)
    case "한컴바탕", "함초롬바탕", "HBatang", "HBatangB":
        return "AppleMyungjo"
    case "바탕", "바탕체", "Batang", "BatangChe":
        return "AppleMyungjo"
    case "새바탕", "새바탕체":
        return "AppleMyungjo"
    case "궁서", "궁서체", "Gungsuh", "GungsuhChe":
        return "AppleMyungjo"
    case "HY신명조", "HYSinMyeongJo", "HYSinMyeongJoMedium":
        return "AppleMyungjo"
    case "HY견명조", "HYGyeonMyeongJo":
        return "AppleMyungjo"
    case "휴먼명조", "HumanMyeongjo":
        return "AppleMyungjo"
    case "나눔명조", "NanumMyeongjo":
        return "NanumMyeongjo"

    // Sans-serif 계열 (돋움/고딕)
    case "한컴돋움", "함초롬돋움", "HDotum", "HDotumB":
        return "AppleSDGothicNeo-Regular"
    case "돋움", "돋움체", "Dotum", "DotumChe":
        return "AppleSDGothicNeo-Regular"
    case "굴림", "굴림체", "Gulim", "GulimChe":
        return "AppleSDGothicNeo-Regular"
    case "새돋움", "새돋움체", "새굴림":
        return "AppleSDGothicNeo-Regular"
    case "맑은 고딕", "Malgun Gothic", "MalgunGothic":
        return "AppleSDGothicNeo-Regular"
    case "HY중고딕", "HYJungGoThicMedium":
        return "AppleSDGothicNeo-Medium"
    case "HY견고딕", "HYGyeonGoThic":
        return "AppleSDGothicNeo-Bold"
    case "HY헤드라인M", "HYHeadLineMedium":
        return "AppleSDGothicNeo-Bold"
    case "HY그래픽", "HYGraphicMedium":
        return "AppleSDGothicNeo-Medium"
    case "나눔고딕", "NanumGothic":
        return "NanumGothic"

    // 영문 폰트
    case "Arial":
        return "ArialMT"
    case "Times New Roman":
        return "TimesNewRomanPSMT"
    case "Courier New":
        return "CourierNewPSMT"
    case "Calibri":
        return "Helvetica Neue"
    case "Tahoma":
        return "Helvetica Neue"
    case "Verdana":
        return "Verdana"

    default:
        // 알 수 없는 폰트는 시스템 기본 사용
        return hwpFont
    }
}
