//! 문서를 변경하지 않지만 파일·stdout 산출물을 만드는 CLI output 어댑터.
//!
//! 읽기 전용 query와 달리 파일 시스템 부작용을 가질 수 있으므로 별도 경계에 둔다.

pub(crate) mod doclang;
pub(crate) mod pdf;
pub(crate) mod preview;
pub(crate) mod raster;
pub(crate) mod tabular;
pub(crate) mod text;
pub(crate) mod vector;

pub(crate) fn allows_implicit_sibling_resources(format: rhwp::parser::FileFormat) -> bool {
    // HML sibling paths are untrusted input and require an explicit resolver policy.
    !matches!(format, rhwp::parser::FileFormat::Hml)
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
