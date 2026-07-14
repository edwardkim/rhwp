use crate::model::control::{Control, Equation};
use crate::model::document::{Document, Section};
use crate::model::page::PageDef;
use crate::model::paragraph::{ColumnBreakType, Paragraph};
use crate::model::shape::{
    CommonObjAttr, HorzAlign, HorzRelTo, RectangleShape, ShapeObject, TextWrap, VertAlign,
    VertRelTo,
};
use crate::model::table::{Cell, Table};
use crate::parser::HmlImportMetadata;

use super::error::{unsupported_ir, HmlExportError};
use super::xml::XmlWriter;

pub(crate) fn write_body(
    writer: &mut XmlWriter,
    document: &Document,
    metadata: &HmlImportMetadata,
) -> Result<(), HmlExportError> {
    writer.open("BODY", &[]);
    super::fragments::write_at_anchor(writer, metadata, "BODY", 0);
    for (section_index, section) in document.sections.iter().enumerate() {
        write_section(writer, section, section_index)?;
        super::fragments::write_at_anchor(writer, metadata, "BODY", section_index + 1);
    }
    writer.close("BODY");
    Ok(())
}

fn write_section(
    writer: &mut XmlWriter,
    section: &Section,
    section_index: usize,
) -> Result<(), HmlExportError> {
    writer.open("SECTION", &[("Id", section_index.to_string())]);
    for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
        let page_def = (paragraph_index == 0).then_some(&section.section_def.page_def);
        let path = format!("/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]");
        write_paragraph(writer, paragraph, page_def, &path)?;
    }
    writer.close("SECTION");
    Ok(())
}

fn write_paragraph(
    writer: &mut XmlWriter,
    paragraph: &Paragraph,
    page_def: Option<&PageDef>,
    path: &str,
) -> Result<(), HmlExportError> {
    writer.open(
        "P",
        &[
            ("ParaShape", paragraph.para_shape_id.to_string()),
            ("Style", paragraph.style_id.to_string()),
            (
                "PageBreak",
                (paragraph.column_type == ColumnBreakType::Page).to_string(),
            ),
            (
                "ColumnBreak",
                (paragraph.column_type == ColumnBreakType::Column).to_string(),
            ),
        ],
    );
    if let Some(page_def) = page_def {
        write_section_definition(writer, paragraph, page_def);
    }
    write_paragraph_content(writer, paragraph, path)?;
    writer.close("P");
    Ok(())
}

fn write_section_definition(writer: &mut XmlWriter, paragraph: &Paragraph, page: &PageDef) {
    writer.open(
        "TEXT",
        &[("CharShape", char_shape_at(paragraph, 0).to_string())],
    );
    writer.open("SECDEF", &[]);
    writer.open(
        "PAGEDEF",
        &[
            ("Width", page.width.to_string()),
            ("Height", page.height.to_string()),
            ("Landscape", u8::from(page.landscape).to_string()),
        ],
    );
    writer.empty(
        "PAGEMARGIN",
        &[
            ("Left", page.margin_left.to_string()),
            ("Right", page.margin_right.to_string()),
            ("Top", page.margin_top.to_string()),
            ("Bottom", page.margin_bottom.to_string()),
            ("Header", page.margin_header.to_string()),
            ("Footer", page.margin_footer.to_string()),
            ("Gutter", page.margin_gutter.to_string()),
        ],
    );
    writer.close("PAGEDEF");
    writer.close("SECDEF");
    writer.close("TEXT");
}

fn write_paragraph_content(
    writer: &mut XmlWriter,
    paragraph: &Paragraph,
    path: &str,
) -> Result<(), HmlExportError> {
    let characters = paragraph.text.chars().collect::<Vec<_>>();
    let offsets = paragraph_offsets(paragraph, &characters, path)?;
    let mut expected_offset = 0u32;
    let mut control_index = 0usize;
    let mut text_start = 0usize;

    for (index, actual_offset) in offsets.iter().copied().enumerate() {
        if actual_offset > expected_offset {
            write_text_slice(writer, paragraph, &characters, &offsets, text_start, index);
            insert_controls_until(
                writer,
                paragraph,
                path,
                &mut control_index,
                &mut expected_offset,
                actual_offset,
            )?;
            text_start = index;
        }
        ensure_offset(path, actual_offset, expected_offset)?;
        expected_offset += characters[index].len_utf16() as u32;
    }
    write_text_slice(
        writer,
        paragraph,
        &characters,
        &offsets,
        text_start,
        characters.len(),
    );
    write_remaining_controls(writer, paragraph, path, control_index, expected_offset)?;
    if characters.is_empty() && paragraph.controls.is_empty() {
        write_empty_text(writer, paragraph);
    }
    Ok(())
}

fn paragraph_offsets(
    paragraph: &Paragraph,
    characters: &[char],
    path: &str,
) -> Result<Vec<u32>, HmlExportError> {
    if paragraph.char_offsets.len() == characters.len() {
        return Ok(paragraph.char_offsets.clone());
    }
    Err(unsupported_ir(
        path,
        "문단 문자 위치 정보가 편집된 텍스트와 일치하지 않습니다",
    ))
}

fn insert_controls_until(
    writer: &mut XmlWriter,
    paragraph: &Paragraph,
    path: &str,
    control_index: &mut usize,
    expected_offset: &mut u32,
    target_offset: u32,
) -> Result<(), HmlExportError> {
    while *control_index < paragraph.controls.len() && *expected_offset + 8 <= target_offset {
        write_control_run(
            writer,
            paragraph,
            &paragraph.controls[*control_index],
            *expected_offset,
            &format!("{path}/CONTROL[{control_index}]"),
        )?;
        *control_index += 1;
        *expected_offset += 8;
    }
    Ok(())
}

fn write_remaining_controls(
    writer: &mut XmlWriter,
    paragraph: &Paragraph,
    path: &str,
    mut control_index: usize,
    mut offset: u32,
) -> Result<(), HmlExportError> {
    while control_index < paragraph.controls.len() {
        write_control_run(
            writer,
            paragraph,
            &paragraph.controls[control_index],
            offset,
            &format!("{path}/CONTROL[{control_index}]"),
        )?;
        control_index += 1;
        offset += 8;
    }
    Ok(())
}

fn write_text_slice(
    writer: &mut XmlWriter,
    paragraph: &Paragraph,
    characters: &[char],
    offsets: &[u32],
    start: usize,
    end: usize,
) {
    let mut run_start = start;
    while run_start < end {
        let shape_id = char_shape_at(paragraph, offsets[run_start]);
        let run_end = (run_start + 1..end)
            .find(|index| char_shape_at(paragraph, offsets[*index]) != shape_id)
            .unwrap_or(end);
        let text = characters[run_start..run_end].iter().collect::<String>();
        writer.open("TEXT", &[("CharShape", shape_id.to_string())]);
        writer.open("CHAR", &[]);
        writer.text(&text);
        writer.close("CHAR");
        writer.close("TEXT");
        run_start = run_end;
    }
}

fn write_empty_text(writer: &mut XmlWriter, paragraph: &Paragraph) {
    writer.open(
        "TEXT",
        &[("CharShape", char_shape_at(paragraph, 0).to_string())],
    );
    writer.empty("CHAR", &[]);
    writer.close("TEXT");
}

fn write_control_run(
    writer: &mut XmlWriter,
    paragraph: &Paragraph,
    control: &Control,
    offset: u32,
    path: &str,
) -> Result<(), HmlExportError> {
    writer.open(
        "TEXT",
        &[("CharShape", char_shape_at(paragraph, offset).to_string())],
    );
    let result = match control {
        Control::Equation(equation) => write_equation(writer, equation),
        Control::Table(table) => write_table(writer, table, path),
        Control::Shape(shape) => write_shape(writer, shape, path),
        _ => Err(unsupported_ir(
            path,
            "HML reader가 매핑하지 않는 컨트롤입니다",
        )),
    };
    writer.close("TEXT");
    result
}

fn write_equation(writer: &mut XmlWriter, equation: &Equation) -> Result<(), HmlExportError> {
    let mut attributes = vec![
        ("BaseLine", equation.baseline.to_string()),
        ("BaseUnit", equation.font_size.to_string()),
        ("TextColor", equation.color.to_string()),
        ("Version", equation.version_info.clone()),
    ];
    if !equation.font_name.is_empty() {
        attributes.push(("Font", equation.font_name.clone()));
    }
    writer.open("EQUATION", &attributes);
    writer.open("SCRIPT", &[]);
    writer.text(&equation.script);
    writer.close("SCRIPT");
    writer.close("EQUATION");
    Ok(())
}

fn write_shape(
    writer: &mut XmlWriter,
    shape: &ShapeObject,
    path: &str,
) -> Result<(), HmlExportError> {
    let ShapeObject::Rectangle(rectangle) = shape else {
        return Err(unsupported_ir(
            path,
            "HML reader가 매핑하지 않는 도형 종류입니다",
        ));
    };
    write_rectangle(writer, rectangle, path)
}

fn write_rectangle(
    writer: &mut XmlWriter,
    rectangle: &RectangleShape,
    path: &str,
) -> Result<(), HmlExportError> {
    writer.open("RECTANGLE", &rectangle_coordinates(rectangle));
    write_shape_object(writer, &rectangle.common, "Figure");
    writer.open("DRAWINGOBJECT", &[]);
    write_shape_component(writer, rectangle);
    write_line_shape(writer, rectangle);
    write_rectangle_fill(writer, rectangle);
    if let Some(text_box) = &rectangle.drawing.text_box {
        writer.open("DRAWTEXT", &[]);
        writer.empty(
            "TEXTMARGIN",
            &padding_attributes(
                text_box.margin_left,
                text_box.margin_right,
                text_box.margin_top,
                text_box.margin_bottom,
            ),
        );
        writer.open("PARALIST", &[]);
        for (index, paragraph) in text_box.paragraphs.iter().enumerate() {
            write_paragraph(
                writer,
                paragraph,
                None,
                &format!("{path}/DRAWTEXT/P[{index}]"),
            )?;
        }
        writer.close("PARALIST");
        writer.close("DRAWTEXT");
    }
    writer.close("DRAWINGOBJECT");
    writer.close("RECTANGLE");
    Ok(())
}

fn rectangle_coordinates(rectangle: &RectangleShape) -> Vec<(&'static str, String)> {
    ["X0", "X1", "X2", "X3"]
        .into_iter()
        .zip(rectangle.x_coords)
        .chain(["Y0", "Y1", "Y2", "Y3"].into_iter().zip(rectangle.y_coords))
        .map(|(name, value)| (name, value.to_string()))
        .collect()
}

fn write_shape_object(writer: &mut XmlWriter, common: &CommonObjAttr, numbering_type: &str) {
    writer.open(
        "SHAPEOBJECT",
        &[
            ("NumberingType", numbering_type.into()),
            ("TextWrap", text_wrap_name(common.text_wrap).into()),
        ],
    );
    writer.empty(
        "SIZE",
        &[
            ("Width", common.width.to_string()),
            ("Height", common.height.to_string()),
        ],
    );
    writer.empty("POSITION", &position_attributes(common));
    writer.close("SHAPEOBJECT");
}

fn write_shape_component(writer: &mut XmlWriter, rectangle: &RectangleShape) {
    let shape = &rectangle.drawing.shape_attr;
    writer.empty(
        "SHAPECOMPONENT",
        &[
            ("XPos", shape.offset_x.to_string()),
            ("YPos", shape.offset_y.to_string()),
            ("OriWidth", shape.original_width.to_string()),
            ("OriHeight", shape.original_height.to_string()),
            ("CurWidth", shape.current_width.to_string()),
            ("CurHeight", shape.current_height.to_string()),
        ],
    );
}

fn write_line_shape(writer: &mut XmlWriter, rectangle: &RectangleShape) {
    writer.empty(
        "LINESHAPE",
        &[
            ("Width", rectangle.drawing.border_line.width.to_string()),
            ("Style", "Solid".into()),
            ("EndCap", "Flat".into()),
            ("Alpha", "0".into()),
        ],
    );
}

fn write_rectangle_fill(writer: &mut XmlWriter, rectangle: &RectangleShape) {
    if let Some(solid) = rectangle.drawing.fill.solid {
        writer.open("FILLBRUSH", &[]);
        writer.empty(
            "WINDOWBRUSH",
            &[
                ("FaceColor", solid.background_color.to_string()),
                ("HatchColor", solid.pattern_color.to_string()),
                ("Alpha", rectangle.drawing.fill.alpha.to_string()),
            ],
        );
        writer.close("FILLBRUSH");
    }
}

fn write_table(writer: &mut XmlWriter, table: &Table, path: &str) -> Result<(), HmlExportError> {
    writer.open(
        "TABLE",
        &[
            ("RowCount", table.row_count.to_string()),
            ("ColCount", table.col_count.to_string()),
            ("CellSpacing", table.cell_spacing.to_string()),
            ("BorderFill", table.border_fill_id.to_string()),
        ],
    );
    write_shape_object(writer, &table.common, "Table");
    writer.empty(
        "INSIDEMARGIN",
        &padding_attributes(
            table.padding.left,
            table.padding.right,
            table.padding.top,
            table.padding.bottom,
        ),
    );
    for row in 0..table.row_count {
        writer.open("ROW", &[]);
        for (cell_index, cell) in table
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.row == row)
        {
            write_cell(writer, cell, &format!("{path}/CELL[{cell_index}]"))?;
        }
        writer.close("ROW");
    }
    writer.close("TABLE");
    Ok(())
}

fn write_cell(writer: &mut XmlWriter, cell: &Cell, path: &str) -> Result<(), HmlExportError> {
    writer.open(
        "CELL",
        &[
            ("ColAddr", cell.col.to_string()),
            ("RowAddr", cell.row.to_string()),
            ("ColSpan", cell.col_span.to_string()),
            ("RowSpan", cell.row_span.to_string()),
            ("Width", cell.width.to_string()),
            ("Height", cell.height.to_string()),
            ("BorderFill", cell.border_fill_id.to_string()),
        ],
    );
    writer.empty(
        "CELLMARGIN",
        &padding_attributes(
            cell.padding.left,
            cell.padding.right,
            cell.padding.top,
            cell.padding.bottom,
        ),
    );
    writer.open("PARALIST", &[]);
    for (index, paragraph) in cell.paragraphs.iter().enumerate() {
        write_paragraph(writer, paragraph, None, &format!("{path}/P[{index}]"))?;
    }
    writer.close("PARALIST");
    writer.close("CELL");
    Ok(())
}

fn position_attributes(common: &CommonObjAttr) -> Vec<(&'static str, String)> {
    vec![
        ("HorzOffset", (common.horizontal_offset as i32).to_string()),
        ("VertOffset", (common.vertical_offset as i32).to_string()),
        ("TreatAsChar", common.treat_as_char.to_string()),
        ("FlowWithText", common.flow_with_text.to_string()),
        ("AllowOverlap", common.allow_overlap.to_string()),
        ("HorzRelTo", horz_rel_name(common.horz_rel_to).into()),
        ("VertRelTo", vert_rel_name(common.vert_rel_to).into()),
        ("HorzAlign", horz_align_name(common.horz_align).into()),
        ("VertAlign", vert_align_name(common.vert_align).into()),
    ]
}

fn padding_attributes(left: i16, right: i16, top: i16, bottom: i16) -> Vec<(&'static str, String)> {
    vec![
        ("Left", left.to_string()),
        ("Right", right.to_string()),
        ("Top", top.to_string()),
        ("Bottom", bottom.to_string()),
    ]
}

fn char_shape_at(paragraph: &Paragraph, offset: u32) -> u32 {
    paragraph
        .char_shapes
        .iter()
        .rev()
        .find(|reference| reference.start_pos <= offset)
        .map(|reference| reference.char_shape_id)
        .unwrap_or(0)
}

fn ensure_offset(path: &str, actual: u32, expected: u32) -> Result<(), HmlExportError> {
    if actual == expected {
        return Ok(());
    }
    Err(unsupported_ir(
        path,
        &format!("문자 위치 {actual}을 HML 제어문자 위치 {expected}로 역변환할 수 없습니다"),
    ))
}

fn vert_rel_name(value: VertRelTo) -> &'static str {
    match value {
        VertRelTo::Page => "Page",
        VertRelTo::Para => "Para",
        VertRelTo::Paper => "Paper",
    }
}

fn vert_align_name(value: VertAlign) -> &'static str {
    match value {
        VertAlign::Center => "Center",
        VertAlign::Bottom => "Bottom",
        VertAlign::Inside => "Inside",
        VertAlign::Outside => "Outside",
        VertAlign::Top => "Top",
    }
}

fn horz_rel_name(value: HorzRelTo) -> &'static str {
    match value {
        HorzRelTo::Page => "Page",
        HorzRelTo::Column => "Column",
        HorzRelTo::Para => "Para",
        HorzRelTo::Paper => "Paper",
    }
}

fn horz_align_name(value: HorzAlign) -> &'static str {
    match value {
        HorzAlign::Center => "Center",
        HorzAlign::Right => "Right",
        HorzAlign::Inside => "Inside",
        HorzAlign::Outside => "Outside",
        HorzAlign::Left => "Left",
    }
}

fn text_wrap_name(value: TextWrap) -> &'static str {
    match value {
        TextWrap::Tight => "Tight",
        TextWrap::Through => "Through",
        TextWrap::TopAndBottom => "TopAndBottom",
        TextWrap::BehindText => "BehindText",
        TextWrap::InFrontOfText => "InFrontOfText",
        TextWrap::Square => "Square",
    }
}
