use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--help") | Some("-h") => print_help(),
        Some("--version") | Some("-V") => println!("rhwp v{}", rhwp::version()),
        Some("export-svg") => export_svg(&args[2..]),
        Some("export-png") => export_png(&args[2..]),
        Some("export-pdf") => export_pdf(&args[2..]),
        Some("export-text") => export_text(&args[2..]),
        Some("export-markdown") => export_markdown(&args[2..]),
        Some("info") => show_info(&args[2..]),
        Some("dump") => dump_controls(&args[2..]),
        Some("dump-pages") => dump_pages(&args[2..]),
        Some("diag") => diag_document(&args[2..]),
        Some("convert") => convert_hwp(&args[2..]),
        Some("build-from-ingest") => build_from_ingest(&args[2..]),
        Some("dump-records") => dump_raw_records(&args[2..]),
        Some("test-shape") => test_shape_roundtrip(&args[2..]),
        Some("test-caption") => test_caption(&args[2..]),
        Some("gen-table") => gen_table(&args[2..]),
        Some("gen-pua") => gen_pua_test(&args[2..]),
        Some("test-field") => test_field_roundtrip(&args[2..]),
        Some("ir-diff") => ir_diff(&args[2..]),
        Some("thumbnail") => extract_thumbnail(&args[2..]),
        _ => {
            println!("rhwp v{}", rhwp::version());
            println!("사용법: rhwp <명령> [옵션]");
            println!("'rhwp --help'로 자세한 사용법을 확인하세요.");
        }
    }
}

fn print_help() {
    println!("rhwp v{} - HWP 파일 뷰어", rhwp::version());
    println!();
    println!("사용법: rhwp <명령> [옵션]");
    println!();
    println!("명령:");
    println!("  export-svg <파일.hwp> [옵션]");
    println!("      HWP 파일을 SVG로 내보내기");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!("      --show-para-marks       문단부호(↵/↓) 표시");
    println!("      --show-control-codes    조판부호 보이기 (문단부호 + 개체 마커 등)");
    println!("      --debug-overlay         디버그 오버레이 (문단/표 경계 + 인덱스 라벨)");
    println!("      --respect-vpos-reset    LINE_SEG vpos=0 리셋을 단/페이지 강제 경계로 처리");
    println!("      --show-grid             1mm 격자 오버레이 (레이아웃 디버깅용)");
    println!("      --font-style            @font-face local() 참조 삽입 (폰트 데이터 미포함)");
    println!("      --embed-fonts           폰트 서브셋 임베딩 (사용 글자만 base64)");
    println!("      --embed-fonts=full      폰트 전체 임베딩 (base64)");
    println!("      --font-path <경로>      폰트 파일 탐색 경로 (여러 번 지정 가능)");
    println!();
    println!("  export-png <파일.hwp> [옵션]   (native-skia feature 필요)");
    println!("      HWP 파일을 PNG로 내보내기 (Skia raster backend, AI 파이프라인 + VLM 연동)");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!("      --font-path <경로>      폰트 파일 탐색 경로 (여러 번 지정 가능)");
    println!("                              한컴 전용 폰트 (HY견명조 등) 가 시스템에 없을 때 ttfs 디렉토리 지정");
    println!("      --scale <배율>          렌더링 배율 (기본: 1.0)");
    println!("      --max-dimension <픽셀>  한 변 최대 픽셀 (longest edge). VLM 입력 한도용.");
    println!("                              명시 --scale 이 없으면 자동 scale 계산 (페이지 → 한도 안)");
    println!("      --vlm-target <프리셋>   VLM 입력 프리셋 (현재: claude)");
    println!("                              claude: 1568 px / 1.15 MP (Claude Vision 정합)");
    println!("                              다른 VLM (gpt4v/gemini/qwen-vl/llava) 은 이슈 #613 후속");
    println!();
    println!("  export-text <파일.hwp> [옵션]");
    println!("      페이지별 텍스트를 TXT로 내보내기");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!();
    println!("  export-markdown <파일.hwp> [옵션]");
    println!("      페이지별 텍스트를 Markdown(.md)으로 내보내기");
    println!();
    println!("      -o, --output <폴더>     출력 폴더 (기본: output/)");
    println!("      -p, --page <번호>       특정 페이지만 내보내기 (0부터 시작)");
    println!("  info <파일.hwp>");
    println!("      HWP 파일 정보 표시");
    println!();
    println!("  dump <파일.hwp> [--section <번호>] [--para <번호>]");
    println!("      문서 조판부호 구조 덤프 (디버깅용)");
    println!();
    println!("  dump-pages <파일.hwp> [-p <번호>] [--respect-vpos-reset]");
    println!("      페이지네이션 결과 덤프 (페이지별 문단/표 배치 목록)");
    println!();
    println!("  diag <파일.hwp>");
    println!("      문서 구조 진단 (번호/글머리표/개요 분석)");
    println!();
    println!("  convert <입력.hwp> <출력.hwp>");
    println!("      배포용(읽기전용) HWP를 편집 가능한 HWP로 변환");
    println!();
    println!("  ir-diff <파일A.hwpx> <파일B.hwp> [-s <구역>] [-p <문단>]");
    println!("      두 파일의 IR(중간표현) 비교 (HWPX↔HWP 불일치 검출)");
    println!("      비교 항목: text, char_count, char_offsets, char_shapes, line_segs,");
    println!("                 controls(타입+속성), tab_extended, ParaShape, TabDef");
    println!("      표: page_break, outer_margin, treat_as_char, wrap, size, v_offset/h_offset");
    println!("      그림/도형: treat_as_char, wrap, size, v_offset/h_offset, vert_rel/horz_rel");
    println!();
    println!("  thumbnail <파일.hwp> [옵션]");
    println!("      HWP 파일에서 썸네일(PrvImage) 추출");
    println!();
    println!("      -o, --output <파일>       출력 파일 경로 (기본: 입력명_thumb.png)");
    println!("      --base64                  base64 문자열을 stdout에 출력");
    println!("      --data-uri                data:image/... URI 형식으로 stdout에 출력");
    println!();
    println!("옵션:");
    println!("  -h, --help      도움말 표시");
    println!("  -V, --version   버전 표시");
}

fn export_svg(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-svg <파일.hwp> [옵션] (rhwp --help 참조)");
        return;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut show_para_marks = false;
    let mut show_control_codes = false;
    let mut debug_overlay = false;
    let mut show_grid = false;
    let mut respect_vpos_reset = false;
    let mut font_embed_mode = rhwp::renderer::svg::FontEmbedMode::None;
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
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
            "--debug-overlay" => {
                debug_overlay = true;
                i += 1;
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            "--show-grid" => {
                show_grid = true;
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
                    return;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                i += 1;
            }
        }
    }

    // 파일 읽기
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    // 문서 로드
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    if show_para_marks {
        doc.set_show_paragraph_marks(true);
    }
    if show_control_codes {
        doc.set_show_control_codes(true);
    }
    if debug_overlay {
        doc.set_debug_overlay(true);
    }
    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

    // 출력 폴더 생성
    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!("오류: 출력 폴더를 생성할 수 없습니다 - {}: {}", output_dir, e);
            return;
        }
    }

    // 페이지 범위 결정
    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!("오류: 페이지 번호가 범위를 벗어났습니다 (0~{})", page_count - 1);
                return;
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

    for page_num in &pages {
        let svg_result = if font_embed_mode != rhwp::renderer::svg::FontEmbedMode::None {
            doc.render_page_svg_with_fonts(*page_num, font_embed_mode, &font_paths)
        } else {
            doc.render_page_svg_native(*page_num)
        };
        match svg_result {
            Ok(mut svg) => {
                // 격자 오버레이 삽입
                if show_grid {
                    svg = insert_grid_overlay(&svg);
                }
                let svg_filename = if page_count == 1 {
                    format!("{}.svg", file_stem)
                } else {
                    format!("{}_{:03}.svg", file_stem, page_num + 1)
                };
                let svg_path = output_path.join(&svg_filename);

                match fs::write(&svg_path, &svg) {
                    Ok(_) => println!("  → {}", svg_path.display()),
                    Err(e) => eprintln!("오류: SVG 저장 실패 - {}: {}", svg_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 렌더링 실패 - {:?}", page_num, e);
            }
        }
    }

    println!("내보내기 완료: {}개 SVG 파일 → {}/", pages.len(), output_dir);
}

/// SVG에 1mm 격자 오버레이를 삽입한다.
/// `<svg ...>` 태그 직후에 격자 패턴 정의와 배경 rect를 추가.
fn insert_grid_overlay(svg: &str) -> String {
    // SVG viewBox에서 크기 추출
    let (width, height) = extract_svg_dimensions(svg);
    // 1mm = 96dpi 기준 3.7795px
    let grid_px = 96.0 * 25.4 / 96.0; // 1mm in px at 96dpi... 실제로 SVG 좌표는 px
    // 96dpi: 1inch = 25.4mm, 1px = 25.4/96 = 0.2646mm, 1mm = 96/25.4 = 3.7795px
    let grid_size = 96.0 / 25.4; // 3.7795 px per mm

    let g = format!("{:.4}", grid_size);
    let w = format!("{:.2}", width);
    let h = format!("{:.2}", height);
    let grid_defs = format!(
        "<defs><pattern id=\"rhwp-grid\" width=\"{g}\" height=\"{g}\" patternUnits=\"userSpaceOnUse\"><path d=\"M {g} 0 L 0 0 0 {g}\" fill=\"none\" stroke=\"#CCCCCC\" stroke-width=\"0.3\"/></pattern></defs>\n<rect width=\"{w}\" height=\"{h}\" fill=\"url(#rhwp-grid)\"/>\n"
    );

    // 페이지 배경(fill="#ffffff") rect 직후에 격자를 삽입
    // 이렇게 해야 흰색 배경 위에, 본문 컨텐츠 아래에 격자가 표시됨
    let bg_pattern = "fill=\"#ffffff\"/>";
    if let Some(pos) = svg.find(bg_pattern) {
        let insert_pos = pos + bg_pattern.len();
        // defs는 SVG 시작 부분에, 격자 rect는 배경 뒤에
        let defs_part = format!(
            "<defs><pattern id=\"rhwp-grid\" width=\"{g}\" height=\"{g}\" patternUnits=\"userSpaceOnUse\"><path d=\"M {g} 0 L 0 0 0 {g}\" fill=\"none\" stroke=\"#CCCCCC\" stroke-width=\"0.3\"/></pattern></defs>"
        );
        let grid_rect = format!(
            "\n<rect width=\"{w}\" height=\"{h}\" fill=\"url(#rhwp-grid)\"/>"
        );
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

#[cfg(not(feature = "native-skia"))]
fn export_png(_args: &[String]) {
    eprintln!("오류: export-png 명령은 native-skia feature 가 활성화되어야 합니다.");
    eprintln!("       cargo build --release --features native-skia");
}

#[cfg(feature = "native-skia")]
fn export_png(args: &[String]) {
    use rhwp::document_core::queries::rendering::{PngExportOptions, VlmTarget};

    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-png <파일.hwp> [옵션] (rhwp --help 참조)");
        return;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;
    let mut font_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut scale: Option<f64> = None;
    let mut max_dimension: Option<i32> = None;
    let mut vlm_target: Option<VlmTarget> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
                }
            }
            "--font-path" => {
                if i + 1 < args.len() {
                    font_paths.push(std::path::PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("오류: --font-path 뒤에 경로가 필요합니다.");
                    return;
                }
            }
            "--scale" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(s) if s.is_finite() && s > 0.0 => scale = Some(s),
                        _ => {
                            eprintln!("오류: --scale 값이 올바르지 않습니다 (양수 실수 필요).");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --scale 뒤에 배율 값이 필요합니다.");
                    return;
                }
            }
            "--max-dimension" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<i32>() {
                        Ok(n) if n > 0 => max_dimension = Some(n),
                        _ => {
                            eprintln!("오류: --max-dimension 값이 올바르지 않습니다 (양수 정수 필요).");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --max-dimension 뒤에 픽셀 값이 필요합니다.");
                    return;
                }
            }
            "--vlm-target" => {
                if i + 1 < args.len() {
                    match VlmTarget::from_str(&args[i + 1]) {
                        Some(t) => vlm_target = Some(t),
                        None => {
                            eprintln!("오류: --vlm-target 값이 올바르지 않습니다 (지원: claude).");
                            eprintln!("       다른 VLM 프리셋 (gpt4v / gemini / qwen-vl / llava) 은 이슈 #613 후속 task.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --vlm-target 뒤에 프리셋 이름이 필요합니다.");
                    return;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                i += 1;
            }
        }
    }

    let png_options = PngExportOptions {
        scale,
        max_dimension,
        vlm_target,
        font_paths: font_paths.clone(),
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let core = match rhwp::document_core::DocumentCore::from_bytes(&data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {:?}", e);
            return;
        }
    };

    let page_count = core.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!("오류: 출력 폴더를 생성할 수 없습니다 - {}: {}", output_dir, e);
            return;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count as u32 {
                eprintln!("오류: 페이지 번호가 범위를 벗어났습니다 (0~{})", page_count - 1);
                return;
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
            || png_options.vlm_target.is_some();
        let result = if has_options {
            core.render_page_png_native_with_export_options(*page_num, &png_options)
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
                println!(
                    "  → {} ({} bytes)",
                    png_path.display(),
                    png_bytes.len()
                );
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
}

fn export_pdf(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-pdf <파일.hwp> [-o 출력.pdf] [-p 페이지]");
        return;
    }

    let file_path = &args[0];
    let mut output_file = String::new();
    let mut target_page: Option<u32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 파일 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
                }
            }
            _ => { i += 1; }
        }
    }

    // 기본 출력 파일명
    if output_file.is_empty() {
        let stem = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        output_file = format!("output/{}.pdf", stem);
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);

    // 출력 디렉토리 생성
    if let Some(parent) = Path::new(&output_file).parent() {
        if !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }

    // 페이지 범위 결정
    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!("오류: 페이지 번호가 범위를 벗어났습니다 (0~{})", page_count - 1);
                return;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    // SVG 렌더링 → PDF 변환
    let mut svg_pages: Vec<String> = Vec::new();
    for page_num in &pages {
        match doc.render_page_svg(*page_num) {
            Ok(svg) => svg_pages.push(svg),
            Err(e) => {
                eprintln!("오류: 페이지 {} 렌더링 실패 - {:?}", page_num, e);
                return;
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use rhwp::renderer::pdf;
        match pdf::svgs_to_pdf(&svg_pages) {
            Ok(pdf_bytes) => {
                match fs::write(&output_file, &pdf_bytes) {
                    Ok(_) => println!("  → {} ({}KB, {}페이지)", output_file, pdf_bytes.len() / 1024, svg_pages.len()),
                    Err(e) => eprintln!("오류: PDF 저장 실패 - {}", e),
                }
            }
            Err(e) => eprintln!("오류: PDF 변환 실패 - {}", e),
        }
    }

    println!("PDF 내보내기 완료");
}

fn export_text(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-text <파일.hwp> [옵션] (rhwp --help 참조)");
        return;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                i += 1;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    if page_count == 0 {
        eprintln!("오류: 문서에 페이지가 없습니다.");
        return;
    }

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!("오류: 출력 폴더를 생성할 수 없습니다 - {}: {}", output_dir, e);
            return;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!("오류: 페이지 번호가 범위를 벗어났습니다 (0~{})", page_count - 1);
                return;
            }
            vec![p]
        }
        None => (0..page_count).collect(),
    };

    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

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
                    Ok(_) => println!("  → {}", txt_path.display()),
                    Err(e) => eprintln!("오류: TXT 저장 실패 - {}: {}", txt_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} 텍스트 추출 실패 - {:?}", page_num, e);
            }
        }
    }

    println!("텍스트 내보내기 완료: {}개 TXT 파일 → {}/", pages.len(), output_dir);
}

fn export_markdown(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp export-markdown <파일.hwp> [옵션] (rhwp --help 참조)");
        return;
    }

    let file_path = &args[0];
    let mut output_dir = "output".to_string();
    let mut target_page: Option<u32> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("오류: --output 뒤에 폴더 경로가 필요합니다.");
                    return;
                }
            }
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u32>() {
                        Ok(n) => target_page = Some(n),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호가 올바르지 않습니다.");
                            return;
                        }
                    }
                    i += 2;
                } else {
                    eprintln!("오류: --page 뒤에 페이지 번호가 필요합니다.");
                    return;
                }
            }
            _ => {
                eprintln!("알 수 없는 옵션: {}", args[i]);
                i += 1;
            }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let page_count = doc.page_count();
    println!("문서 로드 완료: {} ({}페이지)", file_path, page_count);
    if page_count == 0 {
        eprintln!("오류: 문서에 페이지가 없습니다.");
        return;
    }

    let output_path = Path::new(&output_dir);
    if !output_path.exists() {
        if let Err(e) = fs::create_dir_all(output_path) {
            eprintln!("오류: 출력 폴더를 생성할 수 없습니다 - {}: {}", output_dir, e);
            return;
        }
    }

    let pages: Vec<u32> = match target_page {
        Some(p) => {
            if p >= page_count {
                eprintln!("오류: 페이지 번호가 범위를 벗어났습니다 (0~{})", page_count - 1);
                return;
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

                    let try_control = match (sec_idx, para_idx, control_idx) {
                        (Some(si), Some(pi), Some(ci)) => Some((*si, *pi, *ci)),
                        _ => None,
                    };

                    let (mime, image_data) = if let Some((si, pi, ci)) = try_control {
                        match (
                            doc.get_control_image_mime_native(si, pi, ci),
                            doc.get_control_image_data_native(si, pi, ci),
                        ) {
                            (Ok(m), Ok(d)) => (m, d),
                            _ => {
                                if *bin_data_id == 0 {
                                    eprintln!(
                                        "경고: 페이지 {} 이미지 추출 실패 (s{} p{} c{}), fallback bin_data_id 없음",
                                        page_num, si, pi, ci
                                    );
                                    markdown = markdown.replace(&token, "");
                                    continue;
                                }
                                let fb_mime = match doc.get_bin_data_image_mime_native(*bin_data_id) {
                                    Ok(m) => m,
                                    Err(e) => {
                                        eprintln!(
                                            "경고: 페이지 {} 이미지 MIME fallback 실패 (bin={}): {:?}",
                                            page_num, bin_data_id, e
                                        );
                                        markdown = markdown.replace(&token, "");
                                        continue;
                                    }
                                };
                                let fb_data = match doc.get_bin_data_image_data_native(*bin_data_id) {
                                    Ok(d) => d,
                                    Err(e) => {
                                        eprintln!(
                                            "경고: 페이지 {} 이미지 데이터 fallback 실패 (bin={}): {:?}",
                                            page_num, bin_data_id, e
                                        );
                                        markdown = markdown.replace(&token, "");
                                        continue;
                                    }
                                };
                                (fb_mime, fb_data)
                            }
                        }
                    } else {
                        if *bin_data_id == 0 {
                            eprintln!(
                                "경고: 페이지 {} 이미지 추출 실패 (문서 좌표 없음, bin_data_id=0)",
                                page_num
                            );
                            markdown = markdown.replace(&token, "");
                            continue;
                        }
                        let fb_mime = match doc.get_bin_data_image_mime_native(*bin_data_id) {
                            Ok(m) => m,
                            Err(e) => {
                                eprintln!(
                                    "경고: 페이지 {} 이미지 MIME fallback 실패 (bin={}): {:?}",
                                    page_num, bin_data_id, e
                                );
                                markdown = markdown.replace(&token, "");
                                continue;
                            }
                        };
                        let fb_data = match doc.get_bin_data_image_data_native(*bin_data_id) {
                            Ok(d) => d,
                            Err(e) => {
                                eprintln!(
                                    "경고: 페이지 {} 이미지 데이터 fallback 실패 (bin={}): {:?}",
                                    page_num, bin_data_id, e
                                );
                                markdown = markdown.replace(&token, "");
                                continue;
                            }
                        };
                        (fb_mime, fb_data)
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
                        eprintln!(
                            "경고: 이미지 저장 실패 - {}: {}",
                            image_path.display(),
                            e
                        );
                        markdown = markdown.replace(&token, "");
                        continue;
                    }

                    let image_link = format!("![image {}]({}/{})", img_idx + 1, assets_dir_name, image_filename);
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
                    Ok(_) => println!("  → {}", md_path.display()),
                    Err(e) => eprintln!("오류: Markdown 저장 실패 - {}: {}", md_path.display(), e),
                }
            }
            Err(e) => {
                eprintln!("오류: 페이지 {} Markdown 생성 실패 - {:?}", page_num, e);
            }
        }
    }

    if written_image_count > 0 {
        println!(
            "Markdown 내보내기 완료: {}개 MD 파일, {}개 이미지 → {}/",
            pages.len(),
            written_image_count,
            output_dir
        );
    } else {
        println!("Markdown 내보내기 완료: {}개 MD 파일 → {}/", pages.len(), output_dir);
    }
}

fn show_info(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        return;
    }

    let file_path = &args[0];

    // 파일 읽기
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let file_size = data.len();

    // HWP 파싱
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let document = doc.document();

    println!("파일: {}", file_path);
    println!("크기: {} bytes", file_size);
    println!(
        "버전: {}.{}.{}.{}",
        document.header.version.major,
        document.header.version.minor,
        document.header.version.build,
        document.header.version.revision,
    );
    println!("압축: {}", if document.header.compressed { "예" } else { "아니오" });
    println!("암호화: {}", if document.header.encrypted { "예" } else { "아니오" });
    println!("배포용: {}", if document.header.distribution { "예" } else { "아니오" });
    println!("구역 수: {}", document.sections.len());
    println!("페이지 수: {}", doc.page_count());

    // 용지 정보
    for (sec_idx, section) in document.sections.iter().enumerate() {
        let page_def = &section.section_def.page_def;
        let orientation = if page_def.landscape { "가로" } else { "세로" };
        println!("구역{} 용지: {}×{} HWPUNIT, 방향={} (여백: 좌{} 우{} 상{} 하{})",
            sec_idx,
            page_def.width, page_def.height, orientation,
            page_def.margin_left, page_def.margin_right,
            page_def.margin_top, page_def.margin_bottom,
        );
        println!("  머리말여백={} 꼬리말여백={} 제본여백={}",
            page_def.margin_header, page_def.margin_footer,
            page_def.margin_gutter);
        if section.section_def.hide_empty_line {
            println!("  빈 줄 감추기: 활성");
        }
    }

    // 폰트 목록
    let lang_names = ["한글", "영어", "한자", "일어", "기타", "기호", "사용자"];
    for (i, fonts) in document.doc_info.font_faces.iter().enumerate() {
        if !fonts.is_empty() {
            let name = if i < lang_names.len() { lang_names[i] } else { "기타" };
            let font_names: Vec<&str> = fonts.iter().map(|f| f.name.as_str()).collect();
            println!("폰트({}): {}", name, font_names.join(", "));
        }
    }

    // 스타일 목록
    if !document.doc_info.styles.is_empty() {
        let style_names: Vec<&str> = document.doc_info.styles.iter().map(|s| s.local_name.as_str()).collect();
        println!("스타일: {}", style_names.join(", "));
    }

    // 문단 통계
    let total_paras: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
    println!("총 문단 수: {}", total_paras);

    // [Task #554] HWP3 → HWP5 변환본 식별 휴리스틱 정보
    // 한컴이 HWP3 → HWP5 변환 시 ParaShape/CharShape 를 거의 재사용하지 않고 매우 적은
    // 수만 생성한다. 직접 작성본은 작성자가 다양한 스타일을 사용하므로 비율이 paragraph
    // 와 비슷하거나 더 높다. 임계값 < 0.05 / < 0.15 로 27 fixture 100% 분류 (Stage 1).
    let ps_count = document.doc_info.para_shapes.len();
    let cs_count = document.doc_info.char_shapes.len();
    if total_paras > 0 {
        let ps_ratio = ps_count as f64 / total_paras as f64;
        let cs_ratio = cs_count as f64 / total_paras as f64;
        let origin = if total_paras > 50 && ps_ratio < 0.05 && cs_ratio < 0.15 {
            "HWP3 변환본 추정 (margin_bottom -1600 HU 보정 적용)"
        } else if total_paras <= 50 {
            "판정 불가 (문단 수 ≤ 50, 비율 왜곡 회피)"
        } else {
            "한컴 한글 직접 작성 추정"
        };
        println!("ParaShape: {} (PS/문단 = {:.3})", ps_count, ps_ratio);
        println!("CharShape: {} (CS/문단 = {:.3})", cs_count, cs_ratio);
        println!("Origin 추정: {}", origin);
    }

    // BinData 정보
    if !document.doc_info.bin_data_list.is_empty() {
        println!("BinData:");
        for (idx, bd) in document.doc_info.bin_data_list.iter().enumerate() {
            let type_str = match bd.data_type {
                rhwp::model::bin_data::BinDataType::Link => "Link",
                rhwp::model::bin_data::BinDataType::Embedding => "Embedding",
                rhwp::model::bin_data::BinDataType::Storage => "Storage",
            };
            let ext = bd.extension.as_deref().unwrap_or("?");
            // 로드된 데이터 크기 확인
            let loaded_size = document.bin_data_content
                .iter()
                .find(|c| c.id == bd.storage_id)
                .map(|c| c.data.len())
                .unwrap_or(0);
            println!("  [{}] {} (ID: {}, ext: {}, loaded: {} bytes)", idx, type_str, bd.storage_id, ext, loaded_size);
        }
    }

    // 테이블 및 그림 정보
    use rhwp::model::control::Control;
    let mut table_idx = 0;
    let mut picture_idx = 0;

    fn count_pictures(ctrl: &Control, picture_idx: &mut usize, location: &str) {
        match ctrl {
            Control::Picture(pic) => {
                *picture_idx += 1;
                println!(
                    "그림{} [{}]: bin_data_id={}, size={}×{}",
                    *picture_idx, location,
                    pic.image_attr.bin_data_id,
                    pic.common.width, pic.common.height,
                );
            }
            Control::Table(table) => {
                // 표 내부 셀의 문단에서도 그림 검색
                for (cell_idx, cell) in table.cells.iter().enumerate() {
                    for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                        for cc in &cp.controls {
                            let loc = format!("{}→셀{}:문단{}", location, cell_idx, cp_idx);
                            count_pictures(cc, picture_idx, &loc);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for (sec_idx, section) in document.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            for ctrl in &para.controls {
                let location = format!("구역{}:문단{}", sec_idx, para_idx);
                match ctrl {
                    Control::Table(table) => {
                        table_idx += 1;
                        let page_break_str = match table.page_break {
                            rhwp::model::table::TablePageBreak::None => "나누지 않음",
                            rhwp::model::table::TablePageBreak::CellBreak => "셀 단위 나눔",
                            rhwp::model::table::TablePageBreak::RowBreak => "나눔(행 단위)",
                        };
                        println!(
                            "표{} [{}]: {}행×{}열, 셀 {}개, 쪽나눔={} (attr=0x{:08x}), 제목반복={}",
                            table_idx, location,
                            table.row_count, table.col_count, table.cells.len(),
                            page_break_str, table.raw_table_record_attr, table.repeat_header,
                        );
                        count_pictures(ctrl, &mut picture_idx, &location);
                    }
                    Control::Picture(_) => {
                        count_pictures(ctrl, &mut picture_idx, &location);
                    }
                    Control::Shape(shape) => {
                        use rhwp::model::shape::ShapeObject;
                        let s = shape.as_ref();
                        let shape_type = s.shape_name();
                        let common = s.common();
                        let border_info = match shape.as_ref() {
                            ShapeObject::Rectangle(r) => format!(
                                ", border(color={:#010x}, width={}, attr={:#010x})",
                                r.drawing.border_line.color,
                                r.drawing.border_line.width,
                                r.drawing.border_line.attr,
                            ),
                            ShapeObject::Line(l) => format!(
                                ", border(color={:#010x}, width={}, attr={:#010x})",
                                l.drawing.border_line.color,
                                l.drawing.border_line.width,
                                l.drawing.border_line.attr,
                            ),
                            _ => String::new(),
                        };
                        println!(
                            "도형 [{}]: {}, size={}×{}, treat_as_char={}{}",
                            location, shape_type,
                            common.width, common.height,
                            common.treat_as_char,
                            border_info,
                        );
                        // 그룹 자식 상세 정보
                        if let ShapeObject::Group(g) = shape.as_ref() {
                            for (i, child) in g.children.iter().enumerate() {
                                let ctype = child.shape_name();
                                let cattr = child.shape_attr();
                                let eff_w = (cattr.current_width as f64 * cattr.render_sx) as i32;
                                let eff_h = (cattr.current_height as f64 * cattr.render_sy) as i32;
                                println!("  자식[{}]: {}, orig={}×{}, scale=({:.3},{:.3}), eff={}×{} at ({:.0},{:.0})",
                                    i, ctype,
                                    cattr.current_width, cattr.current_height,
                                    cattr.render_sx, cattr.render_sy,
                                    eff_w, eff_h,
                                    cattr.render_tx, cattr.render_ty);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// HWPUNIT(u32)을 mm로 변환
fn hu_to_mm(hu: u32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

/// HWPUNIT(i32)을 mm로 변환
fn hu_to_mm_i(hu: i32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

fn dump_pages(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-pages <파일.hwp> [-p <페이지번호>]");
        return;
    }

    let file_path = &args[0];
    let mut target_page: Option<u32> = None;
    let mut respect_vpos_reset = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--page" | "-p" => {
                if i + 1 < args.len() {
                    target_page = args[i + 1].parse().ok();
                    i += 2;
                } else { i += 1; }
            }
            "--respect-vpos-reset" => {
                respect_vpos_reset = true;
                i += 1;
            }
            _ => { i += 1; }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    if respect_vpos_reset {
        doc.set_respect_vpos_reset(true);
    }

    println!("문서 로드: {} ({}페이지)", file_path, doc.page_count());
    print!("{}", doc.dump_page_items(target_page));
}

fn dump_controls(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp dump <파일.hwp> [--section <번호>] [--para <번호>]");
        return;
    }

    let file_path = &args[0];
    let mut filter_section: Option<usize> = None;
    let mut filter_para: Option<usize> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "-s" => {
                if i + 1 < args.len() {
                    filter_section = args[i + 1].parse().ok();
                    i += 2;
                } else { i += 1; }
            }
            "--para" | "-p" => {
                if i + 1 < args.len() {
                    filter_para = args[i + 1].parse().ok();
                    i += 2;
                } else { i += 1; }
            }
            _ => { i += 1; }
        }
    }

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let document = doc.document();

    // border_fill 상세 덤프 (필터 없을 때 전체, 필터 있을 때 관련 bf만)
    if filter_section.is_none() && filter_para.is_none() {
        for (i, bf) in document.doc_info.border_fills.iter().enumerate() {
            let fill = &bf.fill;
            let solid_info = fill.solid.as_ref().map(|s| format!("bg=#{:06X} pat_type={} pat_color=#{:06X}", s.background_color, s.pattern_type, s.pattern_color)).unwrap_or_default();
            let grad_info = if fill.gradient.is_some() { " gradient" } else { "" };
            let img_info = fill.image.as_ref().map(|img| format!(" image(bin_id={}, mode={:?})", img.bin_data_id, img.fill_mode)).unwrap_or_default();
            println!("  border_fill[{}] fill_type={:?} {}{}{}", i, fill.fill_type, solid_info, grad_info, img_info);
        }
    }

    use rhwp::model::control::Control;
    use rhwp::model::shape::{ShapeObject, VertRelTo, HorzRelTo, TextWrap};
    use rhwp::model::paragraph::ColumnBreakType;

    let vert_str = |v: &VertRelTo| -> &str {
        match v {
            VertRelTo::Paper => "용지",
            VertRelTo::Page => "쪽",
            VertRelTo::Para => "문단",
        }
    };
    let horz_str = |h: &HorzRelTo| -> &str {
        match h {
            HorzRelTo::Paper => "용지",
            HorzRelTo::Page => "쪽",
            HorzRelTo::Column => "단",
            HorzRelTo::Para => "문단",
        }
    };
    let wrap_str = |w: &TextWrap| -> &str {
        match w {
            TextWrap::Square => "어울림",
            TextWrap::Tight => "자리차지",
            TextWrap::Through => "글뒤로",
            TextWrap::TopAndBottom => "위아래",
            TextWrap::BehindText => "글뒤로",
            TextWrap::InFrontOfText => "글앞으로",
        }
    };
    let break_str = |b: &ColumnBreakType| -> &str {
        match b {
            ColumnBreakType::None => "",
            ColumnBreakType::Section => "[구역나누기]",
            ColumnBreakType::MultiColumn => "[다단나누기]",
            ColumnBreakType::Page => "[쪽나누기]",
            ColumnBreakType::Column => "[단나누기]",
        }
    };

    // 도형 공통 속성 출력 헬퍼
    let dump_common = |c: &rhwp::model::shape::CommonObjAttr, indent: &str| {
        println!("{}  크기: {:.1}mm × {:.1}mm ({}×{} HU)",
            indent, hu_to_mm(c.width), hu_to_mm(c.height), c.width, c.height);
        println!("{}  위치: 가로={} 오프셋={:.1}mm({}) 정렬={:?}, 세로={} 오프셋={:.1}mm({}) 정렬={:?}",
            indent, horz_str(&c.horz_rel_to),
            hu_to_mm(c.horizontal_offset), c.horizontal_offset, c.horz_align,
            vert_str(&c.vert_rel_to),
            hu_to_mm(c.vertical_offset), c.vertical_offset, c.vert_align);
        println!("{}  배치: {}, 글자처럼={}, z={}",
            indent, wrap_str(&c.text_wrap), c.treat_as_char, c.z_order);
    };

    // 도형 요소 속성 출력 헬퍼
    let dump_shape_attr = |sa: &rhwp::model::shape::ShapeComponentAttr, indent: &str| {
        let eff_w = (sa.current_width as f64 * sa.render_sx) as u32;
        let eff_h = (sa.current_height as f64 * sa.render_sy) as u32;
        println!("{}  요소: orig={}×{}, curr={}×{}, M=[{:.3},{:.3},{:.0}; {:.3},{:.3},{:.0}], offset=({},{}), eff={:.1}mm×{:.1}mm",
            indent, sa.original_width, sa.original_height,
            sa.current_width, sa.current_height,
            sa.render_sx, sa.render_b, sa.render_tx,
            sa.render_c, sa.render_sy, sa.render_ty,
            sa.offset_x, sa.offset_y,
            hu_to_mm(eff_w), hu_to_mm(eff_h));
        if sa.horz_flip || sa.vert_flip || sa.rotation_angle != 0 {
            println!("{}  변환: 뒤집기=({},{}), 회전={}",
                indent, sa.horz_flip, sa.vert_flip, sa.rotation_angle);
        }
    };

    // 재귀적 도형 덤프
    fn dump_shape(
        shape: &ShapeObject, indent: &str,
        dump_common_fn: &dyn Fn(&rhwp::model::shape::CommonObjAttr, &str),
        dump_sa_fn: &dyn Fn(&rhwp::model::shape::ShapeComponentAttr, &str),
    ) {
        match shape {
            ShapeObject::Line(s) => {
                println!("{}[직선] start=({},{}) end=({},{})",
                    indent, s.start.x, s.start.y, s.end.x, s.end.y);
                println!("{}  선: color={:#010x}, width={}, style={:#06x}",
                    indent, s.drawing.border_line.color, s.drawing.border_line.width, s.drawing.border_line.attr);
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Rectangle(s) => {
                println!("{}[사각형] round={}%", indent, s.round_rate);
                println!("{}  선: color={:#010x}, width={}, style={:#06x}",
                    indent, s.drawing.border_line.color, s.drawing.border_line.width, s.drawing.border_line.attr);
                println!("{}  채우기: {:?}{}", indent, s.drawing.fill.fill_type,
                    if let Some(ref img) = s.drawing.fill.image { format!(", image=bin_data_id={}, mode={:?}", img.bin_data_id, img.fill_mode) } else { String::new() });
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
                if let Some(tb) = &s.drawing.text_box {
                    println!("{}  글상자: list_attr={:#010x}, margins=({},{},{},{}), max_width={}, paras={}",
                        indent, tb.list_attr, tb.margin_left, tb.margin_right, tb.margin_top, tb.margin_bottom,
                        tb.max_width, tb.paragraphs.len());
                    for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                        let text_preview = if tp.text.is_empty() {
                            "(빈)".to_string()
                        } else if tp.text.chars().count() > 60 {
                            let end = tp.text.char_indices().nth(60).map(|(i,_)|i).unwrap_or(tp.text.len());
                            format!("\"{}...\"", &tp.text[..end])
                        } else {
                            format!("\"{}\"", tp.text)
                        };
                        println!("{}    p[{}]: ps_id={}, cc={}, text={}, ls_count={}, ctrls={}",
                            indent, tpi, tp.para_shape_id, tp.char_count, text_preview,
                            tp.line_segs.len(), tp.controls.len());
                        for (li, ls) in tp.line_segs.iter().enumerate() {
                            println!("{}      ls[{}]: vpos={}, lh={}, th={}, bl={}, cs={}, sw={}",
                                indent, li, ls.vertical_pos, ls.line_height, ls.text_height,
                                ls.baseline_distance, ls.column_start, ls.segment_width);
                        }
                    }
                }
            }
            ShapeObject::Ellipse(s) => {
                println!("{}[타원]", indent);
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Arc(s) => {
                println!("{}[호]", indent);
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Polygon(s) => {
                println!("{}[다각형] points={}", indent, s.points.len());
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
                // 좌표 범위 출력
                if !s.points.is_empty() {
                    let min_x = s.points.iter().map(|p| p.x).min().unwrap();
                    let max_x = s.points.iter().map(|p| p.x).max().unwrap();
                    let min_y = s.points.iter().map(|p| p.y).min().unwrap();
                    let max_y = s.points.iter().map(|p| p.y).max().unwrap();
                    println!("{}  좌표범위: x=[{},{}], y=[{},{}]", indent, min_x, max_x, min_y, max_y);
                }
            }
            ShapeObject::Curve(s) => {
                println!("{}[곡선] points={}", indent, s.points.len());
                dump_common_fn(&s.common, indent);
                dump_sa_fn(&s.drawing.shape_attr, indent);
            }
            ShapeObject::Group(g) => {
                println!("{}[묶음] children={}", indent, g.children.len());
                dump_common_fn(&g.common, indent);
                dump_sa_fn(&g.shape_attr, indent);
                let child_indent = format!("{}  ", indent);
                for (ci, child) in g.children.iter().enumerate() {
                    print!("{}child[{}] ", child_indent, ci);
                    dump_shape(child, &child_indent, dump_common_fn, dump_sa_fn);
                }
            }
            ShapeObject::Picture(p) => {
                println!("{}[그림] bin_data_id={}", indent, p.image_attr.bin_data_id);
                dump_common_fn(&p.common, indent);
                dump_sa_fn(&p.shape_attr, indent);
            }
            ShapeObject::Chart(c) => {
                println!("{}[차트] type={:?} series={} raw_chart_data={}B", indent, c.chart_type, c.series.len(), c.raw_chart_data.len());
                dump_common_fn(&c.common, indent);
                dump_sa_fn(&c.drawing.shape_attr, indent);
            }
            ShapeObject::Ole(o) => {
                println!("{}[OLE] bin_data_id={} extent={}x{} flags=0x{:02X} raw={}B", indent, o.bin_data_id, o.extent_x, o.extent_y, o.flags, o.raw_tag_data.len());
                dump_common_fn(&o.common, indent);
                dump_sa_fn(&o.drawing.shape_attr, indent);
            }
        }
    }

    for (sec_idx, section) in document.sections.iter().enumerate() {
        if let Some(fs) = filter_section {
            if sec_idx != fs { continue; }
        }

        let pd = &section.section_def.page_def;
        println!("=== 구역 {} ===", sec_idx);
        println!("  용지: {:.1}mm × {:.1}mm ({}×{} HU), {}",
            hu_to_mm(pd.width), hu_to_mm(pd.height), pd.width, pd.height,
            if pd.landscape { "가로" } else { "세로" });
        println!("  여백: 좌={:.1} 우={:.1} 상={:.1} 하={:.1} 머리말={:.1} 꼬리말={:.1} mm",
            hu_to_mm(pd.margin_left), hu_to_mm(pd.margin_right),
            hu_to_mm(pd.margin_top), hu_to_mm(pd.margin_bottom),
            hu_to_mm(pd.margin_header), hu_to_mm(pd.margin_footer));

        // 바탕쪽 정보
        if !section.section_def.master_pages.is_empty() {
            println!("  바탕쪽: {}개", section.section_def.master_pages.len());
            for (mi, mp) in section.section_def.master_pages.iter().enumerate() {
                println!("    [{}] {:?}, 문단 {}개, 영역 {}×{} HU, is_ext={}, overlap={}, ext_flags=0x{:04X}, text_ref={}, num_ref={}",
                    mi, mp.apply_to, mp.paragraphs.len(), mp.text_width, mp.text_height,
                    mp.is_extension, mp.overlap, mp.ext_flags, mp.text_ref, mp.num_ref);
                for (pi, para) in mp.paragraphs.iter().enumerate() {
                    println!("      p[{}]: cc={}, text=\"{}\"", pi, para.controls.len(),
                        if para.text.is_empty() { "(빈 문단)".to_string() } else { para.text.chars().take(30).collect::<String>() });
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        let ctrl_name = match ctrl {
                            Control::Table(t) => {
                                let cell_texts: Vec<String> = t.cells.iter().take(3)
                                    .map(|c| {
                                        c.paragraphs.iter()
                                            .map(|p| p.text.chars().take(20).collect::<String>())
                                            .collect::<Vec<_>>().join("|")
                                    })
                                    .collect();
                                format!("표({}x{}, tac={}, wrap={:?}, vert={:?}/{}, horz={:?}/{}, size={}x{}, cells=[{}])",
                                    t.row_count, t.col_count, t.common.treat_as_char,
                                    t.common.text_wrap, t.common.vert_rel_to, t.common.vertical_offset,
                                    t.common.horz_rel_to, t.common.horizontal_offset,
                                    t.common.width, t.common.height,
                                    cell_texts.join("; "))
                            },
                            Control::Shape(s) => {
                                let mut desc = format!("도형(ctrl_id=0x{:08X}, w={}, h={}, attr=0x{:08X}, wc={:?}, hc={:?})",
                                    s.common().ctrl_id, s.common().width, s.common().height,
                                    s.common().attr, s.common().width_criterion, s.common().height_criterion);
                                // TextBox 내용 출력
                                if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                                    desc += &format!(" 글상자({}문단)", tb.paragraphs.len());
                                    for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                                        let tp_text: String = tp.text.chars().take(20).collect();
                                        desc += &format!("\n          tb_p[{}]: cc={} text=\"{}\"", tpi, tp.controls.len(), tp_text);
                                        for (tci, tc) in tp.controls.iter().enumerate() {
                                            let tc_name = match tc {
                                                Control::AutoNumber(an) => format!("자동번호({:?})", an.number_type),
                                                _ => format!("{:?}", std::mem::discriminant(tc)),
                                            };
                                            desc += &format!("\n            tb_ctrl[{}]: {}", tci, tc_name);
                                        }
                                    }
                                }
                                desc
                            }
                            Control::Picture(p) => {
                                let wm = p.image_attr.watermark_preset().map(|s| format!(", watermark={}", s)).unwrap_or_default();
                                format!("그림(bin_id={}, w={}, h={}, tac={}{})", p.image_attr.bin_data_id, p.common.width, p.common.height, p.common.treat_as_char, wm)
                            },
                            Control::Header(_) => "머리말".to_string(),
                            Control::Footer(_) => "꼬리말".to_string(),
                            _ => format!("{:?}", std::mem::discriminant(ctrl)),
                        };
                        println!("        ctrl[{}]: {}", ci, ctrl_name);
                    }
                }
            }
        }
        if section.section_def.hide_master_page {
            println!("  바탕쪽 감추기: true");
        }

        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            if let Some(fp) = filter_para {
                if para_idx != fp { continue; }
            }

            let text_preview = if para.text.is_empty() {
                "(빈 문단)".to_string()
            } else {
                let preview = if para.text.chars().count() > 50 {
                    let end = para.text.char_indices().nth(50).map(|(i,_)|i).unwrap_or(para.text.len());
                    format!("\"{}...\"", &para.text[..end])
                } else {
                    format!("\"{}\"", para.text)
                };
                preview
            };

            let break_info = break_str(&para.column_type);
            println!("\n--- 문단 {}.{} --- cc={}, text_len={}, controls={} {}",
                sec_idx, para_idx, para.char_count, para.text.chars().count(),
                para.controls.len(), break_info);
            println!("  텍스트: {}", text_preview);
            // char_shapes 출력
            if !para.char_shapes.is_empty() {
                let text_chars: Vec<char> = para.text.chars().collect();
                for (ci, cs) in para.char_shapes.iter().enumerate() {
                    let next_pos = para.char_shapes.get(ci + 1).map(|n| n.start_pos).unwrap_or(u32::MAX);
                    let char_at = text_chars.iter().enumerate()
                        .find(|(i, _)| {
                            if *i < para.char_offsets.len() { para.char_offsets[*i] >= cs.start_pos && para.char_offsets[*i] < next_pos }
                            else { false }
                        })
                        .map(|(_, c)| *c);
                    if let Some(chs) = document.doc_info.char_shapes.get(cs.char_shape_id as usize) {
                        let bold = (chs.attr & 0x02) != 0;
                        let spacing = chs.spacings[0]; // 한국어 자간
                        let ratio = chs.ratios[0]; // 한국어 장평
                        println!("  [CS] pos={} id={} bold={} spacing={}% ratio={}% char={:?}",
                            cs.start_pos, cs.char_shape_id, bold, spacing, ratio,
                            char_at.map(|c| c.to_string()).unwrap_or_default());
                    }
                }
            }
            if let Some(ps) = document.doc_info.para_shapes.get(para.para_shape_id as usize) {
                // 문단 모양 기본 정보 (항상 출력)
                println!("  [PS] ps_id={} align={:?} spacing: before={} after={} line={}/{:?}",
                    para.para_shape_id, ps.alignment,
                    ps.spacing_before, ps.spacing_after,
                    ps.line_spacing, ps.line_spacing_type);
                println!("       margins: left={} right={} indent={} border_fill_id={}",
                    ps.margin_left, ps.margin_right, ps.indent, ps.border_fill_id);
                if ps.border_fill_id > 0 {
                    println!("       border_spacing: left={} right={} top={} bottom={}",
                        ps.border_spacing[0], ps.border_spacing[1],
                        ps.border_spacing[2], ps.border_spacing[3]);
                }
                if ps.head_type != rhwp::model::style::HeadType::None {
                    println!("       head={:?} level={} num_id={} attr1=0x{:08X} attr2=0x{:08X} raw_extra={:?}",
                        ps.head_type, ps.para_level, ps.numbering_id, ps.attr1, ps.attr2,
                        &para.raw_header_extra);
                }
                {
                    let td_id = ps.tab_def_id;
                    if let Some(td) = document.doc_info.tab_defs.get(td_id as usize) {
                        let tabs_str: Vec<String> = td.tabs.iter().enumerate()
                            .map(|(i, t)| format!("tab[{}] pos={} ({:.1}mm) type={} fill={}",
                                i, t.position, hu_to_mm(t.position), t.tab_type, t.fill_type))
                            .collect();
                        println!("       tab_def_id={} auto_left={} auto_right={} tabs=[{}]",
                            td_id, td.auto_tab_left, td.auto_tab_right,
                            if tabs_str.is_empty() { "(없음)".to_string() } else { tabs_str.join(", ") });
                    } else {
                        println!("       tab_def_id={} (정의 없음)", td_id);
                    }
                }
            }
            // line_segs 출력
            if !para.line_segs.is_empty() {
                for (li, ls) in para.line_segs.iter().enumerate() {
                    println!("  ls[{}]: ts={}, vpos={}, lh={}, th={}, bl={}, ls={}, cs={}, sw={}, tag=0x{:08X}",
                        li, ls.text_start, ls.vertical_pos, ls.line_height, ls.text_height,
                        ls.baseline_distance, ls.line_spacing, ls.column_start, ls.segment_width, ls.tag);
                }
            }

            for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
                let prefix = format!("  [{}] ", ctrl_idx);
                match ctrl {
                    Control::ColumnDef(cd) => {
                        let ct = match cd.column_type {
                            rhwp::model::page::ColumnType::Normal => "일반",
                            rhwp::model::page::ColumnType::Distribute => "배분",
                            rhwp::model::page::ColumnType::Parallel => "병행",
                        };
                        println!("{}단정의: {}단, 유형={}, 간격={:.1}mm({}), 같은너비={}",
                            prefix, cd.column_count, ct,
                            hu_to_mm_i(cd.spacing as i32), cd.spacing, cd.same_width);
                        if !cd.widths.is_empty() {
                            // 비례값일 경우 body_width 기준으로 실제 mm 변환
                            let body_width_hu = {
                                let spd = &section.section_def.page_def;
                                let (pw, _) = if spd.landscape { (spd.height, spd.width) } else { (spd.width, spd.height) };
                                (pw - spd.margin_left - spd.margin_right - spd.margin_gutter) as f64
                            };
                            let total: f64 = if cd.proportional_widths {
                                cd.widths.iter().chain(cd.gaps.iter())
                                    .map(|&v| (v as u16) as f64).sum()
                            } else {
                                1.0
                            };
                            let cols_info: Vec<String> = cd.widths.iter().enumerate()
                                .map(|(i, w)| {
                                    let gap = cd.gaps.get(i).copied().unwrap_or(0);
                                    if cd.proportional_widths && total > 0.0 {
                                        let w_hu = (*w as u16) as f64 / total * body_width_hu;
                                        let g_hu = (gap as u16) as f64 / total * body_width_hu;
                                        format!("너비={:.1}mm 간격={:.1}mm", w_hu * 25.4 / 7200.0, g_hu * 25.4 / 7200.0)
                                    } else {
                                        format!("너비={:.1}mm 간격={:.1}mm", hu_to_mm_i(*w as i32), hu_to_mm_i(gap as i32))
                                    }
                                })
                                .collect();
                            println!("{}  단별: [{}]", prefix, cols_info.join(", "));
                        }
                        if cd.separator_type > 0 {
                            println!("{}  구분선: type={}, width={}, color={:#010x}",
                                prefix, cd.separator_type, cd.separator_width, cd.separator_color);
                        }
                    }
                    Control::SectionDef(sd) => {
                        let spd = &sd.page_def;
                        println!("{}구역정의: 용지 {:.1}×{:.1}mm, {}, flags=0x{:08X}",
                            prefix,
                            hu_to_mm(spd.width), hu_to_mm(spd.height),
                            if spd.landscape { "가로" } else { "세로" }, sd.flags);
                        if sd.hide_header || sd.hide_footer || sd.hide_master_page {
                            println!("{}  감추기: 머리말={} 꼬리말={} 바탕쪽={}",
                                prefix, sd.hide_header, sd.hide_footer, sd.hide_master_page);
                        }
                    }
                    Control::Table(table) => {
                        println!("{}표: {}행×{}열, 셀={}, 쪽나눔={:?} (attr=0x{:08x}), padding=({},{},{},{}), cs={}",
                            prefix, table.row_count, table.col_count,
                            table.cells.len(), table.page_break, table.raw_table_record_attr,
                            table.padding.left, table.padding.right, table.padding.top, table.padding.bottom,
                            table.cell_spacing);
                        if !table.zones.is_empty() {
                            for (zi, z) in table.zones.iter().enumerate() {
                                println!("{}  zone[{}] row={}..{} col={}..{} bf={}",
                                    prefix, zi, z.start_row, z.end_row, z.start_col, z.end_col, z.border_fill_id);
                            }
                        }
                        {
                            let c = &table.common;
                            println!("{}  [common] treat_as_char={}, wrap={}, vert={}({}={:.1}mm), horz={}({}={:.1}mm)",
                                prefix, c.treat_as_char, wrap_str(&c.text_wrap),
                                vert_str(&c.vert_rel_to), c.vertical_offset, hu_to_mm(c.vertical_offset),
                                horz_str(&c.horz_rel_to), c.horizontal_offset, hu_to_mm(c.horizontal_offset));
                            println!("{}  [common] size={}×{}({:.1}×{:.1}mm), valign={:?}, halign={:?}",
                                prefix, c.width, c.height, hu_to_mm(c.width), hu_to_mm(c.height),
                                c.vert_align, c.horz_align);
                            println!("{}  [outer_margin] left={:.1}mm({}) right={:.1}mm({}) top={:.1}mm({}) bottom={:.1}mm({})",
                                prefix,
                                hu_to_mm_i(table.outer_margin_left as i32), table.outer_margin_left,
                                hu_to_mm_i(table.outer_margin_right as i32), table.outer_margin_right,
                                hu_to_mm_i(table.outer_margin_top as i32), table.outer_margin_top,
                                hu_to_mm_i(table.outer_margin_bottom as i32), table.outer_margin_bottom);
                            if table.raw_ctrl_data.len() >= 20 {
                                println!("{}  [raw] {:02X?}", prefix, &table.raw_ctrl_data[..20.min(table.raw_ctrl_data.len())]);
                            }
                        }
                        // 셀 상세 출력
                        fn dump_table_deep(table: &rhwp::model::table::Table, indent: &str, depth: usize) {
                            for (ci, cell) in table.cells.iter().enumerate() {
                                let text_preview: String = cell.paragraphs.iter()
                                    .map(|p| p.text.chars().take(30).collect::<String>())
                                    .collect::<Vec<_>>().join("|");
                                println!("{}셀[{}] r={},c={} rs={},cs={} h={} w={} pad=({},{},{},{}) aim={} bf={} paras={} text=\"{}\"",
                                    indent, ci, cell.row, cell.col, cell.row_span, cell.col_span,
                                    cell.height, cell.width,
                                    cell.padding.left, cell.padding.right, cell.padding.top, cell.padding.bottom,
                                    cell.apply_inner_margin,
                                    cell.border_fill_id, cell.paragraphs.len(), text_preview);
                                if let Some(ref fname) = cell.field_name {
                                    println!("{}  field=\"{}\"", indent, fname);
                                }
                                // 셀 내 LINE_SEG 상세
                                for (pi, cp) in cell.paragraphs.iter().enumerate() {
                                    if !cp.line_segs.is_empty() || !cp.controls.is_empty() {
                                        let ls_info: Vec<String> = cp.line_segs.iter().enumerate()
                                            .map(|(li, ls)| format!("ls[{}] vpos={} lh={} ls={}", li, ls.vertical_pos, ls.line_height, ls.line_spacing))
                                            .collect();
                                        println!("{}  p[{}] ps_id={} ctrls={} text_len={} {}",
                                            indent, pi, cp.para_shape_id, cp.controls.len(),
                                            cp.text.len(), ls_info.join(", "));
                                    }
                                    // 셀 내부 컨트롤 상세
                                    for (ci, ctrl) in cp.controls.iter().enumerate() {
                                        match ctrl {
                                            Control::Picture(p) => {
                                                println!("{}    ctrl[{}] 그림: bin_id={}, w={} h={} ({:.1}×{:.1}mm), tac={}, wrap={:?}, vert={:?}(off={}), horz={:?}(off={}), orig={}×{}, cur={}×{}, crop=({},{},{},{})",
                                                    indent, ci, p.image_attr.bin_data_id,
                                                    p.common.width, p.common.height,
                                                    p.common.width as f64 / 7200.0 * 25.4,
                                                    p.common.height as f64 / 7200.0 * 25.4,
                                                    p.common.treat_as_char,
                                                    p.common.text_wrap, p.common.vert_rel_to, p.common.vertical_offset,
                                                    p.common.horz_rel_to, p.common.horizontal_offset,
                                                    p.shape_attr.original_width, p.shape_attr.original_height,
                                                    p.shape_attr.current_width, p.shape_attr.current_height,
                                                    p.crop.left, p.crop.top, p.crop.right, p.crop.bottom);
                                                println!("{}      [image_attr] effect={:?} brightness={} contrast={} watermark={}",
                                                    indent, p.image_attr.effect, p.image_attr.brightness, p.image_attr.contrast,
                                                    p.image_attr.watermark_preset().unwrap_or("none"));
                                            }
                                            Control::Shape(s) => {
                                                println!("{}    ctrl[{}] {}: tac={}, wrap={:?}",
                                                    indent, ci, s.shape_name(), s.common().treat_as_char, s.common().text_wrap);
                                            }
                                            _ => {}
                                        }
                                    }
                                    // 내부 표 재귀
                                    if depth < 3 {
                                        for ctrl in &cp.controls {
                                            if let Control::Table(inner) = ctrl {
                                                println!("{}  p[{}] 내부표: {}행×{}열, 셀={}, cs={}, pad=({},{},{},{})",
                                                    indent, pi, inner.row_count, inner.col_count,
                                                    inner.cells.len(), inner.cell_spacing,
                                                    inner.padding.left, inner.padding.right, inner.padding.top, inner.padding.bottom);
                                                let next_indent = format!("{}    ", indent);
                                                dump_table_deep(inner, &next_indent, depth + 1);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        dump_table_deep(table, &format!("{}  ", prefix), 0);
                    }
                    Control::Shape(shape) => {
                        print!("{}", prefix);
                        dump_shape(shape, "  ", &dump_common, &dump_shape_attr);
                    }
                    Control::Picture(pic) => {
                        let sa = &pic.shape_attr;
                        println!("{}그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}",
                            prefix, pic.image_attr.bin_data_id, pic.common.width, pic.common.height,
                            pic.common.width as f64 / 7200.0 * 25.4, pic.common.height as f64 / 7200.0 * 25.4,
                            sa.original_width, sa.original_height,
                            sa.original_width as f64 / 7200.0 * 25.4, sa.original_height as f64 / 7200.0 * 25.4,
                            sa.current_width, sa.current_height,
                            sa.current_width as f64 / 7200.0 * 25.4, sa.current_height as f64 / 7200.0 * 25.4,
                            pic.common.treat_as_char);
                        println!("{}  [image_attr] effect={:?} brightness={} contrast={} watermark={}",
                            prefix, pic.image_attr.effect, pic.image_attr.brightness, pic.image_attr.contrast,
                            pic.image_attr.watermark_preset().unwrap_or("none"));
                        println!("{}  border_x={:?} border_y={:?} border_color=#{:06X} border_width={} ({:.2}mm) border_attr={:?}",
                            prefix, pic.border_x, pic.border_y,
                            pic.border_color, pic.border_width, pic.border_width as f64 / 7200.0 * 25.4,
                            pic.border_attr);
                        println!("{}  crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
                            prefix, pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom,
                            pic.crop.left as f64 / 7200.0 * 25.4, pic.crop.top as f64 / 7200.0 * 25.4,
                            pic.crop.right as f64 / 7200.0 * 25.4, pic.crop.bottom as f64 / 7200.0 * 25.4);
                        dump_common(&pic.common, "  ");
                    }
                    Control::Header(h) => {
                        let text: String = h.paragraphs.iter()
                            .filter(|p| !p.text.is_empty())
                            .map(|p| p.text.clone())
                            .collect::<Vec<_>>()
                            .join(" ");
                        println!("{}머리말({:?}): paras={} \"{}\"", prefix, h.apply_to, h.paragraphs.len(), text);
                        for (hpi, hp) in h.paragraphs.iter().enumerate() {
                            if !hp.controls.is_empty() {
                                for (hci, hc) in hp.controls.iter().enumerate() {
                                    let cn = match hc {
                                        Control::AutoNumber(an) => format!("자동번호({:?})", an.number_type),
                                        Control::Shape(s) => {
                                            let c = s.common();
                                            let mut desc = format!("Shape horz={:?}/{} halign={:?} w={} h={}",
                                                c.horz_rel_to, c.horizontal_offset, c.horz_align, c.width, c.height);
                                            if let Some(tb) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                                                let text: String = tb.paragraphs.iter()
                                                    .flat_map(|p| p.text.chars().take(20))
                                                    .collect();
                                                desc += &format!(" text={:?}", text);
                                            }
                                            desc
                                        }
                                        Control::Table(t) => {
                                            let mut desc = format!("표 {}행×{}열 셀={}", t.row_count, t.col_count, t.cells.len());
                                            for (si, cell) in t.cells.iter().enumerate() {
                                                let cell_text: String = cell.paragraphs.iter()
                                                    .flat_map(|p| p.text.chars().take(20))
                                                    .collect();
                                                desc += &format!("\n{}    셀[{}] text={:?}", prefix, si, cell_text);
                                                for (cpi, cp) in cell.paragraphs.iter().enumerate() {
                                                    for (cci, cc) in cp.controls.iter().enumerate() {
                                                        let ccn = match cc {
                                                            Control::AutoNumber(an) => format!("자동번호({:?})", an.number_type),
                                                            Control::Shape(s) => {
                                            let c = s.common();
                                            let mut d = format!("Shape vert={:?}/{} valign={:?} horz={:?}/{} halign={:?} w={} h={}",
                                                c.vert_rel_to, c.vertical_offset, c.vert_align,
                                                c.horz_rel_to, c.horizontal_offset, c.horz_align, c.width, c.height);
                                            if let Some(tb) = s.drawing().and_then(|dd| dd.text_box.as_ref()) {
                                                for (tpi, tp) in tb.paragraphs.iter().enumerate() {
                                                    let t: String = tp.text.chars().take(30).collect();
                                                    d += &format!(" tb_p[{}] ps_id={} text={:?}", tpi, tp.para_shape_id, t);
                                                }
                                            }
                                            d
                                        }
                                        _ => format!("{:?}", std::mem::discriminant(cc)),
                                                        };
                                                        desc += &format!("\n{}      p[{}]c[{}]: {}", prefix, cpi, cci, ccn);
                                                    }
                                                }
                                            }
                                            desc
                                        }
                                        Control::Picture(pic) => {
                                            let sa = &pic.shape_attr;
                                            format!("그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}, crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
                                            pic.image_attr.bin_data_id, pic.common.width, pic.common.height,
                                            pic.common.width as f64 / 7200.0 * 25.4, pic.common.height as f64 / 7200.0 * 25.4,
                                            sa.original_width, sa.original_height,
                                            sa.original_width as f64 / 7200.0 * 25.4, sa.original_height as f64 / 7200.0 * 25.4,
                                            sa.current_width, sa.current_height,
                                            sa.current_width as f64 / 7200.0 * 25.4, sa.current_height as f64 / 7200.0 * 25.4,
                                            pic.common.treat_as_char,
                                            pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom,
                                            pic.crop.left as f64 / 7200.0 * 25.4, pic.crop.top as f64 / 7200.0 * 25.4,
                                            pic.crop.right as f64 / 7200.0 * 25.4, pic.crop.bottom as f64 / 7200.0 * 25.4)
                                        },
                                        _ => format!("{:?}", std::mem::discriminant(hc)),
                                    };
                                    let display = if cn.chars().count() > 30 {
                                        format!("{}...(truncated)", cn.chars().take(30).collect::<String>())
                                    } else {
                                        cn
                                    };
                                    println!("{}  hp[{}] ctrl[{}]: {}", prefix, hpi, hci, display);
                                }
                            }
                        }
                    }
                    Control::Footer(f) => {
                        let text: String = f.paragraphs.iter()
                            .filter(|p| !p.text.is_empty())
                            .map(|p| p.text.clone())
                            .collect::<Vec<_>>()
                            .join(" ");
                        println!("{}꼬리말({:?}): paras={} \"{}\"", prefix, f.apply_to, f.paragraphs.len(), text);
                        for (fpi, fp) in f.paragraphs.iter().enumerate() {
                            if !fp.controls.is_empty() {
                                for (fci, fc) in fp.controls.iter().enumerate() {
                                    let cn = match fc {
                                        Control::Picture(pic) => {
                                            let sa = &pic.shape_attr;
                                            format!("그림: bin_id={}, common={}×{} ({:.1}×{:.1}mm), orig={}×{} ({:.1}×{:.1}mm), cur={}×{} ({:.1}×{:.1}mm), tac={}, crop=({},{},{},{}) crop_mm=({:.2},{:.2},{:.2},{:.2})",
                                            pic.image_attr.bin_data_id, pic.common.width, pic.common.height,
                                            pic.common.width as f64 / 7200.0 * 25.4, pic.common.height as f64 / 7200.0 * 25.4,
                                            sa.original_width, sa.original_height,
                                            sa.original_width as f64 / 7200.0 * 25.4, sa.original_height as f64 / 7200.0 * 25.4,
                                            sa.current_width, sa.current_height,
                                            sa.current_width as f64 / 7200.0 * 25.4, sa.current_height as f64 / 7200.0 * 25.4,
                                            pic.common.treat_as_char,
                                            pic.crop.left, pic.crop.top, pic.crop.right, pic.crop.bottom,
                                            pic.crop.left as f64 / 7200.0 * 25.4, pic.crop.top as f64 / 7200.0 * 25.4,
                                            pic.crop.right as f64 / 7200.0 * 25.4, pic.crop.bottom as f64 / 7200.0 * 25.4)
                                        },
                                        _ => format!("{:?}", std::mem::discriminant(fc)),
                                    };
                                    println!("{}  fp[{}] ctrl[{}]: {}", prefix, fpi, fci, cn);
                                }
                            }
                        }
                    }
                    Control::Footnote(fn_) => {
                        println!("{}각주: paragraphs={}", prefix, fn_.paragraphs.len());
                    }
                    Control::Endnote(en) => {
                        println!("{}미주: paragraphs={}", prefix, en.paragraphs.len());
                    }
                    Control::AutoNumber(an) => {
                        println!("{}자동번호: type={:?}, number={}", prefix, an.number_type, an.number);
                    }
                    Control::NewNumber(nn) => {
                        println!("{}새번호: type={:?}, number={}", prefix, nn.number_type, nn.number);
                    }
                    Control::PageNumberPos(pn) => {
                        println!("{}쪽번호위치: format={}, pos={}", prefix, pn.format, pn.position);
                    }
                    Control::Bookmark(bm) => {
                        println!("{}책갈피: \"{}\"", prefix, bm.name);
                    }
                    Control::Hyperlink(hl) => {
                        println!("{}하이퍼링크: \"{}\"", prefix, hl.url);
                    }
                    Control::Ruby(r) => {
                        println!("{}덧말: \"{}\"", prefix, r.ruby_text);
                    }
                    Control::PageHide(ph) => {
                        println!("{}감추기: header={}, footer={}, master={}, border={}, fill={}, page_num={}",
                            prefix, ph.hide_header, ph.hide_footer, ph.hide_master_page, ph.hide_border, ph.hide_fill, ph.hide_page_num);
                    }
                    Control::HiddenComment(_) => {
                        println!("{}숨은설명", prefix);
                    }
                    Control::Field(f) => {
                        let name = f.field_name().unwrap_or("(이름없음)");
                        println!("{}필드: {:?} name=\"{}\" cmd=\"{}\"", prefix, f.field_type, name, f.command);
                    }
                    Control::CharOverlap(co) => {
                        println!("{}글자겹침: {:?}", prefix, co.chars);
                    }
                    Control::Equation(eq) => {
                        println!("{}수식: script=\"{}\" font_size={} font=\"{}\" size={}x{} tac={}",
                            prefix, eq.script, eq.font_size, eq.font_name,
                            eq.common.width, eq.common.height, eq.common.treat_as_char);
                    }
                    Control::Form(f) => {
                        println!("{}양식개체: {:?} name=\"{}\" caption=\"{}\" {}x{}",
                            prefix, f.form_type, f.name, f.caption, f.width, f.height);
                    }
                    Control::Unknown(u) => {
                        println!("{}알수없음: ctrl_id={:#010x}", prefix, u.ctrl_id);
                    }
                }
            }
        }
    }

    println!("\n=== 완료: {} 구역, {} 문단 ===",
        document.sections.len(),
        document.sections.iter().map(|s| s.paragraphs.len()).sum::<usize>());
}

fn diag_document(args: &[String]) {
    if args.is_empty() {
        eprintln!("오류: HWP 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp diag <파일.hwp>");
        return;
    }

    let file_path = &args[0];
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return;
        }
    };

    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let document = doc.document();
    use rhwp::model::style::HeadType;

    // === DocInfo 요약 ===
    println!("=== DocInfo 요약 ===");
    println!("  Numbering: {}개", document.doc_info.numberings.len());
    for (i, num) in document.doc_info.numberings.iter().enumerate() {
        let formats: Vec<String> = num.level_formats.iter()
            .enumerate()
            .filter(|(_, f)| !f.is_empty())
            .map(|(lv, f)| format!("L{}=\"{}\"", lv + 1, f))
            .collect();
        println!("    [{}] start={}, formats: {}", i, num.start_number, formats.join(", "));
    }

    println!("  Bullet: {}개", document.doc_info.bullets.len());
    for (i, bullet) in document.doc_info.bullets.iter().enumerate() {
        println!("    [{}] char='{}' (U+{:04X})", i, bullet.bullet_char, bullet.bullet_char as u32);
    }

    // === ParaShape head_type 분포 ===
    println!("\n=== ParaShape head_type 분포 ===");
    let mut count_none = 0u32;
    let mut count_outline = 0u32;
    let mut count_number = 0u32;
    let mut count_bullet = 0u32;
    for ps in &document.doc_info.para_shapes {
        match ps.head_type {
            HeadType::None => count_none += 1,
            HeadType::Outline => count_outline += 1,
            HeadType::Number => count_number += 1,
            HeadType::Bullet => count_bullet += 1,
        }
    }
    println!("  None: {}개, Outline: {}개, Number: {}개, Bullet: {}개",
        count_none, count_outline, count_number, count_bullet);

    // === SectionDef 개요번호 ===
    println!("\n=== SectionDef 개요번호 ===");
    for (sec_idx, section) in document.sections.iter().enumerate() {
        // SectionDef의 raw_ctrl_extra에서 바이트 14-15 추출 (outline_numbering_id)
        // 현재 outline_numbering_id 필드가 없으므로 파싱 전 상태에서는 raw_ctrl_extra 참조
        // 6단계에서 필드 추가 후 직접 참조로 변경 예정
        let sd = &section.section_def;
        let num_ref = if sd.outline_numbering_id > 0 {
            format!(" → Numbering[{}]", sd.outline_numbering_id - 1)
        } else {
            " (없음)".to_string()
        };
        println!("  구역{}: outline_numbering_id={}{}, flags={:#010x}",
            sec_idx, sd.outline_numbering_id, num_ref, sd.flags);
    }

    // === 비None head_type 문단 ===
    println!("\n=== 비None head_type 문단 ===");
    for (sec_idx, section) in document.sections.iter().enumerate() {
        for (para_idx, para) in section.paragraphs.iter().enumerate() {
            if let Some(ps) = document.doc_info.para_shapes.get(para.para_shape_id as usize) {
                if ps.head_type != HeadType::None {
                    let text_preview: String = para.text.chars().take(40).collect();
                    let text_display = if para.text.chars().count() > 40 {
                        format!("\"{}...\"", text_preview)
                    } else {
                        format!("\"{}\"", text_preview)
                    };
                    println!("  구역{}:문단{} head={:?} level={} num_id={} text={}",
                        sec_idx, para_idx,
                        ps.head_type, ps.para_level, ps.numbering_id,
                        text_display);
                }
            }
        }
    }
}

fn convert_hwp(args: &[String]) {
    if args.len() < 2 {
        eprintln!("오류: 입력 파일과 출력 파일 경로를 지정해주세요.");
        eprintln!("사용법: rhwp convert <입력.hwp> <출력.hwp>");
        return;
    }

    let input_path = &args[0];
    let output_path = &args[1];

    // 입력 파일 읽기
    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", input_path, e);
            return;
        }
    };

    // 문서 로드
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return;
        }
    };

    let was_distribution = doc.document().header.distribution;
    if !was_distribution {
        println!("{}: 이미 편집 가능한 문서입니다.", input_path);
    }

    // 변환
    match doc.convert_to_editable_native() {
        Ok(_) => {
            if was_distribution {
                println!("배포용 → 편집 가능 변환 완료");
            }
        }
        Err(e) => {
            eprintln!("오류: 변환 실패 - {}", e);
            return;
        }
    }

    // 직렬화
    match doc.export_hwp_native() {
        Ok(bytes) => {
            match fs::write(output_path, &bytes) {
                Ok(_) => {
                    println!("저장 완료: {} ({}KB)", output_path, bytes.len() / 1024);
                }
                Err(e) => {
                    eprintln!("오류: 파일 저장 실패 - {}: {}", output_path, e);
                }
            }
        }
        Err(e) => {
            eprintln!("오류: 직렬화 실패 - {}", e);
        }
    }
}

/// `rhwp build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>`
///
/// Claude Code Skill (`rhwp-exam-ingest`)이 생성한 JSON 중간 표현을 HWPX로 변환한다.
/// Task #660 (Neumann 본 작업 1단계).
fn build_from_ingest(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx>");
        return;
    }

    let mut input_path: Option<&str> = None;
    let mut output_path: Option<&str> = None;
    let mut media_dir: Option<&str> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: -o 옵션에 값이 필요합니다");
                    return;
                }
                output_path = Some(&args[i + 1]);
                i += 2;
            }
            "--media-dir" => {
                if i + 1 >= args.len() {
                    eprintln!("오류: --media-dir 옵션에 값이 필요합니다");
                    return;
                }
                media_dir = Some(&args[i + 1]);
                i += 2;
            }
            other => {
                if input_path.is_none() {
                    input_path = Some(other);
                } else {
                    eprintln!("경고: 알 수 없는 인자 '{}' 무시", other);
                }
                i += 1;
            }
        }
    }

    let input = match input_path {
        Some(p) => p,
        None => {
            eprintln!("오류: 입력 ingest JSON 경로가 누락되었습니다");
            return;
        }
    };
    let output = match output_path {
        Some(p) => p,
        None => {
            eprintln!("오류: -o <출력 경로> 가 누락되었습니다");
            return;
        }
    };

    let bytes = match fs::read(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 입력 파일 읽기 실패 - {}: {}", input, e);
            return;
        }
    };

    let ingest = match rhwp::parser::ingest::parse_ingest_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: ingest JSON 파싱 실패 - {}", e);
            return;
        }
    };

    if let Some(md) = media_dir {
        let p = Path::new(md);
        if !p.exists() {
            eprintln!(
                "경고: 미디어 디렉토리가 존재하지 않습니다 ({}). 본 단계는 이미지 placeholder로 처리됩니다.",
                md
            );
        }
    }

    let doc = rhwp::document_core::builders::exam_paper::build_exam_paper(&ingest);

    let hwpx_bytes = match rhwp::serializer::serialize_hwpx(&doc) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: HWPX 직렬화 실패 - {}", e);
            return;
        }
    };

    match fs::write(output, &hwpx_bytes) {
        Ok(_) => println!(
            "저장 완료: {} ({}바이트, 문제 {}개, 문단 {}개)",
            output,
            hwpx_bytes.len(),
            ingest.questions.len(),
            doc.sections.iter().map(|s| s.paragraphs.len()).sum::<usize>()
        ),
        Err(e) => eprintln!("오류: 파일 저장 실패 - {}: {}", output, e),
    }
}

fn dump_raw_records(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp dump-records <파일.hwp>");
        return;
    }
    let data = match fs::read(&args[0]) {
        Ok(d) => d,
        Err(e) => { eprintln!("오류: {}", e); return; }
    };
    use rhwp::parser::cfb_reader::CfbReader;
    use rhwp::parser::record::Record;
    let mut cfb = match CfbReader::open(&data) {
        Ok(c) => c,
        Err(e) => { eprintln!("오류: {:?}", e); return; }
    };
    // FileHeader에서 압축 여부 확인
    let header = cfb.read_stream_raw("FileHeader").unwrap_or_default();
    let compressed = header.len() >= 40 && (header[36] & 0x01) != 0;
    let section = match cfb.read_body_text_section(0, compressed, false) {
        Ok(s) => s,
        Err(e) => { eprintln!("오류: {:?}", e); return; }
    };
    let records = match Record::read_all(&section) {
        Ok(r) => r,
        Err(e) => { eprintln!("오류: {:?}", e); return; }
    };
    let tag_name = |id: u16| -> &str {
        match id {
            66 => "PARA_HEADER", 67 => "PARA_TEXT", 68 => "PARA_CHAR_SHAPE",
            69 => "PARA_LINE_SEG", 70 => "PARA_RANGE_TAG", 71 => "CTRL_HEADER",
            72 => "LIST_HEADER", 73 => "PAGE_DEF", 74 => "FOOTNOTE_SHAPE",
            75 => "PAGE_BORDER_FILL", 76 => "SHAPE_COMPONENT", 77 => "TABLE",
            78 => "SC_LINE", 79 => "SC_RECT", 80 => "SC_ELLIPSE",
            81 => "SC_ARC", 82 => "SC_POLYGON", 83 => "SC_CURVE",
            85 => "SC_PICTURE", 86 => "SC_CONTAINER", 89 => "CTRL_DATA",
            _ => "?",
        }
    };
    for (i, rec) in records.iter().enumerate() {
        let indent = "  ".repeat(rec.level as usize);
        println!("[{:3}] {}tag={:<3} {:16} lv={} sz={}",
            i, indent, rec.tag_id, tag_name(rec.tag_id), rec.level, rec.data.len());
        // shape 관련 레코드만 hex 덤프
        if matches!(rec.tag_id, 71 | 72 | 76 | 79 | 85 | 89) {
            // 16바이트씩 나눠서 hex 출력
            for chunk in rec.data.chunks(16) {
                let hex: String = chunk.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join(" ");
                println!("       {}  {}", indent, hex);
            }
        }
    }
}

fn test_shape_roundtrip(args: &[String]) {
    let input = if args.is_empty() { "saved/g555-s.hwp" } else { &args[0] };
    let output = if args.len() > 1 { &args[1] } else { "/tmp/test-shape-out.hwp" };

    let data = match fs::read(input) {
        Ok(d) => d,
        Err(e) => { eprintln!("입력 파일 읽기 오류: {}", e); return; }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => { eprintln!("HWP 파싱 오류: {:?}", e); return; }
    };

    let _ = doc.convert_to_editable_native();

    // 글상자 생성 (9000 x 6750 HWPUNIT)
    let result = doc.create_shape_control_native(0, 0, 0, 9000, 6750, 0, 0, false, "InFrontOfText", "rectangle", false, false, &[]);
    match &result {
        Ok(r) => eprintln!("글상자 생성 성공: {}", r),
        Err(e) => { eprintln!("글상자 생성 실패: {:?}", e); return; }
    }

    match doc.export_hwp_native() {
        Ok(bytes) => {
            if let Err(e) = fs::write(output, &bytes) {
                eprintln!("파일 저장 오류: {}", e);
            } else {
                eprintln!("저장 완료: {} ({}KB)", output, bytes.len() / 1024);
            }
        }
        Err(e) => eprintln!("직렬화 오류: {:?}", e),
    }
}

/// 캡션 방향별 테스트: 4개 이미지에 각각 Bottom/Top/Left/Right 캡션을 설정하고 SVG 출력
fn test_caption(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp test-caption <파일.hwp>");
        return;
    }

    let data = match fs::read(&args[0]) {
        Ok(d) => d,
        Err(e) => { eprintln!("파일 읽기 오류: {}", e); return; }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => { eprintln!("파싱 오류: {}", e); return; }
    };

    // 문단 0: 컨트롤 2,3 / 문단 1: 컨트롤 0,1
    let pic_refs: [(usize, usize); 4] = [(0, 2), (0, 3), (1, 0), (1, 1)];

    // 4개 이미지에 각각 다른 캡션 방향 설정
    let directions = [
        ("Bottom", "Top"),
        ("Top", "Top"),
        ("Left", "Center"),
        ("Right", "Center"),
    ];

    for (i, ((para, ci), (dir, va))) in pic_refs.iter().zip(directions.iter()).enumerate() {
        let json = format!(
            r#"{{"hasCaption":true,"captionDirection":"{}","captionVertAlign":"{}","captionWidth":8504,"captionSpacing":850}}"#,
            dir, va
        );
        println!("[{}] para={}, ci={}, dir={}, va={}", i, para, ci, dir, va);
        match doc.set_picture_properties_native(0, *para, *ci, &json) {
            Ok(r) => println!("  결과: {}", r),
            Err(e) => println!("  오류: {:?}", e),
        }
    }

    // 캡션 상태 확인
    for (i, (para, ci)) in pic_refs.iter().enumerate() {
        let section = &doc.document().sections[0];
        let p = &section.paragraphs[*para];
        if let rhwp::model::control::Control::Picture(pic) = &p.controls[*ci] {
            println!("[{}] caption={:?}", i, pic.caption.as_ref().map(|c| {
                format!("dir={:?}, paras={}, text={:?}",
                    c.direction, c.paragraphs.len(),
                    c.paragraphs.first().map(|p| &p.text))
            }));
        }
    }

    // SVG 출력
    let output_dir = "output/caption-test";
    let _ = fs::create_dir_all(output_dir);
    let page_count = doc.page_count();
    println!("페이지 수: {}", page_count);
    for p in 0..page_count {
        let svg = doc.render_page_svg(p).expect("SVG 렌더링 오류");
        let path = format!("{}/caption-test-p{}.svg", output_dir, p);
        fs::write(&path, &svg).unwrap();
        println!("  → {}", path);
    }
    println!("완료");
}

fn gen_table(args: &[String]) {
    let rows: u16 = args.first().and_then(|s| s.parse().ok()).unwrap_or(1000);
    let cols: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(6);
    let output = args.get(2).map(|s| s.as_str()).unwrap_or("output/gen_table.hwp");

    println!("{}행 × {}열 표 생성 중...", rows, cols);

    let mut core = rhwp::document_core::DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서 생성 실패");

    // 표 생성
    let result = core.create_table_native(0, 0, 0, rows, cols)
        .expect("표 생성 실패");
    println!("  표 생성: {}", result);

    // 결과에서 paraIdx 파싱
    let table_para_idx: usize = result.split("\"paraIdx\":").nth(1)
        .and_then(|s| s.split(&[',', '}'][..]).next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1);
    println!("  표 문단 인덱스: {}", table_para_idx);

    // 배치 모드로 셀 내용 채우기
    core.begin_batch_native().expect("배치 시작 실패");

    let headers = ["번호", "이름", "부서", "직급", "연락처", "비고"];
    // 헤더 행
    for (ci, header) in headers.iter().enumerate().take(cols as usize) {
        let _ = core.insert_text_in_cell_native(0, table_para_idx, 0, ci, 0, 0, header);
    }

    // 데이터 행
    let departments = ["개발팀", "기획팀", "디자인팀", "영업팀", "인사팀", "재무팀"];
    let positions = ["사원", "대리", "과장", "차장", "부장"];
    for row in 1..rows as usize {
        for col in 0..cols as usize {
            let cell_idx = row * cols as usize + col;
            let text = match col {
                0 => format!("{}", row),
                1 => format!("홍길동{}", row),
                2 => departments[row % departments.len()].to_string(),
                3 => positions[row % positions.len()].to_string(),
                4 => format!("010-{:04}-{:04}", 1000 + row % 9000, 1000 + (row * 7) % 9000),
                5 => if row % 3 == 0 { "특이사항 없음".to_string() } else { String::new() },
                _ => format!("R{}C{}", row, col),
            };
            if !text.is_empty() {
                let _ = core.insert_text_in_cell_native(0, table_para_idx, 0, cell_idx, 0, 0, &text);
            }
        }
        if row % 100 == 0 {
            println!("  {} / {} 행 완료", row, rows);
        }
    }

    core.end_batch_native().expect("배치 종료 실패");
    println!("  셀 내용 입력 완료");

    // 저장
    let bytes = core.export_hwp_native().expect("HWP 내보내기 실패");
    let out_path = Path::new(output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(out_path, bytes).expect("파일 저장 실패");
    println!("저장 완료: {} ({}행 × {}열)", output, rows, cols);
}

/// PUA (Private Use Area) 문자 셋트를 입력한 HWP 테스트 문서 생성.
///
/// Task #509 (PUA 회귀 정정) 의 한컴 정답지 확보용. 본 라이브러리가 발견한
/// 14 샘플 광범위 PUA 코드포인트 18 종을 한 문서에 입력 → 한컴 편집기로 PDF
/// 출력 + rhwp SVG 출력 시각 비교.
///
/// 사용:
///   rhwp gen-pua [output_path]
///   기본 출력: output/pua-test.hwp
fn gen_pua_test(args: &[String]) {
    let output = args.first().map(|s| s.as_str()).unwrap_or("output/pua-test.hwp");

    println!("PUA 문자 셋트 입력 HWP 문서 생성 중...");

    let mut core = rhwp::document_core::DocumentCore::new_empty();
    core.create_blank_document_native().expect("빈 문서 생성 실패");

    // PUA 코드포인트 셋트 (Task #509 Stage 1 의 14 샘플 광범위 통계 정합)
    // (codepoint, 영역 분류, 사용 샘플, 본 라이브러리 현재 매핑)
    let pua_set: &[(u32, &str, &str, &str)] = &[
        // ── Basic PUA (0xF020~0xF0FF) — 매핑 표 적용 영역 ──
        (0x0F076, "Basic",      "mel-001",      "❖ U+2756"),
        (0x0F09F, "Basic",      "biz_plan",     "• U+2022"),
        (0x0F0A0, "Basic",      "synam-001",    "▪ U+25AA"),
        (0x0F0A7, "Basic",      "kps-ai",       "▪ U+25AA"),
        (0x0F0E8, "Basic",      "kps-ai",       "(미정의)"),
        (0x0F0F2, "Basic",      "KTX",          "⇩ U+21E9 (의도 정정 후보)"),
        (0x0F0FE, "Basic",      "k-water-rfp",  "☑ U+2611"),
        // ── Basic PUA — 매핑 표 외 영역 ──
        (0x0F53A, "Basic-out",  "hwpspec",      "(매핑 표 외)"),
        // ── Supplementary PUA-A (0xF0000~0xFFFFD) — 매핑 표 미지원 영역 ──
        (0xF02B1, "Suppl-A",    "mel-001",      "(매핑 표 외)"),
        (0xF02B2, "Suppl-A",    "mel-001",      "(매핑 표 외)"),
        (0xF02B3, "Suppl-A",    "mel-001",      "(매핑 표 외)"),
        (0xF02B4, "Suppl-A",    "mel-001",      "(매핑 표 외)"),
        (0xF02B5, "Suppl-A",    "mel-001",      "(매핑 표 외)"),
        (0xF02B6, "Suppl-A",    "mel-001",      "(매핑 표 외)"),
        (0xF02B7, "Suppl-A",    "mel-001",      "(매핑 표 외)"),
        (0xF02B8, "Suppl-A",    "mel-001",      "(매핑 표 외)"),
        (0xF02B9, "Suppl-A",    "mel-001",      "(매핑 표 외)"),
        (0xF02EF, "Suppl-A",    "KTX (회귀)",   "(매핑 표 외) ★"),
    ];

    println!("  PUA 코드포인트 {} 종 입력", pua_set.len());

    core.begin_batch_native().expect("배치 시작 실패");

    // 첫 paragraph (0번) 에 제목 입력
    let title = "[PUA 회귀 검증 — Task #509]";
    core.insert_text_native(0, 0, 0, title).expect("제목 입력 실패");

    // 각 PUA 글자별로 paragraph 추가:
    // "U+0F0F2 (Basic, KTX): {char}    ← 한컴 정답지 / rhwp 비교"
    // 빈 paragraph 추가 + 텍스트 입력 패턴
    for (i, &(cp, area, sample, mapping)) in pua_set.iter().enumerate() {
        let pi = i + 1; // 0번은 제목, 1번부터 PUA paragraphs

        // 새 paragraph 추가 (pi 위치에 새 문단 삽입)
        core.insert_paragraph_native(0, pi)
            .unwrap_or_else(|e| panic!("paragraph 추가 실패 (pi={}): {:?}", pi, e));

        // PUA 글자 char 변환 (i32 unsafe 회피)
        let pua_char = char::from_u32(cp)
            .unwrap_or_else(|| panic!("invalid codepoint U+{:05X}", cp));

        // 텍스트: "U+0F0F2 (Basic, KTX, ⇩ U+21E9 매핑): " + PUA + "  ← 한컴 PDF 글리프 정답지"
        let text = format!(
            "U+{:05X} ({}, {}, {}): {}  ← 한컴 PDF 정답지",
            cp, area, sample, mapping, pua_char
        );

        core.insert_text_native(0, pi, 0, &text)
            .unwrap_or_else(|e| panic!("텍스트 입력 실패 (pi={}): {:?}", pi, e));
    }

    core.end_batch_native().expect("배치 종료 실패");

    // 저장
    let bytes = core.export_hwp_native().expect("HWP 내보내기 실패");
    let out_path = Path::new(output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(out_path, bytes).expect("파일 저장 실패");
    println!("저장 완료: {} ({} 종 PUA)", output, pua_set.len());
    println!();
    println!("다음 단계:");
    println!("  1. 한컴 2022 편집기에서 본 파일 열기 → PDF 출력 (정답지)");
    println!("  2. rhwp export-svg {} → SVG 출력 비교", output);
    println!("  3. 시각 비교로 매핑 정합 확정");
}

fn test_field_roundtrip(args: &[String]) {
    let input = args.first().map(|s| s.as_str()).unwrap_or("hwp_webctl/bsbc01_10_000.hwp");
    let output = args.get(1).map(|s| s.as_str()).unwrap_or("output/field_test.hwp");

    let data = std::fs::read(input).expect("파일 읽기 실패");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&data)
        .expect("문서 파싱 실패");

    // 1. 필드 목록 출력
    let fields = core.collect_all_fields();
    println!("=== 필드 목록 ({}개) ===", fields.len());
    for fi in &fields {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }

    // 2. 필드에 값 설정
    let test_data = [
        ("mbizNm", "청소년 자립지원사업"),
        ("newCtnuTxt", "계속"),
        ("chargerNm", "홍길동"),
        ("telno", "02-1234-5678"),
        ("sFisYear", "2026"),
        // 셀 필드
        ("bizPurps", "청소년 자립 역량 강화"),
        ("bizPrdTxt", "2026.01 ~ 2026.12"),
        ("insttNm", "시청 복지과"),
    ];

    println!("\n=== 필드 값 설정 ===");
    for (name, value) in &test_data {
        match core.set_field_value_by_name(name, value) {
            Ok(r) => println!("  ✓ {} = \"{}\" → {}", name, value, r),
            Err(e) => println!("  ✗ {} = \"{}\" → {}", name, value, e),
        }
    }

    // 3. 설정 후 확인
    println!("\n=== 설정 후 확인 ===");
    let fields2 = core.collect_all_fields();
    for fi in &fields2 {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }

    // 3.5 pi=0 문단 텍스트 직접 확인
    let para0 = &core.document().sections[0].paragraphs[0];

    // 4. 직렬화 → 저장
    let saved = core.export_hwp_native().expect("직렬화 실패");
    std::fs::write(output, &saved).expect("저장 실패");
    println!("\n저장: {} ({}바이트)", output, saved.len());

    // 5. 재로딩 → 필드 확인
    let mut core2 = rhwp::document_core::DocumentCore::from_bytes(&saved)
        .expect("재로딩 실패");
    let fields3 = core2.collect_all_fields();
    println!("\n=== 재로딩 후 확인 ===");
    for fi in &fields3 {
        let name = fi.field.field_name().unwrap_or("(이름없음)");
        println!("  {} = \"{}\"", name, fi.value);
    }
}

fn control_tag(c: &rhwp::model::control::Control) -> &'static str {
    use rhwp::model::control::Control;
    match c {
        Control::SectionDef(_) => "secd",
        Control::ColumnDef(_) => "cold",
        Control::Table(_) => "tbl",
        Control::Shape(_) => "shape",
        Control::Picture(_) => "pic",
        Control::Header(_) => "head",
        Control::Footer(_) => "foot",
        Control::Footnote(_) => "fn",
        Control::Endnote(_) => "en",
        Control::AutoNumber(_) => "atno",
        Control::NewNumber(_) => "nwno",
        Control::PageNumberPos(_) => "pgnp",
        Control::Bookmark(_) => "bokm",
        Control::Hyperlink(_) => "hlk",
        Control::Ruby(_) => "ruby",
        Control::CharOverlap(_) => "tcps",
        Control::PageHide(_) => "pghd",
        Control::HiddenComment(_) => "tcmt",
        Control::Equation(_) => "eqed",
        Control::Field(_) => "field",
        Control::Form(_) => "form",
        Control::Unknown(_) => "unknown",
    }
}

fn diff_table(
    diffs: &mut Vec<String>,
    ci: usize,
    a: &rhwp::model::table::Table,
    b: &rhwp::model::table::Table,
) {
    if a.row_count != b.row_count {
        diffs.push(format!("ctrl[{}] tbl rows: A={} vs B={}", ci, a.row_count, b.row_count));
    }
    if a.col_count != b.col_count {
        diffs.push(format!("ctrl[{}] tbl cols: A={} vs B={}", ci, a.col_count, b.col_count));
    }
    if a.page_break != b.page_break {
        diffs.push(format!("ctrl[{}] tbl page_break: A={:?} vs B={:?}", ci, a.page_break, b.page_break));
    }
    if a.repeat_header != b.repeat_header {
        diffs.push(format!("ctrl[{}] tbl repeat_header: A={} vs B={}", ci, a.repeat_header, b.repeat_header));
    }
    if a.cell_spacing != b.cell_spacing {
        diffs.push(format!("ctrl[{}] tbl cell_spacing: A={} vs B={}", ci, a.cell_spacing, b.cell_spacing));
    }
    if a.border_fill_id != b.border_fill_id {
        diffs.push(format!("ctrl[{}] tbl border_fill_id: A={} vs B={}", ci, a.border_fill_id, b.border_fill_id));
    }
    if a.outer_margin_left != b.outer_margin_left
        || a.outer_margin_right != b.outer_margin_right
        || a.outer_margin_top != b.outer_margin_top
        || a.outer_margin_bottom != b.outer_margin_bottom
    {
        diffs.push(format!(
            "ctrl[{}] tbl outer_margin: A=({},{},{},{}) vs B=({},{},{},{})",
            ci,
            a.outer_margin_left, a.outer_margin_top, a.outer_margin_right, a.outer_margin_bottom,
            b.outer_margin_left, b.outer_margin_top, b.outer_margin_right, b.outer_margin_bottom,
        ));
    }
    diff_common_obj(diffs, ci, "tbl", &a.common, &b.common);
}

fn diff_common_obj(
    diffs: &mut Vec<String>,
    ci: usize,
    tag: &str,
    a: &rhwp::model::shape::CommonObjAttr,
    b: &rhwp::model::shape::CommonObjAttr,
) {
    if a.treat_as_char != b.treat_as_char {
        diffs.push(format!("ctrl[{}] {} tac: A={} vs B={}", ci, tag, a.treat_as_char, b.treat_as_char));
    }
    if a.text_wrap != b.text_wrap {
        diffs.push(format!("ctrl[{}] {} wrap: A={:?} vs B={:?}", ci, tag, a.text_wrap, b.text_wrap));
    }
    if a.width != b.width || a.height != b.height {
        diffs.push(format!("ctrl[{}] {} size: A={}x{} vs B={}x{}", ci, tag, a.width, a.height, b.width, b.height));
    }
    if a.vertical_offset != b.vertical_offset {
        diffs.push(format!("ctrl[{}] {} v_offset: A={} vs B={}", ci, tag, a.vertical_offset, b.vertical_offset));
    }
    if a.horizontal_offset != b.horizontal_offset {
        diffs.push(format!("ctrl[{}] {} h_offset: A={} vs B={}", ci, tag, a.horizontal_offset, b.horizontal_offset));
    }
    if a.vert_rel_to != b.vert_rel_to {
        diffs.push(format!("ctrl[{}] {} vert_rel: A={:?} vs B={:?}", ci, tag, a.vert_rel_to, b.vert_rel_to));
    }
    if a.horz_rel_to != b.horz_rel_to {
        diffs.push(format!("ctrl[{}] {} horz_rel: A={:?} vs B={:?}", ci, tag, a.horz_rel_to, b.horz_rel_to));
    }
}

fn ir_diff(args: &[String]) {
    if args.len() < 2 {
        eprintln!("사용법: rhwp ir-diff <파일A> <파일B> [-s <구역>] [-p <문단>]");
        return;
    }

    let file_a = &args[0];
    let file_b = &args[1];
    let mut section_filter: Option<usize> = None;
    let mut para_filter: Option<usize> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-s" | "--section" if i + 1 < args.len() => {
                section_filter = args[i + 1].parse().ok();
                i += 2;
            }
            "-p" | "--para" if i + 1 < args.len() => {
                para_filter = args[i + 1].parse().ok();
                i += 2;
            }
            _ => { i += 1; }
        }
    }

    let data_a = match fs::read(file_a) {
        Ok(d) => d,
        Err(e) => { eprintln!("오류: {} 읽기 실패: {}", file_a, e); return; }
    };
    let data_b = match fs::read(file_b) {
        Ok(d) => d,
        Err(e) => { eprintln!("오류: {} 읽기 실패: {}", file_b, e); return; }
    };

    let doc_a = match rhwp::parser::parse_document(&data_a) {
        Ok(d) => d,
        Err(e) => { eprintln!("오류: {} 파싱 실패: {:?}", file_a, e); return; }
    };
    let doc_b = match rhwp::parser::parse_document(&data_b) {
        Ok(d) => d,
        Err(e) => { eprintln!("오류: {} 파싱 실패: {:?}", file_b, e); return; }
    };

    let name_a = Path::new(file_a).file_name().unwrap_or_default().to_string_lossy();
    let name_b = Path::new(file_b).file_name().unwrap_or_default().to_string_lossy();
    println!("=== IR 비교: {} vs {} ===", name_a, name_b);

    // 구역 수 비교
    if doc_a.sections.len() != doc_b.sections.len() {
        println!("[차이] 구역 수: A={} vs B={}", doc_a.sections.len(), doc_b.sections.len());
    }

    let sec_count = doc_a.sections.len().min(doc_b.sections.len());
    let mut total_diffs = 0u32;

    for sec_idx in 0..sec_count {
        if let Some(sf) = section_filter {
            if sec_idx != sf { continue; }
        }

        let sec_a = &doc_a.sections[sec_idx];
        let sec_b = &doc_b.sections[sec_idx];

        if sec_a.paragraphs.len() != sec_b.paragraphs.len() {
            println!("[차이] 구역 {}: 문단 수 A={} vs B={}", sec_idx, sec_a.paragraphs.len(), sec_b.paragraphs.len());
            total_diffs += 1;
        }

        let para_count = sec_a.paragraphs.len().min(sec_b.paragraphs.len());
        for pi in 0..para_count {
            if let Some(pf) = para_filter {
                if pi != pf { continue; }
            }

            let pa = &sec_a.paragraphs[pi];
            let pb = &sec_b.paragraphs[pi];
            let mut diffs: Vec<String> = Vec::new();

            // 텍스트 비교
            if pa.text != pb.text {
                diffs.push(format!("text: A={:?} vs B={:?}",
                    pa.text.chars().take(30).collect::<String>(),
                    pb.text.chars().take(30).collect::<String>()));
            }

            // char_count 비교
            if pa.char_count != pb.char_count {
                diffs.push(format!("cc: A={} vs B={}", pa.char_count, pb.char_count));
            }

            // char_offsets 비교
            if pa.char_offsets != pb.char_offsets {
                let len_a = pa.char_offsets.len();
                let len_b = pb.char_offsets.len();
                if len_a != len_b {
                    diffs.push(format!("char_offsets len: A={} vs B={}", len_a, len_b));
                } else {
                    let first_diff = pa.char_offsets.iter().zip(pb.char_offsets.iter())
                        .enumerate()
                        .find(|(_, (a, b))| a != b);
                    if let Some((idx, (a, b))) = first_diff {
                        diffs.push(format!("char_offsets[{}]: A={} vs B={}", idx, a, b));
                    }
                }
            }

            // para_shape_id 비교
            if pa.para_shape_id != pb.para_shape_id {
                diffs.push(format!("ps_id: A={} vs B={}", pa.para_shape_id, pb.para_shape_id));
            }

            // tab_extended 비교
            if pa.tab_extended.len() != pb.tab_extended.len() {
                diffs.push(format!("tab_ext count: A={} vs B={}", pa.tab_extended.len(), pb.tab_extended.len()));
            } else {
                for (ti, (ta, tb)) in pa.tab_extended.iter().zip(pb.tab_extended.iter()).enumerate() {
                    if ta != tb {
                        diffs.push(format!("tab_ext[{}]: A={:?} vs B={:?}", ti, ta, tb));
                        break;
                    }
                }
            }

            // LINE_SEG 비교
            if pa.line_segs.len() != pb.line_segs.len() {
                diffs.push(format!("line_segs count: A={} vs B={}", pa.line_segs.len(), pb.line_segs.len()));
            } else {
                for (li, (la, lb)) in pa.line_segs.iter().zip(pb.line_segs.iter()).enumerate() {
                    if la.text_start != lb.text_start {
                        diffs.push(format!("ls[{}].ts: A={} vs B={}", li, la.text_start, lb.text_start));
                    }
                    if la.vertical_pos != lb.vertical_pos {
                        diffs.push(format!("ls[{}].vpos: A={} vs B={}", li, la.vertical_pos, lb.vertical_pos));
                    }
                    if la.line_height != lb.line_height {
                        diffs.push(format!("ls[{}].lh: A={} vs B={}", li, la.line_height, lb.line_height));
                    }
                    if la.text_height != lb.text_height {
                        diffs.push(format!("ls[{}].th: A={} vs B={}", li, la.text_height, lb.text_height));
                    }
                    if la.baseline_distance != lb.baseline_distance {
                        diffs.push(format!("ls[{}].bl: A={} vs B={}", li, la.baseline_distance, lb.baseline_distance));
                    }
                    if la.line_spacing != lb.line_spacing {
                        diffs.push(format!("ls[{}].ls: A={} vs B={}", li, la.line_spacing, lb.line_spacing));
                    }
                    if la.column_start != lb.column_start {
                        diffs.push(format!("ls[{}].cs: A={} vs B={}", li, la.column_start, lb.column_start));
                    }
                    if la.segment_width != lb.segment_width {
                        diffs.push(format!("ls[{}].sw: A={} vs B={}", li, la.segment_width, lb.segment_width));
                    }
                }
            }

            // 컨트롤 식별 비교
            if pa.controls.len() != pb.controls.len() {
                diffs.push(format!("controls count: A={} vs B={}", pa.controls.len(), pb.controls.len()));
            }
            {
                use rhwp::model::control::Control;
                let ctrl_count = pa.controls.len().min(pb.controls.len());
                for ci in 0..ctrl_count {
                    let ca = &pa.controls[ci];
                    let cb = &pb.controls[ci];
                    match (ca, cb) {
                        (Control::Table(ta), Control::Table(tb)) => {
                            diff_table(&mut diffs, ci, ta, tb);
                        }
                        (Control::Picture(pic_a), Control::Picture(pic_b)) => {
                            diff_common_obj(&mut diffs, ci, "pic", &pic_a.common, &pic_b.common);
                        }
                        (Control::Shape(sa), Control::Shape(sb)) => {
                            diff_common_obj(&mut diffs, ci, "shape", sa.common(), sb.common());
                        }
                        _ if control_tag(ca) != control_tag(cb) => {
                            diffs.push(format!("ctrl[{}] type: A={} vs B={}", ci, control_tag(ca), control_tag(cb)));
                        }
                        _ => {}
                    }
                }
            }

            // char_shapes 비교
            if pa.char_shapes.len() != pb.char_shapes.len() {
                diffs.push(format!("char_shapes count: A={} vs B={}", pa.char_shapes.len(), pb.char_shapes.len()));
            } else {
                for (ci, (ca, cb)) in pa.char_shapes.iter().zip(pb.char_shapes.iter()).enumerate() {
                    if ca.start_pos != cb.start_pos {
                        diffs.push(format!("cs[{}].pos: A={} vs B={}", ci, ca.start_pos, cb.start_pos));
                        break;
                    }
                    if ca.char_shape_id != cb.char_shape_id {
                        diffs.push(format!("cs[{}].id: A={} vs B={}", ci, ca.char_shape_id, cb.char_shape_id));
                        break;
                    }
                }
            }

            if !diffs.is_empty() {
                let text_preview: String = pa.text.chars().take(30).collect();
                println!("\n--- 문단 {}.{} --- \"{}\"", sec_idx, pi, text_preview);
                for d in &diffs {
                    println!("  [차이] {}", d);
                }
                total_diffs += diffs.len() as u32;
            }
        }
    }

    // doc_info 비교: ParaShape
    {
        let ps_a = &doc_a.doc_info.para_shapes;
        let ps_b = &doc_b.doc_info.para_shapes;
        if ps_a.len() != ps_b.len() {
            println!("\n[차이] ParaShape 수: A={} vs B={}", ps_a.len(), ps_b.len());
            total_diffs += 1;
        }
        let ps_count = ps_a.len().min(ps_b.len());
        for i in 0..ps_count {
            let a = &ps_a[i]; let b = &ps_b[i];
            let mut ps_diffs: Vec<String> = Vec::new();
            if a.margin_left != b.margin_left { ps_diffs.push(format!("ml: {}vs{}", a.margin_left, b.margin_left)); }
            if a.margin_right != b.margin_right { ps_diffs.push(format!("mr: {}vs{}", a.margin_right, b.margin_right)); }
            if a.indent != b.indent { ps_diffs.push(format!("indent: {}vs{}", a.indent, b.indent)); }
            if a.tab_def_id != b.tab_def_id { ps_diffs.push(format!("tab_def: {}vs{}", a.tab_def_id, b.tab_def_id)); }
            if a.spacing_before != b.spacing_before { ps_diffs.push(format!("sb: {}vs{}", a.spacing_before, b.spacing_before)); }
            if a.spacing_after != b.spacing_after { ps_diffs.push(format!("sa: {}vs{}", a.spacing_after, b.spacing_after)); }
            if a.line_spacing != b.line_spacing { ps_diffs.push(format!("ls: {}vs{}", a.line_spacing, b.line_spacing)); }
            if !ps_diffs.is_empty() {
                println!("  [PS {}] {}", i, ps_diffs.join(", "));
                total_diffs += ps_diffs.len() as u32;
            }
        }
    }

    // doc_info 비교: TabDef
    {
        let td_a = &doc_a.doc_info.tab_defs;
        let td_b = &doc_b.doc_info.tab_defs;
        if td_a.len() != td_b.len() {
            println!("\n[차이] TabDef 수: A={} vs B={}", td_a.len(), td_b.len());
            total_diffs += 1;
        }
        let td_count = td_a.len().min(td_b.len());
        for i in 0..td_count {
            let a = &td_a[i]; let b = &td_b[i];
            if a.tabs.len() != b.tabs.len() {
                println!("  [TD {}] 탭 수: A={} vs B={}", i, a.tabs.len(), b.tabs.len());
                total_diffs += 1;
            } else {
                for (ti, (ta, tb)) in a.tabs.iter().zip(b.tabs.iter()).enumerate() {
                    if ta.position != tb.position || ta.tab_type != tb.tab_type || ta.fill_type != tb.fill_type {
                        println!("  [TD {}][{}] pos: {}vs{}, type: {}vs{}, fill: {}vs{}",
                            i, ti, ta.position, tb.position, ta.tab_type, tb.tab_type, ta.fill_type, tb.fill_type);
                        total_diffs += 1;
                    }
                }
            }
        }
    }

    println!("\n=== 비교 완료: 차이 {} 건 ===", total_diffs);
}

fn extract_thumbnail(args: &[String]) {
    if args.is_empty() {
        eprintln!("사용법: rhwp thumbnail <파일.hwp> [옵션]");
        eprintln!("  -o, --output <파일>   출력 파일 경로");
        eprintln!("  --base64              base64 문자열 출력");
        eprintln!("  --data-uri            data:image/... URI 출력");
        std::process::exit(1);
    }

    let input_path = &args[0];
    let mut output_path: Option<String> = None;
    let mut mode = "file"; // "file", "base64", "data-uri"

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                if i < args.len() {
                    output_path = Some(args[i].clone());
                }
            }
            "--base64" => mode = "base64",
            "--data-uri" => mode = "data-uri",
            _ => {}
        }
        i += 1;
    }

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다: {} ({})", input_path, e);
            std::process::exit(1);
        }
    };

    let result = match rhwp::parser::extract_thumbnail_only(&data) {
        Some(r) => r,
        None => {
            eprintln!("오류: PrvImage 썸네일이 없습니다: {}", input_path);
            std::process::exit(1);
        }
    };

    let mime = match result.format.as_str() {
        "png" => "image/png",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    };

    match mode {
        "base64" => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&result.data);
            println!("{}", b64);
        }
        "data-uri" => {
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&result.data);
            println!("data:{};base64,{}", mime, b64);
        }
        _ => {
            // 파일 출력
            let out = output_path.unwrap_or_else(|| {
                let stem = Path::new(input_path)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                let ext = &result.format;
                format!("output/{}_thumb.{}", stem, ext)
            });

            // 출력 디렉토리 생성
            if let Some(parent) = Path::new(&out).parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).ok();
                }
            }

            match fs::write(&out, &result.data) {
                Ok(_) => {
                    println!("썸네일 추출 완료: {} ({}x{}, {} bytes, {})",
                        out, result.width, result.height, result.data.len(), result.format);
                }
                Err(e) => {
                    eprintln!("오류: 파일 저장 실패: {} ({})", out, e);
                    std::process::exit(1);
                }
            }
        }
    }
}
