//! 문서를 변경하지 않지만 파일·stdout 산출물을 만드는 CLI output 어댑터.
//!
//! 읽기 전용 query와 달리 파일 시스템 부작용을 가질 수 있으므로 별도 경계에 둔다.

pub(crate) mod doclang;
pub(crate) mod kitty;
pub(crate) mod pdf;
pub(crate) mod preview;
pub(crate) mod raster;
pub(crate) mod sixel;
pub(crate) mod tabular;
pub(crate) mod text;
pub(crate) mod vector;

/// 파일 시스템 한 이름의 바이트 한도(ext4·APFS 255바이트, NTFS 255 UTF-16 단위 이하).
pub(crate) const MAX_OUTPUT_FILE_NAME_BYTES: usize = 255;

/// 원본 이름에서 만든 산출 파일·폴더 이름. `suffix`(쪽 번호·확장자 등)를 보존하고,
/// 전체가 [`MAX_OUTPUT_FILE_NAME_BYTES`] 를 넘으면 `stem` 을 UTF-8 문자 경계에서 줄인다.
/// 같은 입력이면 같은 이름이다(#9010).
pub(crate) fn output_file_name(stem: &str, suffix: &str) -> String {
    let budget = MAX_OUTPUT_FILE_NAME_BYTES.saturating_sub(suffix.len());
    let mut end = stem.len().min(budget);
    while !stem.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{}", &stem[..end], suffix)
}

pub(crate) fn allows_implicit_sibling_resources(format: rhwp::parser::FileFormat) -> bool {
    // HML sibling paths are untrusted input and require an explicit resolver policy.
    !matches!(format, rhwp::parser::FileFormat::Hml)
}

/// 사용자 환경을 우선하며, 없으면 실제 Native 공급에서 문서의 명시 대체를 고른다.
/// 쪽수·측정·paint를 읽기 전에 동일한 환경 세대를 확정한다.
#[derive(Default)]
pub(crate) struct OutputFontSupply {
    pub svg_style: String,
    pub report: Option<serde_json::Value>,
}

pub(crate) fn apply_font_environment(
    document: &mut rhwp::DocumentCore,
    environment: Option<rhwp::renderer::font_environment::FontEnvironment>,
    font_paths: &mut Vec<std::path::PathBuf>,
) -> Result<OutputFontSupply, rhwp::error::HwpError> {
    let mut output_supply = OutputFontSupply::default();
    let environment = match environment {
        Some(environment) => Some(environment),
        None => {
            #[cfg(not(target_arch = "wasm32"))]
            let supply =
                rhwp::renderer::native_webfonts::prepare(&document.document().doc_info, font_paths)
                    .map_err(rhwp::error::HwpError::RenderError)?;
            #[cfg(not(target_arch = "wasm32"))]
            {
                output_supply.svg_style = supply.svg_style();
                output_supply.report = Some(
                    serde_json::to_value(&supply)
                        .map_err(|e| rhwp::error::HwpError::RenderError(e.to_string()))?,
                );
                font_paths.extend(supply.paths.iter().cloned());
                if !supply.unavailable.is_empty() {
                    eprintln!(
                        "웹폰트를 준비하지 못한 글꼴: {}",
                        supply.unavailable.join(", ")
                    );
                }
            }
            let native = rhwp::renderer::font_environment::FontEnvironment::from_native_document_substitutions(
                &document.document().doc_info,
                font_paths,
            )?;
            #[cfg(not(target_arch = "wasm32"))]
            {
                supply.merge_into_environment(native)?
            }
            #[cfg(target_arch = "wasm32")]
            {
                native
            }
        }
    };
    document.set_font_environment(environment)?;
    Ok(output_supply)
}

/// Remove and validate the optional environment before command-specific parsing.
pub(crate) fn font_environment_args(
    args: &[String],
) -> Result<
    (
        Vec<String>,
        Option<rhwp::renderer::font_environment::FontEnvironment>,
    ),
    i32,
> {
    let mut rest = Vec::new();
    let mut environment = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] != "--font-environment" {
            rest.push(args[i].clone());
            i += 1;
            continue;
        }
        if environment.is_some() || i + 1 >= args.len() {
            eprintln!("오류: --font-environment 뒤에 JSON 파일을 한 번 지정하세요.");
            return Err(crate::EXIT_USAGE);
        }
        let json = std::fs::read_to_string(&args[i + 1]).map_err(|e| {
            eprintln!("오류: 폰트 환경 파일을 읽을 수 없습니다: {e}");
            crate::EXIT_USAGE
        })?;
        environment = Some(
            rhwp::renderer::font_environment::FontEnvironment::from_json(&json).map_err(|e| {
                eprintln!("오류: {e}");
                crate::EXIT_USAGE
            })?,
        );
        i += 2;
    }
    Ok((rest, environment))
}
