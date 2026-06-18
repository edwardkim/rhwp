use crate::error::HwpError;
use crate::model::ColorRef;
use crate::paint::{
    GlyphOutlinePayloadKind, GlyphRunOrientation, GlyphRunReplayEligibility,
    LayerGlyphOutlinePaint, LayerGlyphRunPaint, LayerNode, LayerNodeKind, PageLayerTree, PaintOp,
    ResourceArena, TextVariantKind, TextVariantQuality,
};
use std::collections::{BTreeMap, BTreeSet};

pub type LayerRenderResult<T> = Result<T, HwpError>;

/// visual layer tree를 backend 출력으로 재생한다.
pub trait LayerRenderer {
    fn render_page(&mut self, tree: &PageLayerTree) -> LayerRenderResult<()>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RasterRenderOptions {
    pub max_dimension: i32,
    pub max_pixels: u64,
    pub scale: f64,
    pub dpi: Option<f64>,
    pub transparent: bool,
    pub background_color: Option<ColorRef>,
    pub color_space: RasterColorSpace,
    pub format: RasterOutputFormat,
}

impl Default for RasterRenderOptions {
    fn default() -> Self {
        Self {
            max_dimension: 16_384,
            max_pixels: 67_108_864,
            scale: 1.0,
            dpi: None,
            transparent: true,
            background_color: None,
            color_space: RasterColorSpace::Srgb,
            format: RasterOutputFormat::Png,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterColorSpace {
    Srgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterOutputFormat {
    Png,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterRenderOutput {
    pub bytes: Vec<u8>,
    pub format: RasterOutputFormat,
    pub width: i32,
    pub height: i32,
    pub dpi: Option<f64>,
    pub color_space: RasterColorSpace,
}

/// visual layer tree를 raster 결과로 직접 내보내는 backend 계약.
pub trait LayerRasterRenderer {
    fn render_png(&self, tree: &PageLayerTree) -> LayerRenderResult<Vec<u8>> {
        self.render_png_with_options(tree, RasterRenderOptions::default())
    }

    fn render_png_with_options(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<Vec<u8>> {
        let mut png_options = options;
        png_options.format = RasterOutputFormat::Png;
        self.render_raster(tree, png_options)
            .map(|output| output.bytes)
    }

    fn render_raster(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRenderOutput>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantSelectionBackend {
    NativeSkia,
    CanvasKit,
    Canvas2D,
    Svg,
}

impl VariantSelectionBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NativeSkia => "nativeSkia",
            Self::CanvasKit => "canvasKit",
            Self::Canvas2D => "canvas2d",
            Self::Svg => "svg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantSelectedReason {
    GlyphRunStrictEligible,
    GlyphOutlineStrictProfile,
    DefaultTextRunFallback,
    NoSupportedVariant,
}

impl VariantSelectedReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GlyphRunStrictEligible => "glyphRunStrictEligible",
            Self::GlyphOutlineStrictProfile => "glyphOutlineStrictProfile",
            Self::DefaultTextRunFallback => "defaultTextRunFallback",
            Self::NoSupportedVariant => "noSupportedVariant",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariantRejectReason {
    BackendDoesNotSupportVariant,
    FontNotPortable,
    ExternalFontNotVerified,
    FontFaceMissing,
    FontBlobMissing,
    FontBlobNotPortable,
    FontBlobBytesMissing,
    FontBlobDataRefMismatch,
    FontBlobDigestMismatch,
    FaceIndexUnsupported,
    VariationUnsupported,
    GlyphIdOutOfRange,
    MissingGlyph,
    ClusterMismatch,
    UnsupportedPaintEffect,
    IncompleteVariantSet,
    VariantPartCountMismatch,
    VariantDuplicatePart,
    VariantPartsIncomplete,
    GlyphOutlineUnsupported,
    UnsupportedOutlinePayload,
    MixedGlyphOutlinePayload,
    EmptyGlyphOutlinePayload,
    GlyphOutlineStrokeStyleUnsupported,
    UnsupportedColorGlyph,
    UnsupportedBitmapGlyph,
    UnsupportedSvgGlyph,
    MissingGlyphPayloadResource,
    MixedPerGlyphAuthorityPending,
    GlyphTransformAuthorityPending,
    VerticalGlyphOrientationAuthorityPending,
    PositionAdjustedNotAllowed,
    PositionAdjustedResidualTooLarge,
}

impl VariantRejectReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackendDoesNotSupportVariant => "backendDoesNotSupportVariant",
            Self::FontNotPortable => "fontNotPortable",
            Self::ExternalFontNotVerified => "externalFontNotVerified",
            Self::FontFaceMissing => "fontFaceMissing",
            Self::FontBlobMissing => "fontBlobMissing",
            Self::FontBlobNotPortable => "fontBlobNotPortable",
            Self::FontBlobBytesMissing => "fontBlobBytesMissing",
            Self::FontBlobDataRefMismatch => "fontBlobDataRefMismatch",
            Self::FontBlobDigestMismatch => "fontBlobDigestMismatch",
            Self::FaceIndexUnsupported => "faceIndexUnsupported",
            Self::VariationUnsupported => "variationUnsupported",
            Self::GlyphIdOutOfRange => "glyphIdOutOfRange",
            Self::MissingGlyph => "missingGlyph",
            Self::ClusterMismatch => "clusterMismatch",
            Self::UnsupportedPaintEffect => "unsupportedPaintEffect",
            Self::IncompleteVariantSet => "incompleteVariantSet",
            Self::VariantPartCountMismatch => "variantPartCountMismatch",
            Self::VariantDuplicatePart => "variantDuplicatePart",
            Self::VariantPartsIncomplete => "variantPartsIncomplete",
            Self::GlyphOutlineUnsupported => "glyphOutlineUnsupported",
            Self::UnsupportedOutlinePayload => "unsupportedOutlinePayload",
            Self::MixedGlyphOutlinePayload => "mixedGlyphOutlinePayload",
            Self::EmptyGlyphOutlinePayload => "emptyGlyphOutlinePayload",
            Self::GlyphOutlineStrokeStyleUnsupported => "glyphOutlineStrokeStyleUnsupported",
            Self::UnsupportedColorGlyph => "unsupportedColorGlyph",
            Self::UnsupportedBitmapGlyph => "unsupportedBitmapGlyph",
            Self::UnsupportedSvgGlyph => "unsupportedSvgGlyph",
            Self::MissingGlyphPayloadResource => "missingGlyphPayloadResource",
            Self::MixedPerGlyphAuthorityPending => "mixedPerGlyphAuthorityPending",
            Self::GlyphTransformAuthorityPending => "glyphTransformAuthorityPending",
            Self::VerticalGlyphOrientationAuthorityPending => {
                "verticalGlyphOrientationAuthorityPending"
            }
            Self::PositionAdjustedNotAllowed => "positionAdjustedNotAllowed",
            Self::PositionAdjustedResidualTooLarge => "positionAdjustedResidualTooLarge",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextVariantSelectionOptions {
    pub backend: VariantSelectionBackend,
    pub prefer_strict_outline: bool,
    pub allow_position_adjusted: bool,
    pub max_position_adjusted_residual_px: f64,
    pub max_canvas_glyph_id: u32,
    pub allow_colrv0_color_layers: bool,
    /// Compatibility name for the first P19-supported COLRv1 graph subset:
    /// solid paths, single linear/radial/full-circle sweep gradient paths, and
    /// transform chains ending in one supported leaf.
    pub allow_colrv1_stage1_color_graph: bool,
    pub allow_bitmap_glyph: bool,
    pub allow_svg_glyph: bool,
}

impl Default for TextVariantSelectionOptions {
    fn default() -> Self {
        Self::canvaskit()
    }
}

impl TextVariantSelectionOptions {
    pub fn canvaskit() -> Self {
        Self {
            backend: VariantSelectionBackend::CanvasKit,
            prefer_strict_outline: false,
            allow_position_adjusted: true,
            max_position_adjusted_residual_px: 0.25,
            max_canvas_glyph_id: u16::MAX as u32,
            allow_colrv0_color_layers: false,
            allow_colrv1_stage1_color_graph: false,
            allow_bitmap_glyph: false,
            allow_svg_glyph: false,
        }
    }

    pub fn canvaskit_strict_outline() -> Self {
        Self {
            prefer_strict_outline: true,
            ..Self::canvaskit()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextVariantSelectionReport {
    pub backend: VariantSelectionBackend,
    pub equivalence_group: String,
    pub selected_variant_id: Option<String>,
    pub selected_variant_kind: Option<TextVariantKind>,
    pub selected_reason: VariantSelectedReason,
    pub fallback_required: bool,
    pub rejected_variants: Vec<TextVariantRejectReport>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextVariantRejectReport {
    pub variant_id: String,
    pub variant_kind: TextVariantKind,
    pub reasons: Vec<VariantRejectReason>,
}

pub fn analyze_text_variant_selection(
    tree: &PageLayerTree,
    options: TextVariantSelectionOptions,
) -> Vec<TextVariantSelectionReport> {
    let mut groups = BTreeMap::<String, TextVariantGroupState>::new();
    let mut next_order = 0usize;
    collect_text_variant_groups(&tree.root, &mut groups, &mut next_order);
    groups
        .into_iter()
        .map(|(equivalence_group, group)| group.finish(equivalence_group, options, &tree.resources))
        .collect()
}

#[derive(Debug, Default)]
struct TextVariantGroupState {
    fallback_present: bool,
    variants: BTreeMap<String, TextVariantCandidate>,
}

impl TextVariantGroupState {
    fn finish(
        self,
        equivalence_group: String,
        options: TextVariantSelectionOptions,
        resources: &ResourceArena,
    ) -> TextVariantSelectionReport {
        let mut evaluated = self
            .variants
            .into_values()
            .map(|candidate| {
                let reasons = candidate.reject_reasons(options, resources);
                EvaluatedTextVariantCandidate { candidate, reasons }
            })
            .collect::<Vec<_>>();
        evaluated.sort_by_key(|evaluated| evaluated.candidate.order);
        let rejected_variants = evaluated
            .iter()
            .filter(|evaluated| !evaluated.reasons.is_empty())
            .map(|evaluated| TextVariantRejectReport {
                variant_id: evaluated.candidate.variant_id.clone(),
                variant_kind: evaluated.candidate.variant_kind,
                reasons: evaluated.reasons.clone(),
            })
            .collect::<Vec<_>>();
        let outline_selection = options
            .prefer_strict_outline
            .then(|| {
                evaluated.iter().find(|evaluated| {
                    evaluated.candidate.variant_kind == TextVariantKind::GlyphOutline
                        && evaluated.reasons.is_empty()
                })
            })
            .flatten();
        let glyph_selection = evaluated.iter().find(|evaluated| {
            evaluated.candidate.variant_kind == TextVariantKind::GlyphRun
                && evaluated.reasons.is_empty()
        });
        let fallback_outline_selection = (!options.prefer_strict_outline)
            .then(|| {
                evaluated.iter().find(|evaluated| {
                    evaluated.candidate.variant_kind == TextVariantKind::GlyphOutline
                        && evaluated.reasons.is_empty()
                })
            })
            .flatten();
        let selected = outline_selection
            .or(glyph_selection)
            .or(fallback_outline_selection);
        if let Some(selected) = selected {
            return TextVariantSelectionReport {
                backend: options.backend,
                equivalence_group,
                selected_variant_id: Some(selected.candidate.variant_id.clone()),
                selected_variant_kind: Some(selected.candidate.variant_kind),
                selected_reason: selected_reason_for_variant(selected.candidate.variant_kind),
                fallback_required: false,
                rejected_variants,
            };
        }
        TextVariantSelectionReport {
            backend: options.backend,
            equivalence_group,
            selected_variant_id: self.fallback_present.then(|| "textRun".to_string()),
            selected_variant_kind: self.fallback_present.then_some(TextVariantKind::TextRun),
            selected_reason: if self.fallback_present {
                VariantSelectedReason::DefaultTextRunFallback
            } else {
                VariantSelectedReason::NoSupportedVariant
            },
            fallback_required: true,
            rejected_variants,
        }
    }
}

#[derive(Debug)]
struct EvaluatedTextVariantCandidate {
    candidate: TextVariantCandidate,
    reasons: Vec<VariantRejectReason>,
}

fn selected_reason_for_variant(variant_kind: TextVariantKind) -> VariantSelectedReason {
    match variant_kind {
        TextVariantKind::GlyphRun => VariantSelectedReason::GlyphRunStrictEligible,
        TextVariantKind::GlyphOutline => VariantSelectedReason::GlyphOutlineStrictProfile,
        TextVariantKind::TextRun => {
            unreachable!("TextRun fallback is tracked through fallback_present, not candidates")
        }
    }
}

#[derive(Debug)]
struct TextVariantCandidate {
    order: usize,
    variant_id: String,
    variant_kind: TextVariantKind,
    part_counts: BTreeSet<u32>,
    present_parts: BTreeSet<u32>,
    duplicate_part: bool,
    glyph_runs: Vec<LayerGlyphRunPaint>,
    glyph_outlines: Vec<LayerGlyphOutlinePaint>,
}

impl TextVariantCandidate {
    fn new(order: usize, variant_id: String, variant_kind: TextVariantKind) -> Self {
        Self {
            order,
            variant_id,
            variant_kind,
            part_counts: BTreeSet::new(),
            present_parts: BTreeSet::new(),
            duplicate_part: false,
            glyph_runs: Vec::new(),
            glyph_outlines: Vec::new(),
        }
    }

    fn add_glyph_run(&mut self, run: &LayerGlyphRunPaint) {
        self.part_counts.insert(run.variant.part_count);
        self.duplicate_part |= !self.present_parts.insert(run.variant.part_index);
        self.glyph_runs.push(run.clone());
    }

    fn add_glyph_outline(&mut self, outline: &LayerGlyphOutlinePaint) {
        self.part_counts.insert(outline.variant.part_count);
        self.duplicate_part |= !self.present_parts.insert(outline.variant.part_index);
        self.glyph_outlines.push(outline.clone());
    }

    fn reject_reasons(
        &self,
        options: TextVariantSelectionOptions,
        resources: &ResourceArena,
    ) -> Vec<VariantRejectReason> {
        let mut reasons = BTreeSet::<VariantRejectReason>::new();
        self.collect_structure_reasons(&mut reasons);
        match self.variant_kind {
            TextVariantKind::TextRun => {
                unreachable!("TextRun fallback is tracked through fallback_present, not candidates")
            }
            TextVariantKind::GlyphRun => {
                if !matches!(
                    options.backend,
                    VariantSelectionBackend::CanvasKit | VariantSelectionBackend::NativeSkia
                ) {
                    reasons.insert(VariantRejectReason::BackendDoesNotSupportVariant);
                }
                for run in &self.glyph_runs {
                    collect_glyph_run_reject_reasons(run, options, resources, &mut reasons);
                }
            }
            TextVariantKind::GlyphOutline => {
                if matches!(options.backend, VariantSelectionBackend::Canvas2D) {
                    reasons.insert(VariantRejectReason::BackendDoesNotSupportVariant);
                }
                for outline in &self.glyph_outlines {
                    collect_glyph_outline_reject_reasons(outline, options, resources, &mut reasons);
                }
            }
        }
        reasons.into_iter().collect()
    }

    fn collect_structure_reasons(&self, reasons: &mut BTreeSet<VariantRejectReason>) {
        if self.part_counts.is_empty() || self.part_counts.contains(&0) {
            reasons.insert(VariantRejectReason::IncompleteVariantSet);
        }
        if self.part_counts.len() > 1 {
            reasons.insert(VariantRejectReason::VariantPartCountMismatch);
        }
        if self.duplicate_part {
            reasons.insert(VariantRejectReason::VariantDuplicatePart);
        }
        let expected = self.part_counts.iter().next().copied().unwrap_or_default();
        if expected == 0
            || self.present_parts.len() as u32 != expected
            || !(0..expected).all(|index| self.present_parts.contains(&index))
        {
            reasons.insert(VariantRejectReason::VariantPartsIncomplete);
        }
    }
}

fn collect_text_variant_groups(
    node: &LayerNode,
    groups: &mut BTreeMap<String, TextVariantGroupState>,
    next_order: &mut usize,
) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                collect_text_variant_groups(child, groups, next_order);
            }
        }
        LayerNodeKind::ClipRect { child, .. } => {
            collect_text_variant_groups(child, groups, next_order);
        }
        LayerNodeKind::Leaf { ops } => {
            let fallback_present = ops.iter().any(|op| matches!(op, PaintOp::TextRun { .. }));
            for op in ops {
                match op {
                    PaintOp::GlyphRun { run, .. } => {
                        let group = groups
                            .entry(run.variant.equivalence_group.clone())
                            .or_default();
                        group.fallback_present |= fallback_present;
                        let candidate = group
                            .variants
                            .entry(run.variant.variant_id.clone())
                            .or_insert_with(|| {
                                let order = *next_order;
                                *next_order = (*next_order).saturating_add(1);
                                TextVariantCandidate::new(
                                    order,
                                    run.variant.variant_id.clone(),
                                    run.variant.variant_kind,
                                )
                            });
                        candidate.add_glyph_run(run);
                    }
                    PaintOp::GlyphOutline { outline, .. } => {
                        let group = groups
                            .entry(outline.variant.equivalence_group.clone())
                            .or_default();
                        group.fallback_present |= fallback_present;
                        let candidate = group
                            .variants
                            .entry(outline.variant.variant_id.clone())
                            .or_insert_with(|| {
                                let order = *next_order;
                                *next_order = (*next_order).saturating_add(1);
                                TextVariantCandidate::new(
                                    order,
                                    outline.variant.variant_id.clone(),
                                    outline.variant.variant_kind,
                                )
                            });
                        candidate.add_glyph_outline(outline);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn collect_glyph_run_reject_reasons(
    run: &LayerGlyphRunPaint,
    options: TextVariantSelectionOptions,
    resources: &ResourceArena,
    reasons: &mut BTreeSet<VariantRejectReason>,
) {
    if !run.paint_style.is_fill_only_glyph_replay() {
        reasons.insert(VariantRejectReason::UnsupportedPaintEffect);
    }
    if run.glyph_transforms.is_some() {
        reasons.insert(VariantRejectReason::GlyphTransformAuthorityPending);
    }
    match run.orientation {
        GlyphRunOrientation::Horizontal => {}
        GlyphRunOrientation::MixedPerGlyph => {
            reasons.insert(VariantRejectReason::MixedPerGlyphAuthorityPending);
        }
        GlyphRunOrientation::VerticalUpright | GlyphRunOrientation::VerticalSideways => {
            reasons.insert(VariantRejectReason::VerticalGlyphOrientationAuthorityPending);
        }
    }
    if matches!(
        options.backend,
        VariantSelectionBackend::CanvasKit | VariantSelectionBackend::NativeSkia
    ) {
        if !run.shape_key.font_instance.variations.is_empty() {
            reasons.insert(VariantRejectReason::VariationUnsupported);
        }
        if matches!(
            run.diagnostics.replay_eligibility,
            GlyphRunReplayEligibility::Portable
        ) {
            collect_glyph_run_font_resource_reject_reasons(run, resources, reasons);
        }
    }
    collect_text_variant_diagnostics_reject_reasons(&run.diagnostics, options, reasons);
    if run
        .glyph_ids
        .iter()
        .any(|glyph_id| *glyph_id > options.max_canvas_glyph_id)
    {
        reasons.insert(VariantRejectReason::GlyphIdOutOfRange);
    }
}

fn collect_glyph_run_font_resource_reject_reasons(
    run: &LayerGlyphRunPaint,
    resources: &ResourceArena,
    reasons: &mut BTreeSet<VariantRejectReason>,
) {
    let font_resources = resources.font_resources();
    let Some(face) = font_resources
        .faces
        .iter()
        .find(|face| face.id == run.shape_key.font_instance.face_key)
    else {
        reasons.insert(VariantRejectReason::FontFaceMissing);
        return;
    };

    if face.face_index != 0 {
        reasons.insert(VariantRejectReason::FaceIndexUnsupported);
    }

    let Some(blob) = font_resources
        .blobs
        .iter()
        .find(|blob| blob.id == face.blob_key)
    else {
        reasons.insert(VariantRejectReason::FontBlobMissing);
        return;
    };

    let crate::paint::FontPortability::PortableBlob { digest, data_ref } = &blob.portability else {
        reasons.insert(VariantRejectReason::FontBlobNotPortable);
        return;
    };

    if blob.data_ref.as_ref() != Some(data_ref) {
        reasons.insert(VariantRejectReason::FontBlobDataRefMismatch);
    }

    match resources.font_blob_bytes_for_ref(data_ref) {
        Some(bytes) => {
            let actual_digest = crate::paint::resource_digest_hex(bytes);
            if !font_digest_matches_resource_digest(digest, &actual_digest)
                || !blob.digest.as_ref().is_none_or(|digest| {
                    font_digest_matches_resource_digest(digest, &actual_digest)
                })
            {
                reasons.insert(VariantRejectReason::FontBlobDigestMismatch);
            }
        }
        None => {
            reasons.insert(VariantRejectReason::FontBlobBytesMissing);
        }
    }
}

fn font_digest_matches_resource_digest(digest: &crate::paint::FontDigest, actual: &str) -> bool {
    digest.algorithm == crate::paint::RESOURCE_KEY_ALGORITHM && digest.value == actual
}

fn collect_glyph_outline_reject_reasons(
    outline: &LayerGlyphOutlinePaint,
    options: TextVariantSelectionOptions,
    resources: &ResourceArena,
    reasons: &mut BTreeSet<VariantRejectReason>,
) {
    if !outline.has_exclusive_payload_family() {
        reasons.insert(VariantRejectReason::MixedGlyphOutlinePayload);
    }
    if matches!(
        outline.payload_kind,
        GlyphOutlinePayloadKind::MonochromeFill | GlyphOutlinePayloadKind::MonochromeFillStroke
    ) && outline.paths.is_empty()
    {
        reasons.insert(VariantRejectReason::EmptyGlyphOutlinePayload);
    }
    if !outline.paint_style.is_fill_only_glyph_replay() {
        reasons.insert(VariantRejectReason::UnsupportedPaintEffect);
    }
    match outline.payload_kind {
        GlyphOutlinePayloadKind::MonochromeFill => {
            if outline.stroke.is_some() {
                reasons.insert(VariantRejectReason::UnsupportedOutlinePayload);
            }
        }
        GlyphOutlinePayloadKind::MonochromeFillStroke => {
            if !outline
                .stroke
                .as_ref()
                .is_some_and(|stroke| stroke.is_strict_subset())
            {
                reasons.insert(VariantRejectReason::GlyphOutlineStrokeStyleUnsupported);
            }
        }
        GlyphOutlinePayloadKind::ColorLayers => match outline.color_layers.as_ref() {
            Some(color_layers)
                if color_layers.has_colrv0_resolved_layer_contract()
                    && options.allow_colrv0_color_layers => {}
            Some(color_layers)
                if color_layers.has_colrv1_supported_graph_contract()
                    && options.allow_colrv1_stage1_color_graph => {}
            _ => {
                reasons.insert(VariantRejectReason::UnsupportedColorGlyph);
            }
        },
        GlyphOutlinePayloadKind::BitmapGlyph => match outline.bitmap_glyph.as_ref() {
            Some(bitmap_glyph) if !bitmap_glyph.has_strict_visual_contract() => {
                reasons.insert(VariantRejectReason::UnsupportedBitmapGlyph);
            }
            Some(_) if !options.allow_bitmap_glyph => {
                reasons.insert(VariantRejectReason::UnsupportedBitmapGlyph);
            }
            Some(bitmap_glyph) if resources.image_bytes(bitmap_glyph.image_ref).is_none() => {
                reasons.insert(VariantRejectReason::MissingGlyphPayloadResource);
            }
            Some(_) => {}
            None => {
                reasons.insert(VariantRejectReason::UnsupportedBitmapGlyph);
            }
        },
        GlyphOutlinePayloadKind::SvgGlyph => match outline.svg_glyph.as_ref() {
            Some(svg_glyph) if !svg_glyph.has_static_sanitized_contract() => {
                reasons.insert(VariantRejectReason::UnsupportedSvgGlyph);
            }
            Some(_) if !options.allow_svg_glyph => {
                reasons.insert(VariantRejectReason::UnsupportedSvgGlyph);
            }
            Some(svg_glyph) if resources.svg_fragment(svg_glyph.svg_ref).is_none() => {
                reasons.insert(VariantRejectReason::MissingGlyphPayloadResource);
            }
            Some(_) => {}
            None => {
                reasons.insert(VariantRejectReason::UnsupportedSvgGlyph);
            }
        },
    }
    collect_text_variant_diagnostics_reject_reasons(&outline.diagnostics, options, reasons);
}

fn collect_text_variant_diagnostics_reject_reasons(
    diagnostics: &crate::paint::GlyphRunDiagnostics,
    options: TextVariantSelectionOptions,
    reasons: &mut BTreeSet<VariantRejectReason>,
) {
    match diagnostics.replay_eligibility {
        GlyphRunReplayEligibility::Portable => {}
        GlyphRunReplayEligibility::ConditionalExternalFont => {
            reasons.insert(VariantRejectReason::ExternalFontNotVerified);
        }
        GlyphRunReplayEligibility::LocalDiagnosticOnly
        | GlyphRunReplayEligibility::NotReplayable => {
            reasons.insert(VariantRejectReason::FontNotPortable);
        }
    }
    match diagnostics.quality {
        TextVariantQuality::Exact => {}
        TextVariantQuality::PositionAdjusted if !options.allow_position_adjusted => {
            reasons.insert(VariantRejectReason::PositionAdjustedNotAllowed);
        }
        TextVariantQuality::PositionAdjusted
            if diagnostics.max_residual_after_adjustment_px
                <= options.max_position_adjusted_residual_px => {}
        TextVariantQuality::PositionAdjusted => {
            reasons.insert(VariantRejectReason::PositionAdjustedResidualTooLarge);
        }
        TextVariantQuality::Approximate
        | TextVariantQuality::DiagnosticOnly
        | TextVariantQuality::Omitted => {
            reasons.insert(VariantRejectReason::UnsupportedPaintEffect);
        }
    }
    if diagnostics.missing_glyph_count > 0 {
        reasons.insert(VariantRejectReason::MissingGlyph);
    }
    if diagnostics.cluster_mismatch_count > 0 {
        reasons.insert(VariantRejectReason::ClusterMismatch);
    }
    if diagnostics.used_fallback_font_count > 0 {
        reasons.insert(VariantRejectReason::FontNotPortable);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::{
        font_blob_resource_key, resource_digest_hex, BinaryResourceKind, BinaryResourceRef,
        BitmapGlyphFiltering, BitmapGlyphPayload, BitmapGlyphScalingPolicy, ColorGlyphFormat,
        ColorGradientStop, ColorLayersPayload, ColorLinearGradient, ColorPaintGraphNode,
        ColorPaintGraphNodeKind, ColorPaintGraphPayload, ColorPaintLinearGradientPathNode,
        ColorPaintRadialGradientPathNode, ColorPaintSolidPathNode, ColorPaintSweepGradientPathNode,
        ColorRadialGradient, ColorSweepGradient, FontBlobKey, FontBlobResource, FontColorGlyphRef,
        FontDigest, FontFaceKey, FontFaceResource, FontFallbackPolicyId, FontInstanceKey,
        FontPortability, FontResourceSource, GlyphCluster, GlyphOutlineFillRule,
        GlyphOutlinePaintOrder, GlyphOutlinePayloadKind, GlyphOutlineStrokeCap,
        GlyphOutlineStrokeJoin, GlyphOutlineStrokeStyle, GlyphRange, GlyphRunDiagnostics,
        GlyphRunOrientation, GlyphTransform, ImageResourceId, LayerAffineTransform,
        LayerGlyphOutlinePath, LayerNode, LayerPoint, LayerVector, PaintTextStyle,
        PaintVariantMeta, ResolvedColor, ResourceArena, ScriptTag, ShapeKey, ShapingEngineId,
        SvgGlyphPayload, SvgResourceId, TextDirection, TextRunPlacement, TextSourceId,
        TextSourceRange, TextSourceSpan, VariationAxisValue, WritingMode,
    };
    use crate::renderer::render_tree::{BoundingBox, FieldMarkerType, TextRunNode};
    use crate::renderer::{PathCommand, TextStyle};

    fn bbox() -> BoundingBox {
        BoundingBox::new(0.0, 0.0, 24.0, 24.0)
    }

    fn text_op() -> PaintOp {
        PaintOp::TextRun {
            bbox: bbox(),
            run: TextRunNode {
                text: "A".to_string(),
                style: TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
                    ..Default::default()
                },
                char_shape_id: None,
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
                field_marker: FieldMarkerType::None,
            },
        }
    }

    fn variant(kind: TextVariantKind, id: &str) -> PaintVariantMeta {
        let mut variant = PaintVariantMeta::text_run_default("text-0");
        variant.variant_id = id.to_string();
        variant.variant_kind = kind;
        variant.is_default_fallback = false;
        variant.requires = match kind {
            TextVariantKind::GlyphRun => {
                vec!["fontResources".to_string(), "text.glyphRun".to_string()]
            }
            TextVariantKind::GlyphOutline => {
                variant.anchor_op_id = Some("text-0".to_string());
                vec![
                    "text.glyphOutline".to_string(),
                    "text.glyphOutline.strictSidecar".to_string(),
                ]
            }
            TextVariantKind::TextRun => Vec::new(),
        };
        variant.quality = Some(TextVariantQuality::Exact);
        variant
    }

    fn diagnostics() -> GlyphRunDiagnostics {
        GlyphRunDiagnostics {
            quality: TextVariantQuality::Exact,
            replay_eligibility: GlyphRunReplayEligibility::Portable,
            strict_visual_eligible: true,
            max_origin_delta_px: 0.0,
            max_advance_delta_px: 0.0,
            max_residual_after_adjustment_px: 0.0,
            cluster_mismatch_count: 0,
            missing_glyph_count: 0,
            used_fallback_font_count: 0,
            reason: None,
        }
    }

    fn source() -> TextSourceSpan {
        TextSourceSpan {
            id: TextSourceId(0),
            utf8_range: TextSourceRange::new(0, 1),
            utf16_range: TextSourceRange::new(0, 1),
            stable_source_key: None,
        }
    }

    fn placement() -> TextRunPlacement {
        TextRunPlacement {
            run_to_page: LayerAffineTransform {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                e: 0.0,
                f: 12.0,
            },
            baseline_y: 0.0,
        }
    }

    fn shape_key() -> ShapeKey {
        ShapeKey {
            font_instance: FontInstanceKey {
                face_key: FontFaceKey("face-0".to_string()),
                size_px: 12.0,
                variations: Vec::new(),
                synthetic_bold: false,
                synthetic_italic: false,
            },
            direction: TextDirection::Ltr,
            writing_mode: WritingMode::HorizontalTb,
            script: Some(ScriptTag("DFLT".to_string())),
            language: None,
            features: Vec::new(),
            shaping_engine: ShapingEngineId("test".to_string()),
            fallback_policy: FontFallbackPolicyId("none".to_string()),
        }
    }

    fn glyph_run(mut diagnostics: GlyphRunDiagnostics, glyph_id: u32) -> PaintOp {
        diagnostics.strict_visual_eligible = diagnostics.strict_visual_eligible
            && diagnostics.missing_glyph_count == 0
            && diagnostics.cluster_mismatch_count == 0;
        PaintOp::GlyphRun {
            bbox: bbox(),
            run: Box::new(LayerGlyphRunPaint {
                source: source(),
                variant: variant(TextVariantKind::GlyphRun, "glyphRun"),
                paint_style: PaintTextStyle::from(&TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
                    ..Default::default()
                }),
                shape_key: shape_key(),
                placement: placement(),
                glyph_ids: vec![glyph_id],
                positions: vec![LayerPoint { x: 0.0, y: 0.0 }],
                advances: None,
                clusters: vec![GlyphCluster {
                    source_range_utf8: TextSourceRange::new(0, 1),
                    source_range_utf16: Some(TextSourceRange::new(0, 1)),
                    text_range_utf8: Some(TextSourceRange::new(0, 1)),
                    glyph_range: GlyphRange::new(0, 1),
                    flags: Vec::new(),
                }],
                direction: TextDirection::Ltr,
                bidi_level: None,
                writing_mode: WritingMode::HorizontalTb,
                orientation: GlyphRunOrientation::Horizontal,
                glyph_transforms: None,
                diagnostics,
            }),
        }
    }

    fn glyph_outline(stroke: Option<GlyphOutlineStrokeStyle>) -> PaintOp {
        PaintOp::GlyphOutline {
            bbox: bbox(),
            outline: Box::new(LayerGlyphOutlinePaint {
                source: source(),
                variant: variant(TextVariantKind::GlyphOutline, "glyphOutline"),
                payload_kind: if stroke.is_some() {
                    GlyphOutlinePayloadKind::MonochromeFillStroke
                } else {
                    GlyphOutlinePayloadKind::MonochromeFill
                },
                color_layers: None,
                bitmap_glyph: None,
                svg_glyph: None,
                paint_style: PaintTextStyle::from(&TextStyle {
                    font_family: "Test".to_string(),
                    font_size: 12.0,
                    shade_color: 0x00FF_FFFF,
                    ..Default::default()
                }),
                placement: placement(),
                paths: vec![LayerGlyphOutlinePath {
                    glyph_id: 42,
                    source_range_utf8: TextSourceRange::new(0, 1),
                    glyph_range: GlyphRange::new(0, 1),
                    commands: vec![
                        PathCommand::MoveTo(0.0, 0.0),
                        PathCommand::LineTo(10.0, 0.0),
                        PathCommand::LineTo(10.0, 10.0),
                        PathCommand::ClosePath,
                    ],
                    fill_rule: GlyphOutlineFillRule::NonZero,
                }],
                stroke,
                diagnostics: diagnostics(),
            }),
        }
    }

    fn color_layers_payload(kind: ColorPaintGraphNodeKind) -> ColorLayersPayload {
        let source_font_ref = FontColorGlyphRef {
            face_key: Some("fixture-face".to_string()),
            glyph_id: Some(42),
            palette_index: Some(0),
            color_format: Some(ColorGlyphFormat::ColrV1),
        };
        let source_range_utf8 = Some(TextSourceRange::new(0, 1));
        let glyph_range = Some(GlyphRange::new(0, 1));
        let black = ResolvedColor {
            color_space: Some("sRGB".to_string()),
            rgba: [0.0, 0.0, 0.0, 1.0],
        };
        let commands = vec![
            PathCommand::MoveTo(0.0, 0.0),
            PathCommand::LineTo(10.0, 0.0),
            PathCommand::LineTo(10.0, 10.0),
            PathCommand::ClosePath,
        ];
        let red = ResolvedColor {
            color_space: Some("sRGB".to_string()),
            rgba: [1.0, 0.0, 0.0, 1.0],
        };
        let stops = vec![
            ColorGradientStop {
                offset: 0.0,
                color: black.clone(),
            },
            ColorGradientStop {
                offset: 1.0,
                color: red,
            },
        ];
        let (solid_path, linear_gradient_path, radial_gradient_path, sweep_gradient_path) =
            match kind {
                ColorPaintGraphNodeKind::SolidPath => (
                    Some(ColorPaintSolidPathNode {
                        commands: commands.clone(),
                        fill: black.clone(),
                        fill_rule: GlyphOutlineFillRule::NonZero,
                        source_glyph_id: Some(42),
                        palette_index: Some(0),
                    }),
                    None,
                    None,
                    None,
                ),
                ColorPaintGraphNodeKind::LinearGradientPath => (
                    None,
                    Some(ColorPaintLinearGradientPathNode {
                        commands: commands.clone(),
                        gradient: ColorLinearGradient {
                            x0: 0.0,
                            y0: 0.0,
                            x1: 10.0,
                            y1: 10.0,
                            stops: stops.clone(),
                        },
                        fill_rule: GlyphOutlineFillRule::NonZero,
                        source_glyph_id: Some(42),
                        palette_index: Some(0),
                    }),
                    None,
                    None,
                ),
                ColorPaintGraphNodeKind::RadialGradientPath => (
                    None,
                    None,
                    Some(ColorPaintRadialGradientPathNode {
                        commands: commands.clone(),
                        gradient: ColorRadialGradient {
                            cx: 5.0,
                            cy: 5.0,
                            radius: 5.0,
                            stops: stops.clone(),
                        },
                        fill_rule: GlyphOutlineFillRule::NonZero,
                        source_glyph_id: Some(42),
                        palette_index: Some(0),
                    }),
                    None,
                ),
                ColorPaintGraphNodeKind::SweepGradientPath => (
                    None,
                    None,
                    None,
                    Some(ColorPaintSweepGradientPathNode {
                        commands,
                        gradient: ColorSweepGradient {
                            cx: 5.0,
                            cy: 5.0,
                            start_angle_degrees: 0.0,
                            end_angle_degrees: 360.0,
                            stops,
                        },
                        fill_rule: GlyphOutlineFillRule::NonZero,
                        source_glyph_id: Some(42),
                        palette_index: Some(0),
                    }),
                ),
                _ => (None, None, None, None),
            };
        ColorLayersPayload {
            color_format: ColorGlyphFormat::ColrV1,
            source_font_ref: Some(source_font_ref.clone()),
            palette_ref: None,
            layers: Vec::new(),
            paint_graph: Some(ColorPaintGraphPayload {
                root_node_id: 0,
                nodes: vec![ColorPaintGraphNode {
                    node_id: 0,
                    kind,
                    solid_path,
                    linear_gradient_path,
                    radial_gradient_path,
                    sweep_gradient_path,
                    transform: None,
                    source_range_utf8,
                    glyph_range,
                    source_font_ref: Some(source_font_ref),
                }],
            }),
            source_range_utf8,
            glyph_range,
        }
    }

    fn color_layers_outline(kind: ColorPaintGraphNodeKind) -> PaintOp {
        let mut op = glyph_outline(None);
        if let PaintOp::GlyphOutline { outline, .. } = &mut op {
            outline.payload_kind = GlyphOutlinePayloadKind::ColorLayers;
            outline.paths.clear();
            outline.color_layers = Some(color_layers_payload(kind));
            outline
                .variant
                .requires
                .push("text.glyphOutline.colorLayers".to_string());
            outline
                .variant
                .requires
                .push("text.glyphOutline.colorLayers.colrV1".to_string());
        }
        op
    }

    fn bitmap_glyph_outline() -> PaintOp {
        let mut op = glyph_outline(None);
        if let PaintOp::GlyphOutline { outline, .. } = &mut op {
            outline.payload_kind = GlyphOutlinePayloadKind::BitmapGlyph;
            outline.paths.clear();
            outline.bitmap_glyph = Some(BitmapGlyphPayload {
                image_ref: ImageResourceId(0),
                source_range_utf8: TextSourceRange::new(0, 1),
                glyph_range: GlyphRange::new(0, 1),
                placement: BoundingBox::new(0.0, 0.0, 12.0, 12.0),
                alpha_premultiplied: true,
                scaling_policy: BitmapGlyphScalingPolicy::SourceExact,
                filtering: BitmapGlyphFiltering::Linear,
                transform_to_run: None,
            });
            outline
                .variant
                .requires
                .push("text.glyphOutline.bitmapGlyph".to_string());
        }
        op
    }

    fn svg_glyph_outline() -> PaintOp {
        let mut op = glyph_outline(None);
        if let PaintOp::GlyphOutline { outline, .. } = &mut op {
            outline.payload_kind = GlyphOutlinePayloadKind::SvgGlyph;
            outline.paths.clear();
            outline.svg_glyph = Some(SvgGlyphPayload {
                svg_ref: SvgResourceId(0),
                source_range_utf8: TextSourceRange::new(0, 1),
                glyph_range: GlyphRange::new(0, 1),
                view_box: BoundingBox::new(0.0, 0.0, 10.0, 10.0),
                intrinsic_size: Some(LayerVector { dx: 10.0, dy: 10.0 }),
                static_sanitized: true,
                script_allowed: false,
                animation_allowed: false,
                external_resources_allowed: false,
                interactivity_allowed: false,
                transform_to_run: None,
            });
            outline
                .variant
                .requires
                .push("text.glyphOutline.svgGlyph".to_string());
        }
        op
    }

    fn tree(ops: Vec<PaintOp>) -> PageLayerTree {
        PageLayerTree::new(100.0, 100.0, LayerNode::leaf(bbox(), None, ops))
    }

    fn add_portable_font_resources(resources: &mut ResourceArena) {
        let font_bytes = [0_u8, 1, 2, 3];
        resources.intern_font_blob_bytes(&font_bytes);
        let blob_key = FontBlobKey("blob-0".to_string());
        let face_key = FontFaceKey("face-0".to_string());
        let digest_value = resource_digest_hex(font_bytes);
        let digest = FontDigest {
            algorithm: crate::paint::RESOURCE_KEY_ALGORITHM.to_string(),
            value: digest_value.clone(),
        };
        let data_ref = BinaryResourceRef {
            kind: BinaryResourceKind::FontBlob,
            id: font_blob_resource_key(font_bytes.len(), &digest_value),
        };
        resources.font_resources_mut().blobs.push(FontBlobResource {
            id: blob_key.clone(),
            digest: Some(digest.clone()),
            source: FontResourceSource::Embedded,
            data_ref: Some(data_ref.clone()),
            portability: FontPortability::PortableBlob { digest, data_ref },
        });
        resources.font_resources_mut().faces.push(FontFaceResource {
            id: face_key,
            blob_key,
            face_index: 0,
            postscript_name: None,
            family_names: Vec::new(),
            style_names: Vec::new(),
            weight_class: None,
            width_class: None,
            italic: None,
        });
    }

    fn first_report(
        ops: Vec<PaintOp>,
        options: TextVariantSelectionOptions,
    ) -> TextVariantSelectionReport {
        let mut tree = tree(ops);
        add_portable_font_resources(&mut tree.resources);
        analyze_text_variant_selection(&tree, options)
            .into_iter()
            .next()
            .unwrap()
    }

    fn first_report_with_resource_setup(
        ops: Vec<PaintOp>,
        options: TextVariantSelectionOptions,
        setup: impl FnOnce(&mut ResourceArena),
    ) -> TextVariantSelectionReport {
        let mut tree = tree(ops);
        setup(&mut tree.resources);
        analyze_text_variant_selection(&tree, options)
            .into_iter()
            .next()
            .unwrap()
    }

    fn native_skia_options() -> TextVariantSelectionOptions {
        TextVariantSelectionOptions {
            backend: VariantSelectionBackend::NativeSkia,
            ..TextVariantSelectionOptions::canvaskit()
        }
    }

    #[test]
    fn canvaskit_selects_strict_glyph_run() {
        let report = first_report(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            TextVariantSelectionOptions::canvaskit(),
        );
        assert_eq!(report.selected_variant_id.as_deref(), Some("glyphRun"));
        assert_eq!(
            report.selected_reason,
            VariantSelectedReason::GlyphRunStrictEligible
        );
        assert!(!report.fallback_required);
        assert!(report.rejected_variants.is_empty());
    }

    #[test]
    fn canvaskit_rejects_large_glyph_ids_and_falls_back() {
        let report = first_report(
            vec![text_op(), glyph_run(diagnostics(), u16::MAX as u32 + 1)],
            TextVariantSelectionOptions::canvaskit(),
        );
        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert_eq!(
            report.rejected_variants[0].reasons,
            vec![VariantRejectReason::GlyphIdOutOfRange]
        );
    }

    #[test]
    fn canvaskit_keeps_default_face_without_variation_as_font_proof_control() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            TextVariantSelectionOptions::canvaskit(),
            add_portable_font_resources,
        );

        assert_eq!(report.selected_variant_id.as_deref(), Some("glyphRun"));
        assert!(!report.fallback_required);
        assert!(report.rejected_variants.is_empty());
    }

    #[test]
    fn canvaskit_rejects_portable_glyph_run_without_font_face_proof() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            TextVariantSelectionOptions::canvaskit(),
            |_| {},
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert_eq!(
            report.rejected_variants[0].reasons,
            vec![VariantRejectReason::FontFaceMissing]
        );
        assert_eq!(
            VariantRejectReason::FontFaceMissing.as_str(),
            "fontFaceMissing"
        );
    }

    #[test]
    fn canvaskit_rejects_font_face_without_blob_proof() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            TextVariantSelectionOptions::canvaskit(),
            |resources| {
                resources.font_resources_mut().faces.push(FontFaceResource {
                    id: FontFaceKey("face-0".to_string()),
                    blob_key: FontBlobKey("blob-0".to_string()),
                    face_index: 0,
                    postscript_name: None,
                    family_names: Vec::new(),
                    style_names: Vec::new(),
                    weight_class: None,
                    width_class: None,
                    italic: None,
                });
            },
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert_eq!(
            report.rejected_variants[0].reasons,
            vec![VariantRejectReason::FontBlobMissing]
        );
        assert_eq!(
            VariantRejectReason::FontBlobMissing.as_str(),
            "fontBlobMissing"
        );
    }

    #[test]
    fn canvaskit_rejects_font_blob_without_resource_bytes() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            TextVariantSelectionOptions::canvaskit(),
            |resources| {
                let digest_value = resource_digest_hex([0_u8, 1, 2, 3]);
                let digest = FontDigest {
                    algorithm: crate::paint::RESOURCE_KEY_ALGORITHM.to_string(),
                    value: digest_value.clone(),
                };
                let data_ref = BinaryResourceRef {
                    kind: BinaryResourceKind::FontBlob,
                    id: font_blob_resource_key(4, &digest_value),
                };
                resources.font_resources_mut().blobs.push(FontBlobResource {
                    id: FontBlobKey("blob-0".to_string()),
                    digest: Some(digest.clone()),
                    source: FontResourceSource::Embedded,
                    data_ref: Some(data_ref.clone()),
                    portability: FontPortability::PortableBlob { digest, data_ref },
                });
                resources.font_resources_mut().faces.push(FontFaceResource {
                    id: FontFaceKey("face-0".to_string()),
                    blob_key: FontBlobKey("blob-0".to_string()),
                    face_index: 0,
                    postscript_name: None,
                    family_names: Vec::new(),
                    style_names: Vec::new(),
                    weight_class: None,
                    width_class: None,
                    italic: None,
                });
            },
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert_eq!(
            report.rejected_variants[0].reasons,
            vec![VariantRejectReason::FontBlobBytesMissing]
        );
        assert_eq!(
            VariantRejectReason::FontBlobBytesMissing.as_str(),
            "fontBlobBytesMissing"
        );
    }

    #[test]
    fn canvaskit_rejects_non_portable_font_blob_proof() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            TextVariantSelectionOptions::canvaskit(),
            |resources| {
                let digest = FontDigest {
                    algorithm: crate::paint::RESOURCE_KEY_ALGORITHM.to_string(),
                    value: resource_digest_hex([0_u8, 1, 2, 3]),
                };
                resources.font_resources_mut().blobs.push(FontBlobResource {
                    id: FontBlobKey("blob-0".to_string()),
                    digest: Some(digest.clone()),
                    source: FontResourceSource::SystemResolved,
                    data_ref: None,
                    portability: FontPortability::ResolvedButNotEmbedded {
                        digest: Some(digest),
                    },
                });
                resources.font_resources_mut().faces.push(FontFaceResource {
                    id: FontFaceKey("face-0".to_string()),
                    blob_key: FontBlobKey("blob-0".to_string()),
                    face_index: 0,
                    postscript_name: None,
                    family_names: Vec::new(),
                    style_names: Vec::new(),
                    weight_class: None,
                    width_class: None,
                    italic: None,
                });
            },
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert_eq!(
            report.rejected_variants[0].reasons,
            vec![VariantRejectReason::FontBlobNotPortable]
        );
        assert_eq!(
            VariantRejectReason::FontBlobNotPortable.as_str(),
            "fontBlobNotPortable"
        );
    }

    #[test]
    fn canvaskit_rejects_font_blob_digest_mismatch() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            TextVariantSelectionOptions::canvaskit(),
            |resources| {
                add_portable_font_resources(resources);
                resources.font_resources_mut().blobs[0].digest = Some(FontDigest {
                    algorithm: crate::paint::RESOURCE_KEY_ALGORITHM.to_string(),
                    value: resource_digest_hex([9_u8, 9, 9, 9]),
                });
            },
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert_eq!(
            report.rejected_variants[0].reasons,
            vec![VariantRejectReason::FontBlobDigestMismatch]
        );
        assert_eq!(
            VariantRejectReason::FontBlobDigestMismatch.as_str(),
            "fontBlobDigestMismatch"
        );
    }

    #[test]
    fn canvaskit_rejects_font_blob_data_ref_mismatch() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            TextVariantSelectionOptions::canvaskit(),
            |resources| {
                add_portable_font_resources(resources);
                resources.font_resources_mut().blobs[0].data_ref = None;
            },
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert_eq!(
            report.rejected_variants[0].reasons,
            vec![VariantRejectReason::FontBlobDataRefMismatch]
        );
        assert_eq!(
            VariantRejectReason::FontBlobDataRefMismatch.as_str(),
            "fontBlobDataRefMismatch"
        );
    }

    #[test]
    fn canvaskit_rejects_variation_instances_until_exact_construction_is_proven() {
        let mut op = glyph_run(diagnostics(), 42);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.shape_key.font_instance.variations = vec![VariationAxisValue {
                tag: "wght".to_string(),
                value: 700.0,
            }];
        }
        let report = first_report(
            vec![text_op(), op],
            TextVariantSelectionOptions::canvaskit(),
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::VariationUnsupported));
        assert_eq!(
            VariantRejectReason::VariationUnsupported.as_str(),
            "variationUnsupported"
        );
    }

    #[test]
    fn canvaskit_rejects_non_default_collection_face_until_exact_construction_is_proven() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            TextVariantSelectionOptions::canvaskit(),
            |resources| {
                resources.font_resources_mut().faces.push(FontFaceResource {
                    id: FontFaceKey("face-0".to_string()),
                    blob_key: FontBlobKey("blob-0".to_string()),
                    face_index: 1,
                    postscript_name: None,
                    family_names: Vec::new(),
                    style_names: Vec::new(),
                    weight_class: None,
                    width_class: None,
                    italic: None,
                });
            },
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::FaceIndexUnsupported));
        assert_eq!(
            VariantRejectReason::FaceIndexUnsupported.as_str(),
            "faceIndexUnsupported"
        );
    }

    #[test]
    fn native_skia_keeps_default_face_without_variation_as_font_proof_control() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            native_skia_options(),
            add_portable_font_resources,
        );

        assert_eq!(report.selected_variant_id.as_deref(), Some("glyphRun"));
        assert!(!report.fallback_required);
        assert!(report.rejected_variants.is_empty());
    }

    #[test]
    fn native_skia_rejects_variation_instances_until_exact_construction_is_proven() {
        let mut op = glyph_run(diagnostics(), 42);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.shape_key.font_instance.variations = vec![VariationAxisValue {
                tag: "wght".to_string(),
                value: 700.0,
            }];
        }
        let report = first_report(vec![text_op(), op], native_skia_options());

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::VariationUnsupported));
    }

    #[test]
    fn native_skia_rejects_non_default_collection_face_until_exact_construction_is_proven() {
        let report = first_report_with_resource_setup(
            vec![text_op(), glyph_run(diagnostics(), 42)],
            native_skia_options(),
            |resources| {
                resources.font_resources_mut().faces.push(FontFaceResource {
                    id: FontFaceKey("face-0".to_string()),
                    blob_key: FontBlobKey("blob-0".to_string()),
                    face_index: 1,
                    postscript_name: None,
                    family_names: Vec::new(),
                    style_names: Vec::new(),
                    weight_class: None,
                    width_class: None,
                    italic: None,
                });
            },
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::FaceIndexUnsupported));
    }

    #[test]
    fn canvaskit_rejects_unsupported_text_effects() {
        let mut op = glyph_run(diagnostics(), 42);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.paint_style.shadow_type = 1;
        }
        let report = first_report(
            vec![text_op(), op],
            TextVariantSelectionOptions::canvaskit(),
        );
        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedPaintEffect));
    }

    #[test]
    fn mixed_per_glyph_runs_keep_text_fallback_until_orientation_authority_exists() {
        let mut op = glyph_run(diagnostics(), 42);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.orientation = GlyphRunOrientation::MixedPerGlyph;
            run.glyph_transforms = Some(vec![GlyphTransform {
                xx: 1.0,
                xy: 0.0,
                yx: 0.0,
                yy: 1.0,
                tx: 0.0,
                ty: 0.0,
            }]);
        }
        let report = first_report(
            vec![text_op(), op],
            TextVariantSelectionOptions::canvaskit(),
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::MixedPerGlyphAuthorityPending));
        assert!(!report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedPaintEffect));
        assert_eq!(
            VariantRejectReason::MixedPerGlyphAuthorityPending.as_str(),
            "mixedPerGlyphAuthorityPending"
        );
    }

    #[test]
    fn glyph_transform_runs_keep_text_fallback_until_transform_authority_exists() {
        let mut op = glyph_run(diagnostics(), 42);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.glyph_transforms = Some(vec![GlyphTransform {
                xx: 1.0,
                xy: 0.0,
                yx: 0.0,
                yy: 1.0,
                tx: 0.0,
                ty: 0.0,
            }]);
        }
        let report = first_report(
            vec![text_op(), op],
            TextVariantSelectionOptions::canvaskit(),
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert_eq!(
            report.rejected_variants[0].reasons,
            vec![VariantRejectReason::GlyphTransformAuthorityPending]
        );
        assert_eq!(
            VariantRejectReason::GlyphTransformAuthorityPending.as_str(),
            "glyphTransformAuthorityPending"
        );
    }

    #[test]
    fn vertical_glyph_runs_keep_text_fallback_until_orientation_authority_exists() {
        let mut op = glyph_run(diagnostics(), 42);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.orientation = GlyphRunOrientation::VerticalUpright;
        }
        let report = first_report(
            vec![text_op(), op],
            TextVariantSelectionOptions::canvaskit(),
        );

        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::VerticalGlyphOrientationAuthorityPending));
        assert!(!report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedPaintEffect));
        assert_eq!(
            VariantRejectReason::VerticalGlyphOrientationAuthorityPending.as_str(),
            "verticalGlyphOrientationAuthorityPending"
        );
    }

    #[test]
    fn canvaskit_position_adjusted_threshold_is_explicit() {
        let mut diagnostics = diagnostics();
        diagnostics.quality = TextVariantQuality::PositionAdjusted;
        diagnostics.max_residual_after_adjustment_px = 0.2;
        let accepted = first_report(
            vec![text_op(), glyph_run(diagnostics.clone(), 42)],
            TextVariantSelectionOptions::canvaskit(),
        );
        assert_eq!(accepted.selected_variant_id.as_deref(), Some("glyphRun"));

        diagnostics.max_residual_after_adjustment_px = 0.5;
        let rejected = first_report(
            vec![text_op(), glyph_run(diagnostics, 42)],
            TextVariantSelectionOptions::canvaskit(),
        );
        assert_eq!(
            rejected.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(rejected.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::PositionAdjustedResidualTooLarge));
    }

    #[test]
    fn canvaskit_can_disallow_position_adjusted_variants() {
        let mut diagnostics = diagnostics();
        diagnostics.quality = TextVariantQuality::PositionAdjusted;
        diagnostics.max_residual_after_adjustment_px = 0.0;
        let rejected = first_report(
            vec![text_op(), glyph_run(diagnostics, 42)],
            TextVariantSelectionOptions {
                allow_position_adjusted: false,
                ..TextVariantSelectionOptions::canvaskit()
            },
        );
        assert_eq!(
            rejected.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert_eq!(
            rejected.rejected_variants[0].reasons,
            vec![VariantRejectReason::PositionAdjustedNotAllowed]
        );
    }

    #[test]
    fn strict_outline_profile_can_select_glyph_outline() {
        let report = first_report(
            vec![text_op(), glyph_run(diagnostics(), 42), glyph_outline(None)],
            TextVariantSelectionOptions::canvaskit_strict_outline(),
        );
        assert_eq!(report.selected_variant_id.as_deref(), Some("glyphOutline"));
        assert_eq!(
            report.selected_reason,
            VariantSelectedReason::GlyphOutlineStrictProfile
        );
    }

    #[test]
    fn glyph_outline_uses_position_adjusted_residual_gate() {
        let mut outline = glyph_outline(None);
        if let PaintOp::GlyphOutline { outline, .. } = &mut outline {
            outline.diagnostics.quality = TextVariantQuality::PositionAdjusted;
            outline.diagnostics.max_residual_after_adjustment_px = 0.5;
        }
        let report = first_report(
            vec![text_op(), outline],
            TextVariantSelectionOptions::canvaskit_strict_outline(),
        );
        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::PositionAdjustedResidualTooLarge));
    }

    #[test]
    fn outline_stroke_payload_requires_strict_stroke_subset() {
        let report = first_report(
            vec![
                text_op(),
                glyph_outline(Some(GlyphOutlineStrokeStyle {
                    color: 0x00000000,
                    width: 1.0,
                    join: GlyphOutlineStrokeJoin::Round,
                    cap: GlyphOutlineStrokeCap::Butt,
                    miter_limit: 2.0,
                    paint_order: GlyphOutlinePaintOrder::FillThenStroke,
                })),
            ],
            TextVariantSelectionOptions::canvaskit_strict_outline(),
        );
        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::GlyphOutlineStrokeStyleUnsupported));
    }

    #[test]
    fn outline_stroke_payload_rejects_fill_only_paint_order() {
        let report = first_report(
            vec![
                text_op(),
                glyph_outline(Some(GlyphOutlineStrokeStyle {
                    color: 0x00000000,
                    width: 1.0,
                    join: GlyphOutlineStrokeJoin::Miter,
                    cap: GlyphOutlineStrokeCap::Butt,
                    miter_limit: 2.0,
                    paint_order: GlyphOutlinePaintOrder::FillOnly,
                })),
            ],
            TextVariantSelectionOptions::canvaskit_strict_outline(),
        );
        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::GlyphOutlineStrokeStyleUnsupported));
    }

    #[test]
    fn canvas2d_keeps_text_fallback_for_variant_sidecars() {
        let report = first_report(
            vec![text_op(), glyph_run(diagnostics(), 42), glyph_outline(None)],
            TextVariantSelectionOptions {
                backend: VariantSelectionBackend::Canvas2D,
                ..TextVariantSelectionOptions::canvaskit()
            },
        );
        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.fallback_required);
        assert_eq!(report.rejected_variants.len(), 2);
        assert!(report.rejected_variants.iter().all(|reject| reject
            .reasons
            .contains(&VariantRejectReason::BackendDoesNotSupportVariant)));
    }

    #[test]
    fn incomplete_variant_parts_are_rejected_before_selection() {
        let mut op = glyph_run(diagnostics(), 42);
        if let PaintOp::GlyphRun { run, .. } = &mut op {
            run.variant.part_count = 2;
        }
        let report = first_report(
            vec![text_op(), op],
            TextVariantSelectionOptions::canvaskit(),
        );
        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::VariantPartsIncomplete));
    }

    #[test]
    fn advanced_glyph_outline_payloads_are_gated_by_default() {
        let color_report = first_report(
            vec![
                text_op(),
                color_layers_outline(ColorPaintGraphNodeKind::SolidPath),
            ],
            TextVariantSelectionOptions::canvaskit_strict_outline(),
        );
        assert_eq!(
            color_report.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(color_report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedColorGlyph));

        let bitmap_report = first_report(
            vec![text_op(), bitmap_glyph_outline()],
            TextVariantSelectionOptions::canvaskit_strict_outline(),
        );
        assert_eq!(
            bitmap_report.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(bitmap_report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedBitmapGlyph));

        let svg_report = first_report(
            vec![text_op(), svg_glyph_outline()],
            TextVariantSelectionOptions::canvaskit_strict_outline(),
        );
        assert_eq!(
            svg_report.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(svg_report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedSvgGlyph));
    }

    #[test]
    fn advanced_bitmap_and_svg_glyph_payloads_reject_missing_resources_when_allowed() {
        let bitmap_report = first_report(
            vec![text_op(), bitmap_glyph_outline()],
            TextVariantSelectionOptions {
                allow_bitmap_glyph: true,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            },
        );
        assert_eq!(
            bitmap_report.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(bitmap_report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::MissingGlyphPayloadResource));

        let svg_report = first_report(
            vec![text_op(), svg_glyph_outline()],
            TextVariantSelectionOptions {
                allow_svg_glyph: true,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            },
        );
        assert_eq!(
            svg_report.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(svg_report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::MissingGlyphPayloadResource));
    }

    #[test]
    fn advanced_bitmap_and_svg_glyph_payloads_select_only_with_resources() {
        let bitmap_report = first_report_with_resource_setup(
            vec![text_op(), bitmap_glyph_outline()],
            TextVariantSelectionOptions {
                allow_bitmap_glyph: true,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            },
            |resources| {
                assert_eq!(
                    resources.intern_image_bytes(&[1, 2, 3, 4]),
                    ImageResourceId(0)
                );
            },
        );
        assert_eq!(
            bitmap_report.selected_variant_kind,
            Some(TextVariantKind::GlyphOutline)
        );
        assert!(bitmap_report.rejected_variants.is_empty());

        let svg_report = first_report_with_resource_setup(
            vec![text_op(), svg_glyph_outline()],
            TextVariantSelectionOptions {
                allow_svg_glyph: true,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            },
            |resources| {
                assert_eq!(resources.intern_svg_fragment("<path/>"), SvgResourceId(0));
            },
        );
        assert_eq!(
            svg_report.selected_variant_kind,
            Some(TextVariantKind::GlyphOutline)
        );
        assert!(svg_report.rejected_variants.is_empty());
    }

    #[test]
    fn invalid_advanced_glyph_payloads_fallback_even_when_family_is_allowed() {
        let mut backend_default_bitmap = bitmap_glyph_outline();
        if let PaintOp::GlyphOutline { outline, .. } = &mut backend_default_bitmap {
            outline.bitmap_glyph.as_mut().unwrap().scaling_policy =
                BitmapGlyphScalingPolicy::BackendDefault;
        }
        let bitmap_report = first_report(
            vec![text_op(), backend_default_bitmap],
            TextVariantSelectionOptions {
                allow_bitmap_glyph: true,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            },
        );
        assert_eq!(
            bitmap_report.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(bitmap_report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedBitmapGlyph));

        let mut unsafe_svg = svg_glyph_outline();
        if let PaintOp::GlyphOutline { outline, .. } = &mut unsafe_svg {
            outline
                .svg_glyph
                .as_mut()
                .unwrap()
                .external_resources_allowed = true;
        }
        let svg_report = first_report(
            vec![text_op(), unsafe_svg],
            TextVariantSelectionOptions {
                allow_svg_glyph: true,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            },
        );
        assert_eq!(
            svg_report.selected_variant_kind,
            Some(TextVariantKind::TextRun)
        );
        assert!(svg_report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedSvgGlyph));
    }

    #[test]
    fn colrv1_graph_supported_gradient_gate_can_select_gradient_subset() {
        for kind in [
            ColorPaintGraphNodeKind::LinearGradientPath,
            ColorPaintGraphNodeKind::RadialGradientPath,
            ColorPaintGraphNodeKind::SweepGradientPath,
        ] {
            let report = first_report(
                vec![text_op(), color_layers_outline(kind)],
                TextVariantSelectionOptions {
                    allow_colrv1_stage1_color_graph: true,
                    ..TextVariantSelectionOptions::canvaskit_strict_outline()
                },
            );
            assert_eq!(
                report.selected_variant_kind,
                Some(TextVariantKind::GlyphOutline)
            );
            assert!(report.rejected_variants.is_empty());
        }
    }

    #[test]
    fn colrv1_unsupported_graph_node_kind_gate_is_explicit() {
        let report = first_report(
            vec![
                text_op(),
                color_layers_outline(ColorPaintGraphNodeKind::Composite),
            ],
            TextVariantSelectionOptions {
                allow_colrv1_stage1_color_graph: true,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            },
        );
        assert_eq!(report.selected_variant_kind, Some(TextVariantKind::TextRun));
        assert!(report.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedColorGlyph));
    }

    #[test]
    fn canvaskit_and_canvas2d_share_advanced_payload_reject_reason() {
        let canvas_kit = first_report(
            vec![
                text_op(),
                color_layers_outline(ColorPaintGraphNodeKind::SolidPath),
            ],
            TextVariantSelectionOptions::canvaskit_strict_outline(),
        );
        let canvas_2d = first_report(
            vec![
                text_op(),
                color_layers_outline(ColorPaintGraphNodeKind::SolidPath),
            ],
            TextVariantSelectionOptions {
                backend: VariantSelectionBackend::Canvas2D,
                ..TextVariantSelectionOptions::canvaskit_strict_outline()
            },
        );
        assert!(canvas_kit.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedColorGlyph));
        assert!(canvas_2d.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::UnsupportedColorGlyph));
        assert!(canvas_2d.rejected_variants[0]
            .reasons
            .contains(&VariantRejectReason::BackendDoesNotSupportVariant));
    }
}
