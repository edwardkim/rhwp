//! 기존 source 단위 검사 중 공개 API 계약을 integration 경로로 이관한다.
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::composer::compose_paragraph;

/// 빈 문단
#[test]
fn test_compose_empty_paragraph() {
    let para = Paragraph::default();
    let composed = compose_paragraph(&para);
    assert!(composed.lines.is_empty());
    assert!(composed.inline_controls.is_empty());
}

#[test]
fn hft_ascii_advances_match_independent_print() {
    // 원본115쪽 Print 8/9쪽 88개와 분할 Print 148쪽 31개의
    // Type3 glyph advance는 모두 0.244em이다. 1024 단위 표의 양자화만 허용한다.
    let found = rhwp::renderer::font_metrics_data::find_metric("한양신명조", false, false).unwrap();
    // i도 같은 원본/분할 Print에서 CharProc SHA와277/1000 자연 폭이 같다.
    // 예전281/1024는 저장4줄을3줄로 재조판하게 한 과소 측정이었다.
    for (character, independent_em) in [('.', 0.244), ('i', 0.277)] {
        let advance =
            f64::from(found.metric.get_width(character).unwrap()) / f64::from(found.metric.em_size);
        assert!(
            (advance - independent_em).abs() <= 0.5 / 1024.0,
            "독립 Print의 {character} 자연 폭: 실제{advance}, 독립{independent_em}"
        );
    }
}
