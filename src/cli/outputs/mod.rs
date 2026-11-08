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
                if supply.mappings.is_empty() && supply.glyph_fallbacks.is_empty() {
                    native
                } else {
                    let mut value = serde_json::to_value(native)
                        .map_err(|e| rhwp::error::HwpError::RenderError(e.to_string()))?;
                    if value.is_null() {
                        value = serde_json::json!({"id":"native-webfont-supply", "substitutions":{}, "preserveSourceMetrics":true});
                    }
                    let object = value.as_object_mut().ok_or_else(|| {
                        rhwp::error::HwpError::RenderError("글꼴 환경이 올바르지 않습니다".into())
                    })?;
                    let mappings = object
                        .entry("substitutions")
                        .or_insert_with(|| serde_json::json!({}))
                        .as_object_mut()
                        .ok_or_else(|| {
                            rhwp::error::HwpError::RenderError(
                                "글꼴 치환이 올바르지 않습니다".into(),
                            )
                        })?;
                    for (source, target) in &supply.mappings {
                        mappings.insert(source.clone(), target.clone().into());
                    }
                    if let Some(programs) = object
                        .get_mut("selectedPrograms")
                        .and_then(|v| v.as_object_mut())
                    {
                        for source in supply.mappings.keys() {
                            programs.remove(source);
                        }
                    }
                    if let Some(entries) = object
                        .get_mut("documentSubstitutions")
                        .and_then(|v| v.as_array_mut())
                    {
                        entries.retain(|entry| {
                            !entry["source"]
                                .as_str()
                                .is_some_and(|source| supply.mappings.contains_key(source))
                        });
                    }
                    if !supply.glyph_fallbacks.is_empty() {
                        object.insert(
                            "documentGlyphFallbacks".into(),
                            supply.glyph_fallbacks.into(),
                        );
                    }
                    use sha2::{Digest, Sha256};
                    let identity = serde_json::to_vec(&(&value, &supply.resources))
                        .map_err(|e| rhwp::error::HwpError::RenderError(e.to_string()))?;
                    let digest: String = Sha256::digest(identity)
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect();
                    value["id"] = format!("native-webfont-{}", &digest[..24]).into();
                    Some(
                        rhwp::renderer::font_environment::FontEnvironment::from_json(
                            &value.to_string(),
                        )?,
                    )
                }
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
