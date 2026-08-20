use std::fs;

use crate::{
    cli::commands::edit::runtime::finish_edit_write, load_document, EXIT_RUNTIME, EXIT_USAGE,
};

pub(super) fn edit_set_hf_picture(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-hf-picture <파일> --section N --para N --ctrl N --inner-para N --inner-ctrl N --props <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut inner_para: Option<usize> = None;
    let mut inner_ctrl: Option<usize> = None;
    let mut props: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" | "--inner-para" | "--inner-ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        "--ctrl" => ctrl = Some(n),
                        "--inner-para" => inner_para = Some(n),
                        _ => inner_ctrl = Some(n),
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
                    Some(v) if !v.is_empty() => props = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 그림 속성 JSON이 필요합니다.");
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
    let (
        Some(file_path),
        Some(section),
        Some(para),
        Some(ctrl),
        Some(inner_para),
        Some(inner_ctrl),
        Some(props),
    ) = (
        file_path, section, para, ctrl, inner_para, inner_ctrl, props,
    )
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
        if let Err(e) = doc.set_header_footer_picture_properties_native(
            section, para, ctrl, inner_para, inner_ctrl, props,
        ) {
            eprintln!("오류: 머리말/꼬리말 그림 속성 변경 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfpic",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "innerPara": inner_para,
            "innerCtrl": inner_ctrl,
            "props": props
        }),
        &[(section, para)],
        &format!(
            "머리말/꼬리말 그림 속성 변경 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl} 내부 {inner_para}/{inner_ctrl}"
        ),
        &format!("머리말/꼬리말 그림 속성 변경 완료: {file_path}"),
    )
}

pub(super) fn edit_apply_hf_template(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-hf-template <파일> --header|--footer --template <0-10> [--section N] [--apply-to 0|1|2] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut template: Option<u8> = None;
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
            "--template" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --template 뒤에 0~10 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<u8>() {
                    Ok(n) if n <= 10 => template = Some(n),
                    _ => {
                        eprintln!("오류: --template 은 0~10 만 허용합니다: {v}");
                        return EXIT_USAGE;
                    }
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
    let (Some(file_path), Some(is_header), Some(template)) = (file_path, is_header, template)
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
        if let Err(e) = doc.apply_hf_template_native(section, is_header, apply_to, template) {
            eprintln!("오류: 머리말/꼬리말 마당 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hftpl",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "templateId": template
        }),
        &[(section, 0)],
        &format!("{kind} 마당 적용 예정: {file_path} 구역 {section} template {template}"),
        &format!("{kind} 마당 적용 완료: {file_path}"),
    )
}

pub(super) fn edit_toggle_hide_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit toggle-hide-hf <파일> --header|--footer [--page N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut page: u32 = 0;
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
            "--page" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<u32>() {
                    Ok(n) => page = n,
                    Err(_) => {
                        eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다: {v}");
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
    let mut hidden = false;
    if !dry_run {
        match doc.toggle_hide_header_footer_native(page, is_header) {
            Ok(raw) => {
                hidden = serde_json::from_str::<serde_json::Value>(&raw)
                    .ok()
                    .and_then(|v| v.get("hidden").and_then(|h| h.as_bool()))
                    .unwrap_or(false);
            }
            Err(e) => {
                eprintln!("오류: 머리말/꼬리말 감추기 토글 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfhide",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "page": page,
            "isHeader": is_header,
            "hidden": hidden
        }),
        &[(0, 0)],
        &format!("{kind} 감추기 토글 예정: {file_path} 쪽 {page}"),
        &format!("{kind} 감추기 토글 완료: {file_path}"),
    )
}

pub(super) fn edit_apply_para_format_in_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-para-format-in-hf <파일> --header|--footer --props <JSON> [--section N] [--apply-to 0|1|2] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut props: Option<&str> = None;
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
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 문단 서식 JSON 이 필요합니다.");
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
    let (Some(file_path), Some(is_header), Some(props)) = (file_path, is_header, props) else {
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
            doc.apply_para_format_in_hf_native(section, is_header, apply_to, para, props)
        {
            eprintln!("오류: 머리말/꼬리말 문단 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfpfmt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "props": props
        }),
        &[(section, 0)],
        &format!("{kind} 문단 서식 적용 예정: {file_path} 구역 {section}"),
        &format!("{kind} 문단 서식 적용 완료: {file_path}"),
    )
}
