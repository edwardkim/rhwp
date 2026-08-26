//! 본문 텍스트, 문단, 나누기 편집 명령.

use std::{fs, path::Path, process};

use super::runtime::finish_edit_write;
use crate::{
    edit_output_format, edit_serialize, edit_verify_report, load_document, provenance,
    EditOutputFormat, ENVELOPE_SCHEMA_VERSION, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};

pub(super) fn edit_insert_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-text <파일> --text <문자열> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";

    let mut file_path: Option<&str> = None;
    let mut text_arg: Option<&str> = None;
    let mut section_arg: u32 = 0;
    let mut para_arg: u32 = 0;
    let mut offset_arg: u32 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
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
            "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터).");
                    return EXIT_USAGE;
                };
                let Ok(value) = v.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터): {v}");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => section_arg = value,
                    "--para" => para_arg = value,
                    _ => offset_arg = value,
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

    let (Some(file_path), Some(text)) = (file_path, text_arg) else {
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

    let sec = section_arg as usize;
    let para = para_arg as usize;
    let offset = offset_arg as usize;
    let section_count = doc.document().sections.len();
    if sec >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section_arg}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[sec].paragraphs.len();
    if para >= para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section_arg} 문단 0~{}): {para_arg}",
            para_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_chars = doc.document().sections[sec].paragraphs[para]
        .text
        .chars()
        .count();
    if offset > para_chars {
        eprintln!("오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {para_chars}): {offset_arg}");
        return EXIT_USAGE;
    }

    if !dry_run {
        if let Err(e) = doc.insert_text_native(sec, para, offset, text) {
            eprintln!("오류: 텍스트 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }

    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_inserted.{}", stem, out_format.ext())
    });

    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if !dry_run {
        let out_bytes = match edit_serialize(&mut doc, out_format) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "오류: {} 직렬화 실패 - {}",
                    out_format.label().to_uppercase(),
                    e
                );
                return EXIT_RUNTIME;
            }
        };
        if let Err(e) = fs::write(&output_path, &out_bytes) {
            eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
            return EXIT_RUNTIME;
        }
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }

    let changed_pages = if dry_run {
        serde_json::Value::Null
    } else {
        match doc.pages_covering_paragraphs(&[(sec, para)]) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    };
    let inserted_chars = text.chars().count();

    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "section": section_arg,
            "paragraph": para_arg,
            "offset": offset_arg,
            "text": text,
            "insertedChars": inserted_chars,
            "dryRun": dry_run,
            "changedPages": changed_pages,
        });
        if !dry_run {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        // 삽입 문자열은 호출자 인자이지 문서 유래가 아니다 — 표지는 항상 싣되
        // untrustedFields 는 비운다 (키 부재 = 판정 안 함).
        println!("{}", provenance::marked(envelope, "edit"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "삽입 예정: {} 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg} ← {inserted_chars}자",
            file_path
        );
    } else {
        println!(
            "텍스트 삽입 완료: {} → {} — 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg} ← {inserted_chars}자",
            file_path, output_path
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}
/// [#4992] `edit insert-paragraph` — 지정 자리에 빈 문단을 끼운다.
pub(super) fn edit_insert_paragraph(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-paragraph <파일> [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section_arg: u32 = 0;
    let mut para_arg: u32 = 0;
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
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터).");
                    return EXIT_USAGE;
                };
                let Ok(value) = v.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터): {v}");
                    return EXIT_USAGE;
                };
                if name == "--section" {
                    section_arg = value;
                } else {
                    para_arg = value;
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
    let sec = section_arg as usize;
    let para = para_arg as usize;
    let section_count = doc.document().sections.len();
    if sec >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section_arg}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[sec].paragraphs.len();
    if para > para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section_arg} 문단 0~{para_count}): {para_arg}"
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.insert_paragraph_native(sec, para) {
            eprintln!("오류: 문단 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "paragraph",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section_arg, "paragraph": para_arg }),
        &[(sec, para)],
        &format!("문단 삽입 예정: {file_path} 구역 {section_arg} 문단 {para_arg}"),
        &format!("문단 삽입 완료: {file_path}"),
    )
}
/// [#4993] `edit insert-page-break` — 쪽 나눔 삽입.
pub(super) fn edit_insert_page_break(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-page-break <파일> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section_arg: u32 = 0;
    let mut para_arg: u32 = 0;
    let mut offset_arg: u32 = 0;
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
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터).");
                    return EXIT_USAGE;
                };
                let Ok(value) = v.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터): {v}");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => section_arg = value,
                    "--para" => para_arg = value,
                    _ => offset_arg = value,
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
    let sec = section_arg as usize;
    let para = para_arg as usize;
    let offset = offset_arg as usize;
    let section_count = doc.document().sections.len();
    if sec >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section_arg}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[sec].paragraphs.len();
    if para >= para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section_arg} 문단 0~{}): {para_arg}",
            para_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.insert_page_break_native(sec, para, offset) {
            eprintln!("오류: 쪽 나눔 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "pagebreak",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section_arg,
            "paragraph": para_arg,
            "offset": offset_arg
        }),
        &[(sec, para)],
        &format!(
            "쪽 나눔 예정: {file_path} 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg}"
        ),
        &format!("쪽 나눔 삽입 완료: {file_path}"),
    )
}
/// [#5019] `edit insert-column-break` — 단 나눔 삽입.
pub(super) fn edit_insert_column_break(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-column-break <파일> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section_arg: u32 = 0;
    let mut para_arg: u32 = 0;
    let mut offset_arg: u32 = 0;
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
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터).");
                    return EXIT_USAGE;
                };
                let Ok(value) = v.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터): {v}");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => section_arg = value,
                    "--para" => para_arg = value,
                    _ => offset_arg = value,
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
    let sec = section_arg as usize;
    let para = para_arg as usize;
    let offset = offset_arg as usize;
    let section_count = doc.document().sections.len();
    if sec >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section_arg}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[sec].paragraphs.len();
    if para >= para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section_arg} 문단 0~{}): {para_arg}",
            para_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.insert_column_break_native(sec, para, offset) {
            eprintln!("오류: 단 나눔 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "colbreak",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section_arg,
            "paragraph": para_arg,
            "offset": offset_arg
        }),
        &[(sec, para)],
        &format!(
            "단 나눔 예정: {file_path} 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg}"
        ),
        &format!("단 나눔 삽입 완료: {file_path}"),
    )
}
/// [#5120] `edit set-numbering-restart` — 문단 번호 다시 시작.
pub(super) fn edit_set_numbering_restart(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-numbering-restart <파일> --mode N [--count N] [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut mode_arg: Option<u8> = None;
    let mut start_num: u32 = 1;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--mode" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
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
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--mode" => match v.parse::<u8>() {
                        Ok(n) => mode_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --mode 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u32>() {
                        Ok(n) => start_num = n,
                        Err(_) => {
                            eprintln!("오류: --count 뒤에 0 이상의 정수가 필요합니다: {v}");
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
    let (Some(file_path), Some(mode)) = (file_path, mode_arg) else {
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
        if let Err(e) = doc.set_numbering_restart_native(section, para, mode, start_num) {
            eprintln!("오류: 번호 다시 시작 설정 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "numrst",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "count": start_num }),
        &[(section, para)],
        &format!("번호 다시 시작 예정: {file_path} 구역 {section} 문단 {para} mode {mode}"),
        &format!("번호 다시 시작 설정 완료: {file_path}"),
    )
}
/// [#5011] `edit delete-text` — 문단 좌표 텍스트 삭제.
pub(super) fn edit_delete_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-text <파일> --count <글자수> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut count_arg: Option<usize> = None;
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
                        _ => {
                            if n == 0 {
                                eprintln!("오류: --count 는 1 이상이어야 합니다.");
                                return EXIT_USAGE;
                            }
                            count_arg = Some(n);
                        }
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
    let (Some(file_path), Some(count)) = (file_path, count_arg) else {
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
        if let Err(e) = doc.delete_text_native(section, para, offset, count) {
            eprintln!("오류: 텍스트 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "deltext",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "count": count
        }),
        &[(section, para)],
        &format!(
            "텍스트 삭제 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset} 글자 {count}"
        ),
        &format!("텍스트 삭제 완료: {file_path}"),
    )
}
/// [#5012] `edit delete-paragraph` — 문단 삭제.
pub(super) fn edit_delete_paragraph(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-paragraph <파일> [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
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
                    Ok(n) => {
                        if name == "--section" {
                            section = n;
                        } else {
                            para = n;
                        }
                    }
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
    let Some(file_path) = file_path else {
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
        if let Err(e) = doc.delete_paragraph_native(section, para) {
            eprintln!("오류: 문단 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delpara",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para }),
        &[(section, para.saturating_sub(1))],
        &format!("문단 삭제 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("문단 삭제 완료: {file_path}"),
    )
}
/// [#5018] `edit merge-paragraph` — 문단 병합.
pub(super) fn edit_merge_paragraph(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit merge-paragraph <파일> [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
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
                    Ok(n) => {
                        if name == "--section" {
                            section = n;
                        } else {
                            para = n;
                        }
                    }
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
    let Some(file_path) = file_path else {
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
        if let Err(e) = doc.merge_paragraph_native(section, para) {
            eprintln!("오류: 문단 병합 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "mergepara",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para }),
        &[(section, para.saturating_sub(1))],
        &format!("문단 병합 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("문단 병합 완료: {file_path}"),
    )
}
/// [#5082] `edit split-paragraph` — 본문 문단 분할. 코어 `split_paragraph_native`.
pub(super) fn edit_split_paragraph(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-paragraph <파일> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
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
        if let Err(e) = doc.split_paragraph_native(section, para, offset, None) {
            eprintln!("오류: 문단 분할 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "splitpara",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "offset": offset }),
        &[(section, para)],
        &format!("문단 분할 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("문단 분할 완료: {file_path}"),
    )
}
