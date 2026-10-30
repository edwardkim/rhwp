//! 논리 조판과 물리 Print 종이의 공통 paint 참조 프레임.

use crate::model::style::UnderlineType;
use crate::paint::RenderProfile;
use crate::paint::{
    LayerNode, LayerNodeKind, PaintOp, TextDecorationKind, TextDecorationPaintGeometry,
};
use crate::renderer::image_resolver::detect_image_mime_type;
use crate::renderer::render_tree::{
    BoundingBox, ImageNode, RectangleNode, RenderNode, RenderNodeType,
};
use crate::renderer::{hwpunit_to_px, DEFAULT_DPI, HWPUNIT_PER_INCH};

/// 원래 정수 HU를 정확히 재현하는 px만 정수 단위로 되돌린다.
/// 역변환 오차로 반 장치 픽셀의 위/아래가 바뀌지 않게 하며 연속 좌표는 보존한다.
fn source_device_coordinate(px: f64, pixel: f64) -> f64 {
    let hu = (px * HWPUNIT_PER_INCH / DEFAULT_DPI).round() as i32;
    if hwpunit_to_px(hu, DEFAULT_DPI).to_bits() == px.to_bits() {
        f64::from(hu) / (pixel * HWPUNIT_PER_INCH / DEFAULT_DPI)
    } else {
        px / pixel
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PagePaintContext {
    print: bool,
    logical_size: (f64, f64),
    output_size: (f64, f64),
    axis_aligned_graphics: bool,
    print_paper: Option<super::print_paper::PrintPaper>,
    image_column_origin: Option<(f64, f64)>,
    image_column_blocked: bool,
    table_fill_origin: super::table_fill_origin::TableFillOrigin,
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
            axis_aligned_graphics: true,
            print_paper: None,
            image_column_origin: None,
            image_column_blocked: false,
            table_fill_origin: Default::default(),
        }
    }

    pub(crate) fn with_hangul2024_print(mut self, enabled: bool) -> Self {
        if self.print && enabled {
            self.print_paper =
                super::print_paper::PrintPaper::from_css(self.logical_size.0, self.logical_size.1);
            if let Some(paper) = self.print_paper {
                self.output_size = paper.size;
            }
        }
        self
    }

    pub(crate) fn output_size(self) -> (f64, f64) {
        self.output_size
    }

    /// 글줄 그림은 실제 소유 단의 원점과 상대 위치를 따로 장치 정수로 변환한다.
    /// 셀·글상자·각주·별도 개체 좌표계를 본문 단의 좌표계로 간주하지 않는다.
    pub(crate) fn for_children(mut self, node: &RenderNode) -> Self {
        self.table_fill_origin = self.table_fill_origin.for_children(node);
        self.image_column_blocked |= node.layer.is_some()
            || matches!(
                &node.node_type,
                RenderNodeType::MasterPage
                    | RenderNodeType::Header
                    | RenderNodeType::Footer
                    | RenderNodeType::FootnoteArea
                    | RenderNodeType::TableCell(_)
                    | RenderNodeType::TextBox
            )
            || matches!(&node.node_type,
                RenderNodeType::Group(group) if group.paint_transform.is_some());
        if self.image_column_blocked {
            self.image_column_origin = None;
        } else if matches!(&node.node_type, RenderNodeType::Column(_)) {
            self.image_column_origin = Some((node.bbox.x, node.bbox.y));
        }
        self
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
        let (sx, sy) = self.print_paper.map_or_else(
            || (ratio(self.logical_size.0), ratio(self.logical_size.1)),
            |paper| {
                (
                    paper.device_units.0 * (600.0 / 96.0),
                    paper.device_units.1 * (600.0 / 96.0),
                )
            },
        );
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
                LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                    publish(child, table_owned, sx, sy)
                }
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
        let (sx, sy) = self.print_paper.map_or_else(
            || (ratio(self.logical_size.0), ratio(self.logical_size.1)),
            |paper| {
                (
                    paper.device_units.0 * (600.0 / 96.0),
                    paper.device_units.1 * (600.0 / 96.0),
                )
            },
        );
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
                LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                    collect(
                        child,
                        sources,
                        rounded,
                        plane,
                        active_layer,
                        hangul2024_print,
                    )
                }
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
                LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                    publish(child, sources, transform)
                }
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
                LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
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
        let (sx, sy) = self.print_paper.map_or_else(
            || (ratio(self.logical_size.0), ratio(self.logical_size.1)),
            |paper| {
                (
                    paper.device_units.0 * (600.0 / 96.0),
                    paper.device_units.1 * (600.0 / 96.0),
                )
            },
        );
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
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
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

    /// 양쪽 backend가 실제로 재생할 그림 frame에서 장치 압축의 너비를 얻는다.
    pub(crate) fn image_codec_width_points(
        self,
        image: &ImageNode,
        frame: BoundingBox,
    ) -> Option<f64> {
        if !self.print || !self.axis_aligned_graphics || image.transform.has_transform() {
            return None;
        }
        let width = image.paint_bbox(&frame).width * 0.75;
        (width.is_finite() && width > 0.0).then_some(width)
    }

    /// 회전/affine 아래의 좌표는 최종 장치 좌표가 아니므로 이 계약을 적용하지 않는다.
    pub(crate) fn without_rectangle_mapping(mut self) -> Self {
        self.axis_aligned_graphics = false;
        self
    }

    /// 시작·끝을 같은 원본 참조에서 변환한다. PAPER/PAGE 축은 절대 좌표를 유지한다.
    fn fill_device_coordinate(
        self,
        value: f64,
        pixel: f64,
        axis: usize,
        table_background: bool,
    ) -> f64 {
        if let Some(origin) = self
            .table_fill_origin
            .axis(axis)
            .filter(|_| table_background && self.print_paper.is_some())
        {
            super::print_coordinate::relative_device(origin, 0.0, pixel).round()
                + super::print_coordinate::relative_device(value, origin, pixel).round()
        } else {
            source_device_coordinate(value, pixel).round()
        }
    }

    /// 원래 글상자·테두리 프레임과 별개로 Print 단색 채움의 중첩을 결정한다.
    pub(crate) fn rectangle_fill_frame(
        self,
        frame: BoundingBox,
        rect: &RectangleNode,
        device_pixel: Option<f64>,
    ) -> Option<BoundingBox> {
        if !self.print || !self.axis_aligned_graphics {
            return None;
        }
        super::rectangle_fill::frame(frame, rect, device_pixel?)
    }

    /// 불투명 단색 채움의 최종 끝점을 Print 장치에서 물리 종이에 대응한다.
    /// NORMAL 선의 채움 중첩 이후에 소비하며 원래 조판·테두리·텍스트 상자는 유지한다.
    pub(crate) fn solid_fill_frame(
        self,
        bounds: BoundingBox,
        rect: &RectangleNode,
        device_pixel: Option<f64>,
        node: &RenderNode,
    ) -> BoundingBox {
        let Some(pixel) = device_pixel.filter(|p| p.is_finite() && *p > 0.0) else {
            return bounds;
        };
        if !self.print
            || !self.axis_aligned_graphics
            || rect.transform.has_transform()
            || !super::device_fill::eligible(rect)
            || ![bounds.x, bounds.y, bounds.width, bounds.height]
                .into_iter()
                .all(f64::is_finite)
            || bounds.width <= 0.0
            || bounds.height <= 0.0
        {
            return bounds;
        }
        let devices = (
            (self.logical_size.0 / pixel).round(),
            (self.logical_size.1 / pixel).round(),
        );
        if ![devices.0, devices.1, self.output_size.0, self.output_size.1]
            .into_iter()
            .all(|extent| extent.is_finite() && extent > 0.0)
        {
            return bounds;
        }
        let (x_unit, y_unit) = self.print_paper.map_or(
            (
                self.output_size.0 / devices.0,
                self.output_size.1 / devices.1,
            ),
            |paper| paper.device_units,
        );
        let background = node.table_background_paint;
        let (x, width) = if node.layer.is_some_and(|layer| layer.master_page) {
            // 바탕쪽의 가로 프레임은 rectangle_frame에서 이미 확정했다.
            (bounds.x, bounds.width)
        } else {
            let left = self.fill_device_coordinate(bounds.x, pixel, 0, background);
            let right = self.fill_device_coordinate(bounds.x + bounds.width, pixel, 0, background);
            (left * x_unit, (right - left) * x_unit)
        };
        let top = self.fill_device_coordinate(bounds.y, pixel, 1, background);
        let bottom = self.fill_device_coordinate(bounds.y + bounds.height, pixel, 1, background);
        BoundingBox::new(x, top * y_unit, width, (bottom - top) * y_unit)
    }

    /// 독립 Print의 실선 그림 테두리는 채운 띠와 장치 한 픽셀의 외곽 pen을 갖는다.
    /// 원본 HU·논리 프레임은 유지하고 양쪽 backend가 소비할 인쇄 선폭만 만든다.
    pub(crate) fn picture_border_pen(self, rect: &mut RectangleNode, pixel: Option<f64>) {
        let (Some(width_hu), Some(pixel)) = (
            rect.picture_border_width_hu,
            pixel.filter(|p| p.is_finite() && *p > 0.0),
        ) else {
            return;
        };
        if !self.print
            || !self.axis_aligned_graphics
            || rect.transform.has_transform()
            || rect.image_frame_owner.is_none()
            || rect.style.stroke_dash != crate::renderer::StrokeDash::Solid
            || width_hu < 0
            || !rect.style.stroke_width.is_finite()
        {
            return;
        }
        // 선언 폭 0은 화면용 0.1mm 기본값이 아니라 인쇄 장치의 cosmetic pen이다.
        let declared = if width_hu == 0 {
            0.0
        } else {
            rect.style.stroke_width / pixel
        };
        let band = declared.round().max(1.0);
        rect.print_picture_pen = Some([band * pixel, pixel]);
        rect.style.stroke_width = (band + 1.0) * pixel;
        // 빈 명시적 반복 구간은 실선이다. 재생기는 확정한 장치 선폭을 그대로 소비한다.
        rect.style.stroke_dash = crate::renderer::StrokeDash::Explicit([0.0; 6]);
    }

    /// 그림의 중심선 경로는 source 경계의 각 끝점을 장치 정수로 변환한다.
    /// 선언 폭 0의 Print 경로에는 화면용 기본 폭의 안쪽 겹침이 없다.
    pub(crate) fn picture_border_frame(
        self,
        bounds: BoundingBox,
        rect: &RectangleNode,
    ) -> Option<BoundingBox> {
        let paper = self.print_paper?;
        if !self.print
            || !self.axis_aligned_graphics
            || rect.print_picture_pen.is_none()
            || rect.transform.has_transform()
            || ![bounds.x, bounds.y, bounds.width, bounds.height]
                .into_iter()
                .all(f64::is_finite)
            || bounds.width <= 0.0
            || bounds.height <= 0.0
        {
            return None;
        }
        let device = 600.0 / 96.0;
        let left = (bounds.x * device).round();
        let top = (bounds.y * device).round();
        let right = ((bounds.x + bounds.width) * device).round();
        let bottom = ((bounds.y + bounds.height) * device).round();
        Some(BoundingBox::new(
            left * paper.device_units.0,
            top * paper.device_units.1,
            (right - left) * paper.device_units.0,
            (bottom - top) * paper.device_units.1,
        ))
    }

    /// 정수 장치 좌표 → 물리 종이 viewport의 가로 축 변환.
    /// 원본 종이/그림자/텍스트 배치나 세로 축을 이 결과로 덮어쓰지 않는다.
    /// 근거: issue8153의 두 종이·15개 독립 Print 사각형 원시 연산.
    pub(crate) fn rectangle_frame(
        self,
        node: &RenderNode,
        rect: &mut RectangleNode,
        device_pixel: Option<f64>,
    ) -> BoundingBox {
        let Some(pixel) = device_pixel.filter(|p| p.is_finite() && *p > 0.0) else {
            return node.bbox;
        };
        if !self.print
            || !self.axis_aligned_graphics
            || !node.layer.as_ref().is_some_and(|layer| layer.master_page)
            || rect.transform.has_transform()
        {
            return node.bbox;
        }
        let bounds = node.bbox;
        let (width, _) = self.logical_size;
        let (output_width, _) = self.output_size;
        if ![bounds.x, bounds.width, width, output_width]
            .into_iter()
            .all(f64::is_finite)
            || bounds.width < 0.0
            || width <= 0.0
            || output_width <= 0.0
        {
            return bounds;
        }
        let device_width = (width / pixel).round();
        if device_width <= 0.0 {
            return bounds;
        }
        let left = (bounds.x / pixel).round();
        let right = ((bounds.x + bounds.width) / pixel).round().max(left + 1.0);
        let unit = self
            .print_paper
            .map_or(output_width / device_width, |paper| paper.device_units.0);
        // 한컴의 퇴화 사각형도 장치의 한 픽셀 폭을 갖는다. 원본의 0폭은 보존한다.
        if rect.style.stroke_width.is_finite() && rect.style.stroke_width > 0.0 {
            rect.style.stroke_width = (rect.style.stroke_width / pixel).round().max(1.0) * pixel;
        }
        BoundingBox::new(left * unit, bounds.y, (right - left) * unit, bounds.height)
    }

    pub(crate) fn image_frame(
        self,
        node: &RenderNode,
        image: &ImageNode,
        _device_pixel: Option<f64>,
    ) -> BoundingBox {
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
        let (sx, sy) = self.print_paper.map_or_else(
            || (ctm(width) * point_to_css, ctm(height) * point_to_css),
            |paper| paper.device_units,
        );
        if ![sx, sy]
            .into_iter()
            .all(|value| value.is_finite() && value > 0.0)
        {
            return bbox;
        }
        // 원본의 앞쪽 구간을 잘랐다면 시작점은 첫 원본 픽셀이 아니라
        // clipping 경계다. 독립 Print의 crop 이미지/clip 연산자는 이 축의
        // 앞 장치 경계를 공유한다. 전체 원본과 뒤쪽만 자른 축은 기존
        // 첫 픽셀 원점을 유지하며, crop 단위나 내용으로 대상을 추정하지 않는다.
        let (left, top) = image.crop.map_or((0, 0), |(left, top, _, _)| (left, top));
        let column = self.image_column_origin.filter(|(x, y)| {
            self.print_paper.is_some()
                && !self.image_column_blocked
                && self.axis_aligned_graphics
                && x.is_finite()
                && y.is_finite()
                && node.layer.is_none()
                && image.fill_mode.is_none()
                && !image.transform.has_transform()
                && image.content_inset.is_none()
                && image.cell_context.is_none()
                && image.header_footer_ref.is_none()
                && left == 0
                && top == 0
        });
        // 원본이 글자처럼 취급한 셀 그림은 표의 실제 참조 축을 상속한다.
        // 셀·표 레이어가 있다는 이유로 본문 단의 소유를 버리지 않는다.
        // 중첩 표·별도 좌표계는 TableFillOrigin이 독립적으로 차단한다.
        let inline_cell = self.print_paper.is_some()
            && self.axis_aligned_graphics
            && node.source_inline_picture
            && node.layer.is_none()
            && image.cell_context.is_some()
            && image.header_footer_ref.is_none()
            && image.fill_mode.is_none()
            && image.content_inset.is_none()
            && !image.transform.has_transform();
        let axis = |index| {
            column
                .map(|(x, y)| if index == 0 { x } else { y })
                .or_else(|| {
                    inline_cell
                        .then(|| self.table_fill_origin.inline_picture_axis(index))
                        .flatten()
                })
        };
        let origin = |value: f64, clipped_start: bool, column_origin: Option<f64>| {
            if clipped_start {
                (value * device_per_css + 1e-9).floor()
            } else if let Some(anchor) = column_origin {
                // 원본 단 원점과 글줄 안 상대 위치는 각각의 정수 장치 좌표다.
                // 더한 뒤 ceil/round하면 서로 다른 소유 단계의 반올림이 사라진다.
                (anchor * device_per_css).round() + ((value - anchor) * device_per_css).round()
            } else {
                (value * device_per_css - 1e-9).ceil()
            }
        };
        BoundingBox::new(
            origin(bbox.x, left > 0, axis(0)) * sx,
            origin(bbox.y, top > 0, axis(1)) * sy,
            (bbox.width * device_per_css).round() * sx,
            (bbox.height * device_per_css).round() * sy,
        )
    }

    /// 영역 맞춤 배경의 재생 그림과 원래 채움 clip을 함께 확정한다.
    pub(super) fn bitmap_fill_frame(
        self,
        node: &RenderNode,
        image: &ImageNode,
        device_pixel: Option<f64>,
    ) -> Option<super::bitmap_fill_frame::BitmapFillFrame> {
        use crate::model::style::ImageFillMode;
        use crate::renderer::image_header::{
            canvaskit_encoded_image_header, CanvasKitEncodedImageFormat,
        };
        let pixel = device_pixel.filter(|p| p.is_finite() && *p > 0.0)?;
        if !self.print
            || !self.axis_aligned_graphics
            || image.transform.has_transform()
            || image.crop.is_some()
            || image.content_inset.is_some()
        {
            return None;
        }
        let header = canvaskit_encoded_image_header(image.data.as_deref()?)?;
        if !header.is_within_decode_limits()
            || !matches!(
                header.format,
                CanvasKitEncodedImageFormat::Bmp
                    | CanvasKitEncodedImageFormat::Png
                    | CanvasKitEncodedImageFormat::Jpeg
            )
        {
            return None;
        }
        let bounds = node.bbox;
        // 가운데 채움의 원본이 양축 모두 채움 영역보다 큰 경우에는
        // 한컴 Print가 영역에 맞춘 채움과 같은 장치 프레임을 사용한다.
        // 원본 크기가 명시된 도형과 한 축이라도 영역에 들어가는 그림은
        // 가운데 정렬 계약이 다르므로 기존 경로에서 처리한다.
        let fills_area = matches!(
            image.fill_mode,
            Some(ImageFillMode::FitToSize | ImageFillMode::Total)
        ) || (image.fill_mode == Some(ImageFillMode::Center)
            && image.original_size.is_none()
            && bounds.width < f64::from(header.width)
            && bounds.height < f64::from(header.height));
        if !fills_area {
            return None;
        }
        let background = node.table_background_paint;
        let left = self.fill_device_coordinate(bounds.x, pixel, 0, background);
        let top = self.fill_device_coordinate(bounds.y, pixel, 1, background);
        let width =
            if background && self.print_paper.is_some() && self.table_fill_origin.axis(0).is_some()
            {
                self.fill_device_coordinate(bounds.x + bounds.width, pixel, 0, background) - left
            } else {
                source_device_coordinate(bounds.width, pixel).round()
            };
        let height =
            if background && self.print_paper.is_some() && self.table_fill_origin.axis(1).is_some()
            {
                self.fill_device_coordinate(bounds.y + bounds.height, pixel, 1, background) - top
            } else {
                source_device_coordinate(bounds.height, pixel).round()
            };
        super::bitmap_fill_frame::map(
            BoundingBox::new(left, top, width, height),
            (header.width, header.height),
            (
                (self.logical_size.0 / pixel).round(),
                (self.logical_size.1 / pixel).round(),
            ),
            self.output_size,
            self.print_paper.map(|paper| paper.device_units),
        )
    }
}
