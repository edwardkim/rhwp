//! 고정형: composer_without_stored_lines의 공개 입력·편집·저장 계약.
//! 주석 갱신: 2026-11-04T00:23:31+09:00; 직전 파일 변경 커밋: 2026-11-03T21:45:03+09:00 39b4f81f6aab4cc08c6145ed6911db2a8fe5125e.
//! 측정: 2026-11-03, 96 DPI·색상 경계 보정 전 2px raw 실루엣. 시각 측정 비해당: 합성 Paragraph의 조합 결과 문자열·원본 글자 모양 ID를 검사한다. 페이지 배치·좌표를 고정하지 않는다.
//! 정식 증거: mydocs/pr/assets/semanticist21-20261103/validation.md 및 전쪽 TSV·확대 PNG.
use rhwp::model::paragraph::{CharShapeRef, Paragraph};
use rhwp::renderer::composer::compose_paragraph;

/// LineSeg 없는 텍스트 문단
#[test]
fn test_compose_no_line_segs() {
    let para = Paragraph {
        text: "텍스트만 있음".to_string(),
        char_shapes: vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 7,
        }],
        ..Default::default()
    };

    let composed = compose_paragraph(&para);
    assert_eq!(composed.lines.len(), 1);
    assert_eq!(composed.lines[0].runs[0].text, "텍스트만 있음");
    assert_eq!(composed.lines[0].runs[0].char_style_id, 7);
}
