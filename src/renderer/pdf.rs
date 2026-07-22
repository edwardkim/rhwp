//! PDF renderers.
//!
//! The compatibility backend converts SVG output with svg2pdf/pdf-writer. The
//! opt-in direct backend records PageLayerTree replay into a Skia PDF canvas.
//! Both backends support single and multiple pages and are native-only.

/// Native PDF implementation selected by callers such as `export-pdf`.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PdfBackend {
    /// Existing PageRenderTree/layered SVG -> svg2pdf compatibility path.
    #[default]
    CompatibilitySvg,
    /// PageLayerTree -> Skia PDF recording path. Requires `native-skia`.
    DirectLayer,
}

#[cfg(not(target_arch = "wasm32"))]
impl PdfBackend {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "svg" | "compat" | "compat-svg" | "compatibility" => Some(Self::CompatibilitySvg),
            "direct" | "direct-layer" | "layer" | "layer-skia" | "skia" => Some(Self::DirectLayer),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::CompatibilitySvg => "svg",
            Self::DirectLayer => "direct",
        }
    }
}

/// Options specific to direct PageLayerTree PDF recording.
#[cfg(not(target_arch = "wasm32"))]
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct DirectPdfExportOptions {
    /// Font directories loaded into the native Skia font manager.
    pub font_paths: Vec<std::path::PathBuf>,
    /// Raster resolution for effects that the PDF canvas cannot record as vectors.
    pub raster_dpi: f32,
    /// Optional deterministic document metadata. Dates are intentionally omitted.
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for DirectPdfExportOptions {
    fn default() -> Self {
        Self {
            font_paths: Vec::new(),
            raster_dpi: 144.0,
            title: None,
            author: None,
            subject: None,
            keywords: None,
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
fn validate_direct_pdf_tree(
    tree: &crate::paint::PageLayerTree,
    page_index: usize,
) -> Result<(), String> {
    use crate::model::image::ImageEffect;
    use crate::paint::{LayerNode, LayerNodeKind, PaintOp};
    use crate::renderer::render_tree::{BoundingBox, ShapeTransform};
    use crate::renderer::{ArrowStyle, LineRenderType, PathCommand, ShapeStyle};

    fn is_valid_skia_scalar(value: f64) -> bool {
        value.is_finite() && value.abs() <= f32::MAX as f64
    }

    fn validate_bbox(bbox: BoundingBox, page_index: usize, context: &str) -> Result<(), String> {
        if [bbox.x, bbox.y, bbox.width, bbox.height]
            .iter()
            .any(|value| !is_valid_skia_scalar(*value))
            || bbox.width < 0.0
            || bbox.height < 0.0
        {
            return Err(format!(
                "direct PDF page {} has invalid {context} bounds",
                page_index + 1
            ));
        }
        Ok(())
    }

    fn unsupported(page_index: usize, op: &str, detail: &str) -> String {
        format!(
            "direct PDF page {} cannot replay {op} without visual loss: {detail}; use the svg backend",
            page_index + 1
        )
    }

    fn validate_transform(
        transform: ShapeTransform,
        page_index: usize,
        op: &str,
    ) -> Result<(), String> {
        if !is_valid_skia_scalar(transform.rotation) {
            return Err(format!(
                "direct PDF page {} has invalid {op} rotation",
                page_index + 1
            ));
        }
        Ok(())
    }

    fn validate_shape_style(
        style: &ShapeStyle,
        has_gradient: bool,
        page_index: usize,
        op: &str,
    ) -> Result<(), String> {
        if has_gradient {
            return Err(unsupported(page_index, op, "gradient fill"));
        }
        if style.pattern.is_some() {
            return Err(unsupported(page_index, op, "pattern fill"));
        }
        if style.shadow.is_some() {
            return Err(unsupported(page_index, op, "shape shadow"));
        }
        if !is_valid_skia_scalar(style.opacity) || !(0.0..=1.0).contains(&style.opacity) {
            return Err(format!(
                "direct PDF page {} has invalid {op} opacity: {}",
                page_index + 1,
                style.opacity
            ));
        }
        if !is_valid_skia_scalar(style.stroke_width) || style.stroke_width < 0.0 {
            return Err(format!(
                "direct PDF page {} has invalid {op} stroke width: {}",
                page_index + 1,
                style.stroke_width
            ));
        }
        Ok(())
    }

    fn validate_image_payload(data: &[u8], page_index: usize, context: &str) -> Result<(), String> {
        let format = match crate::renderer::image_resolver::detect_image_mime_type(data) {
            "image/png" => image::ImageFormat::Png,
            "image/jpeg" => image::ImageFormat::Jpeg,
            "image/gif" => image::ImageFormat::Gif,
            "image/bmp" => image::ImageFormat::Bmp,
            "image/tiff" => image::ImageFormat::Tiff,
            format => {
                return Err(format!(
                    "direct PDF page {} does not support {context} payload format {format}; use the svg backend",
                    page_index + 1
                ));
            }
        };
        image::load_from_memory_with_format(data, format)
            .map(|_| ())
            .map_err(|error| {
                format!(
                    "direct PDF page {} cannot decode {context} payload: {error}",
                    page_index + 1
                )
            })
    }

    fn validate_node(node: &LayerNode, page_index: usize) -> Result<(), String> {
        validate_bbox(node.bounds, page_index, "layer")?;
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    validate_node(child, page_index)?;
                }
            }
            LayerNodeKind::ClipRect { clip, child, .. } => {
                validate_bbox(*clip, page_index, "clip")?;
                validate_node(child, page_index)?;
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    validate_bbox(op.bounds(), page_index, "paint operation")?;
                    match op {
                        PaintOp::PageBackground { background, .. } => {
                            if background.gradient.is_some() {
                                return Err(unsupported(
                                    page_index,
                                    "page background",
                                    "gradient fill",
                                ));
                            }
                            if !is_valid_skia_scalar(background.border_width)
                                || background.border_width < 0.0
                            {
                                return Err(format!(
                                    "direct PDF page {} has invalid page background border width",
                                    page_index + 1
                                ));
                            }
                            if let Some(image) = &background.image {
                                validate_image_payload(
                                    &image.data,
                                    page_index,
                                    "page background image",
                                )?;
                                if image.brightness != 0 || image.contrast != 0 {
                                    return Err(unsupported(
                                        page_index,
                                        "page background image",
                                        "unbaked brightness or contrast",
                                    ));
                                }
                                if image.effect == ImageEffect::Pattern8x8 {
                                    return Err(unsupported(
                                        page_index,
                                        "page background image",
                                        "Pattern8x8 effect",
                                    ));
                                }
                            }
                        }
                        PaintOp::Line { line, .. } => {
                            validate_transform(line.transform, page_index, "line")?;
                            if line.style.line_type != LineRenderType::Single
                                || line.style.start_arrow != ArrowStyle::None
                                || line.style.end_arrow != ArrowStyle::None
                                || line.style.shadow.is_some()
                            {
                                return Err(unsupported(
                                    page_index,
                                    "line",
                                    "multi-line, arrow, or shadow style",
                                ));
                            }
                            if [line.x1, line.y1, line.x2, line.y2, line.style.width]
                                .iter()
                                .any(|value| !is_valid_skia_scalar(*value))
                                || line.style.width < 0.0
                            {
                                return Err(format!(
                                    "direct PDF page {} has invalid line geometry",
                                    page_index + 1
                                ));
                            }
                        }
                        PaintOp::Rectangle { rect, .. } => {
                            validate_transform(rect.transform, page_index, "rectangle")?;
                            if !is_valid_skia_scalar(rect.corner_radius) || rect.corner_radius < 0.0
                            {
                                return Err(format!(
                                    "direct PDF page {} has invalid rectangle corner radius",
                                    page_index + 1
                                ));
                            }
                            validate_shape_style(
                                &rect.style,
                                rect.gradient.is_some(),
                                page_index,
                                "rectangle",
                            )?;
                        }
                        PaintOp::Ellipse { ellipse, .. } => {
                            validate_transform(ellipse.transform, page_index, "ellipse")?;
                            validate_shape_style(
                                &ellipse.style,
                                ellipse.gradient.is_some(),
                                page_index,
                                "ellipse",
                            )?;
                        }
                        PaintOp::Path { path, .. } => {
                            validate_transform(path.transform, page_index, "path")?;
                            validate_shape_style(
                                &path.style,
                                path.gradient.is_some(),
                                page_index,
                                "path",
                            )?;
                            if path.connector_endpoints.is_some() || path.line_style.is_some() {
                                return Err(unsupported(
                                    page_index,
                                    "path",
                                    "connector line style",
                                ));
                            }
                            let finite = path.commands.iter().all(|command| match *command {
                                PathCommand::MoveTo(x, y) | PathCommand::LineTo(x, y) => {
                                    is_valid_skia_scalar(x) && is_valid_skia_scalar(y)
                                }
                                PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                                    [x1, y1, x2, y2, x, y]
                                        .iter()
                                        .all(|value| is_valid_skia_scalar(*value))
                                }
                                PathCommand::ArcTo(rx, ry, rotation, _, _, x, y) => {
                                    [rx, ry, rotation, x, y]
                                        .iter()
                                        .all(|value| is_valid_skia_scalar(*value))
                                }
                                PathCommand::ClosePath => true,
                            });
                            if !finite {
                                return Err(format!(
                                    "direct PDF page {} has invalid path geometry",
                                    page_index + 1
                                ));
                            }
                        }
                        PaintOp::Image {
                            image, resolved, ..
                        } => {
                            validate_transform(image.transform, page_index, "image")?;
                            if let Some(data) = resolved
                                .as_deref()
                                .map(|payload| payload.data.as_slice())
                                .or(image.data.as_deref())
                            {
                                validate_image_payload(data, page_index, "image")?;
                            }
                            let effects_are_baked = resolved
                                .as_deref()
                                .is_some_and(|payload| payload.suppress_effects);
                            if !effects_are_baked && (image.brightness != 0 || image.contrast != 0)
                            {
                                return Err(unsupported(
                                    page_index,
                                    "image",
                                    "unbaked brightness or contrast",
                                ));
                            }
                            if !effects_are_baked && image.effect == ImageEffect::Pattern8x8 {
                                return Err(unsupported(page_index, "image", "Pattern8x8 effect"));
                            }
                            if !is_valid_skia_scalar(image.opacity)
                                || !(0.0..=1.0).contains(&image.opacity)
                            {
                                return Err(format!(
                                    "direct PDF page {} has invalid image opacity: {}",
                                    page_index + 1,
                                    image.opacity
                                ));
                            }
                        }
                        PaintOp::TextRun { run, .. } => {
                            if [
                                run.baseline,
                                run.rotation,
                                run.style.font_size,
                                run.style.letter_spacing,
                                run.style.ratio,
                            ]
                            .iter()
                            .any(|value| !is_valid_skia_scalar(*value))
                            {
                                return Err(format!(
                                    "direct PDF page {} has invalid text geometry",
                                    page_index + 1
                                ));
                            }
                        }
                        PaintOp::FootnoteMarker { marker, .. } => {
                            if !is_valid_skia_scalar(marker.base_font_size) {
                                return Err(format!(
                                    "direct PDF page {} has invalid footnote marker font size",
                                    page_index + 1
                                ));
                            }
                        }
                        PaintOp::Equation { equation, .. } => {
                            if !is_valid_skia_scalar(equation.font_size)
                                || !is_valid_skia_scalar(equation.layout_box.width)
                                || !is_valid_skia_scalar(equation.layout_box.height)
                            {
                                return Err(format!(
                                    "direct PDF page {} has invalid equation geometry",
                                    page_index + 1
                                ));
                            }
                        }
                        PaintOp::GlyphRun { .. }
                        | PaintOp::GlyphOutline { .. }
                        | PaintOp::CharOverlap { .. }
                        | PaintOp::TextControlMark { .. }
                        | PaintOp::TabLeader { .. }
                        | PaintOp::TextDecoration { .. }
                        | PaintOp::FormObject { .. }
                        | PaintOp::Placeholder { .. }
                        | PaintOp::RawSvg { .. } => {}
                    }
                }
            }
        }
        Ok(())
    }

    validate_node(&tree.root, page_index)
}

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
    // [#2864] 조달 순서는 renderer::font_paths 가 단일 정의한다.
    // 종전의 ttfs/hwp·ttfs/windows(로컬 전용)와 /mnt/c/Windows/Fonts(WSL2 전용)는
    // 제거했다 — 서버·컨테이너에서 무의미하고 /mnt/c 는 #2268 간헐 행의 원인이었다.
    crate::renderer::font_paths::load_into_fontdb(&mut fontdb, &options.font_paths);
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

/// Record PageLayerTree pages directly into a native Skia PDF document.
#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
pub fn layer_trees_to_pdf(layer_trees: &[crate::paint::PageLayerTree]) -> Result<Vec<u8>, String> {
    layer_trees_to_pdf_with_options(layer_trees, &DirectPdfExportOptions::default())
}

/// Record PageLayerTree pages directly into a native Skia PDF document.
///
/// PageLayerTree coordinates are CSS pixels at 96 DPI. PDF page coordinates
/// are points at 72 DPI, so the page and canvas are scaled by 72/96. Skia keeps
/// text, paths, and supported effects as vector commands and uses `raster_dpi`
/// for effects without a native PDF representation.
#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
pub fn layer_trees_to_pdf_with_options(
    layer_trees: &[crate::paint::PageLayerTree],
    options: &DirectPdfExportOptions,
) -> Result<Vec<u8>, String> {
    const CSS_PX_TO_PDF_POINT: f64 = 72.0 / 96.0;
    const MAX_PDF_PAGE_DIMENSION_POINTS: f64 = 14_400.0;

    if layer_trees.is_empty() {
        return Err("direct PDF export requires at least one page".to_string());
    }
    if !options.raster_dpi.is_finite() || options.raster_dpi <= 0.0 {
        return Err(format!(
            "invalid direct PDF raster dpi: {}",
            options.raster_dpi
        ));
    }

    let page_dimension = |value: f64, label: &str| -> Result<f32, String> {
        let points = value * CSS_PX_TO_PDF_POINT;
        if !points.is_finite() || points <= 0.0 {
            return Err(format!("invalid direct PDF page {label}: {value}"));
        }
        if points > MAX_PDF_PAGE_DIMENSION_POINTS {
            return Err(format!(
                "direct PDF page {label} exceeds {MAX_PDF_PAGE_DIMENSION_POINTS} points: {points}"
            ));
        }
        Ok(points as f32)
    };

    let page_sizes = layer_trees
        .iter()
        .enumerate()
        .map(|(page_index, tree)| {
            validate_direct_pdf_tree(tree, page_index)?;
            Ok((
                page_dimension(tree.page_width, "width")?,
                page_dimension(tree.page_height, "height")?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;

    let metadata = skia_safe::pdf::Metadata {
        title: options.title.clone().unwrap_or_default(),
        author: options.author.clone().unwrap_or_default(),
        subject: options.subject.clone().unwrap_or_default(),
        keywords: options.keywords.clone().unwrap_or_default(),
        creator: "rhwp".to_string(),
        producer: "rhwp PageLayerTree direct PDF (Skia)".to_string(),
        raster_dpi: Some(options.raster_dpi),
        ..Default::default()
    };
    let renderer = crate::renderer::skia::SkiaLayerRenderer::new()
        .with_font_paths(options.font_paths.as_slice());
    let mut output = Vec::new();

    {
        let mut document = skia_safe::pdf::new_document(&mut output, Some(&metadata));
        for (tree, &(width, height)) in layer_trees.iter().zip(page_sizes.iter()) {
            let mut page = document.begin_page((width, height), None);
            let canvas = page.canvas();
            canvas.clear(skia_safe::Color::WHITE);
            canvas.scale((CSS_PX_TO_PDF_POINT as f32, CSS_PX_TO_PDF_POINT as f32));
            renderer
                .render_page_to_canvas_strict(canvas, tree, options.raster_dpi / 96.0)
                .map_err(|error| format!("direct PDF page replay failed: {error}"))?;
            document = page.end_page();
        }
        document.close();
    }

    if !output.starts_with(b"%PDF-") || !output.windows(5).any(|window| window == b"%%EOF") {
        return Err("Skia direct PDF writer returned an incomplete document".to_string());
    }
    Ok(output)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn pdf_backend_aliases_are_explicit() {
        assert_eq!(PdfBackend::default(), PdfBackend::CompatibilitySvg);
        assert_eq!(PdfBackend::parse("svg"), Some(PdfBackend::CompatibilitySvg));
        assert_eq!(
            PdfBackend::parse("compat"),
            Some(PdfBackend::CompatibilitySvg)
        );
        assert_eq!(PdfBackend::parse("direct"), Some(PdfBackend::DirectLayer));
        assert_eq!(PdfBackend::parse("skia"), Some(PdfBackend::DirectLayer));
        assert_eq!(PdfBackend::parse("unknown"), None);
        assert_eq!(PdfBackend::CompatibilitySvg.as_str(), "svg");
        assert_eq!(PdfBackend::DirectLayer.as_str(), "direct");
    }

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

    #[cfg(feature = "native-skia")]
    fn direct_pdf_test_tree(
        width: f64,
        height: f64,
        fill_color: crate::model::ColorRef,
    ) -> crate::paint::PageLayerTree {
        use crate::paint::{LayerNode, PaintOp, RenderProfile};
        use crate::renderer::render_tree::{BoundingBox, RectangleNode};
        use crate::renderer::ShapeStyle;

        let page_bounds = BoundingBox::new(0.0, 0.0, width, height);
        let rectangle = RectangleNode::new(
            0.0,
            ShapeStyle {
                fill_color: Some(fill_color),
                ..Default::default()
            },
            None,
        );
        crate::paint::PageLayerTree::with_profile(
            width,
            height,
            LayerNode::leaf(
                page_bounds,
                None,
                vec![PaintOp::rectangle(
                    BoundingBox::new(8.0, 8.0, width - 16.0, height - 16.0),
                    rectangle,
                )],
            ),
            RenderProfile::Print,
        )
    }

    #[cfg(feature = "native-skia")]
    #[test]
    fn skia_direct_pdf_records_multiple_layer_pages() {
        let pages = [
            direct_pdf_test_tree(96.0, 144.0, 0x000000ff),
            direct_pdf_test_tree(192.0, 96.0, 0x0000ff00),
        ];
        let options = DirectPdfExportOptions {
            title: Some("P37 direct PDF test".to_string()),
            ..Default::default()
        };

        let pdf = layer_trees_to_pdf_with_options(&pages, &options).unwrap();

        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.windows(5).any(|window| window == b"%%EOF"));
        assert!(pdf
            .windows(b"P37 direct PDF test".len())
            .any(|window| window == b"P37 direct PDF test"));
        assert!(pdf
            .windows(b"/Count 2".len())
            .any(|window| window == b"/Count 2"));
        assert!(pdf
            .windows(b"/MediaBox [0 0 72 108]".len())
            .any(|window| window == b"/MediaBox [0 0 72 108]"));
        assert!(pdf
            .windows(b"/MediaBox [0 0 144 72]".len())
            .any(|window| window == b"/MediaBox [0 0 144 72]"));
        assert!(!pdf
            .windows(b"/Subtype /Image".len())
            .any(|window| window == b"/Subtype /Image"));
    }

    #[cfg(feature = "native-skia")]
    #[test]
    fn skia_direct_pdf_rejects_invalid_export_inputs() {
        assert_eq!(
            layer_trees_to_pdf(&[]).unwrap_err(),
            "direct PDF export requires at least one page"
        );

        let valid = direct_pdf_test_tree(96.0, 96.0, 0x000000ff);
        for raster_dpi in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            let options = DirectPdfExportOptions {
                raster_dpi,
                ..Default::default()
            };
            assert!(
                layer_trees_to_pdf_with_options(std::slice::from_ref(&valid), &options)
                    .unwrap_err()
                    .contains("invalid direct PDF raster dpi")
            );
        }

        for invalid in [
            direct_pdf_test_tree(0.0, 96.0, 0x000000ff),
            direct_pdf_test_tree(f64::NAN, 96.0, 0x000000ff),
            direct_pdf_test_tree(20_000.0, 96.0, 0x000000ff),
        ] {
            assert!(layer_trees_to_pdf(std::slice::from_ref(&invalid)).is_err());
        }

        let mut oversized_layer = direct_pdf_test_tree(96.0, 96.0, 0x000000ff);
        oversized_layer.root.bounds.x = f32::MAX as f64 * 2.0;
        assert!(layer_trees_to_pdf(&[oversized_layer])
            .unwrap_err()
            .contains("invalid layer bounds"));
    }

    #[cfg(feature = "native-skia")]
    #[test]
    fn skia_direct_pdf_rejects_corrupt_and_unvalidated_images() {
        use crate::paint::{LayerNode, PaintOp, RenderProfile};
        use crate::renderer::render_tree::{BoundingBox, ImageNode};
        use image::{ImageFormat, Rgb, RgbImage};
        use std::io::Cursor;

        let image = RgbImage::from_fn(64, 64, |x, y| {
            Rgb([
                (x.wrapping_mul(17) ^ y.wrapping_mul(31)) as u8,
                (x.wrapping_mul(47).wrapping_add(y.wrapping_mul(13))) as u8,
                (x.wrapping_mul(7) ^ y.wrapping_mul(53)) as u8,
            ])
        });
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("encode test PNG");
        let mut corrupt_png = cursor.into_inner();
        let idat_type_offset = corrupt_png
            .windows(4)
            .position(|window| window == b"IDAT")
            .expect("encoded PNG must contain IDAT");
        corrupt_png[idat_type_offset + 4] ^= 0xff;
        assert!(
            skia_safe::Image::from_encoded(skia_safe::Data::new_copy(&corrupt_png)).is_some(),
            "fixture must pass Skia's encoded-header check"
        );
        assert!(
            image::load_from_memory_with_format(&corrupt_png, ImageFormat::Png).is_err(),
            "fixture must fail strict PNG decoding"
        );

        let page_bounds = BoundingBox::new(0.0, 0.0, 96.0, 96.0);
        let tree = crate::paint::PageLayerTree::with_profile(
            96.0,
            96.0,
            LayerNode::leaf(
                page_bounds,
                None,
                vec![PaintOp::image(
                    BoundingBox::new(8.0, 8.0, 32.0, 32.0),
                    ImageNode::new(1, Some(corrupt_png)),
                    None,
                )],
            ),
            RenderProfile::Print,
        );

        let error = layer_trees_to_pdf(&[tree]).unwrap_err();
        assert!(
            error.contains("cannot decode image payload"),
            "unexpected error: {error}"
        );

        let malformed_gif = crate::paint::PageLayerTree::with_profile(
            96.0,
            96.0,
            LayerNode::leaf(
                page_bounds,
                None,
                vec![PaintOp::image(
                    BoundingBox::new(8.0, 8.0, 32.0, 32.0),
                    ImageNode::new(2, Some(b"GIF89a\x01\x00\x01\x00\x80\x00\x00".to_vec())),
                    None,
                )],
            ),
            RenderProfile::Print,
        );
        let error = layer_trees_to_pdf(&[malformed_gif]).unwrap_err();
        assert!(
            error.contains("cannot decode image payload"),
            "unexpected error: {error}"
        );

        let unsupported = crate::paint::PageLayerTree::with_profile(
            96.0,
            96.0,
            LayerNode::leaf(
                page_bounds,
                None,
                vec![PaintOp::image(
                    BoundingBox::new(8.0, 8.0, 32.0, 32.0),
                    ImageNode::new(3, Some(b"unrecognized image payload".to_vec())),
                    None,
                )],
            ),
            RenderProfile::Print,
        );
        let error = layer_trees_to_pdf(&[unsupported]).unwrap_err();
        assert!(
            error.contains("does not support image payload format application/octet-stream"),
            "unexpected error: {error}"
        );
    }

    #[cfg(feature = "native-skia")]
    #[test]
    fn skia_direct_pdf_rejects_known_lossy_shape_replay() {
        use crate::paint::{LayerNode, PaintOp, RenderProfile};
        use crate::renderer::render_tree::{BoundingBox, RectangleNode};
        use crate::renderer::{GradientFillInfo, ShapeStyle};

        let bbox = BoundingBox::new(0.0, 0.0, 96.0, 96.0);
        let gradient = GradientFillInfo {
            gradient_type: 1,
            angle: 0,
            center_x: 50,
            center_y: 50,
            colors: vec![0x000000ff, 0x00ff0000],
            positions: vec![0.0, 1.0],
        };
        let tree = crate::paint::PageLayerTree::with_profile(
            96.0,
            96.0,
            LayerNode::leaf(
                bbox,
                None,
                vec![PaintOp::rectangle(
                    bbox,
                    RectangleNode::new(0.0, ShapeStyle::default(), Some(Box::new(gradient))),
                )],
            ),
            RenderProfile::Print,
        );

        let error = layer_trees_to_pdf(&[tree]).unwrap_err();
        assert!(error.contains("gradient fill"), "unexpected error: {error}");
        assert!(error.contains("use the svg backend"));
    }

    #[cfg(feature = "native-skia")]
    #[test]
    fn skia_direct_pdf_raw_svg_fallback_is_strict_and_uses_requested_dpi() {
        use crate::paint::{LayerNode, PaintOp, RenderProfile};
        use crate::renderer::render_tree::{BoundingBox, RawSvgNode};

        let page_bounds = BoundingBox::new(0.0, 0.0, 96.0, 96.0);
        let svg_bounds = BoundingBox::new(8.0, 8.0, 32.0, 16.0);
        let tree_with_svg = |svg: &str| {
            crate::paint::PageLayerTree::with_profile(
                96.0,
                96.0,
                LayerNode::leaf(
                    page_bounds,
                    None,
                    vec![PaintOp::raw_svg(
                        svg_bounds,
                        RawSvgNode::new(svg.to_string()),
                    )],
                ),
                RenderProfile::Print,
            )
        };

        let invalid = layer_trees_to_pdf(&[tree_with_svg("<path")]).unwrap_err();
        assert!(invalid.contains("raw SVG raster fallback failed"));

        let options = DirectPdfExportOptions {
            raster_dpi: 192.0,
            ..Default::default()
        };
        let pdf = layer_trees_to_pdf_with_options(
            &[tree_with_svg(
                r##"<rect x="8" y="8" width="32" height="16" fill="#f00"/>"##,
            )],
            &options,
        )
        .unwrap();
        assert!(pdf.windows(9).any(|window| window == b"/Width 64"));
        assert!(pdf.windows(10).any(|window| window == b"/Height 32"));
    }
}
