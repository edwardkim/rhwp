//! Shape lifecycle command adapters.

use std::fs;

use super::runtime::finish_edit_write;
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

pub(super) fn edit_insert_shape(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-shape <파일> --width N --height N [--section N] [--para N] [--offset N] [--x N] [--y N] [--shape rectangle] [--wrap InFrontOfText] [--treat-as-char] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut width_arg: Option<u32> = None;
    let mut height_arg: Option<u32> = None;
    let mut x_hu: u32 = 0;
    let mut y_hu: u32 = 0;
    let mut shape_type = "rectangle".to_string();
    let mut wrap = "InFrontOfText".to_string();
    let mut treat_as_char = false;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" => {
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
                        _ => offset = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--width" | "--height" | "--x" | "--y" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT).");
                    return EXIT_USAGE;
                };
                match v.parse::<u32>() {
                    Ok(n) => match name.as_str() {
                        "--width" => width_arg = Some(n),
                        "--height" => height_arg = Some(n),
                        "--x" => x_hu = n,
                        _ => y_hu = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--shape" => {
                i += 1;
                match args.get(i) {
                    Some(v) => shape_type = v.clone(),
                    None => {
                        eprintln!("오류: --shape 뒤에 도형 종류가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--wrap" => {
                i += 1;
                match args.get(i) {
                    Some(v) => wrap = v.clone(),
                    None => {
                        eprintln!("오류: --wrap 뒤에 감싸기 값이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--treat-as-char" => treat_as_char = true,
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
    let (Some(file_path), Some(width), Some(height)) = (file_path, width_arg, height_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if width == 0 && height == 0 {
        eprintln!("오류: --width 와 --height 가 모두 0입니다.");
        return EXIT_USAGE;
    }
    if shape_type.trim().is_empty() {
        eprintln!("오류: --shape 은 비어 있을 수 없습니다.");
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
        if let Err(e) = doc.create_shape_control_native(
            section,
            para,
            offset,
            width,
            height,
            x_hu,
            y_hu,
            treat_as_char,
            &wrap,
            &shape_type,
            false,
            false,
            &[],
        ) {
            eprintln!("오류: 도형 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "shape",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "width": width,
            "height": height,
            "x": x_hu,
            "y": y_hu,
        }),
        &[(section, para)],
        &format!("도형 삽입 예정: {file_path} 구역 {section} 문단 {para} {width}x{height}"),
        &format!("도형 삽입 완료: {file_path}"),
    )
}

pub(super) fn edit_delete_shape(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-shape <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
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
                        _ => ctrl = Some(n),
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
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
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
        if let Err(e) = doc.delete_shape_control_native(section, para, ctrl) {
            eprintln!("오류: 도형 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delshape",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("도형 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("도형 삭제 완료: {file_path}"),
    )
}

pub(super) fn edit_group_shapes(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit group-shapes <파일> --targets P,C;P,C [--section N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut targets: Vec<(usize, usize)> = Vec::new();
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => section = n,
                    Err(_) => {
                        eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--targets" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --targets 뒤에 para,ctrl;para,ctrl 목록이 필요합니다.");
                    return EXIT_USAGE;
                };
                match parse_shape_targets(v) {
                    Some(list) => targets.extend(list),
                    None => {
                        eprintln!("오류: --targets 형식이 아닙니다 (예: 0,1;0,2): {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--target" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --target 뒤에 para,ctrl 이 필요합니다.");
                    return EXIT_USAGE;
                };
                match parse_shape_target(v) {
                    Some(pair) => targets.push(pair),
                    None => {
                        eprintln!("오류: --target 형식이 아닙니다 (예: 0,1): {v}");
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
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if targets.len() < 2 {
        eprintln!("오류: 묶으려면 --targets 또는 --target 을 2개 이상 지정해야 합니다.");
        eprintln!("{USAGE}");
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
    let mut group_para = targets[0].0;
    let mut group_ctrl = targets[0].1;
    if !dry_run {
        match doc.group_shapes_native(section, &targets) {
            Ok(raw) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    if let Some(n) = v["paraIdx"].as_u64() {
                        group_para = n as usize;
                    }
                    if let Some(n) = v["controlIdx"].as_u64() {
                        group_ctrl = n as usize;
                    }
                }
            }
            Err(e) => {
                eprintln!("오류: 도형 묶기 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "grpshape",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": group_para,
            "ctrl": group_ctrl,
            "count": targets.len(),
        }),
        &[(section, group_para)],
        &format!(
            "도형 묶기 예정: {file_path} 구역 {section} {}개",
            targets.len()
        ),
        &format!("도형 묶기 완료: {file_path}"),
    )
}

fn parse_shape_targets(raw: &str) -> Option<Vec<(usize, usize)>> {
    let mut out = Vec::new();
    for piece in raw.split([';', '|']) {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        out.push(parse_shape_target(piece)?);
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn parse_shape_target(raw: &str) -> Option<(usize, usize)> {
    let sep = if raw.contains(',') {
        ','
    } else if raw.contains(':') {
        ':'
    } else {
        return None;
    };
    let mut parts = raw.split(sep);
    let para = parts.next()?.trim().parse::<usize>().ok()?;
    let ctrl = parts.next()?.trim().parse::<usize>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((para, ctrl))
}

/// `edit ungroup-shape` — 묶음 풀기. 코어 `ungroup_shape_native`.
pub(super) fn edit_ungroup_shape(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit ungroup-shape <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
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
                        _ => ctrl = Some(n),
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
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
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
        if let Err(e) = doc.ungroup_shape_native(section, para, ctrl) {
            eprintln!("오류: 도형 묶음 풀기 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "ungroup",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("도형 묶음 풀기 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("도형 묶음 풀기 완료: {file_path}"),
    )
}
