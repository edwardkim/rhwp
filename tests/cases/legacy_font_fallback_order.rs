//! 관계형·고정형: 돋움·굴림 계열 대체 글꼴 목록의 순서와 굵기에 따른 선택 계약.
//! 주석 갱신: 2026-11-01T06:04:22+09:00; 직전 파일 커밋: 2026-10-19T01:41:15+09:00 (`a7483726829151f430ade97ca0e7ea0417c27886`).
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
