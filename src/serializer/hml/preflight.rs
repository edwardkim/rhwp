use crate::model::control::Control;
use crate::model::document::Document;
use crate::model::paragraph::{ColumnBreakType, Paragraph};
use crate::model::shape::{
    CommonObjAttr, RectangleShape, ShapeComponentAttr, ShapeObject, TextBox,
};
use crate::model::style::{
    BorderFill, BorderLineType, CharShape, Fill, FillType, ParaShape, ShapeBorderLine, TabDef,
};
use crate::model::table::{Cell, Table, TablePageBreak, VerticalAlign};
use crate::parser::HmlImportMetadata;

use super::error::{HmlExportError, HmlSaveBlocker};

pub(crate) fn validate_document(
    document: &Document,
    metadata: &HmlImportMetadata,
) -> Result<(), HmlExportError> {
    let blockers = collect_blockers(document, metadata);
    if blockers.is_empty() {
        Ok(())
    } else {
        Err(HmlExportError::UnsupportedIr { blockers })
    }
}

pub(crate) fn collect_blockers(
    document: &Document,
    metadata: &HmlImportMetadata,
) -> Vec<HmlSaveBlocker> {
    let mut blockers = Vec::new();
    validate_metadata(document, metadata, &mut blockers);
    validate_resources(document, &mut blockers);
    validate_binary_data(document, &mut blockers);
    for (section_index, section) in document.sections.iter().enumerate() {
        validate_section(section, section_index, &mut blockers);
        for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
            validate_paragraph(
                paragraph,
                &format!("/HWPML/BODY/SECTION[{section_index}]/P[{paragraph_index}]"),
                &mut blockers,
            );
        }
    }
    blockers
}

fn validate_metadata(
    document: &Document,
    metadata: &HmlImportMetadata,
    blockers: &mut Vec<HmlSaveBlocker>,
) {
    for value in [&metadata.sub_version, &metadata.style]
        .into_iter()
        .flatten()
    {
        validate_xml_value(value, "/HWPML", blockers);
    }
    for fragment in &metadata.preserved_fragments {
        if let Err(message) = super::raw_fragment::validate(fragment, document.sections.len()) {
            blockers.push(HmlSaveBlocker {
                code: "HML_INVALID_RAW_FRAGMENT",
                xml_path: fragment.xml_path.clone(),
                message,
            });
        }
    }
}

fn validate_resources(document: &Document, blockers: &mut Vec<HmlSaveBlocker>) {
    let info = &document.doc_info;
    validate_fonts(document, blockers);
    for (index, shape) in info.char_shapes.iter().enumerate() {
        validate_char_shape(
            shape,
            &resource_path("CHARSHAPELIST", "CHARSHAPE", index),
            blockers,
        );
    }
    for (index, tab) in info.tab_defs.iter().enumerate() {
        validate_tab_def(tab, &resource_path("TABDEFLIST", "TABDEF", index), blockers);
    }
    for (index, shape) in info.para_shapes.iter().enumerate() {
        validate_para_shape(
            shape,
            &resource_path("PARASHAPELIST", "PARASHAPE", index),
            blockers,
        );
    }
    for (index, fill) in info.border_fills.iter().enumerate() {
        validate_border_fill(fill, index, blockers);
    }
    for (index, style) in info.styles.iter().enumerate() {
        let path = resource_path("STYLELIST", "STYLE", index);
        validate_xml_value(&style.local_name, &path, blockers);
        validate_xml_value(&style.english_name, &path, blockers);
        let unsupported = style.raw_data.is_some() || style.style_type > 1;
        push_if(
            unsupported,
            &path,
            "style fields cannot round-trip through HML",
            blockers,
        );
    }
}

fn validate_fonts(document: &Document, blockers: &mut Vec<HmlSaveBlocker>) {
    let path = "/HWPML/HEAD/MAPPINGTABLE/FACENAMELIST";
    push_if(
        document.doc_info.font_faces.len() != 7,
        path,
        "HML requires all seven language font groups",
        blockers,
    );
    for (language, fonts) in document.doc_info.font_faces.iter().enumerate().take(7) {
        for (index, font) in fonts.iter().enumerate() {
            let font_path = format!("{path}/FONTFACE[{language}]/FONT[{index}]");
            validate_xml_value(&font.name, &font_path, blockers);
            let omitted = font.raw_data.is_some()
                || font.alt_type > 2
                || font.alt_name.is_some()
                || font.type_info.is_some()
                || font.default_name.is_some()
                || font.subst_font.is_some();
            push_if(
                omitted,
                &font_path,
                "font fields omitted by the HML reader",
                blockers,
            );
        }
    }
}

fn validate_binary_data(document: &Document, blockers: &mut Vec<HmlSaveBlocker>) {
    push_if(
        !document.doc_info.bin_data_list.is_empty(),
        "/HWPML/HEAD/MAPPINGTABLE/BINDATALIST",
        "BinData descriptors are not mapped by the HML reader",
        blockers,
    );
    push_if(
        !document.bin_data_content.is_empty(),
        "/HWPML/BINDATA",
        "embedded BinData content is not mapped by the HML reader",
        blockers,
    );
}

fn validate_section(
    section: &crate::model::document::Section,
    index: usize,
    blockers: &mut Vec<HmlSaveBlocker>,
) {
    let path = format!("/HWPML/BODY/SECTION[{index}]");
    push_if(
        section.section_def.flags != 0,
        &path,
        "section flags are not mapped by the HML reader",
        blockers,
    );
    push_if(
        section.section_def.page_def.binding != Default::default(),
        &format!("{path}/P[0]/TEXT/SECDEF/PAGEDEF"),
        "page binding is not mapped by the HML reader",
        blockers,
    );
}

fn resource_path(list: &str, item: &str, index: usize) -> String {
    format!("/HWPML/HEAD/MAPPINGTABLE/{list}/{item}[{index}]")
}

fn validate_char_shape(shape: &CharShape, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    let default = CharShape::default();
    let omitted = shape.raw_data.is_some()
        || shape.attr != default.attr
        || shape.italic != default.italic
        || shape.bold != default.bold
        || shape.underline_type != default.underline_type
        || shape.outline_type != default.outline_type
        || shape.shadow_type != default.shadow_type
        || shape.shadow_offset_x != default.shadow_offset_x
        || shape.shadow_offset_y != default.shadow_offset_y
        || shape.underline_color != default.underline_color
        || shape.shadow_color != default.shadow_color
        || shape.strike_color != default.strike_color
        || shape.strikethrough != default.strikethrough
        || shape.subscript != default.subscript
        || shape.superscript != default.superscript
        || shape.emboss != default.emboss
        || shape.engrave != default.engrave
        || shape.emphasis_dot != default.emphasis_dot
        || shape.underline_shape != default.underline_shape
        || shape.strike_shape != default.strike_shape
        || shape.kerning != default.kerning
        || shape.use_font_space != default.use_font_space;
    push_if(
        omitted,
        path,
        "character-shape fields omitted by the HML reader",
        blockers,
    );
}

fn validate_tab_def(tab: &TabDef, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    let omitted = tab.raw_data.is_some()
        || tab.attr != 0
        || !tab.tabs.is_empty()
        || tab.auto_tab_left
        || tab.auto_tab_right;
    push_if(
        omitted,
        path,
        "tab-definition fields omitted by the HML reader",
        blockers,
    );
}

fn validate_para_shape(shape: &ParaShape, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    let omitted = shape.raw_data.is_some()
        || shape.attr1 != 0
        || shape.numbering_id != 0
        || shape.border_spacing != [0; 4]
        || shape.attr2 != 0
        || shape.attr3 != 0
        || shape.line_spacing_v2 != 0
        || shape.head_type != Default::default()
        || shape.break_latin_word.is_some();
    push_if(
        omitted,
        path,
        "paragraph-shape fields omitted by the HML reader",
        blockers,
    );
}

fn validate_border_fill(fill: &BorderFill, index: usize, blockers: &mut Vec<HmlSaveBlocker>) {
    let path = resource_path("BORDERFILLLIST", "BORDERFILL", index);
    let container_omitted = fill.raw_data.is_some()
        || fill.attr != 0
        || fill.diagonal.diagonal_type != 0
        || fill.diagonal.width != 0
        || fill.diagonal.color != 0
        || fill.center_line != Default::default()
        || !is_default_fill(&fill.fill);
    push_if(
        container_omitted,
        &path,
        "border-fill fields omitted by the HML reader",
        blockers,
    );
    for (name, line) in ["LEFTBORDER", "RIGHTBORDER", "TOPBORDER", "BOTTOMBORDER"]
        .into_iter()
        .zip(fill.borders)
    {
        let unsupported = !matches!(line.line_type, BorderLineType::None | BorderLineType::Solid)
            || line.width > 15
            || line.color != 0;
        push_if(
            unsupported,
            &format!("{path}/{name}"),
            "border-line value cannot round-trip through HML",
            blockers,
        );
    }
}

fn is_default_fill(fill: &Fill) -> bool {
    fill.fill_type == FillType::None
        && fill.solid.is_none()
        && fill.gradient.is_none()
        && fill.image.is_none()
        && fill.alpha == 0
}

fn validate_paragraph(paragraph: &Paragraph, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    validate_xml_value(&paragraph.text, path, blockers);
    push_if(
        !paragraph_layout_is_reconstructable(paragraph),
        path,
        "paragraph character offsets cannot reconstruct inline control positions",
        blockers,
    );
    push_if(
        !paragraph_char_shapes_are_reconstructable(paragraph),
        path,
        "paragraph character-shape runs cannot be reconstructed by the HML reader",
        blockers,
    );
    let supported_break = matches!(
        (paragraph.column_type, paragraph.raw_break_type),
        (ColumnBreakType::None, 0)
            | (ColumnBreakType::Page, 0x04)
            | (ColumnBreakType::Column, 0x08)
    );
    push_if(
        !supported_break,
        path,
        "paragraph break kind cannot round-trip through HML",
        blockers,
    );
    for (index, control) in paragraph.controls.iter().enumerate() {
        let control_path = format!("{path}/CONTROL[{index}]");
        match control {
            Control::Table(table) => {
                validate_table(table, &format!("{control_path}/TABLE"), blockers)
            }
            Control::Equation(equation) => {
                validate_equation(equation, &format!("{control_path}/EQUATION"), blockers)
            }
            Control::Shape(shape) => match shape.as_ref() {
                ShapeObject::Rectangle(rectangle) => {
                    validate_rectangle(rectangle, &format!("{control_path}/RECTANGLE"), blockers)
                }
                _ => blockers.push(unsupported(
                    control_path,
                    "shape kind is not mapped by the HML reader",
                )),
            },
            _ => blockers.push(unsupported(
                control_path,
                "control kind is not mapped by the HML reader",
            )),
        }
    }
}

fn validate_equation(
    equation: &crate::model::control::Equation,
    path: &str,
    blockers: &mut Vec<HmlSaveBlocker>,
) {
    validate_xml_value(&equation.script, &format!("{path}/SCRIPT"), blockers);
    validate_xml_value(&equation.version_info, path, blockers);
    validate_xml_value(&equation.font_name, path, blockers);

    let expected_size =
        crate::renderer::equation::intrinsic_size_hwp(&equation.script, equation.font_size);
    push_if(
        (equation.common.width, equation.common.height) != expected_size,
        path,
        "equation intrinsic size does not match its script and BaseUnit",
        blockers,
    );
    let mut common_without_packed_attr = equation.common.clone();
    common_without_packed_attr.attr = 0;
    validate_common(
        &common_without_packed_attr,
        crate::parser::tags::CTRL_EQUATION,
        path,
        blockers,
    );
    let common = &equation.common;
    let packed_attr =
        crate::document_core::converters::common_obj_attr_writer::pack_common_attr_bits(common);
    let omitted = !matches!(common.attr, 0) && common.attr != packed_attr
        || common.vertical_offset != 0
        || common.horizontal_offset != 0
        || !common.treat_as_char
        || common.flow_with_text
        || common.allow_overlap
        || common.vert_rel_to != Default::default()
        || common.vert_align != Default::default()
        || common.horz_rel_to != Default::default()
        || common.horz_align != Default::default()
        || common.text_wrap != Default::default();
    push_if(
        omitted,
        path,
        "equation object placement is not represented by HML EQUATION",
        blockers,
    );
    push_if(
        equation.unknown != 0 || !equation.raw_ctrl_data.is_empty(),
        path,
        "equation binary-only fields are not represented by HML EQUATION",
        blockers,
    );
}

fn validate_xml_value(value: &str, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    if let Err(message) = super::raw_fragment::validate_xml_chars(value) {
        blockers.push(HmlSaveBlocker {
            code: "HML_INVALID_XML_CHARACTER",
            xml_path: path.to_string(),
            message,
        });
    }
}

fn paragraph_layout_is_reconstructable(paragraph: &Paragraph) -> bool {
    paragraph_run_offsets(paragraph).is_some_and(|(_, raw_end)| paragraph.char_count == raw_end + 1)
}

fn paragraph_char_shapes_are_reconstructable(paragraph: &Paragraph) -> bool {
    let Some((run_offsets, _)) = paragraph_run_offsets(paragraph) else {
        return true;
    };
    let mut expected = Vec::new();
    for offset in run_offsets {
        let shape_id = char_shape_at(paragraph, offset);
        if expected
            .last()
            .is_none_or(|(_, previous_id)| *previous_id != shape_id)
        {
            expected.push((offset, shape_id));
        }
    }
    paragraph.char_shapes.len() == expected.len()
        && paragraph
            .char_shapes
            .iter()
            .zip(expected)
            .all(|(actual, expected)| (actual.start_pos, actual.char_shape_id) == expected)
}

fn paragraph_run_offsets(paragraph: &Paragraph) -> Option<(Vec<u32>, u32)> {
    let characters = paragraph.text.chars().collect::<Vec<_>>();
    if paragraph.char_offsets.len() != characters.len() {
        return None;
    }
    let mut run_offsets = Vec::with_capacity(characters.len() + paragraph.controls.len());
    let mut expected = 0u32;
    let mut controls = 0usize;
    for (character, actual) in characters.iter().zip(&paragraph.char_offsets) {
        while controls < paragraph.controls.len() && expected + 8 <= *actual {
            run_offsets.push(expected);
            expected += 8;
            controls += 1;
        }
        if *actual != expected {
            return None;
        }
        run_offsets.push(*actual);
        expected += character.len_utf16() as u32;
    }
    while controls < paragraph.controls.len() {
        run_offsets.push(expected);
        expected += 8;
        controls += 1;
    }
    if run_offsets.is_empty() {
        run_offsets.push(0);
    }
    Some((run_offsets, expected))
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

fn validate_rectangle(rectangle: &RectangleShape, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    let coords_canonicalize = (rectangle.x_coords == [0; 4] && rectangle.common.width != 0)
        || (rectangle.y_coords == [0; 4] && rectangle.common.height != 0);
    push_if(
        rectangle.round_rate != 0 || coords_canonicalize,
        path,
        "rectangle geometry omitted or canonicalized by the HML reader",
        blockers,
    );
    let size_canonicalizes = (rectangle.common.width == 0
        && rectangle.drawing.shape_attr.original_width != 0)
        || (rectangle.common.height == 0 && rectangle.drawing.shape_attr.original_height != 0);
    push_if(
        size_canonicalizes,
        &format!("{path}/SHAPEOBJECT"),
        "zero object size is replaced from original shape size by the HML reader",
        blockers,
    );
    validate_common(&rectangle.common, 0x2472_6563, path, blockers);
    validate_shape_attr(&rectangle.drawing.shape_attr, path, blockers);
    validate_shape_line(&rectangle.drawing.border_line, path, blockers);
    validate_shape_fill(&rectangle.drawing.fill, path, blockers);
    validate_drawing_extras(rectangle, path, blockers);
    if let Some(text_box) = &rectangle.drawing.text_box {
        validate_text_box(text_box, rectangle.common.width, path, blockers);
        for (index, paragraph) in text_box.paragraphs.iter().enumerate() {
            validate_paragraph(paragraph, &format!("{path}/DRAWTEXT/P[{index}]"), blockers);
        }
    }
}

fn validate_common(
    common: &CommonObjAttr,
    ctrl_id: u32,
    path: &str,
    blockers: &mut Vec<HmlSaveBlocker>,
) {
    let omitted = common.ctrl_id != ctrl_id
        || common.attr != 0
        || common.z_order != 0
        || padding_nonzero(common.margin)
        || common.instance_id != 0
        || common.prevent_page_break != 0
        || common.hwp5_gen_shape_attr_bit26
        || common.size_protect
        || common.hwp5_gen_shape_attr_bit28
        || common.text_flow != Default::default()
        || common.width_criterion != Default::default()
        || common.height_criterion != Default::default()
        || !common.description.is_empty()
        || common.numbering_type != Default::default()
        || !common.raw_extra.is_empty();
    push_if(
        omitted,
        &format!("{path}/SHAPEOBJECT"),
        "common-object fields omitted by the HML reader",
        blockers,
    );
}

fn padding_nonzero(padding: crate::model::Padding) -> bool {
    [padding.left, padding.right, padding.top, padding.bottom] != [0; 4]
}

fn validate_shape_attr(shape: &ShapeComponentAttr, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    let default = ShapeComponentAttr::default();
    let omitted = shape.ctrl_id != default.ctrl_id
        || shape.is_two_ctrl_id != default.is_two_ctrl_id
        || shape.group_level != default.group_level
        || shape.local_file_version != default.local_file_version
        || shape.current_width_was_zero
        || shape.current_height_was_zero
        || shape.flip != default.flip
        || shape.horz_flip != default.horz_flip
        || shape.vert_flip != default.vert_flip
        || shape.rotation_angle != default.rotation_angle
        || shape.rotate_image != default.rotate_image
        || shape.rotation_center.x != default.rotation_center.x
        || shape.rotation_center.y != default.rotation_center.y
        || !shape.raw_rendering.is_empty()
        || shape.render_tx != default.render_tx
        || shape.render_ty != default.render_ty
        || shape.render_sx != default.render_sx
        || shape.render_sy != default.render_sy
        || shape.render_b != default.render_b
        || shape.render_c != default.render_c
        || (shape.current_width == 0 && shape.original_width != 0)
        || (shape.current_height == 0 && shape.original_height != 0);
    push_if(
        omitted,
        &format!("{path}/DRAWINGOBJECT/SHAPECOMPONENT"),
        "shape-component fields cannot round-trip through HML",
        blockers,
    );
}

fn validate_shape_line(line: &ShapeBorderLine, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    let omitted = line.attr != 65 || line.color != 0 || line.outline_style != 0;
    push_if(
        omitted,
        &format!("{path}/DRAWINGOBJECT/LINESHAPE"),
        "line fields cannot round-trip through HML",
        blockers,
    );
}

fn validate_shape_fill(fill: &Fill, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    let supported = if fill.fill_type == FillType::None {
        is_default_fill(fill)
    } else {
        fill.fill_type == FillType::Solid
            && fill.solid.is_some_and(|solid| solid.pattern_type == -1)
            && fill.gradient.is_none()
            && fill.image.is_none()
    };
    push_if(
        !supported,
        &format!("{path}/DRAWINGOBJECT/FILLBRUSH"),
        "fill fields cannot round-trip through HML",
        blockers,
    );
}

fn validate_drawing_extras(
    rectangle: &RectangleShape,
    path: &str,
    blockers: &mut Vec<HmlSaveBlocker>,
) {
    let drawing = &rectangle.drawing;
    let omitted = drawing.shadow_type != 0
        || drawing.shadow_color != 0
        || drawing.shadow_offset_x != 0
        || drawing.shadow_offset_y != 0
        || drawing.inst_id != 0
        || drawing.shadow_alpha != 0
        || drawing.caption.is_some();
    push_if(
        omitted,
        &format!("{path}/DRAWINGOBJECT"),
        "drawing fields omitted by the HML reader",
        blockers,
    );
}

fn validate_text_box(
    text_box: &TextBox,
    width: u32,
    path: &str,
    blockers: &mut Vec<HmlSaveBlocker>,
) {
    let omitted = text_box.paragraphs.is_empty()
        || text_box.list_attr != 0
        || text_box.vertical_all
        || text_box.vertical_align != VerticalAlign::Center
        || text_box.max_width != width
        || !text_box.raw_list_header_extra.is_empty();
    push_if(
        omitted,
        &format!("{path}/DRAWTEXT"),
        "text-box fields cannot round-trip through HML",
        blockers,
    );
}

fn table_attr_has_unrepresentable_bits(table: &Table) -> bool {
    table.attr & !0x01 != 0 || (table.attr & 0x01 != 0 && !table.common.treat_as_char)
}

fn validate_table(table: &Table, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    let expected_rows = (0..table.row_count)
        .map(|row| table.cells.iter().filter(|cell| cell.row == row).count() as i16)
        .collect::<Vec<_>>();
    let mut rebuilt = table.clone();
    rebuilt.rebuild_grid();
    let omitted = table_attr_has_unrepresentable_bits(table)
        || table.page_break != TablePageBreak::CellBreak
        || !table.repeat_header
        || !table.zones.is_empty()
        || table.caption.is_some()
        || [
            table.outer_margin_left,
            table.outer_margin_right,
            table.outer_margin_top,
            table.outer_margin_bottom,
        ] != [0; 4]
        || !table.raw_ctrl_data.is_empty()
        || table.raw_table_record_attr != 0
        || !table.raw_table_record_extra.is_empty()
        || table.row_sizes != expected_rows
        || table.cell_grid != rebuilt.cell_grid
        || table.common.text_wrap != Default::default()
        || table
            .cells
            .windows(2)
            .any(|cells| cells[0].row > cells[1].row)
        || table
            .cells
            .iter()
            .any(|cell| cell.row >= table.row_count || cell.col >= table.col_count);
    push_if(
        omitted,
        path,
        "table fields cannot round-trip through HML",
        blockers,
    );
    validate_common(&table.common, 0, path, blockers);
    for (index, cell) in table.cells.iter().enumerate() {
        validate_cell(cell, &format!("{path}/CELL[{index}]"), blockers);
    }
}

fn validate_cell(cell: &Cell, path: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    let omitted = cell.list_header_width_ref != 0
        || cell.text_direction != 0
        || cell.vertical_align != VerticalAlign::Center
        || !cell.apply_inner_margin
        || cell.is_header
        || !cell.raw_list_extra.is_empty()
        || cell.field_name.is_some();
    push_if(
        omitted,
        path,
        "cell fields cannot round-trip through HML",
        blockers,
    );
    for (index, paragraph) in cell.paragraphs.iter().enumerate() {
        validate_paragraph(paragraph, &format!("{path}/P[{index}]"), blockers);
    }
}

fn push_if(condition: bool, path: &str, message: &str, blockers: &mut Vec<HmlSaveBlocker>) {
    if condition {
        blockers.push(unsupported(path.to_string(), message));
    }
}

fn unsupported(path: String, message: impl Into<String>) -> HmlSaveBlocker {
    HmlSaveBlocker {
        code: "HML_UNSUPPORTED_IR",
        xml_path: path,
        message: message.into(),
    }
}
