//! Sixel stdout 또는 페이지별 파일: stdout 그래픽과 stderr 진단을 분리한다.

use crate::EXIT_USAGE;
#[cfg(feature = "native-skia")]
use crate::{EXIT_OK, EXIT_RUNTIME};

#[cfg(not(feature = "native-skia"))]
pub(crate) fn export_sixel(_args: &[String]) -> i32 {
    eprintln!("오류: export-sixel 명령은 native-skia feature 가 활성화되어야 합니다.");
    eprintln!("       cargo build --release --features native-skia");
    EXIT_USAGE
}

#[cfg(feature = "native-skia")]
pub(crate) fn export_sixel(args: &[String]) -> i32 {
    use rhwp::paint::RenderProfile;
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    let mut file = None;
    let mut target_page = None;
    let mut output_dir = None;
    let mut width = 360u32;
    let mut fonts = Vec::<PathBuf>::new();
    let mut profile = RenderProfile::Screen;
    let mut compat = false;
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "--page" | "-p" | "--output" | "-o" | "--width" | "--font-path" | "--profile"
            | "--compat" => {
                let Some(value) = args.get(index + 1) else {
                    eprintln!("오류: {arg} 뒤에 값이 필요합니다.");
                    return EXIT_USAGE;
                };
                match arg {
                    "--page" | "-p" => match value.parse() {
                        Ok(number) => target_page = Some(number),
                        Err(_) => {
                            eprintln!("오류: 페이지 번호는 0 이상의 정수여야 합니다.");
                            return EXIT_USAGE;
                        }
                    },
                    "--output" | "-o" => output_dir = Some(PathBuf::from(value)),
                    "--width" => match value.parse::<u32>() {
                        Ok(number)
                            if number > 0
                                && number
                                    <= RasterRenderOptions::default().max_dimension as u32 =>
                        {
                            width = number
                        }
                        _ => {
                            eprintln!("오류: --width는 1..16384 범위의 픽셀 너비여야 합니다.");
                            return EXIT_USAGE;
                        }
                    },
                    "--font-path" => fonts.push(PathBuf::from(value)),
                    "--profile" => match RenderProfile::parse(value) {
                        Some(selected) => profile = selected,
                        None => {
                            eprintln!("오류: --profile 값은 screen|print|high-quality|fast-preview 입니다.");
                            return EXIT_USAGE;
                        }
                    },
                    "--compat" => match crate::cli::parse_compat_generation(value) {
                        Some(selected) => compat = selected,
                        None => {
                            eprintln!("오류: --compat 값은 2022|2024 입니다.");
                            return EXIT_USAGE;
                        }
                    },
                    _ => unreachable!(),
                }
                index += 2;
            }
            flag if flag.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션: {flag}");
                return EXIT_USAGE;
            }
            path => {
                if file.replace(path).is_some() {
                    eprintln!("오류: 입력 파일은 한 개만 지정하세요.");
                    return EXIT_USAGE;
                }
                index += 1;
            }
        }
    }
    let Some(file) = file else {
        eprintln!("사용법: rhwp export-sixel <파일.hwp|파일.hwpx> [-p 0] [--width 360] [-o 폴더]");
        return EXIT_USAGE;
    };
    let data = match std::fs::read(file) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {file}: {error}");
            return EXIT_RUNTIME;
        }
    };
    let mut core = match crate::load_document_core(&data) {
        Ok(core) => core,
        Err(error) => return error.report(),
    };
    core.set_hangul2024_compat(compat);
    if super::allows_implicit_sibling_resources(rhwp::parser::detect_format(&data)) {
        if let Some(parent) = Path::new(file).parent() {
            core.populate_external_images_from_dir(parent);
        }
    }
    let page_count = core.page_count();
    if page_count == 0 || target_page.is_some_and(|page| page >= page_count) {
        eprintln!(
            "오류: 페이지 번호 {}가 범위를 벗어났습니다 (전체 {page_count}쪽, 0부터 시작).",
            target_page.unwrap_or(0)
        );
        return EXIT_USAGE;
    }
    let render = |page| {
        core.render_page_pixels_native_at_width(page, width, profile, &fonts)
            .and_then(|pixels| rhwp::renderer::sixel::encode_sixel(&pixels))
    };
    if let Some(directory) = output_dir {
        if let Err(error) = std::fs::create_dir_all(&directory) {
            eprintln!("오류: Sixel 출력 폴더 생성 실패: {error}");
            return EXIT_RUNTIME;
        }
        let pages = match target_page {
            Some(page) => page..page + 1,
            None => 0..page_count,
        };
        let single_page = target_page.is_some() || page_count == 1;
        let stem = Path::new(file)
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("output");
        for page in pages {
            let bytes = match render(page) {
                Ok(bytes) => bytes,
                Err(error) => {
                    eprintln!("오류: {}쪽 Sixel 내보내기 실패: {error}", page + 1);
                    return EXIT_RUNTIME;
                }
            };
            let name = if single_page {
                format!("{stem}.six")
            } else {
                format!("{stem}_{:03}.six", page + 1)
            };
            let path = directory.join(name);
            if let Err(error) = std::fs::write(&path, bytes) {
                eprintln!("오류: {} 저장 실패: {error}", path.display());
                return EXIT_RUNTIME;
            }
            eprintln!("저장: {}", path.display());
        }
        return EXIT_OK;
    }
    let bytes = match render(target_page.unwrap_or(0)) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("오류: Sixel 내보내기 실패: {error}");
            return EXIT_RUNTIME;
        }
    };
    let mut stdout = std::io::stdout().lock();
    if let Err(error) = stdout.write_all(&bytes).and_then(|()| stdout.flush()) {
        eprintln!("오류: Sixel stdout 출력 실패: {error}");
        return EXIT_RUNTIME;
    }
    EXIT_OK
}
