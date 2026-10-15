//! native-skia 기반 PNG 출력 어댑터.

#[cfg(feature = "native-skia")]
use std::fs;
#[cfg(feature = "native-skia")]
use std::path::Path;

#[cfg(feature = "native-skia")]
use super::allows_implicit_sibling_resources;
#[cfg(feature = "native-skia")]
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
    target_pages: Vec<u32>,
    font_paths: Vec<std::path::PathBuf>,
    scale: Option<f64>,
    max_dimension: Option<i32>,
    vlm_target: Option<rhwp::document_core::queries::rendering::VlmTarget>,
    dpi: Option<f64>,
    render_profile: rhwp::paint::RenderProfile,
    hangul2024_compat: bool,
    json: bool,
    metadata_only: bool,
    render_tree_output: Option<String>,
}

#[cfg(feature = "native-skia")]
fn parse_export_png_args<'a>(args: &'a [String]) -> Result<PngExportArgs<'a>, i32> {
    use rhwp::document_core::queries::rendering::VlmTarget;

    // [#3359] 위치 인자 파싱은 export-structure/export-text(#3349) 규약과 동일.
    let mut file_path: Option<&str> = None;
    let mut output_dir = "output".to_string();
    let mut target_pages: Vec<u32> = Vec::new();
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut scale: Option<f64> = None;
    let mut max_dimension: Option<i32> = None;
    let mut vlm_target: Option<VlmTarget> = None;
    let mut dpi: Option<f64> = None;
    // 기본 PNG는 Studio Canvas와 같은 screen 레이어 트리를 재생한다. 인쇄용
    // 고품질 출력은 명시적인 `--profile high-quality`로 선택한다.
    let mut render_profile = rhwp::paint::RenderProfile::Screen;
    let mut hangul2024_compat = false;
    let mut json = false;
    let mut metadata_only = false;
    let mut render_tree_output = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--render-tree-output" => {
                let Some(directory) = args.get(i + 1) else {
                    eprintln!("오류: --render-tree-output 뒤에 폴더 경로가 필요합니다.");
                    return Err(EXIT_USAGE);
                };
                render_tree_output = Some(directory.clone());
                i += 2;
            }
            "--metadata-only" => {
                metadata_only = true;
                json = true;
                i += 1;
            }
            "--json" => {
                json = true;
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
                        Ok(n) if !target_pages.contains(&n) => target_pages.push(n),
                        Ok(_) => {
                            eprintln!("오류: 중복 페이지 번호입니다.");
                            return Err(EXIT_USAGE);
                        }
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
        target_pages,
        font_paths,
        scale,
        max_dimension,
        vlm_target,
        dpi,
        render_profile,
        hangul2024_compat,
        json,
        metadata_only,
        render_tree_output,
    })
}

#[cfg(feature = "native-skia")]
pub(crate) fn export_png(args: &[String]) -> i32 {
    use rhwp::document_core::queries::rendering::PngExportOptions;

    let (filtered_args, font_environment) = match super::font_environment_args(args) {
        Ok(options) => options,
        Err(code) => return code,
    };
    let PngExportArgs {
        file_path,
        output_dir,
        target_pages,
        font_paths,
        scale,
        max_dimension,
        vlm_target,
        dpi,
        render_profile,
        hangul2024_compat,
        json,
        metadata_only,
        render_tree_output,
    } = match parse_export_png_args(&filtered_args) {
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
    if let Err(e) = core.set_font_environment(font_environment) {
        eprintln!("오류: 폰트 환경을 적용할 수 없습니다: {e}");
        return EXIT_USAGE;
    }

    // [#3302] 외부 연결 그림(HWP3 pic_type=0 등)의 같은 디렉터리 자동 적재 — export-svg
    // 의 #741 규칙과 동일. 누락 시 skia 렌더가 회색 placeholder 를 그린다 (SO-SUEOP 1쪽 실측).
    if allows_implicit_sibling_resources(rhwp::parser::detect_format(&data)) {
        if let Some(parent) = Path::new(file_path).parent() {
            let _loaded = core.populate_external_images_from_dir(parent);
        }
    }

    let page_count = core.page_count();
    if !json {
        println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
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

    if target_pages.iter().any(|&page| page >= page_count as u32) {
        eprintln!("오류: 페이지 번호가 범위를 벗어났습니다 (전체 {page_count}쪽).");
        return EXIT_USAGE;
    }
    let pages: Vec<u32> = if target_pages.is_empty() {
        (0..page_count as u32).collect()
    } else {
        target_pages
    };
    if metadata_only {
        println!(
            "{}",
            rhwp::provenance::marked(
                serde_json::json!({
                    "schemaVersion": rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION,
                    "tool": "export-png",
                    "backend": "native-skia",
                    "source": file_path,
                    "pageCount": page_count,
                    "renderedCount": 0,
                    "metadataOnly": true,
                    "pages": [],
                }),
                "export-png"
            )
        );
        return EXIT_OK;
    }

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");
    if let Some(directory) = &render_tree_output {
        if let Err(error) = fs::create_dir_all(directory) {
            eprintln!("오류: render tree 출력 폴더 생성 실패: {error}");
            return EXIT_RUNTIME;
        }
    }

    let total_pages = pages.len();
    let mut success = 0;
    let mut total_bytes = 0usize;
    let mut exported_pages = Vec::new();
    let has_options = png_options.scale.is_some()
        || png_options.max_dimension.is_some()
        || png_options.vlm_target.is_some()
        || png_options.dpi.is_some()
        || render_profile != rhwp::paint::RenderProfile::Screen;
    let exporter = if has_options || !font_paths.is_empty() {
        rhwp::document_core::queries::png_export::NativePngExporter::new(
            render_profile,
            png_options,
        )
    } else {
        rhwp::document_core::queries::png_export::NativePngExporter::system_defaults()
    };

    for page_num in &pages {
        let result = exporter.render_page(&core, *page_num);
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
                let tree_path = if let Some(directory) = &render_tree_output {
                    let path =
                        Path::new(directory).join(format!("render_tree_{:03}.json", page_num + 1));
                    let result = core
                        .build_page_render_tree(*page_num)
                        .map(|tree| tree.root.to_json())
                        .map_err(|error| error.to_string())
                        .and_then(|tree| fs::write(&path, tree).map_err(|error| error.to_string()));
                    if let Err(error) = result {
                        eprintln!(
                            "오류: 페이지 {} render tree 저장 실패: {error}",
                            page_num + 1
                        );
                        continue;
                    }
                    Some(path)
                } else {
                    None
                };
                if !json {
                    println!("  → {} ({} bytes)", png_path.display(), png_bytes.len());
                }
                exported_pages.push(serde_json::json!({
                    "page": page_num,
                    "path": png_path,
                    "bytes": png_bytes.len(),
                    "renderTreePath": tree_path,
                }));
                total_bytes += png_bytes.len();
                success += 1;
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 렌더링 실패 - {:?}", page_num + 1, e);
            }
        }
    }

    if json {
        let profile = match render_profile {
            rhwp::paint::RenderProfile::Screen => "screen",
            rhwp::paint::RenderProfile::Print => "print",
            rhwp::paint::RenderProfile::HighQuality => "high-quality",
            rhwp::paint::RenderProfile::FastPreview => "fast-preview",
        };
        println!(
            "{}",
            rhwp::provenance::marked(
                serde_json::json!({
                    "schemaVersion": rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION,
                    "tool": "export-png",
                    "source": file_path,
                    "backend": "native-skia",
                    "comparisonProfile": profile,
                    "layoutGeneration": if hangul2024_compat { "2024" } else { "2022" },
                    "outputDir": output_dir,
                    "pageCount": page_count,
                    "renderedCount": success,
                    "pages": exported_pages,
                }),
                "export-png"
            )
        );
    } else {
        println!(
            "내보내기 완료: {}개 PNG 파일 → {}/ ({:.1} MB)",
            success,
            output_dir,
            total_bytes as f64 / 1024.0 / 1024.0
        );
    }

    // [#2707] 성공 수 집계는 이미 정확했지만 종료 코드가 항상 0이었다.
    if success == total_pages {
        EXIT_OK
    } else {
        EXIT_RUNTIME
    }
}
