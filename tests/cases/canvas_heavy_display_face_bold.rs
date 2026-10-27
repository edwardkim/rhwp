//! Canvas 위치 지정 글자도 SVG 처럼 `is_visually_bold` 로 굵기를 정한다.
//! CharShape.bold=false 로 저장된 HY헤드라인M·HY견고딕(한양견고딕) 제목이 얇게 그려지던 회귀를 막는다.

use rhwp::renderer::canvas_text_font::CanvasTextFont;
use rhwp::renderer::TextStyle;

fn descriptor(face: &str) -> String {
    let style = TextStyle {
        font_family: face.into(),
        font_size: 20.0,
        ..Default::default()
    };
    CanvasTextFont::for_positioned_text(&style, 0.0)
        .descriptor()
        .to_string()
}

#[test]
fn heavy_display_faces_paint_bold_on_canvas() {
    for face in ["HY헤드라인M", "HY견고딕", "한양견고딕", "HYGothic-Extra"] {
        assert!(
            descriptor(face).starts_with("bold "),
            "{face}: {}",
            descriptor(face)
        );
    }
    assert!(!descriptor("함초롬바탕").starts_with("bold "));
}
