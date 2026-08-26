use rhwp::model::shape::{
    CommonObjAttr, HorzRelTo, ShapeComponentAttr, ShapeObject, TextWrap, VertRelTo,
};

use crate::{hu_to_mm, hu_to_mm_i};

pub(super) fn vert_str(value: &VertRelTo) -> &str {
    match value {
        VertRelTo::Paper => "용지",
        VertRelTo::Page => "쪽",
        VertRelTo::Para => "문단",
    }
}

pub(super) fn horz_str(value: &HorzRelTo) -> &str {
    match value {
        HorzRelTo::Paper => "용지",
        HorzRelTo::Page => "쪽",
        HorzRelTo::Column => "단",
        HorzRelTo::Para => "문단",
    }
}

pub(super) fn wrap_str(value: &TextWrap) -> &str {
    match value {
        TextWrap::Square => "어울림",
        TextWrap::Tight => "빈 공간 채움",
        TextWrap::Through => "통과",
        TextWrap::TopAndBottom => "자리차지",
        TextWrap::BehindText => "글뒤로",
        TextWrap::InFrontOfText => "글앞으로",
    }
}

pub(super) fn dump_common(common: &CommonObjAttr, indent: &str) {
    println!(
        "{}  크기: {:.1}mm × {:.1}mm ({}×{} HU)",
        indent,
        hu_to_mm(common.width),
        hu_to_mm(common.height),
        common.width,
        common.height
    );
    println!(
        "{}  위치: 가로={} 오프셋={:.1}mm({}) 정렬={:?}, 세로={} 오프셋={:.1}mm({}) 정렬={:?}",
        indent,
        horz_str(&common.horz_rel_to),
        hu_to_mm(common.horizontal_offset),
        common.horizontal_offset,
        common.horz_align,
        vert_str(&common.vert_rel_to),
        hu_to_mm(common.vertical_offset),
        common.vertical_offset,
        common.vert_align
    );
    println!(
        "{}  배치: {}, 글자처럼={}, z={}",
        indent,
        wrap_str(&common.text_wrap),
        common.treat_as_char,
        common.z_order
    );
    println!(
        "{}  바깥 여백: left={:.2}mm({}) right={:.2}mm({}) top={:.2}mm({}) bottom={:.2}mm({})",
        indent,
        hu_to_mm_i(common.margin.left as i32),
        common.margin.left,
        hu_to_mm_i(common.margin.right as i32),
        common.margin.right,
        hu_to_mm_i(common.margin.top as i32),
        common.margin.top,
        hu_to_mm_i(common.margin.bottom as i32),
        common.margin.bottom
    );
}

pub(super) fn dump_shape_attr(attr: &ShapeComponentAttr, indent: &str) {
    let effective_width = (attr.current_width as f64 * attr.render_sx) as u32;
    let effective_height = (attr.current_height as f64 * attr.render_sy) as u32;
    println!("{}  요소: orig={}×{}, curr={}×{}, M=[{:.3},{:.3},{:.0}; {:.3},{:.3},{:.0}], offset=({},{}), eff={:.1}mm×{:.1}mm",
        indent, attr.original_width, attr.original_height,
        attr.current_width, attr.current_height,
        attr.render_sx, attr.render_b, attr.render_tx,
        attr.render_c, attr.render_sy, attr.render_ty,
        attr.offset_x, attr.offset_y,
        hu_to_mm(effective_width), hu_to_mm(effective_height));
    if attr.horz_flip || attr.vert_flip || attr.rotation_angle != 0 {
        println!(
            "{}  변환: 뒤집기=({},{}), 회전={}",
            indent, attr.horz_flip, attr.vert_flip, attr.rotation_angle
        );
    }
}

pub(super) fn dump_shape_control(shape: &ShapeObject, prefix: &str) {
    print!("{}", prefix);
    dump_shape(shape, "  ");
}

fn dump_shape(shape: &ShapeObject, indent: &str) {
    match shape {
        ShapeObject::Line(line) => {
            println!(
                "{}[직선] start=({},{}) end=({},{})",
                indent, line.start.x, line.start.y, line.end.x, line.end.y
            );
            println!(
                "{}  선: color={:#010x}, width={}, style={:#06x}",
                indent,
                line.drawing.border_line.color,
                line.drawing.border_line.width,
                line.drawing.border_line.attr
            );
            dump_common(&line.common, indent);
            dump_shape_attr(&line.drawing.shape_attr, indent);
        }
        ShapeObject::Rectangle(rectangle) => {
            println!("{}[사각형] round={}%", indent, rectangle.round_rate);
            println!(
                "{}  선: color={:#010x}, width={}, style={:#06x}",
                indent,
                rectangle.drawing.border_line.color,
                rectangle.drawing.border_line.width,
                rectangle.drawing.border_line.attr
            );
            println!(
                "{}  채우기: {:?}{}",
                indent,
                rectangle.drawing.fill.fill_type,
                rectangle
                    .drawing
                    .fill
                    .image
                    .as_ref()
                    .map(|image| format!(
                        ", image=bin_data_id={}, mode={:?}",
                        image.bin_data_id, image.fill_mode
                    ))
                    .unwrap_or_default()
            );
            dump_common(&rectangle.common, indent);
            dump_shape_attr(&rectangle.drawing.shape_attr, indent);
            dump_text_box(rectangle.drawing.text_box.as_ref(), indent);
        }
        ShapeObject::Ellipse(ellipse) => {
            println!("{}[타원]", indent);
            dump_common(&ellipse.common, indent);
            dump_shape_attr(&ellipse.drawing.shape_attr, indent);
        }
        ShapeObject::Arc(arc) => {
            println!("{}[호]", indent);
            dump_common(&arc.common, indent);
            dump_shape_attr(&arc.drawing.shape_attr, indent);
        }
        ShapeObject::Polygon(polygon) => {
            println!("{}[다각형] points={}", indent, polygon.points.len());
            dump_common(&polygon.common, indent);
            dump_shape_attr(&polygon.drawing.shape_attr, indent);
            if !polygon.points.is_empty() {
                let min_x = polygon.points.iter().map(|point| point.x).min().unwrap();
                let max_x = polygon.points.iter().map(|point| point.x).max().unwrap();
                let min_y = polygon.points.iter().map(|point| point.y).min().unwrap();
                let max_y = polygon.points.iter().map(|point| point.y).max().unwrap();
                println!(
                    "{}  좌표범위: x=[{},{}], y=[{},{}]",
                    indent, min_x, max_x, min_y, max_y
                );
            }
        }
        ShapeObject::Curve(curve) => {
            println!("{}[곡선] points={}", indent, curve.points.len());
            dump_common(&curve.common, indent);
            dump_shape_attr(&curve.drawing.shape_attr, indent);
        }
        ShapeObject::Group(group) => {
            println!("{}[묶음] children={}", indent, group.children.len());
            dump_common(&group.common, indent);
            dump_shape_attr(&group.shape_attr, indent);
            let child_indent = format!("{}  ", indent);
            for (index, child) in group.children.iter().enumerate() {
                print!("{}child[{}] ", child_indent, index);
                dump_shape(child, &child_indent);
            }
        }
        ShapeObject::Picture(picture) => {
            println!(
                "{}[그림] bin_data_id={}",
                indent, picture.image_attr.bin_data_id
            );
            dump_common(&picture.common, indent);
            dump_shape_attr(&picture.shape_attr, indent);
        }
        ShapeObject::Chart(chart) => {
            println!(
                "{}[차트] type={:?} series={} raw_chart_data={}B",
                indent,
                chart.chart_type,
                chart.series.len(),
                chart.raw_chart_data.len()
            );
            dump_common(&chart.common, indent);
            dump_shape_attr(&chart.drawing.shape_attr, indent);
        }
        ShapeObject::Ole(ole) => {
            println!(
                "{}[OLE] bin_data_id={} extent={}x{} flags=0x{:02X} raw={}B",
                indent,
                ole.bin_data_id,
                ole.extent_x,
                ole.extent_y,
                ole.flags,
                ole.raw_tag_data.len()
            );
            dump_common(&ole.common, indent);
            dump_shape_attr(&ole.drawing.shape_attr, indent);
        }
    }
}

fn dump_text_box(text_box: Option<&rhwp::model::shape::TextBox>, indent: &str) {
    let Some(text_box) = text_box else {
        return;
    };
    println!(
        "{}  글상자: list_attr={:#010x}, margins=({},{},{},{}), max_width={}, paras={}",
        indent,
        text_box.list_attr,
        text_box.margin_left,
        text_box.margin_right,
        text_box.margin_top,
        text_box.margin_bottom,
        text_box.max_width,
        text_box.paragraphs.len()
    );
    for (paragraph_index, paragraph) in text_box.paragraphs.iter().enumerate() {
        let text_preview = if paragraph.text.is_empty() {
            "(빈)".to_string()
        } else if paragraph.text.chars().count() > 60 {
            let end = paragraph
                .text
                .char_indices()
                .nth(60)
                .map(|(index, _)| index)
                .unwrap_or(paragraph.text.len());
            format!("\"{}...\"", &paragraph.text[..end])
        } else {
            format!("\"{}\"", paragraph.text)
        };
        println!(
            "{}    p[{}]: ps_id={}, cc={}, text={}, ls_count={}, ctrls={}",
            indent,
            paragraph_index,
            paragraph.para_shape_id,
            paragraph.char_count,
            text_preview,
            paragraph.line_segs.len(),
            paragraph.controls.len()
        );
        for (line_index, line) in paragraph.line_segs.iter().enumerate() {
            println!(
                "{}      ls[{}]: vpos={}, lh={}, th={}, bl={}, cs={}, sw={}",
                indent,
                line_index,
                line.vertical_pos,
                line.line_height,
                line.text_height,
                line.baseline_distance,
                line.column_start,
                line.segment_width
            );
        }
    }
}

pub(super) fn dump_picture(picture: &rhwp::model::image::Picture, prefix: &str) {
    let attr = &picture.shape_attr;
    println!("{}그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}",
        prefix, picture.image_attr.bin_data_id, picture.common.width, picture.common.height,
        picture.common.width as f64 / 7200.0 * 25.4, picture.common.height as f64 / 7200.0 * 25.4,
        attr.original_width, attr.original_height,
        attr.original_width as f64 / 7200.0 * 25.4, attr.original_height as f64 / 7200.0 * 25.4,
        attr.current_width, attr.current_height,
        attr.current_width as f64 / 7200.0 * 25.4, attr.current_height as f64 / 7200.0 * 25.4,
        picture.common.treat_as_char);
    println!(
        "{}  [placement] wrap={:?} vert={:?}(off={}) horz={:?}(off={}) vert_align={:?}",
        prefix,
        picture.common.text_wrap,
        picture.common.vert_rel_to,
        picture.common.vertical_offset,
        picture.common.horz_rel_to,
        picture.common.horizontal_offset,
        picture.common.vert_align
    );
    println!(
        "{}  [image_attr] effect={:?} brightness={} contrast={} watermark={}{}",
        prefix,
        picture.image_attr.effect,
        picture.image_attr.brightness,
        picture.image_attr.contrast,
        picture.image_attr.watermark_preset().unwrap_or("none"),
        picture
            .image_attr
            .external_path
            .as_ref()
            .map(|path| format!(" external_path=\"{}\"", path))
            .unwrap_or_default()
    );
    println!("{}  border_x={:?} border_y={:?} border_color=#{:06X} border_width={} ({:.2}mm) border_attr={:?}",
        prefix, picture.border_x, picture.border_y, picture.border_color, picture.border_width,
        picture.border_width as f64 / 7200.0 * 25.4, picture.border_attr);
    println!(
        "{}  crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
        prefix,
        picture.crop.left,
        picture.crop.top,
        picture.crop.right,
        picture.crop.bottom,
        picture.crop.left as f64 / 7200.0 * 25.4,
        picture.crop.top as f64 / 7200.0 * 25.4,
        picture.crop.right as f64 / 7200.0 * 25.4,
        picture.crop.bottom as f64 / 7200.0 * 25.4
    );
    if let Some(caption) = &picture.caption {
        let text = caption
            .paragraphs
            .iter()
            .map(|paragraph| paragraph.text.clone())
            .collect::<Vec<_>>()
            .join("|");
        println!(
            "{}  caption: dir={:?} width={} paras={} text={:?}",
            prefix,
            caption.direction,
            caption.width,
            caption.paragraphs.len(),
            text
        );
    }
    let shape_indent = format!("{}  ", prefix);
    dump_shape_attr(attr, &shape_indent);
    dump_common(&picture.common, "  ");
}
