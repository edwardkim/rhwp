//! 본문과 표 셀의 글자·문단·스타일 서식 편집 명령.

use std::fs;

use super::runtime::finish_edit_write;
use super::tables::{resolve_table_cell, CellResolveError};
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

/// `edit apply-char-format` — 본문 문단 글자 범위에 글자 서식을 적용한다.
pub(super) fn edit_apply_char_format(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-char-format <파일> --props <JSON> [--section N] [--para N] [--offset N] [--count N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut count_arg: Option<usize> = None;
    let mut props_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        "--offset" => offset = n,
                        _ => count_arg = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 글자 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
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
    let (Some(file_path), Some(props)) = (file_path, props_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
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
    let Some(sec) = doc.document().sections.get(section) else {
        eprintln!("오류: 구역 {section} 이 없습니다.");
        return EXIT_USAGE;
    };
    let Some(paragraph) = sec.paragraphs.get(para) else {
        eprintln!("오류: 문단 {para} 이 없습니다.");
        return EXIT_USAGE;
    };
    let para_len = paragraph.text.chars().count();
    if offset > para_len {
        eprintln!("오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {para_len}): {offset}");
        return EXIT_USAGE;
    }
    let end = match count_arg {
        Some(n) => offset.saturating_add(n).min(para_len),
        None => para_len,
    };
    if !dry_run {
        if let Err(e) = doc.apply_char_format_native(section, para, offset, end, props) {
            eprintln!("오류: 글자 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "chfmt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "count": end.saturating_sub(offset),
            "text": props
        }),
        &[(section, para)],
        &format!("글자 서식 적용 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("글자 서식 적용 완료: {file_path}"),
    )
}

/// `edit apply-para-format` — 본문 문단에 문단 서식을 적용한다.
pub(super) fn edit_apply_para_format(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-para-format <파일> --props <JSON> [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut props_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        _ => para = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 문단 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
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
    let (Some(file_path), Some(props)) = (file_path, props_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
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
    let Some(sec) = doc.document().sections.get(section) else {
        eprintln!("오류: 구역 {section} 이 없습니다.");
        return EXIT_USAGE;
    };
    if sec.paragraphs.get(para).is_none() {
        eprintln!("오류: 문단 {para} 이 없습니다.");
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.apply_para_format_native(section, para, props) {
            eprintln!("오류: 문단 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "pfmt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "text": props
        }),
        &[(section, para)],
        &format!("문단 서식 적용 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("문단 서식 적용 완료: {file_path}"),
    )
}

/// `edit apply-char-format-in-cell` — 표 셀 글자 서식. 코어 `apply_char_format_in_cell_native`.
pub(super) fn edit_apply_char_format_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-char-format-in-cell <파일> (--table N --row N --col N | --section N --para N --ctrl N --cell N) [--cell-para N] [--start N] [--end N] [--offset N] [--count N] [--props JSON] [--bold] [--font-size N] [--color 색] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut section_arg: Option<usize> = None;
    let mut para_arg: Option<usize> = None;
    let mut ctrl_arg: Option<usize> = None;
    let mut cell_arg: Option<usize> = None;
    let mut cell_para_arg: usize = 0;
    let mut start_arg: Option<usize> = None;
    let mut end_arg: Option<usize> = None;
    let mut offset_arg: Option<usize> = None;
    let mut count_arg: Option<usize> = None;
    let mut props_arg: Option<String> = None;
    let mut bold_flag = false;
    let mut font_size_arg: Option<i32> = None;
    let mut color_arg: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--section" | "--para" | "--ctrl" | "--cell"
            | "--cell-para" | "--start" | "--end" | "--offset" | "--count" | "--font-size" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--col" => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--font-size" => match v.parse::<i32>() {
                        Ok(n) if n > 0 => font_size_arg = Some(n),
                        _ => {
                            eprintln!("오류: --font-size 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            "--section" => section_arg = Some(n),
                            "--para" => para_arg = Some(n),
                            "--ctrl" => ctrl_arg = Some(n),
                            "--cell" => cell_arg = Some(n),
                            "--cell-para" => cell_para_arg = n,
                            "--start" => start_arg = Some(n),
                            "--end" => end_arg = Some(n),
                            "--offset" => offset_arg = Some(n),
                            _ => count_arg = Some(n),
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props_arg = Some(v.clone()),
                    _ => {
                        eprintln!("오류: --props 뒤에 글자 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--color" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => color_arg = Some(v.clone()),
                    _ => {
                        eprintln!("오류: --color 뒤에 색 값이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--bold" => bold_flag = true,
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
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let mut props_val = match props_arg.as_deref() {
        Some(raw) => match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(serde_json::Value::Object(map)) => serde_json::Value::Object(map),
            Ok(_) | Err(_) => {
                eprintln!("오류: --props 는 JSON 객체여야 합니다: {raw}");
                return EXIT_USAGE;
            }
        },
        None => serde_json::json!({}),
    };
    if bold_flag {
        props_val["bold"] = serde_json::json!(true);
    }
    if let Some(n) = font_size_arg {
        props_val["fontSize"] = serde_json::json!(n);
    }
    if let Some(ref c) = color_arg {
        props_val["textColor"] = serde_json::json!(c);
    }
    if props_val.as_object().is_none_or(|o| o.is_empty()) {
        eprintln!("오류: --props 또는 --bold/--font-size/--color 가 필요합니다.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }
    let props = props_val.to_string();
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
    let (sec, parent_para, ctrl, cell_idx, para_lens, table_no, row, col) = match (
        table_arg,
        row_arg,
        col_arg,
        section_arg,
        para_arg,
        ctrl_arg,
        cell_arg,
    ) {
        (Some(table_no), Some(row), Some(col), _, _, _, _) => {
            match resolve_table_cell(doc.document(), table_no, row, col) {
                Ok((sec, para, ctrl, cell_idx, para_lens, _)) => {
                    (sec, para, ctrl, cell_idx, para_lens, table_no, row, col)
                }
                Err(CellResolveError::Usage(msg)) => {
                    eprintln!("{msg}");
                    return EXIT_USAGE;
                }
                Err(CellResolveError::Runtime(msg)) => {
                    eprintln!("{msg}");
                    return EXIT_RUNTIME;
                }
            }
        }
        (_, _, _, Some(sec), Some(para), Some(ctrl), Some(cell_idx)) => {
            let para_lens = match cell_para_lens(doc.document(), sec, para, ctrl, cell_idx) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return EXIT_USAGE;
                }
            };
            (sec, para, ctrl, cell_idx, para_lens, 0, 0, 0)
        }
        _ => {
            eprintln!("{USAGE}");
            return EXIT_USAGE;
        }
    };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_len = para_lens[cell_para_arg];
    let start = start_arg.or(offset_arg).unwrap_or(0);
    if start > para_len {
        eprintln!("오류: --start/--offset 이 문단 길이를 넘습니다 (문단 길이 {para_len}): {start}");
        return EXIT_USAGE;
    }
    let end = if let Some(e) = end_arg {
        e
    } else if let Some(n) = count_arg {
        start.saturating_add(n)
    } else {
        para_len
    };
    if end < start || end > para_len {
        eprintln!("오류: --end 가 범위를 벗어났습니다 (시작 {start}, 문단 길이 {para_len}): {end}");
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.apply_char_format_in_cell_native(
            sec,
            parent_para,
            ctrl,
            cell_idx,
            cell_para_arg,
            start,
            end,
            &props,
        ) {
            eprintln!("오류: 셀 글자 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "chfmtcell",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "section": sec,
            "paragraph": parent_para,
            "ctrl": ctrl,
            "cellPara": cell_para_arg,
            "innerPara": cell_para_arg,
            "offset": start,
            "count": end.saturating_sub(start),
            "text": props,
            "props": props,
            "bold": bold_flag,
            "fontSize": font_size_arg,
            "color": color_arg,
        }),
        &[(sec, parent_para)],
        &format!(
            "셀 글자 서식 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} {start}..{end}"
        ),
        &format!("셀 글자 서식 적용 완료: {file_path}"),
    )
}

fn cell_para_lens(
    document: &rhwp::model::document::Document,
    section: usize,
    para: usize,
    ctrl: usize,
    cell_idx: usize,
) -> Result<Vec<usize>, String> {
    use rhwp::model::control::Control;
    let Some(sec) = document.sections.get(section) else {
        return Err(format!("오류: 구역 {section} 이 없습니다."));
    };
    let Some(paragraph) = sec.paragraphs.get(para) else {
        return Err(format!("오류: 문단 {para} 이 없습니다."));
    };
    let Some(Control::Table(table)) = paragraph.controls.get(ctrl) else {
        return Err(format!("오류: 문단 {para} 컨트롤 {ctrl} 은 표가 아닙니다."));
    };
    let Some(cell) = table.cells.get(cell_idx) else {
        return Err(format!(
            "오류: --cell 이 범위를 벗어났습니다 (셀 0~{}): {cell_idx}",
            table.cells.len().saturating_sub(1)
        ));
    };
    Ok(cell
        .paragraphs
        .iter()
        .map(|p| p.text.chars().count())
        .collect())
}

/// `edit apply-style` — 본문 문단에 스타일을 적용한다.
pub(super) fn edit_apply_style(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-style <파일> --style N [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut style_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--style" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        _ => style_arg = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
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
    let (Some(file_path), Some(style_id)) = (file_path, style_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
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
    if style_id >= doc.document().doc_info.styles.len() {
        eprintln!(
            "오류: --style 이 범위를 벗어났습니다 (스타일 0~{}): {style_id}",
            doc.document().doc_info.styles.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let Some(sec) = doc.document().sections.get(section) else {
        eprintln!("오류: 구역 {section} 이 없습니다.");
        return EXIT_USAGE;
    };
    if sec.paragraphs.get(para).is_none() {
        eprintln!("오류: 문단 {para} 이 없습니다.");
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.apply_style_native(section, para, style_id) {
            eprintln!("오류: 스타일 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "style",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": style_id
        }),
        &[(section, para)],
        &format!("스타일 적용 예정: {file_path} 구역 {section} 문단 {para} 스타일 {style_id}"),
        &format!("스타일 적용 완료: {file_path}"),
    )
}

/// `edit apply-cell-style` — 표 셀 문단에 스타일을 적용한다.
pub(super) fn edit_apply_cell_style(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-cell-style <파일> --table <번호> --row <행> --col <열> --style N [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut cell_para_arg: usize = 0;
    let mut style_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--cell-para" | "--style" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--cell-para" | "--style" => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            "--style" => style_arg = Some(n),
                            _ => cell_para_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
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
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(style_id)) =
        (file_path, table_arg, row_arg, col_arg, style_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
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
    if style_id >= doc.document().doc_info.styles.len() {
        eprintln!(
            "오류: --style 이 범위를 벗어났습니다 (스타일 0~{}): {style_id}",
            doc.document().doc_info.styles.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) =
            doc.apply_cell_style_native(sec, para, ctrl, cell_idx, cell_para_arg, style_id)
        {
            eprintln!("오류: 셀 스타일 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellstyle",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "paragraph": cell_para_arg,
            "ctrl": style_id
        }),
        &[(sec, para)],
        &format!(
            "셀 스타일 적용 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} 스타일 {style_id}"
        ),
        &format!("셀 스타일 적용 완료: {file_path}"),
    )
}

/// `edit apply-para-format-in-cell` — 표 셀 문단에 문단 서식을 적용한다.
pub(super) fn edit_apply_para_format_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-para-format-in-cell <파일> --table <번호> --row <행> --col <열> --props <JSON> [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut cell_para_arg: usize = 0;
    let mut props_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--cell-para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--cell-para" => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            _ => cell_para_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 문단 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
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
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(props)) =
        (file_path, table_arg, row_arg, col_arg, props_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
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
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) =
            doc.apply_para_format_in_cell_native(sec, para, ctrl, cell_idx, cell_para_arg, props)
        {
            eprintln!("오류: 셀 문단 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellpfmt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "paragraph": cell_para_arg,
            "text": props
        }),
        &[(sec, para)],
        &format!(
            "셀 문단 서식 적용 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg}"
        ),
        &format!("셀 문단 서식 적용 완료: {file_path}"),
    )
}
