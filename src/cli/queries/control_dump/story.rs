use rhwp::model::control::Control;
use rhwp::model::document::Section;
use rhwp::model::header_footer::{Footer, Header};

pub(super) fn dump_master_pages(section: &Section) {
    if section.section_def.master_pages.is_empty() {
        return;
    }
    println!("  바탕쪽: {}개", section.section_def.master_pages.len());
    for (master_index, master) in section.section_def.master_pages.iter().enumerate() {
        println!("    [{}] {:?}, 문단 {}개, 영역 {}×{} HU, is_ext={}, overlap={}, ext_flags=0x{:04X}, text_ref={}, num_ref={}",
            master_index, master.apply_to, master.paragraphs.len(), master.text_width,
            master.text_height, master.is_extension, master.overlap, master.ext_flags,
            master.text_ref, master.num_ref);
        for (paragraph_index, paragraph) in master.paragraphs.iter().enumerate() {
            println!(
                "      p[{}]: cc={}, text=\"{}\"",
                paragraph_index,
                paragraph.controls.len(),
                if paragraph.text.is_empty() {
                    "(빈 문단)".to_string()
                } else {
                    paragraph.text.chars().take(30).collect::<String>()
                }
            );
            for (control_index, control) in paragraph.controls.iter().enumerate() {
                println!(
                    "        ctrl[{}]: {}",
                    control_index,
                    master_control_name(control)
                );
            }
        }
    }
}

fn master_control_name(control: &Control) -> String {
    match control {
        Control::Table(table) => {
            let cell_texts = table
                .cells
                .iter()
                .take(3)
                .map(|cell| {
                    cell.paragraphs
                        .iter()
                        .map(|paragraph| paragraph.text.chars().take(20).collect::<String>())
                        .collect::<Vec<_>>()
                        .join("|")
                })
                .collect::<Vec<_>>();
            format!(
                "표({}x{}, tac={}, wrap={:?}, vert={:?}/{}, horz={:?}/{}, size={}x{}, cells=[{}])",
                table.row_count,
                table.col_count,
                table.common.treat_as_char,
                table.common.text_wrap,
                table.common.vert_rel_to,
                table.common.vertical_offset,
                table.common.horz_rel_to,
                table.common.horizontal_offset,
                table.common.width,
                table.common.height,
                cell_texts.join("; ")
            )
        }
        Control::Shape(shape) => master_shape_name(shape),
        Control::Picture(picture) => {
            let watermark = picture
                .image_attr
                .watermark_preset()
                .map(|preset| format!(", watermark={}", preset))
                .unwrap_or_default();
            format!(
                "그림(bin_id={}, w={}, h={}, tac={}{})",
                picture.image_attr.bin_data_id,
                picture.common.width,
                picture.common.height,
                picture.common.treat_as_char,
                watermark
            )
        }
        Control::Header(_) => "머리말".to_string(),
        Control::Footer(_) => "꼬리말".to_string(),
        _ => format!("{:?}", std::mem::discriminant(control)),
    }
}

fn master_shape_name(shape: &rhwp::model::shape::ShapeObject) -> String {
    let mut description = format!(
        "도형(ctrl_id=0x{:08X}, w={}, h={}, attr=0x{:08X}, wc={:?}, hc={:?})",
        shape.common().ctrl_id,
        shape.common().width,
        shape.common().height,
        shape.common().attr,
        shape.common().width_criterion,
        shape.common().height_criterion
    );
    if let Some(text_box) = shape
        .drawing()
        .and_then(|drawing| drawing.text_box.as_ref())
    {
        description += &format!(" 글상자({}문단)", text_box.paragraphs.len());
        for (paragraph_index, paragraph) in text_box.paragraphs.iter().enumerate() {
            let text = paragraph.text.chars().take(20).collect::<String>();
            description += &format!(
                "\n          tb_p[{}]: cc={} text=\"{}\"",
                paragraph_index,
                paragraph.controls.len(),
                text
            );
            for (control_index, control) in paragraph.controls.iter().enumerate() {
                let name = match control {
                    Control::AutoNumber(auto_number) => {
                        format!("자동번호({:?})", auto_number.number_type)
                    }
                    _ => format!("{:?}", std::mem::discriminant(control)),
                };
                description += &format!("\n            tb_ctrl[{}]: {}", control_index, name);
            }
        }
    }
    description
}

pub(super) fn dump_header(header: &Header, prefix: &str) {
    let text = header
        .paragraphs
        .iter()
        .filter(|paragraph| !paragraph.text.is_empty())
        .map(|paragraph| paragraph.text.clone())
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "{}머리말({:?}): paras={} \"{}\"",
        prefix,
        header.apply_to,
        header.paragraphs.len(),
        text
    );
    for (paragraph_index, paragraph) in header.paragraphs.iter().enumerate() {
        for (control_index, control) in paragraph.controls.iter().enumerate() {
            let name = header_control_name(control, prefix);
            let display = if name.chars().count() > 30 {
                format!(
                    "{}...(truncated)",
                    name.chars().take(30).collect::<String>()
                )
            } else {
                name
            };
            println!(
                "{}  hp[{}] ctrl[{}]: {}",
                prefix, paragraph_index, control_index, display
            );
        }
    }
}

fn header_control_name(control: &Control, prefix: &str) -> String {
    match control {
        Control::AutoNumber(auto_number) => format!("자동번호({:?})", auto_number.number_type),
        Control::Shape(shape) => story_shape_name(shape),
        Control::Table(table) => header_table_name(table, prefix),
        Control::Picture(picture) => picture_name(picture),
        _ => format!("{:?}", std::mem::discriminant(control)),
    }
}

fn story_shape_name(shape: &rhwp::model::shape::ShapeObject) -> String {
    let common = shape.common();
    let mut description = format!(
        "Shape horz={:?}/{} halign={:?} w={} h={}",
        common.horz_rel_to,
        common.horizontal_offset,
        common.horz_align,
        common.width,
        common.height
    );
    if let Some(text_box) = shape
        .drawing()
        .and_then(|drawing| drawing.text_box.as_ref())
    {
        let text = text_box
            .paragraphs
            .iter()
            .flat_map(|paragraph| paragraph.text.chars().take(20))
            .collect::<String>();
        description += &format!(" text={:?}", text);
    }
    description
}

fn header_table_name(table: &rhwp::model::table::Table, prefix: &str) -> String {
    let mut description = format!(
        "표 {}행×{}열 셀={}",
        table.row_count,
        table.col_count,
        table.cells.len()
    );
    for (cell_index, cell) in table.cells.iter().enumerate() {
        let cell_text = cell
            .paragraphs
            .iter()
            .flat_map(|paragraph| paragraph.text.chars().take(20))
            .collect::<String>();
        description += &format!("\n{}    셀[{}] text={:?}", prefix, cell_index, cell_text);
        for (paragraph_index, paragraph) in cell.paragraphs.iter().enumerate() {
            for (control_index, control) in paragraph.controls.iter().enumerate() {
                description += &format!(
                    "\n{}      p[{}]c[{}]: {}",
                    prefix,
                    paragraph_index,
                    control_index,
                    header_cell_control_name(control)
                );
            }
        }
    }
    description
}

fn header_cell_control_name(control: &Control) -> String {
    match control {
        Control::AutoNumber(auto_number) => format!("자동번호({:?})", auto_number.number_type),
        Control::Shape(shape) => {
            let common = shape.common();
            let mut description = format!(
                "Shape vert={:?}/{} valign={:?} horz={:?}/{} halign={:?} w={} h={}",
                common.vert_rel_to,
                common.vertical_offset,
                common.vert_align,
                common.horz_rel_to,
                common.horizontal_offset,
                common.horz_align,
                common.width,
                common.height
            );
            if let Some(text_box) = shape
                .drawing()
                .and_then(|drawing| drawing.text_box.as_ref())
            {
                for (paragraph_index, paragraph) in text_box.paragraphs.iter().enumerate() {
                    description += &format!(
                        " tb_p[{}] ps_id={} text={:?}",
                        paragraph_index,
                        paragraph.para_shape_id,
                        paragraph.text.chars().take(30).collect::<String>()
                    );
                }
            }
            description
        }
        _ => format!("{:?}", std::mem::discriminant(control)),
    }
}

pub(super) fn dump_footer(footer: &Footer, prefix: &str) {
    let text = footer
        .paragraphs
        .iter()
        .filter(|paragraph| !paragraph.text.is_empty())
        .map(|paragraph| paragraph.text.clone())
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "{}꼬리말({:?}): paras={} \"{}\"",
        prefix,
        footer.apply_to,
        footer.paragraphs.len(),
        text
    );
    for (paragraph_index, paragraph) in footer.paragraphs.iter().enumerate() {
        for (control_index, control) in paragraph.controls.iter().enumerate() {
            let name = match control {
                Control::Picture(picture) => picture_name(picture),
                _ => format!("{:?}", std::mem::discriminant(control)),
            };
            println!(
                "{}  fp[{}] ctrl[{}]: {}",
                prefix, paragraph_index, control_index, name
            );
        }
    }
}

fn picture_name(picture: &rhwp::model::image::Picture) -> String {
    let attr = &picture.shape_attr;
    format!("그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}, crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
        picture.image_attr.bin_data_id, picture.common.width, picture.common.height,
        picture.common.width as f64 / 7200.0 * 25.4, picture.common.height as f64 / 7200.0 * 25.4,
        attr.original_width, attr.original_height, attr.original_width as f64 / 7200.0 * 25.4,
        attr.original_height as f64 / 7200.0 * 25.4, attr.current_width, attr.current_height,
        attr.current_width as f64 / 7200.0 * 25.4, attr.current_height as f64 / 7200.0 * 25.4,
        picture.common.treat_as_char, picture.crop.left, picture.crop.top, picture.crop.right,
        picture.crop.bottom, picture.crop.left as f64 / 7200.0 * 25.4,
        picture.crop.top as f64 / 7200.0 * 25.4, picture.crop.right as f64 / 7200.0 * 25.4,
        picture.crop.bottom as f64 / 7200.0 * 25.4)
}
