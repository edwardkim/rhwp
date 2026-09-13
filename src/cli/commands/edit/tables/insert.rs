//! 표 생성 CLI 옵션을 공통 코어 생성 경로에 전달한다.
use std::fs;

use crate::cli::commands::edit::runtime::finish_edit_write;
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};
use rhwp::document_core::{TableColumnWidths, TableCreationOptions};
use rhwp::model::{control::Control, style::Alignment};

pub(in crate::cli::commands::edit) fn edit_insert_table(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-table <파일> --rows N --cols N [--section N --para N --offset N | --at-field 이름] [--widths 2000,3000 | 40%,60%] [--alignments left,center] [--repeat-header true|false] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut rows_arg: Option<u16> = None;
    let mut cols_arg: Option<u16> = None;
    let mut section_arg: usize = 0;
    let mut para_arg: usize = 0;
    let mut offset_arg: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut at_field: Option<String> = None;
    let mut coordinate_given = false;
    let mut options = TableCreationOptions {
        repeat_header: Some(true),
        ..Default::default()
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--rows" | "--cols" | "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                coordinate_given |= matches!(name.as_str(), "--section" | "--para" | "--offset");
                match name.as_str() {
                    "--rows" | "--cols" => match v.parse::<u16>() {
                        Ok(n) if n >= 1 => {
                            if name == "--rows" {
                                rows_arg = Some(n);
                            } else if n > 256 {
                                eprintln!("오류: --cols 는 1~256 이어야 합니다: {v}");
                                return EXIT_USAGE;
                            } else {
                                cols_arg = Some(n);
                            }
                        }
                        _ => {
                            eprintln!("오류: {name} 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--section" => section_arg = n,
                            "--para" => para_arg = n,
                            _ => offset_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--at-field" | "--widths" | "--alignments" | "--repeat-header" => {
                let name = &args[i];
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 값이 필요합니다.");
                    return EXIT_USAGE;
                };
                let parsed = match name.as_str() {
                    "--at-field" if !value.is_empty() => {
                        at_field = Some(value.clone());
                        Ok(())
                    }
                    "--widths" => parse_widths(value).map(|v| options.column_widths = Some(v)),
                    "--alignments" => {
                        parse_alignments(value).map(|v| options.column_alignments = Some(v))
                    }
                    "--repeat-header" => value
                        .parse::<bool>()
                        .map(|v| options.repeat_header = Some(v))
                        .map_err(|_| "--repeat-header 는 true 또는 false여야 합니다".to_string()),
                    _ => Err("필드 이름은 비어 있을 수 없습니다".to_string()),
                };
                if let Err(error) = parsed {
                    eprintln!("오류: {error}");
                    return EXIT_USAGE;
                }
            }
            "--no-repeat-header" => options.repeat_header = Some(false),
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(rows), Some(cols)) = (file_path, rows_arg, cols_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if at_field.is_some() && coordinate_given {
        eprintln!("오류: --at-field 와 --section/--para/--offset 은 함께 사용할 수 없습니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if let Some(name) = &at_field {
        let fields = doc.collect_all_fields();
        let matches: Vec<_> = fields
            .iter()
            .filter(|f| f.field.field_name() == Some(name.as_str()))
            .collect();
        if matches.len() != 1 {
            eprintln!(
                "오류: --at-field 는 유일한 필드 이름이어야 합니다: {name} ({}개)",
                matches.len()
            );
            return EXIT_USAGE;
        }
        let field = matches[0];
        if !field.location.nested_path.is_empty() {
            eprintln!("오류: --at-field 는 본문 필드만 지원합니다. 표 셀·글상자 내부 필드: {name}");
            return EXIT_USAGE;
        }
        // 필드의 시작/끝 제어 문자와 안내문을 분할하지 않는다. 필드 문단 바로
        // 뒤에 독립된 표 host를 마련하므로 빈 필드와 채워진 필드를 모두 보존한다.
        section_arg = field.location.section_index;
        para_arg = field.location.para_index + 1;
        offset_arg = 0;
        if let Err(error) = doc.insert_paragraph_native(section_arg, para_arg) {
            eprintln!("오류: 표 삽입 위치 생성 실패 - {error}");
            return EXIT_RUNTIME;
        }
    }
    let Some(paragraph) = doc
        .document()
        .sections
        .get(section_arg)
        .and_then(|s| s.paragraphs.get(para_arg))
    else {
        eprintln!("오류: 표 삽입 구역·문단이 범위를 벗어났습니다.");
        return EXIT_USAGE;
    };
    if offset_arg > paragraph.text.chars().count() {
        eprintln!("오류: --offset 이 문단 텍스트 길이를 벗어났습니다.");
        return EXIT_USAGE;
    }
    // dry-run도 같은 인메모리 생성·검증을 수행하고 저장만 생략한다.
    let created = match doc.create_table_with_options_native(
        section_arg,
        para_arg,
        offset_arg,
        rows,
        cols,
        &options,
    ) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("오류: 표 생성 실패 - {error}");
            return EXIT_USAGE;
        }
    };
    let created: serde_json::Value = match serde_json::from_str(&created) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("오류: 표 생성 응답 해석 실패 - {error}");
            return EXIT_RUNTIME;
        }
    };
    let (Some(table_para), Some(control)) =
        (created["paraIdx"].as_u64(), created["controlIdx"].as_u64())
    else {
        eprintln!("오류: 생성한 표 좌표가 없습니다.");
        return EXIT_RUNTIME;
    };
    let Control::Table(table) = &doc.document().sections[section_arg].paragraphs
        [table_para as usize]
        .controls[control as usize]
    else {
        eprintln!("오류: 생성한 표를 찾을 수 없습니다.");
        return EXIT_RUNTIME;
    };
    let widths = table.get_column_widths();
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "table",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section_arg,
            "paragraph": para_arg,
            "offset": offset_arg,
            "rows": rows,
            "cols": cols,
            "atField": at_field,
            "tableParagraph": table_para,
            "control": control,
            "widths": widths,
            "repeatHeader": options.repeat_header
        }),
        &[(section_arg, table_para as usize)],
        &format!("표 생성 예정: {file_path} {rows}x{cols} 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg}"),
        &format!("표 생성 완료: {file_path}"),
    )
}

fn parse_widths(value: &str) -> Result<TableColumnWidths, String> {
    let parts: Vec<_> = value.split(',').map(str::trim).collect();
    if parts.iter().all(|p| p.ends_with('%')) {
        parts
            .iter()
            .map(|p| p.strip_suffix('%').unwrap_or(p).parse::<f64>())
            .collect::<Result<Vec<_>, _>>()
            .map(TableColumnWidths::Percent)
            .map_err(|_| "열 비율을 읽을 수 없습니다 (예: 20%,30%,50%)".into())
    } else {
        parts
            .iter()
            .map(|p| p.parse::<u32>())
            .collect::<Result<Vec<_>, _>>()
            .map(TableColumnWidths::Absolute)
            .map_err(|_| "열 너비는 HWPUNIT 정수 또는 모두 %여야 합니다 (단위 혼용 불가)".into())
    }
}

fn parse_alignments(value: &str) -> Result<Vec<Alignment>, String> {
    value
        .split(',')
        .map(|part| match part.trim() {
            "left" => Ok(Alignment::Left),
            "center" => Ok(Alignment::Center),
            "right" => Ok(Alignment::Right),
            _ => Err("열 정렬은 left, center, right 중 하나여야 합니다".into()),
        })
        .collect()
}
