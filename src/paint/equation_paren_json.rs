//! 조판에서 발행한 구분자 윤곽을 CanvasKit의 기존 FontGlyph 경로로 전달한다.
//! 글줄 점유·본체·구분자의 상대 원점을 그대로 두며 JS에서 윤곽을 재계산하지 않는다.
use crate::renderer::equation::layout::{LayoutBox, LayoutKind};
use crate::renderer::equation::GlyphOutlineCommand;

fn delimiter(
    layout: &LayoutBox,
    text: &str,
    width: f64,
    x: f64,
    outline: &Option<Vec<GlyphOutlineCommand>>,
    right_side: bool,
) -> LayoutBox {
    let kind = if let Some(outline) = outline {
        LayoutKind::FontGlyph {
            text: text.to_owned(),
            font_family: "Latin Modern Math, STIX Two Math, serif",
            outline: Some(outline.clone()),
            paint: None,
        }
    } else {
        // 혼합 구분자의 미보정 쪽은 기존 Paren 계약으로 그린다.
        let LayoutKind::Paren {
            delimiter_height,
            glyph_delimiters,
            ..
        } = &layout.kind
        else {
            unreachable!()
        };
        LayoutKind::Paren {
            left: if right_side {
                String::new()
            } else {
                text.to_owned()
            },
            right: if right_side {
                text.to_owned()
            } else {
                String::new()
            },
            left_width: if right_side { 0.0 } else { width },
            right_width: if right_side { width } else { 0.0 },
            delimiter_height: *delimiter_height,
            glyph_delimiters: *glyph_delimiters,
            left_outline: None,
            right_outline: None,
            body: Box::new(LayoutBox {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                baseline: 0.0,
                font_size: layout.font_size,
                kind: LayoutKind::Empty,
            }),
        }
    };
    LayoutBox {
        x,
        y: 0.0,
        width,
        height: layout.height,
        baseline: layout.baseline,
        font_size: layout.font_size,
        kind,
    }
}

pub(super) fn write_kind(buf: &mut String, layout: &LayoutBox) -> bool {
    let LayoutKind::Paren {
        left,
        right,
        left_width,
        right_width,
        left_outline,
        right_outline,
        body,
        ..
    } = &layout.kind
    else {
        return false;
    };
    if left_outline.is_none() && right_outline.is_none() {
        return false;
    }
    buf.push_str("{\"type\":\"row\",\"children\":[");
    if !left.is_empty() {
        super::write_equation_layout_box(
            buf,
            &delimiter(layout, left, *left_width, 0.0, left_outline, false),
        );
        buf.push(',');
    }
    super::write_equation_layout_box(buf, body);
    if !right.is_empty() {
        buf.push(',');
        super::write_equation_layout_box(
            buf,
            &delimiter(
                layout,
                right,
                *right_width,
                layout.width - right_width,
                right_outline,
                true,
            ),
        );
    }
    buf.push_str("]}");
    true
}
