//! Text, Markdown, and LLM-ready output adapters.

use std::fs;
use std::path::Path;

use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{load_document, truncate_page_texts, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn export_text(args: &[String]) -> i32 {
    // [#3237] --json: 결과를 파일 대신 stdout JSON 으로 낸다. stdout 은 순수 JSON 이어야
    // 하므로 이 모드에서는 진행 메시지를 찍지 않는다. 위치 무관 플래그다 (info 와 동일 규약).
    let json_mode = args.iter().any(|a| a == "--json");
    let args: Vec<String> = args
        .iter()
        .filter(|a| a.as_str() != "--json")
        .cloned()
        .collect();
    // [#3349] 위치 인자 파싱을 export-structure/export-tables 규약으로 통일 —
    // 첫 비플래그 토큰이 파일이고 옵션은 위치 무관이다. 파일 선행을 강제하면
    // `-p 0 --json 파일` 에서 `-p` 가 파일로 잡혀 "알 수 없는 옵션: 0" 이 된다.
    let mut file_path: Option<&str> = None;
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    // [#3787 S7] 기본은 **무제한**이다 — 종전 호출의 산출을 조용히 줄이지 않는다.
    let mut max_chars: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                i += 1;
                match args.get(i) {
                    Some(p) => output_dir = p.clone(),
                    None => {
                        eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--max-chars" => {
                i += 1;
                match args.get(i).and_then(|v| v.parse::<usize>().ok()) {
                    Some(n) if n >= 1 => max_chars = Some(n),
                    _ => {
                        eprintln!("오류: --max-chars 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--page" | "-p" => {
                i += 1;
                match args.get(i) {
                    Some(v) => match v.parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return EXIT_USAGE;
                        }
                    },
                    None => {
                        eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
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
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-text <파일.hwp> [옵션] (rhwp --help 참조)");
        return EXIT_USAGE;
    };

    // [#3787 S7] `--max-chars` 는 **에이전트 컨텍스트**를 지키는 상한이다. 파일
    // 저장 모드에는 지킬 컨텍스트가 없고, 거기서 조용히 잘린 .txt 를 남기면 절단
    // 사실을 실을 봉투조차 없다. 아무 일도 안 하는 플래그는 함정이므로 거부한다.
    if max_chars.is_some() && !json_mode {
        eprintln!(
            "오류: --max-chars 는 --json 과 함께 써야 합니다 (봉투에 절단 사실을 싣는 옵션)."
        );
        return EXIT_USAGE;
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let page_count = doc.page_count();
    if !json_mode {
        println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    }
    if page_count == 0 {
        eprintln!("오류: 문서에 페이지가 없습니다.");
        return EXIT_RUNTIME;
    }

    let output_path = Path::new(&output_dir);
    if !json_mode && !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return EXIT_RUNTIME;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return EXIT_USAGE;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    // [#3237] JSON 모드: 파일을 쓰지 않고 요청 페이지 전체를 stdout JSON 하나로 낸다.
    if json_mode {
        let mut extracted = Vec::with_capacity(pages.len());
        for page_num in &pages {
            match doc.extract_page_text_native(*page_num) {
                Ok(text) => extracted.push((*page_num, text)),
                Err(e) => {
                    eprintln!("오류: 페이지 {} 텍스트 추출 실패 - {}", page_num, e);
                    return EXIT_RUNTIME;
                }
            }
        }
        // [#3787 S7] 총량을 보고하려면 전수 추출이 불가피하다 — `--max-chars` 의 목적은
        // 추출 시간이 아니라 **출력 컨텍스트** 절약이므로 추출 후 표시만 절단한다
        // (`search --limit` 이 전수 grep 후 절단하는 것과 같은 이유, #3353).
        let (page_objs, omitted_count) = truncate_page_texts(&extracted, max_chars);
        let result = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "pageCount": page_objs.len(),
            "truncated": omitted_count > 0,
            "omittedCount": omitted_count,
            "pages": page_objs,
        });
        println!("{}", result);
        return EXIT_OK;
    }

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    // [#2707] 요청한 페이지 수가 아니라 실제로 저장에 성공한 페이지 수를 센다.
    let mut written = 0usize;

    for page_num in &pages {
        match doc.extract_page_text_native(*page_num) {
            Ok(mut text) => {
                if !text.ends_with('\n') {
                    text.push('\n');
                }

                let txt_filename = if page_count == 1 {
                    format!("{}.txt", file_stem)
                } else {
                    format!("{}_{:03}.txt", file_stem, page_num + 1)
                };
                let txt_path = output_path.join(&txt_filename);

                match fs::write(&txt_path, text.as_bytes()) {
                    Ok(_) => {
                        println!("  → {}", txt_path.display());
                        written += 1;
                    }
                    Err(e) => eprintln!("오류: TXT 저장 실패 - {}: {}", txt_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 텍스트 추출 실패 - {:?}", page_num, e);
            }
        }
    }

    println!(
        "텍스트 내보내기 완료: {}개 TXT 파일 → {}/",
        written, output_dir
    );

    // [#2707] 한 장이라도 못 썼으면 런타임 실패다.
    if written == pages.len() {
        EXIT_OK
    } else {
        EXIT_RUNTIME
    }
}

/// `table-to-csv` — 본문 최상위 표를 RFC 4180 CSV 로 내보낸다 (#3719 §6).
///
/// `export-tables` 의 격자 JSON 은 병합을 span 으로 보존하지만 표 계산기는 직사각
/// 격자만 먹는다. 앵커 셀을 그대로 이어 붙이면 병합 행에서 열이 밀리므로,
/// `table_csv::grid_to_csv` 가 격자를 채워서(덮인 칸 = 빈 문자열) 낸다.
struct ExportMarkdownArgs<'a> {
    file_path: &'a str,
    output_dir: String,
    target_page: Option<u32>,
    json_mode: bool,
}

fn parse_export_markdown_args(args: &[String]) -> Result<ExportMarkdownArgs<'_>, i32> {
    let mut file_path: Option<&str> = None;
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return Err(EXIT_USAGE);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return Err(EXIT_USAGE);
                }
                i += 1;
            }
        }
    }

    let Some(file_path) = file_path else {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-markdown <파일.hwp> [옵션] (rhwp --help 참조)");
        return Err(EXIT_USAGE);
    };

    Ok(ExportMarkdownArgs {
        file_path,
        output_dir,
        target_page,
        json_mode,
    })
}

fn markdown_bin_data_image(
    doc: &rhwp::wasm_api::HwpDocument,
    page_num: u32,
    bin_data_id: u16,
) -> Option<(String, Vec<u8>)> {
    let mime = match doc.get_bin_data_image_mime_native(bin_data_id) {
        Ok(mime) => mime,
        Err(e) => {
            eprintln!(
                "경고: 페이지 {} 이미지 MIME fallback 실패 (bin={}): {:?}",
                page_num, bin_data_id, e
            );
            return None;
        }
    };
    let data = match doc.get_bin_data_image_data_native(bin_data_id) {
        Ok(data) => data,
        Err(e) => {
            eprintln!(
                "경고: 페이지 {} 이미지 데이터 fallback 실패 (bin={}): {:?}",
                page_num, bin_data_id, e
            );
            return None;
        }
    };
    Some((mime, data))
}

fn markdown_image_data(
    doc: &rhwp::wasm_api::HwpDocument,
    page_num: u32,
    sec_idx: Option<usize>,
    para_idx: Option<usize>,
    control_idx: Option<usize>,
    bin_data_id: u16,
) -> Option<(String, Vec<u8>)> {
    if let (Some(si), Some(pi), Some(ci)) = (sec_idx, para_idx, control_idx) {
        if let (Ok(mime), Ok(data)) = (
            doc.get_control_image_mime_native(si, pi, &[], ci),
            doc.get_control_image_data_native(si, pi, &[], ci),
        ) {
            return Some((mime, data));
        }
        if bin_data_id == 0 {
            eprintln!(
                "경고: 페이지 {} 이미지 추출 실패 (s{} p{} c{}), fallback bin_data_id 없음",
                page_num, si, pi, ci
            );
            return None;
        }
    } else if bin_data_id == 0 {
        eprintln!(
            "경고: 페이지 {} 이미지 추출 실패 (문서 좌표 없음, bin_data_id=0)",
            page_num
        );
        return None;
    }

    markdown_bin_data_image(doc, page_num, bin_data_id)
}

pub(crate) fn export_markdown(args: &[String]) -> i32 {
    // [#3359] 위치 인자 파싱은 export-structure/export-text(#3349) 규약과 동일.
    // [#3596] --json: 산출물 매니페스트를 stdout 순수 JSON 으로. 추출 동작 무변경.
    let ExportMarkdownArgs {
        file_path,
        output_dir,
        target_page,
        json_mode,
    } = match parse_export_markdown_args(args) {
        Ok(parsed) => parsed,
        Err(code) => return code,
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let page_count = doc.page_count();
    if !json_mode {
        println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    }
    if page_count == 0 {
        eprintln!("오류: 문서에 페이지가 없습니다.");
        return EXIT_RUNTIME;
    }

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!(
                "오류: 출력 폴더를 생성할 수 없습니다 - {}: {}",
                output_dir, e
            );
            return EXIT_RUNTIME;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return EXIT_USAGE;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    let assets_dir_name = format!("{}_assets", file_stem);
    let assets_dir_path = output_path.join(&assets_dir_name);
    let mut written_image_count: usize = 0;
    // [#2707] 요청한 페이지 수가 아니라 실제로 저장에 성공한 MD 페이지 수를 센다.
    // 이미지 실패는 경고로 남기고 MD 자체는 저장되므로 페이지 실패로 세지 않는다.
    let mut written_page_count = 0usize;
    // [#3596] --json 매니페스트용 페이지별 산출물 기록.
    let mut manifest: Vec<serde_json::Value> = Vec::new();

    let mime_to_ext = |mime: &str| -> &'static str {
        match mime {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/gif" => "gif",
            "image/bmp" => "bmp",
            "image/webp" => "webp",
            _ => "bin",
        }
    };

    for page_num in &pages {
        match doc.extract_page_markdown_with_images_native(*page_num) {
            Ok((mut markdown, image_refs)) => {
                for (img_idx, (sec_idx, para_idx, control_idx, bin_data_id)) in
                    image_refs.iter().enumerate()
                {
                    let token = format!("[[RHWP_IMAGE:{}]]", img_idx + 1);

                    let Some((mime, image_data)) = markdown_image_data(
                        &doc,
                        *page_num,
                        *sec_idx,
                        *para_idx,
                        *control_idx,
                        *bin_data_id,
                    ) else {
                        markdown = markdown.replace(&token, "");
                        continue;
                    };

                    if !assets_dir_path.exists() {
                        if let Err(e) = fs::create_dir_all(&assets_dir_path) {
                            eprintln!(
                                "오류: 이미지 출력 폴더 생성 실패 - {}: {}",
                                assets_dir_path.display(),
                                e
                            );
                            markdown = markdown.replace(&token, "");
                            continue;
                        }
                    }

                    let ext = mime_to_ext(&mime);
                    let image_filename = format!(
                        "{}_p{:03}_img{:03}.{}",
                        file_stem,
                        page_num + 1,
                        img_idx + 1,
                        ext
                    );
                    let image_path = assets_dir_path.join(&image_filename);

                    if let Err(e) = fs::write(&image_path, &image_data) {
                        eprintln!("경고: 이미지 저장 실패 - {}: {}", image_path.display(), e);
                        markdown = markdown.replace(&token, "");
                        continue;
                    }

                    let image_link = format!(
                        "![image {}]({}/{})",
                        img_idx + 1,
                        assets_dir_name,
                        image_filename
                    );
                    markdown = markdown.replace(&token, &image_link);
                    written_image_count += 1;
                }

                if !markdown.ends_with('\n') {
                    markdown.push('\n');
                }

                let md_filename = if page_count == 1 {
                    format!("{}.md", file_stem)
                } else {
                    format!("{}_{:03}.md", file_stem, page_num + 1)
                };
                let md_path = output_path.join(&md_filename);

                match fs::write(&md_path, markdown.as_bytes()) {
                    Ok(_) => {
                        if json_mode {
                            manifest.push(serde_json::json!({
                                "page": page_num,
                                "path": md_path.display().to_string(),
                                "bytes": markdown.len(),
                            }));
                        } else {
                            println!("  → {}", md_path.display());
                        }
                        written_page_count += 1;
                    }
                    Err(e) => eprintln!("오류: Markdown 저장 실패 - {}: {}", md_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} Markdown 생성 실패 - {:?}", page_num, e);
            }
        }
    }

    // [#2707] 한 장이라도 못 썼으면 런타임 실패다. [#3596] JSON 모드의 실패는
    // stdout 을 비워 부분 매니페스트를 성공으로 오인하지 않게 한다(export-svg 규약).
    if written_page_count != pages.len() {
        if !json_mode {
            println!(
                "Markdown 내보내기 완료: {}개 MD 파일 → {}/",
                written_page_count, output_dir
            );
        }
        return EXIT_RUNTIME;
    }

    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                "source": file_path,
                "format": "markdown",
                "outputDir": output_dir,
                "pageCount": page_count,
                "renderedCount": written_page_count,
                "imageCount": written_image_count,
                "pages": manifest,
            })
        );
    } else if written_image_count > 0 {
        println!(
            "Markdown 내보내기 완료: {}개 MD 파일, {}개 이미지 → {}/",
            written_page_count, written_image_count, output_dir
        );
    } else {
        println!(
            "Markdown 내보내기 완료: {}개 MD 파일 → {}/",
            written_page_count, output_dir
        );
    }

    EXIT_OK
}
