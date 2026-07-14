//! PDF 렌더러 (Task #21)
//!
//! SVG 렌더러의 출력을 svg2pdf + pdf-writer로 PDF를 생성한다.
//! 단일/다중 페이지 모두 지원. 네이티브 전용 (WASM 미지원).

/// PDF 내보내기 폰트 설정.
///
/// `export-pdf`는 SVG를 usvg/svg2pdf로 변환하므로 generic font family와 수식 SVG
/// font-family를 PDF 변환 직전에 조정한다.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfExportOptions {
    /// serif generic fallback family.
    pub fallback_serif: String,
    /// sans-serif generic fallback family.
    pub fallback_sans: String,
    /// monospace generic fallback family.
    pub fallback_mono: String,
    /// 사용자 지정 수식 우선 폰트. None이면 기존 수식 font-family 체인을 유지한다.
    pub equation_font: Option<String>,
    /// 사용자 지정 폰트 탐색 디렉토리. 기본 탐색 경로보다 먼저 로드한다.
    pub font_paths: Vec<std::path::PathBuf>,
    /// 텍스트를 PDF 폰트로 임베드할지 여부. `false` 면 글리프를 path 로 변환한다.
    ///
    /// [Task #2264] 임베드 경로(폰트 서브셋)가 PDF 변환 메모리의 지배항이다.
    /// 실측(텍스트 1639개·이미지 2개인 1페이지 기준): `svg2pdf::to_chunk` 최대 RSS 가
    /// 164 MB → 69 MB 로 떨어진다. 대신 **PDF 의 텍스트 선택·검색 기능을 잃는다**
    /// (시각적 출력은 동일). 기본값은 종전 동작인 `true` 다.
    pub embed_text: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for PdfExportOptions {
    fn default() -> Self {
        Self {
            fallback_serif: default_serif_family().to_string(),
            fallback_sans: default_sans_family().to_string(),
            fallback_mono: default_mono_family().to_string(),
            equation_font: None,
            font_paths: Vec::new(),
            embed_text: true,
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
fn default_serif_family() -> &'static str {
    "바탕"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
fn default_sans_family() -> &'static str {
    "맑은 고딕"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
fn default_mono_family() -> &'static str {
    "D2Coding"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
fn default_serif_family() -> &'static str {
    "Noto Serif CJK KR"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
fn default_sans_family() -> &'static str {
    "Noto Sans CJK KR"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
fn default_mono_family() -> &'static str {
    "Noto Sans Mono CJK KR"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
fn default_serif_family() -> &'static str {
    "AppleMyungjo"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
fn default_sans_family() -> &'static str {
    "Apple SD Gothic Neo"
}

#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
fn default_mono_family() -> &'static str {
    "Menlo"
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(any(target_os = "windows", target_os = "linux", target_os = "macos"))
))]
fn default_serif_family() -> &'static str {
    "Noto Serif CJK KR"
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(any(target_os = "windows", target_os = "linux", target_os = "macos"))
))]
fn default_sans_family() -> &'static str {
    "Noto Sans CJK KR"
}

#[cfg(all(
    not(target_arch = "wasm32"),
    not(any(target_os = "windows", target_os = "linux", target_os = "macos"))
))]
fn default_mono_family() -> &'static str {
    "Noto Sans Mono CJK KR"
}

/// 폰트 데이터베이스를 초기화 (시스템 폰트 + 프로젝트 폰트 로드)
#[cfg(not(target_arch = "wasm32"))]
fn create_fontdb(options: &PdfExportOptions) -> usvg::fontdb::Database {
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    for dir in &options.font_paths {
        if dir.exists() {
            fontdb.load_fonts_dir(dir);
        } else {
            eprintln!(
                "WARN: PDF font path '{}' not found. 해당 경로의 폰트는 로드하지 않습니다.",
                dir.display()
            );
        }
    }
    for dir in &["ttfs", "ttfs/windows", "ttfs/hwp"] {
        if std::path::Path::new(dir).exists() {
            fontdb.load_fonts_dir(dir);
        }
    }
    if std::path::Path::new("/mnt/c/Windows/Fonts").exists() {
        fontdb.load_fonts_dir("/mnt/c/Windows/Fonts");
    }
    fontdb.set_serif_family(options.fallback_serif.as_str());
    fontdb.set_sans_serif_family(options.fallback_sans.as_str());
    fontdb.set_monospace_family(options.fallback_mono.as_str());
    warn_missing_family(
        &fontdb,
        "serif",
        &options.fallback_serif,
        "--fallback-serif",
    );
    warn_missing_family(
        &fontdb,
        "sans-serif",
        &options.fallback_sans,
        "--fallback-sans",
    );
    warn_missing_family(
        &fontdb,
        "monospace",
        &options.fallback_mono,
        "--fallback-mono",
    );
    if let Some(equation_font) = options.equation_font.as_deref() {
        let family = first_font_family(equation_font);
        if !family.is_empty() {
            warn_missing_family(&fontdb, "equation", &family, "--equation-font");
        }
    }
    fontdb
}

#[cfg(not(target_arch = "wasm32"))]
fn warn_missing_family(
    fontdb: &usvg::fontdb::Database,
    kind: &str,
    family: &str,
    option_name: &str,
) {
    if !font_family_exists(fontdb, family) {
        eprintln!(
            "WARN: fallback {kind} font '{family}' not found.\n      한글 또는 수식이 빈칸으로 렌더링될 수 있습니다.\n      {option_name} \"<family>\" 로 설치된 폰트를 지정하세요."
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn font_family_exists(fontdb: &usvg::fontdb::Database, family: &str) -> bool {
    fontdb.faces().any(|face| {
        face.families
            .iter()
            .any(|(name, _)| name == family || name.eq_ignore_ascii_case(family))
    })
}

/// SVG에서 없는 한글 폰트명에 fallback 추가
#[cfg(not(target_arch = "wasm32"))]
fn add_font_fallbacks(svg: &str, options: &PdfExportOptions) -> String {
    let serif = css_family_for_attr(&options.fallback_serif);
    let sans = css_family_for_attr(&options.fallback_sans);
    svg.replace(
        "font-family=\"휴먼명조\"",
        &format!("font-family=\"휴먼명조, {serif}, serif\""),
    )
    .replace(
        "font-family=\"HCI Poppy\"",
        &format!("font-family=\"HCI Poppy, {sans}, sans-serif\""),
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_pdf_font_options(svg: &str, options: &PdfExportOptions) -> String {
    let svg = add_font_fallbacks(svg, options);
    if let Some(equation_font) = options.equation_font.as_deref() {
        let attr = format!(
            "font-family=\"{}\"",
            escape_xml_attr(&equation_font_chain(equation_font))
        );
        svg.replace(
            crate::renderer::equation::svg_render::DEFAULT_EQUATION_FONT_FAMILY_ATTR,
            &attr,
        )
    } else {
        svg
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn equation_font_chain(equation_font: &str) -> String {
    if equation_font.contains(',') {
        return equation_font.trim().to_string();
    }
    let first = css_family_for_attr(equation_font);
    let default =
        "'Latin Modern Math', 'STIX Two Text', 'STIX Two Math', 'Times New Roman', 'Times', serif";
    if first == "'Latin Modern Math'" {
        default.to_string()
    } else {
        format!("{first}, {default}")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn first_font_family(value: &str) -> String {
    value
        .split(',')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn css_family_for_attr(family: &str) -> String {
    let family = family.trim();
    if family.eq_ignore_ascii_case("serif")
        || family.eq_ignore_ascii_case("sans-serif")
        || family.eq_ignore_ascii_case("monospace")
    {
        return family.to_string();
    }
    let escaped = escape_xml_attr(family);
    format!("'{escaped}'")
}

#[cfg(not(target_arch = "wasm32"))]
fn escape_xml_attr(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// 단일 SVG를 PDF로 변환
#[cfg(not(target_arch = "wasm32"))]
pub fn svg_to_pdf(svg_content: &str) -> Result<Vec<u8>, String> {
    svgs_to_pdf(&[svg_content.to_string()])
}

/// 단일 SVG를 옵션 기반 PDF로 변환
#[cfg(not(target_arch = "wasm32"))]
pub fn svg_to_pdf_with_options(
    svg_content: &str,
    options: &PdfExportOptions,
) -> Result<Vec<u8>, String> {
    svgs_to_pdf_with_options(&[svg_content.to_string()], options)
}

/// 여러 SVG 페이지를 단일 다중 페이지 PDF로 생성
#[cfg(not(target_arch = "wasm32"))]
pub fn svgs_to_pdf(svg_pages: &[String]) -> Result<Vec<u8>, String> {
    svgs_to_pdf_with_options(svg_pages, &PdfExportOptions::default())
}

/// 여러 SVG 페이지를 옵션 기반 단일 다중 페이지 PDF로 생성
#[cfg(not(target_arch = "wasm32"))]
pub fn svgs_to_pdf_with_options(
    svg_pages: &[String],
    export_options: &PdfExportOptions,
) -> Result<Vec<u8>, String> {
    if svg_pages.is_empty() {
        return Err("페이지가 없습니다".to_string());
    }
    use pdf_writer::{Finish, Pdf, Ref};
    use std::collections::HashMap;

    let fontdb = create_fontdb(export_options);
    let mut options = usvg::Options::default();
    options.fontdb = std::sync::Arc::new(fontdb);

    let mut alloc = Ref::new(1);
    let catalog_ref = alloc.bump();
    let page_tree_ref = alloc.bump();

    // 각 페이지의 SVG를 파싱하여 chunk + page 정보 수집
    struct PageData {
        chunk: pdf_writer::Chunk,
        svg_ref: Ref,
        width: f32,
        height: f32,
    }

    let mut page_datas: Vec<PageData> = Vec::new();

    for svg in svg_pages {
        let svg_with_fallback = apply_pdf_font_options(svg, export_options);
        let tree = usvg::Tree::from_str(&svg_with_fallback, &options)
            .map_err(|e| format!("SVG 파싱 실패: {}", e))?;

        // [Task #2264] 텍스트 임베드(폰트 서브셋)가 PDF 변환 메모리의 지배항이다.
        // `embed_text=false` 면 글리프를 path 로 변환해 서브셋 경로를 통째로 건너뛴다.
        let mut conversion = svg2pdf::ConversionOptions::default();
        conversion.embed_text = export_options.embed_text;

        let (chunk, svg_ref) = svg2pdf::to_chunk(&tree, conversion)
            .map_err(|e| format!("SVG→chunk 변환 실패: {:?}", e))?;

        let dpi_ratio = 72.0 / 96.0; // 96 DPI → 72 pt
        let w = tree.size().width() * dpi_ratio;
        let h = tree.size().height() * dpi_ratio;

        page_datas.push(PageData {
            chunk,
            svg_ref,
            width: w,
            height: h,
        });
    }

    // 각 chunk를 재번호화하고 페이지 참조 수집
    let mut page_refs: Vec<Ref> = Vec::new();
    let mut renumbered_chunks: Vec<pdf_writer::Chunk> = Vec::new();
    let mut svg_refs_remapped: Vec<Ref> = Vec::new();

    for pd in &page_datas {
        let page_ref = alloc.bump();
        let content_ref = alloc.bump();
        page_refs.push(page_ref);

        // chunk 재번호화
        let mut map = HashMap::new();
        let renumbered = pd
            .chunk
            .renumber(|old| *map.entry(old).or_insert_with(|| alloc.bump()));

        let remapped_svg_ref = map.get(&pd.svg_ref).copied().unwrap_or(pd.svg_ref);
        svg_refs_remapped.push(remapped_svg_ref);
        renumbered_chunks.push(renumbered);
    }

    // PDF 생성
    let mut pdf = Pdf::new();
    pdf.catalog(catalog_ref).pages(page_tree_ref);
    pdf.pages(page_tree_ref)
        .count(page_refs.len() as i32)
        .kids(page_refs.iter().copied());

    // 각 페이지 생성
    let svg_name = pdf_writer::Name(b"S1");

    for (i, pd) in page_datas.iter().enumerate() {
        let page_ref = page_refs[i];
        let content_ref = alloc.bump();
        let svg_ref = svg_refs_remapped[i];

        let mut page = pdf.page(page_ref);
        page.media_box(pdf_writer::Rect::new(0.0, 0.0, pd.width, pd.height));
        page.parent(page_tree_ref);
        page.contents(content_ref);

        let mut resources = page.resources();
        resources.x_objects().pair(svg_name, svg_ref);
        resources.finish();
        page.finish();

        // 컨텐츠 스트림: SVG XObject를 페이지 크기에 맞게 배치
        let mut content = pdf_writer::Content::new();
        content.transform([pd.width, 0.0, 0.0, pd.height, 0.0, 0.0]);
        content.x_object(svg_name);

        pdf.stream(content_ref, &content.finish());
    }

    // 모든 chunk를 PDF에 추가
    for chunk in &renumbered_chunks {
        pdf.extend(chunk);
    }

    // 문서 정보
    let info_ref = alloc.bump();
    pdf.document_info(info_ref)
        .producer(pdf_writer::TextStr("rhwp"));

    Ok(pdf.finish())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn default_pdf_font_options_are_os_specific_and_non_empty() {
        let options = PdfExportOptions::default();
        assert!(!options.fallback_serif.is_empty());
        assert!(!options.fallback_sans.is_empty());
        assert!(!options.fallback_mono.is_empty());
        assert!(options.equation_font.is_none());
    }

    #[test]
    fn pdf_font_options_replace_generic_fallbacks_and_equation_font() {
        let options = PdfExportOptions {
            fallback_serif: "Noto Serif CJK KR".to_string(),
            fallback_sans: "Noto Sans CJK KR".to_string(),
            fallback_mono: "Noto Sans Mono CJK KR".to_string(),
            equation_font: Some("STIX Two Math".to_string()),
            font_paths: Vec::new(),
            embed_text: true,
        };
        let svg = format!(
            r#"<svg><text font-family="휴먼명조">가</text><text font-family="HCI Poppy">A</text><text {}>x</text></svg>"#,
            crate::renderer::equation::svg_render::DEFAULT_EQUATION_FONT_FAMILY_ATTR
        );

        let out = apply_pdf_font_options(&svg, &options);

        assert!(out.contains(r#"font-family="휴먼명조, 'Noto Serif CJK KR', serif""#));
        assert!(out.contains(r#"font-family="HCI Poppy, 'Noto Sans CJK KR', sans-serif""#));
        assert!(out
            .contains(r#"font-family="&apos;STIX Two Math&apos;, &apos;Latin Modern Math&apos;"#));
    }

    #[test]
    fn equation_font_accepts_full_family_chain() {
        let chain = equation_font_chain("'Custom Math', 'Fallback Math', serif");
        assert_eq!(chain, "'Custom Math', 'Fallback Math', serif");
    }
}
