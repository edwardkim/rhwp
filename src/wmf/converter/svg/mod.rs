mod device_context;
mod node;
mod ternary_raster_operator;
mod util;

use crate::wmf::{
    converter::{
        svg::{
            device_context::DeviceContext,
            node::{Data, Node},
            ternary_raster_operator::TernaryRasterOperator,
            util::{as_point_string, url_string, Fill, Stroke},
        },
        GraphicsObject, PlayError, SelectedGraphicsObject,
    },
    imports::*,
    parser::*,
};

#[derive(Default)]
pub struct SVGPlayer {
    context_stack: Vec<DeviceContext>,
    context_current: DeviceContext,
    definitions: Vec<Node>,
    elements: Vec<Node>,
    object_selected: SelectedGraphicsObject,
    current_clip_id: Option<String>,
}

impl SVGPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    fn issue_definition_id(&self) -> String {
        format!("defs{}", self.definitions.len())
    }

    #[inline]
    fn push_element(&mut self, record_number: usize, mut element: Node) {
        if record_number > 0 {
            element = element.set("id", format!("elem{record_number}"));
        }

        if let Some(ref clip_id) = self.current_clip_id {
            element = element.set("clip-path", format!("url(#{clip_id})"));
        }

        self.elements.push(element);
    }

    fn selected_brush(&self) -> &Brush {
        &self.object_selected.brush
    }

    fn selected_pen(&self) -> &Pen {
        &self.object_selected.pen
    }
}

impl crate::wmf::converter::Player for SVGPlayer {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn generate(self) -> Result<Vec<u8>, PlayError> {
        let Self {
            context_current,
            definitions,
            elements,
            ..
        } = self;

        // [Task #860] WMF binary 의 SetWindowExt 가 actual element 의 bbox 보다 작은 경우
        // (예: HWP3 sample14 의 WMF 가 1189 인데 image y+height=2304) viewBox 가 element
        // 를 cover 못 해 rsvg-convert 렌더 시 잘림. element 들의 max x+width / y+height
        // 까지 viewBox 확장 (한컴 SVG 변환의 자동 확장 시멘틱 정합).
        let (raw_vb_x, raw_vb_y, mut vb_w, mut vb_h) = context_current.window.as_view_box();
        let mut vb_x = i32::from(raw_vb_x);
        let mut vb_y = i32::from(raw_vb_y);
        let mut vb_w_i32 = i32::from(vb_w);
        let mut vb_h_i32 = i32::from(vb_h);
        let mut vb_right = vb_x + vb_w_i32;
        let mut vb_bottom = vb_y + vb_h_i32;
        for elem in elements.iter() {
            let max_x = element_max_x(elem);
            let max_y = element_max_y(elem);
            let min_x = element_min_x(elem);
            let min_y = element_min_y(elem);
            if let Some(mx) = max_x {
                if mx > vb_right {
                    vb_right = mx;
                }
            }
            if let Some(my) = max_y {
                if my > vb_bottom {
                    vb_bottom = my;
                }
            }
            // [Task #860 Stage D] element 의 min y 가 viewBox y 보다 작으면 viewBox 위쪽
            // 확장 (HWP3 sample14 의 WMF: BoundingBox origin (329, 1536) 인데 element
            // y=1008 — Placeable header 의 BoundingBox 와 element 좌표 mismatch 보정).
            if let Some(nx) = min_x {
                if nx < vb_x {
                    vb_x = nx;
                }
            }
            if let Some(ny) = min_y {
                if ny < vb_y {
                    vb_y = ny;
                }
            }
        }
        vb_w_i32 = vb_right - vb_x;
        vb_h_i32 = vb_bottom - vb_y;
        // 사용 안 함 (호환성)
        let _ = (&mut vb_w, &mut vb_h);

        let mut document = Node::new("svg")
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("viewBox", format!("{vb_x} {vb_y} {vb_w_i32} {vb_h_i32}"));

        if !definitions.is_empty() {
            let mut defs = Node::new("defs");
            for v in definitions {
                defs = defs.add(v);
            }

            document = document.add(defs);
        }

        // [Task #860 Stage D] WMF SetWindowExt y < 0 (Cartesian, bottom-up) 인 경우
        // SVG (top-down) 정합 위해 y-flip transform 적용:
        //   transform="translate(0, vb_h) scale(1, -1)"
        // 이는 viewBox 안 element 의 y 좌표를 flip 하여 한컴 정합.
        // HWP3 sample14 의 WMF 가 bottom-up 좌표계 사용 (캡션 outline 위, BMP 박스 아래
        // 의 시각이 한컴과 반대로 나타나는 결함 정정).
        if context_current.window.y_inverted {
            let group =
                Node::new("g").set("transform", format!("translate(0,{vb_h_i32}) scale(1,-1)"));
            let mut group = group;
            for v in elements {
                group = group.add(v);
            }
            document = document.add(group);
        } else {
            for v in elements {
                document = document.add(v);
            }
        }

        Ok(document.to_string().into_bytes())
    }

    // .
    // .
    // Functions to handle Bitmap Record
    // .
    // .
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn bit_blt(mut self, record_number: usize, record: META_BITBLT) -> Result<Self, PlayError> {
        let operator = match record {
            META_BITBLT::WithBitmap {
                raster_operation,
                height,
                width,
                y_dest,
                x_dest,
                target,
                ..
            } => {
                let pt = self.context_current.point_s_to_absolute_point(&PointS {
                    x: x_dest,
                    y: y_dest,
                });
                let mut operator =
                    TernaryRasterOperator::new(raster_operation, pt.x, pt.y, height, width);

                if raster_operation.use_selected_brush() {
                    operator = operator.brush(self.selected_brush().clone());
                }

                if raster_operation.use_source() {
                    operator = operator.source_bitmap16(target);
                }

                operator
            }
            META_BITBLT::WithoutBitmap {
                raster_operation,
                height,
                width,
                y_dest,
                x_dest,
                ..
            } => {
                let pt = self.context_current.point_s_to_absolute_point(&PointS {
                    x: x_dest,
                    y: y_dest,
                });
                let mut operator =
                    TernaryRasterOperator::new(raster_operation, pt.x, pt.y, height, width);

                if raster_operation.use_selected_brush() {
                    operator = operator.brush(self.selected_brush().clone());
                }

                operator
            }
        };

        let Some(elem) =
            operator
                .run(&mut self.definitions)
                .map_err(|err| PlayError::InvalidRecord {
                    cause: err.to_string(),
                })?
        else {
            return Ok(self);
        };

        self.push_element(record_number, elem);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn device_independent_bitmap_bit_blt(
        mut self,
        record_number: usize,
        record: META_DIBBITBLT,
    ) -> Result<Self, PlayError> {
        let operator = match record {
            META_DIBBITBLT::WithBitmap {
                raster_operation,
                height,
                width,
                y_dest,
                x_dest,
                target,
                ..
            } => {
                let pt = self.context_current.point_s_to_absolute_point(&PointS {
                    x: x_dest,
                    y: y_dest,
                });
                let mut operator =
                    TernaryRasterOperator::new(raster_operation, pt.x, pt.y, height, width);

                if raster_operation.use_selected_brush() {
                    operator = operator.brush(self.selected_brush().clone());
                }

                if raster_operation.use_source() {
                    operator = operator.source_bitmap(*target);
                }

                operator
            }
            META_DIBBITBLT::WithoutBitmap {
                raster_operation,
                height,
                width,
                y_dest,
                x_dest,
                ..
            } => {
                let pt = self.context_current.point_s_to_absolute_point(&PointS {
                    x: x_dest,
                    y: y_dest,
                });
                let mut operator =
                    TernaryRasterOperator::new(raster_operation, pt.x, pt.y, height, width);

                if raster_operation.use_selected_brush() {
                    operator = operator.brush(self.selected_brush().clone());
                }

                operator
            }
        };

        let Some(elem) =
            operator
                .run(&mut self.definitions)
                .map_err(|err| PlayError::InvalidRecord {
                    cause: err.to_string(),
                })?
        else {
            return Ok(self);
        };

        self.push_element(record_number, elem);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn device_independent_bitmap_stretch_blt(
        mut self,
        record_number: usize,
        record: META_DIBSTRETCHBLT,
    ) -> Result<Self, PlayError> {
        let operator = match record {
            META_DIBSTRETCHBLT::WithBitmap {
                raster_operation,
                dest_height,
                dest_width,
                y_dest,
                x_dest,
                target,
                ..
            } => {
                let pt = self.context_current.point_s_to_absolute_point(&PointS {
                    x: x_dest,
                    y: y_dest,
                });
                let mut operator = TernaryRasterOperator::new(
                    raster_operation,
                    pt.x,
                    pt.y,
                    dest_height,
                    dest_width,
                );

                if raster_operation.use_selected_brush() {
                    operator = operator.brush(self.selected_brush().clone());
                }

                if raster_operation.use_source() {
                    operator = operator.source_bitmap(*target);
                }

                operator
            }
            META_DIBSTRETCHBLT::WithoutBitmap {
                raster_operation,
                dest_height,
                dest_width,
                y_dest,
                x_dest,
                ..
            } => {
                let pt = self.context_current.point_s_to_absolute_point(&PointS {
                    x: x_dest,
                    y: y_dest,
                });
                let mut operator = TernaryRasterOperator::new(
                    raster_operation,
                    pt.x,
                    pt.y,
                    dest_height,
                    dest_width,
                );

                if raster_operation.use_selected_brush() {
                    operator = operator.brush(self.selected_brush().clone());
                }

                operator
            }
        };

        let Some(elem) =
            operator
                .run(&mut self.definitions)
                .map_err(|err| PlayError::InvalidRecord {
                    cause: err.to_string(),
                })?
        else {
            return Ok(self);
        };

        self.push_element(record_number, elem);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_device_independent_bitmap_to_dev(
        self,
        record_number: usize,
        record: META_SETDIBTODEV,
    ) -> Result<Self, PlayError> {
        info!("META_SETDIBTODEV: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn stretch_blt(
        mut self,
        record_number: usize,
        record: META_STRETCHBLT,
    ) -> Result<Self, PlayError> {
        let operator = match record {
            META_STRETCHBLT::WithBitmap {
                raster_operation,
                dest_height,
                dest_width,
                y_dest,
                x_dest,
                target,
                ..
            } => {
                let pt = self.context_current.point_s_to_absolute_point(&PointS {
                    x: x_dest,
                    y: y_dest,
                });
                let mut operator = TernaryRasterOperator::new(
                    raster_operation,
                    pt.x,
                    pt.y,
                    dest_height,
                    dest_width,
                );

                if raster_operation.use_selected_brush() {
                    operator = operator.brush(self.selected_brush().clone());
                }

                if raster_operation.use_source() {
                    operator = operator.source_bitmap16(target);
                }

                operator
            }
            META_STRETCHBLT::WithoutBitmap {
                raster_operation,
                dest_height,
                dest_width,
                y_dest,
                x_dest,
                ..
            } => {
                let pt = self.context_current.point_s_to_absolute_point(&PointS {
                    x: x_dest,
                    y: y_dest,
                });
                let mut operator = TernaryRasterOperator::new(
                    raster_operation,
                    pt.x,
                    pt.y,
                    dest_height,
                    dest_width,
                );

                if raster_operation.use_selected_brush() {
                    operator = operator.brush(self.selected_brush().clone());
                }

                operator
            }
        };

        let Some(elem) =
            operator
                .run(&mut self.definitions)
                .map_err(|err| PlayError::InvalidRecord {
                    cause: err.to_string(),
                })?
        else {
            return Ok(self);
        };

        self.push_element(record_number, elem);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn stretch_device_independent_bitmap(
        mut self,
        record_number: usize,
        record: META_STRETCHDIB,
    ) -> Result<Self, PlayError> {
        let META_STRETCHDIB {
            raster_operation,
            dest_height,
            dest_width,
            y_dst,
            x_dst,
            dib,
            ..
        } = record;

        let pt = self
            .context_current
            .point_s_to_absolute_point(&PointS { x: x_dst, y: y_dst });
        let mut operator =
            TernaryRasterOperator::new(raster_operation, pt.x, pt.y, dest_height, dest_width);

        if raster_operation.use_selected_brush() {
            operator = operator.brush(self.selected_brush().clone());
        }

        if raster_operation.use_source() {
            operator = operator.source_bitmap(dib);
        }

        let Some(elem) =
            operator
                .run(&mut self.definitions)
                .map_err(|err| PlayError::InvalidRecord {
                    cause: err.to_string(),
                })?
        else {
            return Ok(self);
        };

        self.push_element(record_number, elem);

        Ok(self)
    }

    // .
    // .
    // Functions to handle Control Record
    // .
    // .
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn eof(self, record_number: usize, _: META_EOF) -> Result<Self, PlayError> {
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn header(mut self, record_number: usize, header: MetafileHeader) -> Result<Self, PlayError> {
        let (placeable, header) = match header {
            MetafileHeader::StartsWithHeader(header) => (None, header),
            MetafileHeader::StartsWithPlaceable(placeable, header) => (Some(placeable), header),
        };

        if let Some(placeable) = placeable {
            let Rect {
                left,
                top,
                right,
                bottom,
            } = placeable.bounding_box;

            self.context_current = self
                .context_current
                .window_origin(left, top)
                .window_ext(right - left, bottom - top);
        }

        self.context_current = self
            .context_current
            .create_object_table(header.number_of_objects);

        Ok(self)
    }

    // .
    // .
    // Functions to handle Drawing Record
    // .
    // .

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn arc(mut self, record_number: usize, record: META_ARC) -> Result<Self, PlayError> {
        let stroke = Stroke::from(self.selected_pen().clone());
        let start = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: record.x_start_arc,
                y: record.y_start_arc,
            });

            self.context_current = self.context_current.extend_window(&point);
            point
        };
        let end = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: record.x_end_arc,
                y: record.y_end_arc,
            });

            self.context_current = self.context_current.extend_window(&point);
            point
        };
        let (rx, ry) = (
            (record.right_rect - record.left_rect) / 2,
            (record.bottom_rect - record.top_rect) / 2,
        );
        let center = self.context_current.point_s_to_absolute_point(&PointS {
            x: record.left_rect + rx,
            y: record.top_rect + ry,
        });
        // Start and end vectors relative to the center of the ellipse
        let start_dx = f32::from(start.x - center.x);
        let start_dy = f32::from(start.y - center.y);
        let end_dx = f32::from(end.x - center.x);
        let end_dy = f32::from(end.y - center.y);

        // Calculate cross product to determine if the arc is larger than 180
        // degrees. Invert the sign because upper-left is origin.
        let cross = -(start_dx * end_dy - start_dy * end_dx);
        // If the arc is less than 180 degrees (equivalent to the cross product
        // is positive), it is not the larger arc.
        let large_arc = i16::from(cross < 0.0);

        // sweep is always 0 ( equivalent to "counter-clockwise" in SVG )
        let data = Data::new()
            .move_to(format!("{} {}", start.x, start.y))
            .elliptical_arc_to(format!(
                "{} {} {} {} {} {} {}",
                rx, ry, 0, large_arc, 0, end.x, end.y
            ));
        let path = Node::new("path").set("fill", "none").set("d", data);
        let path = stroke.set_props(path);

        self.context_current = self.context_current.drawing_position(end);
        self.push_element(record_number, path);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn chord(mut self, record_number: usize, record: META_CHORD) -> Result<Self, PlayError> {
        // Calculate ellipse center and radii from bounding rectangle
        let rx = (record.right_rect - record.left_rect) / 2;
        let ry = (record.bottom_rect - record.top_rect) / 2;
        if rx == 0 || ry == 0 {
            info!("META_CHORD is skipped because rx or ry is zero.");
            return Ok(self);
        }
        let center = self.context_current.point_s_to_absolute_point(&PointS {
            x: record.left_rect + rx,
            y: record.top_rect + ry,
        });

        // Convert radial endpoints from WMF coordinates to SVG absolute
        // coordinates
        let p1 = self.context_current.point_s_to_absolute_point(&PointS {
            x: record.x_radial1,
            y: record.y_radial1,
        });
        let p2 = self.context_current.point_s_to_absolute_point(&PointS {
            x: record.x_radial2,
            y: record.y_radial2,
        });

        // Build SVG path for chord: move to first radial, draw arc, line to
        // center, close path
        let fill = match Fill::from(self.selected_brush().clone()) {
            Fill::Pattern { pattern } => {
                let id = self.issue_definition_id();
                self.definitions.push(pattern.set("id", id.as_str()));
                url_string(format!("#{id}").as_str())
            }
            Fill::Value { value } => value,
        };
        let fill_rule = self.context_current.poly_fill_rule();
        let stroke = Stroke::from(self.selected_pen().clone());

        // SVG arc parameters:
        // - always small arc (large_arc=0)
        // - always clockwise (sweep=1)
        let large_arc = 0;
        let sweep = 1;
        let data = Data::new()
            .move_to(format!("{} {}", p1.x, p1.y))
            .elliptical_arc_to(format!(
                "{} {} 0 {} {} {} {}",
                rx, ry, large_arc, sweep, p2.x, p2.y
            ))
            .line_to(format!("{} {}", center.x, center.y))
            .close();
        let path = Node::new("path")
            .set("fill", fill.as_str())
            .set("fill-rule", fill_rule.as_str())
            .set("d", data);
        let path = stroke.set_props(path);

        self.push_element(record_number, path);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn ellipse(mut self, record_number: usize, record: META_ELLIPSE) -> Result<Self, PlayError> {
        let (rx, ry) = (
            (record.right_rect - record.left_rect) / 2,
            (record.bottom_rect - record.top_rect) / 2,
        );

        if rx == 0 || ry == 0 {
            info!(
                %rx, %ry,
                "META_ELLIPSE is skipped because rx or ry is zero.",
            );

            return Ok(self);
        }

        let stroke = Stroke::from(self.selected_pen().clone());
        let fill = match Fill::from(self.selected_brush().clone()) {
            Fill::Pattern { pattern } => {
                let id = self.issue_definition_id();
                self.definitions.push(pattern.set("id", id.as_str()));
                url_string(format!("#{id}").as_str())
            }
            Fill::Value { value } => value,
        };
        let fill_rule = self.context_current.poly_fill_rule();
        let point = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: record.left_rect + rx,
                y: record.top_rect + ry,
            });

            self.context_current = self.context_current.extend_window(&point);
            point
        };

        let ellipse = Node::new("ellipse")
            .set("fill", fill.as_str())
            .set("fill-rule", fill_rule.as_str())
            .set("cx", point.x)
            .set("cy", point.y)
            .set("rx", rx)
            .set("ry", ry);
        let ellipse = stroke.set_props(ellipse);

        self.push_element(record_number, ellipse);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn ext_flood_fill(
        self,
        record_number: usize,
        record: META_EXTFLOODFILL,
    ) -> Result<Self, PlayError> {
        info!("META_EXTFLOODFILL: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn ext_text_out(
        mut self,
        record_number: usize,
        record: META_EXTTEXTOUT,
    ) -> Result<Self, PlayError> {
        use unicode_segmentation::UnicodeSegmentation;
        use unicode_width::UnicodeWidthStr;

        let font = &self.object_selected.font;
        let text_content =
            record
                .into_utf8(font.charset)
                .map_err(|err| PlayError::InvalidRecord {
                    cause: err.to_string(),
                })?;
        let point = {
            let point = PointS {
                x: if self.context_current.text_align_update_cp {
                    self.context_current.drawing_position.x
                } else {
                    record.x
                },
                y: if self.context_current.text_align_update_cp {
                    self.context_current.drawing_position.y
                } else {
                    record.y
                } + match self.context_current.text_align_vertical {
                    // [Task #965 / PR #918 Stage 33-A] WMF 의 ExtTextOut y 는
                    // text_align_vertical 에 따라 reference point 가 결정된다:
                    //   VTA_BASELINE (default): y 가 baseline — 그대로 사용
                    //   VTA_TOP: y 가 cell 의 top — baseline = y + ascent
                    //   VTA_BOTTOM: y 가 cell 의 bottom — baseline = y - descent
                    // font.height 의 부호는 magnitude 의 해석 (cell vs char) 만 바꾸며
                    // reference point 와 무관하다 (이전 구현이 font.height < 0 일 때
                    // -font.height 만큼 y 를 더했던 것은 잘못된 보정으로, 텍스트가 박스
                    // 하단으로 baseline shift 되는 원인).
                    VerticalTextAlignmentMode::VTA_TOP => {
                        let em = font.height.abs();
                        (em as f64 * 0.8) as i16
                    }
                    VerticalTextAlignmentMode::VTA_BOTTOM => {
                        let em = font.height.abs();
                        -((em as f64 * 0.2) as i16)
                    }
                    VerticalTextAlignmentMode::VTA_BASELINE => 0,
                    // VTA_CENTER / VTA_LEFT: 드물게 사용; baseline 과 동일 처리
                    _ => 0,
                },
            };

            let point = if self.context_current.text_align_update_cp {
                point
            } else {
                self.context_current.point_s_to_absolute_point(&point)
            };

            self.context_current = self.context_current.extend_window(&point);
            point
        };
        let text_align = self.context_current.as_css_text_align();
        let shape_inside = if let (true, Some(rect)) = (
            record.fw_opts.contains(&ExtTextOutOptions::ETO_CLIPPED),
            record.rectangle,
        ) {
            let tl = {
                let point = PointS {
                    x: rect.left,
                    y: rect.top,
                };
                let point = if self.context_current.text_align_update_cp {
                    self.context_current.point_s_to_relative_point(&point)
                } else {
                    self.context_current.point_s_to_absolute_point(&point)
                };

                self.context_current = self.context_current.extend_window(&point);
                point
            };
            let tr = {
                let point = PointS {
                    x: rect.right,
                    y: rect.top,
                };
                let point = if self.context_current.text_align_update_cp {
                    self.context_current.point_s_to_relative_point(&point)
                } else {
                    self.context_current.point_s_to_absolute_point(&point)
                };

                self.context_current = self.context_current.extend_window(&point);
                point
            };
            let bl = {
                let point = PointS {
                    x: rect.left,
                    y: rect.bottom,
                };
                let point = if self.context_current.text_align_update_cp {
                    self.context_current.point_s_to_relative_point(&point)
                } else {
                    self.context_current.point_s_to_absolute_point(&point)
                };

                self.context_current = self.context_current.extend_window(&point);
                point
            };
            let br = {
                let point = PointS {
                    x: rect.right,
                    y: rect.bottom,
                };
                let point = if self.context_current.text_align_update_cp {
                    self.context_current.point_s_to_relative_point(&point)
                } else {
                    self.context_current.point_s_to_absolute_point(&point)
                };

                self.context_current = self.context_current.extend_window(&point);
                point
            };

            Some(format!(
                "shape-inside: polygon({} {} {} {});",
                as_point_string(&tl),
                as_point_string(&bl),
                as_point_string(&br),
                as_point_string(&tr),
            ))
        } else {
            None
        };

        let mut text = Node::new("text")
            .set("x", point.x)
            .set("y", point.y)
            .set("text-anchor", text_align)
            .set(
                "dominant-baseline",
                self.context_current.as_css_text_align_vertical(),
            )
            .set("fill", self.context_current.text_color_as_css_color());

        if record.dx.len() <= 1 {
            text = text.add(Node::new_text(&text_content));
        } else {
            // https://cgit.freedesktop.org/libreoffice/core/tree/emfio/source/reader/wmfreader.cxx?id=c0b14ab9aa4d713a6b718ef07b9e0379b88e97d3#n693
            for (i, s) in text_content.graphemes(true).enumerate() {
                let dx = if i == 0 {
                    0
                } else {
                    *record.dx.get(i - 1).unwrap_or(&0)
                };

                let mut tspan = Node::new("tspan").add(Node::new_text(s));

                if dx != 0 {
                    let excess_dx = (font.height.abs() / 2) * i16::try_from(s.width()).unwrap_or(0);
                    let dx = core::cmp::max(dx - excess_dx, 0);

                    tspan = tspan.set("dx", dx);
                }

                text = text.add(tspan);
            }
        }

        let (mut text, mut styles) = self.object_selected.font.set_props(text, &point);

        if let Some(shape_inside) = shape_inside {
            styles.push(shape_inside);
        }

        if !styles.is_empty() {
            text = text.set("style", styles.join(""));
        }

        if self.context_current.text_align_update_cp {
            let dx = (font.height.abs() / 2) * i16::try_from(text_content.width()).unwrap_or(0);
            let point = PointS {
                x: point.x + dx,
                y: point.y,
            };
            self.context_current = self.context_current.drawing_position(point);
        }

        // HACK: background color
        // https://stackoverflow.com/a/31013492
        if self.context_current.bk_mode == MixMode::OPAQUE {
            let id = self.issue_definition_id();
            let brush = if matches!(self.selected_brush(), Brush::Null) {
                Brush::Solid {
                    color_ref: self.context_current.text_bk_color.clone(),
                }
            } else {
                self.selected_brush().clone()
            };

            self.definitions
                .push(brush.as_filter().set("id", id.as_str()));

            text = text.set("filter", url_string(format!("#{id}").as_str()));
        }

        self.push_element(record_number, text);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn fill_region(self, record_number: usize, record: META_FILLREGION) -> Result<Self, PlayError> {
        info!("META_FILLREGION: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn flood_fill(self, record_number: usize, record: META_FLOODFILL) -> Result<Self, PlayError> {
        info!("META_FLOODFILL: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn frame_region(
        self,
        record_number: usize,
        record: META_FRAMEREGION,
    ) -> Result<Self, PlayError> {
        info!("META_FRAMEREGION: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn invert_region(
        self,
        record_number: usize,
        record: META_INVERTREGION,
    ) -> Result<Self, PlayError> {
        info!("META_INVERTREGION: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn line_to(mut self, record_number: usize, record: META_LINETO) -> Result<Self, PlayError> {
        let stroke = Stroke::from(self.selected_pen().clone());
        let point = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: record.x,
                y: record.y,
            });

            self.context_current = self.context_current.extend_window(&point);
            point
        };

        let data = Data::new()
            .move_to(format!(
                "{} {}",
                self.context_current.drawing_position.x, self.context_current.drawing_position.y
            ))
            .line_to(format!("{} {}", point.x, point.y));
        let path = Node::new("path").set("fill", "none").set("d", data);
        let path = stroke.set_props(path);

        self.context_current = self.context_current.drawing_position(point);
        self.push_element(record_number, path);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn paint_region(
        self,
        record_number: usize,
        record: META_PAINTREGION,
    ) -> Result<Self, PlayError> {
        info!("META_PAINTREGION: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn pat_blt(mut self, record_number: usize, record: META_PATBLT) -> Result<Self, PlayError> {
        if record.width == 0 || record.height == 0 {
            info!(
                %record.width,
                %record.height,
                "META_PATBLT is skipped because width or height is zero.",
            );

            return Ok(self);
        }

        let fill = match Fill::from(self.selected_brush().clone()) {
            Fill::Pattern { pattern } => {
                let id = self.issue_definition_id();
                self.definitions.push(pattern.set("id", id.as_str()));
                url_string(format!("#{id}").as_str())
            }
            Fill::Value { value } => value,
        };
        let fill_rule = self.context_current.poly_fill_rule();

        let rect = Node::new("rect")
            .set("fill", fill.as_str())
            .set("fill-rule", fill_rule.as_str())
            .set("stroke", "none")
            .set("x", record.x_left)
            .set("y", record.y_left)
            .set("height", record.height)
            .set("width", record.width);

        self.push_element(record_number, rect);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn pie(mut self, record_number: usize, record: META_PIE) -> Result<Self, PlayError> {
        let brush = self.selected_brush();
        let stroke = Stroke::from(brush.clone());
        let fill = match Fill::from(brush.clone()) {
            Fill::Pattern { pattern } => {
                let id = self.issue_definition_id();
                self.definitions.push(pattern.set("id", id.as_str()));
                url_string(format!("#{id}").as_str())
            }
            Fill::Value { value } => value,
        };
        let fill_rule = self.context_current.poly_fill_rule();
        let (rx, ry) = (
            (record.right_rect - record.left_rect) / 2,
            (record.bottom_rect - record.top_rect) / 2,
        );
        let (center_x, center_y) = (record.left_rect + rx, record.top_rect + ry);

        let ellipse = Node::new("ellipse")
            .set("fill", fill.as_str())
            .set("fill-rule", fill_rule.as_str())
            .set("cx", center_x)
            .set("cy", center_y)
            .set("rx", rx)
            .set("ry", ry);
        let ellipse = stroke.set_props(ellipse);

        let stroke = Stroke::from(self.selected_pen().clone());
        let p1 = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: record.x_radial1,
                y: record.y_radial1,
            });

            self.context_current = self.context_current.extend_window(&point);
            point
        };
        let center = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: center_x,
                y: center_y,
            });
            self.context_current = self.context_current.extend_window(&point);
            point
        };
        let p2 = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: record.x_radial2,
                y: record.y_radial2,
            });

            self.context_current = self.context_current.extend_window(&point);
            point
        };

        let data = Data::new()
            .move_to(format!("{} {}", p1.x, p1.y))
            .line_to(format!("{} {}", center.x, center.y))
            .line_to(format!("{} {}", p2.x, p2.y));
        let path = Node::new("path").set("fill", "none").set("d", data);
        let path = stroke.set_props(path);

        self.context_current = self.context_current.drawing_position(p2);
        self.push_element(record_number, ellipse);
        self.push_element(record_number, path);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn polyline(mut self, record_number: usize, record: META_POLYLINE) -> Result<Self, PlayError> {
        let stroke = Stroke::from(self.selected_pen().clone());
        let Some(point) = record.a_points.first() else {
            return Err(PlayError::InvalidRecord {
                cause: "aPoints[0] is not defined".to_owned(),
            });
        };

        let mut coordinate = {
            let point = self.context_current.point_s_to_absolute_point(point);
            self.context_current = self.context_current.extend_window(&point);
            point
        };

        let mut data = Data::new().move_to(format!("{} {}", coordinate.x, coordinate.y));

        for i in 1..record.number_of_points {
            let Some(point) = record.a_points.get(i as usize) else {
                return Err(PlayError::InvalidRecord {
                    cause: format!("aPoints[{i}] is not defined"),
                });
            };

            coordinate = {
                let point = self.context_current.point_s_to_absolute_point(point);
                self.context_current = self.context_current.extend_window(&point);
                point
            };

            data = data.line_to(format!("{} {}", coordinate.x, coordinate.y));
        }

        let path = Node::new("path").set("fill", "none").set("d", data);
        let path = stroke.set_props(path);

        self.context_current = self.context_current.drawing_position(coordinate);
        self.push_element(record_number, path);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn polygon(mut self, record_number: usize, record: META_POLYGON) -> Result<Self, PlayError> {
        if record.number_of_points == 0 {
            info!(%record.number_of_points, "polygon has no points");
            return Ok(self);
        }

        let stroke = Stroke::from(self.selected_pen().clone());
        let fill = match Fill::from(self.selected_brush().clone()) {
            Fill::Pattern { pattern } => {
                let id = self.issue_definition_id();
                self.definitions.push(pattern.set("id", id.as_str()));
                url_string(format!("#{id}").as_str())
            }
            Fill::Value { value } => value,
        };
        let fill_rule = self.context_current.poly_fill_rule();

        let mut points = Vec::with_capacity(record.number_of_points as usize);

        for i in 0..record.number_of_points {
            let Some(point) = record.a_points.get(i as usize) else {
                return Err(PlayError::InvalidRecord {
                    cause: format!("aPoints[{i}] is not defined"),
                });
            };

            let point = {
                let point = self.context_current.point_s_to_absolute_point(point);
                self.context_current = self.context_current.extend_window(&point);
                point
            };

            points.push(as_point_string(&point));
        }

        let polygon = Node::new("polygon")
            .set("fill", fill.as_str())
            .set("fill-rule", fill_rule.as_str())
            .set("points", points.join(" "));
        let polygon = stroke.set_props(polygon);

        self.push_element(record_number, polygon);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn poly_polygon(
        mut self,
        record_number: usize,
        record: META_POLYPOLYGON,
    ) -> Result<Self, PlayError> {
        let stroke = Stroke::from(self.selected_pen().clone());
        let fill = match Fill::from(self.selected_brush().clone()) {
            Fill::Pattern { pattern } => {
                let id = self.issue_definition_id();
                self.definitions.push(pattern.set("id", id.as_str()));
                url_string(format!("#{id}").as_str())
            }
            Fill::Value { value } => value,
        };
        let fill_rule = self.context_current.poly_fill_rule();

        let mut a_point: VecDeque<_> = record.poly_polygon.a_points.into();
        let mut current_point_index = 0;

        for i in 0..record.poly_polygon.number_of_polygons {
            let Some(points_of_polygon) = record.poly_polygon.a_points_per_polygon.get(i as usize)
            else {
                return Err(PlayError::InvalidRecord {
                    cause: format!("aPointsPerPolygon[{i}] is not defined"),
                });
            };

            let mut points = Vec::with_capacity(*points_of_polygon as usize);

            for _ in 0..*points_of_polygon {
                let Some(point) = a_point.pop_front() else {
                    return Err(PlayError::InvalidRecord {
                        cause: format!("aPoints[{current_point_index}] is not defined"),
                    });
                };

                let point = {
                    let point = self.context_current.point_s_to_absolute_point(&point);
                    self.context_current = self.context_current.extend_window(&point);
                    point
                };

                points.push(as_point_string(&point));
                current_point_index += 1;
            }

            let polygon = Node::new("polygon")
                .set("fill", fill.as_str())
                .set("fill-rule", fill_rule.as_str())
                .set("points", points.join(" "));
            let polygon = stroke.set_props(polygon);

            self.push_element(record_number, polygon);
        }

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn rectangle(
        mut self,
        record_number: usize,
        record: META_RECTANGLE,
    ) -> Result<Self, PlayError> {
        let stroke = Stroke::from(self.selected_pen().clone());
        let fill = match Fill::from(self.selected_brush().clone()) {
            Fill::Pattern { pattern } => {
                let id = self.issue_definition_id();
                self.definitions.push(pattern.set("id", id.as_str()));
                url_string(format!("#{id}").as_str())
            }
            Fill::Value { value } => value,
        };
        let fill_rule = self.context_current.poly_fill_rule();
        let tl = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: record.left_rect,
                y: record.top_rect,
            });

            self.context_current = self.context_current.extend_window(&point);
            point
        };
        let br = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: record.right_rect,
                y: record.bottom_rect,
            });

            self.context_current = self.context_current.extend_window(&point);
            point
        };

        let rect = Node::new("rect")
            .set("fill", fill.as_str())
            .set("fill-rule", fill_rule.as_str())
            .set("x", tl.x)
            .set("y", tl.y)
            .set("height", br.y - tl.y)
            .set("width", br.x - tl.x);
        let rect = stroke.set_props(rect);

        self.push_element(record_number, rect);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn round_rect(
        mut self,
        record_number: usize,
        record: META_ROUNDRECT,
    ) -> Result<Self, PlayError> {
        let (width, height) = (
            record.right_rect - record.left_rect,
            record.bottom_rect - record.top_rect,
        );

        if width == 0 || height == 0 {
            info!(
                %width, %height,
                "META_ROUNDRECT is skipped because width or height is zero.",
            );

            return Ok(self);
        }

        let stroke = Stroke::from(self.selected_pen().clone());
        let fill = match Fill::from(self.selected_brush().clone()) {
            Fill::Pattern { pattern } => {
                let id = self.issue_definition_id();
                self.definitions.push(pattern.set("id", id.as_str()));
                url_string(format!("#{id}").as_str())
            }
            Fill::Value { value } => value,
        };
        let fill_rule = self.context_current.poly_fill_rule();
        let point = {
            let point = self.context_current.point_s_to_absolute_point(&PointS {
                x: record.left_rect,
                y: record.top_rect,
            });

            self.context_current = self.context_current.extend_window(&point);
            point
        };

        let rect = Node::new("rect")
            .set("fill", fill.as_str())
            .set("fill-rule", fill_rule.as_str())
            .set("x", point.x)
            .set("y", point.y)
            .set("height", height)
            .set("width", width)
            .set("rx", record.width)
            .set("ry", record.height);
        let rect = stroke.set_props(rect);

        self.push_element(record_number, rect);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_pixel(self, record_number: usize, record: META_SETPIXEL) -> Result<Self, PlayError> {
        info!("META_SETPIXEL: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn text_out(mut self, record_number: usize, record: META_TEXTOUT) -> Result<Self, PlayError> {
        let font = &self.object_selected.font;
        let text_content =
            record
                .into_utf8(font.charset)
                .map_err(|err| PlayError::InvalidRecord {
                    cause: err.to_string(),
                })?;
        let point = {
            let point = PointS {
                x: record.x_start,
                y: record.y_start
                    + match self.context_current.text_align_vertical {
                        // [Task #965 / PR #918 Stage 33-A] META_TEXTOUT 의 y 도 동일.
                        // ext_text_out 의 baseline 보정과 일관성 유지.
                        VerticalTextAlignmentMode::VTA_TOP => {
                            let em = font.height.abs();
                            (em as f64 * 0.8) as i16
                        }
                        VerticalTextAlignmentMode::VTA_BOTTOM => {
                            let em = font.height.abs();
                            -((em as f64 * 0.2) as i16)
                        }
                        VerticalTextAlignmentMode::VTA_BASELINE => 0,
                        _ => 0,
                    },
            };

            let point = if self.context_current.text_align_update_cp {
                self.context_current.point_s_to_relative_point(&point)
            } else {
                self.context_current.point_s_to_absolute_point(&point)
            };

            self.context_current = self.context_current.extend_window(&point);
            point
        };

        let text = Node::new("text")
            .set("x", point.x)
            .set("y", point.y)
            .set("fill", self.context_current.text_color_as_css_color())
            .add(Node::new_text(text_content));
        let (text, styles) = self.object_selected.font.set_props(text, &point);
        let text = text.set("style", styles.join(""));

        self.push_element(record_number, text);

        Ok(self)
    }

    // .
    // .
    // Functions to handle Object Record
    // .
    // .
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn create_brush_indirect(
        mut self,
        record_number: usize,
        record: META_CREATEBRUSHINDIRECT,
    ) -> Result<Self, PlayError> {
        self.context_current
            .object_table
            .push(GraphicsObject::Brush(record.create_brush()));

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn create_font_indirect(
        mut self,
        record_number: usize,
        record: META_CREATEFONTINDIRECT,
    ) -> Result<Self, PlayError> {
        self.context_current
            .object_table
            .push(GraphicsObject::Font(record.font));

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn create_palette(
        mut self,
        record_number: usize,
        record: META_CREATEPALETTE,
    ) -> Result<Self, PlayError> {
        self.context_current
            .object_table
            .push(GraphicsObject::Palette(record.palette));

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn create_pattern_brush(
        mut self,
        record_number: usize,
        record: META_CREATEPATTERNBRUSH,
    ) -> Result<Self, PlayError> {
        self.context_current
            .object_table
            .push(GraphicsObject::Brush(record.create_brush()));

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn create_pen_indirect(
        mut self,
        record_number: usize,
        record: META_CREATEPENINDIRECT,
    ) -> Result<Self, PlayError> {
        self.context_current
            .object_table
            .push(GraphicsObject::Pen(record.pen));

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn create_region(
        mut self,
        record_number: usize,
        record: META_CREATEREGION,
    ) -> Result<Self, PlayError> {
        self.context_current
            .object_table
            .push(GraphicsObject::Region(record.region));

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn delete_object(
        mut self,
        record_number: usize,
        record: META_DELETEOBJECT,
    ) -> Result<Self, PlayError> {
        self.context_current
            .object_table
            .delete(record.object_index as usize);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn create_device_independent_bitmap_pattern_brush(
        mut self,
        record_number: usize,
        record: META_DIBCREATEPATTERNBRUSH,
    ) -> Result<Self, PlayError> {
        self.context_current
            .object_table
            .push(GraphicsObject::Brush(record.create_brush()));

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn select_clip_region(
        mut self,
        record_number: usize,
        record: META_SELECTCLIPREGION,
    ) -> Result<Self, PlayError> {
        let object = self
            .context_current
            .object_table
            .get(record.region as usize);

        if let GraphicsObject::Region(region) = object {
            let rect = &region.bounding_rectangle;
            let id = format!("clip{record_number}");
            let mut clip = Node::new("clipPath").set("id", &id);
            let rect_node = Node::new("rect")
                .set("x", rect.left)
                .set("y", rect.top)
                .set("width", rect.right - rect.left)
                .set("height", rect.bottom - rect.top);
            clip = clip.add(rect_node);
            self.definitions.push(clip);
            self.current_clip_id = Some(id);
        } else {
            self.current_clip_id = None;
        }

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn select_object(
        mut self,
        record_number: usize,
        record: META_SELECTOBJECT,
    ) -> Result<Self, PlayError> {
        let object = self
            .context_current
            .object_table
            .get(record.object_index as usize)
            .clone();
        let selected = self.object_selected.clone();

        self.object_selected = match object {
            GraphicsObject::Brush(v) => selected.brush(v),
            GraphicsObject::Font(v) => selected.font(v),
            GraphicsObject::Palette(v) => selected.palette(v),
            GraphicsObject::Pen(v) => selected.pen(v),
            GraphicsObject::Region(v) => selected.region(v),
            GraphicsObject::Null => {
                return Err(PlayError::UnexpectedGraphicsObject {
                    cause: "Graphics Object is null".to_owned(),
                })
            }
        };

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn select_palette(
        mut self,
        record_number: usize,
        record: META_SELECTPALETTE,
    ) -> Result<Self, PlayError> {
        let object = self
            .context_current
            .object_table
            .get(record.palette as usize);

        let GraphicsObject::Palette(palette) = object else {
            return Err(PlayError::UnexpectedGraphicsObject {
                cause: "Graphics Object is not palette object".to_owned(),
            });
        };

        self.object_selected = self.object_selected.palette(palette.clone());

        Ok(self)
    }

    // .
    // .
    // Functions to handle State Record
    // .
    // .
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn animate_palette(
        self,
        record_number: usize,
        record: META_ANIMATEPALETTE,
    ) -> Result<Self, PlayError> {
        info!("META_ANIMATEPALETTE: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn exclude_clip_rect(
        self,
        record_number: usize,
        record: META_EXCLUDECLIPRECT,
    ) -> Result<Self, PlayError> {
        info!("META_EXCLUDECLIPRECT: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn intersect_clip_rect(
        mut self,
        record_number: usize,
        record: META_INTERSECTCLIPRECT,
    ) -> Result<Self, PlayError> {
        let META_INTERSECTCLIPRECT {
            bottom,
            right,
            top,
            left,
            ..
        } = record;

        self.context_current = self.context_current.clipping_region(Rect {
            left,
            top,
            right,
            bottom,
        });

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn move_to(mut self, record_number: usize, record: META_MOVETO) -> Result<Self, PlayError> {
        let point = self.context_current.point_s_to_absolute_point(&PointS {
            x: record.x,
            y: record.y,
        });
        self.context_current = self
            .context_current
            .extend_window(&point)
            .drawing_position(point);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn offset_clip_region(
        self,
        record_number: usize,
        record: META_OFFSETCLIPRGN,
    ) -> Result<Self, PlayError> {
        info!("META_OFFSETCLIPRGN: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn offset_viewport_origin(
        self,
        record_number: usize,
        record: META_OFFSETVIEWPORTORG,
    ) -> Result<Self, PlayError> {
        info!("META_OFFSETVIEWPORTORG: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn offset_window_origin(
        self,
        record_number: usize,
        record: META_OFFSETWINDOWORG,
    ) -> Result<Self, PlayError> {
        info!("META_OFFSETWINDOWORG: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn realize_palette(
        self,
        record_number: usize,
        record: META_REALIZEPALETTE,
    ) -> Result<Self, PlayError> {
        info!("META_REALIZEPALETTE: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn resize_palette(
        self,
        record_number: usize,
        record: META_RESIZEPALETTE,
    ) -> Result<Self, PlayError> {
        info!("META_RESIZEPALETTE: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn restore_device_context(
        mut self,
        record_number: usize,
        record: META_RESTOREDC,
    ) -> Result<Self, PlayError> {
        let context = if record.n_saved_dc < 0 {
            self.context_current.clone().into()
        } else if (record.n_saved_dc as usize) < self.context_stack.len() {
            self.context_stack.remove(record.n_saved_dc as usize).into()
        } else {
            None
        };

        if let Some(ctx) = context {
            self.context_current = ctx;
        }

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn save_device_context(
        mut self,
        record_number: usize,
        record: META_SAVEDC,
    ) -> Result<Self, PlayError> {
        self.context_stack.push(self.context_current.clone());

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn scale_viewport_ext(
        self,
        record_number: usize,
        record: META_SCALEVIEWPORTEXT,
    ) -> Result<Self, PlayError> {
        info!("META_SCALEVIEWPORTEXT: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn scale_window_ext(
        mut self,
        record_number: usize,
        record: META_SCALEWINDOWEXT,
    ) -> Result<Self, PlayError> {
        let scale_x = (self.context_current.window.scale_x * f32::from(record.x_num))
            / f32::from(record.x_denom);
        let scale_y = (self.context_current.window.scale_y * f32::from(record.y_num))
            / f32::from(record.y_denom);

        self.context_current = self.context_current.window_scale(scale_x, scale_y);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_bk_color(
        mut self,
        record_number: usize,
        record: META_SETBKCOLOR,
    ) -> Result<Self, PlayError> {
        self.context_current = self.context_current.text_bk_color(record.color_ref);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_bk_mode(
        mut self,
        record_number: usize,
        record: META_SETBKMODE,
    ) -> Result<Self, PlayError> {
        self.context_current = self.context_current.bk_mode(record.bk_mode);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_layout(self, record_number: usize, record: META_SETLAYOUT) -> Result<Self, PlayError> {
        info!("META_SETLAYOUT: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_map_mode(
        mut self,
        record_number: usize,
        record: META_SETMAPMODE,
    ) -> Result<Self, PlayError> {
        self.context_current = self.context_current.map_mode(record.map_mode);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_mapper_flags(
        self,
        record_number: usize,
        record: META_SETMAPPERFLAGS,
    ) -> Result<Self, PlayError> {
        info!("META_SETMAPPERFLAGS: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_pal_entries(
        self,
        record_number: usize,
        record: META_SETPALENTRIES,
    ) -> Result<Self, PlayError> {
        info!("META_SETPALENTRIES: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_polyfill_mode(
        mut self,
        record_number: usize,
        record: META_SETPOLYFILLMODE,
    ) -> Result<Self, PlayError> {
        self.context_current = self.context_current.poly_fill_mode(record.poly_fill_mode);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_relabs(self, record_number: usize, record: META_SETRELABS) -> Result<Self, PlayError> {
        info!("META_SETRELABS: reserved record and not supported");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_raster_operation(
        mut self,
        record_number: usize,
        record: META_SETROP2,
    ) -> Result<Self, PlayError> {
        self.context_current = self.context_current.draw_mode(record.draw_mode);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_stretch_blt_mode(
        self,
        record_number: usize,
        record: META_SETSTRETCHBLTMODE,
    ) -> Result<Self, PlayError> {
        info!("META_SETSTRETCHBLTMODE: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_text_align(
        mut self,
        record_number: usize,
        record: META_SETTEXTALIGN,
    ) -> Result<Self, PlayError> {
        let update_cp = record.text_alignment_mode & (TextAlignmentMode::TA_UPDATECP as u16)
            == TextAlignmentMode::TA_UPDATECP as u16;
        let align_horizontal = [TextAlignmentMode::TA_CENTER, TextAlignmentMode::TA_RIGHT]
            .into_iter()
            .find(|a| record.text_alignment_mode & (*a as u16) == *a as u16)
            .unwrap_or(TextAlignmentMode::TA_LEFT);
        // [Task #965 / PR #918 Stage 33-A] SetTextAlign vertical bits 정합.
        // WMF [MS-WMF] 2.1.2.18 TextAlignmentMode 의 vertical 부분:
        //   TA_TOP = 0x0000 (default)
        //   TA_BOTTOM = 0x0008
        //   TA_BASELINE = 0x0018 (TA_BOTTOM | extra bit)
        // 우리 enum 의 VTA_* 매핑:
        //   VTA_TOP = 0x0000, VTA_BOTTOM = 0x0002, VTA_LEFT = 0x0008, VTA_BASELINE = 0x0018.
        // 이전 구현은 `mode & VTA_TOP(=0)` 가 항상 true 라서 BASELINE/BOTTOM 인 mode 도
        // VTA_TOP 으로 매핑되어 baseline 이 cell-top 보정 (+ascent) 만큼 아래로 shift 되는
        // 박스 외부 표시 회귀의 원인. 우선순위: BASELINE → BOTTOM → TOP.
        let v_bits = record.text_alignment_mode & 0x0018;
        let align_vertical = if v_bits == 0x0018 {
            VerticalTextAlignmentMode::VTA_BASELINE
        } else if v_bits == 0x0008 {
            // TA_BOTTOM — 우리 enum 의 가장 가까운 대응은 VTA_LEFT (값 0x0008) 이지만
            // 의미상 BOTTOM 이므로 VTA_BOTTOM 으로 매핑 (ext_text_out 의 baseline
            // 계산 로직에서 동일 처리).
            VerticalTextAlignmentMode::VTA_BOTTOM
        } else {
            VerticalTextAlignmentMode::VTA_TOP
        };

        self.context_current = self
            .context_current
            .text_align_update_cp(update_cp)
            .text_align_horizontal(align_horizontal)
            .text_align_vertical(align_vertical);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_text_char_extra(
        self,
        record_number: usize,
        record: META_SETTEXTCHAREXTRA,
    ) -> Result<Self, PlayError> {
        info!("META_SETTEXTCHAREXTRA: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_text_color(
        mut self,
        record_number: usize,
        record: META_SETTEXTCOLOR,
    ) -> Result<Self, PlayError> {
        self.context_current = self.context_current.text_color(record.color_ref);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_text_justification(
        self,
        record_number: usize,
        record: META_SETTEXTJUSTIFICATION,
    ) -> Result<Self, PlayError> {
        info!("META_SETTEXTJUSTIFICATION: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_viewport_ext(
        self,
        record_number: usize,
        record: META_SETVIEWPORTEXT,
    ) -> Result<Self, PlayError> {
        info!("META_SETVIEWPORTEXT: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_viewport_origin(
        self,
        record_number: usize,
        record: META_SETVIEWPORTORG,
    ) -> Result<Self, PlayError> {
        info!("META_SETVIEWPORTORG: not implemented");
        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_window_ext(
        mut self,
        record_number: usize,
        record: META_SETWINDOWEXT,
    ) -> Result<Self, PlayError> {
        self.context_current = self.context_current.window_ext(record.x, record.y);

        Ok(self)
    }

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn set_window_origin(
        mut self,
        record_number: usize,
        record: META_SETWINDOWORG,
    ) -> Result<Self, PlayError> {
        self.context_current = self.context_current.window_origin(record.x, record.y);

        Ok(self)
    }

    // .
    // .
    // Functions to handle Escape Record
    // .
    // .

    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip(self),
        err(level = tracing::Level::ERROR, Display),
    ))]
    fn escape(self, record_number: usize, record: META_ESCAPE) -> Result<Self, PlayError> {
        Ok(self)
    }
}

/// [Task #860] Node 의 x+width attribute 합산 (viewBox 자동 확장용).
/// `<image>`, `<rect>`, `<polygon>` 등의 좌표 attribute 를 parse 하여
/// 최대 x+width 위치 반환. attribute 미존재 시 None.
fn element_max_x(elem: &Node) -> Option<i32> {
    let s = elem.to_string();
    let x = parse_attr_i32(&s, "x").unwrap_or(0);
    let w = parse_attr_i32(&s, "width").unwrap_or(0);
    if w > 0 {
        Some(x + w)
    } else {
        None
    }
}

/// [Task #860] Node 의 y+height attribute 합산 (viewBox 자동 확장용).
/// [Task #864] text 는 height 미보유 → font-size 사용.
fn element_max_y(elem: &Node) -> Option<i32> {
    let s = elem.to_string();
    let y = parse_attr_i32(&s, "y").unwrap_or(0);
    let h = parse_attr_i32(&s, "height").unwrap_or(0);
    if h > 0 {
        return Some(y + h);
    }
    // text element: y + font-size (approx)
    if s.starts_with("<text ") {
        if let Some(fs) = parse_attr_i32(&s, "font-size") {
            return Some(y + fs);
        }
    }
    None
}

/// [Task #860 Stage D] Node 의 x attribute (viewBox 위쪽 확장 용).
fn element_min_x(elem: &Node) -> Option<i32> {
    let s = elem.to_string();
    parse_attr_i32(&s, "x")
}

/// [Task #860 Stage D] Node 의 y attribute (viewBox 위쪽 확장 용).
fn element_min_y(elem: &Node) -> Option<i32> {
    let s = elem.to_string();
    parse_attr_i32(&s, "y")
}

/// 단순 SVG attribute 값 parse (i32). 미존재 또는 parse 실패 시 None.
fn parse_attr_i32(s: &str, attr: &str) -> Option<i32> {
    let needle = format!(" {attr}=\"");
    let start = s.find(&needle)?;
    let val_start = start + needle.len();
    let val_end = s[val_start..].find('"')?;
    s[val_start..val_start + val_end].parse().ok()
}
