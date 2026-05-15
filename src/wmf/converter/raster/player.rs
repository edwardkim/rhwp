//! [Task #902 v2 Stage 12~13] WMF raster Player.
//!
//! 알고리즘 출처: LibreOffice emfio (MPL 2.0)
//!   wmfreader.cxx + mtftools.cxx 의 WMF record 처리 로직 포팅.

use super::state::*;
use crate::wmf::converter::{PlayError, Player};
use crate::wmf::parser::*;

use tiny_skia::{
    Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform,
};

/// WMF records 를 tiny-skia Pixmap 에 직접 렌더링하는 Player.
///
/// 출력: PNG bytes (`generate()` 호출 시).
pub struct RasterPlayer {
    state: RasterState,
    /// DC stack (META_SAVEDC / META_RESTOREDC)
    state_stack: Vec<RasterState>,
    /// 출력 canvas 크기 (device pixels).
    canvas_width: u32,
    canvas_height: u32,
    /// 누적 logical 좌표 bbox — header 의 bound 후 SetWindowExt 등에 갱신.
    extent: (i16, i16),
    /// 실제 raster canvas (native only).
    pixmap: Pixmap,
}

impl RasterPlayer {
    pub fn new(canvas_width: u32, canvas_height: u32) -> Option<Self> {
        let mut pixmap = Pixmap::new(canvas_width.max(1), canvas_height.max(1))?;
        pixmap.fill(Color::WHITE);
        Some(Self {
            state: RasterState::default(),
            state_stack: Vec::new(),
            canvas_width,
            canvas_height,
            extent: (1024, 1024),
            pixmap,
        })
    }

    pub fn default_canvas() -> Option<Self> {
        Self::new(1024, 1024)
    }

    /// Logical 좌표 → device pixel 변환 (canvas 크기 반영).
    fn logical_to_pixel(&self, x: i16, y: i16) -> (f32, f32) {
        let (dx, dy) = self.state.logical_to_device(x, y);
        // device 좌표를 canvas 크기로 정규화
        let scale_x = self.canvas_width as f32 / f32::from(self.extent.0.max(1));
        let scale_y = self.canvas_height as f32 / f32::from(self.extent.1.max(1));
        (dx * scale_x, dy * scale_y)
    }

    /// [Stage 23] DIB 를 pixmap 에 blit (DIBSTRETCHBLT/DIBBITBLT/STRETCHDIB 공통).
    fn blit_dib(
        &mut self,
        dib: crate::wmf::parser::DeviceIndependentBitmap,
        x_dest: i16,
        y_dest: i16,
        dest_w: i16,
        dest_h: i16,
    ) {
        use crate::wmf::converter::Bitmap;
        let bmp = Bitmap::from(dib).to_vec();
        let Ok(img) = image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp) else { return };
        let (dx0, dy0) = self.logical_to_pixel(x_dest, y_dest);
        let (dx1, dy1) = self.logical_to_pixel(
            x_dest.saturating_add(dest_w),
            y_dest.saturating_add(dest_h),
        );
        let target_w = (dx1 - dx0).abs().ceil() as u32;
        let target_h = (dy1 - dy0).abs().ceil() as u32;
        if target_w == 0 || target_h == 0 { return; }

        let resized = img.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let x0 = dx0.floor() as i32;
        let y0 = dy0.floor() as i32;
        let pw = self.pixmap.width() as i32;
        let ph = self.pixmap.height() as i32;
        let pixels = self.pixmap.pixels_mut();
        for sy in 0..target_h as i32 {
            let py = y0 + sy;
            if py < 0 || py >= ph { continue; }
            for sx in 0..target_w as i32 {
                let px = x0 + sx;
                if px < 0 || px >= pw { continue; }
                let p = rgba.get_pixel(sx as u32, sy as u32);
                if let Some(c) = tiny_skia::PremultipliedColorU8::from_rgba(p[0], p[1], p[2], 255) {
                    let idx = (py * pw + px) as usize;
                    pixels[idx] = c;
                }
            }
        }
    }

    /// Selected pen 기반 stroke 생성.
    fn build_stroke_paint(&self) -> Option<(Paint<'static>, Stroke)> {
        let pen_idx = self.state.selected_pen?;
        let obj = self.state.object_table.get(&pen_idx)?;
        let RasterObject::Pen(pen) = obj else { return None };
        if pen.is_null {
            return None;
        }
        let mut paint = Paint::default();
        paint.set_color_rgba8(
            pen.color.red,
            pen.color.green,
            pen.color.blue,
            255,
        );
        paint.anti_alias = true;
        let mut stroke = Stroke::default();
        stroke.width = (pen.width.max(1) as f32)
            * (self.canvas_width as f32 / f32::from(self.extent.0.max(1)));
        Some((paint, stroke))
    }

    /// Selected brush 기반 fill paint 생성.
    fn build_fill_paint(&self) -> Option<Paint<'static>> {
        let brush_idx = self.state.selected_brush?;
        let obj = self.state.object_table.get(&brush_idx)?;
        let RasterObject::Brush(brush) = obj else { return None };
        if brush.is_null {
            return None;
        }
        let mut paint = Paint::default();
        paint.set_color_rgba8(
            brush.color.red,
            brush.color.green,
            brush.color.blue,
            255,
        );
        paint.anti_alias = true;
        Some(paint)
    }
}

impl Player for RasterPlayer {
    fn generate(self) -> Result<Vec<u8>, PlayError> {
        self.pixmap
            .encode_png()
            .map_err(|err| PlayError::InvalidRecord {
                cause: format!("PNG encode 실패: {err}"),
            })
    }

    // === Header / EOF ===

    fn header(
        mut self,
        _record_number: usize,
        record: MetafileHeader,
    ) -> Result<Self, PlayError> {
        // Placeable WMF 의 bound (window size 추정) — emfio 의 ReadHeader.
        if let MetafileHeader::StartsWithPlaceable(placeable, _header) = &record {
            let w = (placeable.bounding_box.right - placeable.bounding_box.left).abs();
            let h = (placeable.bounding_box.bottom - placeable.bounding_box.top).abs();
            self.extent = (w.max(1), h.max(1));
            self.state.window_ext = self.extent;
        }
        Ok(self)
    }

    fn eof(self, _record_number: usize, _record: META_EOF) -> Result<Self, PlayError> {
        Ok(self)
    }

    // === Bitmap records ===
    fn bit_blt(self, _: usize, _: META_BITBLT) -> Result<Self, PlayError> { Ok(self) }

    fn device_independent_bitmap_bit_blt(
        mut self,
        _: usize,
        record: META_DIBBITBLT,
    ) -> Result<Self, PlayError> {
        // [Stage 23] DIBBITBLT — DIBSTRETCHBLT 와 동일 구조이나 source=dest 크기 동일.
        let (x_dest, y_dest, w, h, target) = match record {
            META_DIBBITBLT::WithBitmap {
                x_dest, y_dest, width, height, target, ..
            } => (x_dest, y_dest, width, height, Some(*target)),
            META_DIBBITBLT::WithoutBitmap { .. } => return Ok(self),
        };
        let Some(dib) = target else { return Ok(self) };
        self.blit_dib(dib, x_dest, y_dest, w, h);
        Ok(self)
    }

    fn device_independent_bitmap_stretch_blt(
        mut self,
        _: usize,
        record: META_DIBSTRETCHBLT,
    ) -> Result<Self, PlayError> {
        // [Task #902 v2 Stage 18] LO mtftools.cxx 의 DIBStretchBlt 포팅
        // DIB → BMP bytes (Bitmap::from) → image crate decode → pixmap blit
        use crate::wmf::converter::Bitmap;
        let (x_dest, y_dest, dest_w, dest_h, target) = match record {
            META_DIBSTRETCHBLT::WithBitmap {
                x_dest,
                y_dest,
                dest_width,
                dest_height,
                target,
                ..
            } => (x_dest, y_dest, dest_width, dest_height, Some(*target)),
            META_DIBSTRETCHBLT::WithoutBitmap { .. } => return Ok(self),
        };
        let Some(dib) = target else { return Ok(self) };

        let bmp = Bitmap::from(dib).to_vec();
        let img = match image::load_from_memory_with_format(&bmp, image::ImageFormat::Bmp) {
            Ok(im) => im,
            Err(_) => return Ok(self),
        };

        let (dx0, dy0) = self.logical_to_pixel(x_dest, y_dest);
        let (dx1, dy1) = self.logical_to_pixel(
            x_dest.saturating_add(dest_w),
            y_dest.saturating_add(dest_h),
        );
        let target_w = (dx1 - dx0).abs().ceil() as u32;
        let target_h = (dy1 - dy0).abs().ceil() as u32;
        if target_w == 0 || target_h == 0 { return Ok(self); }

        let resized = img.resize_exact(
            target_w,
            target_h,
            image::imageops::FilterType::Lanczos3,
        );
        let rgba = resized.to_rgba8();

        let x0 = dx0.floor() as i32;
        let y0 = dy0.floor() as i32;
        let pw = self.pixmap.width() as i32;
        let ph = self.pixmap.height() as i32;
        let pixels = self.pixmap.pixels_mut();

        for sy in 0..target_h as i32 {
            let py = y0 + sy;
            if py < 0 || py >= ph { continue; }
            for sx in 0..target_w as i32 {
                let px = x0 + sx;
                if px < 0 || px >= pw { continue; }
                let p = rgba.get_pixel(sx as u32, sy as u32);
                if let Some(c) = tiny_skia::PremultipliedColorU8::from_rgba(
                    p[0], p[1], p[2], 255,
                ) {
                    let idx = (py * pw + px) as usize;
                    pixels[idx] = c;
                }
            }
        }
        Ok(self)
    }

    fn set_device_independent_bitmap_to_dev(self, _: usize, _: META_SETDIBTODEV) -> Result<Self, PlayError> { Ok(self) }
    fn stretch_blt(self, _: usize, _: META_STRETCHBLT) -> Result<Self, PlayError> { Ok(self) }

    fn stretch_device_independent_bitmap(
        mut self,
        _: usize,
        record: META_STRETCHDIB,
    ) -> Result<Self, PlayError> {
        // [Stage 23] STRETCHDIB — DIBSTRETCHBLT 와 동일 구조이나 source rect 가
        // dest rect 와 다를 수 있음 (stretch).
        self.blit_dib(record.dib, record.x_dst, record.y_dst, record.dest_width, record.dest_height);
        Ok(self)
    }

    // === Drawing records ===
    fn arc(mut self, _: usize, record: META_ARC) -> Result<Self, PlayError> {
        // [Task #902 v2 Stage 20] LO mtftools.cxx 의 DrawArc 포팅
        // bounding rect 의 ellipse 중심 + start/end 각도로 호 그리기
        let (x0, y0) = self.logical_to_pixel(record.left_rect, record.top_rect);
        let (x1, y1) = self.logical_to_pixel(record.right_rect, record.bottom_rect);
        let (sx, sy) = self.logical_to_pixel(record.x_start_arc, record.y_start_arc);
        let (ex, ey) = self.logical_to_pixel(record.x_end_arc, record.y_end_arc);
        let _ = (sx, sy, ex, ey);
        if let Some(path) = arc_path(x0, y0, x1, y1, sx, sy, ex, ey, false) {
            if let Some((sp, s)) = self.build_stroke_paint() {
                self.pixmap.stroke_path(&path, &sp, &s, Transform::identity(), None);
            }
        }
        Ok(self)
    }

    fn chord(mut self, _: usize, record: META_CHORD) -> Result<Self, PlayError> {
        // chord = arc + start/end 연결 직선 (closed path) → fill + stroke
        let (x0, y0) = self.logical_to_pixel(record.left_rect, record.top_rect);
        let (x1, y1) = self.logical_to_pixel(record.right_rect, record.bottom_rect);
        let (sx, sy) = self.logical_to_pixel(record.x_radial1, record.y_radial1);
        let (ex, ey) = self.logical_to_pixel(record.x_radial2, record.y_radial2);
        if let Some(path) = arc_path(x0, y0, x1, y1, sx, sy, ex, ey, true) {
            if let Some(fp) = self.build_fill_paint() {
                self.pixmap.fill_path(&path, &fp, FillRule::Winding, Transform::identity(), None);
            }
            if let Some((sp, s)) = self.build_stroke_paint() {
                self.pixmap.stroke_path(&path, &sp, &s, Transform::identity(), None);
            }
        }
        Ok(self)
    }

    fn ellipse(mut self, _: usize, record: META_ELLIPSE) -> Result<Self, PlayError> {
        // LO mtftools.cxx 의 DrawEllipse 포팅 — bbox 의 ellipse fill + stroke
        let (x0, y0) = self.logical_to_pixel(record.left_rect, record.top_rect);
        let (x1, y1) = self.logical_to_pixel(record.right_rect, record.bottom_rect);
        let cx = (x0 + x1) / 2.0;
        let cy = (y0 + y1) / 2.0;
        let rx = ((x1 - x0).abs()) / 2.0;
        let ry = ((y1 - y0).abs()) / 2.0;
        if rx < 0.5 || ry < 0.5 { return Ok(self); }

        // tiny-skia 의 PathBuilder 는 cubic bezier 만 지원 → 4 개 bezier 로 ellipse 근사
        // 마법 상수 c = 0.5522847498 (4/3 * (sqrt(2) - 1))
        const KAPPA: f32 = 0.552_284_8;
        let ox = rx * KAPPA;
        let oy = ry * KAPPA;
        let mut pb = PathBuilder::new();
        pb.move_to(cx - rx, cy);
        pb.cubic_to(cx - rx, cy - oy,   cx - ox, cy - ry,   cx,      cy - ry);
        pb.cubic_to(cx + ox, cy - ry,   cx + rx, cy - oy,   cx + rx, cy);
        pb.cubic_to(cx + rx, cy + oy,   cx + ox, cy + ry,   cx,      cy + ry);
        pb.cubic_to(cx - ox, cy + ry,   cx - rx, cy + oy,   cx - rx, cy);
        pb.close();
        if let Some(path) = pb.finish() {
            if let Some(fill_paint) = self.build_fill_paint() {
                self.pixmap.fill_path(&path, &fill_paint, FillRule::Winding, Transform::identity(), None);
            }
            if let Some((stroke_paint, stroke)) = self.build_stroke_paint() {
                self.pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        Ok(self)
    }
    fn ext_flood_fill(self, _: usize, _: META_EXTFLOODFILL) -> Result<Self, PlayError> { Ok(self) }
    fn ext_text_out(
        mut self,
        _: usize,
        record: META_EXTTEXTOUT,
    ) -> Result<Self, PlayError> {
        // LO mtftools.cxx 의 DrawText 포팅 — DX byte-aware 합산 + glyph 렌더
        let Some(font_idx) = self.state.selected_font else { return Ok(self) };
        let Some(obj) = self.state.object_table.get(&font_idx).cloned() else {
            return Ok(self);
        };
        let RasterObject::Font(font_info) = obj else {
            return Ok(self);
        };

        // record 의 byte 배열 → UTF-8 변환 (charset 기반)
        let text = match record.into_utf8(font_info.charset) {
            Ok(t) => t,
            Err(_) => return Ok(self),
        };
        if text.is_empty() { return Ok(self); }

        let (origin_px_x, origin_px_y) = self.logical_to_pixel(record.x, record.y);

        // logical → pixel scale (font_size_logical 변환)
        let scale_x = self.canvas_width as f32 / f32::from(self.extent.0.max(1));
        let scale_y = self.canvas_height as f32 / f32::from(self.extent.1.max(1));

        // font.height: WMF 의 lfHeight 는 음수 = cell height 의 절대값, 양수 = em height
        let font_size_logical = f32::from(font_info.height.abs());

        let text_color = self.state.text_color.clone();

        super::text::draw_text_with_dx(
            &mut self.pixmap,
            &text,
            &record.dx,
            origin_px_x,
            origin_px_y,
            font_size_logical,
            scale_x,
            scale_y,
            &text_color,
            font_info.weight,
            font_info.italic,
        );

        Ok(self)
    }
    fn fill_region(self, _: usize, _: META_FILLREGION) -> Result<Self, PlayError> { Ok(self) }
    fn flood_fill(self, _: usize, _: META_FLOODFILL) -> Result<Self, PlayError> { Ok(self) }
    fn frame_region(self, _: usize, _: META_FRAMEREGION) -> Result<Self, PlayError> { Ok(self) }
    fn invert_region(self, _: usize, _: META_INVERTREGION) -> Result<Self, PlayError> { Ok(self) }

    fn line_to(mut self, _: usize, record: META_LINETO) -> Result<Self, PlayError> {
        // LO mtftools.cxx 의 DrawLineTo 포팅
        let (x0, y0) = self.logical_to_pixel(
            self.state.current_position.0,
            self.state.current_position.1,
        );
        let (x1, y1) = self.logical_to_pixel(record.x, record.y);
        if let Some((paint, stroke)) = self.build_stroke_paint() {
            let mut pb = PathBuilder::new();
            pb.move_to(x0, y0);
            pb.line_to(x1, y1);
            if let Some(path) = pb.finish() {
                self.pixmap.stroke_path(
                    &path,
                    &paint,
                    &stroke,
                    Transform::identity(),
                    None,
                );
            }
        }
        self.state.current_position = (record.x, record.y);
        Ok(self)
    }
    fn paint_region(self, _: usize, _: META_PAINTREGION) -> Result<Self, PlayError> { Ok(self) }
    fn pat_blt(self, _: usize, _: META_PATBLT) -> Result<Self, PlayError> { Ok(self) }
    fn pie(mut self, _: usize, record: META_PIE) -> Result<Self, PlayError> {
        // pie = arc + center 로 연결된 wedge (closed path) → fill + stroke
        let (x0, y0) = self.logical_to_pixel(record.left_rect, record.top_rect);
        let (x1, y1) = self.logical_to_pixel(record.right_rect, record.bottom_rect);
        let (sx, sy) = self.logical_to_pixel(record.x_radial1, record.y_radial1);
        let (ex, ey) = self.logical_to_pixel(record.x_radial2, record.y_radial2);
        let cx = (x0 + x1) / 2.0;
        let cy = (y0 + y1) / 2.0;
        if let Some(path) = arc_path_pie(x0, y0, x1, y1, sx, sy, ex, ey, cx, cy) {
            if let Some(fp) = self.build_fill_paint() {
                self.pixmap.fill_path(&path, &fp, FillRule::Winding, Transform::identity(), None);
            }
            if let Some((sp, s)) = self.build_stroke_paint() {
                self.pixmap.stroke_path(&path, &sp, &s, Transform::identity(), None);
            }
        }
        Ok(self)
    }

    fn polyline(mut self, _: usize, record: META_POLYLINE) -> Result<Self, PlayError> {
        // LO mtftools.cxx 의 DrawPolyLine 포팅
        if record.a_points.is_empty() {
            return Ok(self);
        }
        if let Some((paint, stroke)) = self.build_stroke_paint() {
            let mut pb = PathBuilder::new();
            for (i, p) in record.a_points.iter().enumerate() {
                let (px, py) = self.logical_to_pixel(p.x, p.y);
                if i == 0 { pb.move_to(px, py); } else { pb.line_to(px, py); }
            }
            if let Some(path) = pb.finish() {
                self.pixmap.stroke_path(
                    &path,
                    &paint,
                    &stroke,
                    Transform::identity(),
                    None,
                );
            }
        }
        Ok(self)
    }

    fn polygon(mut self, _: usize, record: META_POLYGON) -> Result<Self, PlayError> {
        // LO mtftools.cxx 의 DrawPolygon 포팅 — fill + stroke
        if record.a_points.is_empty() {
            return Ok(self);
        }
        let mut pb = PathBuilder::new();
        for (i, p) in record.a_points.iter().enumerate() {
            let (px, py) = self.logical_to_pixel(p.x, p.y);
            if i == 0 { pb.move_to(px, py); } else { pb.line_to(px, py); }
        }
        pb.close();
        let fill_rule = match self.state.poly_fill_mode {
            PolyFillMode::ALTERNATE => FillRule::EvenOdd,
            PolyFillMode::WINDING => FillRule::Winding,
        };
        if let Some(path) = pb.finish() {
            if let Some(fill_paint) = self.build_fill_paint() {
                self.pixmap.fill_path(
                    &path,
                    &fill_paint,
                    fill_rule,
                    Transform::identity(),
                    None,
                );
            }
            if let Some((stroke_paint, stroke)) = self.build_stroke_paint() {
                self.pixmap.stroke_path(
                    &path,
                    &stroke_paint,
                    &stroke,
                    Transform::identity(),
                    None,
                );
            }
        }
        Ok(self)
    }

    fn poly_polygon(mut self, _: usize, record: META_POLYPOLYGON) -> Result<Self, PlayError> {
        // LO mtftools.cxx 의 DrawPolyPolygon 포팅
        // 단일 path 의 다중 서브경로로 합성 — fill-rule (winding/alternate) 적용.
        let mut pb = PathBuilder::new();
        let mut a_point_iter = record.poly_polygon.a_points.iter();
        for i in 0..record.poly_polygon.number_of_polygons {
            let Some(&count) = record.poly_polygon.a_points_per_polygon.get(i as usize) else {
                continue;
            };
            for j in 0..count {
                let Some(p) = a_point_iter.next() else { break };
                let (px, py) = self.logical_to_pixel(p.x, p.y);
                if j == 0 { pb.move_to(px, py); } else { pb.line_to(px, py); }
            }
            pb.close();
        }
        let fill_rule = match self.state.poly_fill_mode {
            PolyFillMode::ALTERNATE => FillRule::EvenOdd,
            PolyFillMode::WINDING => FillRule::Winding,
        };
        if let Some(path) = pb.finish() {
            if let Some(fill_paint) = self.build_fill_paint() {
                self.pixmap.fill_path(
                    &path,
                    &fill_paint,
                    fill_rule,
                    Transform::identity(),
                    None,
                );
            }
            if let Some((stroke_paint, stroke)) = self.build_stroke_paint() {
                self.pixmap.stroke_path(
                    &path,
                    &stroke_paint,
                    &stroke,
                    Transform::identity(),
                    None,
                );
            }
        }
        Ok(self)
    }

    fn rectangle(mut self, _: usize, record: META_RECTANGLE) -> Result<Self, PlayError> {
        // LO mtftools.cxx 의 DrawRect 포팅
        let (x0, y0) = self.logical_to_pixel(record.left_rect, record.top_rect);
        let (x1, y1) = self.logical_to_pixel(record.right_rect, record.bottom_rect);
        let mut pb = PathBuilder::new();
        pb.move_to(x0, y0);
        pb.line_to(x1, y0);
        pb.line_to(x1, y1);
        pb.line_to(x0, y1);
        pb.close();
        if let Some(path) = pb.finish() {
            if let Some(fill_paint) = self.build_fill_paint() {
                self.pixmap.fill_path(
                    &path,
                    &fill_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
            if let Some((stroke_paint, stroke)) = self.build_stroke_paint() {
                self.pixmap.stroke_path(
                    &path,
                    &stroke_paint,
                    &stroke,
                    Transform::identity(),
                    None,
                );
            }
        }
        Ok(self)
    }

    fn round_rect(mut self, _: usize, record: META_ROUNDRECT) -> Result<Self, PlayError> {
        // 단순화: 라운드 코너 무시하고 rectangle 로 처리 (Stage 16+ 정밀화)
        let (x0, y0) = self.logical_to_pixel(record.left_rect, record.top_rect);
        let (x1, y1) = self.logical_to_pixel(record.right_rect, record.bottom_rect);
        let mut pb = PathBuilder::new();
        pb.move_to(x0, y0);
        pb.line_to(x1, y0);
        pb.line_to(x1, y1);
        pb.line_to(x0, y1);
        pb.close();
        if let Some(path) = pb.finish() {
            if let Some(fp) = self.build_fill_paint() {
                self.pixmap.fill_path(&path, &fp, FillRule::Winding, Transform::identity(), None);
            }
            if let Some((sp, s)) = self.build_stroke_paint() {
                self.pixmap.stroke_path(&path, &sp, &s, Transform::identity(), None);
            }
        }
        Ok(self)
    }
    fn set_pixel(self, _: usize, _: META_SETPIXEL) -> Result<Self, PlayError> { Ok(self) }
    fn text_out(self, _: usize, _: META_TEXTOUT) -> Result<Self, PlayError> { Ok(self) }

    // === Object records ===
    fn create_brush_indirect(
        mut self,
        _: usize,
        record: META_CREATEBRUSHINDIRECT,
    ) -> Result<Self, PlayError> {
        use crate::wmf::parser::LogBrush;
        let idx = lowest_free_slot(&self.state.object_table);
        let (color, is_null) = match &record.log_brush {
            LogBrush::Solid { color_ref } => (color_ref.clone(), false),
            LogBrush::Hatched { color_ref, .. } => (color_ref.clone(), false),
            LogBrush::Null => (ColorRef::white(), true),
            _ => (ColorRef::white(), false),
        };
        self.state.object_table.insert(
            idx,
            RasterObject::Brush(BrushInfo { color, style: 0, is_null }),
        );
        Ok(self)
    }

    fn create_pen_indirect(
        mut self,
        _: usize,
        record: META_CREATEPENINDIRECT,
    ) -> Result<Self, PlayError> {
        use crate::wmf::parser::PenStyle;
        let idx = lowest_free_slot(&self.state.object_table);
        let is_null = matches!(record.pen.style.style, PenStyle::PS_NULL);
        self.state.object_table.insert(
            idx,
            RasterObject::Pen(PenInfo {
                color: record.pen.color_ref.clone(),
                width: i32::from(record.pen.width.x.max(1)),
                style: 0,
                is_null,
            }),
        );
        Ok(self)
    }

    fn create_font_indirect(
        mut self,
        _: usize,
        record: META_CREATEFONTINDIRECT,
    ) -> Result<Self, PlayError> {
        let idx = lowest_free_slot(&self.state.object_table);
        self.state.object_table.insert(
            idx,
            RasterObject::Font(FontInfo {
                height: record.font.height,
                width: record.font.width,
                escapement: record.font.escapement,
                orientation: record.font.orientation,
                weight: record.font.weight,
                italic: record.font.italic,
                underline: record.font.underline,
                strike_out: record.font.strike_out,
                charset: record.font.charset,
                facename: record.font.facename.clone(),
            }),
        );
        Ok(self)
    }

    fn create_palette(self, _: usize, _: META_CREATEPALETTE) -> Result<Self, PlayError> { Ok(self) }
    fn create_pattern_brush(self, _: usize, _: META_CREATEPATTERNBRUSH) -> Result<Self, PlayError> { Ok(self) }

    fn create_region(mut self, _: usize, _record: META_CREATEREGION) -> Result<Self, PlayError> {
        let idx = lowest_free_slot(&self.state.object_table);
        self.state.object_table.insert(idx, RasterObject::Region);
        Ok(self)
    }

    fn delete_object(mut self, _: usize, record: META_DELETEOBJECT) -> Result<Self, PlayError> {
        self.state.object_table.remove(&record.object_index);
        if self.state.selected_pen == Some(record.object_index) { self.state.selected_pen = None; }
        if self.state.selected_brush == Some(record.object_index) { self.state.selected_brush = None; }
        if self.state.selected_font == Some(record.object_index) { self.state.selected_font = None; }
        Ok(self)
    }

    fn create_device_independent_bitmap_pattern_brush(self, _: usize, _: META_DIBCREATEPATTERNBRUSH) -> Result<Self, PlayError> { Ok(self) }
    fn select_clip_region(self, _: usize, _: META_SELECTCLIPREGION) -> Result<Self, PlayError> { Ok(self) }

    fn select_object(mut self, _: usize, record: META_SELECTOBJECT) -> Result<Self, PlayError> {
        if let Some(obj) = self.state.object_table.get(&record.object_index) {
            match obj {
                RasterObject::Pen(_) => self.state.selected_pen = Some(record.object_index),
                RasterObject::Brush(_) => self.state.selected_brush = Some(record.object_index),
                RasterObject::Font(_) => self.state.selected_font = Some(record.object_index),
                _ => {}
            }
        }
        Ok(self)
    }

    fn select_palette(self, _: usize, _: META_SELECTPALETTE) -> Result<Self, PlayError> { Ok(self) }

    // === State records ===
    fn animate_palette(self, _: usize, _: META_ANIMATEPALETTE) -> Result<Self, PlayError> { Ok(self) }
    fn exclude_clip_rect(self, _: usize, _: META_EXCLUDECLIPRECT) -> Result<Self, PlayError> {
        // [Stage 21] ExcludeClipRect — 단순화: 현재 clip 유지 (region 차집합 미구현)
        // 향후 region 처리 follow-up
        Ok(self)
    }

    fn intersect_clip_rect(
        mut self,
        _: usize,
        record: META_INTERSECTCLIPRECT,
    ) -> Result<Self, PlayError> {
        // [Stage 21] LO mtftools.cxx 의 IntersectClipRect 포팅 —
        // 현재 clip 과 새 rect 의 교집합 계산.
        let (x0, y0) = self.logical_to_pixel(record.left, record.top);
        let (x1, y1) = self.logical_to_pixel(record.right, record.bottom);
        let new_clip = (
            x0.floor() as i32,
            y0.floor() as i32,
            x1.ceil() as i32,
            y1.ceil() as i32,
        );
        self.state.clip_rect = Some(match self.state.clip_rect {
            None => new_clip,
            Some((cx0, cy0, cx1, cy1)) => (
                cx0.max(new_clip.0),
                cy0.max(new_clip.1),
                cx1.min(new_clip.2),
                cy1.min(new_clip.3),
            ),
        });
        Ok(self)
    }
    fn move_to(mut self, _: usize, record: META_MOVETO) -> Result<Self, PlayError> {
        self.state.current_position = (record.x, record.y);
        Ok(self)
    }
    fn offset_clip_region(self, _: usize, _: META_OFFSETCLIPRGN) -> Result<Self, PlayError> { Ok(self) }
    fn offset_viewport_origin(mut self, _: usize, record: META_OFFSETVIEWPORTORG) -> Result<Self, PlayError> {
        self.state.viewport_origin.0 = self.state.viewport_origin.0.saturating_add(record.x_offset);
        self.state.viewport_origin.1 = self.state.viewport_origin.1.saturating_add(record.y_offset);
        Ok(self)
    }
    fn offset_window_origin(mut self, _: usize, record: META_OFFSETWINDOWORG) -> Result<Self, PlayError> {
        self.state.window_origin.0 = self.state.window_origin.0.saturating_add(record.x_offset);
        self.state.window_origin.1 = self.state.window_origin.1.saturating_add(record.y_offset);
        Ok(self)
    }
    fn realize_palette(self, _: usize, _: META_REALIZEPALETTE) -> Result<Self, PlayError> { Ok(self) }
    fn resize_palette(self, _: usize, _: META_RESIZEPALETTE) -> Result<Self, PlayError> { Ok(self) }
    fn restore_device_context(mut self, _: usize, record: META_RESTOREDC) -> Result<Self, PlayError> {
        if record.n_saved_dc < 0 {
            for _ in 0..(-record.n_saved_dc as usize).min(self.state_stack.len()) {
                if let Some(s) = self.state_stack.pop() { self.state = s; }
            }
        } else {
            let idx = record.n_saved_dc as usize;
            if idx < self.state_stack.len() {
                self.state = self.state_stack.remove(idx);
            }
        }
        Ok(self)
    }
    fn save_device_context(mut self, _: usize, _: META_SAVEDC) -> Result<Self, PlayError> {
        self.state_stack.push(self.state.clone());
        Ok(self)
    }
    fn scale_viewport_ext(mut self, _: usize, record: META_SCALEVIEWPORTEXT) -> Result<Self, PlayError> {
        let nx = (i32::from(self.state.viewport_ext.0) * i32::from(record.x_num)
            / i32::from(record.x_denom.max(1))) as i16;
        let ny = (i32::from(self.state.viewport_ext.1) * i32::from(record.y_num)
            / i32::from(record.y_denom.max(1))) as i16;
        self.state.viewport_ext = (nx, ny);
        self.state.viewport_ext_set = true;
        Ok(self)
    }
    fn scale_window_ext(mut self, _: usize, record: META_SCALEWINDOWEXT) -> Result<Self, PlayError> {
        let nx = (i32::from(self.state.window_ext.0) * i32::from(record.x_num)
            / i32::from(record.x_denom.max(1))) as i16;
        let ny = (i32::from(self.state.window_ext.1) * i32::from(record.y_num)
            / i32::from(record.y_denom.max(1))) as i16;
        self.state.window_ext = (nx, ny);
        Ok(self)
    }
    fn set_bk_color(mut self, _: usize, record: META_SETBKCOLOR) -> Result<Self, PlayError> {
        self.state.bk_color = record.color_ref;
        Ok(self)
    }
    fn set_bk_mode(mut self, _: usize, record: META_SETBKMODE) -> Result<Self, PlayError> {
        self.state.bk_mode = record.bk_mode;
        Ok(self)
    }
    fn set_layout(self, _: usize, _: META_SETLAYOUT) -> Result<Self, PlayError> { Ok(self) }
    fn set_map_mode(mut self, _: usize, record: META_SETMAPMODE) -> Result<Self, PlayError> {
        self.state.map_mode = record.map_mode;
        Ok(self)
    }
    fn set_mapper_flags(self, _: usize, _: META_SETMAPPERFLAGS) -> Result<Self, PlayError> { Ok(self) }
    fn set_pal_entries(self, _: usize, _: META_SETPALENTRIES) -> Result<Self, PlayError> { Ok(self) }
    fn set_polyfill_mode(mut self, _: usize, record: META_SETPOLYFILLMODE) -> Result<Self, PlayError> {
        self.state.poly_fill_mode = record.poly_fill_mode;
        Ok(self)
    }
    fn set_relabs(self, _: usize, _: META_SETRELABS) -> Result<Self, PlayError> { Ok(self) }
    fn set_raster_operation(mut self, _: usize, record: META_SETROP2) -> Result<Self, PlayError> {
        self.state.rop2 = Some(record.draw_mode);
        Ok(self)
    }
    fn set_stretch_blt_mode(self, _: usize, _: META_SETSTRETCHBLTMODE) -> Result<Self, PlayError> { Ok(self) }
    fn set_text_align(mut self, _: usize, record: META_SETTEXTALIGN) -> Result<Self, PlayError> {
        self.state.text_align = record.text_alignment_mode;
        Ok(self)
    }
    fn set_text_char_extra(self, _: usize, _: META_SETTEXTCHAREXTRA) -> Result<Self, PlayError> { Ok(self) }
    fn set_text_color(mut self, _: usize, record: META_SETTEXTCOLOR) -> Result<Self, PlayError> {
        self.state.text_color = record.color_ref;
        Ok(self)
    }
    fn set_text_justification(self, _: usize, _: META_SETTEXTJUSTIFICATION) -> Result<Self, PlayError> { Ok(self) }
    fn set_viewport_ext(mut self, _: usize, record: META_SETVIEWPORTEXT) -> Result<Self, PlayError> {
        self.state.viewport_ext = (record.x, record.y);
        self.state.viewport_ext_set = true;
        Ok(self)
    }
    fn set_viewport_origin(mut self, _: usize, record: META_SETVIEWPORTORG) -> Result<Self, PlayError> {
        self.state.viewport_origin = (record.x, record.y);
        Ok(self)
    }
    fn set_window_ext(mut self, _: usize, record: META_SETWINDOWEXT) -> Result<Self, PlayError> {
        let (w, h) = (record.x.abs(), record.y.abs());
        self.state.window_ext = (w, h);
        // header 의 extent 보다 큰 ext 시 갱신 — placeable 부재 케이스 대응
        if !matches!(self.extent, (1024, 1024)) {
            // header 가 이미 설정한 경우는 그대로 유지
        } else if w > 0 && h > 0 {
            self.extent = (w, h);
        }
        Ok(self)
    }
    fn set_window_origin(mut self, _: usize, record: META_SETWINDOWORG) -> Result<Self, PlayError> {
        self.state.window_origin = (record.x, record.y);
        Ok(self)
    }
    fn escape(self, _: usize, _: META_ESCAPE) -> Result<Self, PlayError> { Ok(self) }
}

/// [Stage 30] WMF spec [MS-WMF] §3.1.4: 객체는 lowest available slot 에 할당.
/// DeleteObject 후 인덱스 재사용 — 기존 `len()` 방식은 spec 위반.
fn lowest_free_slot(table: &std::collections::HashMap<u16, RasterObject>) -> u16 {
    for i in 0u16..=u16::MAX {
        if !table.contains_key(&i) {
            return i;
        }
    }
    0
}

/// [Stage 20] LO mtftools.cxx GetEllipticalArc 알고리즘 포팅.
/// bounding rect (x0,y0)-(x1,y1) 의 ellipse 의 start->end 호 경로 생성.
/// close_chord=true 면 start↔end 직선으로 닫음 (chord). 아니면 open arc.
fn arc_path(
    x0: f32, y0: f32, x1: f32, y1: f32,
    sx: f32, sy: f32, ex: f32, ey: f32,
    close_chord: bool,
) -> Option<tiny_skia::Path> {
    let cx = (x0 + x1) / 2.0;
    let cy = (y0 + y1) / 2.0;
    let rx = ((x1 - x0).abs()) / 2.0;
    let ry = ((y1 - y0).abs()) / 2.0;
    if rx < 0.5 || ry < 0.5 { return None; }

    // start/end 각도 계산 (ellipse normalized space)
    let start_angle = ((sy - cy) / ry).atan2((sx - cx) / rx);
    let end_angle = ((ey - cy) / ry).atan2((ex - cx) / rx);
    // WMF: anti-clockwise from start to end
    let mut sweep = end_angle - start_angle;
    if sweep <= 0.0 { sweep += std::f32::consts::TAU; }

    // sweep 을 8 step 으로 cubic bezier 근사
    let segments = 8;
    let step = sweep / segments as f32;
    let mut pb = PathBuilder::new();
    let p0_x = cx + rx * start_angle.cos();
    let p0_y = cy + ry * start_angle.sin();
    pb.move_to(p0_x, p0_y);
    for i in 1..=segments {
        let a = start_angle + step * i as f32;
        let p_x = cx + rx * a.cos();
        let p_y = cy + ry * a.sin();
        pb.line_to(p_x, p_y);
    }
    if close_chord {
        pb.close();
    }
    pb.finish()
}

/// pie 경로 — arc + center 직선 wedge.
fn arc_path_pie(
    x0: f32, y0: f32, x1: f32, y1: f32,
    sx: f32, sy: f32, ex: f32, ey: f32,
    cx: f32, cy: f32,
) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    pb.move_to(cx, cy);
    let _ = (x0, y0, x1, y1, sx, sy, ex, ey);
    // 단순화: cx→start→end→cx wedge — 향후 정밀화
    pb.line_to(sx, sy);
    pb.line_to(ex, ey);
    pb.close();
    pb.finish()
}
