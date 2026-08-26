//! native-skia와 GPU 기반 PNG 출력 어댑터.

#[cfg(any(feature = "native-skia", feature = "gpu"))]
use std::fs;
#[cfg(any(feature = "native-skia", feature = "gpu"))]
use std::path::Path;

#[cfg(any(feature = "native-skia", feature = "gpu"))]
use super::allows_implicit_sibling_resources;
#[cfg(any(feature = "native-skia", feature = "gpu"))]
use crate::load_document_core;
use crate::{EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

#[cfg(not(feature = "native-skia"))]
pub(crate) fn export_png(_args: &[String]) -> i32 {
    eprintln!("오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.");
    eprintln!("       cargo build --release --features native-skia");
    // [#2707] 기능이 아예 빌드되지 않은 바이너리다. 0으로 끝내면 스크립트가 성공으로 읽는다.
    EXIT_USAGE
}

#[cfg(feature = "native-skia")]
struct PngExportArgs<'a> {
    file_path: &'a str,
    output_dir: String,
    target_page: Option<u32>,
    font_paths: Vec<std::path::PathBuf>,
    scale: Option<f64>,
    max_dimension: Option<i32>,
    vlm_target: Option<rhwp::document_core::queries::rendering::VlmTarget>,
    dpi: Option<f64>,
    render_profile: rhwp::paint::RenderProfile,
    hangul2024_compat: bool,
}

#[cfg(feature = "native-skia")]
fn parse_export_png_args<'a>(args: &'a [String]) -> Result<PngExportArgs<'a>, i32> {
    use rhwp::document_core::queries::rendering::VlmTarget;

    // [#3359] 위치 인자 파싱은 export-structure/export-text(#3349) 규약과 동일.
    let mut file_path: Option<&str> = None;
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut scale: Option<f64> = None;
    let mut max_dimension: Option<i32> = None;
    let mut vlm_target: Option<VlmTarget> = None;
    let mut dpi: Option<f64> = None;
    // PNG export is print-equivalent output. Editor visuals require an explicit screen profile.
    let mut render_profile = rhwp::paint::RenderProfile::HighQuality;
    let mut hangul2024_compat = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
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
                    let Some(profile) = rhwp::paint::RenderProfile::parse(&args[i + 1]) else {
                        eprintln!(
                            "오류: --profile 값이 올바르지 않습니다 (screen|print|high-quality|fast-preview)."
                        );
                        return Err(EXIT_USAGE);
                    };
                    render_profile = profile;
                    i += 2;
                } else {
                    eprintln!("오류: --profile 뒤에 프로필 이름이 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--font-path" => {
                if i + 1 < args.len() {
                    font_paths.push(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --font-path 뒤에 경로가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--scale" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(s) if s.is_finite() && s > 0.0 => scale = Some(s),
                        _ => {
                            eprintln!("오류: --scale 값이 올바르지 않습니다 (양수 실수 필요).");
                            return Err(EXIT_USAGE);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --scale 뒤에 배율 값이 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--max-dimension" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<i32>() {
                        Ok(n) if n > 0 => max_dimension = Some(n),
                        _ => {
                            eprintln!(
                                "오류: --max-dimension 값이 올바르지 않습니다 (양수 정수 필요)."
                            );
                            return Err(EXIT_USAGE);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --max-dimension 뒤에 픽셀 값이 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--dpi" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(d) if d.is_finite() && d > 0.0 => dpi = Some(d),
                        _ => {
                            eprintln!("오류: --dpi 값이 올바르지 않습니다 (양수 실수 필요).");
                            return Err(EXIT_USAGE);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --dpi 뒤에 DPI 값이 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--vlm-target" => {
                if i + 1 < args.len() {
                    match VlmTarget::from_str(&args[i + 1]) {
                        Some(t) => vlm_target = Some(t),
                        None => {
                            eprintln!(
                                "오류: --vlm-target 값이 올바르지 않습니다 (지원: {}).",
                                VlmTarget::all_names()
                            );
                            return Err(EXIT_USAGE);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --vlm-target 뒤에 프리셋 이름이 필요합니다.");
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
        eprintln!("사용법: rhwp export-png <파일.hwp> [옵션] (rhwp --help 참조)");
        return Err(EXIT_USAGE);
    };

    Ok(PngExportArgs {
        file_path,
        output_dir,
        target_page,
        font_paths,
        scale,
        max_dimension,
        vlm_target,
        dpi,
        render_profile,
        hangul2024_compat,
    })
}

#[cfg(feature = "native-skia")]
pub(crate) fn export_png(args: &[String]) -> i32 {
    use rhwp::document_core::queries::rendering::PngExportOptions;

    let PngExportArgs {
        file_path,
        output_dir,
        target_page,
        font_paths,
        scale,
        max_dimension,
        vlm_target,
        dpi,
        render_profile,
        hangul2024_compat,
    } = match parse_export_png_args(args) {
        Ok(options) => options,
        Err(code) => return code,
    };

    let png_options = PngExportOptions {
        scale,
        max_dimension,
        vlm_target,
        dpi,
        font_paths: font_paths.clone(),
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let mut core = match load_document_core(&data) {
        Ok(c) => c,
        Err(e) => return e.report(),
    };

    if hangul2024_compat {
        core.set_hangul2024_compat(true);
    }

    // [#3302] 외부 연결 그림(HWP3 pic_type=0 등)의 같은 디렉터리 자동 적재 — export-svg
    // 의 #741 규칙과 동일. 누락 시 skia 렌더가 회색 placeholder 를 그린다 (SO-SUEOP 1쪽 실측).
    if allows_implicit_sibling_resources(rhwp::parser::detect_format(&data)) {
        if let Some(parent) = Path::new(file_path).parent() {
            let _loaded = core.populate_external_images_from_dir(parent);
        }
    }

    let page_count = core.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

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
            if p >= page_count as u32 {
                eprintln!(
                    "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                    page_count - 1
                );
                return EXIT_USAGE;
            }
            vec![p]
        }
        None => (0..page_count as u32).collect(),
    };

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    let total_pages = pages.len();
    let mut success = 0;
    let mut total_bytes = 0usize;

    for page_num in &pages {
        let has_options = png_options.scale.is_some()
            || png_options.max_dimension.is_some()
            || png_options.vlm_target.is_some()
            || png_options.dpi.is_some()
            || render_profile != rhwp::paint::RenderProfile::Screen;
        let result = if has_options {
            core.render_page_png_native_with_profile_and_export_options(
                *page_num,
                render_profile,
                &png_options,
            )
        } else if !font_paths.is_empty() {
            core.render_page_png_native_with_fonts(*page_num, &font_paths)
        } else {
            core.render_page_png_native(*page_num)
        };
        match result {
            Ok(png_bytes) => {
                let png_filename = if total_pages == 1 {
                    format!("{}.png", file_stem)
                } else {
                    format!("{}_{:03}.png", file_stem, page_num + 1)
                };
                let png_path = output_path.join(&png_filename);
                if let Err(e) = fs::write(&png_path, &png_bytes) {
                    eprintln!("오류: 페이지 {} PNG 저장 실패 - {}", page_num + 1, e);
                    continue;
                }
                println!("  → {} ({} bytes)", png_path.display(), png_bytes.len());
                total_bytes += png_bytes.len();
                success += 1;
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 렌더링 실패 - {:?}", page_num + 1, e);
            }
        }
    }

    println!(
        "내보내기 완료: {}개 PNG 파일 → {}/ ({:.1} MB)",
        success,
        output_dir,
        total_bytes as f64 / 1024.0 / 1024.0
    );

    // [#2707] 성공 수 집계는 이미 정확했지만 종료 코드가 항상 0이었다.
    if success == total_pages {
        EXIT_OK
    } else {
        EXIT_RUNTIME
    }
}

// ============================================================================
// [gym_gpu_raster] export-png-gpu — GPU 가속 SVG→PNG 래스터화 (feature = "gpu")
//
// 파싱·레이아웃은 GPU로 가속되지 않는다(분기 지배적). 이 명령은 그 경계를 넘지 않고, 기존
// SVG 산출(render_page_svg_native)이 만든 벡터를 **픽셀로 굽는 단계만** GPU(vello/wgpu)로
// 옮긴다. 대량 문서 코퍼스를 VLM 입력 이미지로 굽는 에이전트 파이프라인이 대상이다.
// ============================================================================

/// gpu feature 없이 빌드된 바이너리 — export-png 의 native-skia 스텁과 같은 계약.
#[cfg(not(feature = "gpu"))]
pub(crate) fn export_png_gpu(_args: &[String]) -> i32 {
    eprintln!("오류: export-png-gpu 명령은 gpu feature 가 활성화되어야 합니다.");
    eprintln!("       cargo build --release --features gpu");
    // 기능이 아예 빌드되지 않은 바이너리다. 0으로 끝내면 스크립트가 성공으로 오인한다(#2707).
    EXIT_USAGE
}

#[cfg(feature = "gpu")]
struct GpuPngExportArgs<'a> {
    file_path: &'a str,
    output_dir: String,
    target_page: Option<u32>,
    scale: f64,
    font_paths: Vec<std::path::PathBuf>,
    benchmark: bool,
    repeat: u32,
}

#[cfg(feature = "gpu")]
fn parse_export_png_gpu_args<'a>(args: &'a [String]) -> Result<GpuPngExportArgs<'a>, i32> {
    let mut file_path: Option<&str> = None;
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut scale: f64 = 2.0;
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut benchmark = false;
    let mut repeat: u32 = 1;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_export_png_gpu_usage();
                return Err(EXIT_OK);
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
            "--scale" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(s) if s.is_finite() && s > 0.0 => scale = s,
                        _ => {
                            eprintln!("오류: --scale 값이 올바르지 않습니다 (양수 실수 필요).");
                            return Err(EXIT_USAGE);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --scale 뒤에 배율 값이 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--font-path" => {
                if i + 1 < args.len() {
                    font_paths.push(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --font-path 뒤에 경로가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            "--benchmark" => {
                benchmark = true;
                i += 1;
            }
            "--repeat" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) if n >= 1 => repeat = n,
                        _ => {
                            eprintln!("오류: --repeat 값이 올바르지 않습니다 (1 이상 정수 필요).");
                            return Err(EXIT_USAGE);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --repeat 뒤에 반복 횟수가 필요합니다.");
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
        eprintln!("사용법: rhwp export-png-gpu <파일.hwp|파일.hwpx> [옵션] (rhwp export-png-gpu --help 참조)");
        return Err(EXIT_USAGE);
    };

    Ok(GpuPngExportArgs {
        file_path,
        output_dir,
        target_page,
        scale,
        font_paths,
        benchmark,
        repeat,
    })
}

#[cfg(feature = "gpu")]
fn create_gpu_export_context(
    benchmark: bool,
    repeat: u32,
) -> Result<(rhwp::renderer::gpu::GpuContext, f64), i32> {
    use rhwp::renderer::gpu;
    use std::time::Instant;

    let init_start = Instant::now();
    let ctx = match gpu::GpuContext::new() {
        Ok(context) => context,
        Err(e) => {
            eprintln!("오류: GPU 컨텍스트 생성 실패 - {e}");
            eprintln!(
                "      (헤드리스 Vulkan/DX12/Metal 어댑터가 필요합니다. `rhwp gpu-info` 로 확인하세요.)"
            );
            return Err(EXIT_RUNTIME);
        }
    };
    let init_ms = init_start.elapsed().as_secs_f64() * 1000.0;
    println!("GPU 어댑터: {}", ctx.adapter_summary());
    println!("GPU 컨텍스트 초기화(일회성): {:.1} ms", init_ms);
    if benchmark {
        println!(
            "벤치마크 모드: 각 페이지 래스터화를 {}회 반복해 최솟값(노이즈 최소)을 취합니다.\n",
            repeat
        );
    }
    Ok((ctx, init_ms))
}

#[cfg(feature = "gpu")]
fn select_gpu_export_pages(target_page: Option<u32>, page_count: u32) -> Result<Vec<u32>, i32> {
    match target_page {
        Some(page) if page >= page_count => {
            eprintln!(
                "오류: 페이지 번호가 범위를 벗어났습니다 (0~{})",
                page_count - 1
            );
            Err(EXIT_USAGE)
        }
        Some(page) => Ok(vec![page]),
        None => Ok((0..page_count).collect()),
    }
}

#[cfg(feature = "gpu")]
pub(crate) fn export_png_gpu(args: &[String]) -> i32 {
    use rhwp::renderer::gpu;
    use std::time::Instant;

    let GpuPngExportArgs {
        file_path,
        output_dir,
        target_page,
        scale,
        font_paths,
        benchmark,
        repeat,
    } = match parse_export_png_gpu_args(args) {
        Ok(options) => options,
        Err(code) => return code,
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let mut core = match load_document_core(&data) {
        Ok(c) => c,
        Err(e) => return e.report(),
    };

    // 외부 연결 그림 자동 적재 — export-svg/export-png 와 동일 규칙(#3302).
    if allows_implicit_sibling_resources(rhwp::parser::detect_format(&data)) {
        if let Some(parent) = Path::new(file_path).parent() {
            let _loaded = core.populate_external_images_from_dir(parent);
        }
    }

    let page_count = core.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

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

    let pages = match select_gpu_export_pages(target_page, page_count) {
        Ok(pages) => pages,
        Err(code) => return code,
    };

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    // 배치 전체에서 재사용할 단 하나의 컨텍스트와 일회성 생성 비용.
    let (mut ctx, init_ms) = match create_gpu_export_context(benchmark, repeat) {
        Ok(context) => context,
        Err(code) => return code,
    };

    let total_pages = pages.len();
    let mut success = 0usize;
    let mut total_bytes = 0usize;

    // 벤치마크 누적기(래스터화 단계만 — 파싱·인코딩은 두 경로 공통이라 별도 집계).
    let mut sum_svg_ms = 0.0f64; // 레이아웃+SVG 생성(CPU, 두 경로 공통 입력)
    let mut sum_parse_ms = 0.0f64; // usvg 파싱+텍스트 셰이핑(두 경로 공통)
    let mut sum_gpu_ms = 0.0f64; // vello: scene 빌드+GPU 래스터+리드백
    let mut sum_cpu_ms = 0.0f64; // resvg: tiny-skia CPU 래스터
    let mut sum_encode_ms = 0.0f64; // PNG 인코딩(두 경로 공통 코드)
    let mut worst_mean_abs = 0.0f64;
    let mut worst_pct = 0.0f64;
    let mut dims_all_match = true;

    for page_num in &pages {
        // 1) 레이아웃+SVG 생성 (CPU, 두 경로 공통 입력)
        let t = Instant::now();
        let svg = match core.render_page_svg_native(*page_num) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("오류: 페이지 {} SVG 생성 실패 - {:?}", page_num + 1, e);
                continue;
            }
        };
        sum_svg_ms += t.elapsed().as_secs_f64() * 1000.0;

        // 2) usvg 파싱 (두 경로 공통 벡터 트리)
        let t = Instant::now();
        let tree = match gpu::parse_svg(&svg, &font_paths) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("오류: 페이지 {} SVG 파싱 실패 - {e}", page_num + 1);
                continue;
            }
        };
        sum_parse_ms += t.elapsed().as_secs_f64() * 1000.0;

        // 3) GPU 래스터화 (repeat 회 중 최솟값)
        let mut gpu_best = f64::INFINITY;
        let mut gpu_img = None;
        for _ in 0..repeat {
            let t = Instant::now();
            match ctx.rasterize(&tree, scale) {
                Ok(img) => {
                    let ms = t.elapsed().as_secs_f64() * 1000.0;
                    gpu_best = gpu_best.min(ms);
                    gpu_img = Some(img);
                }
                Err(e) => {
                    eprintln!("오류: 페이지 {} GPU 래스터화 실패 - {e}", page_num + 1);
                    break;
                }
            }
        }
        let Some(gpu_img) = gpu_img else { continue };
        sum_gpu_ms += gpu_best;

        // 4) PNG 인코딩 (공통 코드)
        let t = Instant::now();
        let png_bytes = match gpu_img.encode_png() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("오류: 페이지 {} PNG 인코딩 실패 - {e}", page_num + 1);
                continue;
            }
        };
        sum_encode_ms += t.elapsed().as_secs_f64() * 1000.0;

        let png_filename = if total_pages == 1 {
            format!("{}.png", file_stem)
        } else {
            format!("{}_{:03}.png", file_stem, page_num + 1)
        };
        let png_path = output_path.join(&png_filename);
        if let Err(e) = fs::write(&png_path, &png_bytes) {
            eprintln!("오류: 페이지 {} PNG 저장 실패 - {}", page_num + 1, e);
            continue;
        }
        println!(
            "  → {} ({}x{}, {} bytes, GPU {:.1} ms)",
            png_path.display(),
            gpu_img.width,
            gpu_img.height,
            png_bytes.len(),
            gpu_best
        );
        total_bytes += png_bytes.len();
        success += 1;

        // 5) 벤치마크: 같은 트리를 CPU(resvg)로도 굽고, 시간·픽셀차를 잰다.
        if benchmark {
            let mut cpu_best = f64::INFINITY;
            let mut cpu_img = None;
            for _ in 0..repeat {
                let t = Instant::now();
                match gpu::cpu_rasterize(&tree, scale) {
                    Ok(img) => {
                        cpu_best = cpu_best.min(t.elapsed().as_secs_f64() * 1000.0);
                        cpu_img = Some(img);
                    }
                    Err(e) => {
                        eprintln!("경고: 페이지 {} CPU 래스터화 실패 - {e}", page_num + 1);
                        break;
                    }
                }
            }
            if let Some(cpu_img) = cpu_img {
                sum_cpu_ms += cpu_best;
                // CPU PNG 도 저장(눈 검증용).
                if let Ok(cpu_png) = cpu_img.encode_png() {
                    let cpu_name = format!("{}_{:03}.cpu.png", file_stem, page_num + 1);
                    let _ = fs::write(output_path.join(cpu_name), &cpu_png);
                }
                let d = gpu::diff(&gpu_img, &cpu_img);
                if !d.dims_match {
                    dims_all_match = false;
                    println!(
                        "     [벤치] p{}: GPU {:.1}ms / CPU {:.1}ms · 치수 불일치 GPU {}x{} vs CPU {}x{}",
                        page_num + 1,
                        gpu_best,
                        cpu_best,
                        d.width_a,
                        d.height_a,
                        d.width_b,
                        d.height_b
                    );
                } else {
                    worst_mean_abs = worst_mean_abs.max(d.mean_abs);
                    worst_pct = worst_pct.max(d.pct_pixels_over_thresh);
                    println!(
                        "     [벤치] p{}: GPU {:.1}ms / CPU {:.1}ms (x{:.2}) · 픽셀차 평균 {:.2}/255, |Δ|≥16 {:.2}%",
                        page_num + 1,
                        gpu_best,
                        cpu_best,
                        cpu_best / gpu_best,
                        d.mean_abs,
                        d.pct_pixels_over_thresh * 100.0
                    );
                }
            }
        }
    }

    println!(
        "\n내보내기 완료: {}개 PNG → {}/ ({:.1} MB), 배율 {}x",
        success,
        output_dir,
        total_bytes as f64 / 1024.0 / 1024.0,
        scale
    );

    if benchmark && success > 0 {
        let n = success as f64;
        println!("\n==================== 정직한 벤치마크 요약 ====================");
        println!(
            "표본: {} 페이지, 배율 {}x, 반복 {}회(최솟값), 어댑터 {}",
            success,
            scale,
            repeat,
            ctx.adapter_summary()
        );
        println!("공통 단계(두 경로 동일 입력, 가속 대상 아님):");
        println!(
            "  레이아웃+SVG 생성 : 합계 {:8.1} ms  (페이지당 {:6.2} ms)",
            sum_svg_ms,
            sum_svg_ms / n
        );
        println!(
            "  usvg 파싱+셰이핑  : 합계 {:8.1} ms  (페이지당 {:6.2} ms)",
            sum_parse_ms,
            sum_parse_ms / n
        );
        println!(
            "  PNG 인코딩        : 합계 {:8.1} ms  (페이지당 {:6.2} ms)",
            sum_encode_ms,
            sum_encode_ms / n
        );
        println!("래스터화 단계(비교 대상):");
        println!(
            "  CPU (resvg/tiny-skia) : 합계 {:8.1} ms  (페이지당 {:6.2} ms)",
            sum_cpu_ms,
            sum_cpu_ms / n
        );
        println!(
            "  GPU (vello/wgpu)      : 합계 {:8.1} ms  (페이지당 {:6.2} ms)",
            sum_gpu_ms,
            sum_gpu_ms / n
        );
        if sum_gpu_ms > 0.0 {
            println!(
                "  → 래스터화만: GPU가 CPU 대비 {:.2}x",
                sum_cpu_ms / sum_gpu_ms
            );
        }
        // 엔드투엔드(초기화 포함/제외) — 소규모에서 GPU가 손해 보는 구간을 정직하게 보인다.
        let e2e_common = sum_svg_ms + sum_parse_ms + sum_encode_ms;
        let e2e_cpu = e2e_common + sum_cpu_ms;
        let e2e_gpu_no_init = e2e_common + sum_gpu_ms;
        let e2e_gpu_with_init = e2e_gpu_no_init + init_ms;
        println!("엔드투엔드(공통 단계 포함):");
        println!("  CPU 경로              : {:8.1} ms", e2e_cpu);
        println!(
            "  GPU 경로(초기화 제외) : {:8.1} ms  → {:.2}x",
            e2e_gpu_no_init,
            e2e_cpu / e2e_gpu_no_init
        );
        println!(
            "  GPU 경로(초기화 포함) : {:8.1} ms  (일회성 {:.1} ms 포함) → {:.2}x",
            e2e_gpu_with_init,
            init_ms,
            e2e_cpu / e2e_gpu_with_init
        );
        println!("시각 일치(GPU vs CPU, 같은 벡터 입력):");
        if dims_all_match {
            println!(
                "  치수 전 페이지 일치 · 최악 평균 픽셀차 {:.2}/255 · 최악 |Δ|≥16 비율 {:.2}%",
                worst_mean_abs,
                worst_pct * 100.0
            );
            println!("  (차이는 레이아웃이 아니라 두 래스터라이저의 안티에일리어싱 방식 차이다.)");
        } else {
            println!("  일부 페이지 치수 불일치 — 위 로그 참조.");
        }
        println!("=============================================================");
    }

    if success == total_pages {
        EXIT_OK
    } else {
        EXIT_RUNTIME
    }
}

#[cfg(feature = "gpu")]
fn print_export_png_gpu_usage() {
    println!("rhwp export-png-gpu <파일.hwp|파일.hwpx> [옵션]");
    println!("  기존 SVG 산출을 GPU(vello/wgpu)로 래스터화해 페이지별 PNG로 내보낸다.");
    println!("  파싱·레이아웃은 GPU 대상이 아니다(분기 지배적) — 래스터화 단계만 GPU로 옮긴다.");
    println!();
    println!("  -o, --output <폴더>   출력 폴더 (기본: output/)");
    println!("  -p, --page <번호>     특정 페이지만 (0부터)");
    println!("  --scale <배율>        렌더 배율 (기본: 2.0)");
    println!("  --font-path <경로>    폰트 파일/디렉터리 탐색 경로 (여러 번 지정 가능)");
    println!("  --benchmark           같은 벡터를 CPU(resvg)로도 굽고 시간·픽셀차를 보고");
    println!("  --repeat <N>          각 페이지 래스터화 반복 후 최솟값 (기본: 1)");
}

/// gpu feature 없이 빌드된 바이너리 — gpu-info 스텁.
#[cfg(not(feature = "gpu"))]
pub(crate) fn gpu_info(_args: &[String]) -> i32 {
    eprintln!("오류: gpu-info 명령은 gpu feature 가 활성화되어야 합니다.");
    eprintln!("       cargo build --release --features gpu");
    EXIT_USAGE
}

/// 사용 가능한 GPU 어댑터를 열거한다 — export-png-gpu 가 어떤 GPU를 쓸지 확인용.
#[cfg(feature = "gpu")]
pub(crate) fn gpu_info(_args: &[String]) -> i32 {
    use rhwp::renderer::gpu;
    let adapters = gpu::probe_adapters();
    if adapters.is_empty() {
        println!("사용 가능한 GPU 어댑터가 없습니다.");
        return EXIT_RUNTIME;
    }
    println!("사용 가능한 GPU 어댑터 ({}개):", adapters.len());
    for (idx, a) in adapters.iter().enumerate() {
        println!("  [{}] {}", idx, a);
    }
    println!();
    match gpu::GpuContext::new() {
        Ok(ctx) => {
            println!(
                "export-png-gpu 가 선택할 어댑터(HighPerformance): {}",
                ctx.adapter_summary()
            );
            EXIT_OK
        }
        Err(e) => {
            eprintln!("경고: 헤드리스 렌더 컨텍스트 생성 실패 - {e}");
            EXIT_RUNTIME
        }
    }
}
