//! 원본 WMF 창과 글꼴을 출력 장치의 처리와 분리해 전달한다.
//!
//! SVG의 좌표는 이미 창 원점 기준이다. LOGFONT의 원본 높이·폭과 창을
//! 함께 보존해야 장치 폭을 구할 때 SVG의 글자 높이로 원본을 추측하지 않는다.

use super::{device_context::Window, DeviceContext, Node};
use crate::wmf::parser::Font;

#[derive(Default)]
pub(super) struct SourceContract {
    polygon_seen: bool,
    non_pixel_units_seen: bool,
}

fn window(element: Node, source: &Window) -> Node {
    element.set(
        "data-rhwp-wmf-window",
        format!(
            "{} {} {} {} {} {} {} {}",
            source.origin_x,
            source.origin_y,
            source.x,
            source.y,
            source.scale_x,
            source.scale_y,
            u8::from(source.x_inverted),
            u8::from(source.y_inverted),
        ),
    )
}

impl SourceContract {
    pub(super) fn element(
        &mut self,
        mut element: Node,
        context: &DeviceContext,
        font: &Font,
    ) -> Node {
        self.non_pixel_units_seen |= context.map_mode != crate::wmf::parser::MapMode::MM_TEXT;
        // 이 상태는 그래픽 DC의 Save/Restore와 별개인 source 그리기 순서다.
        // 출력 쪽이 이를 숫자 직렬화 계약으로 해석하며 여기서는 반올림하지 않는다.
        if element.has_name("polygon") {
            self.polygon_seen = true;
        }
        if element.has_name("text") {
            element = window(element, &context.window)
                .set("data-rhwp-wmf-polygon-seen", u8::from(self.polygon_seen))
                .set("data-rhwp-wmf-map-mode", context.map_mode as u16)
                .set(
                    "data-rhwp-wmf-text-align",
                    format!(
                        "{} {} {}",
                        context.as_css_text_align(),
                        context.as_css_text_align_vertical(),
                        u8::from(context.text_align_update_cp),
                    ),
                )
                .set(
                    "data-rhwp-wmf-font",
                    format!(
                        "{} {} {} {} {} {}",
                        font.height,
                        font.width,
                        font.escapement,
                        font.orientation,
                        font.weight,
                        u8::from(font.italic),
                    ),
                );
        }
        element
    }

    pub(super) fn document(self, element: Node, context: &DeviceContext) -> Node {
        window(element, &context.window)
            .set("data-rhwp-wmf-source-contract", "1")
            .set("data-rhwp-wmf-polygon-seen", u8::from(self.polygon_seen))
            .set(
                "data-rhwp-wmf-pixel-grid",
                u8::from(
                    !self.non_pixel_units_seen
                        && context.map_mode == crate::wmf::parser::MapMode::MM_TEXT,
                ),
            )
    }
}
