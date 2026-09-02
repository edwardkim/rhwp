//! `dump-pages` 페이지네이션 진단 조회 어댑터.

use std::fs;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{load_document, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn run(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!(
            "사용법: rhwp dump-pages <파일.hwp> [-p <페이지번호>] [--respect-vpos-reset] [--compat 2022|2024] [--json]"
        );
        return EXIT_USAGE;
    }

    let file_path = &args[0];
    let mut target_page: Option<u32> = None;
    let mut respect_vpos_reset = false;
    let mut json_mode = false;
    let mut hangul2024_compat = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(number) => target_page = Some(number),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다: {}", args[i + 1]);
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: {} 뒤에 페이지 번호가 필요합니다.", args[i]);
                    return EXIT_USAGE;
                }
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--compat" => {
                if i + 1 < args.len() {
                    match crate::cli::parse_compat_generation(args[i + 1].as_str()) {
                        Some(enabled) => hangul2024_compat = enabled,
                        None => {
                            eprintln!(
                                "오류: --compat 값이 올바르지 않습니다(2022|2024): {}",
                                args[i + 1]
                            );
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --compat 뒤에 2022 또는 2024 가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                return EXIT_USAGE;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, error);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&data) {
        Ok(doc) => doc,
        Err(error) => return error.report(),
    };

    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }
    if hangul2024_compat {
        doc.set_hangul2024_compat(true);
    }

    let page_count = doc.page_count();
    if let Some(page) = target_page {
        if page >= page_count {
            eprintln!(
                "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                page_count.saturating_sub(1)
            );
            return EXIT_USAGE;
        }
    }

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "pageCount": page_count,
            "pageFilter": target_page,
            "respectVposReset": respect_vpos_reset,
            "pages": doc.dump_page_items_json(target_page),
        });
        println!("{}", provenance::marked(envelope, "dump-pages"));
    } else {
        println!("문서 로드: {} ({}페이지)", file_path, page_count);
        print!("{}", doc.dump_page_items(target_page));
    }
    EXIT_OK
}
