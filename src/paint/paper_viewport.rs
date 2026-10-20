//! 논리 조판과 물리 Print 종이의 공통 paint 참조 프레임.

use crate::model::style::UnderlineType;
use crate::paint::RenderProfile;
use crate::paint::{
    LayerNode, LayerNodeKind, PaintOp, TextDecorationKind, TextDecorationPaintGeometry,
};
use crate::renderer::image_resolver::detect_image_mime_type;
use crate::renderer::render_tree::{BoundingBox, ImageNode, RenderNode};

#[derive(Clone, Copy)]
pub(crate) struct PagePaintContext {
    print: bool,
    logical_size: (f64, f64),
    output_size: (f64, f64),
}

impl PagePaintContext {
    pub(crate) fn new(profile: RenderProfile, width: f64, height: f64) -> Self {
        let print = profile == RenderProfile::Print;
        // HU에서 변환한 정수 point가 부동소수점 오차로 한 단계 내려가지 않게 한다.
        let physical_extent = |px: f64| (px * 0.75 + 1e-9).floor() / 0.75;
        Self {
            print,
            logical_size: (width, height),
            output_size: if print {
                (physical_extent(width), physical_extent(height))
            } else {
                (width, height)
            },
        }
    }

    pub(crate) fn output_size(self) -> (f64, f64) {
        self.output_size
    }

    /// 표 괘선은 논리 조판 상자를 유지하되 물리 Print 종이의 정밀 CTM을 쓴다.
    /// 독립 Print의 가로·세로 괘선에서 확인한 공통 프레임이며, 장치 정수
    /// 반올림이나 끝점의 pen 확장은 이 좌표 변환으로 추측하지 않는다.
    pub(crate) fn publish_table_border_geometry(self, node: &mut LayerNode) {
        if !self.print {
            return;
        }
        let ratio = |extent: f64| {
            (extent * 0.75 + 1e-9).floor() / (extent * 600.0 / 96.0).round() * (600.0 / 72.0)
        };
        let (sx, sy) = (ratio(self.logical_size.0), ratio(self.logical_size.1));
        if ![sx, sy]
            .into_iter()
            .all(|value| value.is_finite() && value > 0.0)
        {
            return;
        }
        fn publish(node: &mut LayerNode, table_owned: bool, sx: f64, sy: f64) {
            match &mut node.kind {
                LayerNodeKind::Group {
                    children,
                    group_kind,
                    ..
                } => {
                    // 소유를 모든 자손에 상속하면 셀 본문의 도형까지 표 괘선으로
                    // 바뀐다. 표가 직접 발행한 선만 같은 물리 프레임을 소비한다.
                    let table = matches!(group_kind, crate::paint::layer_tree::GroupKind::Table(_));
                    for child in children {
                        publish(child, table, sx, sy);
                    }
                }
                LayerNodeKind::ClipRect { child, .. } => publish(child, table_owned, sx, sy),
                LayerNodeKind::Leaf { ops } if table_owned => {
                    for op in ops {
                        if let PaintOp::Line { bbox, line } = op {
                            line.x1 *= sx;
                            line.x2 *= sx;
                            line.y1 *= sy;
                            line.y2 *= sy;
                            *bbox = line.ink_bbox();
                        }
                    }
                }
                LayerNodeKind::Leaf { .. } => {}
            }
        }
        publish(node, false, sx, sy);
    }

    /// 독립 Print의 도형 polygon/글자 shade fill은 이후 device CTM의 소수점
    /// 출력을 6→3자리로 바꾼다. 2020의 BorderFill 배경은 바꾸지 않지만
    /// 2024의 같은 셀 배경은 바꾼다(동일 입력 C308/C311).
    /// 이 상태는 q/Q와 쪽 경계를 넘어 유지된다(C275/C307/C308 대조).
    /// 원본 HFT의 논리 좌표는 fill 전의 정밀 CTM만 물리 종이에 투영한다.
    /// Source bbox·run·clip은 그대로 두고 모든 glyph 대안에 같은 결정을 전달한다.
    pub(crate) fn publish_glyph_geometry(
        self,
        node: &mut LayerNode,
        prior_solid_fill: bool,
        hangul2024_print: bool,
    ) {
        if !self.print {
            return;
        }
        let ratio =
            |px: f64| (px * 0.75 + 1e-9).floor() / (px * 600.0 / 96.0).round() * (600.0 / 72.0);
        let (sx, sy) = (ratio(self.logical_size.0), ratio(self.logical_size.1));
        if ![sx, sy].iter().all(|v| v.is_finite() && *v > 0.0) {
            return;
        }
        let transform = crate::paint::LayerAffineTransform {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        };
        let mut sources = std::collections::HashSet::new();
        fn collect(
            node: &LayerNode,
            sources: &mut std::collections::HashSet<u32>,
            rounded: &mut bool,
            plane: crate::paint::PaintReplayPlane,
            inherited_layer: Option<crate::renderer::render_tree::RenderLayerInfo>,
            hangul2024_print: bool,
        ) {
            let active_layer = node.layer.or(inherited_layer);
            match &node.kind {
                LayerNodeKind::Group { children, .. } => {
                    for child in children {
                        collect(
                            child,
                            sources,
                            rounded,
                            plane,
                            active_layer,
                            hangul2024_print,
                        );
                    }
                }
                LayerNodeKind::ClipRect { child, .. } => collect(
                    child,
                    sources,
                    rounded,
                    plane,
                    active_layer,
                    hangul2024_print,
                ),
                LayerNodeKind::Leaf { ops } => {
                    for op in ops {
                        if crate::paint::paint_op_replay_plane_with_layer(op, active_layer) != plane
                        {
                            continue;
                        }
                        *rounded |= PagePaintContext::op_has_print_solid_fill(op, hangul2024_print);
                        if let PaintOp::TextRun {
                            run,
                            source: Some(source),
                            ..
                        } = op
                        {
                            if run.style.hft_print_frame && !*rounded {
                                sources.insert(source.id.0);
                            }
                        }
                    }
                }
            }
        }
        fn publish(
            node: &mut LayerNode,
            sources: &std::collections::HashSet<u32>,
            transform: crate::paint::LayerAffineTransform,
        ) {
            match &mut node.kind {
                LayerNodeKind::Group { children, .. } => {
                    for child in children {
                        publish(child, sources, transform);
                    }
                }
                LayerNodeKind::ClipRect { child, .. } => publish(child, sources, transform),
                LayerNodeKind::Leaf { ops } => {
                    for op in ops {
                        match op {
                            PaintOp::TextRun {
                                source: Some(source),
                                paint_transform,
                                ..
                            }
                            | PaintOp::CharOverlap {
                                source: Some(source),
                                paint_transform,
                                ..
                            } if sources.contains(&source.id.0) => {
                                *paint_transform = Some(transform)
                            }
                            PaintOp::GlyphRun {
                                run,
                                paint_transform,
                                ..
                            } if sources.contains(&run.source.id.0) => {
                                *paint_transform = Some(transform)
                            }
                            PaintOp::GlyphOutline {
                                outline,
                                paint_transform,
                                ..
                            } if sources.contains(&outline.source.id.0) => {
                                *paint_transform = Some(transform)
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        fn header_footer_fill(node: &LayerNode, owned: bool, hangul2024_print: bool) -> bool {
            match &node.kind {
                LayerNodeKind::Group {
                    children,
                    group_kind,
                    ..
                } => {
                    let owned = owned
                        || matches!(
                            group_kind,
                            crate::paint::layer_tree::GroupKind::Header
                                | crate::paint::layer_tree::GroupKind::Footer
                        );
                    children
                        .iter()
                        .any(|child| header_footer_fill(child, owned, hangul2024_print))
                }
                LayerNodeKind::ClipRect { child, .. } => {
                    header_footer_fill(child, owned, hangul2024_print)
                }
                LayerNodeKind::Leaf { ops } => {
                    owned
                        && ops.iter().any(|op| {
                            PagePaintContext::op_has_print_solid_fill(op, hangul2024_print)
                        })
                }
            }
        }
        // 인쇄기는 머리말·꼬리말 개체를 본문보다 먼저 출력한다. 그림 테두리
        // 비트만 바꾼 독립 Print에서 첫 본문 이전의 장치 정밀도 전환을 확인했다.
        // 편집·가림의 레이어 및 원문 순서는 보존하고 공통 Print 상태만 질의한다.
        let mut rounded = prior_solid_fill || header_footer_fill(node, false, hangul2024_print);
        for plane in crate::paint::PaintReplayPlane::ORDERED {
            collect(
                node,
                &mut sources,
                &mut rounded,
                plane,
                None,
                hangul2024_print,
            );
        }
        publish(node, &sources, transform);
    }

    fn shape_has_print_solid_fill(style: &crate::renderer::ShapeStyle) -> bool {
        style.opacity > 0.0 && style.fill_color.is_some()
    }

    fn rectangle_has_print_polygon_fill(
        rect: &crate::renderer::render_tree::RectangleNode,
        hangul2024_print: bool,
    ) -> bool {
        // BorderFill에서 생산한 배경 사각형과 원본 도형은 같은 paint primitive를
        // 쓰지만 Print의 device 상태 계약은 다르다. 원본 도형 control 소유는
        // shape layout에서 이미 발행한다. 색·위치·쪽 번호로 배경을 추측하지 않는다.
        ((hangul2024_print || rect.control_index.is_some())
            && Self::shape_has_print_solid_fill(&rect.style))
            || (rect.picture_border
                && rect.style.opacity > 0.0
                && rect.style.stroke_color.is_some()
                && rect.style.stroke_width > 0.0)
    }

    fn op_has_print_solid_fill(op: &PaintOp, hangul2024_print: bool) -> bool {
        match op {
            PaintOp::Rectangle { bbox, rect } => {
                bbox.width > 0.0
                    && bbox.height > 0.0
                    && Self::rectangle_has_print_polygon_fill(rect, hangul2024_print)
            }
            PaintOp::Ellipse { bbox, ellipse } => {
                bbox.width > 0.0
                    && bbox.height > 0.0
                    && Self::shape_has_print_solid_fill(&ellipse.style)
            }
            PaintOp::Path { path, .. } => Self::shape_has_print_solid_fill(&path.style),
            PaintOp::TextRun { run, .. } | PaintOp::CharOverlap { run, .. } => {
                !run.text.is_empty()
                    && crate::model::color::char_shade(run.style.shade_color).is_some()
            }
            // 종이 clear, raster image, glyph 윤곽은 source의 polygon fill이 아니다.
            _ => false,
        }
    }

    pub(crate) fn node_has_print_solid_fill(node: &RenderNode, hangul2024_print: bool) -> bool {
        use crate::renderer::render_tree::RenderNodeType;
        if !node.visible || node.editor_only {
            return false;
        }
        let filled = match &node.node_type {
            RenderNodeType::Rectangle(rect) => {
                node.bbox.width > 0.0
                    && node.bbox.height > 0.0
                    && Self::rectangle_has_print_polygon_fill(rect, hangul2024_print)
            }
            RenderNodeType::Ellipse(ellipse) => {
                node.bbox.width > 0.0
                    && node.bbox.height > 0.0
                    && Self::shape_has_print_solid_fill(&ellipse.style)
            }
            RenderNodeType::Path(path) => Self::shape_has_print_solid_fill(&path.style),
            RenderNodeType::TextRun(run) => {
                !run.text.is_empty()
                    && crate::model::color::char_shade(run.style.shade_color).is_some()
            }
            _ => false,
        };
        filled
            || node
                .children
                .iter()
                .any(|child| Self::node_has_print_solid_fill(child, hangul2024_print))
    }

    /// Hancom Print places solid horizontal underlines in its physical paper
    /// frame. Text still owns its original layout metrics and glyph origins.
    /// Independent six-size Print strokes establish this frame (C224/C240).
    pub(crate) fn publish_decoration_geometry(self, node: &mut LayerNode) {
        if !self.print {
            return;
        }
        let ratio = |px: f64| {
            let device_extent = (px * 600.0 / 96.0).round();
            let point_extent = (px * 0.75 + 1e-9).floor();
            point_extent / device_extent * (600.0 / 72.0)
        };
        let sx = ratio(self.logical_size.0);
        let sy = ratio(self.logical_size.1);
        if ![sx, sy].iter().all(|x| x.is_finite() && *x > 0.0) {
            return;
        }
        Self::publish_decoration_geometry_at(node, sx, sy);
    }

    fn publish_decoration_geometry_at(node: &mut LayerNode, sx: f64, sy: f64) {
        match &mut node.kind {
            LayerNodeKind::Group { children, .. } => {
                for child in children {
                    Self::publish_decoration_geometry_at(child, sx, sy);
                }
            }
            LayerNodeKind::ClipRect { child, .. } => {
                Self::publish_decoration_geometry_at(child, sx, sy)
            }
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    if let PaintOp::TextDecoration {
                        bbox,
                        run,
                        kind: TextDecorationKind::Underline,
                        paint_geometry,
                        ..
                    } = op
                    {
                        // Rotated/vertical, script-shifted and multi-stroke visuals
                        // have separate baseline contracts; do not project them by
                        // a horizontal single-stroke rule.
                        if run.style.underline != UnderlineType::Bottom
                            || run.style.underline_shape != 0
                            || run.rotation != 0.0
                            || run.is_vertical
                            || run.style.superscript
                            || run.style.subscript
                        {
                            continue;
                        }
                        let (font_size, baseline) = run
                            .style
                            .script_draw_metrics(run.style.font_size, run.baseline);
                        let underline_y = bbox.y
                            + baseline
                            + font_size
                                * crate::renderer::text_decoration::UNDERLINE_BASELINE_RATIO;
                        *paint_geometry = Some(TextDecorationPaintGeometry {
                            bbox: BoundingBox::new(
                                bbox.x * sx,
                                bbox.y + underline_y * (sy - 1.0),
                                bbox.width * sx,
                                bbox.height,
                            ),
                            inline_scale: sx,
                        });
                    }
                }
            }
        }
    }

    pub(crate) fn image_frame(self, node: &RenderNode, image: &ImageNode) -> BoundingBox {
        let bbox = node.bbox;
        let is_master = node.layer.as_ref().is_some_and(|layer| layer.master_page);
        let raster = image.data.as_deref().is_some_and(|bytes| {
            matches!(
                detect_image_mime_type(bytes),
                "image/jpeg"
                    | "image/png"
                    | "image/bmp"
                    | "image/tiff"
                    | "image/gif"
                    | "image/webp"
                    | "image/x-pcx"
            )
        });
        if !self.print || !raster {
            return bbox;
        }
        let (width, height) = self.logical_size;
        if ![width, height, bbox.width, bbox.height]
            .iter()
            .all(|extent| extent.is_finite() && *extent > 0.0)
            || ![bbox.x, bbox.y].into_iter().all(f64::is_finite)
        {
            return bbox;
        }
        if image.transform.rotation != 0.0 {
            // 회전 그림은 기존 프레임을 유지한다. 아래 장치 축 계약은
            // 회전하지 않은 그림에서 입증했으며 회전축에 재사용하지 않는다.
            return if is_master {
                let (output_width, output_height) = self.output_size;
                BoundingBox::new(
                    bbox.x * output_width / width,
                    bbox.y * output_height / height,
                    bbox.width * output_width / width,
                    bbox.height * output_height / height,
                )
            } else {
                bbox
            };
        }
        // 독립 Print의 세 소수 원점과 비등방 축소(C352/C354/C356)는
        // 위치를 다음 600dpi 장치 원점, 길이를 최근접 장치 길이로 발행한다.
        // C305의 음영 뒤 checker도 정밀 raster CTM을 유지한다. glyph의
        // 선행 polygon/shade 정밀도 상태를 그림으로 전파하지 않는다.
        // source/layout bbox는 보존하고 crop/fill은 이 paint 프레임을 소비한다.
        let device_per_css = 600.0 / 96.0;
        let point_to_css = 96.0 / 72.0;
        let ctm = |extent: f64| (extent * 0.75 + 1e-9).floor() / (extent * device_per_css).round();
        let (sx, sy) = (ctm(width) * point_to_css, ctm(height) * point_to_css);
        if ![sx, sy]
            .into_iter()
            .all(|value| value.is_finite() && value > 0.0)
        {
            return bbox;
        }
        BoundingBox::new(
            (bbox.x * device_per_css - 1e-9).ceil() * sx,
            (bbox.y * device_per_css - 1e-9).ceil() * sy,
            (bbox.width * device_per_css).round() * sx,
            (bbox.height * device_per_css).round() * sy,
        )
    }
}
