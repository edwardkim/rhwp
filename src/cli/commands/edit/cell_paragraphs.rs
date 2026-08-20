//! 표 셀 문단 구조 편집 명령.

use std::fs;

use super::runtime::finish_edit_write;
use super::tables::{resolve_table_cell, CellResolveError};
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

/// `edit split-paragraph-in-cell` — 표 셀 문단을 오프셋에서 나눈다.
pub(super) fn edit_split_paragraph_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-paragraph-in-cell <파일> --table <번호> --row <행> --col <열> [--cell-para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut cell_para_arg: usize = 0;
    let mut offset_arg: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--offset" | "--cell-para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--offset" | "--cell-para" => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            "--offset" => offset_arg = n,
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
    let (Some(file_path), Some(table_no), Some(row), Some(col)) =
        (file_path, table_arg, row_arg, col_arg)
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
    if offset_arg > para_lens[cell_para_arg] {
        eprintln!(
            "오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {}): {offset_arg}",
            para_lens[cell_para_arg]
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.split_paragraph_in_cell_native(
            sec,
            para,
            ctrl,
            cell_idx,
            cell_para_arg,
            offset_arg,
            None,
        ) {
            eprintln!("오류: 셀 문단 분할 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellsplit",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "paragraph": cell_para_arg,
            "offset": offset_arg
        }),
        &[(sec, para)],
        &format!(
            "셀 문단 분할 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} 오프셋 {offset_arg}"
        ),
        &format!("셀 문단 분할 완료: {file_path}"),
    )
}

/// `edit merge-paragraph-in-cell` — 표 셀 문단을 바로 앞 문단과 합친다.
pub(super) fn edit_merge_paragraph_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit merge-paragraph-in-cell <파일> --table <번호> --row <행> --col <열> [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut cell_para_arg: usize = 1;
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
    let (Some(file_path), Some(table_no), Some(row), Some(col)) =
        (file_path, table_arg, row_arg, col_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if cell_para_arg == 0 {
        eprintln!("오류: --cell-para 는 1 이상이어야 합니다 (첫 문단은 병합할 수 없습니다).");
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
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 1~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.merge_paragraph_in_cell_native(sec, para, ctrl, cell_idx, cell_para_arg)
        {
            eprintln!("오류: 셀 문단 병합 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellmerge",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "paragraph": cell_para_arg
        }),
        &[(sec, para)],
        &format!("셀 문단 병합 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg}"),
        &format!("셀 문단 병합 완료: {file_path}"),
    )
}
