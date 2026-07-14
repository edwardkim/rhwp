use crate::model::document::Document;
use crate::model::style::{
    border_width_mm_str, Alignment, BorderLine, BorderLineType, CharShape, LineSpacingType,
    ParaShape,
};
use crate::parser::HmlImportMetadata;

use super::error::{unsupported_ir, HmlExportError};
use super::xml::XmlWriter;

const LANGUAGE_NAMES: [&str; 7] = [
    "Hangul", "Latin", "Hanja", "Japanese", "Other", "Symbol", "User",
];

pub(crate) fn write_head(
    writer: &mut XmlWriter,
    document: &Document,
    metadata: &HmlImportMetadata,
) -> Result<(), HmlExportError> {
    writer.open("HEAD", &[("SecCnt", document.sections.len().to_string())]);
    super::fragments::write_at_anchor(writer, metadata, "HEAD", 0);
    writer.open("MAPPINGTABLE", &[]);
    write_fonts(writer, document);
    write_border_fills(writer, document)?;
    write_char_shapes(writer, document);
    write_tab_defs(writer, document);
    write_para_shapes(writer, document);
    write_styles(writer, document);
    writer.close("MAPPINGTABLE");
    super::fragments::write_at_anchor(writer, metadata, "HEAD", 1);
    writer.close("HEAD");
    Ok(())
}

fn write_fonts(writer: &mut XmlWriter, document: &Document) {
    writer.open("FACENAMELIST", &[]);
    for (language_index, fonts) in document.doc_info.font_faces.iter().enumerate().take(7) {
        let language = LANGUAGE_NAMES[language_index];
        writer.open(
            "FONTFACE",
            &[
                ("Count", fonts.len().to_string()),
                ("Lang", language.into()),
            ],
        );
        for (id, font) in fonts.iter().enumerate() {
            let mut attributes = vec![("Id", id.to_string()), ("Name", font.name.clone())];
            let font_type = match font.alt_type {
                1 => Some("ttf"),
                2 => Some("hft"),
                _ => None,
            };
            if let Some(font_type) = font_type {
                attributes.push(("Type", font_type.to_string()));
            }
            writer.empty("FONT", &attributes);
        }
        writer.close("FONTFACE");
    }
    writer.close("FACENAMELIST");
}

fn write_border_fills(writer: &mut XmlWriter, document: &Document) -> Result<(), HmlExportError> {
    writer.open(
        "BORDERFILLLIST",
        &[("Count", document.doc_info.border_fills.len().to_string())],
    );
    for (index, fill) in document.doc_info.border_fills.iter().enumerate() {
        writer.open("BORDERFILL", &[("Id", (index + 1).to_string())]);
        for (name, line) in [
            ("LEFTBORDER", fill.borders[0]),
            ("RIGHTBORDER", fill.borders[1]),
            ("TOPBORDER", fill.borders[2]),
            ("BOTTOMBORDER", fill.borders[3]),
        ] {
            let path =
                format!("/HWPML/HEAD/MAPPINGTABLE/BORDERFILLLIST/BORDERFILL[{index}]/{name}");
            write_border_line(writer, name, line, &path)?;
        }
        writer.close("BORDERFILL");
    }
    writer.close("BORDERFILLLIST");
    Ok(())
}

fn write_border_line(
    writer: &mut XmlWriter,
    name: &str,
    line: BorderLine,
    path: &str,
) -> Result<(), HmlExportError> {
    let line_type = match line.line_type {
        BorderLineType::None => "None",
        BorderLineType::Solid => "Solid",
        value => {
            return Err(unsupported_ir(
                path,
                format!("HML reader가 역변환할 수 없는 테두리선 종류입니다: {value:?}"),
            ))
        }
    };
    writer.empty(
        name,
        &[
            ("Type", line_type.into()),
            ("Width", format!("{}mm", border_width_mm_str(line.width))),
        ],
    );
    Ok(())
}

fn write_char_shapes(writer: &mut XmlWriter, document: &Document) {
    writer.open(
        "CHARSHAPELIST",
        &[("Count", document.doc_info.char_shapes.len().to_string())],
    );
    for (id, shape) in document.doc_info.char_shapes.iter().enumerate() {
        writer.open(
            "CHARSHAPE",
            &[
                ("Id", id.to_string()),
                ("Height", shape.base_size.to_string()),
                ("BorderFillId", shape.border_fill_id.to_string()),
                ("TextColor", shape.text_color.to_string()),
                ("ShadeColor", shape.shade_color.to_string()),
            ],
        );
        write_language_array(writer, "FONTID", &shape.font_ids);
        write_language_array(writer, "RATIO", &shape.ratios);
        write_language_array(writer, "CHARSPACING", &shape.spacings);
        write_language_array(writer, "RELSIZE", &shape.relative_sizes);
        write_language_array(writer, "CHAROFFSET", &shape.char_offsets);
        writer.close("CHARSHAPE");
    }
    writer.close("CHARSHAPELIST");
}

fn write_language_array<T: ToString>(writer: &mut XmlWriter, name: &str, values: &[T; 7]) {
    let attributes = LANGUAGE_NAMES
        .iter()
        .zip(values)
        .map(|(language, value)| (*language, value.to_string()))
        .collect::<Vec<_>>();
    writer.empty(name, &attributes);
}

fn write_tab_defs(writer: &mut XmlWriter, document: &Document) {
    writer.open(
        "TABDEFLIST",
        &[("Count", document.doc_info.tab_defs.len().to_string())],
    );
    for (id, _) in document.doc_info.tab_defs.iter().enumerate() {
        writer.empty("TABDEF", &[("Id", id.to_string())]);
    }
    writer.close("TABDEFLIST");
}

fn write_para_shapes(writer: &mut XmlWriter, document: &Document) {
    writer.open(
        "PARASHAPELIST",
        &[("Count", document.doc_info.para_shapes.len().to_string())],
    );
    for (id, shape) in document.doc_info.para_shapes.iter().enumerate() {
        write_para_shape(writer, id, shape);
    }
    writer.close("PARASHAPELIST");
}

fn write_para_shape(writer: &mut XmlWriter, id: usize, shape: &ParaShape) {
    writer.open(
        "PARASHAPE",
        &[
            ("Id", id.to_string()),
            ("Align", alignment_name(shape.alignment).into()),
            ("TabDef", shape.tab_def_id.to_string()),
            ("Level", shape.para_level.to_string()),
        ],
    );
    writer.empty(
        "PARAMARGIN",
        &[
            ("Left", shape.margin_left.to_string()),
            ("Right", shape.margin_right.to_string()),
            ("Indent", shape.indent.to_string()),
            ("Prev", shape.spacing_before.to_string()),
            ("Next", shape.spacing_after.to_string()),
            ("LineSpacing", shape.line_spacing.to_string()),
            (
                "LineSpacingType",
                line_spacing_name(shape.line_spacing_type).into(),
            ),
        ],
    );
    writer.empty(
        "PARABORDER",
        &[("BorderFill", shape.border_fill_id.to_string())],
    );
    writer.close("PARASHAPE");
}

fn write_styles(writer: &mut XmlWriter, document: &Document) {
    writer.open(
        "STYLELIST",
        &[("Count", document.doc_info.styles.len().to_string())],
    );
    for (id, style) in document.doc_info.styles.iter().enumerate() {
        writer.empty(
            "STYLE",
            &[
                ("Id", id.to_string()),
                ("Name", style.local_name.clone()),
                ("EngName", style.english_name.clone()),
                (
                    "Type",
                    if style.style_type == 1 {
                        "Char"
                    } else {
                        "Para"
                    }
                    .into(),
                ),
                ("NextStyle", style.next_style_id.to_string()),
                ("LangId", style.lang_id.to_string()),
                ("ParaShape", style.para_shape_id.to_string()),
                ("CharShape", style.char_shape_id.to_string()),
            ],
        );
    }
    writer.close("STYLELIST");
}

fn alignment_name(value: Alignment) -> &'static str {
    match value {
        Alignment::Left => "Left",
        Alignment::Right => "Right",
        Alignment::Center => "Center",
        Alignment::Distribute => "Distribute",
        Alignment::Split => "Split",
        Alignment::Justify => "Justify",
    }
}

fn line_spacing_name(value: LineSpacingType) -> &'static str {
    match value {
        LineSpacingType::Fixed => "Fixed",
        LineSpacingType::SpaceOnly => "BetweenLines",
        LineSpacingType::Minimum => "AtLeast",
        LineSpacingType::Percent => "Percent",
    }
}
