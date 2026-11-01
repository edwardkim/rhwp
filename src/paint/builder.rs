use crate::model::style::UnderlineType;
use crate::paint::font_glyph::lower_font_native_glyph_sidecars_excluding;
use crate::paint::layer_tree::{
    CacheHint, ClipKind, GroupKind, LayerNode, LayerNodeKind, LayerOutputOptions, PageLayerTree,
};
use crate::paint::paint_op::{PaintOp, TextDecorationKind};
use crate::paint::paper_viewport::PagePaintContext;
use crate::paint::profile::RenderProfile;
use crate::paint::shaping_glyph::lower_horizontal_shaping_page_sidecars;
use crate::paint::shaping_glyph_vertical::lower_vertical_shaping_page_sidecars;
use crate::paint::EmbeddedFontFace;
use crate::renderer::render_tree::{
    BoundingBox, ControlCode, NodeId, PageRenderTree, RectangleNode, RenderNode, RenderNodeType,
};
use std::collections::HashMap;

const TEXT_MARK_BODY_CLIP_RIGHT_PAD: f64 = 48.0;

#[derive(Clone, Copy)]
struct ImagePaintFrame {
    source: BoundingBox,
    paint: BoundingBox,
}

impl ImagePaintFrame {
    fn border_frame(self, border: BoundingBox) -> BoundingBox {
        // 외곽선의 원본 여백은 선폭에서 결정된다. 그림의 인쇄 프레임을
        // 따라 이동하되, 바깥으로 확장된 테두리를 그림 크기로 덮어쓰지 않는다.
        BoundingBox::new(
            self.paint.x + (border.x - self.source.x),
            self.paint.y + (border.y - self.source.y),
            self.paint.width + (border.width - self.source.width),
            self.paint.height + (border.height - self.source.height),
        )
    }
}

/// semantic render tree를 visual layer tree로 내린다.
pub struct LayerBuilder {
    profile: RenderProfile,
    layout_dpi: f64,
    image_reference_origin: (f64, f64),
    image_page_size: (f64, f64),
    output_options: LayerOutputOptions,
    bin_data_epoch: u32,
    page_context: Option<PagePaintContext>,
    image_frames: HashMap<NodeId, ImagePaintFrame>,
    metafile_device_pixel_width: Option<f64>,
    hancom2024_print_bitmap: bool,
    metafile_fonts: super::metafile_fonts::MetafileFonts,
    prior_print_solid_fill: bool,
    hangul2024_print: bool,
    print_2024_raster: Option<bool>,
    metafile_catalogue: Option<std::sync::Arc<super::metafile_print::Catalogue>>,
    shape_group_depth: usize,
}

impl LayerBuilder {
    pub fn new(profile: RenderProfile) -> Self {
        Self {
            profile,
            layout_dpi: crate::renderer::DEFAULT_DPI,
            image_reference_origin: (0.0, 0.0),
            image_page_size: (0.0, 0.0),
            output_options: LayerOutputOptions::default(),
            bin_data_epoch: 0,
            page_context: None,
            image_frames: HashMap::new(),
            metafile_device_pixel_width: None,
            hancom2024_print_bitmap: false,
            metafile_fonts: Default::default(),
            prior_print_solid_fill: false,
            hangul2024_print: false,
            print_2024_raster: None,
            metafile_catalogue: None,
            shape_group_depth: 0,
        }
    }

    /// 인쇄 품질 계산에 사용할 실제 레이아웃 DPI다.
    pub(crate) fn with_layout_dpi(mut self, dpi: f64) -> Self {
        if dpi.is_finite() && dpi > 0.0 {
            self.layout_dpi = dpi;
        }
        self
    }

    /// 원본 형식과 분리된 출력 장치 픽셀의 레이아웃 단위 크기다.
    pub fn with_metafile_device_pixel_width(mut self, width: Option<f64>) -> Self {
        self.metafile_device_pixel_width = width.filter(|v| v.is_finite() && *v > 0.0);
        self
    }

    /// 그림 색의 출력 세대를 장치 픽셀 값과 독립적으로 전달한다.
    pub fn with_hancom2024_print_bitmap(mut self, enabled: bool) -> Self {
        self.hancom2024_print_bitmap = enabled && self.profile == RenderProfile::Print;
        self
    }

    pub fn with_output_options(mut self, output_options: LayerOutputOptions) -> Self {
        self.output_options = output_options;
        self
    }

    pub fn with_bin_data_epoch(mut self, bin_data_epoch: u32) -> Self {
        self.bin_data_epoch = bin_data_epoch;
        self
    }

    pub(crate) fn with_metafile_fonts(
        mut self,
        fonts: &super::metafile_fonts::MetafileFonts,
    ) -> Self {
        self.metafile_fonts = fonts.clone();
        self
    }

    pub(crate) fn with_prior_print_solid_fill(mut self, filled: bool) -> Self {
        self.prior_print_solid_fill = filled;
        self
    }

    pub(crate) fn with_hangul2024_print(mut self, enabled: bool) -> Self {
        self.hangul2024_print = enabled;
        self.print_2024_raster = Some(enabled);
        self
    }

    pub(crate) fn page_has_print_solid_fill(tree: &PageRenderTree, hangul2024_print: bool) -> bool {
        PagePaintContext::node_has_print_solid_fill(&tree.root, hangul2024_print)
    }

    pub fn build(&mut self, tree: &PageRenderTree) -> PageLayerTree {
        self.build_with_embedded_fonts(tree, &[])
    }

    pub(crate) fn with_metafile_catalogue(
        mut self,
        catalogue: Option<std::sync::Arc<super::metafile_print::Catalogue>>,
    ) -> Self {
        self.metafile_catalogue = catalogue;
        self
    }

    pub fn build_with_embedded_fonts(
        &mut self,
        tree: &PageRenderTree,
        fonts: &[EmbeddedFontFace<'_>],
    ) -> PageLayerTree {
        if self.profile == RenderProfile::Print
            && !fonts.is_empty()
            && (super::metafile::has_wmf(&tree.root)
                || (self.hancom2024_print_bitmap
                    && super::text_shadow::needs_font_metrics(&tree.root))
                || super::dot_leader::needs_font_program(&tree.root))
        {
            for font in fonts {
                let _ = self
                    .metafile_fonts
                    .register(font.family, font.bytes, font.face_index);
                if let Some(alias) = font.alternate_family {
                    let _ = self
                        .metafile_fonts
                        .register(alias, font.bytes, font.face_index);
                }
            }
        }
        let (page_width, page_height) = match &tree.root.node_type {
            RenderNodeType::Page(page) => (page.width, page.height),
            _ => (tree.root.bbox.width, tree.root.bbox.height),
        };

        let context = PagePaintContext::new(self.profile, page_width, page_height)
            .with_hangul2024_print(self.hangul2024_print);
        self.image_page_size = (page_width, page_height);
        fn first_column(node: &RenderNode) -> Option<(f64, f64)> {
            if matches!(node.node_type, RenderNodeType::Column(_)) {
                Some((node.bbox.x, node.bbox.y))
            } else {
                node.children.iter().find_map(first_column)
            }
        }
        self.image_reference_origin = first_column(&tree.root).unwrap_or((0.0, 0.0));
        let (page_width, page_height) = context.output_size();
        self.page_context = Some(context);
        self.image_frames.clear();

        let root = LayerNode::group(
            tree.root.bbox,
            Some(tree.root.id),
            self.build_children(&tree.root),
            self.cache_hint_for(&tree.root.node_type),
            GroupKind::Generic,
        )
        .with_layer(tree.root.layer);

        let mut layer_tree =
            PageLayerTree::with_profile(page_width, page_height, root, self.profile)
                .with_output_options(self.output_options)
                .with_bin_data_epoch(self.bin_data_epoch);
        layer_tree
            .resources
            .set_print_2024_raster(self.print_2024_raster);
        publish_line_decoration_trims(&mut layer_tree.root);
        // Vertical exact-source publication owns its leaf slots first. The
        // horizontal and nominal lowerers remain independent and only share
        // the final claim set used to suppress duplicate nominal GlyphRuns.
        let mut claimed_glyph_run_sources = lower_vertical_shaping_page_sidecars(
            &mut layer_tree.root,
            &tree.frame,
            &mut layer_tree.resources,
        );
        claimed_glyph_run_sources.extend(lower_horizontal_shaping_page_sidecars(
            &mut layer_tree.root,
            tree.frame.horizontal_shaping_sidecars(),
            &mut layer_tree.resources,
        ));
        lower_font_native_glyph_sidecars_excluding(
            &mut layer_tree.root,
            &mut layer_tree.resources,
            fonts,
            &claimed_glyph_run_sources,
        );
        context.publish_table_border_geometry(&mut layer_tree.root);
        context.publish_decoration_geometry(&mut layer_tree.root);
        context.publish_glyph_geometry(
            &mut layer_tree.root,
            self.prior_print_solid_fill,
            self.hangul2024_print,
        );
        layer_tree
    }

    fn build_children(&mut self, node: &RenderNode) -> Vec<LayerNode> {
        let previous_origin = self.image_reference_origin;
        if matches!(node.node_type, RenderNodeType::Column(_)) {
            self.image_reference_origin = (node.bbox.x, node.bbox.y);
        }
        let previous_context = self.page_context;
        let previous_group_depth = self.shape_group_depth;
        if matches!(&node.node_type, RenderNodeType::Group(_)) {
            self.shape_group_depth += 1;
        }
        self.page_context = self.page_context.map(|context| context.for_children(node));
        if matches!(&node.node_type, RenderNodeType::Group(group) if group.paint_transform.is_some())
        {
            self.page_context = self
                .page_context
                .map(PagePaintContext::without_rectangle_mapping);
        }
        let mut children = crate::renderer::paint_order::children(node)
            .filter_map(|child| self.build_node(child))
            .collect::<Vec<_>>();
        self.page_context = previous_context;
        self.shape_group_depth = previous_group_depth;
        if self.output_options.show_control_codes {
            let label = match &node.node_type {
                // 그림·수식은 전용 paint 하위화 경로에서 조판부호를 생성한다.
                RenderNodeType::Image(_) | RenderNodeType::Equation(_) => None,
                _ => node.control_code_label(),
            };
            if let Some(label) = label {
                children.push(
                    LayerNode::leaf(
                        node.bbox,
                        Some(node.id),
                        vec![PaintOp::control_label(node.bbox, label)],
                    )
                    .with_layer(node.layer),
                );
            }
        }
        self.image_reference_origin = previous_origin;
        children
    }

    fn build_node(&mut self, node: &RenderNode) -> Option<LayerNode> {
        if let Some(paths) = &node.paint_clip {
            let mut unclipped = node.clone();
            unclipped.paint_clip = None;
            let child = self.build_node(&unclipped)?;
            return Some(LayerNode {
                bounds: node.bbox,
                source_node_id: Some(node.id),
                layer: node.layer,
                kind: LayerNodeKind::ClipPath {
                    paths: paths.clone(),
                    child: Box::new(child),
                },
            });
        }
        if !self.profile.shows_editor_visuals() {
            if let Some(bbox) = node.print_bbox {
                let mut print_node = node.clone();
                print_node.bbox = bbox;
                print_node.print_bbox = None;
                return self.build_node(&print_node);
            }
        }
        if !node.visible || (node.editor_only && !self.profile.shows_editor_visuals()) {
            return None;
        }
        if matches!(
            &node.node_type,
            RenderNodeType::Placeholder(placeholder)
                if placeholder.kind
                    == crate::renderer::render_tree::PlaceholderKind::MissingPicture
                    && !self.profile.shows_editor_visuals()
        ) {
            return None;
        }

        if let Some(parts) = super::parallel_shadow::split(node) {
            let shadow = self.build_node(&parts.silhouette)?;
            let body = self.build_node(&parts.body)?;
            let mut group = LayerNode::group(
                node.bbox,
                Some(node.id),
                vec![
                    super::parallel_shadow::translated(shadow, parts.offset),
                    body,
                ],
                CacheHint::None,
                GroupKind::Generic,
            )
            .with_layer(node.layer);
            group.bounds = group.visual_bounds(true);
            return Some(group);
        }

        // A rectangle transform owns its text/clip descendants too. Primitive
        // paint transforms affect only the outline, so move it to one common
        // subtree group and clear the primitive transform to avoid double use.
        if let RenderNodeType::Rectangle(rect) = &node.node_type {
            if rect.transform.has_transform() && !node.children.is_empty() {
                let matrix = rect.transform.matrix_about(node.bbox);
                let mut local = node.clone();
                if let RenderNodeType::Rectangle(rect) = &mut local.node_type {
                    rect.transform = Default::default();
                }
                let previous_context = self.page_context;
                self.page_context = self
                    .page_context
                    .map(PagePaintContext::without_rectangle_mapping);
                let child = self.build_node(&local);
                self.page_context = previous_context;
                let child = child?;
                let mut group = LayerNode::group(
                    node.bbox,
                    Some(node.id),
                    vec![child],
                    CacheHint::None,
                    GroupKind::Group(crate::renderer::render_tree::GroupNode {
                        paint_transform: Some(matrix),
                        section_index: rect.section_index,
                        para_index: rect.para_index,
                        control_index: rect.control_index,
                    }),
                )
                .with_layer(node.layer);
                group.bounds = group.visual_bounds(true);
                return Some(group);
            }
        }

        let mut image_fill_clip = None;
        let own_ops = match &node.node_type {
            RenderNodeType::PageBackground(background) => Some(vec![PaintOp::page_background(
                node.bbox,
                background.clone(),
            )]),
            RenderNodeType::TextRun(run) => {
                let mut run = run.clone();
                if run
                    .char_overlap
                    .as_ref()
                    .is_some_and(|overlap| overlap.empty_ink)
                {
                    // 조판에 사용한 원문과 프레임은 유지하고, paint 복제본의 잉크만 비운다.
                    run.display_text = Some(String::new());
                }
                let mut leader_paths = Vec::new();
                if self.profile == RenderProfile::Print && self.hangul2024_print && !run.is_vertical
                {
                    run.style.continuous_shadow = super::text_shadow::prepare(
                        &self.metafile_fonts,
                        run.display_or_text(),
                        &run.style,
                        self.layout_dpi,
                    );
                    if let Some(plan) = &run.style.continuous_shadow {
                        [run.style.shadow_offset_x, run.style.shadow_offset_y] = plan.extent();
                    }
                    leader_paths =
                        super::dot_leader::prepare(&self.metafile_fonts, node.bbox, &mut run);
                }
                let mut ops = text_run_ops(node.bbox, run, self.output_options);
                ops.extend(
                    leader_paths
                        .into_iter()
                        .map(|path| PaintOp::path(node.bbox, path)),
                );
                Some(ops)
            }
            RenderNodeType::FootnoteMarker(marker) => {
                Some(vec![PaintOp::footnote_marker(node.bbox, marker.clone())])
            }
            RenderNodeType::Line(line) => Some(
                line.paint_strokes()
                    .map(|stroke| PaintOp::line(node.bbox, stroke))
                    .collect(),
            ),
            RenderNodeType::Rectangle(rect) => {
                let mut rect = rect.clone();
                let pixel = self
                    .metafile_device_pixel_width
                    .filter(|_| self.hancom2024_print_bitmap);
                if let Some(context) = self.page_context {
                    context.picture_border_pen(&mut rect, pixel);
                }
                let image_frame = rect
                    .image_frame_owner
                    .and_then(|owner| self.image_frames.get(&owner))
                    .map(|frame| {
                        let source = if rect.picture_border_width_hu == Some(0) {
                            frame.source
                        } else {
                            node.bbox
                        };
                        self.page_context
                            .and_then(|context| context.picture_border_frame(source, &rect))
                            .unwrap_or_else(|| frame.border_frame(node.bbox))
                    });
                let frame = image_frame.unwrap_or_else(|| {
                    self.page_context.map_or(node.bbox, |context| {
                        context.rectangle_frame(node, &mut rect, pixel)
                    })
                });
                let fill_frame = self
                    .page_context
                    .and_then(|context| context.rectangle_fill_frame(frame, &rect, pixel));
                // 이미 확정한 그림 테두리 frame은 다시 투영하지 않는다.
                let map_fill = |bounds, rect: &RectangleNode| {
                    if image_frame.is_some() {
                        bounds
                    } else {
                        self.page_context.map_or(bounds, |context| {
                            context.solid_fill_frame(bounds, rect, pixel, node)
                        })
                    }
                };
                if let Some(fill_frame) = fill_frame {
                    let fill = super::rectangle_fill::take_fill(&mut rect);
                    Some(vec![
                        PaintOp::rectangle(map_fill(fill_frame, &fill), fill),
                        PaintOp::rectangle(frame, rect),
                    ])
                } else {
                    Some(vec![PaintOp::rectangle(map_fill(frame, &rect), rect)])
                }
            }
            RenderNodeType::Ellipse(ellipse) => {
                Some(vec![PaintOp::ellipse(node.bbox, ellipse.clone())])
            }
            RenderNodeType::Path(path) => Some(vec![PaintOp::path(node.bbox, path.clone())]),
            RenderNodeType::Image(image) => {
                let pixel = self
                    .metafile_device_pixel_width
                    .filter(|_| self.hancom2024_print_bitmap);
                let fill = self
                    .page_context
                    .and_then(|context| context.bitmap_fill_frame(node, image, pixel));
                let centered_fill_frame = fill.is_some()
                    && image.fill_mode == Some(crate::model::style::ImageFillMode::Center);
                let mut frame = if let Some(fill) = fill {
                    image_fill_clip = Some(fill.clip);
                    fill.image
                } else {
                    self.page_context
                        .map_or(node.bbox, |context| context.image_frame(node, image, pixel))
                };
                if image_fill_clip.is_none()
                    && !node.layer.as_ref().is_some_and(|layer| layer.master_page)
                    && (self.hancom2024_print_bitmap
                        || frame.x != node.bbox.x
                        || frame.y != node.bbox.y
                        || frame.width != node.bbox.width
                        || frame.height != node.bbox.height)
                {
                    self.image_frames.insert(
                        node.id,
                        ImagePaintFrame {
                            source: node.bbox,
                            paint: frame,
                        },
                    );
                }
                // PDF samples already carry display-space colour and continuous alpha.
                // Ordinary HWP pictures retain the Hancom Print contract.
                let pdf_samples = image
                    .data
                    .as_deref()
                    .is_some_and(crate::renderer::image_resolver::png_sampling::pdf_source);
                let mut ops = if let Some(path) = crate::renderer::pdf_stroke::project(image, frame)
                {
                    vec![PaintOp::path(frame, path)]
                } else {
                    let mut image = image.clone();
                    image.hancom2024_print_bitmap = self.hancom2024_print_bitmap && !pdf_samples;
                    if centered_fill_frame {
                        // 원본의 가운데 채움 속성은 유지하고, 공통 프레임으로
                        // 투영한 paint 연산만 그 프레임 전체에 그림을 재생한다.
                        image.fill_mode = Some(crate::model::style::ImageFillMode::FitToSize);
                    }
                    let group_projection = (self.shape_group_depth > 0
                        && image_fill_clip.is_none())
                    .then(|| super::group_picture::project(&image, frame, self.layout_dpi))
                    .flatten();
                    if let Some((full, image, payload)) = group_projection {
                        image_fill_clip = Some(frame);
                        vec![PaintOp::image(full, image, Some(payload))]
                    } else {
                        crate::renderer::image_resolver::publish_print_jpeg_crop(&mut image);
                        let mut resolved = (self.profile == RenderProfile::Print)
                            .then(|| {
                                crate::renderer::image_resolver::print_jpeg_payload(
                                    &image,
                                    frame,
                                    self.layout_dpi,
                                )
                            })
                            .flatten()
                            .or_else(|| {
                                if self.profile != RenderProfile::Print
                                    || self.hancom2024_print_bitmap
                                {
                                    return None;
                                }
                                super::metafile_print::payload(
                                    &image,
                                    node.bbox,
                                    self.image_page_size,
                                    self.image_reference_origin,
                                    self.layout_dpi,
                                    &self.metafile_fonts,
                                    self.metafile_catalogue.as_deref()?,
                                )
                            })
                            .or_else(|| {
                                pixel
                                    .and_then(|_| {
                                        self.page_context?.image_codec_width_points(&image, frame)
                                    })
                                    .and_then(|width| {
                                        crate::renderer::image_resolver::print_device_image_payload(
                                            &image,
                                            width,
                                            (
                                                node.bbox.width * 72.0 / self.layout_dpi,
                                                node.bbox.height * 72.0 / self.layout_dpi,
                                            ),
                                            self.shape_group_depth > 0,
                                        )
                                    })
                            })
                            .or_else(|| {
                                pixel.and_then(|pixel| {
                                    super::metafile_surface::picture_payload(
                                        &image,
                                        frame,
                                        pixel,
                                        &self.metafile_fonts,
                                    )
                                })
                            })
                            .or_else(|| {
                                crate::paint::metafile::image_payload(
                                    &image,
                                    node.bbox,
                                    self.metafile_device_pixel_width,
                                    &self.metafile_fonts,
                                )
                            });
                        if let Some(window) =
                            crate::renderer::image_resolver::resolve_print_jpeg_window(
                                &mut image,
                                resolved.as_ref(),
                            )
                        {
                            resolved = Some(window);
                        }
                        if let Some(payload) = resolved.as_mut().filter(|p| {
                            crate::renderer::image_resolver::png_sampling::print_jpeg_transfer(
                                &p.data,
                            ) && !image.transform.has_transform()
                        }) {
                            if let Some(frame) =
                                crate::renderer::print_raster::PrintRasterFrame::new(
                                    self.image_page_size,
                                    self.image_reference_origin,
                                    node.bbox,
                                    self.layout_dpi,
                                )
                            {
                                frame.attach(&mut payload.data);
                            }
                        }
                        if self.profile == RenderProfile::Print {
                            if let Some((alpha_frame, payload)) =
                                crate::renderer::print_raster::PrintRasterFrame::new(
                                    self.image_page_size,
                                    self.image_reference_origin,
                                    node.bbox,
                                    self.layout_dpi,
                                )
                                .and_then(|source| {
                                    crate::renderer::image_resolver::print_alpha_payload(
                                        &image,
                                        source,
                                        self.layout_dpi,
                                    )
                                })
                            {
                                frame = alpha_frame;
                                resolved = Some(payload);
                            }
                        }
                        vec![PaintOp::image(frame, image, resolved)]
                    }
                };
                if self.output_options.show_control_codes {
                    ops.push(PaintOp::control_label(frame, "[그림]"));
                }
                Some(ops)
            }
            RenderNodeType::Equation(equation) => {
                let mut ops = vec![PaintOp::equation(node.bbox, equation.clone())];
                if self.output_options.show_control_codes {
                    ops.push(PaintOp::control_label(node.bbox, "[수식]"));
                }
                Some(ops)
            }
            RenderNodeType::FormObject(form) => {
                Some(vec![PaintOp::form_object(node.bbox, form.clone())])
            }
            RenderNodeType::Placeholder(placeholder) => {
                Some(vec![PaintOp::placeholder(node.bbox, placeholder.clone())])
            }
            RenderNodeType::RawSvg(raw) => Some(vec![PaintOp::raw_svg(
                node.bbox,
                crate::paint::metafile::raw_svg(raw, self.metafile_device_pixel_width),
            )]),
            _ => None,
        };

        if let Some(ops) = own_ops {
            let own_bounds = ops.first().map_or(node.bbox, PaintOp::bounds);
            let mut own_leaf =
                LayerNode::leaf(own_bounds, Some(node.id), ops).with_layer(node.layer);
            own_leaf = super::metafile_picture::lower(own_leaf);
            if let Some(clip) = image_fill_clip {
                own_leaf = LayerNode::clip_rect(
                    clip,
                    Some(node.id),
                    clip,
                    own_leaf,
                    ClipKind::PrintBitmapFill,
                )
                .with_layer(node.layer);
            }
            // 내부 문단이 없는 사각형도 원본 컨트롤 부호를 출력해야 한다.
            let has_source_marker = self.output_options.show_control_codes
                && matches!(node.control_code, ControlCode::Rectangle(_));
            return if node.children.is_empty() && !has_source_marker {
                Some(own_leaf)
            } else {
                let mut children = Vec::with_capacity(node.children.len() + 1);
                children.push(own_leaf);
                children.extend(self.build_children(node));
                Some(
                    LayerNode::group(
                        node.bbox,
                        Some(node.id),
                        children,
                        self.cache_hint_for(&node.node_type),
                        self.group_kind_for(&node.node_type),
                    )
                    .with_layer(node.layer),
                )
            };
        }

        match &node.node_type {
            RenderNodeType::Body {
                clip_rect: Some(clip),
            } => {
                let effective_clip = if self.output_options.show_paragraph_marks
                    || self.output_options.show_control_codes
                {
                    crate::renderer::render_tree::BoundingBox::new(
                        clip.x,
                        clip.y,
                        clip.width + TEXT_MARK_BODY_CLIP_RIGHT_PAD,
                        clip.height,
                    )
                } else {
                    *clip
                };
                let child = LayerNode::group(
                    node.bbox,
                    Some(node.id),
                    self.build_children(node),
                    self.cache_hint_for(&node.node_type),
                    GroupKind::Body,
                );
                let effective_clip =
                    super::text_shadow::flow_bounds(&child).map_or(effective_clip, |shadow| {
                        let x = effective_clip.x.min(shadow.x);
                        let y = effective_clip.y.min(shadow.y);
                        BoundingBox::new(
                            x,
                            y,
                            (effective_clip.x + effective_clip.width).max(shadow.x + shadow.width)
                                - x,
                            (effective_clip.y + effective_clip.height)
                                .max(shadow.y + shadow.height)
                                - y,
                        )
                    });
                Some(
                    LayerNode::clip_rect(
                        node.bbox,
                        Some(node.id),
                        effective_clip,
                        child,
                        ClipKind::Body,
                    )
                    .with_layer(node.layer),
                )
            }
            RenderNodeType::TableCell(cell) if cell.clip => {
                let children = self.build_children(node);
                Some(super::cell_clip::build(
                    node,
                    cell,
                    children,
                    self.cache_hint_for(&node.node_type),
                ))
            }
            RenderNodeType::TextBox => {
                let clip = node
                    .print_clip_bbox
                    .filter(|_| !self.profile.shows_editor_visuals())
                    .unwrap_or(node.bbox);
                let child = LayerNode::group(
                    node.bbox,
                    Some(node.id),
                    self.build_children(node),
                    self.cache_hint_for(&node.node_type),
                    GroupKind::TextBox,
                );
                Some(
                    LayerNode::clip_rect(node.bbox, Some(node.id), clip, child, ClipKind::TextBox)
                        .with_layer(node.layer),
                )
            }
            _ => Some(
                LayerNode::group(
                    node.bbox,
                    Some(node.id),
                    self.build_children(node),
                    self.cache_hint_for(&node.node_type),
                    self.group_kind_for(&node.node_type),
                )
                .with_layer(node.layer),
            ),
        }
    }

    fn cache_hint_for(&self, node_type: &RenderNodeType) -> CacheHint {
        match node_type {
            RenderNodeType::Header | RenderNodeType::Footer | RenderNodeType::MasterPage => {
                CacheHint::StaticSubtree
            }
            RenderNodeType::PageBackground(_)
                if matches!(self.profile, RenderProfile::FastPreview) =>
            {
                CacheHint::PreferRaster
            }
            _ => CacheHint::None,
        }
    }

    fn group_kind_for(&self, node_type: &RenderNodeType) -> GroupKind {
        match node_type {
            RenderNodeType::MasterPage => GroupKind::MasterPage,
            RenderNodeType::Header => GroupKind::Header,
            RenderNodeType::Footer => GroupKind::Footer,
            RenderNodeType::Body { .. } => GroupKind::Body,
            RenderNodeType::Column(index) => GroupKind::Column(*index),
            RenderNodeType::FootnoteArea => GroupKind::FootnoteArea,
            RenderNodeType::TextLine(line) => GroupKind::TextLine(line.clone()),
            RenderNodeType::Table(table) => GroupKind::Table(table.clone()),
            RenderNodeType::TableCell(cell) => GroupKind::TableCell(cell.clone()),
            RenderNodeType::TextBox => GroupKind::TextBox,
            RenderNodeType::Group(group) => GroupKind::Group(group.clone()),
            _ => GroupKind::Generic,
        }
    }
}

fn line_decoration_trim_target(
    children: &[LayerNode],
) -> Option<(crate::paint::TextSourceSpan, usize)> {
    fn scan(node: &LayerNode, found: &mut Option<Option<(crate::paint::TextSourceSpan, usize)>>) {
        if found.is_some() {
            return;
        }
        match &node.kind {
            LayerNodeKind::Leaf { ops } => {
                for op in ops.iter().rev() {
                    if let PaintOp::TextRun {
                        run,
                        source: Some(source),
                        ..
                    } = op
                    {
                        if run.text.chars().any(|ch| ch != ' ') {
                            if run.is_para_end || run.is_line_break_end {
                                *found = Some(None);
                                return;
                            }
                            let trailing =
                                run.text.chars().rev().take_while(|ch| *ch == ' ').count();
                            *found = Some((trailing > 0).then_some((source.clone(), trailing)));
                            return;
                        }
                    }
                }
            }
            LayerNodeKind::Group { children, .. } => {
                for child in children.iter().rev() {
                    scan(child, found);
                    if found.is_some() {
                        return;
                    }
                }
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                scan(child, found)
            }
        }
    }
    let mut found = None;
    for child in children.iter().rev() {
        scan(child, &mut found);
        if found.is_some() {
            break;
        }
    }
    found.flatten()
}

fn publish_line_decoration_trims(node: &mut LayerNode) {
    if let LayerNodeKind::Group {
        children,
        group_kind: GroupKind::TextLine(_),
        ..
    } = &mut node.kind
    {
        if let Some((source, trim)) = line_decoration_trim_target(children) {
            set_decoration_trim_for_source(children, &source, trim);
        }
    }
    match &mut node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                publish_line_decoration_trims(child);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            publish_line_decoration_trims(child)
        }
        LayerNodeKind::Leaf { .. } => {}
    }
}

fn set_decoration_trim_for_source(
    nodes: &mut [LayerNode],
    source: &crate::paint::TextSourceSpan,
    trim: usize,
) {
    for node in nodes {
        match &mut node.kind {
            LayerNodeKind::Group { children, .. } => {
                set_decoration_trim_for_source(children, source, trim);
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                set_decoration_trim_for_source(std::slice::from_mut(child), source, trim);
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::TextDecoration {
                        source: Some(candidate),
                        trim_trailing_spaces,
                        ..
                    } = op
                    {
                        if candidate == source {
                            *trim_trailing_spaces = trim;
                        }
                    }
                }
            }
        }
    }
}

fn text_run_ops(
    bbox: crate::renderer::render_tree::BoundingBox,
    run: crate::renderer::render_tree::TextRunNode,
    output_options: LayerOutputOptions,
) -> Vec<PaintOp> {
    let has_char_overlap = run.char_overlap.is_some();
    let show_text_marks = output_options.show_paragraph_marks || output_options.show_control_codes;
    let has_control_mark = show_text_marks
        && (run.is_para_end
            || run.is_line_break_end
            || run.text.chars().any(|ch| matches!(ch, ' ' | '\t')));
    // Legacy renderers bypass normal text decoration/leader paint for overlap runs.
    let has_tab_leader = !has_char_overlap && !run.style.tab_leaders.is_empty();
    let has_underline = !has_char_overlap && run.style.underline != UnderlineType::None;
    let has_strikethrough = !has_char_overlap && run.style.strikethrough;
    let has_emphasis_dot = !has_char_overlap && run.style.emphasis_dot > 0;

    let mut ops = Vec::with_capacity(
        1 + has_char_overlap as usize
            + has_control_mark as usize
            + has_tab_leader as usize
            + has_underline as usize
            + has_strikethrough as usize
            + has_emphasis_dot as usize,
    );
    ops.push(PaintOp::text_run(bbox, run.clone()));
    if has_char_overlap {
        ops.push(PaintOp::char_overlap(bbox, run.clone()));
    }
    if has_control_mark {
        ops.push(PaintOp::text_control_mark(bbox, run.clone()));
    }
    if has_tab_leader {
        ops.push(PaintOp::tab_leader(bbox, run.clone()));
    }
    if has_underline {
        ops.push(PaintOp::text_decoration(
            bbox,
            run.clone(),
            TextDecorationKind::Underline,
        ));
    }
    if has_strikethrough {
        ops.push(PaintOp::text_decoration(
            bbox,
            run.clone(),
            TextDecorationKind::Strikethrough,
        ));
    }
    if has_emphasis_dot {
        ops.push(PaintOp::text_decoration(
            bbox,
            run,
            TextDecorationKind::EmphasisDot,
        ));
    }
    ops
}

#[cfg(test)]
#[path = "builder/tests.rs"]
mod tests;
