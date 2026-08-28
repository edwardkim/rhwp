//! Producer-side lowering for font-native bitmap and SVG glyph resources.

use std::collections::{HashMap, HashSet};
use std::io::Read;

use flate2::read::GzDecoder;
use quick_xml::{events::Event, Reader};

use crate::paint::{
    font_blob_resource_key, resource_digest_hex, BinaryResourceKind, BinaryResourceRef,
    BitmapGlyphFiltering, BitmapGlyphPayload, BitmapGlyphScalingPolicy, FontBlobKey,
    FontBlobResource, FontDigest, FontFaceKey, FontFaceResource, FontFallbackPolicyId,
    FontInstanceKey, FontPortability, FontResourceSource, GlyphCluster, GlyphOutlinePayloadKind,
    GlyphRange, GlyphRunDiagnostics, GlyphRunOrientation, GlyphRunReplayEligibility,
    ImageResourceId, LayerAffineTransform, LayerGlyphOutlinePaint, LayerGlyphRunPaint, LayerNode,
    LayerNodeKind, LayerPoint, LayerVector, LocalizedName, PaintOp, PaintTextStyle,
    PaintVariantMeta, ResourceArena, ShapeKey, ShapingEngineId, SvgGlyphPayload, SvgResourceId,
    TextDirection, TextSourceId, TextSourceRange, TextSourceSpan, TextVariantKind,
    TextVariantQuality, WritingMode, RESOURCE_KEY_ALGORITHM,
};
use crate::renderer::render_tree::BoundingBox;
use crate::renderer::style_resolver::detect_lang_category;
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

use super::{MAX_PORTABLE_FONT_BLOB_BYTES, MAX_PORTABLE_GLYPHS_PER_RUN};

const MAX_STATIC_SVG_GLYPH_BYTES: usize = 1024 * 1024;
const MAX_BITMAP_GLYPH_BYTES: usize = 4 * 1024 * 1024;
const MAX_BITMAP_GLYPH_PIXELS: u32 = 4096 * 4096;
const MAX_FONT_NATIVE_SOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_FONT_NATIVE_SIDECARS_PER_PAGE: usize = 128;
const MAX_FONT_NATIVE_ENCODED_BYTES_PER_PAGE: usize = 8 * 1024 * 1024;
const MAX_FONT_NATIVE_DECODED_PIXELS_PER_PAGE: u64 = 32 * 1024 * 1024;
const MAX_PORTABLE_GLYPH_RUNS_PER_PAGE: usize = 4096;
const MAX_PORTABLE_FONT_BYTES_PER_PAGE: usize = 64 * 1024 * 1024;

fn png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if data.len() < 24
        || &data[..8] != PNG_SIGNATURE
        || &data[12..16] != b"IHDR"
        || u32::from_be_bytes(data[8..12].try_into().ok()?) != 13
    {
        return None;
    }
    Some((
        u32::from_be_bytes(data[16..20].try_into().ok()?),
        u32::from_be_bytes(data[20..24].try_into().ok()?),
    ))
}

#[derive(Debug, Clone)]
pub struct FontBitmapGlyphDecodeOptions {
    pub pixels_per_em: u16,
    pub source_range_utf8: TextSourceRange,
    pub glyph_range: GlyphRange,
    pub placement: BoundingBox,
    pub baseline_y: f64,
    pub transform_to_run: Option<LayerAffineTransform>,
    pub color_space: Option<String>,
    pub scaling_policy: BitmapGlyphScalingPolicy,
    pub filtering: BitmapGlyphFiltering,
}

impl FontBitmapGlyphDecodeOptions {
    pub fn new(
        pixels_per_em: u16,
        source_range_utf8: TextSourceRange,
        glyph_range: GlyphRange,
        placement: BoundingBox,
    ) -> Self {
        Self {
            pixels_per_em,
            source_range_utf8,
            glyph_range,
            placement,
            baseline_y: placement.height,
            transform_to_run: None,
            color_space: Some("sRGB".to_string()),
            scaling_policy: BitmapGlyphScalingPolicy::SourceExact,
            filtering: BitmapGlyphFiltering::Linear,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontBitmapGlyphDecodeError {
    SourceFontTooLarge,
    FaceParseFailed,
    GlyphIdOutOfRange,
    InvalidRequestedPpem,
    MissingRasterGlyph,
    UnsupportedRasterFormat,
    InvalidRasterGeometry,
    PayloadTooLarge,
    InvalidPngData,
    InvalidPayloadContract,
}

impl FontBitmapGlyphDecodeError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceFontTooLarge => "sourceFontTooLarge",
            Self::FaceParseFailed => "faceParseFailed",
            Self::GlyphIdOutOfRange => "glyphIdOutOfRange",
            Self::InvalidRequestedPpem => "invalidRequestedPpem",
            Self::MissingRasterGlyph => "missingRasterGlyph",
            Self::UnsupportedRasterFormat => "unsupportedRasterFormat",
            Self::InvalidRasterGeometry => "invalidRasterGeometry",
            Self::PayloadTooLarge => "payloadTooLarge",
            Self::InvalidPngData => "invalidPngData",
            Self::InvalidPayloadContract => "invalidPayloadContract",
        }
    }
}

pub fn decode_font_bitmap_glyph_payload(
    font_data: &[u8],
    face_index: u32,
    glyph_id: u32,
    options: &FontBitmapGlyphDecodeOptions,
    resources: &mut ResourceArena,
) -> Result<BitmapGlyphPayload, FontBitmapGlyphDecodeError> {
    if font_data.len() > MAX_FONT_NATIVE_SOURCE_BYTES {
        return Err(FontBitmapGlyphDecodeError::SourceFontTooLarge);
    }
    if glyph_id > u32::from(u16::MAX) {
        return Err(FontBitmapGlyphDecodeError::GlyphIdOutOfRange);
    }
    if options.pixels_per_em == 0 {
        return Err(FontBitmapGlyphDecodeError::InvalidRequestedPpem);
    }
    let face = ttf_parser::Face::parse(font_data, face_index)
        .map_err(|_| FontBitmapGlyphDecodeError::FaceParseFailed)?;
    let raster = face
        .glyph_raster_image(ttf_parser::GlyphId(glyph_id as u16), options.pixels_per_em)
        .ok_or(FontBitmapGlyphDecodeError::MissingRasterGlyph)?;
    if raster.format != ttf_parser::RasterImageFormat::PNG {
        return Err(FontBitmapGlyphDecodeError::UnsupportedRasterFormat);
    }
    if raster.width == 0 || raster.height == 0 || raster.pixels_per_em == 0 {
        return Err(FontBitmapGlyphDecodeError::InvalidRasterGeometry);
    }
    if raster.data.len() > MAX_BITMAP_GLYPH_BYTES {
        return Err(FontBitmapGlyphDecodeError::PayloadTooLarge);
    }
    let (encoded_width, encoded_height) =
        png_dimensions(raster.data).ok_or(FontBitmapGlyphDecodeError::InvalidPngData)?;
    if encoded_width == 0
        || encoded_height == 0
        || u64::from(encoded_width) * u64::from(encoded_height) > u64::from(MAX_BITMAP_GLYPH_PIXELS)
    {
        return Err(FontBitmapGlyphDecodeError::PayloadTooLarge);
    }
    if encoded_width != u32::from(raster.width) || encoded_height != u32::from(raster.height) {
        return Err(FontBitmapGlyphDecodeError::InvalidRasterGeometry);
    }
    let decoded = image::load_from_memory_with_format(raster.data, image::ImageFormat::Png)
        .map_err(|_| FontBitmapGlyphDecodeError::InvalidPngData)?;
    if decoded.width() != u32::from(raster.width) || decoded.height() != u32::from(raster.height) {
        return Err(FontBitmapGlyphDecodeError::InvalidRasterGeometry);
    }

    let scale = options.pixels_per_em as f64 / raster.pixels_per_em as f64;
    let glyph_id = ttf_parser::GlyphId(glyph_id as u16);
    let uses_sbix_offsets = face
        .tables()
        .sbix
        .and_then(|table| table.best_strike(options.pixels_per_em))
        .and_then(|strike| strike.get(glyph_id))
        .is_some();
    let (x, y) = if uses_sbix_offsets {
        (
            options.placement.x - f64::from(raster.x) * scale,
            options.placement.y + options.baseline_y - f64::from(raster.height) * scale
                + f64::from(raster.y) * scale,
        )
    } else {
        (
            options.placement.x + f64::from(raster.x) * scale,
            options.placement.y + options.baseline_y
                - (f64::from(raster.y) + f64::from(raster.height)) * scale,
        )
    };
    let placement = BoundingBox::new(
        x,
        y,
        f64::from(raster.width) * scale,
        f64::from(raster.height) * scale,
    );
    let mut payload = BitmapGlyphPayload {
        image_ref: ImageResourceId(usize::MAX),
        source_range_utf8: options.source_range_utf8,
        glyph_range: options.glyph_range,
        placement,
        alpha_premultiplied: false,
        transform_to_run: options.transform_to_run,
        scaling_policy: options.scaling_policy,
        filtering: options.filtering,
    };
    if !payload.has_strict_visual_contract() {
        return Err(FontBitmapGlyphDecodeError::InvalidPayloadContract);
    }
    payload.image_ref = resources.intern_image_bytes(raster.data);
    Ok(payload)
}

#[derive(Debug, Clone)]
pub struct FontSvgGlyphDecodeOptions {
    pub source_range_utf8: TextSourceRange,
    pub glyph_range: GlyphRange,
    pub transform_to_run: Option<LayerAffineTransform>,
    pub intrinsic_size: Option<LayerVector>,
}

impl FontSvgGlyphDecodeOptions {
    pub fn new(source_range_utf8: TextSourceRange, glyph_range: GlyphRange) -> Self {
        Self {
            source_range_utf8,
            glyph_range,
            transform_to_run: None,
            intrinsic_size: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSvgGlyphDecodeError {
    SourceFontTooLarge,
    FaceParseFailed,
    GlyphIdOutOfRange,
    MissingSvgGlyph,
    SharedSvgDocument,
    SvgPayloadTooLarge,
    SvgDecompressionFailed,
    InvalidUtf8,
    UnsafeStaticSvg,
    InvalidSvgXml,
    MissingViewBox,
    InvalidViewBox,
    InvalidPayloadContract,
}

impl FontSvgGlyphDecodeError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceFontTooLarge => "sourceFontTooLarge",
            Self::FaceParseFailed => "faceParseFailed",
            Self::GlyphIdOutOfRange => "glyphIdOutOfRange",
            Self::MissingSvgGlyph => "missingSvgGlyph",
            Self::SharedSvgDocument => "sharedSvgDocument",
            Self::SvgPayloadTooLarge => "svgPayloadTooLarge",
            Self::SvgDecompressionFailed => "svgDecompressionFailed",
            Self::InvalidUtf8 => "invalidUtf8",
            Self::UnsafeStaticSvg => "unsafeStaticSvg",
            Self::InvalidSvgXml => "invalidSvgXml",
            Self::MissingViewBox => "missingViewBox",
            Self::InvalidViewBox => "invalidViewBox",
            Self::InvalidPayloadContract => "invalidPayloadContract",
        }
    }
}

pub fn decode_font_svg_glyph_payload(
    font_data: &[u8],
    face_index: u32,
    glyph_id: u32,
    options: &FontSvgGlyphDecodeOptions,
    resources: &mut ResourceArena,
) -> Result<SvgGlyphPayload, FontSvgGlyphDecodeError> {
    if font_data.len() > MAX_FONT_NATIVE_SOURCE_BYTES {
        return Err(FontSvgGlyphDecodeError::SourceFontTooLarge);
    }
    if glyph_id > u32::from(u16::MAX) {
        return Err(FontSvgGlyphDecodeError::GlyphIdOutOfRange);
    }
    let face = ttf_parser::Face::parse(font_data, face_index)
        .map_err(|_| FontSvgGlyphDecodeError::FaceParseFailed)?;
    let document = face
        .glyph_svg_image(ttf_parser::GlyphId(glyph_id as u16))
        .ok_or(FontSvgGlyphDecodeError::MissingSvgGlyph)?;
    if document.start_glyph_id != document.end_glyph_id {
        return Err(FontSvgGlyphDecodeError::SharedSvgDocument);
    }
    let svg_bytes = if document.data.starts_with(&[0x1f, 0x8b]) {
        let mut decoded = Vec::new();
        GzDecoder::new(document.data)
            .take((MAX_STATIC_SVG_GLYPH_BYTES + 1) as u64)
            .read_to_end(&mut decoded)
            .map_err(|_| FontSvgGlyphDecodeError::SvgDecompressionFailed)?;
        if decoded.len() > MAX_STATIC_SVG_GLYPH_BYTES {
            return Err(FontSvgGlyphDecodeError::SvgPayloadTooLarge);
        }
        decoded
    } else {
        if document.data.len() > MAX_STATIC_SVG_GLYPH_BYTES {
            return Err(FontSvgGlyphDecodeError::SvgPayloadTooLarge);
        }
        document.data.to_vec()
    };
    let fragment = std::str::from_utf8(&svg_bytes)
        .map_err(|_| FontSvgGlyphDecodeError::InvalidUtf8)?
        .trim();
    if !crate::renderer::static_svg::static_svg_fragment_has_path_layer(fragment) {
        return Err(FontSvgGlyphDecodeError::UnsafeStaticSvg);
    }

    let mut reader = Reader::from_str(fragment);
    reader.config_mut().trim_text(true);
    let mut view_box = None;
    loop {
        match reader.read_event() {
            Ok(Event::Start(element)) | Ok(Event::Empty(element)) => {
                if view_box.is_none() && element.name().as_ref().eq_ignore_ascii_case(b"svg") {
                    for attribute in element.attributes().with_checks(true) {
                        let attribute =
                            attribute.map_err(|_| FontSvgGlyphDecodeError::InvalidSvgXml)?;
                        if !attribute.key.as_ref().eq_ignore_ascii_case(b"viewBox") {
                            continue;
                        }
                        let value = std::str::from_utf8(attribute.value.as_ref())
                            .map_err(|_| FontSvgGlyphDecodeError::InvalidViewBox)?;
                        let values = value
                            .split(|character: char| {
                                character.is_ascii_whitespace() || character == ','
                            })
                            .filter(|token| !token.is_empty())
                            .map(str::parse::<f64>)
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|_| FontSvgGlyphDecodeError::InvalidViewBox)?;
                        if values.len() != 4
                            || values.iter().any(|value| !value.is_finite())
                            || values[2] <= 0.0
                            || values[3] <= 0.0
                        {
                            return Err(FontSvgGlyphDecodeError::InvalidViewBox);
                        }
                        view_box =
                            Some(BoundingBox::new(values[0], values[1], values[2], values[3]));
                        break;
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(_) => return Err(FontSvgGlyphDecodeError::InvalidSvgXml),
        }
    }
    let view_box = view_box.ok_or(FontSvgGlyphDecodeError::MissingViewBox)?;
    let mut payload = SvgGlyphPayload {
        svg_ref: SvgResourceId(usize::MAX),
        source_range_utf8: options.source_range_utf8,
        glyph_range: options.glyph_range,
        transform_to_run: options.transform_to_run,
        view_box,
        intrinsic_size: options.intrinsic_size,
        static_sanitized: true,
        script_allowed: false,
        animation_allowed: false,
        external_resources_allowed: false,
        interactivity_allowed: false,
    };
    if !payload.has_static_sanitized_contract() {
        return Err(FontSvgGlyphDecodeError::InvalidPayloadContract);
    }
    payload.svg_ref = resources.intern_svg_fragment(fragment);
    Ok(payload)
}

#[derive(Debug, Clone, Copy)]
pub struct EmbeddedFontFace<'a> {
    pub char_shape_id: u32,
    pub language_index: usize,
    pub family: &'a str,
    pub alternate_family: Option<&'a str>,
    pub bytes: &'a [u8],
    pub face_index: u32,
}

pub fn resolve_embedded_font_face_index(
    bytes: &[u8],
    family: &str,
    alternate_family: Option<&str>,
) -> Option<u32> {
    const MAX_COLLECTION_FACES: u32 = 256;
    if bytes.len() > MAX_FONT_NATIVE_SOURCE_BYTES {
        return None;
    }
    let face_count = ttf_parser::fonts_in_collection(bytes).unwrap_or(1);
    if face_count == 0 || face_count > MAX_COLLECTION_FACES {
        return None;
    }
    if face_count == 1 {
        ttf_parser::Face::parse(bytes, 0).ok()?;
        return Some(0);
    }

    let matches = (0..face_count)
        .filter(|face_index| {
            ttf_parser::Face::parse(bytes, *face_index)
                .ok()
                .is_some_and(|face| {
                    face.names().into_iter().any(|name| {
                        matches!(
                            name.name_id,
                            ttf_parser::name_id::FAMILY
                                | ttf_parser::name_id::TYPOGRAPHIC_FAMILY
                                | ttf_parser::name_id::WWS_FAMILY
                        ) && name.to_string().is_some_and(|value| {
                            value.eq_ignore_ascii_case(family)
                                || alternate_family
                                    .is_some_and(|family| value.eq_ignore_ascii_case(family))
                        })
                    })
                })
        })
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        matches.first().copied()
    } else {
        None
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontGlyphLoweringReport {
    pub attempted_runs: usize,
    pub attempted_glyph_runs: usize,
    pub prepared_glyph_font_faces: usize,
    pub emitted_glyph_runs: usize,
    pub emitted_bitmap_glyphs: usize,
    pub emitted_svg_glyphs: usize,
    pub rejected_runs: usize,
    pub rejected_glyph_runs: usize,
}

pub fn lower_font_native_glyph_sidecars(
    root: &mut LayerNode,
    resources: &mut ResourceArena,
    fonts: &[EmbeddedFontFace<'_>],
) -> FontGlyphLoweringReport {
    lower_font_native_glyph_sidecars_excluding(root, resources, fonts, &HashSet::new())
}

/// common shaping이 이미 정확한 GlyphRun alternative를 소유한 text source는
/// nominal portable GlyphRun만 건너뛴다. TextRun과 bitmap/SVG sidecar 경로는
/// 그대로 보존한다.
pub(crate) fn lower_font_native_glyph_sidecars_excluding(
    root: &mut LayerNode,
    resources: &mut ResourceArena,
    fonts: &[EmbeddedFontFace<'_>],
    claimed_glyph_run_sources: &HashSet<u32>,
) -> FontGlyphLoweringReport {
    let mut lowerer = FontGlyphLowerer {
        resources,
        fonts,
        claimed_glyph_run_sources,
        report: FontGlyphLoweringReport::default(),
        next_text_source_id: 0,
        emitted_sidecars: 0,
        emitted_glyph_runs: 0,
        portable_font_bytes: 0,
        portable_font_cache: HashMap::new(),
        encoded_resource_bytes: 0,
        decoded_resource_pixels: 0,
    };
    lowerer.lower_node(root);
    lowerer.report
}

struct FontGlyphLowerer<'a, 'font> {
    resources: &'a mut ResourceArena,
    fonts: &'a [EmbeddedFontFace<'font>],
    claimed_glyph_run_sources: &'a HashSet<u32>,
    report: FontGlyphLoweringReport,
    next_text_source_id: u32,
    emitted_sidecars: usize,
    emitted_glyph_runs: usize,
    portable_font_bytes: usize,
    portable_font_cache: HashMap<usize, Option<PortableFontCacheEntry<'font>>>,
    encoded_resource_bytes: usize,
    decoded_resource_pixels: u64,
}

struct PortableFontCacheEntry<'font> {
    face: ttf_parser::Face<'font>,
    blob_key: FontBlobKey,
    face_key: FontFaceKey,
    digest: FontDigest,
    data_ref: BinaryResourceRef,
    registered: bool,
}

impl FontGlyphLowerer<'_, '_> {
    fn lower_node(&mut self, node: &mut LayerNode) {
        match &mut node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    self.lower_node(child);
                }
            }
            LayerNodeKind::ClipRect { child, .. } => self.lower_node(child),
            LayerNodeKind::Leaf { ops } => self.lower_leaf(ops),
        }
    }

    fn lower_leaf(&mut self, ops: &mut Vec<PaintOp>) {
        let mut lowered = Vec::with_capacity(ops.len());
        for op in ops.drain(..) {
            if let PaintOp::TextRun { bbox, run } = op {
                let text_source_id = self.next_text_source_id;
                self.next_text_source_id = self.next_text_source_id.saturating_add(1);
                let glyph_run = (!self.claimed_glyph_run_sources.contains(&text_source_id))
                    .then(|| self.lower_portable_glyph_run(bbox, &run, text_source_id))
                    .flatten();
                let sidecar = self.lower_text_run(bbox, &run, text_source_id);
                lowered.push(PaintOp::TextRun { bbox, run });
                if let Some(glyph_run) = glyph_run {
                    lowered.push(PaintOp::GlyphRun {
                        bbox,
                        run: Box::new(glyph_run),
                    });
                }
                if let Some(sidecar) = sidecar {
                    lowered.push(PaintOp::GlyphOutline {
                        bbox,
                        outline: Box::new(sidecar),
                    });
                }
            } else {
                lowered.push(op);
            }
        }
        *ops = lowered;
    }

    fn lower_portable_glyph_run(
        &mut self,
        bbox: BoundingBox,
        run: &crate::renderer::render_tree::TextRunNode,
        text_source_id: u32,
    ) -> Option<LayerGlyphRunPaint> {
        let paint_style = PaintTextStyle::from(&run.style);
        if run.text.is_empty()
            || run.text.chars().count() > MAX_PORTABLE_GLYPHS_PER_RUN
            || self.emitted_glyph_runs >= MAX_PORTABLE_GLYPH_RUNS_PER_PAGE
            || !paint_style.is_fill_only_glyph_replay()
            || run.rotation.abs() > f64::EPSILON
            || run.is_vertical
            || run.style.bold
            || run.style.italic
            || run.char_overlap.is_some()
            || run
                .display_text
                .as_deref()
                .is_some_and(|display| display != run.text)
        {
            return None;
        }

        let characters = run.text.chars().collect::<Vec<_>>();
        if characters
            .iter()
            .any(|character| !is_nominal_glyph_replay_scalar(*character))
        {
            return None;
        }
        let char_shape_id = run.char_shape_id?;
        let language_index = detect_lang_category(characters[0]);
        if characters
            .iter()
            .any(|character| detect_lang_category(*character) != language_index)
        {
            return None;
        }
        let font_index = self.fonts.iter().position(|font| {
            font.char_shape_id == char_shape_id && font.language_index == language_index
        })?;
        let font = self.fonts[font_index];
        self.report.attempted_glyph_runs += 1;
        if !self.portable_font_cache.contains_key(&font_index) {
            self.report.prepared_glyph_font_faces += 1;
            let prepared = if font.bytes.is_empty()
                || font.bytes.len() > MAX_PORTABLE_FONT_BLOB_BYTES
            {
                None
            } else {
                ttf_parser::Face::parse(font.bytes, font.face_index)
                    .ok()
                    .map(|face| {
                        let digest_value = resource_digest_hex(font.bytes);
                        let resource_key = font_blob_resource_key(font.bytes.len(), &digest_value);
                        PortableFontCacheEntry {
                            face,
                            blob_key: FontBlobKey(resource_key.clone()),
                            face_key: FontFaceKey(format!(
                                "{resource_key}:face:{}",
                                font.face_index
                            )),
                            digest: FontDigest {
                                algorithm: RESOURCE_KEY_ALGORITHM.to_string(),
                                value: digest_value,
                            },
                            data_ref: BinaryResourceRef {
                                kind: BinaryResourceKind::FontBlob,
                                id: resource_key,
                            },
                            registered: false,
                        }
                    })
            };
            self.portable_font_cache.insert(font_index, prepared);
        }
        let Some(cached_font) = self
            .portable_font_cache
            .get(&font_index)
            .and_then(Option::as_ref)
        else {
            self.report.rejected_glyph_runs += 1;
            return None;
        };
        let face = &cached_font.face;
        let producer_positions = run.replay_positions_for(&run.text);
        if producer_positions.len() != characters.len() + 1
            || producer_positions
                .iter()
                .any(|position| !position.is_finite())
        {
            self.report.rejected_glyph_runs += 1;
            return None;
        }

        let units_per_em = f64::from(face.units_per_em());
        if units_per_em <= 0.0 || !run.style.font_size.is_finite() || run.style.font_size <= 0.0 {
            self.report.rejected_glyph_runs += 1;
            return None;
        }
        let mut glyph_ids = Vec::with_capacity(characters.len());
        let mut positions = Vec::with_capacity(characters.len());
        let mut advances = Vec::with_capacity(characters.len());
        let mut clusters = Vec::with_capacity(characters.len());
        let mut utf8_offset = 0_u32;
        let mut utf16_offset = 0_u32;
        let mut max_advance_delta_px = 0.0_f64;
        for (index, character) in characters.iter().copied().enumerate() {
            let Some(glyph_id) = face
                .glyph_index(character)
                .filter(|glyph_id| glyph_id.0 != 0)
            else {
                self.report.rejected_glyph_runs += 1;
                return None;
            };
            let Some(font_advance) = face.glyph_hor_advance(glyph_id) else {
                self.report.rejected_glyph_runs += 1;
                return None;
            };
            let producer_advance = producer_positions[index + 1] - producer_positions[index];
            if !producer_advance.is_finite() {
                self.report.rejected_glyph_runs += 1;
                return None;
            }
            let nominal_advance = f64::from(font_advance) * run.style.font_size / units_per_em;
            max_advance_delta_px =
                max_advance_delta_px.max((producer_advance - nominal_advance).abs());
            glyph_ids.push(u32::from(glyph_id.0));
            positions.push(LayerPoint {
                x: producer_positions[index],
                y: 0.0,
            });
            advances.push(LayerVector {
                dx: producer_advance,
                dy: 0.0,
            });
            let utf8_end = utf8_offset.saturating_add(character.len_utf8() as u32);
            let utf16_end = utf16_offset.saturating_add(character.len_utf16() as u32);
            clusters.push(GlyphCluster {
                source_range_utf8: TextSourceRange::new(utf8_offset, utf8_end),
                source_range_utf16: Some(TextSourceRange::new(utf16_offset, utf16_end)),
                text_range_utf8: Some(TextSourceRange::new(utf8_offset, utf8_end)),
                glyph_range: GlyphRange::new(index as u32, index as u32 + 1),
                flags: Vec::new(),
            });
            utf8_offset = utf8_end;
            utf16_offset = utf16_end;
        }

        let (blob_key, face_key, digest, data_ref, registered) = {
            let cached_font = self.portable_font_cache[&font_index]
                .as_ref()
                .expect("portable font cache was validated above");
            (
                cached_font.blob_key.clone(),
                cached_font.face_key.clone(),
                cached_font.digest.clone(),
                cached_font.data_ref.clone(),
                cached_font.registered,
            )
        };
        if !registered {
            let is_new_blob = self.resources.font_blob_bytes_for_ref(&data_ref).is_none();
            if is_new_blob
                && self
                    .portable_font_bytes
                    .checked_add(font.bytes.len())
                    .is_none_or(|total| total > MAX_PORTABLE_FONT_BYTES_PER_PAGE)
            {
                self.report.rejected_glyph_runs += 1;
                return None;
            }
            self.resources.intern_font_blob_bytes(font.bytes);
            if is_new_blob {
                self.portable_font_bytes += font.bytes.len();
            }
            if !self
                .resources
                .font_resources()
                .blobs
                .iter()
                .any(|blob| blob.id == blob_key)
            {
                self.resources
                    .font_resources_mut()
                    .blobs
                    .push(FontBlobResource {
                        id: blob_key.clone(),
                        digest: Some(digest.clone()),
                        source: FontResourceSource::Embedded,
                        data_ref: Some(data_ref.clone()),
                        portability: FontPortability::PortableBlob {
                            digest: digest.clone(),
                            data_ref: data_ref.clone(),
                        },
                    });
            }
            if !self
                .resources
                .font_resources()
                .faces
                .iter()
                .any(|face| face.id == face_key)
            {
                let mut family_names = vec![LocalizedName {
                    locale: None,
                    value: font.family.to_string(),
                }];
                if let Some(alternate) = font.alternate_family.filter(|name| *name != font.family) {
                    family_names.push(LocalizedName {
                        locale: None,
                        value: alternate.to_string(),
                    });
                }
                self.resources
                    .font_resources_mut()
                    .faces
                    .push(FontFaceResource {
                        id: face_key.clone(),
                        blob_key,
                        face_index: font.face_index,
                        postscript_name: None,
                        family_names,
                        style_names: Vec::new(),
                        weight_class: Some(face.weight().to_number()),
                        width_class: Some(face.width().to_number()),
                        italic: Some(face.is_italic()),
                    });
            }
            self.portable_font_cache
                .get_mut(&font_index)
                .and_then(Option::as_mut)
                .expect("portable font cache was validated above")
                .registered = true;
        }

        let equivalence_group = format!("text-{text_source_id}");
        let mut variant = PaintVariantMeta::text_run_default(equivalence_group.clone());
        variant.variant_id = "glyphRun".to_string();
        variant.variant_kind = TextVariantKind::GlyphRun;
        variant.is_default_fallback = false;
        variant.requires = vec!["fontResources".to_string(), "text.glyphRun".to_string()];
        variant.quality = Some(TextVariantQuality::PositionAdjusted);
        variant.anchor_op_id = Some(equivalence_group);
        self.emitted_glyph_runs += 1;
        self.report.emitted_glyph_runs += 1;

        Some(LayerGlyphRunPaint {
            source: TextSourceSpan {
                id: TextSourceId(text_source_id),
                utf8_range: TextSourceRange::new(0, run.text.len() as u32),
                utf16_range: TextSourceRange::new(0, run.text.encode_utf16().count() as u32),
                stable_source_key: None,
            },
            variant,
            paint_style,
            shape_key: ShapeKey {
                font_instance: FontInstanceKey {
                    face_key,
                    size_px: run.style.font_size,
                    variations: Vec::new(),
                    synthetic_bold: false,
                    synthetic_italic: false,
                },
                direction: TextDirection::Ltr,
                writing_mode: WritingMode::HorizontalTb,
                script: None,
                language: None,
                features: Vec::new(),
                shaping_engine: ShapingEngineId("ttf-parser-nominal-v1".to_string()),
                fallback_policy: FontFallbackPolicyId("none".to_string()),
            },
            placement: crate::paint::text_shape::text_run_placement(bbox, run),
            glyph_ids,
            positions,
            advances: Some(advances),
            clusters,
            direction: TextDirection::Ltr,
            bidi_level: Some(0),
            writing_mode: WritingMode::HorizontalTb,
            orientation: GlyphRunOrientation::Horizontal,
            glyph_transforms: None,
            diagnostics: GlyphRunDiagnostics {
                quality: TextVariantQuality::PositionAdjusted,
                replay_eligibility: GlyphRunReplayEligibility::Portable,
                strict_visual_eligible: true,
                max_origin_delta_px: 0.0,
                max_advance_delta_px,
                max_residual_after_adjustment_px: 0.0,
                cluster_mismatch_count: 0,
                missing_glyph_count: 0,
                used_fallback_font_count: 0,
                reason: Some("producerPositionedNominalGlyphs".to_string()),
            },
        })
    }

    fn lower_text_run(
        &mut self,
        bbox: BoundingBox,
        run: &crate::renderer::render_tree::TextRunNode,
        text_source_id: u32,
    ) -> Option<LayerGlyphOutlinePaint> {
        let mut characters = run.text.chars();
        let character = characters.next()?;
        if characters.next().is_some() {
            return None;
        }
        let paint_style = PaintTextStyle::from(&run.style);
        if !paint_style.is_fill_only_glyph_replay()
            || run.rotation.abs() > f64::EPSILON
            || run.is_vertical
            || run.style.bold
            || run.style.italic
            || (run.style.ratio - 1.0).abs() > f64::EPSILON
            || self.emitted_sidecars >= MAX_FONT_NATIVE_SIDECARS_PER_PAGE
        {
            return None;
        }
        let char_shape_id = run.char_shape_id?;
        let language_index = crate::renderer::style_resolver::detect_lang_category(character);
        let font = self.fonts.iter().find(|font| {
            font.char_shape_id == char_shape_id && font.language_index == language_index
        })?;
        self.report.attempted_runs += 1;
        if font.bytes.len() > MAX_FONT_NATIVE_SOURCE_BYTES {
            self.report.rejected_runs += 1;
            return None;
        }
        let face = match ttf_parser::Face::parse(font.bytes, font.face_index) {
            Ok(face) => face,
            Err(_) => {
                self.report.rejected_runs += 1;
                return None;
            }
        };
        let glyph_id = match face.glyph_index(character) {
            Some(glyph_id) => u32::from(glyph_id.0),
            None => {
                self.report.rejected_runs += 1;
                return None;
            }
        };
        let source_range = TextSourceRange::new(0, run.text.len() as u32);
        let glyph_range = GlyphRange::new(0, 1);
        let mut bitmap_options = FontBitmapGlyphDecodeOptions::new(
            run.style.font_size.round().clamp(1.0, f64::from(u16::MAX)) as u16,
            source_range,
            glyph_range,
            bbox,
        );
        bitmap_options.baseline_y = run.baseline;
        let glyph_id_u16 = ttf_parser::GlyphId(glyph_id as u16);
        let Some(raster) = face.glyph_raster_image(glyph_id_u16, bitmap_options.pixels_per_em)
        else {
            self.report.rejected_runs += 1;
            return None;
        };
        let encoded_bytes = raster.data.len();
        let decoded_pixels = u64::from(raster.width) * u64::from(raster.height);
        if self.encoded_resource_bytes.saturating_add(encoded_bytes)
            > MAX_FONT_NATIVE_ENCODED_BYTES_PER_PAGE
            || self.decoded_resource_pixels.saturating_add(decoded_pixels)
                > MAX_FONT_NATIVE_DECODED_PIXELS_PER_PAGE
        {
            self.report.rejected_runs += 1;
            return None;
        }
        let bitmap = decode_font_bitmap_glyph_payload(
            font.bytes,
            font.face_index,
            glyph_id,
            &bitmap_options,
            self.resources,
        )
        .ok();
        let payload_kind = if bitmap.is_some() {
            self.report.emitted_bitmap_glyphs += 1;
            self.emitted_sidecars += 1;
            self.encoded_resource_bytes += encoded_bytes;
            self.decoded_resource_pixels += decoded_pixels;
            GlyphOutlinePayloadKind::BitmapGlyph
        } else {
            self.report.rejected_runs += 1;
            return None;
        };

        let equivalence_group = format!("text-{text_source_id}");
        let mut variant = PaintVariantMeta::text_run_default(equivalence_group.clone());
        variant.variant_id = "glyphOutline".to_string();
        variant.variant_kind = TextVariantKind::GlyphOutline;
        variant.is_default_fallback = false;
        variant.requires = vec![format!("text.glyphOutline.{}", payload_kind.as_str())];
        variant.quality = Some(TextVariantQuality::Exact);
        variant.anchor_op_id = Some(equivalence_group);
        let placement = crate::paint::text_shape::text_run_placement(bbox, run);

        Some(LayerGlyphOutlinePaint {
            source: TextSourceSpan {
                id: TextSourceId(text_source_id),
                utf8_range: source_range,
                utf16_range: TextSourceRange::new(0, run.text.encode_utf16().count() as u32),
                stable_source_key: None,
            },
            variant,
            payload_kind,
            color_layers: None,
            bitmap_glyph: bitmap,
            svg_glyph: None,
            paint_style,
            placement,
            paths: Vec::new(),
            stroke: None,
            diagnostics: GlyphRunDiagnostics {
                quality: TextVariantQuality::Exact,
                replay_eligibility: GlyphRunReplayEligibility::Portable,
                strict_visual_eligible: true,
                max_origin_delta_px: 0.0,
                max_advance_delta_px: 0.0,
                max_residual_after_adjustment_px: 0.0,
                cluster_mismatch_count: 0,
                missing_glyph_count: 0,
                used_fallback_font_count: 0,
                reason: Some("fontNativeGlyphPayload".to_string()),
            },
        })
    }
}

fn is_nominal_glyph_replay_scalar(character: char) -> bool {
    let code_point = character as u32;
    let is_old_hangul = matches!(
        code_point,
        0x1100..=0x11ff | 0xa960..=0xa97f | 0xd7b0..=0xd7ff
    );
    let nominal_replay_block = matches!(
        code_point,
        0x0000..=0x02ff
            | 0x0370..=0x058f
            | 0x1e00..=0x1fff
            | 0x2000..=0x2fff
            | 0x2e80..=0xd7af
            | 0xe000..=0xf8ff
            | 0xf900..=0xfb06
            | 0xfe10..=0xfe1f
            | 0xfe30..=0xfe6f
            | 0xff00..=0xffef
            | 0x1d400..=0x1d7ff
            | 0x20000..=0x323af
    );
    if character.is_control() || is_old_hangul || !nominal_replay_block {
        return false;
    }
    !matches!(
        character.general_category(),
        GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
            | GeneralCategory::Format
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_font() -> &'static [u8] {
        include_bytes!("../../tests/fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf")
    }

    fn fixture_glyph_id(character: char) -> u32 {
        let face = ttf_parser::Face::parse(fixture_font(), 0).expect("fixture font parses");
        u32::from(face.glyph_index(character).expect("fixture glyph exists").0)
    }

    fn fixture_ttc() -> &'static [u8] {
        include_bytes!("../../tests/fixtures/fonts/RHWPExactFaceSmoke.ttc")
    }

    fn placement() -> BoundingBox {
        BoundingBox::new(12.0, 18.0, 16.0, 16.0)
    }

    #[test]
    fn lowers_font_native_png_strike_into_strict_bitmap_payload() {
        let mut resources = ResourceArena::default();
        let payload = decode_font_bitmap_glyph_payload(
            fixture_font(),
            0,
            fixture_glyph_id('\u{E100}'),
            &FontBitmapGlyphDecodeOptions::new(
                16,
                TextSourceRange::new(0, 3),
                GlyphRange { start: 0, end: 1 },
                placement(),
            ),
            &mut resources,
        )
        .expect("font-native PNG strike lowers");

        assert!(payload.has_strict_visual_contract());
        assert_eq!(payload.image_ref, ImageResourceId(0));
        assert_eq!(
            (
                payload.placement.x,
                payload.placement.y,
                payload.placement.width,
                payload.placement.height,
            ),
            (10.0, 21.0, 16.0, 16.0)
        );
        assert_eq!(resources.image_count(), 1);
        assert!(resources
            .image_bytes(payload.image_ref)
            .is_some_and(|bytes| bytes.starts_with(b"\x89PNG\r\n\x1a\n")));
    }

    #[test]
    fn png_dimensions_rejects_non_ihdr_and_reports_encoded_size() {
        let face = ttf_parser::Face::parse(fixture_font(), 0).expect("fixture font parses");
        let raster = face
            .glyph_raster_image(ttf_parser::GlyphId(fixture_glyph_id('\u{E100}') as u16), 16)
            .expect("fixture PNG strike exists");
        assert_eq!(png_dimensions(raster.data), Some((16, 16)));

        let mut malformed = raster.data.to_vec();
        malformed[12..16].copy_from_slice(b"JUNK");
        assert_eq!(png_dimensions(&malformed), None);
    }

    #[test]
    fn bitmap_lowering_rejects_oversized_png_header_before_decode() {
        let mut font = fixture_font().to_vec();
        let png_offset = font
            .windows(8)
            .position(|window| window == b"\x89PNG\r\n\x1a\n")
            .expect("fixture embeds PNG data");
        font[png_offset + 16..png_offset + 20].copy_from_slice(&5_000u32.to_be_bytes());
        font[png_offset + 20..png_offset + 24].copy_from_slice(&5_000u32.to_be_bytes());
        let mut resources = ResourceArena::default();

        let result = decode_font_bitmap_glyph_payload(
            &font,
            0,
            fixture_glyph_id('\u{E100}'),
            &FontBitmapGlyphDecodeOptions::new(
                16,
                TextSourceRange::new(0, 1),
                GlyphRange::new(0, 1),
                placement(),
            ),
            &mut resources,
        );

        assert!(matches!(
            result,
            Err(FontBitmapGlyphDecodeError::PayloadTooLarge)
        ));
        assert_eq!(resources.image_count(), 0);
    }

    #[test]
    fn bitmap_lowering_fails_closed_without_png_strike_or_deterministic_options() {
        let mut resources = ResourceArena::default();
        let missing = decode_font_bitmap_glyph_payload(
            fixture_font(),
            0,
            fixture_glyph_id('\u{E101}'),
            &FontBitmapGlyphDecodeOptions::new(
                16,
                TextSourceRange::new(0, 1),
                GlyphRange { start: 0, end: 1 },
                placement(),
            ),
            &mut resources,
        );
        assert!(matches!(
            missing,
            Err(FontBitmapGlyphDecodeError::MissingRasterGlyph)
        ));

        let mut invalid = FontBitmapGlyphDecodeOptions::new(
            16,
            TextSourceRange::new(0, 1),
            GlyphRange { start: 0, end: 1 },
            placement(),
        );
        invalid.scaling_policy = BitmapGlyphScalingPolicy::BackendDefault;
        let invalid = decode_font_bitmap_glyph_payload(
            fixture_font(),
            0,
            fixture_glyph_id('\u{E100}'),
            &invalid,
            &mut resources,
        );
        assert!(matches!(
            invalid,
            Err(FontBitmapGlyphDecodeError::InvalidPayloadContract)
        ));
        assert_eq!(resources.image_count(), 0);
    }

    #[test]
    fn lowers_font_native_static_svg_into_strict_vector_payload() {
        let mut resources = ResourceArena::default();
        let payload = decode_font_svg_glyph_payload(
            fixture_font(),
            0,
            fixture_glyph_id('\u{E101}'),
            &FontSvgGlyphDecodeOptions::new(
                TextSourceRange::new(0, 3),
                GlyphRange { start: 0, end: 1 },
            ),
            &mut resources,
        )
        .expect("font-native static SVG lowers");

        assert!(payload.has_static_sanitized_contract());
        assert_eq!(payload.svg_ref, SvgResourceId(0));
        assert_eq!(
            (
                payload.view_box.x,
                payload.view_box.y,
                payload.view_box.width,
                payload.view_box.height,
            ),
            (0.0, 0.0, 16.0, 16.0)
        );
        assert_eq!(resources.svg_count(), 1);
        assert!(resources
            .svg_fragment(payload.svg_ref)
            .is_some_and(|fragment| fragment.contains("M2 2H14V14H2Z")));
    }

    #[test]
    fn svg_lowering_rejects_unsafe_font_document_without_interning_it() {
        let mut resources = ResourceArena::default();
        let result = decode_font_svg_glyph_payload(
            fixture_font(),
            0,
            fixture_glyph_id('\u{E102}'),
            &FontSvgGlyphDecodeOptions::new(
                TextSourceRange::new(0, 1),
                GlyphRange { start: 0, end: 1 },
            ),
            &mut resources,
        );

        assert!(matches!(
            result,
            Err(FontSvgGlyphDecodeError::UnsafeStaticSvg)
        ));
        assert_eq!(resources.svg_count(), 0);
    }

    #[test]
    fn svg_lowering_rejects_malformed_static_markup() {
        let mut resources = ResourceArena::default();
        let result = decode_font_svg_glyph_payload(
            fixture_font(),
            0,
            fixture_glyph_id('\u{E103}'),
            &FontSvgGlyphDecodeOptions::new(
                TextSourceRange::new(0, 1),
                GlyphRange { start: 0, end: 1 },
            ),
            &mut resources,
        );

        assert!(matches!(
            result,
            Err(FontSvgGlyphDecodeError::InvalidSvgXml)
        ));
        assert_eq!(resources.svg_count(), 0);
    }

    #[test]
    fn svg_lowering_rejects_shared_documents_until_glyph_selection_is_implemented() {
        let mut resources = ResourceArena::default();
        let result = decode_font_svg_glyph_payload(
            fixture_font(),
            0,
            fixture_glyph_id('\u{E104}'),
            &FontSvgGlyphDecodeOptions::new(
                TextSourceRange::new(0, 1),
                GlyphRange { start: 0, end: 1 },
            ),
            &mut resources,
        );

        assert!(matches!(
            result,
            Err(FontSvgGlyphDecodeError::SharedSvgDocument)
        ));
        assert_eq!(resources.svg_count(), 0);
    }

    #[test]
    fn producer_errors_have_stable_diagnostic_names() {
        assert_eq!(
            FontBitmapGlyphDecodeError::UnsupportedRasterFormat.as_str(),
            "unsupportedRasterFormat"
        );
        assert_eq!(
            FontSvgGlyphDecodeError::UnsafeStaticSvg.as_str(),
            "unsafeStaticSvg"
        );
    }

    #[test]
    fn collection_face_resolution_requires_one_exact_family_match() {
        assert_eq!(
            resolve_embedded_font_face_index(fixture_ttc(), "RHWP Exact Face One", None),
            Some(1)
        );
        assert_eq!(
            resolve_embedded_font_face_index(fixture_ttc(), "Missing Family", None),
            None
        );
    }

    fn text_run_for_embedded_font(text: &str) -> crate::renderer::render_tree::TextRunNode {
        crate::renderer::render_tree::TextRunNode {
            text: text.to_string(),
            style: crate::renderer::TextStyle {
                font_family: "substituted browser family".to_string(),
                font_size: 16.0,
                ..Default::default()
            },
            char_shape_id: Some(7),
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: false,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 12.0,
            field_marker: Default::default(),
            layout_positions: None,
            display_text: None,
        }
    }

    #[test]
    fn lowering_matches_char_shape_and_language_instead_of_css_family() {
        let bbox = BoundingBox::new(10.0, 20.0, 16.0, 16.0);
        let run = text_run_for_embedded_font("\u{E100}");
        let mut root = LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]);
        let mut resources = ResourceArena::default();
        let report = lower_font_native_glyph_sidecars(
            &mut root,
            &mut resources,
            &[EmbeddedFontFace {
                char_shape_id: 7,
                language_index: 0,
                family: "fixture family",
                alternate_family: None,
                bytes: fixture_font(),
                face_index: 0,
            }],
        );

        assert_eq!(report.emitted_glyph_runs, 1);
        assert_eq!(report.emitted_bitmap_glyphs, 1);
        let LayerNodeKind::Leaf { ops } = root.kind else {
            panic!("expected leaf");
        };
        assert!(matches!(
            ops.as_slice(),
            [
                PaintOp::TextRun { .. },
                PaintOp::GlyphRun { .. },
                PaintOp::GlyphOutline { .. }
            ]
        ));
        assert_eq!(resources.font_resources().blobs.len(), 1);
        assert_eq!(resources.font_resources().faces.len(), 1);
        assert_eq!(resources.font_blob_count(), 1);
    }

    #[test]
    fn lowering_preserves_exact_nonzero_collection_face_identity() {
        let bbox = BoundingBox::new(10.0, 20.0, 16.0, 16.0);
        let run = text_run_for_embedded_font("\u{E104}");
        let mut root = LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]);
        let mut resources = ResourceArena::default();

        let report = lower_font_native_glyph_sidecars(
            &mut root,
            &mut resources,
            &[EmbeddedFontFace {
                char_shape_id: 7,
                language_index: 0,
                family: "RHWP Exact Face One",
                alternate_family: None,
                bytes: fixture_ttc(),
                face_index: 1,
            }],
        );

        assert_eq!(report.emitted_glyph_runs, 1);
        assert_eq!(resources.font_resources().faces[0].face_index, 1);
        let LayerNodeKind::Leaf { ops } = root.kind else {
            panic!("expected leaf");
        };
        let PaintOp::GlyphRun { run, .. } = &ops[1] else {
            panic!("expected portable GlyphRun");
        };
        assert!(run.shape_key.font_instance.face_key.0.ends_with(":face:1"));
        assert_eq!(run.positions.len(), 1);
        assert_eq!(run.clusters.len(), 1);
    }

    #[test]
    fn portable_glyph_lowering_prepares_each_embedded_face_once() {
        let bbox = BoundingBox::new(10.0, 20.0, 16.0, 16.0);
        let mut root = LayerNode::leaf(
            bbox,
            None,
            vec![
                PaintOp::text_run(bbox, text_run_for_embedded_font("\u{E100}")),
                PaintOp::text_run(bbox, text_run_for_embedded_font("\u{E100}")),
            ],
        );
        let mut resources = ResourceArena::default();

        let report = lower_font_native_glyph_sidecars(
            &mut root,
            &mut resources,
            &[EmbeddedFontFace {
                char_shape_id: 7,
                language_index: 0,
                family: "fixture family",
                alternate_family: None,
                bytes: fixture_font(),
                face_index: 0,
            }],
        );

        assert_eq!(report.attempted_glyph_runs, 2);
        assert_eq!(report.prepared_glyph_font_faces, 1);
        assert_eq!(report.emitted_glyph_runs, 2);
        assert_eq!(resources.font_blob_count(), 1);
        assert_eq!(resources.font_resources().blobs.len(), 1);
        assert_eq!(resources.font_resources().faces.len(), 1);
    }

    #[test]
    fn portable_glyph_lowering_rejects_shaping_required_text() {
        for text in ["e\u{301}", "\u{1100}\u{1161}", "\u{0627}"] {
            let bbox = BoundingBox::new(10.0, 20.0, 40.0, 16.0);
            let run = text_run_for_embedded_font(text);
            let mut root = LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]);
            let mut resources = ResourceArena::default();

            let report = lower_font_native_glyph_sidecars(
                &mut root,
                &mut resources,
                &[EmbeddedFontFace {
                    char_shape_id: 7,
                    language_index: 0,
                    family: "fixture family",
                    alternate_family: None,
                    bytes: fixture_font(),
                    face_index: 0,
                }],
            );

            assert_eq!(report.emitted_glyph_runs, 0, "text={text:?}");
            assert_eq!(resources.font_blob_count(), 0, "text={text:?}");
        }
    }

    #[test]
    fn nominal_glyph_replay_scalar_is_fail_closed_by_script_and_category() {
        for character in ['A', '\u{AC00}', '\u{E104}'] {
            assert!(is_nominal_glyph_replay_scalar(character), "{character:?}");
        }
        for character in ['\u{0301}', '\u{1100}', '\u{0627}', '\u{0E81}', '\u{1F600}'] {
            assert!(!is_nominal_glyph_replay_scalar(character), "{character:?}");
        }
    }

    #[test]
    fn normal_lowering_keeps_text_fallback_for_unrepresented_run_styles() {
        for case in ["bold", "italic", "vertical", "ratio"] {
            let bbox = BoundingBox::new(10.0, 20.0, 16.0, 16.0);
            let mut run = text_run_for_embedded_font("\u{E100}");
            match case {
                "bold" => run.style.bold = true,
                "italic" => run.style.italic = true,
                "vertical" => run.is_vertical = true,
                "ratio" => run.style.ratio = 0.8,
                _ => unreachable!(),
            }
            let mut root = LayerNode::leaf(bbox, None, vec![PaintOp::text_run(bbox, run)]);
            let mut resources = ResourceArena::default();
            let report = lower_font_native_glyph_sidecars(
                &mut root,
                &mut resources,
                &[EmbeddedFontFace {
                    char_shape_id: 7,
                    language_index: 0,
                    family: "fixture family",
                    alternate_family: None,
                    bytes: fixture_font(),
                    face_index: 0,
                }],
            );
            assert_eq!(report.emitted_glyph_runs, 0, "case={case}");
            assert_eq!(report.emitted_bitmap_glyphs, 0, "case={case}");
            let LayerNodeKind::Leaf { ops } = root.kind else {
                panic!("expected leaf");
            };
            assert!(matches!(ops.as_slice(), [PaintOp::TextRun { .. }]));
        }
    }

    #[test]
    fn oversized_source_font_is_rejected_before_parsing_or_interning() {
        let mut oversized = fixture_font().to_vec();
        oversized.resize(MAX_FONT_NATIVE_SOURCE_BYTES + 1, 0);
        assert_eq!(
            resolve_embedded_font_face_index(&oversized, "fixture family", None),
            None
        );

        let mut resources = ResourceArena::default();
        let result = decode_font_bitmap_glyph_payload(
            &oversized,
            0,
            fixture_glyph_id('\u{E100}'),
            &FontBitmapGlyphDecodeOptions::new(
                16,
                TextSourceRange::new(0, 3),
                GlyphRange::new(0, 1),
                BoundingBox::new(10.0, 20.0, 16.0, 16.0),
            ),
            &mut resources,
        );
        assert!(matches!(
            result,
            Err(FontBitmapGlyphDecodeError::SourceFontTooLarge)
        ));
        assert_eq!(resources.image_count(), 0);
    }

    #[test]
    fn normal_lowering_enforces_page_sidecar_budget() {
        let bbox = BoundingBox::new(0.0, 0.0, 16.0, 16.0);
        let ops = (0..MAX_FONT_NATIVE_SIDECARS_PER_PAGE + 1)
            .map(|_| PaintOp::text_run(bbox, text_run_for_embedded_font("\u{E100}")))
            .collect();
        let mut root = LayerNode::leaf(bbox, None, ops);
        let mut resources = ResourceArena::default();
        let report = lower_font_native_glyph_sidecars(
            &mut root,
            &mut resources,
            &[EmbeddedFontFace {
                char_shape_id: 7,
                language_index: 0,
                family: "fixture family",
                alternate_family: None,
                bytes: fixture_font(),
                face_index: 0,
            }],
        );

        assert_eq!(
            report.emitted_bitmap_glyphs,
            MAX_FONT_NATIVE_SIDECARS_PER_PAGE
        );
        let LayerNodeKind::Leaf { ops } = root.kind else {
            panic!("expected leaf");
        };
        assert_eq!(
            ops.iter()
                .filter(|op| matches!(op, PaintOp::GlyphOutline { .. }))
                .count(),
            MAX_FONT_NATIVE_SIDECARS_PER_PAGE
        );
        assert_eq!(resources.image_count(), 1);
    }
}
