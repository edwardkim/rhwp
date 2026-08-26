//! SVG, render tree, 문서 구조 출력 어댑터.

use std::fs;
use std::path::Path;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use super::allows_implicit_sibling_resources;
use crate::{hu_to_mm, load_document, structure_json_value, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

struct SvgExportArgs<'a> {
    file_path: &'a str,
    output_dir: String,
    target_page: Option<u32>,
    show_para_marks: bool,
    show_control_codes: bool,
    annotate_metric_font: bool,
    debug_overlay: bool,
    grid_mm: Option<f64>,
    grid_origin: GridOriginOption,
    respect_vpos_reset: bool,
    hangul2024_compat: bool,
    font_embed_mode: rhwp::renderer::svg::FontEmbedMode,
    font_paths: Vec<std::path::PathBuf>,
    render_profile: Option<rhwp::paint::RenderProfile>,
    json_mode: bool,
}

fn parse_export_svg_args<'a>(args: &'a [String]) -> Result<SvgExportArgs<'a>, i32> {
    // [#3359] 위치 인자 파싱은 export-structure/export-text(#3349) 규약과 동일 —
    // 첫 비플래그 토큰이 파일이고 옵션은 위치 무관이다.
    let mut file_path: Option<&str> = None;
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut show_para_marks = false;
    let mut show_control_codes = false;
    let mut annotate_metric_font = false;
    let mut debug_overlay = false;
    let mut grid_mm: Option<f64> = None;
    let mut grid_origin = GridOriginOption::Fixed((0.0_f64, 0.0_f64));
    let mut respect_vpos_reset = false;
    let mut hangul2024_compat = false;
    let mut font_embed_mode = rhwp::renderer::svg::FontEmbedMode::None;
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut render_profile: Option<rhwp::paint::RenderProfile> = None;
    let mut json_mode = false;

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
            "--show-para-marks" => {
                show_para_marks = true;
                i += 1;
            }
            "--show-control-codes" => {
                show_control_codes = true;
                i += 1;
            }
            "--annotate-metric-font" => {
                annotate_metric_font = true;
                i += 1;
            }
            "--debug-overlay" => {
                debug_overlay = true;
                i += 1;
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
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
            arg if arg == "--show-grid" || arg.starts_with("--show-grid=") => {
                grid_mm = if let Some(value) = arg.strip_prefix("--show-grid=") {
                    match parse_grid_mm(value) {
                        Some(v) => Some(v),
                        None => {
                            eprintln!(
                                "오류: --show-grid 값이 올바르지 않습니다. 예: --show-grid=3mm"
                            );
                            return Err(EXIT_USAGE);
                        }
                    }
                } else {
                    Some(1.0)
                };
                i += 1;
            }
            arg if arg == "--grid-origin" || arg == "--grid-paper-origin" => {
                if i + 1 < args.len() {
                    match parse_grid_origin_option(&args[i + 1]) {
                        Some(v) => grid_origin = v,
                        None => {
                            eprintln!(
                                "오류: --grid-origin 값이 올바르지 않습니다. 예: --grid-origin=15mm,20mm 또는 --grid-origin=auto"
                            );
                            return Err(EXIT_USAGE);
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --grid-origin 뒤에 가로,세로 값이 필요합니다.");
                    return Err(EXIT_USAGE);
                }
            }
            arg if arg.starts_with("--grid-origin=") || arg.starts_with("--grid-paper-origin=") => {
                let value = arg
                    .strip_prefix("--grid-origin=")
                    .or_else(|| arg.strip_prefix("--grid-paper-origin="))
                    .unwrap_or_default();
                match parse_grid_origin_option(value) {
                    Some(v) => grid_origin = v,
                    None => {
                        eprintln!(
                            "오류: --grid-origin 값이 올바르지 않습니다. 예: --grid-origin=15mm,20mm 또는 --grid-origin=auto"
                        );
                        return Err(EXIT_USAGE);
                    }
                }
                i += 1;
            }
            "--font-style" => {
                font_embed_mode = rhwp::renderer::svg::FontEmbedMode::Style;
                i += 1;
            }
            "--embed-fonts" => {
                font_embed_mode = rhwp::renderer::svg::FontEmbedMode::Subset;
                i += 1;
            }
            "--embed-fonts=full" => {
                font_embed_mode = rhwp::renderer::svg::FontEmbedMode::Full;
                i += 1;
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
            "--json" => {
                // [#3286] 산출물 매니페스트를 stdout 에 JSON 으로 — 에이전트가
                // 어떤 파일이 생겼는지 파싱 없이 알 수 있게 한다.
                json_mode = true;
                i += 1;
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
        eprintln!("오류: 문서 파일 경로를 지정해주세요.");
        eprintln!(
            "사용법: rhwp export-svg <파일.hwp|파일.hwpx|파일.hml> [옵션] (rhwp --help 참조)"
        );
        return Err(EXIT_USAGE);
    };

    if render_profile.is_some() && font_embed_mode != rhwp::renderer::svg::FontEmbedMode::None {
        eprintln!("오류: --profile은 --font-style/--embed-fonts와 함께 사용할 수 없습니다.");
        return Err(EXIT_USAGE);
    }

    Ok(SvgExportArgs {
        file_path,
        output_dir,
        target_page,
        show_para_marks,
        show_control_codes,
        annotate_metric_font,
        debug_overlay,
        grid_mm,
        grid_origin,
        respect_vpos_reset,
        hangul2024_compat,
        font_embed_mode,
        font_paths,
        render_profile,
        json_mode,
    })
}

fn configure_svg_document(
    doc: &mut rhwp::wasm_api::HwpDocument,
    show_para_marks: bool,
    show_control_codes: bool,
    annotate_metric_font: bool,
    debug_overlay: bool,
    respect_vpos_reset: bool,
    hangul2024_compat: bool,
) {
    if show_para_marks {
        doc.set_show_paragraph_marks(true);
    }
    if show_control_codes {
        doc.set_show_control_codes(true);
    }
    if annotate_metric_font {
        doc.set_annotate_metric_font(true);
    }
    if debug_overlay {
        doc.set_debug_overlay(true);
    }
    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }
    if hangul2024_compat {
        doc.set_hangul2024_compat(true);
    }
}

pub(crate) fn export_svg(args: &[String]) -> i32 {
    let SvgExportArgs {
        file_path,
        output_dir,
        target_page,
        show_para_marks,
        show_control_codes,
        annotate_metric_font,
        debug_overlay,
        grid_mm,
        grid_origin,
        respect_vpos_reset,
        hangul2024_compat,
        font_embed_mode,
        font_paths,
        render_profile,
        json_mode,
    } = match parse_export_svg_args(args) {
        Ok(options) => options,
        Err(code) => return code,
    };

    // 파일 읽기
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };

    let source_format = rhwp::parser::detect_format(&data);

    // 문서 로드
    let mut doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    // [Task #741 후속] 외부 file path 그림 영역 영역 HWP file 영역 영역 같은 dir 영역
    // 영역 image 영역 영역 자동 load (basename 매칭).
    if allows_implicit_sibling_resources(source_format) {
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            let _loaded = doc.populate_external_images_from_dir(parent);
        }
    }

    configure_svg_document(
        &mut doc,
        show_para_marks,
        show_control_codes,
        annotate_metric_font,
        debug_overlay,
        respect_vpos_reset,
        hangul2024_compat,
    );

    let page_count = doc.page_count();
    if !json_mode {
        // stdout 순수성: --json 모드에서는 데이터(JSON)만 나간다.
        println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    }

    // 출력 폴더 생성
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

    // 페이지 범위 결정
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

    // SVG 내보내기
    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    // [#2707] 요청한 페이지 수가 아니라 실제로 저장에 성공한 페이지 수를 센다.
    let mut manifest: Vec<serde_json::Value> = Vec::new();
    let mut written = 0usize;
    // [#3668] LAYOUT_OVERFLOW_CELL 집계 — 페이지 렌더 직후 take 로 페이지 귀속.
    let mut overflow_cell_total: u64 = 0;

    for page_num in &pages {
        let svg_result = if let Some(profile) = render_profile {
            doc.render_page_svg_layer_with_profile_native(*page_num, profile)
        } else if font_embed_mode != rhwp::renderer::svg::FontEmbedMode::None {
            doc.render_page_svg_with_fonts(*page_num, font_embed_mode, &font_paths)
        } else {
            doc.render_page_svg_native(*page_num)
        };
        let page_overflow_cell_lines = doc.take_overflow_cell_lines();
        overflow_cell_total += u64::from(page_overflow_cell_lines);
        match svg_result {
            Ok(mut svg) => {
                // 격자 오버레이 삽입
                if let Some(mm) = grid_mm {
                    let origin_mm = match grid_origin {
                        GridOriginOption::Fixed(origin) => origin,
                        GridOriginOption::AutoPaper => {
                            match grid_paper_origin_mm(&doc, *page_num) {
                                Some(origin) => origin,
                                None => {
                                    eprintln!(
                                        "오류: 페이지 {}의 격자 기준 위치를 계산할 수 없습니다.",
                                        page_num
                                    );
                                    continue;
                                }
                            }
                        }
                    };
                    svg = insert_grid_overlay(&svg, mm, origin_mm);
                }
                let svg_filename = if page_count == 1 {
                    format!("{}.svg", file_stem)
                } else {
                    format!("{}_{:03}.svg", file_stem, page_num + 1)
                };
                let svg_path = output_path.join(&svg_filename);

                match fs::write(&svg_path, &svg) {
                    Ok(_) => {
                        if json_mode {
                            manifest.push(serde_json::json!({
                                "page": page_num,
                                "path": svg_path.display().to_string(),
                                "bytes": svg.len(),
                                "overflowCellLines": page_overflow_cell_lines,
                            }));
                        } else {
                            println!("  → {}", svg_path.display());
                        }
                        written += 1;
                    }
                    Err(e) => eprintln!("오류: SVG 저장 실패 - {}: {}", svg_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 렌더링 실패 - {:?}", page_num, e);
            }
        }
    }

    // 단건 JSON 명령의 실패는 stdout 을 비워야 한다. 부분 매니페스트를 출력하면
    // 소비자가 성공 결과로 오인하거나 stdout JSON을 파싱한 뒤 실패를 놓친다.
    if written != pages.len() {
        if !json_mode {
            println!("내보내기 완료: {}개 SVG 파일 → {}/", written, output_dir);
        }
        return EXIT_RUNTIME;
    }

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "format": "svg",
            "outputDir": output_dir,
            "pageCount": page_count,
            "renderedCount": written,
            "overflowCellLines": overflow_cell_total,
            "pages": manifest,
        });
        println!("{}", provenance::marked(envelope, "export-svg"));
    } else {
        println!("내보내기 완료: {}개 SVG 파일 → {}/", written, output_dir);
    }

    EXIT_OK
}

pub(crate) fn export_render_tree(args: &[String]) -> i32 {
    // [#3359] 위치 인자 파싱은 export-structure/export-text(#3349) 규약과 동일.
    let mut file_path: Option<&str> = None;
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut show_para_marks = false;
    let mut show_control_codes = false;
    let mut respect_vpos_reset = false;
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
                    return EXIT_USAGE;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return EXIT_USAGE;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "--show-para-marks" => {
                show_para_marks = true;
                i += 1;
            }
            "--show-control-codes" => {
                show_control_codes = true;
                i += 1;
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            "--compat" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: --compat 뒤에 2022 또는 2024 가 필요합니다.");
                    return EXIT_USAGE;
                }
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
                i += 1;
            }
        }
    }

    let Some(file_path) = file_path else {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-render-tree <파일.hwp> [옵션] (rhwp --help 참조)");
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let source_format = rhwp::parser::detect_format(&data);

    let mut doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    if allows_implicit_sibling_resources(source_format) {
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            let _loaded = doc.populate_external_images_from_dir(parent);
        }
    }

    if show_para_marks {
        doc.set_show_paragraph_marks(true);
    }
    if show_control_codes {
        doc.set_show_control_codes(true);
    }
    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }
    if hangul2024_compat {
        doc.set_hangul2024_compat(true);
    }

    let page_count = doc.page_count();
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

    // [#2707] 요청한 페이지 수가 아니라 실제로 저장에 성공한 페이지 수를 센다.
    let mut written = 0usize;

    for page_num in &pages {
        match doc.build_page_render_tree(*page_num) {
            Ok(tree) => {
                let json_path = output_path.join(format!("render_tree_{:03}.json", page_num + 1));
                let json = tree.root.to_json();
                match fs::write(&json_path, json) {
                    Ok(_) => {
                        println!("  → {}", json_path.display());
                        written += 1;
                    }
                    Err(e) => {
                        eprintln!(
                            "오류: render tree 저장 실패 - {}: {}",
                            json_path.display(),
                            e
                        )
                    }
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} render tree 생성 실패 - {:?}", page_num, e);
            }
        }
    }

    println!(
        "내보내기 완료: {}개 render tree JSON 파일 → {}/",
        written, output_dir
    );

    // [#2707] 한 장이라도 못 썼으면 런타임 실패다.
    if written == pages.len() {
        EXIT_OK
    } else {
        EXIT_RUNTIME
    }
}

/// `export-structure` — 문서 개요/조문 계층을 중첩 JSON 트리로 추출 (조문 DB화용).
pub(crate) fn export_structure(args: &[String]) -> i32 {
    use rhwp::document_core::queries::structure::{build_structure, StructureMode};

    let mut file_path: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut mode = StructureMode::Auto;
    // [#3261] --json: 계약 봉투(schemaVersion·source)를 씌운 한 줄 JSON.
    // 기본 출력(무봉투 pretty JSON·-o 파일 저장)은 기존 소비자 계약이라 건드리지 않는다.
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--mode" => {
                i += 1;
                match args.get(i).and_then(|s| StructureMode::parse(s)) {
                    Some(m) => mode = m,
                    None => {
                        eprintln!("오류: --mode 는 auto|outline|clause");
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
        eprintln!(
            "사용법: rhwp export-structure <파일> [--mode auto|outline|clause] [-o out.json]"
        );
        return EXIT_USAGE;
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

    let st = build_structure(doc.document(), mode);

    if json_mode {
        // [#3261] 봉투는 한 줄 — NDJSON(batch)과 같은 스키마로 단건/배치 동일 소비.
        let envelope = structure_json_value(file_path, &st);
        println!("{envelope}");
        return EXIT_OK;
    }

    let json = match serde_json::to_string_pretty(&st) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("오류: JSON 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    match out_path {
        Some(p) => match fs::write(&p, &json) {
            Ok(_) => {
                println!(
                    "구조 추출 완료: mode={} 노드={} → {}",
                    st.mode, st.node_count, p
                );
                EXIT_OK
            }
            Err(e) => {
                eprintln!("오류: 출력 쓰기 실패 - {}: {}", p, e);
                // [#2707] 출력 파일을 못 쓴 실행은 실패다.
                EXIT_RUNTIME
            }
        },
        None => {
            println!("{json}");
            EXIT_OK
        }
    }
}

fn parse_grid_mm(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    let number = trimmed
        .strip_suffix("mm")
        .or_else(|| trimmed.strip_suffix("MM"))
        .unwrap_or(trimmed)
        .trim();
    let mm = number.parse::<f64>().ok()?;
    if mm.is_finite() && mm > 0.0 {
        Some(mm)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
enum GridOriginOption {
    Fixed((f64, f64)),
    AutoPaper,
}

fn parse_grid_origin_option(value: &str) -> Option<GridOriginOption> {
    if value.eq_ignore_ascii_case("auto") {
        return Some(GridOriginOption::AutoPaper);
    }
    parse_grid_origin_mm(value).map(GridOriginOption::Fixed)
}

fn parse_grid_origin_mm(value: &str) -> Option<(f64, f64)> {
    let (x, y) = value.split_once(',')?;
    Some((parse_grid_mm(x)?, parse_grid_mm(y)?))
}

fn grid_paper_origin_mm(doc: &rhwp::wasm_api::HwpDocument, page_num: u32) -> Option<(f64, f64)> {
    let page_info = doc.get_page_info_native(page_num).ok()?;
    let page_info: serde_json::Value = serde_json::from_str(&page_info).ok()?;
    let section_idx = page_info.get("sectionIndex")?.as_u64()? as usize;
    let page_def = &doc
        .document()
        .sections
        .get(section_idx)?
        .section_def
        .page_def;
    Some((
        hu_to_mm(page_def.margin_left),
        hu_to_mm(page_def.margin_top + page_def.margin_header),
    ))
}

/// SVG에 mm 단위 점 격자 오버레이를 삽입한다.
/// export-svg 디버그용 격자는 한컴오피스의 "종이 기준 위치"를 옵션으로 맞출 수 있다.
fn insert_grid_overlay(svg: &str, grid_mm: f64, origin_mm: (f64, f64)) -> String {
    // SVG viewBox에서 크기 추출
    let (width, height) = extract_svg_dimensions(svg);
    // 96dpi: 1inch = 25.4mm, 1px = 25.4/96 = 0.2646mm.
    let grid_size = 96.0 / 25.4 * grid_mm;
    let origin_x = 96.0 / 25.4 * origin_mm.0;
    let origin_y = 96.0 / 25.4 * origin_mm.1;

    let g = format!("{:.4}", grid_size);
    let ox = format!("{:.4}", origin_x);
    let oy = format!("{:.4}", origin_y);
    let w = format!("{:.2}", width);
    let h = format!("{:.2}", height);
    let defs_part = format!(
        "<defs><pattern id=\"rhwp-grid\" x=\"{ox}\" y=\"{oy}\" width=\"{g}\" height=\"{g}\" patternUnits=\"userSpaceOnUse\"><rect x=\"0\" y=\"0\" width=\"1\" height=\"1\" fill=\"#002096\" fill-opacity=\"0.9\"/></pattern></defs>"
    );
    let grid_rect = format!("\n<rect width=\"{w}\" height=\"{h}\" fill=\"url(#rhwp-grid)\"/>");
    let grid_defs =
        format!("{defs_part}\n<rect width=\"{w}\" height=\"{h}\" fill=\"url(#rhwp-grid)\"/>\n");

    // 페이지 배경(fill="#ffffff") rect 직후에 격자를 삽입
    // 이렇게 해야 흰색 배경 위에, 본문 컨텐츠 아래에 격자가 표시됨
    let bg_pattern = "fill=\"#ffffff\"/>";
    if let Some(pos) = svg.find(bg_pattern) {
        let insert_pos = pos + bg_pattern.len();
        // defs는 SVG 시작 부분에, 격자 rect는 배경 뒤에
        // defs를 <svg> 태그 직후에 삽입
        let mut result = svg.to_string();
        // 배경 rect 뒤에 격자 rect 삽입
        result.insert_str(insert_pos, &grid_rect);
        // <svg ...>\n 직후에 defs 삽입
        if let Some(svg_end) = result.find(">\n") {
            result.insert_str(svg_end + 2, &format!("{}\n", defs_part));
        }
        result
    } else {
        // 배경 rect가 없으면 기존 방식
        if let Some(pos) = svg.find(">\n") {
            let insert_pos = pos + 2;
            format!("{}{}{}", &svg[..insert_pos], grid_defs, &svg[insert_pos..])
        } else {
            svg.to_string()
        }
    }
}

/// SVG의 width/height 속성 또는 viewBox에서 크기를 추출한다.
fn extract_svg_dimensions(svg: &str) -> (f64, f64) {
    // viewBox="0 0 W H" 패턴에서 추출
    if let Some(vb_start) = svg.find("viewBox=\"") {
        let vb = &svg[vb_start + 9..];
        if let Some(vb_end) = vb.find('"') {
            let parts: Vec<&str> = vb[..vb_end].split_whitespace().collect();
            if parts.len() == 4 {
                let w: f64 = parts[2].parse().unwrap_or(800.0);
                let h: f64 = parts[3].parse().unwrap_or(1100.0);
                return (w, h);
            }
        }
    }
    // width/height 속성에서 추출
    let w = extract_attr_f64(svg, "width").unwrap_or(800.0);
    let h = extract_attr_f64(svg, "height").unwrap_or(1100.0);
    (w, h)
}

fn extract_attr_f64(svg: &str, attr: &str) -> Option<f64> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = svg.find(&pattern) {
        let val = &svg[start + pattern.len()..];
        if let Some(end) = val.find('"') {
            return val[..end].trim_end_matches("px").parse().ok();
        }
    }
    None
}
