//! `Pretendard-Bold` 처럼 PostScript 이름으로 저장된 face 도 family 메트릭을 찾는다.
//! 종전에는 별칭 규칙에 굵기 접미사가 없어 메트릭 없이 반각 추정 폭으로 조판했다.

use rhwp::renderer::font_metrics_data::find_metric;

#[test]
fn postscript_weight_suffix_resolves_family_metric() {
    let regular = find_metric("Pretendard-Regular", false, false).unwrap();
    assert_eq!(regular.metric.name, "Pretendard");
    assert!(!regular.metric.bold);
    let medium = find_metric("Pretendard-Medium", false, false).unwrap();
    assert!(std::ptr::eq(medium.metric, regular.metric));
    let bold = find_metric("Pretendard-Bold", false, false).unwrap();
    assert_eq!(bold.metric.name, "Pretendard");
    assert!(bold.metric.bold && !bold.bold_fallback);
    // 접미사를 뗀 family 도 별칭 규칙을 그대로 따른다.
    let source_han = find_metric("SourceHanSansK-Medium", false, false).unwrap();
    assert_eq!(source_han.metric.name, "Pretendard");
    assert!(find_metric("NoSuchFont-Bold", false, false).is_none());
    assert!(find_metric("Pretendard-Light", false, false).is_none());
}
