use crate::model::control::{Control, Equation};
use crate::model::document::{Document, FileHeader, HwpVersion, Section, SectionDef};
use crate::model::paragraph::{CharShapeRef, Paragraph};
use crate::model::shape::{CommonObjAttr, DrawingObjAttr, RectangleShape, ShapeObject, TextBox};
use crate::model::style::{BorderFill, CharShape, Font, LineSpacingType, ParaShape, Style, TabDef};
use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};

use super::error::HmlError;
use super::reader::{HmlControl, HmlEquation, HmlParagraph, HmlRectangle, HmlSource, HmlTable};

pub(crate) fn into_document(mut source: HmlSource) -> Result<Document, HmlError> {
    let mut document = Document {
        header: FileHeader {
            version: HwpVersion {
                major: 5,
                minor: 1,
                build: 0,
                revision: 0,
            },
            ..Default::default()
        },
        ..Default::default()
    };
    document.doc_info.hwpml_version = Some(source.version.clone());
    move_resources(&mut source, &mut document);
    for source_section in source.sections {
        let section_def = SectionDef {
            page_def: source_section
                .page_def
                .unwrap_or_else(crate::model::page::PageDef::a4_default),
            ..Default::default()
        };
        let paragraphs = source_section
            .paragraphs
            .into_iter()
            .map(into_paragraph)
            .collect::<Result<Vec<_>, _>>()?;
        document.sections.push(Section {
            section_def,
            paragraphs,
            raw_stream: None,
        });
    }
    document.doc_properties.section_count = document.sections.len() as u16;
    Ok(document)
}

fn move_resources(source: &mut HmlSource, document: &mut Document) {
    add_default_resources(document);
    if !source.font_faces.is_empty() {
        document.doc_info.font_faces = std::mem::take(&mut source.font_faces);
    }
    if !source.border_fills.is_empty() {
        document.doc_info.border_fills = std::mem::take(&mut source.border_fills);
    }
    if !source.char_shapes.is_empty() {
        document.doc_info.char_shapes = std::mem::take(&mut source.char_shapes);
    }
    if !source.para_shapes.is_empty() {
        document.doc_info.para_shapes = std::mem::take(&mut source.para_shapes);
    }
    if !source.tab_defs.is_empty() {
        document.doc_info.tab_defs = std::mem::take(&mut source.tab_defs);
    }
    if !source.styles.is_empty() {
        document.doc_info.styles = std::mem::take(&mut source.styles);
    }
}

fn add_default_resources(document: &mut Document) {
    let font = Font {
        name: "함초롬바탕".to_string(),
        ..Default::default()
    };
    document.doc_info.font_faces = (0..7).map(|_| vec![font.clone()]).collect();
    document.doc_info.char_shapes = vec![CharShape {
        ratios: [100; 7],
        relative_sizes: [100; 7],
        base_size: 1000,
        ..Default::default()
    }];
    document.doc_info.para_shapes = vec![ParaShape {
        line_spacing_type: LineSpacingType::Percent,
        line_spacing: 160,
        ..Default::default()
    }];
    document.doc_info.border_fills = vec![BorderFill::default()];
    document.doc_info.tab_defs = vec![TabDef::default()];
    document.doc_info.styles = vec![Style {
        local_name: "바탕글".to_string(),
        english_name: "Normal".to_string(),
        lang_id: 1042,
        ..Default::default()
    }];
}

fn into_paragraph(source: HmlParagraph) -> Result<Paragraph, HmlError> {
    let controls = source
        .controls
        .into_iter()
        .map(into_control)
        .collect::<Result<Vec<_>, _>>()?;
    let mut char_shapes = source.char_shapes;
    if char_shapes.is_empty() {
        char_shapes.push(CharShapeRef::default());
    }
    Ok(Paragraph {
        para_shape_id: source.para_shape_id,
        style_id: source.style_id,
        column_type: source.column_type,
        raw_break_type: source.raw_break_type,
        char_count: source.raw_pos + 1,
        has_para_text: !source.text.is_empty() || !controls.is_empty(),
        text: source.text,
        char_offsets: source.char_offsets,
        char_shapes,
        ctrl_data_records: vec![None; controls.len()],
        controls,
        ..Default::default()
    })
}

fn into_control(source: HmlControl) -> Result<Control, HmlError> {
    match source {
        HmlControl::Equation(equation) => Ok(into_equation(equation)),
        HmlControl::Rectangle(rectangle) => into_rectangle(rectangle),
        HmlControl::Table(table) => into_table(table),
    }
}

fn into_equation(source: HmlEquation) -> Control {
    let (width, height) =
        crate::renderer::equation::intrinsic_size_hwp(&source.script, source.font_size);
    Control::Equation(Box::new(Equation {
        common: CommonObjAttr {
            ctrl_id: crate::parser::tags::CTRL_EQUATION,
            treat_as_char: true,
            width,
            height,
            ..Default::default()
        },
        script: source.script,
        font_size: source.font_size,
        color: source.color,
        baseline: source.baseline,
        version_info: source.version_info,
        font_name: source.font_name,
        ..Default::default()
    }))
}

fn into_rectangle(source: HmlRectangle) -> Result<Control, HmlError> {
    let text_box = if source.text_box.is_empty() {
        None
    } else {
        let paragraphs = source
            .text_box
            .into_iter()
            .map(into_paragraph)
            .collect::<Result<Vec<_>, _>>()?;
        Some(TextBox {
            paragraphs,
            max_width: source.width,
            vertical_align: VerticalAlign::Center,
            margin_left: source.text_margin.left,
            margin_right: source.text_margin.right,
            margin_top: source.text_margin.top,
            margin_bottom: source.text_margin.bottom,
            ..Default::default()
        })
    };
    let common = CommonObjAttr {
        ctrl_id: 0x2472_6563,
        width: source.width,
        height: source.height,
        horizontal_offset: source.horizontal_offset as u32,
        vertical_offset: source.vertical_offset as u32,
        treat_as_char: source.treat_as_char,
        flow_with_text: source.flow_with_text,
        allow_overlap: source.allow_overlap,
        vert_rel_to: source.vert_rel_to,
        vert_align: source.vert_align,
        horz_rel_to: source.horz_rel_to,
        horz_align: source.horz_align,
        text_wrap: source.text_wrap,
        ..Default::default()
    };
    let drawing = DrawingObjAttr {
        shape_attr: source.shape_attr,
        border_line: source.border_line,
        fill: source.fill,
        text_box,
        ..Default::default()
    };
    let x_coords = resolved_coords(source.x_coords, source.width as i32);
    let y_coords = resolved_coords(source.y_coords, source.height as i32);
    Ok(Control::Shape(Box::new(ShapeObject::Rectangle(
        RectangleShape {
            common,
            drawing,
            x_coords,
            y_coords,
            ..Default::default()
        },
    ))))
}

fn resolved_coords(coords: [i32; 4], extent: i32) -> [i32; 4] {
    if coords == [0; 4] {
        [0, extent, extent, 0]
    } else {
        coords
    }
}

fn into_table(source: HmlTable) -> Result<Control, HmlError> {
    let mut cells = Vec::with_capacity(source.cells.len());
    for source_cell in source.cells {
        let paragraphs = source_cell
            .paragraphs
            .into_iter()
            .map(into_paragraph)
            .collect::<Result<Vec<_>, _>>()?;
        cells.push(Cell {
            col: source_cell.col,
            row: source_cell.row,
            col_span: source_cell.col_span,
            row_span: source_cell.row_span,
            width: source_cell.width,
            height: source_cell.height,
            padding: source_cell.padding,
            border_fill_id: source_cell.border_fill_id,
            paragraphs,
            apply_inner_margin: true,
            vertical_align: VerticalAlign::Center,
            ..Default::default()
        });
    }
    let row_sizes = (0..source.row_count)
        .map(|row| cells.iter().filter(|cell| cell.row == row).count() as i16)
        .collect();
    let attr = u32::from(source.common.treat_as_char);
    let mut table = Table {
        attr,
        common: source.common,
        row_count: source.row_count,
        col_count: source.col_count,
        cell_spacing: source.cell_spacing,
        padding: source.padding,
        row_sizes,
        border_fill_id: source.border_fill_id,
        cells,
        page_break: TablePageBreak::CellBreak,
        repeat_header: true,
        ..Default::default()
    };
    table.rebuild_grid();
    Ok(Control::Table(Box::new(table)))
}
