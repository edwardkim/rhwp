use std::fs;

use crate::{
    cli::commands::edit::runtime::finish_edit_write, load_document, EXIT_RUNTIME, EXIT_USAGE,
};

/// [#5036] `edit insert-header-footer` — 머리말/꼬리말 생성.
pub(super) fn edit_insert_header_footer(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-header-footer <파일> --header|--footer [--section N] [--apply-to 0|1|2] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                if name == "--section" {
                    match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    }
                } else {
                    match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
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
    let (Some(file_path), Some(is_header)) = (file_path, is_header) else {
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
    if !dry_run {
        if let Err(e) = doc.create_header_footer_native(section, is_header, apply_to) {
            eprintln!("오류: 머리말/꼬리말 생성 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hf",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to
        }),
        &[(section, 0)],
        &format!("머리말/꼬리말 생성 예정: {file_path} 구역 {section} apply-to {apply_to}"),
        &format!("머리말/꼬리말 생성 완료: {file_path}"),
    )
}

/// [#5039] `edit delete-header-footer` — 머리말/꼬리말 삭제.
pub(super) fn edit_delete_header_footer(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-header-footer <파일> --header|--footer [--section N] [--apply-to 0|1|2] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                if name == "--section" {
                    match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    }
                } else {
                    match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
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
    let (Some(file_path), Some(is_header)) = (file_path, is_header) else {
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
    if !dry_run {
        if let Err(e) = doc.delete_header_footer_native(section, is_header, apply_to) {
            eprintln!("오류: 머리말/꼬리말 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delhf",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to
        }),
        &[(section, 0)],
        &format!("{kind} 삭제 예정: {file_path} 구역 {section} apply-to {apply_to}"),
        &format!("{kind} 삭제 완료: {file_path}"),
    )
}

/// `edit insert-header-footer-text` — 기존 머리말/꼬리말 문단에 텍스트 삽입.
pub(super) fn edit_insert_header_footer_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-header-footer-text <파일> --header|--footer --text <문자열> [--section N] [--apply-to 0|1|2] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut text_arg: Option<&str> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) => text_arg = Some(v),
                    None => {
                        eprintln!("오류: --text 뒤에 넣을 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--apply-to" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
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
    let (Some(file_path), Some(is_header), Some(text)) = (file_path, is_header, text_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if text.is_empty() {
        eprintln!("오류: --text 는 빈 문자열일 수 없습니다.");
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
    if !dry_run {
        if let Err(e) = doc
            .insert_text_in_header_footer_native(section, is_header, apply_to, para, offset, text)
        {
            eprintln!("오류: 머리말/꼬리말 텍스트 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    let inserted_chars = text.chars().count();
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfins",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "offset": offset,
            "text": text,
            "insertedChars": inserted_chars
        }),
        &[(section, 0)],
        &format!("{kind} 텍스트 삽입 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("{kind} 텍스트 삽입 완료: {file_path}"),
    )
}

/// `edit set-header-footer-text` — 기존 머리말/꼬리말 문단 텍스트 교체.
pub(super) fn edit_set_header_footer_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-header-footer-text <파일> --header|--footer --text <문자열> [--section N] [--apply-to 0|1|2] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut text_arg: Option<&str> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) => text_arg = Some(v),
                    None => {
                        eprintln!("오류: --text 뒤에 바꿀 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--apply-to" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
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
    let (Some(file_path), Some(is_header), Some(text)) = (file_path, is_header, text_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if text.is_empty() {
        eprintln!("오류: --text 는 빈 문자열일 수 없습니다.");
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
    if !dry_run {
        let info_raw =
            match doc.get_header_footer_para_info_native(section, is_header, apply_to, para) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("오류: 머리말/꼬리말 문단 조회 실패 - {e}");
                    return EXIT_RUNTIME;
                }
            };
        let info: serde_json::Value = match serde_json::from_str(&info_raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: 머리말/꼬리말 문단 JSON 파싱 실패 - {e}");
                return EXIT_RUNTIME;
            }
        };
        let char_count = info["charCount"].as_u64().unwrap_or(0) as usize;
        if char_count > 0 {
            if let Err(e) = doc.delete_text_in_header_footer_native(
                section, is_header, apply_to, para, 0, char_count,
            ) {
                eprintln!("오류: 머리말/꼬리말 기존 텍스트 삭제 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
        if let Err(e) =
            doc.insert_text_in_header_footer_native(section, is_header, apply_to, para, 0, text)
        {
            eprintln!("오류: 머리말/꼬리말 텍스트 교체 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfset",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "text": text
        }),
        &[(section, 0)],
        &format!("{kind} 텍스트 교체 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("{kind} 텍스트 교체 완료: {file_path}"),
    )
}

/// `edit delete-hf-text` — 기존 머리말/꼬리말 문단에서 글자를 지운다.
pub(super) fn edit_delete_hf_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-hf-text <파일> --header|--footer --count <글자수> [--section N] [--apply-to 0|1|2] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut count_arg: Option<usize> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" | "--para" | "--offset" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--offset" => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) if n >= 1 => count_arg = Some(n),
                        Ok(_) => {
                            eprintln!("오류: --count 는 1 이상이어야 합니다.");
                            return EXIT_USAGE;
                        }
                        Err(_) => {
                            eprintln!("오류: --count 뒤에 1 이상의 정수가 필요합니다: {v}");
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
    let (Some(file_path), Some(is_header), Some(count)) = (file_path, is_header, count_arg) else {
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
    if !dry_run {
        if let Err(e) = doc
            .delete_text_in_header_footer_native(section, is_header, apply_to, para, offset, count)
        {
            eprintln!("오류: 머리말/꼬리말 텍스트 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfdeltxt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "offset": offset,
            "count": count
        }),
        &[(section, 0)],
        &format!(
            "{kind} 텍스트 삭제 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset} 글자 {count}"
        ),
        &format!("{kind} 텍스트 삭제 완료: {file_path}"),
    )
}

/// `edit split-paragraph-in-hf` — 기존 머리말/꼬리말 문단을 오프셋에서 나눈다.
pub(super) fn edit_split_paragraph_in_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-paragraph-in-hf <파일> --header|--footer [--section N] [--apply-to 0|1|2] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
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
    let (Some(file_path), Some(is_header)) = (file_path, is_header) else {
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
    if !dry_run {
        if let Err(e) = doc.split_paragraph_in_header_footer_native(
            section, is_header, apply_to, para, offset, None,
        ) {
            eprintln!("오류: 머리말/꼬리말 문단 분할 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfsplit",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "offset": offset
        }),
        &[(section, 0)],
        &format!("{kind} 문단 분할 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("{kind} 문단 분할 완료: {file_path}"),
    )
}

/// `edit merge-paragraph-in-hf` — 머리말/꼬리말 문단을 바로 앞 문단과 합친다.
pub(super) fn edit_merge_paragraph_in_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit merge-paragraph-in-hf <파일> --header|--footer [--section N] [--apply-to 0|1|2] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 1;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
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
    let (Some(file_path), Some(is_header)) = (file_path, is_header) else {
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
    if !dry_run {
        if let Err(e) =
            doc.merge_paragraph_in_header_footer_native(section, is_header, apply_to, para)
        {
            eprintln!("오류: 머리말/꼬리말 문단 병합 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfmerge",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para
        }),
        &[(section, 0)],
        &format!("{kind} 문단 병합 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("{kind} 문단 병합 완료: {file_path}"),
    )
}

/// `edit insert-field-in-hf` — 머리말/꼬리말 필드 삽입. 코어 `insert_field_in_hf_native`.
pub(super) fn edit_insert_field_in_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-field-in-hf <파일> --header|--footer --field-type <1|2|3> [--section N] [--apply-to 0|1|2] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut field_type: Option<u8> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--field-type" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --field-type 뒤에 1·2·3 이 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<u8>() {
                    Ok(n) if (1..=3).contains(&n) => field_type = Some(n),
                    _ => {
                        eprintln!("오류: --field-type 은 1(쪽번호)·2(총쪽수)·3(파일이름) 만 허용합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--apply-to" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
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
    let (Some(file_path), Some(is_header), Some(field_type)) = (file_path, is_header, field_type)
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
    if !dry_run {
        if let Err(e) =
            doc.insert_field_in_hf_native(section, is_header, apply_to, para, offset, field_type)
        {
            eprintln!("오류: 머리말/꼬리말 필드 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hffield",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "offset": offset,
            "fieldType": field_type
        }),
        &[(section, 0)],
        &format!("{kind} 필드 삽입 예정: {file_path} 구역 {section} type {field_type}"),
        &format!("{kind} 필드 삽입 완료: {file_path}"),
    )
}
