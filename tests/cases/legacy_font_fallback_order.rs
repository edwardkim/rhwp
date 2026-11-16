//! 검사 종류: 관계형(선택 순서·굵기 계약, 절대 px 고정 없음). 돋움·굴림 계열 대체 글꼴 목록의 순서와 굵기에 따른 선택 계약,
//! 고딕 대체 목록이 리눅스 Noto CJK 한글 family에 닿는 순서 계약.
//! 주석 갱신: 2026-11-17T01:57:01+09:00; 직전 파일 커밋: 2026-10-19T01:41:15+09:00 (`a7483726829151f430ade97ca0e7ea0417c27886`).
//! 입력·기준: 파일 내 글꼴 이름 표본과 대체 목록 API.
//! 선택 문자열을 검사하며 화면 시각 일치율은 측정하지 않는다.
use rhwp::renderer::{generic_fallback, generic_fallback_for_weight};

#[test]
fn legacy_dotum_gulim_keeps_light_fallback_before_modern_sans() {
    // 기존 돋움·굴림의 획 밀도와 일반 플랫폼 고딕의 문자 보완은 별도 계약이다.
    // 전체 문자열을 구현에서 복사하지 않고 필요한 face의 선후 관계를 보호한다.
    for source in ["돋움", "Dotum", "굴림", "Gulim"] {
        let chain = generic_fallback(source);
        let position = |face: &str| chain.find(face).expect("필수 fallback face");
        assert!(chain.starts_with("'Noto Sans KR ExtraLight',"));
        assert!(position("'Noto Sans KR ExtraLight'") < position("'Noto Sans KR'"));
        assert!(position("'Noto Sans KR'") < position("'Malgun Gothic'"));
        assert!(position("'HCR Dotum'") < position("'Pretendard'"));
        assert!(chain.ends_with(",sans-serif"));

        let bold = generic_fallback_for_weight(source, true);
        assert!(
            !bold.contains("ExtraLight"),
            "굵은 원문에 가는 획을 공급함: {source}"
        );
        assert!(bold.starts_with("'Noto Sans KR',"));
        assert!(bold.ends_with(",sans-serif"));
    }
}

#[test]
fn sans_fallback_reaches_linux_noto_cjk_family_like_serif() {
    // 리눅스 fonts-noto-cjk는 Noto 한글을 'Noto Sans CJK KR'·'Noto Serif CJK KR'
    // family로만 등록한다. 웹폰트를 받지 못한 Native PNG는 이 목록에서 한글 face를
    // 찾으므로, 고딕도 명조처럼 'Noto … KR' 바로 뒤에서 CJK family에 닿아야 한다(#9104).
    for source in [
        "",
        "함초롬돋움",
        "맑은 고딕",
        "HY견고딕",
        "신명 태고딕",
        "돋움",
        "KoPub돋움체 Light",
    ] {
        for bold in [false, true] {
            let chain = generic_fallback_for_weight(source, bold);
            assert!(
                chain.contains("'Noto Sans KR','Noto Sans CJK KR',"),
                "고딕 대체 목록에 설치 Noto CJK 한글 family가 없음: {source:?} bold={bold}: {chain}"
            );
            assert!(chain.ends_with(",sans-serif"));
        }
    }
    for source in ["함초롬바탕", "HY신명조", "바탕체"] {
        assert!(generic_fallback(source).contains("'Noto Serif KR','Noto Serif CJK KR',"));
    }
}
