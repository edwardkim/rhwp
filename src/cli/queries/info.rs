//! `info` 문서 메타 조회 어댑터.
//!
//! 사람용 출력과 JSON 봉투 선택만 소유한다. JSON schema의 단일 원천인
//! `info_json_value`는 batch·digest·MCP와 공유하므로 crate root seam을 호출한다.

use std::fs;

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::FileFormat;
use rhwp::wasm_api::HwpDocument;

use crate::{info_json_value, load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

struct InfoArgs<'a> {
    file_path: &'a str,
    json_mode: bool,
}

pub(crate) fn run(args: &[String]) -> i32 {
    let options = match parse_args(args) {
        Ok(options) => options,
        Err(code) => return code,
    };

    let data = match fs::read(options.file_path) {
        Ok(data) => data,
        Err(error) => {
            eprintln!(
                "오류: 파일을 읽을 수 없습니다 - {}: {}",
                options.file_path, error
            );
            return EXIT_RUNTIME;
        }
    };
    let file_size = data.len();
    let detected_format = rhwp::parser::detect_format(&data);
    let doc = match load_document(&data) {
        Ok(doc) => doc,
        Err(error) => return error.report(),
    };

    if options.json_mode {
        let info = info_json_value(options.file_path, file_size, detected_format, &doc);
        println!("{info}");
        return EXIT_OK;
    }

    print_human_info(options.file_path, file_size, detected_format, &doc);
    EXIT_OK
}

fn parse_args(args: &[String]) -> Result<InfoArgs<'_>, i32> {
    let mut json_mode = false;
    let mut file_path: Option<&str> = None;
    for arg in args {
        match arg.as_str() {
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return Err(EXIT_USAGE);
                }
            }
        }
    }
    let Some(file_path) = file_path else {
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        return Err(EXIT_USAGE);
    };
    Ok(InfoArgs {
        file_path,
        json_mode,
    })
}

fn print_human_info(
    file_path: &str,
    file_size: usize,
    detected_format: FileFormat,
    doc: &HwpDocument,
) {
    let document = doc.document();
    print_hml_details(detected_format, document, doc);
    print_file_header(file_path, file_size, detected_format, document, doc);
    print_page_defs(document);
    print_fonts(document);
    print_styles_and_origin(document);
    print_bin_data(document);
    print_control_inventory(document);
}

fn print_hml_details(detected_format: FileFormat, document: &Document, doc: &HwpDocument) {
    if detected_format != FileFormat::Hml {
        return;
    }
    println!("format: HML");
    println!(
        "hwpml_version: {}",
        document
            .doc_info
            .hwpml_version
            .as_deref()
            .unwrap_or("unknown")
    );
    println!("sections: {}", document.sections.len());
    println!("pages: {}", doc.page_count());
    if let Some(metadata) = doc.hml_metadata() {
        let encoding = match metadata.encoding {
            rhwp::parser::hml::HmlEncoding::Utf8 => "UTF-8",
            rhwp::parser::hml::HmlEncoding::Utf16Le => "UTF-16LE",
            rhwp::parser::hml::HmlEncoding::Utf16Be => "UTF-16BE",
        };
        println!("encoding: {encoding}");
        println!("resources: {}", metadata.resource_count);
        println!("warnings: {}", metadata.warnings.len());
        for warning in &metadata.warnings {
            eprintln!(
                "warning [{:?}] {}: {}",
                warning.code, warning.xml_path, warning.message
            );
        }
    }
}

fn print_file_header(
    file_path: &str,
    file_size: usize,
    detected_format: FileFormat,
    document: &Document,
    doc: &HwpDocument,
) {
    println!("파일: {}", file_path);
    println!("크기: {} bytes", file_size);
    if detected_format != FileFormat::Hml {
        println!(
            "버전: {}.{}.{}.{}",
            document.header.version.major,
            document.header.version.minor,
            document.header.version.build,
            document.header.version.revision,
        );
        println!(
            "압축: {}",
            if document.header.compressed {
                "예"
            } else {
                "아니오"
            }
        );
        println!(
            "암호화: {}",
            if document.header.encrypted {
                "예"
            } else {
                "아니오"
            }
        );
        println!(
            "배포용: {}",
            if document.header.distribution {
                "예"
            } else {
                "아니오"
            }
        );
    }
    println!("구역 수: {}", document.sections.len());
    println!("페이지 수: {}", doc.page_count());
}

fn print_page_defs(document: &Document) {
    for (sec_idx, section) in document.sections.iter().enumerate() {
        let page_def = &section.section_def.page_def;
        let orientation = if page_def.landscape {
            "가로"
        } else {
            "세로"
        };
        println!(
            "구역{} 용지: {}×{} HWPUNIT, 방향={} (여백: 좌{} 우{} 상{} 하{})",
            sec_idx,
            page_def.width,
            page_def.height,
            orientation,
            page_def.margin_left,
            page_def.margin_right,
            page_def.margin_top,
            page_def.margin_bottom,
        );
        println!(
            "  머리말여백={} 꼬리말여백={} 제본여백={}",
            page_def.margin_header, page_def.margin_footer, page_def.margin_gutter
        );
        if section.section_def.hide_empty_line {
            println!("  빈 줄 감추기: 활성");
        }
    }
}

fn print_fonts(document: &Document) {
    let lang_names = ["한글", "영어", "한자", "일어", "기타", "기호", "사용자"];
    for (index, fonts) in document.doc_info.font_faces.iter().enumerate() {
        if fonts.is_empty() {
            continue;
        }
        let name = lang_names.get(index).copied().unwrap_or("기타");
        let font_names = fonts
            .iter()
            .enumerate()
            .map(|(font_index, font)| format!("[{}]{}", font_index, font.name))
            .collect::<Vec<_>>();
        println!("폰트({}): {}", name, font_names.join(", "));
    }
}

fn print_styles_and_origin(document: &Document) {
    if !document.doc_info.styles.is_empty() {
        let style_names = document
            .doc_info
            .styles
            .iter()
            .map(|style| style.local_name.as_str())
            .collect::<Vec<_>>();
        println!("스타일: {}", style_names.join(", "));
    }

    let total_paras = document
        .sections
        .iter()
        .map(|section| section.paragraphs.len())
        .sum::<usize>();
    println!("총 문단 수: {}", total_paras);
    if total_paras == 0 {
        return;
    }

    let ps_count = document.doc_info.para_shapes.len();
    let cs_count = document.doc_info.char_shapes.len();
    let ps_ratio = ps_count as f64 / total_paras as f64;
    let cs_ratio = cs_count as f64 / total_paras as f64;
    let origin = if total_paras > 50 && ps_ratio < 0.05 && cs_ratio < 0.15 {
        "HWP3 변환본 추정 (margin_bottom -1600 HU 보정 적용)"
    } else if total_paras <= 50 {
        "판정 불가 (문단 수 ≤ 50, 비율 왜곡 회피)"
    } else {
        "한컴 한글 직접 작성 추정"
    };
    println!("ParaShape: {} (PS/문단 = {:.3})", ps_count, ps_ratio);
    println!("CharShape: {} (CS/문단 = {:.3})", cs_count, cs_ratio);
    println!("Origin 추정: {}", origin);
}

fn print_bin_data(document: &Document) {
    if document.doc_info.bin_data_list.is_empty() {
        return;
    }
    println!("BinData:");
    for (index, bin_data) in document.doc_info.bin_data_list.iter().enumerate() {
        let type_str = match bin_data.data_type {
            rhwp::model::bin_data::BinDataType::Link => "Link",
            rhwp::model::bin_data::BinDataType::Embedding => "Embedding",
            rhwp::model::bin_data::BinDataType::Storage => "Storage",
        };
        let extension = bin_data.extension.as_deref().unwrap_or("?");
        let loaded_size = document
            .bin_data_content
            .iter()
            .find(|content| content.id == bin_data.storage_id)
            .map(|content| content.data.len())
            .unwrap_or(0);
        println!(
            "  [{}] {} (ID: {}, ext: {}, loaded: {} bytes)",
            index, type_str, bin_data.storage_id, extension, loaded_size
        );
    }
}

fn print_control_inventory(document: &Document) {
    let mut table_index = 0;
    let mut picture_index = 0;
    for (section_index, section) in document.sections.iter().enumerate() {
        for (paragraph_index, paragraph) in section.paragraphs.iter().enumerate() {
            for control in &paragraph.controls {
                let location = format!("구역{}:문단{}", section_index, paragraph_index);
                match control {
                    Control::Table(table) => {
                        table_index += 1;
                        let page_break = match table.page_break {
                            rhwp::model::table::TablePageBreak::None => "나누지 않음",
                            rhwp::model::table::TablePageBreak::CellBreak => "셀 단위 나눔",
                            rhwp::model::table::TablePageBreak::RowBreak => "나눔(행 단위)",
                        };
                        println!(
                            "표{} [{}]: {}행×{}열, 셀 {}개, 쪽나눔={} (attr=0x{:08x}), 제목반복={}",
                            table_index,
                            location,
                            table.row_count,
                            table.col_count,
                            table.cells.len(),
                            page_break,
                            table.raw_table_record_attr,
                            table.repeat_header,
                        );
                        count_pictures(control, &mut picture_index, &location);
                    }
                    Control::Picture(_) => {
                        count_pictures(control, &mut picture_index, &location);
                    }
                    Control::Shape(shape) => {
                        let common = shape.common();
                        let border_info = match shape.as_ref() {
                            ShapeObject::Rectangle(rectangle) => format!(
                                ", border(color={:#010x}, width={}, attr={:#010x})",
                                rectangle.drawing.border_line.color,
                                rectangle.drawing.border_line.width,
                                rectangle.drawing.border_line.attr,
                            ),
                            ShapeObject::Line(line) => format!(
                                ", border(color={:#010x}, width={}, attr={:#010x})",
                                line.drawing.border_line.color,
                                line.drawing.border_line.width,
                                line.drawing.border_line.attr,
                            ),
                            _ => String::new(),
                        };
                        println!(
                            "도형 [{}]: {}, size={}×{}, treat_as_char={}{}",
                            location,
                            shape.shape_name(),
                            common.width,
                            common.height,
                            common.treat_as_char,
                            border_info,
                        );
                        print_group_children(shape);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn count_pictures(control: &Control, picture_index: &mut usize, location: &str) {
    match control {
        Control::Picture(picture) => {
            *picture_index += 1;
            println!(
                "그림{} [{}]: bin_data_id={}, size={}×{}",
                *picture_index,
                location,
                picture.image_attr.bin_data_id,
                picture.common.width,
                picture.common.height,
            );
        }
        Control::Table(table) => {
            for (cell_index, cell) in table.cells.iter().enumerate() {
                for (paragraph_index, paragraph) in cell.paragraphs.iter().enumerate() {
                    for child in &paragraph.controls {
                        let nested_location =
                            format!("{}→셀{}:문단{}", location, cell_index, paragraph_index);
                        count_pictures(child, picture_index, &nested_location);
                    }
                }
            }
        }
        _ => {}
    }
}

fn print_group_children(shape: &ShapeObject) {
    let ShapeObject::Group(group) = shape else {
        return;
    };
    for (index, child) in group.children.iter().enumerate() {
        let attr = child.shape_attr();
        let effective_width = (attr.current_width as f64 * attr.render_sx) as i32;
        let effective_height = (attr.current_height as f64 * attr.render_sy) as i32;
        println!(
            "  자식[{}]: {}, orig={}×{}, scale=({:.3},{:.3}), eff={}×{} at ({:.0},{:.0})",
            index,
            child.shape_name(),
            attr.current_width,
            attr.current_height,
            attr.render_sx,
            attr.render_sy,
            effective_width,
            effective_height,
            attr.render_tx,
            attr.render_ty
        );
    }
}
