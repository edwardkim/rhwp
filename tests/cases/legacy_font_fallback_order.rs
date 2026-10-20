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
