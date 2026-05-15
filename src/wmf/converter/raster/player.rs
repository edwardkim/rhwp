//! [Task #902 v2 Stage 12] WMF raster Player baseline.
//!
//! 알고리즘 출처: LibreOffice emfio (MPL 2.0)
//!   wmfreader.cxx + mtftools.cxx 의 WMF record 처리 로직 포팅 baseline.
//!
//! 본 stage 는 Player trait 의 stub 구현. Stage 13+ 에서 점진적으로
//! tiny-skia + fontdue 로 실제 raster 렌더링 구현.

use super::state::*;
use crate::wmf::converter::{PlayError, Player};
use crate::wmf::parser::*;

#[cfg(not(target_arch = "wasm32"))]
use tiny_skia::Pixmap;

/// WMF records 를 tiny-skia Pixmap 에 직접 렌더링하는 Player.
///
/// 출력: PNG bytes (`generate()` 호출 시).
///
/// 현재 (Stage 12) state 추적 + state context 만 구현. 실제 drawing 은
/// Stage 13+ 에서 점진적 구현 — drawing 함수는 NOP (state 만 유지).
#[derive(Clone, Debug)]
pub struct RasterPlayer {
    state: RasterState,
    /// DC stack (META_SAVEDC / META_RESTOREDC)
    state_stack: Vec<RasterState>,
    /// 출력 canvas 크기 (device pixels).
    canvas_width: u32,
    canvas_height: u32,
    /// 누적 logical 좌표 bbox — header 의 bound 후 SetWindowExt 등에 갱신.
    extent: (i16, i16),
}

impl RasterPlayer {
    pub fn new(canvas_width: u32, canvas_height: u32) -> Self {
        Self {
            state: RasterState::default(),
            state_stack: Vec::new(),
            canvas_width,
            canvas_height,
            extent: (1024, 1024),
        }
    }

    /// 빈 canvas 생성 (default 1024x1024).
    pub fn default_canvas() -> Self {
        Self::new(1024, 1024)
    }
}

impl Player for RasterPlayer {
    fn generate(self) -> Result<Vec<u8>, PlayError> {
        // [Task #902 v2 Stage 12] 임시 stub — 빈 흰색 PNG 반환.
        // Stage 13+ 에서 실제 raster 출력 구현.
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut pixmap = Pixmap::new(self.canvas_width, self.canvas_height)
                .ok_or_else(|| PlayError::InvalidRecord {
                    cause: format!(
                        "canvas size {}x{} 생성 실패",
                        self.canvas_width, self.canvas_height
                    ),
                })?;
            pixmap.fill(tiny_skia::Color::WHITE);
            pixmap.encode_png().map_err(|err| PlayError::InvalidRecord {
                cause: format!("PNG encode 실패: {err}"),
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(PlayError::InvalidRecord {
                cause: "RasterPlayer is native-only".to_owned(),
            })
        }
    }

    // === Header / EOF ===

    fn header(
        mut self,
        _record_number: usize,
        record: MetafileHeader,
    ) -> Result<Self, PlayError> {
        // Placeable WMF 의 bound (window size 추정) 사용 — emfio 의 ReadHeader.
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

    // === Bitmap records (Stage 13+ 구현) ===
    fn bit_blt(self, _: usize, _: META_BITBLT) -> Result<Self, PlayError> { Ok(self) }
    fn device_independent_bitmap_bit_blt(self, _: usize, _: META_DIBBITBLT) -> Result<Self, PlayError> { Ok(self) }
    fn device_independent_bitmap_stretch_blt(self, _: usize, _: META_DIBSTRETCHBLT) -> Result<Self, PlayError> { Ok(self) }
    fn set_device_independent_bitmap_to_dev(self, _: usize, _: META_SETDIBTODEV) -> Result<Self, PlayError> { Ok(self) }
    fn stretch_blt(self, _: usize, _: META_STRETCHBLT) -> Result<Self, PlayError> { Ok(self) }
    fn stretch_device_independent_bitmap(self, _: usize, _: META_STRETCHDIB) -> Result<Self, PlayError> { Ok(self) }

    // === Drawing records (Stage 13+ 구현) ===
    fn arc(self, _: usize, _: META_ARC) -> Result<Self, PlayError> { Ok(self) }
    fn chord(self, _: usize, _: META_CHORD) -> Result<Self, PlayError> { Ok(self) }
    fn ellipse(self, _: usize, _: META_ELLIPSE) -> Result<Self, PlayError> { Ok(self) }
    fn ext_flood_fill(self, _: usize, _: META_EXTFLOODFILL) -> Result<Self, PlayError> { Ok(self) }
    fn ext_text_out(self, _: usize, _: META_EXTTEXTOUT) -> Result<Self, PlayError> { Ok(self) }
    fn fill_region(self, _: usize, _: META_FILLREGION) -> Result<Self, PlayError> { Ok(self) }
    fn flood_fill(self, _: usize, _: META_FLOODFILL) -> Result<Self, PlayError> { Ok(self) }
    fn frame_region(self, _: usize, _: META_FRAMEREGION) -> Result<Self, PlayError> { Ok(self) }
    fn invert_region(self, _: usize, _: META_INVERTREGION) -> Result<Self, PlayError> { Ok(self) }
    fn line_to(self, _: usize, _: META_LINETO) -> Result<Self, PlayError> { Ok(self) }
    fn paint_region(self, _: usize, _: META_PAINTREGION) -> Result<Self, PlayError> { Ok(self) }
    fn pat_blt(self, _: usize, _: META_PATBLT) -> Result<Self, PlayError> { Ok(self) }
    fn pie(self, _: usize, _: META_PIE) -> Result<Self, PlayError> { Ok(self) }
    fn polyline(self, _: usize, _: META_POLYLINE) -> Result<Self, PlayError> { Ok(self) }
    fn polygon(self, _: usize, _: META_POLYGON) -> Result<Self, PlayError> { Ok(self) }
    fn poly_polygon(self, _: usize, _: META_POLYPOLYGON) -> Result<Self, PlayError> { Ok(self) }
    fn rectangle(self, _: usize, _: META_RECTANGLE) -> Result<Self, PlayError> { Ok(self) }
    fn round_rect(self, _: usize, _: META_ROUNDRECT) -> Result<Self, PlayError> { Ok(self) }
    fn set_pixel(self, _: usize, _: META_SETPIXEL) -> Result<Self, PlayError> { Ok(self) }
    fn text_out(self, _: usize, _: META_TEXTOUT) -> Result<Self, PlayError> { Ok(self) }

    // === Object records (Stage 13+ 구현) ===
    fn create_brush_indirect(self, _: usize, _: META_CREATEBRUSHINDIRECT) -> Result<Self, PlayError> { Ok(self) }
    fn create_font_indirect(self, _: usize, _: META_CREATEFONTINDIRECT) -> Result<Self, PlayError> { Ok(self) }
    fn create_palette(self, _: usize, _: META_CREATEPALETTE) -> Result<Self, PlayError> { Ok(self) }
    fn create_pattern_brush(self, _: usize, _: META_CREATEPATTERNBRUSH) -> Result<Self, PlayError> { Ok(self) }
    fn create_pen_indirect(self, _: usize, _: META_CREATEPENINDIRECT) -> Result<Self, PlayError> { Ok(self) }
    fn create_region(self, _: usize, _: META_CREATEREGION) -> Result<Self, PlayError> { Ok(self) }
    fn delete_object(self, _: usize, _: META_DELETEOBJECT) -> Result<Self, PlayError> { Ok(self) }
    fn create_device_independent_bitmap_pattern_brush(self, _: usize, _: META_DIBCREATEPATTERNBRUSH) -> Result<Self, PlayError> { Ok(self) }
    fn select_clip_region(self, _: usize, _: META_SELECTCLIPREGION) -> Result<Self, PlayError> { Ok(self) }
    fn select_object(self, _: usize, _: META_SELECTOBJECT) -> Result<Self, PlayError> { Ok(self) }
    fn select_palette(self, _: usize, _: META_SELECTPALETTE) -> Result<Self, PlayError> { Ok(self) }

    // === State records (Stage 13+ 구현) ===
    fn animate_palette(self, _: usize, _: META_ANIMATEPALETTE) -> Result<Self, PlayError> { Ok(self) }
    fn exclude_clip_rect(self, _: usize, _: META_EXCLUDECLIPRECT) -> Result<Self, PlayError> { Ok(self) }
    fn intersect_clip_rect(self, _: usize, _: META_INTERSECTCLIPRECT) -> Result<Self, PlayError> { Ok(self) }
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
            // 음수는 top-most 부터 |n| 번째
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
        self.state.window_ext = (record.x.abs(), record.y.abs());
        Ok(self)
    }
    fn set_window_origin(mut self, _: usize, record: META_SETWINDOWORG) -> Result<Self, PlayError> {
        self.state.window_origin = (record.x, record.y);
        Ok(self)
    }
    fn escape(self, _: usize, _: META_ESCAPE) -> Result<Self, PlayError> { Ok(self) }
}
