//! SVG 호환 및 direct PDF 출력 어댑터.

#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use rhwp::provenance;
#[cfg(not(target_arch = "wasm32"))]
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

#[cfg(not(target_arch = "wasm32"))]
use super::allows_implicit_sibling_resources;
#[cfg(not(target_arch = "wasm32"))]
use crate::load_document;
use crate::{EXIT_RUNTIME, EXIT_USAGE};

#[cfg(not(target_arch = "wasm32"))]
struct PdfExportArgs<'a> {
    file_path: &'a str,
    output_file: String,
    target_page: Option<u32>,
    pdf_backend: rhwp::renderer::pdf::PdfBackend,
    pdf_options: rhwp::renderer::pdf::PdfExportOptions,
    direct_pdf_options: rhwp::renderer::pdf::DirectPdfExportOptions,
    render_profile: Option<rhwp::paint::RenderProfile>,
    hangul2024_compat: bool,
    json_mode: bool,
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_export_pdf_args<'a>(args: &'a [String]) -> Result<PdfExportArgs<'a>, i32> {
    // [#3359] 위치 인자 파싱은 export-structure/export-text(#3349) 규약과 동일.
    let mut file_path: Option<&str> = None;
    let mut output_file = String::new();
    let mut target_page: Option<u32> = None;
    let mut pdf_backend = rhwp::renderer::pdf::PdfBackend::default();
    let mut pdf_options = rhwp::renderer::pdf::PdfExportOptions::default();
    let mut direct_pdf_options = rhwp::renderer::pdf::DirectPdfExportOptions::default();
    let mut render_profile: Option<rhwp::paint::RenderProfile> = None;
    let mut compatibility_only_options = Vec::new();
    let mut direct_raster_dpi_was_set = false;
    // [#3596] --json: 산출물 매니페스트를 stdout 순수 JSON 으로. 렌더 동작 무변경.
    let mut json_mode = false;
    let mut hangul2024_compat = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--compat" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: --compat 뒤에 2022 또는 2024 가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
                match crate::cli::parse_compat_generation(args[i + 1].as_str()) {
                    Some(enabled) => hangul2024_compat = enabled,
                    None => {
                        eprintln!(
                            "오류: --compat 값이 올바르지 않습니다(2022|2024): {}",
                            args[i + 1]
                        );
                        return Err(EXIT_USAGE);
                    }
                }
                i += 2;
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 파일 경로가 필요합니다.");
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
            "--profile" => {
                if i + 1 < args.len() {
                    render_profile = rhwp::paint::RenderProfile::parse(&args[i + 1]);
                    if render_profile.is_none() {
                        eprintln!(
                            "오류: --profile 값이 올바르지 않습니다 (screen|print|high-quality|fast-preview)."
                        );
                        return Err(EXIT_USAGE);
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --profile 뒤에 프로필 이름이 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--backend" => {
                if i + 1 < args.len() {
                    let Some(backend) = rhwp::renderer::pdf::PdfBackend::parse(&args[i + 1]) else {
                        eprintln!("오류: --backend 값이 올바르지 않습니다 (svg|direct).");
                        return Err(EXIT_USAGE);
                    };
                    pdf_backend = backend;
                    i += 2;
                } else {
                    eprintln!("오류: --backend 뒤에 backend 이름이 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            arg if arg.starts_with("--backend=") => {
                let Some(backend) =
                    rhwp::renderer::pdf::PdfBackend::parse(arg.trim_start_matches("--backend="))
                else {
                    eprintln!("오류: --backend 값이 올바르지 않습니다 (svg|direct).");
                    return Err(EXIT_USAGE);
                };
                pdf_backend = backend;
                i += 1;
            }
            "--raster-dpi" => {
                if i + 1 < args.len() {
                    let Ok(raster_dpi) = args[i + 1].parse::<f32>() else {
                        eprintln!("오류: --raster-dpi 값은 양수여야 합니다.");
                        return Err(EXIT_USAGE);
                    };
                    if !raster_dpi.is_finite() || raster_dpi <= 0.0 {
                        eprintln!("오류: --raster-dpi 값은 양수여야 합니다.");
                        return Err(EXIT_USAGE);
                    }
                    direct_pdf_options.raster_dpi = raster_dpi;
                    direct_raster_dpi_was_set = true;
                    i += 2;
                } else {
                    eprintln!("오류: --raster-dpi 뒤에 DPI 값이 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            arg if arg.starts_with("--raster-dpi=") => {
                let Ok(raster_dpi) = arg.trim_start_matches("--raster-dpi=").parse::<f32>() else {
                    eprintln!("오류: --raster-dpi 값은 양수여야 합니다.");
                    return Err(EXIT_USAGE);
                };
                if !raster_dpi.is_finite() || raster_dpi <= 0.0 {
                    eprintln!("오류: --raster-dpi 값은 양수여야 합니다.");
                    return Err(EXIT_USAGE);
                }
                direct_pdf_options.raster_dpi = raster_dpi;
                direct_raster_dpi_was_set = true;
                i += 1;
            }
            "--font-path" => {
                if i + 1 < args.len() {
                    pdf_options
                        .font_paths
                        .push(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --font-path 뒤에 경로가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--fallback-serif" => {
                compatibility_only_options.push("--fallback-serif");
                if i + 1 < args.len() {
                    pdf_options.fallback_serif = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --fallback-serif 뒤에 폰트 family가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            arg if arg.starts_with("--fallback-serif=") => {
                compatibility_only_options.push("--fallback-serif");
                pdf_options.fallback_serif =
                    arg.trim_start_matches("--fallback-serif=").to_string();
                i += 1;
            }
            "--fallback-sans" | "--fallback-sans-serif" => {
                compatibility_only_options.push("--fallback-sans");
                if i + 1 < args.len() {
                    pdf_options.fallback_sans = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --fallback-sans 뒤에 폰트 family가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            arg if arg.starts_with("--fallback-sans=")
                || arg.starts_with("--fallback-sans-serif=") =>
            {
                compatibility_only_options.push("--fallback-sans");
                pdf_options.fallback_sans = arg
                    .strip_prefix("--fallback-sans=")
                    .or_else(|| arg.strip_prefix("--fallback-sans-serif="))
                    .unwrap_or_default()
                    .to_string();
                i += 1;
            }
            "--fallback-mono" | "--fallback-monospace" => {
                compatibility_only_options.push("--fallback-mono");
                if i + 1 < args.len() {
                    pdf_options.fallback_mono = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --fallback-mono 뒤에 폰트 family가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            arg if arg.starts_with("--fallback-mono=")
                || arg.starts_with("--fallback-monospace=") =>
            {
                compatibility_only_options.push("--fallback-mono");
                pdf_options.fallback_mono = arg
                    .strip_prefix("--fallback-mono=")
                    .or_else(|| arg.strip_prefix("--fallback-monospace="))
                    .unwrap_or_default()
                    .to_string();
                i += 1;
            }
            // [Task #2264] 텍스트를 PDF 폰트로 임베드하지 않고 path 로 변환한다.
            // 폰트 서브셋 경로를 건너뛰어 메모리를 크게 줄이는 대신,
            // PDF 의 텍스트 선택·검색 기능을 잃는다 (시각적 출력은 동일).
            "--text-as-paths" => {
                compatibility_only_options.push("--text-as-paths");
                pdf_options.embed_text = false;
                i += 1;
            }
            "--equation-font" | "--equation-font-family" => {
                compatibility_only_options.push("--equation-font");
                if i + 1 < args.len() {
                    pdf_options.equation_font = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("오류: --equation-font 뒤에 폰트 family가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            arg if arg.starts_with("--equation-font=")
                || arg.starts_with("--equation-font-family=") =>
            {
                compatibility_only_options.push("--equation-font");
                pdf_options.equation_font = Some(
                    arg.strip_prefix("--equation-font=")
                        .or_else(|| arg.strip_prefix("--equation-font-family="))
                        .unwrap_or_default()
                        .to_string(),
                );
                i += 1;
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                print_export_pdf_usage();
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
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        print_export_pdf_usage();
        return Err(EXIT_USAGE);
    };

    compatibility_only_options.sort_unstable();
    compatibility_only_options.dedup();
    if pdf_backend == rhwp::renderer::pdf::PdfBackend::DirectLayer
        && !compatibility_only_options.is_empty()
    {
        eprintln!(
            "오류: direct PDF backend는 다음 SVG 호환 옵션을 지원하지 않습니다: {}",
            compatibility_only_options.join(", ")
        );
        return Err(EXIT_USAGE);
    }
    if pdf_backend == rhwp::renderer::pdf::PdfBackend::CompatibilitySvg && direct_raster_dpi_was_set
    {
        eprintln!("오류: --raster-dpi는 direct PDF backend에서만 사용할 수 있습니다.");
        return Err(EXIT_USAGE);
    }

    // 기본 출력 파일명
    if output_file.is_empty() {
        let stem = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        output_file = format!("output/{}.pdf", stem);
    }

    Ok(PdfExportArgs {
        file_path,
        output_file,
        target_page,
        pdf_backend,
        pdf_options,
        direct_pdf_options,
        render_profile,
        hangul2024_compat,
        json_mode,
    })
}

pub(crate) fn export_pdf(args: &[String]) -> i32 {
    if args.first().is_some_and(|a| a == "--help" || a == "-h") {
        print_export_pdf_usage();
        return 0;
    }

    #[cfg(target_arch = "wasm32")]
    {
        eprintln!("오류: PDF 내보내기는 native 빌드에서만 지원됩니다.");
        return 1;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let PdfExportArgs {
            file_path,
            output_file,
            target_page,
            pdf_backend,
            pdf_options,
            mut direct_pdf_options,
            render_profile,
            hangul2024_compat,
            json_mode,
        } = match parse_export_pdf_args(args) {
            Ok(options) => options,
            Err(code) => return code,
        };

        let data = match fs::read(file_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
                return 1;
            }
        };

        let mut doc = match load_document(&data) {
            Ok(d) => d,
            Err(e) => return e.report(),
        };

        // [#3302] 외부 연결 그림 같은 디렉터리 자동 적재 — export-svg/export-png 와 동일 규칙.
        if allows_implicit_sibling_resources(rhwp::parser::detect_format(&data)) {
            if let Some(parent) = Path::new(file_path).parent() {
                let _loaded = doc.populate_external_images_from_dir(parent);
            }
        }

        // 쪽수를 읽기 전에 세션 조판 세대를 확정해야 재페이지네이션이 한 번으로 끝난다.
        if hangul2024_compat {
            doc.set_hangul2024_compat(true);
        }

        let page_count = doc.page_count();
        if !json_mode {
            println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
        }
        if page_count == 0 {
            eprintln!("오류: PDF로 내보낼 페이지가 없습니다.");
            return 1;
        }

        // 출력 디렉토리 생성
        if let Some(parent) = Path::new(&output_file).parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!("오류: 출력 디렉토리를 만들 수 없습니다 - {}", e);
                    return 1;
                }
            }
        }

        // 페이지 범위 결정
        let pages: Vec<u32> = match target_page {
            Some(p) => {
                if p >= page_count {
                    eprintln!(
                        "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                        page_count - 1
                    );
                    return 2;
                }
                vec![p]
            }
            None => (0..page_count).collect(),
        };

        let pdf_result = match pdf_backend {
            rhwp::renderer::pdf::PdfBackend::CompatibilitySvg => match render_profile {
                Some(profile) => doc.render_pages_pdf_native_with_profile_and_options(
                    &pages,
                    profile,
                    &pdf_options,
                ),
                None => doc.render_pages_pdf_native_with_options(&pages, &pdf_options),
            },
            rhwp::renderer::pdf::PdfBackend::DirectLayer => {
                #[cfg(feature = "native-skia")]
                {
                    direct_pdf_options.font_paths = pdf_options.font_paths.clone();
                    doc.render_pages_pdf_direct_native_with_profile_and_options(
                        &pages,
                        render_profile.unwrap_or(rhwp::paint::RenderProfile::Print),
                        &direct_pdf_options,
                    )
                }
                #[cfg(not(feature = "native-skia"))]
                {
                    Err(rhwp::error::HwpError::RenderError(
                        "direct PDF backend requires a build with the native-skia feature"
                            .to_string(),
                    ))
                }
            }
        };
        let pdf_bytes = match pdf_result {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("오류: PDF 변환 실패 - {}", e);
                return 1;
            }
        };
        if let Err(e) = fs::write(&output_file, &pdf_bytes) {
            eprintln!("오류: PDF 저장 실패 - {}", e);
            return 1;
        }
        if json_mode {
            let backend_name = match pdf_backend {
                rhwp::renderer::pdf::PdfBackend::CompatibilitySvg => "svg",
                rhwp::renderer::pdf::PdfBackend::DirectLayer => "direct",
            };
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "source": file_path,
                        "format": "pdf",
                        "backend": backend_name,
                        "output": output_file,
                        "bytes": pdf_bytes.len(),
                        "pageCount": page_count,
                        "renderedCount": pages.len(),
                    }),
                    "export-pdf",
                )
            );
        } else {
            println!(
                "  → {} ({}KB, {}페이지)",
                output_file,
                pdf_bytes.len() / 1024,
                pages.len()
            );
            if pdf_backend == rhwp::renderer::pdf::PdfBackend::DirectLayer {
                println!("PDF backend: direct");
            }
            println!("PDF 내보내기 완료");
        }
        0
    }
}

fn print_export_pdf_usage() {
    eprintln!("사용법: rhwp export-pdf <파일.hwp|파일.hwpx|파일.hml> [옵션]");
    eprintln!("  -o, --output <파일>       출력 PDF 파일");
    eprintln!("  -p, --page <번호>        특정 페이지만 내보내기 (0부터 시작)");
    eprintln!("      --json               산출물 매니페스트를 stdout 에 JSON 으로 출력");
    eprintln!("      --backend <svg|direct> PDF backend (기본값: svg)");
    eprintln!(
        "      --profile <프로필>   layer 출력 프로필 (screen|print|high-quality|fast-preview)"
    );
    eprintln!("      --raster-dpi <DPI>    direct backend fallback raster DPI (기본값: 144)");
    eprintln!("      --compat 2022|2024    목표 한글 조판 세대 (기본: 2022 — 2018·2020 포함)");
    eprintln!("      --font-path <경로>   폰트 파일 탐색 경로 (여러 번 지정 가능)");
    eprintln!("      --fallback-serif <명>");
    eprintln!("      --fallback-sans <명>");
    eprintln!("      --fallback-mono <명>");
    eprintln!("      --equation-font <명>");
    eprintln!("  direct backend는 native-skia feature로 빌드한 native CLI가 필요합니다.");
    eprintln!("  참고: <...>는 자리표시자이며, 실제 입력에는 꺾쇠괄호를 쓰지 않습니다.");
    eprintln!("        공백 없는 값: --font-path ./ttfs");
    eprintln!(
        "        공백 포함 값은 큰따옴표 권장: --font-path \"./My Fonts\", --fallback-sans \"Apple SD Gothic Neo\""
    );
    eprintln!("        작은따옴표는 zsh/bash/PowerShell에서 literal 값이 필요할 때만 사용합니다.");
}
