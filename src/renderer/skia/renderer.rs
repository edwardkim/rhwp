use skia_safe::{
    paint, png_encoder, surfaces, Canvas, Color, Font, FontMgr, FontStyle, Paint, PathBuilder,
    PathEffect, RRect, Rect, Typeface,
};
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap, HashSet};

use crate::error::HwpError;
use crate::model::image::ImageEffect;
use crate::model::ColorRef;
use crate::paint::replay_order::layer_node_has_replay_plane;
use crate::paint::{
    paint_op_replay_plane_with_layer, GlyphRunOrientation, GlyphRunReplayEligibility,
    LayerGlyphRunPaint, LayerNode, LayerNodeKind, LayerOutputOptions, PageLayerTree, PaintOp,
    PaintReplayPlane, ResourceArena, TextVariantQuality,
};
use crate::renderer::form_caption::display_form_caption;
use crate::renderer::layer_renderer::{
    LayerRasterRenderer, LayerRenderResult, RasterOutputFormat, RasterRenderOptions,
    RasterRenderOutput,
};
use crate::renderer::render_tree::RenderLayerInfo;
use crate::renderer::{svg_arc_to_beziers, LineStyle, PathCommand, ShapeStyle, StrokeDash};

use super::equation_conv::render_equation;
use super::font_lookup::{
    collect_system_families, legacy_typeface_for_style, match_system_family_style,
    SystemFontFamilies,
};
use super::image_conv::{draw_image_bytes, draw_svg_fragment, ImageSampling};
use super::text_replay::SkiaTextReplay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NativeGlyphRunReplayProofReason {
    EmptyGlyphIds,
    GlyphPositionCountMismatch,
    AdvanceCountMismatch,
    GlyphTransformUnsupported,
    VerticalOrientationUnsupported,
    StrictVisualIneligible,
    MissingGlyph,
    ClusterMismatch,
    UnsupportedQuality,
    PositionAdjustedResidualTooLarge,
    ReplayEligibilityNotPortable,
    UnsupportedPaintEffect,
    GlyphIdOutOfRange,
    PlacementNotFinite,
    PositionNotFinite,
    FontFaceMissing,
    FontBlobMissing,
    FontBlobNotPortable,
    FontBlobBytesMissing,
    FontBlobDataRefMismatch,
    FontBlobDigestMismatch,
    FaceIndexUnsupported,
    FontVariationUnsupported,
    TypefaceConstructionNotImplemented,
}

impl NativeGlyphRunReplayProofReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmptyGlyphIds => "emptyGlyphIds",
            Self::GlyphPositionCountMismatch => "glyphPositionCountMismatch",
            Self::AdvanceCountMismatch => "advanceCountMismatch",
            Self::GlyphTransformUnsupported => "glyphTransformUnsupported",
            Self::VerticalOrientationUnsupported => "verticalOrientationUnsupported",
            Self::StrictVisualIneligible => "strictVisualIneligible",
            Self::MissingGlyph => "missingGlyph",
            Self::ClusterMismatch => "clusterMismatch",
            Self::UnsupportedQuality => "unsupportedQuality",
            Self::PositionAdjustedResidualTooLarge => "positionAdjustedResidualTooLarge",
            Self::ReplayEligibilityNotPortable => "replayEligibilityNotPortable",
            Self::UnsupportedPaintEffect => "unsupportedPaintEffect",
            Self::GlyphIdOutOfRange => "glyphIdOutOfRange",
            Self::PlacementNotFinite => "placementNotFinite",
            Self::PositionNotFinite => "positionNotFinite",
            Self::FontFaceMissing => "fontFaceMissing",
            Self::FontBlobMissing => "fontBlobMissing",
            Self::FontBlobNotPortable => "fontBlobNotPortable",
            Self::FontBlobBytesMissing => "fontBlobBytesMissing",
            Self::FontBlobDataRefMismatch => "fontBlobDataRefMismatch",
            Self::FontBlobDigestMismatch => "fontBlobDigestMismatch",
            Self::FaceIndexUnsupported => "faceIndexUnsupported",
            Self::FontVariationUnsupported => "fontVariationUnsupported",
            Self::TypefaceConstructionNotImplemented => "typefaceConstructionNotImplemented",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeGlyphRunReplayProof {
    pub contract_replayable: bool,
    pub typeface_constructible: bool,
    pub reasons: Vec<NativeGlyphRunReplayProofReason>,
}

fn native_skia_can_replay_glyph_run(run: &LayerGlyphRunPaint, resources: &ResourceArena) -> bool {
    native_skia_glyph_run_replay_proof(run, resources).typeface_constructible
}

fn native_skia_glyph_run_contract_is_replayable(
    run: &LayerGlyphRunPaint,
    resources: &ResourceArena,
) -> bool {
    native_skia_glyph_run_replay_proof(run, resources).contract_replayable
}

pub fn native_skia_glyph_run_replay_proof(
    run: &LayerGlyphRunPaint,
    resources: &ResourceArena,
) -> NativeGlyphRunReplayProof {
    let mut contract_reasons = BTreeSet::new();
    let mut construction_reasons = BTreeSet::new();

    if run.glyph_ids.is_empty() {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::EmptyGlyphIds);
    }
    if run.glyph_ids.len() != run.positions.len() {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::GlyphPositionCountMismatch);
    }
    if run
        .advances
        .as_ref()
        .is_some_and(|advances| advances.len() != run.glyph_ids.len())
    {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::AdvanceCountMismatch);
    }
    if run.glyph_transforms.is_some() {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::GlyphTransformUnsupported);
    }
    if run.orientation != GlyphRunOrientation::Horizontal {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::VerticalOrientationUnsupported);
    }
    if !run.diagnostics.strict_visual_eligible {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::StrictVisualIneligible);
    }
    if run.diagnostics.missing_glyph_count != 0 {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::MissingGlyph);
    }
    if run.diagnostics.cluster_mismatch_count != 0 {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::ClusterMismatch);
    }
    if !matches!(
        run.diagnostics.quality,
        TextVariantQuality::Exact | TextVariantQuality::PositionAdjusted
    ) {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::UnsupportedQuality);
    }
    if run.diagnostics.quality == TextVariantQuality::PositionAdjusted {
        let tolerance = 0.5_f64.min(0.25_f64.max(run.paint_style.font_size * 0.005));
        if !run.diagnostics.max_residual_after_adjustment_px.is_finite()
            || run.diagnostics.max_residual_after_adjustment_px > tolerance
        {
            contract_reasons
                .insert(NativeGlyphRunReplayProofReason::PositionAdjustedResidualTooLarge);
        }
    }
    if run.diagnostics.replay_eligibility != GlyphRunReplayEligibility::Portable {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::ReplayEligibilityNotPortable);
    }
    if !run.paint_style.is_fill_only_glyph_replay() {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::UnsupportedPaintEffect);
    }
    if run
        .glyph_ids
        .iter()
        .any(|glyph_id| *glyph_id > u16::MAX as u32)
    {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::GlyphIdOutOfRange);
    }
    let font_resources = resources.font_resources();
    let face = font_resources
        .faces
        .iter()
        .find(|face| face.id == run.shape_key.font_instance.face_key);
    if let Some(face) = face {
        if face.face_index != 0 {
            construction_reasons.insert(NativeGlyphRunReplayProofReason::FaceIndexUnsupported);
        }
        let blob = font_resources
            .blobs
            .iter()
            .find(|blob| blob.id == face.blob_key);
        if let Some(blob) = blob {
            if !blob.portability.is_self_contained_replayable() {
                contract_reasons.insert(NativeGlyphRunReplayProofReason::FontBlobNotPortable);
            } else if let crate::paint::FontPortability::PortableBlob { data_ref, .. } =
                &blob.portability
            {
                if blob.data_ref.as_ref() != Some(data_ref) {
                    contract_reasons
                        .insert(NativeGlyphRunReplayProofReason::FontBlobDataRefMismatch);
                }
                match resources.font_blob_bytes_for_ref(data_ref) {
                    Some(bytes) if font_blob_digest_matches(bytes, blob) => {}
                    Some(_) => {
                        contract_reasons
                            .insert(NativeGlyphRunReplayProofReason::FontBlobDigestMismatch);
                    }
                    None => {
                        contract_reasons
                            .insert(NativeGlyphRunReplayProofReason::FontBlobBytesMissing);
                    }
                }
            }
        } else {
            contract_reasons.insert(NativeGlyphRunReplayProofReason::FontBlobMissing);
        }
    } else {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::FontFaceMissing);
    }
    if !run.shape_key.font_instance.variations.is_empty() {
        construction_reasons.insert(NativeGlyphRunReplayProofReason::FontVariationUnsupported);
    }
    let transform = run.placement.run_to_page;
    if ![
        transform.a,
        transform.b,
        transform.c,
        transform.d,
        transform.e,
        transform.f,
        run.placement.baseline_y,
    ]
    .into_iter()
    .all(f64::is_finite)
    {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::PlacementNotFinite);
    }
    if !run
        .positions
        .iter()
        .all(|position| position.x.is_finite() && position.y.is_finite())
    {
        contract_reasons.insert(NativeGlyphRunReplayProofReason::PositionNotFinite);
    }

    let contract_replayable = contract_reasons.is_empty();
    if contract_replayable && construction_reasons.is_empty() {
        construction_reasons
            .insert(NativeGlyphRunReplayProofReason::TypefaceConstructionNotImplemented);
    }
    let typeface_constructible = contract_replayable && construction_reasons.is_empty();
    let mut reasons = contract_reasons
        .into_iter()
        .chain(construction_reasons)
        .collect::<Vec<_>>();
    reasons.sort();

    NativeGlyphRunReplayProof {
        contract_replayable,
        typeface_constructible,
        reasons,
    }
}

fn font_blob_digest_matches(bytes: &[u8], blob: &crate::paint::FontBlobResource) -> bool {
    let actual = crate::paint::resource_digest_hex(bytes);
    let portability_digest_matches = match &blob.portability {
        crate::paint::FontPortability::PortableBlob { digest, .. } => {
            font_digest_matches_resource_bytes(digest, &actual)
        }
        _ => false,
    };
    let blob_digest_matches = blob
        .digest
        .as_ref()
        .is_none_or(|digest| font_digest_matches_resource_bytes(digest, &actual));
    portability_digest_matches && blob_digest_matches
}

fn font_digest_matches_resource_bytes(digest: &crate::paint::FontDigest, actual: &str) -> bool {
    digest.algorithm == crate::paint::RESOURCE_KEY_ALGORITHM && digest.value == actual
}

pub struct SkiaLayerRenderer {
    font_mgr: FontMgr,
    /// 사용자 지정 폰트 디렉토리에서 미리 로드한 폰트 캐시.
    /// key = primary face name (Typeface::family_name), value = Typeface.
    /// SVG 의 `--font-path` 와 같은 패턴으로 ttfs 디렉토리의 한컴 전용 폰트 (HY견명조 등) 도 사용 가능.
    custom_typefaces: HashMap<String, Typeface>,
    /// 시스템에 실제 존재하는 font family 목록.
    /// headless macOS 에서 missing family 를 CoreText 에 넘기면 downloadable font
    /// lookup IPC가 영구 대기할 수 있어, match_family_style 호출 전 사전 필터로 사용한다.
    system_families: SystemFontFamilies,
}

impl SkiaLayerRenderer {
    pub fn new() -> Self {
        // [perf] FontMgr::default() + collect_system_families() (시스템 폰트 family 전수
        // 열거) 는 페이지당 ~8ms 가 드는데, 프로세스(스레드) 내에서 불변이다. 매 렌더마다
        // 재열거하지 않도록 thread-local 로 1회만 계산하고, 이후 new() 는 캐시를 복제
        // (FontMgr = refcount bump, families = HashSet clone ~수십 µs) 해 재사용한다.
        // 폰트 매칭 입력이 동일하므로 렌더 출력은 바이트 단위로 불변이다.
        thread_local! {
            static SKIA_FONT_BASE: (FontMgr, SystemFontFamilies) = {
                let font_mgr = FontMgr::default();
                let system_families = collect_system_families(&font_mgr);
                (font_mgr, system_families)
            };
        }
        SKIA_FONT_BASE.with(|(font_mgr, system_families)| Self {
            font_mgr: font_mgr.clone(),
            custom_typefaces: HashMap::new(),
            system_families: system_families.clone(),
        })
    }

    /// 사용자 지정 폰트 디렉토리 (ttfs 등) 의 폰트를 로드하여 Skia 가 직접 사용 가능하게 한다.
    /// SVG 의 `--font-path` 와 동일한 패턴.
    pub fn with_font_paths(mut self, font_paths: &[std::path::PathBuf]) -> Self {
        let mut search_dirs: Vec<std::path::PathBuf> = font_paths.to_vec();
        for dir in &["ttfs/hwp", "ttfs/windows", "ttfs"] {
            search_dirs.push(std::path::PathBuf::from(dir));
        }
        for dir in &search_dirs {
            if !dir.exists() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let ext = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_lowercase());
                    if !matches!(ext.as_deref(), Some("ttf") | Some("otf") | Some("ttc")) {
                        continue;
                    }
                    if let Ok(data) = std::fs::read(&path) {
                        let skia_data = skia_safe::Data::new_copy(&data);
                        if let Some(typeface) = self.font_mgr.new_from_data(&skia_data, None) {
                            let family = typeface.family_name();
                            self.custom_typefaces.entry(family).or_insert(typeface);
                        }
                    }
                }
            }
        }
        self
    }

    pub fn render_raster_with_options(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRenderOutput> {
        if let Some(dpi) = options.dpi {
            if !dpi.is_finite() || dpi <= 0.0 {
                return Err(HwpError::RenderError(format!("invalid raster dpi: {dpi}")));
            }
        }
        if options.format != RasterOutputFormat::Png {
            return Err(HwpError::RenderError(
                "Skia raster renderer currently supports PNG output".to_string(),
            ));
        }

        let raster_dimension = |value: f64, label: &str| -> LayerRenderResult<i32> {
            if !value.is_finite() || value <= 0.0 {
                return Err(HwpError::RenderError(format!(
                    "invalid page {label}: {value}"
                )));
            }
            if !options.scale.is_finite() || options.scale <= 0.0 {
                return Err(HwpError::RenderError(format!(
                    "invalid raster scale: {}",
                    options.scale
                )));
            }
            if options.max_dimension <= 0 {
                return Err(HwpError::RenderError(format!(
                    "invalid raster max dimension: {}",
                    options.max_dimension
                )));
            }
            let scaled = (value * options.scale).ceil();
            if !scaled.is_finite() || scaled <= 0.0 || scaled > options.max_dimension as f64 {
                return Err(HwpError::RenderError(format!(
                    "raster {label} out of range: {scaled}"
                )));
            }
            Ok(scaled as i32)
        };
        let width = raster_dimension(tree.page_width, "width")?;
        let height = raster_dimension(tree.page_height, "height")?;
        if options.max_pixels == 0 {
            return Err(HwpError::RenderError(
                "invalid raster max pixel count: 0".to_string(),
            ));
        }
        let pixel_count = (width as u64)
            .checked_mul(height as u64)
            .ok_or_else(|| HwpError::RenderError("raster pixel count overflow".to_string()))?;
        if pixel_count > options.max_pixels {
            return Err(HwpError::RenderError(format!(
                "raster pixel count out of range: {pixel_count}"
            )));
        }

        let mut surface = surfaces::raster_n32_premul((width, height))
            .ok_or_else(|| HwpError::RenderError("Skia raster surface 생성 실패".to_string()))?;
        let canvas = surface.canvas();
        let clear_color = if let Some(color) = options.background_color {
            colorref_to_skia(color, 1.0)
        } else if options.transparent {
            Color::from_argb(0, 0, 0, 0)
        } else {
            Color::WHITE
        };
        canvas.clear(clear_color);
        if options.scale != 1.0 {
            canvas.scale((options.scale as f32, options.scale as f32));
        }

        let mut next_text_source_id = 0_u32;
        for replay_plane in PaintReplayPlane::ORDERED {
            if !layer_node_has_replay_plane(&tree.root, replay_plane) {
                continue;
            }
            self.render_node(
                canvas,
                &tree.root,
                &tree.output_options,
                &tree.resources,
                replay_plane,
                None,
                &mut next_text_source_id,
            );
        }

        let image = surface.image_snapshot();
        let mut png_options = png_encoder::Options::default();
        // PNG is lossless at every zlib level. Level 1 avoids spending most of
        // native export time on compression while preserving decoded pixels.
        png_options.z_lib_level = 1;
        let data = png_encoder::encode_image(
            None::<&mut skia_safe::gpu::DirectContext>,
            &image,
            &png_options,
        )
        .ok_or_else(|| HwpError::RenderError("Skia PNG 인코딩 실패".to_string()))?;
        Ok(RasterRenderOutput {
            bytes: data.as_bytes().to_vec(),
            format: RasterOutputFormat::Png,
            width,
            height,
            dpi: options.dpi,
            color_space: options.color_space,
        })
    }

    fn render_node(
        &self,
        canvas: &Canvas,
        node: &LayerNode,
        output_options: &LayerOutputOptions,
        resources: &ResourceArena,
        replay_plane: PaintReplayPlane,
        inherited_layer: Option<RenderLayerInfo>,
        next_text_source_id: &mut u32,
    ) {
        let active_layer = node.layer.or(inherited_layer);
        let clip_enabled = output_options.clip_enabled;
        let apply_dash = |paint: &mut Paint, dash: StrokeDash| {
            let base_width = paint.stroke_width().max(1.0);
            let intervals: Option<[f32; 6]> = match dash {
                StrokeDash::Solid => None,
                StrokeDash::Dash => Some([6.0, 3.0, 0.0, 0.0, 0.0, 0.0]),
                StrokeDash::Dot => Some([2.0, 2.0, 0.0, 0.0, 0.0, 0.0]),
                StrokeDash::DashDot => Some([6.0, 3.0, 2.0, 3.0, 0.0, 0.0]),
                StrokeDash::DashDotDot => Some([6.0, 3.0, 2.0, 3.0, 2.0, 3.0]),
            };
            if let Some(intervals) = intervals {
                let intervals = intervals
                    .into_iter()
                    .filter(|value| *value > 0.0)
                    .map(|value| value * base_width)
                    .collect::<Vec<_>>();
                if let Some(effect) = PathEffect::dash(&intervals, 0.0) {
                    paint.set_path_effect(effect);
                }
            }
        };
        let make_fill_paint = |style: &ShapeStyle| -> Option<Paint> {
            let color = style
                .pattern
                .map(|pattern| pattern.background_color)
                .or(style.fill_color)?;
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(paint::Style::Fill);
            paint.set_color(colorref_to_skia(color, style.opacity as f32));
            Some(paint)
        };
        let make_stroke_paint = |style: &ShapeStyle| -> Option<Paint> {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(paint::Style::Stroke);
            paint.set_stroke_width(if style.stroke_width > 0.0 {
                style.stroke_width as f32
            } else {
                1.0
            });
            paint.set_color(colorref_to_skia(style.stroke_color?, style.opacity as f32));
            apply_dash(&mut paint, style.stroke_dash);
            Some(paint)
        };
        let make_line_paint = |style: &LineStyle| {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_style(paint::Style::Stroke);
            paint.set_stroke_width(if style.width > 0.0 {
                style.width as f32
            } else {
                1.0
            });
            paint.set_color(colorref_to_skia(style.color, 1.0));
            apply_dash(&mut paint, style.dash);
            paint
        };
        let draw_placeholder = |bbox: crate::renderer::render_tree::BoundingBox, label: &str| {
            if bbox.width <= 0.0 || bbox.height <= 0.0 {
                return;
            }
            let rect = Rect::from_xywh(
                bbox.x as f32,
                bbox.y as f32,
                bbox.width as f32,
                bbox.height as f32,
            );
            let mut fill = Paint::default();
            fill.set_anti_alias(true);
            fill.set_style(paint::Style::Fill);
            fill.set_color(Color::from_argb(48, 96, 96, 96));
            canvas.draw_rect(rect, &fill);
            let mut stroke = Paint::default();
            stroke.set_anti_alias(true);
            stroke.set_style(paint::Style::Stroke);
            stroke.set_stroke_width(1.0);
            stroke.set_color(Color::from_argb(160, 96, 96, 96));
            canvas.draw_rect(rect, &stroke);
            let mut font = Font::default();
            font.set_size(10.0);
            let mut text = Paint::default();
            text.set_anti_alias(true);
            text.set_color(Color::from_argb(220, 64, 64, 64));
            canvas.draw_str(
                label,
                (bbox.x as f32 + 4.0, (bbox.y + bbox.height / 2.0) as f32),
                &font,
                &text,
            );
        };
        let draw_image = |data: &[u8],
                          bbox: crate::renderer::render_tree::BoundingBox,
                          fill_mode,
                          original_size,
                          crop,
                          effect| {
            draw_image_bytes(
                canvas,
                data,
                bbox.x as f32,
                bbox.y as f32,
                bbox.width as f32,
                bbox.height as f32,
                fill_mode,
                original_size,
                crop,
                effect,
                ImageSampling::linear(),
            );
        };
        let text_replay = SkiaTextReplay {
            canvas,
            font_mgr: &self.font_mgr,
            custom_typefaces: &self.custom_typefaces,
            system_families: &self.system_families,
            output_options,
        };
        let open_shape_transform =
            |transform: crate::renderer::render_tree::ShapeTransform,
             bbox: &crate::renderer::render_tree::BoundingBox| {
                canvas.save();
                let cx = (bbox.x + bbox.width / 2.0) as f32;
                let cy = (bbox.y + bbox.height / 2.0) as f32;
                if transform.horz_flip {
                    canvas.translate((cx * 2.0, 0.0));
                    canvas.scale((-1.0, 1.0));
                }
                if transform.vert_flip {
                    canvas.translate((0.0, cy * 2.0));
                    canvas.scale((1.0, -1.0));
                }
                if transform.rotation != 0.0 {
                    canvas.rotate(transform.rotation as f32, Some((cx, cy).into()));
                }
            };

        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    self.render_node(
                        canvas,
                        child,
                        output_options,
                        resources,
                        replay_plane,
                        active_layer,
                        next_text_source_id,
                    );
                }
            }
            LayerNodeKind::ClipRect { clip, child, .. } => {
                if !clip_enabled {
                    self.render_node(
                        canvas,
                        child,
                        output_options,
                        resources,
                        replay_plane,
                        active_layer,
                        next_text_source_id,
                    );
                    return;
                }
                canvas.save();
                canvas.clip_rect(
                    Rect::from_xywh(
                        clip.x as f32,
                        clip.y as f32,
                        clip.width as f32,
                        clip.height as f32,
                    ),
                    None,
                    Some(true),
                );
                self.render_node(
                    canvas,
                    child,
                    output_options,
                    resources,
                    replay_plane,
                    active_layer,
                    next_text_source_id,
                );
                canvas.restore();
            }
            LayerNodeKind::Leaf { ops } => {
                let mut variant_order = 0usize;
                let mut glyph_variants =
                    HashMap::<String, HashMap<String, (usize, u32, HashSet<u32>, bool)>>::new();
                let mut glyph_variant_sources = HashMap::<String, u32>::new();
                for op in ops {
                    if paint_op_replay_plane_with_layer(op, active_layer) != replay_plane {
                        continue;
                    }
                    if let PaintOp::GlyphRun { run, .. } = op {
                        glyph_variant_sources
                            .entry(run.variant.equivalence_group.clone())
                            .or_insert(run.source.id.0);
                        let group = glyph_variants
                            .entry(run.variant.equivalence_group.clone())
                            .or_default();
                        let state =
                            group
                                .entry(run.variant.variant_id.clone())
                                .or_insert_with(|| {
                                    let order = variant_order;
                                    variant_order = variant_order.saturating_add(1);
                                    (order, run.variant.part_count, HashSet::new(), true)
                                });
                        if state.1 != run.variant.part_count || run.variant.part_count == 0 {
                            state.3 = false;
                        }
                        if !state.2.insert(run.variant.part_index) {
                            state.3 = false;
                        }
                        state.3 &= native_skia_can_replay_glyph_run(run, resources);
                    }
                }
                let mut selected_text_variants = HashMap::new();
                for (group, variants) in glyph_variants {
                    let mut candidates = variants.into_iter().collect::<Vec<_>>();
                    candidates.sort_by_key(|(_, (order, _, _, _))| *order);
                    for (variant_id, (_, expected_part_count, parts, supported)) in candidates {
                        let parts_complete = parts.len() as u32 == expected_part_count
                            && (0..expected_part_count).all(|index| parts.contains(&index));
                        if supported && parts_complete {
                            selected_text_variants.insert(group, variant_id);
                            break;
                        }
                    }
                }
                let selected_text_sources = selected_text_variants
                    .keys()
                    .filter_map(|group| glyph_variant_sources.get(group).copied())
                    .collect::<HashSet<_>>();
                for op in ops {
                    if paint_op_replay_plane_with_layer(op, active_layer) != replay_plane {
                        continue;
                    }
                    let skip_unselected_text_variant = match op {
                        PaintOp::TextRun { .. } => {
                            let source_id = *next_text_source_id;
                            *next_text_source_id = (*next_text_source_id).saturating_add(1);
                            selected_text_sources.contains(&source_id)
                        }
                        PaintOp::GlyphRun { run, .. } => {
                            match selected_text_variants.get(&run.variant.equivalence_group) {
                                Some(selected) => selected != &run.variant.variant_id,
                                None => true,
                            }
                        }
                        PaintOp::GlyphOutline { .. } => true,
                        _ => false,
                    };
                    if skip_unselected_text_variant {
                        continue;
                    }
                    match op {
                        PaintOp::PageBackground { bbox, background } => {
                            let rect = Rect::from_xywh(
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                            );
                            if let Some(color) = background
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .or(background.background_color)
                            {
                                let mut paint = Paint::default();
                                paint.set_anti_alias(true);
                                paint.set_style(paint::Style::Fill);
                                paint.set_color(colorref_to_skia(color, 1.0));
                                canvas.draw_rect(rect, &paint);
                            }
                            if let Some(image) = &background.image {
                                // [Issue #1156] 워터마크(밝기·대비가 둘 다 0 이 아님)
                                // 인 배경 이미지만 반투명 합성한다. 밝기·대비가 0/0 인
                                // 일반 배경 이미지는 불투명 그대로 (effect 그레이스케일
                                // 등은 draw_image 가 컬러 필터로 처리).
                                // svg.rs/web_canvas.rs render_page_background_image 정합.
                                let is_watermark = image.is_watermark();
                                if is_watermark {
                                    use crate::renderer::render_tree::{
                                        LEGACY_IMAGE_WATERMARK_OPACITY,
                                        REAL_PICTURE_WATERMARK_PAGE_OPACITY,
                                    };
                                    let wm_opacity =
                                        if image.is_real_picture_watermark_tone_preset() {
                                            REAL_PICTURE_WATERMARK_PAGE_OPACITY
                                        } else {
                                            LEGACY_IMAGE_WATERMARK_OPACITY
                                        };
                                    let alpha = (255.0 * wm_opacity).round() as u32;
                                    canvas.save_layer_alpha(Some(rect), alpha);
                                }
                                draw_image(
                                    &image.data,
                                    *bbox,
                                    Some(image.fill_mode),
                                    None,
                                    None,
                                    image.effect,
                                );
                                if is_watermark {
                                    canvas.restore();
                                }
                            }
                            if let Some(color) = background.border_color {
                                let mut paint = Paint::default();
                                paint.set_anti_alias(true);
                                paint.set_style(paint::Style::Stroke);
                                paint.set_stroke_width(if background.border_width > 0.0 {
                                    background.border_width as f32
                                } else {
                                    1.0
                                });
                                paint.set_color(colorref_to_skia(color, 1.0));
                                canvas.draw_rect(rect, &paint);
                            }
                        }
                        PaintOp::TextRun { bbox, run } => {
                            let is_marker = !matches!(
                                run.field_marker,
                                crate::renderer::render_tree::FieldMarkerType::None
                            );
                            text_replay.draw_text(
                                &run.text,
                                *bbox,
                                &run.style,
                                run.baseline,
                                run.rotation,
                                run.is_vertical,
                                run.char_overlap.as_ref(),
                                is_marker,
                                run.is_para_end,
                                run.is_line_break_end,
                            );
                        }
                        PaintOp::GlyphRun { run, .. } => {
                            if !native_skia_can_replay_glyph_run(run, resources) {
                                continue;
                            }
                            // Unreachable until native_skia_can_replay_glyph_run can verify
                            // blob-backed typeface construction. Keep the TextRun fallback.
                        }
                        PaintOp::GlyphOutline { .. } => {}
                        PaintOp::FootnoteMarker { bbox, marker } => {
                            let style = crate::renderer::TextStyle {
                                font_family: marker.font_family.clone(),
                                font_size: (marker.base_font_size * 0.55).max(7.0),
                                color: marker.color,
                                ..Default::default()
                            };
                            text_replay.draw_text(
                                &marker.text,
                                *bbox,
                                &style,
                                bbox.height * 0.4,
                                0.0,
                                false,
                                None,
                                false,
                                false,
                                false,
                            );
                        }
                        PaintOp::Line { bbox, line } => {
                            if line.transform.has_transform() {
                                open_shape_transform(line.transform, bbox);
                            }
                            canvas.draw_line(
                                (line.x1 as f32, line.y1 as f32),
                                (line.x2 as f32, line.y2 as f32),
                                &make_line_paint(&line.style),
                            );
                            if line.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Rectangle { bbox, rect } => {
                            if rect.transform.has_transform() {
                                open_shape_transform(rect.transform, bbox);
                            }
                            let sk_rect = Rect::from_xywh(
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                            );
                            if let Some(fill) = rect
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .map(|color| {
                                    let mut paint = Paint::default();
                                    paint.set_anti_alias(true);
                                    paint.set_style(paint::Style::Fill);
                                    paint.set_color(colorref_to_skia(
                                        color,
                                        rect.style.opacity as f32,
                                    ));
                                    paint
                                })
                                .or_else(|| make_fill_paint(&rect.style))
                            {
                                if rect.corner_radius > 0.0 {
                                    canvas.draw_round_rect(
                                        sk_rect,
                                        rect.corner_radius as f32,
                                        rect.corner_radius as f32,
                                        &fill,
                                    );
                                } else {
                                    canvas.draw_rect(sk_rect, &fill);
                                }
                            }
                            if let Some(stroke) = make_stroke_paint(&rect.style) {
                                if rect.corner_radius > 0.0 {
                                    canvas.draw_round_rect(
                                        sk_rect,
                                        rect.corner_radius as f32,
                                        rect.corner_radius as f32,
                                        &stroke,
                                    );
                                } else {
                                    canvas.draw_rect(sk_rect, &stroke);
                                }
                            }
                            if rect.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Ellipse { bbox, ellipse } => {
                            if ellipse.transform.has_transform() {
                                open_shape_transform(ellipse.transform, bbox);
                            }
                            let oval = Rect::from_xywh(
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                            );
                            if let Some(fill) = ellipse
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .map(|color| {
                                    let mut paint = Paint::default();
                                    paint.set_anti_alias(true);
                                    paint.set_style(paint::Style::Fill);
                                    paint.set_color(colorref_to_skia(
                                        color,
                                        ellipse.style.opacity as f32,
                                    ));
                                    paint
                                })
                                .or_else(|| make_fill_paint(&ellipse.style))
                            {
                                canvas.draw_oval(oval, &fill);
                            }
                            if let Some(stroke) = make_stroke_paint(&ellipse.style) {
                                canvas.draw_oval(oval, &stroke);
                            }
                            if ellipse.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Path { bbox, path } => {
                            if path.transform.has_transform() {
                                open_shape_transform(path.transform, bbox);
                            }
                            let mut builder = PathBuilder::new();
                            let mut current = (0.0, 0.0);
                            for command in &path.commands {
                                match *command {
                                    PathCommand::MoveTo(x, y) => {
                                        builder.move_to((x as f32, y as f32));
                                        current = (x, y);
                                    }
                                    PathCommand::LineTo(x, y) => {
                                        builder.line_to((x as f32, y as f32));
                                        current = (x, y);
                                    }
                                    PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                                        builder.cubic_to(
                                            (x1 as f32, y1 as f32),
                                            (x2 as f32, y2 as f32),
                                            (x as f32, y as f32),
                                        );
                                        current = (x, y);
                                    }
                                    PathCommand::ArcTo(
                                        rx,
                                        ry,
                                        rotation,
                                        large_arc,
                                        sweep,
                                        x,
                                        y,
                                    ) => {
                                        for segment in svg_arc_to_beziers(
                                            current.0, current.1, rx, ry, rotation, large_arc,
                                            sweep, x, y,
                                        ) {
                                            if let PathCommand::CurveTo(x1, y1, x2, y2, ex, ey) =
                                                segment
                                            {
                                                builder.cubic_to(
                                                    (x1 as f32, y1 as f32),
                                                    (x2 as f32, y2 as f32),
                                                    (ex as f32, ey as f32),
                                                );
                                                current = (ex, ey);
                                            }
                                        }
                                    }
                                    PathCommand::ClosePath => {
                                        builder.close();
                                    }
                                }
                            }
                            let sk_path = builder.detach();
                            if let Some(fill) = path
                                .gradient
                                .as_ref()
                                .and_then(|gradient| gradient.colors.first().copied())
                                .map(|color| {
                                    let mut paint = Paint::default();
                                    paint.set_anti_alias(true);
                                    paint.set_style(paint::Style::Fill);
                                    paint.set_color(colorref_to_skia(
                                        color,
                                        path.style.opacity as f32,
                                    ));
                                    paint
                                })
                                .or_else(|| make_fill_paint(&path.style))
                            {
                                canvas.draw_path(&sk_path, &fill);
                            }
                            if let Some(stroke) = make_stroke_paint(&path.style) {
                                canvas.draw_path(&sk_path, &stroke);
                            }
                            if path.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Image {
                            bbox,
                            image,
                            resolved,
                        } => {
                            if image.transform.has_transform() {
                                open_shape_transform(image.transform, bbox);
                            }
                            let data = resolved
                                .as_deref()
                                .map(|payload| payload.data.as_slice())
                                .or(image.data.as_deref());
                            if let Some(data) = data {
                                let effect = if resolved
                                    .as_deref()
                                    .is_some_and(|payload| payload.suppress_effects)
                                {
                                    ImageEffect::RealPic
                                } else {
                                    image.effect
                                };
                                let opacity = image.opacity.clamp(0.0, 1.0);
                                if opacity < 1.0 {
                                    let rect = Rect::from_xywh(
                                        bbox.x as f32,
                                        bbox.y as f32,
                                        bbox.width as f32,
                                        bbox.height as f32,
                                    );
                                    let alpha = (255.0 * opacity).round() as u32;
                                    canvas.save_layer_alpha(Some(rect), alpha);
                                }
                                draw_image(
                                    data,
                                    *bbox,
                                    image.fill_mode,
                                    image.original_size,
                                    image.crop,
                                    effect,
                                );
                                if opacity < 1.0 {
                                    canvas.restore();
                                }
                            } else {
                                draw_placeholder(*bbox, "image");
                            }
                            if image.transform.has_transform() {
                                canvas.restore();
                            }
                        }
                        PaintOp::Equation { bbox, equation } => {
                            canvas.save();
                            let scale_x = if equation.layout_box.width > 0.0 && bbox.width > 0.0 {
                                bbox.width / equation.layout_box.width
                            } else {
                                1.0
                            };
                            if (scale_x - 1.0).abs() > 0.01 {
                                canvas.translate((bbox.x as f32, bbox.y as f32));
                                canvas.scale((scale_x as f32, 1.0));
                                render_equation(
                                    canvas,
                                    &self.font_mgr,
                                    &self.system_families,
                                    &equation.layout_box,
                                    0.0,
                                    0.0,
                                    equation.color,
                                    equation.font_size,
                                );
                            } else {
                                render_equation(
                                    canvas,
                                    &self.font_mgr,
                                    &self.system_families,
                                    &equation.layout_box,
                                    bbox.x,
                                    bbox.y,
                                    equation.color,
                                    equation.font_size,
                                );
                            }
                            canvas.restore();
                        }
                        PaintOp::FormObject { bbox, form } => {
                            self.draw_form_control(canvas, *bbox, form);
                        }
                        PaintOp::Placeholder { bbox, placeholder } => {
                            draw_placeholder(*bbox, placeholder.label.as_str());
                        }
                        PaintOp::RawSvg { bbox, raw } => {
                            if !draw_svg_fragment(
                                canvas,
                                raw.svg.as_str(),
                                bbox.x as f32,
                                bbox.y as f32,
                                bbox.width as f32,
                                bbox.height as f32,
                                ImageSampling::linear(),
                            ) {
                                draw_placeholder(*bbox, "svg");
                            }
                        }
                        PaintOp::CharOverlap { .. }
                        | PaintOp::TextControlMark { .. }
                        | PaintOp::TabLeader { .. }
                        | PaintOp::TextDecoration { .. } => {}
                    }
                }
            }
        }
    }
}

impl LayerRasterRenderer for SkiaLayerRenderer {
    fn render_raster(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRenderOutput> {
        self.render_raster_with_options(tree, options)
    }
}

impl SkiaLayerRenderer {
    fn make_form_font(&self, size: f32) -> Font {
        let style = FontStyle::default();
        let cjk_families = [
            "Malgun Gothic",
            "맑은 고딕",
            "NanumGothic",
            "나눔고딕",
            "AppleGothic",
        ];
        for family in &cjk_families {
            if let Some(tf) = self.custom_typefaces.get(*family).cloned() {
                return Font::new(tf, size);
            }
            if let Some(tf) =
                match_system_family_style(&self.font_mgr, &self.system_families, family, style)
            {
                return Font::new(tf, size);
            }
        }
        if let Some(tf) = legacy_typeface_for_style(&self.font_mgr, style) {
            return Font::new(tf, size);
        }
        let mut f = Font::default();
        f.set_size(size);
        f
    }

    fn draw_form_control(
        &self,
        canvas: &Canvas,
        bbox: crate::renderer::render_tree::BoundingBox,
        form: &crate::renderer::render_tree::FormObjectNode,
    ) {
        use crate::model::control::FormType;

        if bbox.width <= 0.0 || bbox.height <= 0.0 {
            return;
        }

        let x = bbox.x as f32;
        let y = bbox.y as f32;
        let w = bbox.width as f32;
        let h = bbox.height as f32;
        let rect = Rect::from_xywh(x, y, w, h);

        let bg_color = parse_css_color(&form.back_color).unwrap_or(Color::from_rgb(240, 240, 240));
        let fg_color = parse_css_color(&form.fore_color).unwrap_or(Color::from_rgb(0, 0, 0));
        let border_color = Color::from_rgb(160, 160, 160);

        match form.form_type {
            FormType::PushButton => {
                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_style(paint::Style::Fill);
                fill.set_color(bg_color);
                let rrect = RRect::new_rect_xy(rect, 3.0, 3.0);
                canvas.draw_rrect(rrect, &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(paint::Style::Stroke);
                stroke.set_stroke_width(1.0);
                stroke.set_color(border_color);
                canvas.draw_rrect(rrect, &stroke);

                let label = if form.caption.is_empty() {
                    Cow::Borrowed(form.name.as_str())
                } else {
                    display_form_caption(&form.caption)
                };
                if !label.is_empty() {
                    let font = self.make_form_font((h * 0.45).clamp(8.0, 14.0));
                    let mut tp = Paint::default();
                    tp.set_anti_alias(true);
                    tp.set_color(fg_color);
                    let text_w = font.measure_str(label.as_ref(), Some(&tp)).0;
                    let tx = x + (w - text_w) / 2.0;
                    let ty = y + h / 2.0 + font.size() * 0.35;
                    canvas.draw_str(label.as_ref(), (tx, ty), &font, &tp);
                }
            }
            FormType::CheckBox => {
                let box_size = h.min(w).min(14.0);
                let bx = x + 2.0;
                let by = y + (h - box_size) / 2.0;
                let box_rect = Rect::from_xywh(bx, by, box_size, box_size);

                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_style(paint::Style::Fill);
                fill.set_color(bg_color);
                canvas.draw_rect(box_rect, &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(paint::Style::Stroke);
                stroke.set_stroke_width(1.0);
                stroke.set_color(border_color);
                canvas.draw_rect(box_rect, &stroke);

                if form.value != 0 {
                    let mut check = Paint::default();
                    check.set_anti_alias(true);
                    check.set_style(paint::Style::Stroke);
                    check.set_stroke_width(2.0);
                    check.set_color(fg_color);
                    check.set_stroke_cap(paint::Cap::Round);
                    let cx = bx + box_size * 0.2;
                    let cy = by + box_size * 0.55;
                    let mx = bx + box_size * 0.4;
                    let my = by + box_size * 0.75;
                    let ex = bx + box_size * 0.8;
                    let ey = by + box_size * 0.25;
                    let mut builder = PathBuilder::new();
                    builder.move_to((cx, cy));
                    builder.line_to((mx, my));
                    builder.line_to((ex, ey));
                    let path = builder.detach();
                    canvas.draw_path(&path, &check);
                }

                if !form.caption.is_empty() {
                    let caption = display_form_caption(&form.caption);
                    let font = self.make_form_font((h * 0.6).clamp(8.0, 13.0));
                    let mut tp = Paint::default();
                    tp.set_anti_alias(true);
                    tp.set_color(fg_color);
                    let tx = bx + box_size + 4.0;
                    let ty = y + h / 2.0 + font.size() * 0.35;
                    canvas.draw_str(caption.as_ref(), (tx, ty), &font, &tp);
                }
            }
            FormType::RadioButton => {
                let r = h.min(w).min(14.0) / 2.0;
                let cx = x + 2.0 + r;
                let cy = y + h / 2.0;

                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_style(paint::Style::Fill);
                fill.set_color(bg_color);
                canvas.draw_circle((cx, cy), r, &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(paint::Style::Stroke);
                stroke.set_stroke_width(1.0);
                stroke.set_color(border_color);
                canvas.draw_circle((cx, cy), r, &stroke);

                if form.value != 0 {
                    let mut dot = Paint::default();
                    dot.set_anti_alias(true);
                    dot.set_style(paint::Style::Fill);
                    dot.set_color(fg_color);
                    canvas.draw_circle((cx, cy), r * 0.5, &dot);
                }

                if !form.caption.is_empty() {
                    let caption = display_form_caption(&form.caption);
                    let font = self.make_form_font((h * 0.6).clamp(8.0, 13.0));
                    let mut tp = Paint::default();
                    tp.set_anti_alias(true);
                    tp.set_color(fg_color);
                    let tx = cx + r + 4.0;
                    let ty = y + h / 2.0 + font.size() * 0.35;
                    canvas.draw_str(caption.as_ref(), (tx, ty), &font, &tp);
                }
            }
            FormType::ComboBox => {
                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_style(paint::Style::Fill);
                fill.set_color(bg_color);
                canvas.draw_rect(rect, &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(paint::Style::Stroke);
                stroke.set_stroke_width(1.0);
                stroke.set_color(border_color);
                canvas.draw_rect(rect, &stroke);

                // 드롭다운 화살표 영역
                let arrow_w = h.min(20.0);
                let ax = x + w - arrow_w;
                let arrow_rect = Rect::from_xywh(ax, y, arrow_w, h);
                let mut abg = Paint::default();
                abg.set_anti_alias(true);
                abg.set_style(paint::Style::Fill);
                abg.set_color(bg_color);
                canvas.draw_rect(arrow_rect, &abg);
                canvas.draw_line((ax, y), (ax, y + h), &stroke);

                // 화살표 삼각형
                let mut arrow = Paint::default();
                arrow.set_anti_alias(true);
                arrow.set_style(paint::Style::Fill);
                arrow.set_color(Color::from_rgb(80, 80, 80));
                let acx = ax + arrow_w / 2.0;
                let acy = y + h / 2.0;
                let as_ = (arrow_w * 0.25).min(5.0);
                let mut builder = PathBuilder::new();
                builder.move_to((acx - as_, acy - as_ * 0.5));
                builder.line_to((acx + as_, acy - as_ * 0.5));
                builder.line_to((acx, acy + as_ * 0.5));
                builder.close();
                let path = builder.detach();
                canvas.draw_path(&path, &arrow);

                if !form.text.is_empty() {
                    let font = self.make_form_font((h * 0.55).clamp(8.0, 13.0));
                    let mut tp = Paint::default();
                    tp.set_anti_alias(true);
                    tp.set_color(fg_color);
                    let tx = x + 4.0;
                    let ty = y + h / 2.0 + font.size() * 0.35;
                    canvas.draw_str(&form.text, (tx, ty), &font, &tp);
                }
            }
            FormType::Edit => {
                let mut fill = Paint::default();
                fill.set_anti_alias(true);
                fill.set_style(paint::Style::Fill);
                fill.set_color(bg_color);
                canvas.draw_rect(rect, &fill);

                let mut stroke = Paint::default();
                stroke.set_anti_alias(true);
                stroke.set_style(paint::Style::Stroke);
                stroke.set_stroke_width(1.0);
                stroke.set_color(border_color);
                canvas.draw_rect(rect, &stroke);

                if !form.text.is_empty() {
                    let font = self.make_form_font((h * 0.55).clamp(8.0, 13.0));
                    let mut tp = Paint::default();
                    tp.set_anti_alias(true);
                    tp.set_color(fg_color);
                    let tx = x + 4.0;
                    let ty = y + h / 2.0 + font.size() * 0.35;
                    canvas.draw_str(&form.text, (tx, ty), &font, &tp);
                }
            }
        }
    }
}

fn parse_css_color(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::from_rgb(r, g, b))
}

pub(super) fn colorref_to_skia(color: ColorRef, alpha_scale: f32) -> Color {
    let b = ((color >> 16) & 0xFF) as u8;
    let g = ((color >> 8) & 0xFF) as u8;
    let r = (color & 0xFF) as u8;
    let a = (255.0 * alpha_scale.clamp(0.0, 1.0)).round() as u8;
    Color::from_argb(a, r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::FormType;
    use crate::model::shape::TextWrap;
    use crate::model::style::{ImageFillMode, UnderlineType};
    use crate::paint::{
        font_blob_resource_key, resource_digest_hex, BinaryResourceKind, BinaryResourceRef,
        CacheHint, FontBlobKey, FontBlobResource, FontDigest, FontFaceKey, FontFaceResource,
        FontFallbackPolicyId, FontInstanceKey, FontPortability, FontResourceSource, GlyphCluster,
        GlyphRange, GroupKind, LayerAffineTransform, LayerNode, LayerOutputOptions, LayerPoint,
        PaintTextStyle, PaintVariantMeta, ScriptTag, ShapeKey, ShapingEngineId, TextDirection,
        TextSourceId, TextSourceRange, TextSourceSpan, TextVariantKind, WritingMode,
    };
    use crate::renderer::composer::CharOverlapInfo;
    use crate::renderer::equation::ast::EqNode;
    use crate::renderer::equation::layout::EqLayout;
    use crate::renderer::render_tree::{
        BoundingBox, EquationNode, FootnoteMarkerNode, FormObjectNode, ImageNode,
        PageBackgroundImage, PageBackgroundNode, PathNode, PlaceholderNode, RawSvgNode,
        RectangleNode, RenderLayerInfo, TextRunNode,
    };
    use crate::renderer::{GradientFillInfo, PatternFillInfo, TabLeaderInfo, TextStyle};
    use image::{ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    fn decode_rgba(bytes: &[u8]) -> image::RgbaImage {
        image::load_from_memory(bytes)
            .expect("decode png")
            .to_rgba8()
    }

    fn assert_channel(pixel: image::Rgba<u8>, channel: usize, min: u8, max: u8) {
        assert!(
            pixel[channel] >= min && pixel[channel] <= max,
            "pixel={pixel:?}, channel={channel}, expected {min}..={max}"
        );
    }

    fn count_ink(image: &image::RgbaImage) -> usize {
        image.pixels().filter(|pixel| pixel[3] > 0).count()
    }

    fn portable_font_resources() -> ResourceArena {
        let mut resources = ResourceArena::default();
        let font_bytes = [0_u8, 1, 2, 3];
        resources.intern_font_blob_bytes(&font_bytes);
        let blob_key = FontBlobKey("blob-0".to_string());
        let face_key = FontFaceKey("face-0".to_string());
        let digest_value = resource_digest_hex(font_bytes);
        let digest = FontDigest {
            algorithm: "blake3".to_string(),
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
        resources
    }

    fn portable_glyph_run(orientation: GlyphRunOrientation) -> LayerGlyphRunPaint {
        let mut variant = PaintVariantMeta::text_run_default("text-0");
        variant.variant_id = "glyphRun".to_string();
        variant.variant_kind = TextVariantKind::GlyphRun;
        variant.is_default_fallback = false;
        variant.requires = vec!["fontResources".to_string(), "text.glyphRun".to_string()];
        variant.quality = Some(TextVariantQuality::Exact);

        LayerGlyphRunPaint {
            source: TextSourceSpan {
                id: TextSourceId(0),
                utf8_range: TextSourceRange::new(0, 1),
                utf16_range: TextSourceRange::new(0, 1),
                stable_source_key: None,
            },
            variant,
            paint_style: PaintTextStyle::from(&TextStyle {
                font_family: "Test".to_string(),
                font_size: 12.0,
                ..Default::default()
            }),
            shape_key: ShapeKey {
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
            },
            placement: crate::paint::TextRunPlacement {
                run_to_page: LayerAffineTransform {
                    a: 1.0,
                    b: 0.0,
                    c: 0.0,
                    d: 1.0,
                    e: 0.0,
                    f: 0.0,
                },
                baseline_y: 0.0,
            },
            glyph_ids: vec![42],
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
            orientation,
            glyph_transforms: None,
            diagnostics: crate::paint::GlyphRunDiagnostics {
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
            },
        }
    }

    fn solid_png(color: [u8; 4]) -> Vec<u8> {
        let image = RgbaImage::from_pixel(2, 2, Rgba(color));
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("encode png");
        cursor.into_inner()
    }

    fn split_png(
        width: u32,
        height: u32,
        first: [u8; 4],
        second: [u8; 4],
        vertical: bool,
    ) -> Vec<u8> {
        let mut image = RgbaImage::from_pixel(width, height, Rgba(first));
        for y in 0..height {
            for x in 0..width {
                let second_half = if vertical {
                    y >= height / 2
                } else {
                    x >= width / 2
                };
                if second_half {
                    image.put_pixel(x, y, Rgba(second));
                }
            }
        }
        let mut cursor = Cursor::new(Vec::new());
        image
            .write_to(&mut cursor, ImageFormat::Png)
            .expect("encode png");
        cursor.into_inner()
    }

    #[test]
    fn native_skia_keeps_glyph_run_disabled_until_blob_typeface_replay_exists() {
        let resources = portable_font_resources();
        let run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(proof.contract_replayable);
        assert!(!proof.typeface_constructible);
        assert_eq!(
            proof.reasons,
            vec![NativeGlyphRunReplayProofReason::TypefaceConstructionNotImplemented]
        );
        assert_eq!(
            proof.reasons[0].as_str(),
            "typefaceConstructionNotImplemented"
        );
        assert!(native_skia_glyph_run_contract_is_replayable(
            &run, &resources
        ));
        assert!(!native_skia_can_replay_glyph_run(&run, &resources));
    }

    #[test]
    fn native_skia_rejects_vertical_glyph_run_contract_for_now() {
        let resources = portable_font_resources();
        let run = portable_glyph_run(GlyphRunOrientation::VerticalUpright);

        assert!(!native_skia_glyph_run_contract_is_replayable(
            &run, &resources
        ));
        assert!(!native_skia_can_replay_glyph_run(&run, &resources));
    }

    #[test]
    fn native_skia_glyph_run_proof_reports_missing_font_blob_bytes() {
        let mut resources = ResourceArena::default();
        let blob_key = FontBlobKey("blob-0".to_string());
        let face_key = FontFaceKey("face-0".to_string());
        let digest = FontDigest {
            algorithm: "blake3".to_string(),
            value: resource_digest_hex([0_u8, 1, 2, 3]),
        };
        let data_ref = BinaryResourceRef {
            kind: BinaryResourceKind::FontBlob,
            id: font_blob_resource_key(4, &digest.value),
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
        let run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(!proof.contract_replayable);
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FontBlobBytesMissing));
    }

    #[test]
    fn native_skia_glyph_run_proof_reports_portability_font_blob_digest_mismatch() {
        let mut resources = portable_font_resources();
        let wrong_digest = FontDigest {
            algorithm: "blake3".to_string(),
            value: resource_digest_hex([9_u8, 9, 9, 9]),
        };
        if let FontPortability::PortableBlob { digest, .. } =
            &mut resources.font_resources_mut().blobs[0].portability
        {
            *digest = wrong_digest.clone();
        }
        let run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(!proof.contract_replayable);
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FontBlobDigestMismatch));
        assert_eq!(
            NativeGlyphRunReplayProofReason::FontBlobDigestMismatch.as_str(),
            "fontBlobDigestMismatch"
        );
    }

    #[test]
    fn native_skia_glyph_run_proof_reports_blob_font_digest_mismatch() {
        let mut resources = portable_font_resources();
        resources.font_resources_mut().blobs[0].digest = Some(FontDigest {
            algorithm: "blake3".to_string(),
            value: resource_digest_hex([9_u8, 9, 9, 9]),
        });
        let run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(!proof.contract_replayable);
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FontBlobDigestMismatch));
    }

    #[test]
    fn native_skia_glyph_run_proof_rejects_unsupported_font_digest_algorithm() {
        {
            let mut resources = portable_font_resources();
            if let FontPortability::PortableBlob { digest, .. } =
                &mut resources.font_resources_mut().blobs[0].portability
            {
                digest.algorithm = "sha256".to_string();
            }
            let run = portable_glyph_run(GlyphRunOrientation::Horizontal);
            let proof = native_skia_glyph_run_replay_proof(&run, &resources);

            assert!(!proof.contract_replayable);
            assert!(proof
                .reasons
                .contains(&NativeGlyphRunReplayProofReason::FontBlobDigestMismatch));
        }

        {
            let mut resources = portable_font_resources();
            if let Some(digest) = &mut resources.font_resources_mut().blobs[0].digest {
                digest.algorithm = "sha256".to_string();
            }
            let run = portable_glyph_run(GlyphRunOrientation::Horizontal);
            let proof = native_skia_glyph_run_replay_proof(&run, &resources);

            assert!(!proof.contract_replayable);
            assert!(proof
                .reasons
                .contains(&NativeGlyphRunReplayProofReason::FontBlobDigestMismatch));
        }
    }

    #[test]
    fn native_skia_glyph_run_proof_reports_missing_blob_data_ref_metadata() {
        let mut resources = portable_font_resources();
        resources.font_resources_mut().blobs[0].data_ref = None;
        let run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(!proof.contract_replayable);
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FontBlobDataRefMismatch));
        assert_eq!(
            NativeGlyphRunReplayProofReason::FontBlobDataRefMismatch.as_str(),
            "fontBlobDataRefMismatch"
        );
    }

    #[test]
    fn native_skia_glyph_run_proof_reports_mismatched_blob_data_ref_metadata() {
        let mut resources = portable_font_resources();
        resources.font_resources_mut().blobs[0].data_ref = Some(BinaryResourceRef {
            kind: BinaryResourceKind::FontBlob,
            id: "font:blake3:4:wrong".to_string(),
        });
        let run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(!proof.contract_replayable);
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FontBlobDataRefMismatch));
    }

    #[test]
    fn native_skia_glyph_run_proof_separates_replay_eligibility_from_blob_portability() {
        let resources = portable_font_resources();
        let mut run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        run.diagnostics.replay_eligibility = GlyphRunReplayEligibility::ConditionalExternalFont;
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(!proof.contract_replayable);
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::ReplayEligibilityNotPortable));
        assert!(!proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FontBlobNotPortable));
    }

    #[test]
    fn native_skia_glyph_run_proof_reports_face_index_and_variation_limits() {
        let mut resources = portable_font_resources();
        resources.font_resources_mut().faces[0].face_index = 2;
        let mut run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        run.shape_key.font_instance.variations = vec![crate::paint::VariationAxisValue {
            tag: "wght".to_string(),
            value: 700.0,
        }];
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(proof.contract_replayable);
        assert!(!proof.typeface_constructible);
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FaceIndexUnsupported));
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FontVariationUnsupported));
    }

    #[test]
    fn native_skia_glyph_run_proof_keeps_glyph_id_range_guard() {
        let resources = portable_font_resources();
        let mut run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        run.glyph_ids[0] = u16::MAX as u32 + 1;
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(!proof.contract_replayable);
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::GlyphIdOutOfRange));
    }

    #[test]
    fn native_skia_glyph_run_proof_reports_missing_face() {
        let resources = ResourceArena::default();
        let run = portable_glyph_run(GlyphRunOrientation::Horizontal);
        let proof = native_skia_glyph_run_replay_proof(&run, &resources);

        assert!(!proof.contract_replayable);
        assert!(proof
            .reasons
            .contains(&NativeGlyphRunReplayProofReason::FontFaceMissing));
    }

    fn solid_rect_tree(
        page_width: f64,
        page_height: f64,
        bbox: BoundingBox,
        fill_color: ColorRef,
    ) -> PageLayerTree {
        let style = ShapeStyle {
            fill_color: Some(fill_color),
            ..Default::default()
        };
        PageLayerTree::new(
            page_width,
            page_height,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, page_width, page_height),
                None,
                vec![PaintOp::rectangle(
                    bbox,
                    RectangleNode::new(0.0, style, None),
                )],
            ),
        )
    }

    fn solid_rect_op(bbox: BoundingBox, fill_color: ColorRef) -> PaintOp {
        PaintOp::rectangle(
            bbox,
            RectangleNode::new(
                0.0,
                ShapeStyle {
                    fill_color: Some(fill_color),
                    ..Default::default()
                },
                None,
            ),
        )
    }

    fn solid_image_op(bbox: BoundingBox, color: [u8; 4], wrap: TextWrap) -> PaintOp {
        let mut image = ImageNode::new(1, Some(solid_png(color)));
        image.text_wrap = Some(wrap);
        PaintOp::image(bbox, image, None)
    }

    #[test]
    fn renders_png_for_basic_layer_tree() {
        let tree = solid_rect_tree(
            32.0,
            24.0,
            BoundingBox::new(4.0, 4.0, 16.0, 12.0),
            0x000000ff,
        );

        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render png");

        assert_eq!(output.format, RasterOutputFormat::Png);
        assert_eq!(output.width, 32);
        assert_eq!(output.height, 24);
        assert_eq!(&output.bytes[..8], b"\x89PNG\r\n\x1a\n");
        let decoded = image::load_from_memory(&output.bytes).expect("decode png");
        assert_eq!(decoded.width(), 32);
        assert_eq!(decoded.height(), 24);
    }

    #[test]
    fn raster_options_scale_output_size() {
        let tree = PageLayerTree::new(
            10.0,
            12.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 12.0), None, vec![]),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    scale: 2.0,
                    transparent: false,
                    ..Default::default()
                },
            )
            .expect("render scaled png");

        assert_eq!(output.width, 20);
        assert_eq!(output.height, 24);
    }

    #[test]
    fn preserves_colorref_channel_order_in_pixels() {
        let tree = solid_rect_tree(12.0, 12.0, BoundingBox::new(2.0, 2.0, 8.0, 8.0), 0x000000ff);
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render red rect");
        let image = decode_rgba(&output.bytes);
        let pixel = image.get_pixel(4, 4);

        assert!(pixel[0] > 220, "red channel should be high: {pixel:?}");
        assert!(pixel[1] < 32, "green channel should be low: {pixel:?}");
        assert!(pixel[2] < 32, "blue channel should be low: {pixel:?}");
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn clears_transparent_by_default_and_opaque_when_requested() {
        let tree = PageLayerTree::new(
            4.0,
            4.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 4.0, 4.0), None, vec![]),
        );
        let renderer = SkiaLayerRenderer::new();
        let transparent = renderer
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render transparent");
        let opaque = renderer
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    transparent: false,
                    ..Default::default()
                },
            )
            .expect("render opaque");

        assert_eq!(decode_rgba(&transparent.bytes).get_pixel(0, 0)[3], 0);
        assert_eq!(
            decode_rgba(&opaque.bytes).get_pixel(0, 0).0,
            [255, 255, 255, 255]
        );
    }

    #[test]
    fn output_options_control_clip_rect_replay() {
        let style = ShapeStyle {
            fill_color: Some(0x000000ff),
            ..Default::default()
        };
        let child = LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 20.0, 20.0),
            None,
            vec![PaintOp::rectangle(
                BoundingBox::new(0.0, 0.0, 20.0, 20.0),
                RectangleNode::new(0.0, style, None),
            )],
        );
        let clipped = PageLayerTree::new(
            20.0,
            20.0,
            LayerNode::clip_rect(
                BoundingBox::new(0.0, 0.0, 20.0, 20.0),
                None,
                BoundingBox::new(0.0, 0.0, 10.0, 10.0),
                child.clone(),
                crate::paint::ClipKind::Generic,
            ),
        );
        let unclipped = clipped.clone().with_output_options(LayerOutputOptions {
            clip_enabled: false,
            ..Default::default()
        });
        let renderer = SkiaLayerRenderer::new();
        let clipped_png = renderer
            .render_raster_with_options(&clipped, RasterRenderOptions::default())
            .expect("render clipped");
        let unclipped_png = renderer
            .render_raster_with_options(&unclipped, RasterRenderOptions::default())
            .expect("render unclipped");
        let clipped = decode_rgba(&clipped_png.bytes);
        let unclipped = decode_rgba(&unclipped_png.bytes);

        assert_eq!(clipped.get_pixel(15, 15)[3], 0);
        assert_eq!(unclipped.get_pixel(15, 15)[3], 255);
    }

    #[test]
    fn rejects_invalid_raster_options_before_surface_creation() {
        let tree = PageLayerTree::new(
            10.0,
            10.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, vec![]),
        );
        let renderer = SkiaLayerRenderer::new();

        let invalid_scale = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                scale: 0.0,
                ..Default::default()
            },
        );
        assert!(invalid_scale.is_err());

        let invalid_dpi = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                dpi: Some(0.0),
                ..Default::default()
            },
        );
        assert!(invalid_dpi.is_err());

        let oversized = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                max_dimension: 8,
                ..Default::default()
            },
        );
        assert!(oversized.is_err());

        let too_many_pixels = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                max_pixels: 99,
                ..Default::default()
            },
        );
        assert!(too_many_pixels.is_err());

        let invalid_pixel_budget = renderer.render_raster_with_options(
            &tree,
            RasterRenderOptions {
                max_pixels: 0,
                ..Default::default()
            },
        );
        assert!(invalid_pixel_budget.is_err());
    }

    #[test]
    fn raster_output_preserves_metadata_and_background_color() {
        let tree = PageLayerTree::new(
            3.0,
            2.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 3.0, 2.0), None, vec![]),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(
                &tree,
                RasterRenderOptions {
                    dpi: Some(144.0),
                    background_color: Some(0x0000ff00),
                    ..Default::default()
                },
            )
            .expect("render with metadata");
        let image = decode_rgba(&output.bytes);
        let pixel = *image.get_pixel(0, 0);

        assert_eq!(output.dpi, Some(144.0));
        assert_eq!(
            output.color_space,
            crate::renderer::layer_renderer::RasterColorSpace::Srgb
        );
        assert_channel(pixel, 0, 0, 16);
        assert_channel(pixel, 1, 220, 255);
        assert_channel(pixel, 2, 0, 16);
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn rejects_invalid_page_dimensions() {
        let renderer = SkiaLayerRenderer::new();
        let zero_width = PageLayerTree::new(
            0.0,
            10.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 0.0, 10.0), None, vec![]),
        );
        let nan_height = PageLayerTree::new(
            10.0,
            f64::NAN,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 10.0, 10.0), None, vec![]),
        );

        assert!(renderer
            .render_raster_with_options(&zero_width, RasterRenderOptions::default())
            .is_err());
        assert!(renderer
            .render_raster_with_options(&nan_height, RasterRenderOptions::default())
            .is_err());
    }

    #[test]
    fn renders_page_background_fill_border_and_image() {
        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::page_background(
                    BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                    PageBackgroundNode {
                        background_color: Some(0x0000ff00),
                        border_color: Some(0x00ff0000),
                        border_width: 2.0,
                        gradient: None,
                        image: None,
                    },
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render background");
        let image = decode_rgba(&output.bytes);
        let fill = *image.get_pixel(4, 4);
        let border = *image.get_pixel(0, 0);

        assert_channel(fill, 0, 0, 32);
        assert_channel(fill, 1, 180, 255);
        assert_channel(fill, 2, 0, 32);
        assert_eq!(fill[3], 255);
        assert_channel(border, 0, 0, 64);
        assert_channel(border, 1, 0, 64);
        assert_channel(border, 2, 180, 255);
        assert_eq!(border[3], 255);

        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::page_background(
                    BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                    PageBackgroundNode {
                        background_color: None,
                        border_color: None,
                        border_width: 0.0,
                        gradient: None,
                        image: Some(PageBackgroundImage {
                            data: solid_png([0, 0, 255, 255]),
                            fill_mode: ImageFillMode::FitToSize,
                            brightness: 0,
                            contrast: 0,
                            effect: crate::model::image::ImageEffect::RealPic,
                        }),
                    },
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render background image");
        let image = decode_rgba(&output.bytes);
        let pixel = *image.get_pixel(4, 4);

        assert_channel(pixel, 0, 0, 32);
        assert_channel(pixel, 1, 0, 32);
        assert_channel(pixel, 2, 220, 255);
        // [Issue #1156 _v2] 밝기·대비 0/0 배경 이미지는 워터마크가 아니므로
        // 불투명으로 합성한다(is_watermark() = brightness!=0 && contrast!=0).
        // PR #1163 은 _v2 이전의 "RealPic 배경=항상 워터마크 opacity" 가정으로
        // 반투명을 기대했으나, 권위 자료로 확정된 _v2 기준에 맞춰 불투명으로 정정.
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn renders_shape_fallback_fills_for_gradient_pattern_ellipse_path_and_line() {
        let gradient = GradientFillInfo {
            gradient_type: 1,
            angle: 0,
            center_x: 0,
            center_y: 0,
            colors: vec![0x00ff0000, 0x000000ff],
            positions: vec![0.0, 1.0],
        };
        let gradient_rect = RectangleNode::new(
            0.0,
            ShapeStyle {
                fill_color: Some(0x000000ff),
                ..Default::default()
            },
            Some(Box::new(gradient)),
        );
        let pattern_rect = RectangleNode::new(
            0.0,
            ShapeStyle {
                pattern: Some(PatternFillInfo {
                    pattern_type: 1,
                    pattern_color: 0x000000ff,
                    background_color: 0x0000ff00,
                }),
                ..Default::default()
            },
            None,
        );
        let ellipse = crate::renderer::render_tree::EllipseNode::new(
            ShapeStyle {
                fill_color: Some(0x000000ff),
                ..Default::default()
            },
            None,
        );
        let path = PathNode::new(
            vec![
                PathCommand::MoveTo(2.0, 24.0),
                PathCommand::LineTo(12.0, 24.0),
                PathCommand::LineTo(12.0, 34.0),
                PathCommand::LineTo(2.0, 34.0),
                PathCommand::ClosePath,
            ],
            ShapeStyle {
                fill_color: Some(0x00ff0000),
                ..Default::default()
            },
            None,
        );
        let line = crate::renderer::render_tree::LineNode::new(
            18.0,
            30.0,
            34.0,
            30.0,
            LineStyle {
                color: 0x000000ff,
                width: 3.0,
                ..Default::default()
            },
        );
        let tree = PageLayerTree::new(
            40.0,
            40.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 40.0, 40.0),
                None,
                vec![
                    PaintOp::rectangle(BoundingBox::new(2.0, 2.0, 10.0, 8.0), gradient_rect),
                    PaintOp::rectangle(BoundingBox::new(16.0, 2.0, 10.0, 8.0), pattern_rect),
                    PaintOp::ellipse(BoundingBox::new(2.0, 12.0, 10.0, 10.0), ellipse),
                    PaintOp::path(BoundingBox::new(2.0, 24.0, 10.0, 10.0), path),
                    PaintOp::line(BoundingBox::new(18.0, 28.0, 16.0, 4.0), line),
                ],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render shapes");
        let image = decode_rgba(&output.bytes);
        let gradient_pixel = *image.get_pixel(4, 4);
        let pattern_pixel = *image.get_pixel(18, 4);
        let ellipse_pixel = *image.get_pixel(7, 17);
        let path_pixel = *image.get_pixel(7, 29);
        let line_pixel = *image.get_pixel(24, 30);

        assert_channel(gradient_pixel, 2, 180, 255);
        assert_channel(pattern_pixel, 1, 180, 255);
        assert_channel(ellipse_pixel, 0, 180, 255);
        assert_channel(path_pixel, 2, 180, 255);
        assert_channel(line_pixel, 0, 180, 255);
    }

    #[test]
    fn renders_arc_path_segments_as_ink() {
        let path = PathNode::new(
            vec![
                PathCommand::MoveTo(4.0, 12.0),
                PathCommand::ArcTo(8.0, 8.0, 0.0, false, true, 20.0, 12.0),
            ],
            ShapeStyle {
                stroke_color: Some(0x000000ff),
                stroke_width: 2.0,
                ..Default::default()
            },
            None,
        );
        let tree = PageLayerTree::new(
            24.0,
            18.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 24.0, 18.0),
                None,
                vec![PaintOp::path(BoundingBox::new(4.0, 4.0, 16.0, 12.0), path)],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render arc path");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 8);
    }

    #[test]
    fn renders_valid_images_and_invalid_image_placeholders() {
        let tree = PageLayerTree::new(
            20.0,
            10.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 20.0, 10.0),
                None,
                vec![
                    PaintOp::image(
                        BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                        ImageNode::new(1, Some(solid_png([0, 0, 255, 255]))),
                        None,
                    ),
                    PaintOp::image(
                        BoundingBox::new(10.0, 0.0, 8.0, 8.0),
                        ImageNode::new(2, Some(vec![1, 2, 3, 4])),
                        None,
                    ),
                ],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render images");
        let image = decode_rgba(&output.bytes);
        let valid = *image.get_pixel(4, 4);
        let invalid_placeholder = *image.get_pixel(12, 4);

        assert_channel(valid, 2, 220, 255);
        assert!(invalid_placeholder[3] > 0);
    }

    #[test]
    fn behind_text_image_replays_below_flow_across_tree_branches() {
        let bbox = BoundingBox::new(0.0, 0.0, 12.0, 12.0);
        let flow = LayerNode::leaf(bbox, None, vec![solid_rect_op(bbox, 0x000000ff)]);
        let behind = LayerNode::leaf(
            bbox,
            None,
            vec![solid_image_op(bbox, [0, 0, 255, 255], TextWrap::BehindText)],
        );
        let tree = PageLayerTree::new(
            12.0,
            12.0,
            LayerNode::group(
                bbox,
                None,
                vec![flow, behind],
                CacheHint::None,
                GroupKind::Generic,
            ),
        );

        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render behind text order");
        let image = decode_rgba(&output.bytes);
        let center = *image.get_pixel(6, 6);

        assert_channel(center, 0, 180, 255);
        assert_channel(center, 1, 0, 64);
        assert_channel(center, 2, 0, 64);
        assert_eq!(center[3], 255);
    }

    #[test]
    fn behind_text_layered_vector_replays_below_flow_across_tree_branches() {
        let bbox = BoundingBox::new(0.0, 0.0, 12.0, 12.0);
        let flow = LayerNode::leaf(bbox, None, vec![solid_rect_op(bbox, 0x000000ff)]);
        let behind = LayerNode::leaf(bbox, None, vec![solid_rect_op(bbox, 0x00ff0000)])
            .with_layer(Some(RenderLayerInfo::new(Some(TextWrap::BehindText), 1, 1)));
        let tree = PageLayerTree::new(
            12.0,
            12.0,
            LayerNode::group(
                bbox,
                None,
                vec![flow, behind],
                CacheHint::None,
                GroupKind::Generic,
            ),
        );

        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render layered vector order");
        let image = decode_rgba(&output.bytes);
        let center = *image.get_pixel(6, 6);

        assert_channel(center, 0, 180, 255);
        assert_channel(center, 1, 0, 64);
        assert_channel(center, 2, 0, 64);
        assert_eq!(center[3], 255);
    }

    #[test]
    fn in_front_of_text_image_replays_above_flow_when_raw_order_is_earlier() {
        let bbox = BoundingBox::new(0.0, 0.0, 12.0, 12.0);
        let tree = PageLayerTree::new(
            12.0,
            12.0,
            LayerNode::leaf(
                bbox,
                None,
                vec![
                    solid_image_op(bbox, [0, 0, 255, 255], TextWrap::InFrontOfText),
                    solid_rect_op(bbox, 0x000000ff),
                ],
            ),
        );

        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render in-front text order");
        let image = decode_rgba(&output.bytes);
        let center = *image.get_pixel(6, 6);

        assert_channel(center, 0, 0, 64);
        assert_channel(center, 1, 0, 64);
        assert_channel(center, 2, 180, 255);
        assert_eq!(center[3], 255);
    }

    #[test]
    fn renders_cropped_image_source_rects() {
        let mut node = ImageNode::new(
            1,
            Some(split_png(4, 4, [255, 0, 0, 255], [0, 0, 255, 255], true)),
        );
        node.crop = Some((0, 2, 4, 4));
        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::image(
                    BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                    node,
                    None,
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render cropped image");
        let image = decode_rgba(&output.bytes);
        let pixel = *image.get_pixel(4, 4);

        assert_channel(pixel, 0, 0, 48);
        assert_channel(pixel, 2, 180, 255);
    }

    #[test]
    fn renders_tiled_images_using_original_size() {
        let mut node = ImageNode::new(
            1,
            Some(split_png(8, 4, [255, 0, 0, 255], [0, 255, 0, 255], false)),
        );
        node.fill_mode = Some(ImageFillMode::TileAll);
        node.original_size = Some((8.0, 4.0));
        let tree = PageLayerTree::new(
            16.0,
            4.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 16.0, 4.0),
                None,
                vec![PaintOp::image(
                    BoundingBox::new(0.0, 0.0, 16.0, 4.0),
                    node,
                    None,
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render tiled image");
        let image = decode_rgba(&output.bytes);
        let first_tile_left = *image.get_pixel(2, 2);
        let second_tile_left = *image.get_pixel(10, 2);
        let first_tile_right = *image.get_pixel(6, 2);
        let second_tile_right = *image.get_pixel(14, 2);

        assert_channel(first_tile_left, 0, 180, 255);
        assert_channel(second_tile_left, 0, 180, 255);
        assert_channel(first_tile_right, 1, 180, 255);
        assert_channel(second_tile_right, 1, 180, 255);
    }

    #[test]
    fn applies_grayscale_image_effect() {
        let mut node = ImageNode::new(1, Some(solid_png([255, 0, 0, 255])));
        node.effect = ImageEffect::GrayScale;
        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::image(
                    BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                    node,
                    None,
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render grayscale image");
        let pixel = *decode_rgba(&output.bytes).get_pixel(4, 4);
        let max_channel = pixel[0].max(pixel[1]).max(pixel[2]);
        let min_channel = pixel[0].min(pixel[1]).min(pixel[2]);

        assert!(max_channel.abs_diff(min_channel) <= 2, "pixel={pixel:?}");
        assert!(pixel[0] > 40 && pixel[0] < 140, "pixel={pixel:?}");
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn ignores_invalid_image_rects() {
        let tree = PageLayerTree::new(
            8.0,
            8.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 8.0, 8.0),
                None,
                vec![PaintOp::image(
                    BoundingBox::new(f64::NAN, 0.0, 8.0, 8.0),
                    ImageNode::new(1, Some(solid_png([255, 0, 0, 255]))),
                    None,
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render invalid image rect");
        let image = decode_rgba(&output.bytes);

        assert_eq!(count_ink(&image), 0);
    }

    #[test]
    fn renders_text_and_footnote_marker_as_ink() {
        let run = TextRunNode {
            text: "A".to_string(),
            style: TextStyle {
                font_size: 18.0,
                color: 0x00000000,
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
            baseline: 20.0,
            field_marker: Default::default(),
        };
        let marker = FootnoteMarkerNode {
            number: 1,
            text: "1)".to_string(),
            base_font_size: 18.0,
            font_family: String::new(),
            color: 0x00000000,
            section_index: 0,
            para_index: 0,
            control_index: 0,
        };
        let tree = PageLayerTree::new(
            64.0,
            32.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 32.0),
                None,
                vec![
                    PaintOp::text_run(BoundingBox::new(4.0, 4.0, 24.0, 24.0), run),
                    PaintOp::footnote_marker(BoundingBox::new(32.0, 4.0, 24.0, 24.0), marker),
                ],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render text");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_char_overlap_text_run_as_ink() {
        let run = TextRunNode {
            text: "①".to_string(),
            style: TextStyle {
                font_size: 20.0,
                color: 0x00000000,
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
            char_overlap: Some(CharOverlapInfo {
                border_type: 1,
                inner_char_size: 90,
            }),
            border_fill_id: 0,
            baseline: 22.0,
            field_marker: Default::default(),
        };
        let tree = PageLayerTree::new(
            40.0,
            40.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 40.0, 40.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(8.0, 8.0, 24.0, 24.0),
                    run,
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render char overlap");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_tab_leader_for_empty_text_run() {
        let run = TextRunNode {
            text: String::new(),
            style: TextStyle {
                font_size: 18.0,
                color: 0x00000000,
                tab_leaders: vec![TabLeaderInfo {
                    start_x: 8.0,
                    end_x: 72.0,
                    fill_type: 1,
                }],
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
            baseline: 22.0,
            field_marker: Default::default(),
        };
        let tree = PageLayerTree::new(
            88.0,
            36.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 88.0, 36.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(4.0, 4.0, 80.0, 28.0),
                    run,
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render tab leader");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_output_control_marks_as_ink() {
        let run = TextRunNode {
            text: " \t".to_string(),
            style: TextStyle {
                font_size: 18.0,
                color: 0x00000000,
                ..Default::default()
            },
            char_shape_id: None,
            para_shape_id: None,
            section_index: None,
            para_index: None,
            char_start: None,
            cell_context: None,
            is_para_end: true,
            is_line_break_end: false,
            rotation: 0.0,
            is_vertical: false,
            char_overlap: None,
            border_fill_id: 0,
            baseline: 22.0,
            field_marker: Default::default(),
        };
        let tree = PageLayerTree::new(
            72.0,
            36.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 72.0, 36.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(4.0, 4.0, 60.0, 28.0),
                    run,
                )],
            ),
        )
        .with_output_options(LayerOutputOptions {
            show_control_codes: true,
            ..Default::default()
        });
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render control marks");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_decorated_text_as_ink() {
        let run = TextRunNode {
            text: "A".to_string(),
            style: TextStyle {
                font_size: 18.0,
                color: 0x00000000,
                underline: UnderlineType::Bottom,
                strikethrough: true,
                emphasis_dot: 1,
                shade_color: 0x0000ffff,
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
            baseline: 24.0,
            field_marker: Default::default(),
        };
        let tree = PageLayerTree::new(
            48.0,
            40.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 48.0, 40.0),
                None,
                vec![PaintOp::text_run(
                    BoundingBox::new(8.0, 8.0, 32.0, 28.0),
                    run,
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render decorated text");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 0);
    }

    #[test]
    fn renders_equation_layout_as_colored_ink() {
        let font_size = 18.0;
        let layout_box = EqLayout::new(font_size).layout(&EqNode::Fraction {
            numer: Box::new(EqNode::Text("a".to_string())),
            denom: Box::new(EqNode::Text("b".to_string())),
        });
        let equation = EquationNode {
            svg_content: String::new(),
            layout_box,
            color_str: "#ff0000".to_string(),
            color: 0x000000ff,
            font_size,
            section_index: Some(0),
            para_index: Some(0),
            control_index: Some(0),
            cell_index: None,
            cell_para_index: None,
            note_ref: None,
        };
        let tree = PageLayerTree::new(
            64.0,
            48.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 48.0),
                None,
                vec![PaintOp::equation(
                    BoundingBox::new(6.0, 6.0, 44.0, 32.0),
                    equation,
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render equation");
        let image = decode_rgba(&output.bytes);
        let red_ink = image
            .pixels()
            .filter(|pixel| pixel[0] > 160 && pixel[1] < 96 && pixel[2] < 96 && pixel[3] > 0)
            .count();

        assert!(
            red_ink > 0,
            "equation should render using its configured color"
        );
    }

    #[test]
    fn renders_atop_equation_layout_as_colored_ink() {
        let font_size = 18.0;
        let layout_box = EqLayout::new(font_size).layout(&EqNode::Atop {
            top: Box::new(EqNode::Text("a".to_string())),
            bottom: Box::new(EqNode::Text("b".to_string())),
        });
        let equation = EquationNode {
            svg_content: String::new(),
            layout_box,
            color_str: "#00aa00".to_string(),
            color: 0x0000aa00,
            font_size,
            section_index: Some(0),
            para_index: Some(0),
            control_index: Some(0),
            cell_index: None,
            cell_para_index: None,
            note_ref: None,
        };
        let tree = PageLayerTree::new(
            64.0,
            48.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 64.0, 48.0),
                None,
                vec![PaintOp::equation(
                    BoundingBox::new(6.0, 6.0, 44.0, 32.0),
                    equation,
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render atop equation");
        let image = decode_rgba(&output.bytes);
        let green_ink = image
            .pixels()
            .filter(|pixel| pixel[0] < 96 && pixel[1] > 96 && pixel[2] < 96 && pixel[3] > 0)
            .count();

        assert!(
            green_ink > 0,
            "atop equation should render using its configured color"
        );
    }

    #[test]
    fn renders_placeholder_style_ops_as_ink() {
        let form = FormObjectNode {
            form_type: FormType::PushButton,
            caption: "OK".to_string(),
            text: String::new(),
            fore_color: "#000000".to_string(),
            back_color: "#ffffff".to_string(),
            value: 0,
            enabled: true,
            section_index: 0,
            para_index: 0,
            control_index: 0,
            name: "button".to_string(),
            cell_location: None,
        };
        let tree = PageLayerTree::new(
            48.0,
            16.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 48.0, 16.0),
                None,
                vec![
                    PaintOp::placeholder(
                        BoundingBox::new(0.0, 0.0, 14.0, 14.0),
                        PlaceholderNode::new(0, 0, "ph".to_string()),
                    ),
                    PaintOp::raw_svg(
                        BoundingBox::new(16.0, 0.0, 14.0, 14.0),
                        RawSvgNode::new("<invalid".to_string()),
                    ),
                    PaintOp::form_object(BoundingBox::new(32.0, 0.0, 14.0, 14.0), form),
                ],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render placeholders");
        let image = decode_rgba(&output.bytes);

        assert!(count_ink(&image) > 40);
    }

    #[test]
    fn renders_raw_svg_fragment_as_colored_ink() {
        let tree = PageLayerTree::new(
            32.0,
            24.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 32.0, 24.0),
                None,
                vec![PaintOp::raw_svg(
                    BoundingBox::new(4.0, 4.0, 18.0, 12.0),
                    RawSvgNode::new(
                        "<rect x=\"0\" y=\"0\" width=\"18\" height=\"12\" fill=\"#00ff00\"/>"
                            .to_string(),
                    ),
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render raw svg");
        let image = decode_rgba(&output.bytes);
        let green_ink = image
            .pixels()
            .filter(|pixel| pixel[0] < 48 && pixel[1] > 180 && pixel[2] < 48 && pixel[3] > 0)
            .count();

        assert!(
            green_ink > 100,
            "raw SVG fragment should render as green ink"
        );
    }

    #[test]
    fn raw_svg_replay_does_not_load_external_file_hrefs() {
        let external_path = std::env::temp_dir().join(format!(
            "rhwp-skia-raw-svg-external-{}.png",
            std::process::id()
        ));
        std::fs::write(&external_path, solid_png([255, 0, 0, 255])).expect("write external png");
        let external_href = external_path.to_string_lossy();
        let tree = PageLayerTree::new(
            32.0,
            24.0,
            LayerNode::leaf(
                BoundingBox::new(0.0, 0.0, 32.0, 24.0),
                None,
                vec![PaintOp::raw_svg(
                    BoundingBox::new(4.0, 4.0, 20.0, 16.0),
                    RawSvgNode::new(format!(
                        "<image href=\"{}\" x=\"0\" y=\"0\" width=\"20\" height=\"16\"/>",
                        external_href
                    )),
                )],
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render raw svg with external href");
        let _ = std::fs::remove_file(&external_path);
        let image = decode_rgba(&output.bytes);
        let red_ink = image
            .pixels()
            .filter(|pixel| pixel[0] > 180 && pixel[1] < 48 && pixel[2] < 48 && pixel[3] > 0)
            .count();

        assert_eq!(red_ink, 0, "raw SVG replay must not load file hrefs");
    }

    #[test]
    fn group_children_replay_in_order() {
        let red = LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 12.0, 12.0),
            None,
            vec![PaintOp::rectangle(
                BoundingBox::new(0.0, 0.0, 12.0, 12.0),
                RectangleNode::new(
                    0.0,
                    ShapeStyle {
                        fill_color: Some(0x000000ff),
                        ..Default::default()
                    },
                    None,
                ),
            )],
        );
        let blue = LayerNode::leaf(
            BoundingBox::new(3.0, 3.0, 6.0, 6.0),
            None,
            vec![PaintOp::rectangle(
                BoundingBox::new(3.0, 3.0, 6.0, 6.0),
                RectangleNode::new(
                    0.0,
                    ShapeStyle {
                        fill_color: Some(0x00ff0000),
                        ..Default::default()
                    },
                    None,
                ),
            )],
        );
        let tree = PageLayerTree::new(
            12.0,
            12.0,
            LayerNode::group(
                BoundingBox::new(0.0, 0.0, 12.0, 12.0),
                None,
                vec![red, blue],
                CacheHint::None,
                GroupKind::Generic,
            ),
        );
        let output = SkiaLayerRenderer::new()
            .render_raster_with_options(&tree, RasterRenderOptions::default())
            .expect("render group");
        let image = decode_rgba(&output.bytes);
        let center = *image.get_pixel(6, 6);

        assert_channel(center, 0, 0, 64);
        assert_channel(center, 1, 0, 64);
        assert_channel(center, 2, 180, 255);
        assert_eq!(center[3], 255);
    }
}
