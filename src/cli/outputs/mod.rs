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
