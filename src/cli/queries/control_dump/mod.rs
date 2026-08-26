//! `dump`의 사람용 문서·control 진단 조회.
//!
//! 출력 순서는 역사적 CLI 계약이다. 이 모듈은 인자와 문서 순회를 소유하고, 큰 도형·표·story
//! formatter는 책임별 자식 모듈에 위임한다.

mod shape;
mod story;
mod table;

use std::fs;

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{ColumnBreakType, Paragraph};

use crate::{hu_to_mm, hu_to_mm_i, load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

struct Filters<'a> {
    file_path: &'a str,
    section: Option<usize>,
    paragraph: Option<usize>,
}

pub(crate) fn run(args: &[String]) -> i32 {
    let filters = match parse_args(args) {
        Ok(filters) => filters,
        Err(code) => return code,
    };
    let data = match fs::read(filters.file_path) {
        Ok(data) => data,
        Err(error) => {
            eprintln!(
                "오류: 파일을 읽을 수 없습니다 - {}: {}",
                filters.file_path, error
            );
            return EXIT_RUNTIME;
        }
    };
    let doc = match load_document(&data) {
        Ok(doc) => doc,
        Err(error) => return error.report(),
    };
    dump_document(doc.document(), &filters);
    EXIT_OK
}

fn usage() {
    eprintln!("사용법: rhwp dump <파일.hwp|파일.hwpx|파일.hml> [--section <번호>] [--para <번호>]");
}

fn parse_args(args: &[String]) -> Result<Filters<'_>, i32> {
    if args.is_empty() {
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        usage();
        return Err(EXIT_USAGE);
    }
    let file_path = args[0].as_str();
    if file_path.starts_with('-') {
        eprintln!("오류: 알 수 없는 옵션입니다 - {file_path}");
        usage();
        return Err(EXIT_USAGE);
    }

    let mut section = None;
    let mut paragraph = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--section" | "-s" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("오류: --section 뒤에 0 이상의 구역 번호가 필요합니다.");
                    return Err(EXIT_USAGE);
                };
                section = Some(value.parse::<usize>().map_err(|_| {
                    eprintln!("오류: --section 뒤에는 0 이상의 구역 번호가 필요합니다 - {value}");
                    EXIT_USAGE
                })?);
            }
            "--para" | "-p" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    eprintln!("오류: --para 뒤에 0 이상의 문단 번호가 필요합니다.");
                    return Err(EXIT_USAGE);
                };
                paragraph = Some(value.parse::<usize>().map_err(|_| {
                    eprintln!("오류: --para 뒤에는 0 이상의 문단 번호가 필요합니다 - {value}");
                    EXIT_USAGE
                })?);
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                usage();
                return Err(EXIT_USAGE);
            }
            _ => index += 1,
        }
    }
    Ok(Filters {
        file_path,
        section,
        paragraph,
    })
}

fn dump_document(document: &Document, filters: &Filters<'_>) {
    if filters.section.is_none() && filters.paragraph.is_none() {
        dump_border_fills(document);
    }
    for (section_index, section) in document.sections.iter().enumerate() {
        if filters
            .section
            .is_some_and(|selected| section_index != selected)
        {
            continue;
        }
        dump_section_header(section, section_index);
        for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
            if filters
                .paragraph
                .is_some_and(|selected| paragraph_index != selected)
            {
                continue;
            }
            dump_paragraph(document, section, section_index, paragraph_index, paragraph);
        }
    }
    println!(
        "\n=== 완료: {} 구역, {} 문단 ===",
        document.sections.len(),
        document
            .sections
            .iter()
            .map(|section| section.paragraphs.len())
            .sum::<usize>()
    );
}

fn dump_border_fills(document: &Document) {
    for (index, border_fill) in document.doc_info.border_fills.iter().enumerate() {
        let fill = &border_fill.fill;
        let solid_info = fill
            .solid
            .as_ref()
            .map(|solid| {
                format!(
                    "bg=#{:06X} pat_type={} pat_color=#{:06X}",
                    solid.background_color, solid.pattern_type, solid.pattern_color
                )
            })
            .unwrap_or_default();
        let gradient_info = if fill.gradient.is_some() {
            " gradient"
        } else {
            ""
        };
        let image_info = fill
            .image
            .as_ref()
            .map(|image| {
                format!(
                    " image(bin_id={}, mode={:?}, brightness={}, contrast={}, effect={})",
                    image.bin_data_id,
                    image.fill_mode,
                    image.brightness,
                    image.contrast,
                    image.effect
                )
            })
            .unwrap_or_default();
        println!(
            "  border_fill[{}] fill_type={:?} {}{}{}",
            index, fill.fill_type, solid_info, gradient_info, image_info
        );
    }
}

fn dump_section_header(section: &Section, section_index: usize) {
    let page_def = &section.section_def.page_def;
    println!("=== 구역 {} ===", section_index);
    println!(
        "  용지: {:.1}mm × {:.1}mm ({}×{} HU), {}",
        hu_to_mm(page_def.width),
        hu_to_mm(page_def.height),
        page_def.width,
        page_def.height,
        if page_def.landscape {
            "가로"
        } else {
            "세로"
        }
    );
    println!(
        "  여백: 좌={:.1} 우={:.1} 상={:.1} 하={:.1} 머리말={:.1} 꼬리말={:.1} mm",
        hu_to_mm(page_def.margin_left),
        hu_to_mm(page_def.margin_right),
        hu_to_mm(page_def.margin_top),
        hu_to_mm(page_def.margin_bottom),
        hu_to_mm(page_def.margin_header),
        hu_to_mm(page_def.margin_footer)
    );
    story::dump_master_pages(section);
    if section.section_def.hide_master_page {
        println!("  바탕쪽 감추기: true");
    }
}

fn dump_paragraph(
    document: &Document,
    section: &Section,
    section_index: usize,
    paragraph_index: usize,
    paragraph: &Paragraph,
) {
    let text_preview = if paragraph.text.is_empty() {
        "(빈 문단)".to_string()
    } else if paragraph.text.chars().count() > 50 {
        let end = paragraph
            .text
            .char_indices()
            .nth(50)
            .map(|(index, _)| index)
            .unwrap_or(paragraph.text.len());
        format!("\"{}...\"", &paragraph.text[..end])
    } else {
        format!("\"{}\"", paragraph.text)
    };
    println!(
        "\n--- 문단 {}.{} --- cc={}, text_len={}, controls={} {}",
        section_index,
        paragraph_index,
        paragraph.char_count,
        paragraph.text.chars().count(),
        paragraph.controls.len(),
        break_str(&paragraph.column_type)
    );
    println!("  텍스트: {}", text_preview);
    dump_char_shapes(document, paragraph);
    dump_para_shape(document, paragraph);
    dump_line_segs(paragraph);
    for (control_index, control) in paragraph.controls.iter().enumerate() {
        dump_control(section, control, control_index);
    }
}

fn break_str(value: &ColumnBreakType) -> &str {
    match value {
        ColumnBreakType::None => "",
        ColumnBreakType::Section => "[구역나누기]",
        ColumnBreakType::MultiColumn => "[다단나누기]",
        ColumnBreakType::Page => "[쪽나누기]",
        ColumnBreakType::Column => "[단나누기]",
    }
}

fn dump_char_shapes(document: &Document, paragraph: &Paragraph) {
    let text_chars = paragraph.text.chars().collect::<Vec<_>>();
    for (index, char_shape) in paragraph.char_shapes.iter().enumerate() {
        let next_pos = paragraph
            .char_shapes
            .get(index + 1)
            .map(|next| next.start_pos)
            .unwrap_or(u32::MAX);
        let char_at = text_chars
            .iter()
            .enumerate()
            .find(|(text_index, _)| {
                *text_index < paragraph.char_offsets.len()
                    && paragraph.char_offsets[*text_index] >= char_shape.start_pos
                    && paragraph.char_offsets[*text_index] < next_pos
            })
            .map(|(_, value)| *value);
        let Some(shape) = document
            .doc_info
            .char_shapes
            .get(char_shape.char_shape_id as usize)
        else {
            continue;
        };
        println!(
            "  [CS] pos={} id={} bold={} spacing={}% ratio={}% base={} attr=0x{:08X} text=#{:06X} shade=#{:06X} shadow=#{:06X} border_fill_id={} shadow_type={} shadow_off=({}, {}) char={:?}",
            char_shape.start_pos,
            char_shape.char_shape_id,
            (shape.attr & 0x02) != 0,
            shape.spacings[0],
            shape.ratios[0],
            shape.base_size,
            shape.attr,
            shape.text_color,
            shape.shade_color,
            shape.shadow_color,
            shape.border_fill_id,
            shape.shadow_type,
            shape.shadow_offset_x,
            shape.shadow_offset_y,
            char_at.map(|value| value.to_string()).unwrap_or_default()
        );
    }
}

fn dump_para_shape(document: &Document, paragraph: &Paragraph) {
    let Some(shape) = document
        .doc_info
        .para_shapes
        .get(paragraph.para_shape_id as usize)
    else {
        return;
    };
    println!(
        "  [PS] ps_id={} align={:?} spacing: before={} after={} line={}/{:?}",
        paragraph.para_shape_id,
        shape.alignment,
        shape.spacing_before,
        shape.spacing_after,
        shape.line_spacing,
        shape.line_spacing_type
    );
    println!(
        "       margins: left={} right={} indent={} border_fill_id={}",
        shape.margin_left, shape.margin_right, shape.indent, shape.border_fill_id
    );
    println!(
        "       keep: with_next={} keep_lines={} widow_orphan={} pbreak_before={} (attr1=0x{:08X} attr2=0x{:08X})",
        (shape.attr1 >> 17) & 1 != 0 || (shape.attr2 >> 6) & 1 != 0,
        (shape.attr1 >> 18) & 1 != 0 || (shape.attr2 >> 7) & 1 != 0,
        (shape.attr1 >> 16) & 1 != 0 || (shape.attr2 >> 5) & 1 != 0,
        (shape.attr1 >> 19) & 1 != 0 || (shape.attr2 >> 8) & 1 != 0,
        shape.attr1,
        shape.attr2
    );
    if shape.border_fill_id > 0 {
        println!(
            "       border_spacing: left={} right={} top={} bottom={}",
            shape.border_spacing[0],
            shape.border_spacing[1],
            shape.border_spacing[2],
            shape.border_spacing[3]
        );
    }
    if shape.head_type != rhwp::model::style::HeadType::None {
        println!(
            "       head={:?} level={} num_id={} attr1=0x{:08X} attr2=0x{:08X} raw_extra={:?}",
            shape.head_type,
            shape.para_level,
            shape.numbering_id,
            shape.attr1,
            shape.attr2,
            &paragraph.raw_header_extra
        );
    }
    dump_tab_def(document, shape.tab_def_id);
}

fn dump_tab_def(document: &Document, tab_def_id: u16) {
    if let Some(tab_def) = document.doc_info.tab_defs.get(tab_def_id as usize) {
        let tabs = tab_def
            .tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| {
                format!(
                    "tab[{}] pos={} ({:.1}mm) type={} fill={}",
                    index,
                    tab.position,
                    hu_to_mm(tab.position),
                    tab.tab_type,
                    tab.fill_type
                )
            })
            .collect::<Vec<_>>();
        println!(
            "       tab_def_id={} auto_left={} auto_right={} tabs=[{}]",
            tab_def_id,
            tab_def.auto_tab_left,
            tab_def.auto_tab_right,
            if tabs.is_empty() {
                "(없음)".to_string()
            } else {
                tabs.join(", ")
            }
        );
    } else {
        println!("       tab_def_id={} (정의 없음)", tab_def_id);
    }
}

fn dump_line_segs(paragraph: &Paragraph) {
    for (index, line) in paragraph.line_segs.iter().enumerate() {
        println!(
            "  ls[{}]: ts={}, vpos={}, lh={}, th={}, bl={}, ls={}, cs={}, sw={}, tag=0x{:08X}",
            index,
            line.text_start,
            line.vertical_pos,
            line.line_height,
            line.text_height,
            line.baseline_distance,
            line.line_spacing,
            line.column_start,
            line.segment_width,
            line.tag
        );
    }
}

fn dump_control(section: &Section, control: &Control, control_index: usize) {
    let prefix = format!("  [{}] ", control_index);
    match control {
        Control::ColumnDef(column_def) => dump_column_def(section, column_def, &prefix),
        Control::SectionDef(section_def) => dump_section_def(section_def, &prefix),
        Control::Table(table_value) => table::dump(table_value, &prefix),
        Control::Shape(shape_value) => shape::dump_shape_control(shape_value, &prefix),
        Control::Picture(picture) => shape::dump_picture(picture, &prefix),
        Control::Header(header) => story::dump_header(header, &prefix),
        Control::Footer(footer) => story::dump_footer(footer, &prefix),
        Control::Footnote(footnote) => {
            println!("{}각주: paragraphs={}", prefix, footnote.paragraphs.len())
        }
        Control::Endnote(endnote) => {
            println!("{}미주: paragraphs={}", prefix, endnote.paragraphs.len())
        }
        Control::AutoNumber(value) => println!(
            "{}자동번호: type={:?}, number={}",
            prefix, value.number_type, value.number
        ),
        Control::NewNumber(value) => println!(
            "{}새번호: type={:?}, number={}",
            prefix, value.number_type, value.number
        ),
        Control::PageNumberPos(value) => println!(
            "{}쪽번호위치: format={}, pos={}",
            prefix, value.format, value.position
        ),
        Control::Bookmark(value) => println!("{}책갈피: \"{}\"", prefix, value.name),
        Control::IndexMark(value) => println!(
            "{}찾아보기표식: \"{}\" / \"{}\"",
            prefix, value.first_key, value.second_key
        ),
        Control::PageNumCtrl(value) => {
            println!("{}쪽번호시작쪽: {}", prefix, value.page_starts_on.as_hwpx())
        }
        Control::Hyperlink(value) => println!("{}하이퍼링크: \"{}\"", prefix, value.url),
        Control::Ruby(value) => println!("{}덧말: \"{}\"", prefix, value.ruby_text),
        Control::PageHide(value) => println!(
            "{}감추기: header={}, footer={}, master={}, border={}, fill={}, page_num={}",
            prefix,
            value.hide_header,
            value.hide_footer,
            value.hide_master_page,
            value.hide_border,
            value.hide_fill,
            value.hide_page_num
        ),
        Control::HiddenComment(_) => println!("{}숨은설명", prefix),
        Control::Field(value) => println!(
            "{}필드: {:?} name=\"{}\" cmd=\"{}\"",
            prefix,
            value.field_type,
            value.field_name().unwrap_or("(이름없음)"),
            value.command
        ),
        Control::CharOverlap(value) => println!("{}글자겹침: {:?}", prefix, value.chars),
        Control::Equation(value) => println!(
            "{}수식: script=\"{}\" font_size={} font=\"{}\" size={}x{} tac={}",
            prefix,
            value.script,
            value.font_size,
            value.font_name,
            value.common.width,
            value.common.height,
            value.common.treat_as_char
        ),
        Control::Form(value) => println!(
            "{}양식개체: {:?} name=\"{}\" caption=\"{}\" {}x{}",
            prefix, value.form_type, value.name, value.caption, value.width, value.height
        ),
        Control::Unknown(value) => {
            println!("{}알수없음: ctrl_id={:#010x}", prefix, value.ctrl_id)
        }
    }
}

fn dump_column_def(section: &Section, column_def: &rhwp::model::page::ColumnDef, prefix: &str) {
    let column_type = match column_def.column_type {
        rhwp::model::page::ColumnType::Normal => "일반",
        rhwp::model::page::ColumnType::Distribute => "배분",
        rhwp::model::page::ColumnType::Parallel => "병행",
    };
    println!(
        "{}단정의: {}단, 유형={}, 간격={:.1}mm({}), 같은너비={}",
        prefix,
        column_def.column_count,
        column_type,
        hu_to_mm_i(column_def.spacing as i32),
        column_def.spacing,
        column_def.same_width
    );
    if !column_def.widths.is_empty() {
        dump_column_widths(section, column_def, prefix);
    }
    if column_def.separator_type > 0 {
        println!(
            "{}  구분선: type={}, width={}, color={:#010x}",
            prefix,
            column_def.separator_type,
            column_def.separator_width,
            column_def.separator_color
        );
    }
}

fn dump_column_widths(section: &Section, column_def: &rhwp::model::page::ColumnDef, prefix: &str) {
    let page_def = &section.section_def.page_def;
    let page_width = if page_def.landscape {
        page_def.height
    } else {
        page_def.width
    };
    let body_width =
        (page_width - page_def.margin_left - page_def.margin_right - page_def.margin_gutter) as f64;
    let total = if column_def.proportional_widths {
        column_def
            .widths
            .iter()
            .chain(column_def.gaps.iter())
            .map(|&value| (value as u16) as f64)
            .sum()
    } else {
        1.0
    };
    let columns = column_def
        .widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            let gap = column_def.gaps.get(index).copied().unwrap_or(0);
            if column_def.proportional_widths && total > 0.0 {
                let width_hu = (*width as u16) as f64 / total * body_width;
                let gap_hu = (gap as u16) as f64 / total * body_width;
                format!(
                    "너비={:.1}mm 간격={:.1}mm",
                    width_hu * 25.4 / 7200.0,
                    gap_hu * 25.4 / 7200.0
                )
            } else {
                format!(
                    "너비={:.1}mm 간격={:.1}mm",
                    hu_to_mm_i(*width as i32),
                    hu_to_mm_i(gap as i32)
                )
            }
        })
        .collect::<Vec<_>>();
    println!("{}  단별: [{}]", prefix, columns.join(", "));
}

fn dump_section_def(section_def: &rhwp::model::document::SectionDef, prefix: &str) {
    let page_def = &section_def.page_def;
    println!(
        "{}구역정의: 용지 {:.1}×{:.1}mm, {}, flags=0x{:08X}",
        prefix,
        hu_to_mm(page_def.width),
        hu_to_mm(page_def.height),
        if page_def.landscape {
            "가로"
        } else {
            "세로"
        },
        section_def.flags
    );
    if section_def.hide_header || section_def.hide_footer || section_def.hide_master_page {
        println!(
            "{}  감추기: 머리말={} 꼬리말={} 바탕쪽={}",
            prefix, section_def.hide_header, section_def.hide_footer, section_def.hide_master_page
        );
    }
}
