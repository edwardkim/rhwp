//! EMF Player — 레코드 시퀀스를 순회하며 DC/ObjectTable을 갱신하고 SVG 노드를 발행한다.
//!
//! 단계 12 범위: 드로잉(선/사각형/타원/호/패스/폴리라인16). 텍스트·비트맵은 단계 13.

use std::fmt::Write;

use crate::emf::parser::objects::{Header, LogBrush, LogPen, PointL, RectL};
use crate::emf::parser::records::{ExtTextOut, Record, StretchDIBits};
use crate::emf::Error;

use super::device_context::{DcStack, GraphicsObject, ObjectTable};
use super::svg::{colorref_to_rgb, escape_xml, SvgBuilder};

use base64::Engine;

pub struct Player {
    pub dc_stack: DcStack,
    pub objects: ObjectTable,
    pub svg: SvgBuilder,
    pub render_rect: (f32, f32, f32, f32),
    pub header: Option<Header>,

    // 패스 상태
    path_active: bool,
    path_d: String,
}

impl Player {
    #[must_use]
    pub fn new(render_rect: (f32, f32, f32, f32)) -> Self {
        Self {
            dc_stack: DcStack::new(),
            objects: ObjectTable::new(),
            svg: SvgBuilder::new(),
            render_rect,
            header: None,
            path_active: false,
            path_d: String::new(),
        }
    }

    /// 레코드 시퀀스 전체 재생.
    pub fn play(&mut self, records: &[Record]) -> Result<(), Error> {
        // 먼저 헤더를 찾아 매핑 행렬을 세운다.
        if let Some(Record::Header(h)) = records.iter().find(|r| matches!(r, Record::Header(_))) {
            self.header = Some(h.clone());
        }
        self.open_root_group();

        for rec in records {
            self.exec(rec);
        }

        self.svg.close_group();
        Ok(())
    }

    fn open_root_group(&mut self) {
        // Bounds → render_rect 매핑. Bounds가 비어 있으면 identity.
        let (rx, ry, rw, rh) = self.render_rect;
        let m = if let Some(h) = &self.header {
            let w = (h.bounds.right - h.bounds.left) as f32;
            let hh = (h.bounds.bottom - h.bounds.top) as f32;
            if w > 0.0 && hh > 0.0 {
                let sx = rw / w;
                let sy = rh / hh;
                let tx = rx - h.bounds.left as f32 * sx;
                let ty = ry - h.bounds.top as f32 * sy;
                [sx, 0.0, 0.0, sy, tx, ty]
            } else {
                [1.0, 0.0, 0.0, 1.0, rx, ry]
            }
        } else {
            [1.0, 0.0, 0.0, 1.0, rx, ry]
        };
        self.svg.open_group_matrix(m);
    }

    fn exec(&mut self, rec: &Record) {
        match rec {
            Record::Header(_) | Record::Eof => {}

            // 객체
            Record::CreatePen { handle, pen } => {
                self.objects.insert(*handle, GraphicsObject::Pen(*pen))
            }
            Record::CreateBrushIndirect { handle, brush } => {
                self.objects.insert(*handle, GraphicsObject::Brush(*brush))
            }
            Record::ExtCreateFontIndirectW { handle, font } => self
                .objects
                .insert(*handle, GraphicsObject::Font(font.clone())),
            Record::SelectObject { handle } => self.select_object(*handle),
            Record::DeleteObject { handle } => {
                self.objects.remove(*handle);
            }

            // 상태 — DC
            Record::SaveDC => self.dc_stack.save(),
            Record::RestoreDC { relative } => {
                self.dc_stack.restore(*relative);
            }
            Record::SetWorldTransform(_) | Record::ModifyWorldTransform { .. } => {
                // 단계 12에서는 WorldTransform을 DC에 저장만 하고 출력 적용은 생략.
                // 단계 13~14에서 개별 도형에 transform 적용.
            }

            // 좌표계/색상
            Record::SetMapMode(m) => self.dc_stack.current_mut().map_mode = *m,
            Record::SetWindowExtEx(s) => self.dc_stack.current_mut().window_ext = (s.cx, s.cy),
            Record::SetWindowOrgEx(p) => self.dc_stack.current_mut().window_org = (p.x, p.y),
            Record::SetViewportExtEx(s) => self.dc_stack.current_mut().viewport_ext = (s.cx, s.cy),
            Record::SetViewportOrgEx(p) => self.dc_stack.current_mut().viewport_org = (p.x, p.y),
            Record::SetBkMode(v) => self.dc_stack.current_mut().bk_mode = *v,
            Record::SetTextAlign(v) => self.dc_stack.current_mut().text_align = *v,
            Record::SetTextColor(v) => self.dc_stack.current_mut().text_color = *v,
            Record::SetBkColor(v) => self.dc_stack.current_mut().bk_color = *v,

            // 드로잉
            Record::MoveToEx(p) => self.dc_stack.current_mut().current_pos = (p.x, p.y),
            Record::LineTo(p) => self.emit_line_to(p),
            Record::Rectangle(r) => self.emit_rect(r, None),
            Record::RoundRect {
                rect,
                corner_w,
                corner_h,
            } => self.emit_rect(rect, Some((*corner_w, *corner_h))),
            Record::Ellipse(r) => self.emit_ellipse(r),
            Record::Arc { rect, start, end } => self.emit_arc_like(rect, start, end, ArcKind::Arc),
            Record::Chord { rect, start, end } => {
                self.emit_arc_like(rect, start, end, ArcKind::Chord)
            }
            Record::Pie { rect, start, end } => self.emit_arc_like(rect, start, end, ArcKind::Pie),
            Record::Polyline16 { points, .. } => self.emit_polyline16(points, false),
            Record::Polygon16 { points, .. } => self.emit_polyline16(points, true),
            Record::PolyBezier16 { points, .. } => self.emit_polybezier16(points),

            // 패스
            Record::BeginPath => {
                self.path_active = true;
                self.path_d.clear();
            }
            Record::EndPath => {
                self.path_active = false;
            }
            Record::CloseFigure => {
                if !self.path_d.is_empty() {
                    self.path_d.push_str(" Z");
                }
            }
            Record::FillPath(_) => {
                let (fill, stroke) = (self.fill_spec(), None);
                self.emit_path(fill, stroke);
            }
            Record::StrokePath(_) => {
                self.emit_path(None, Some(self.stroke_spec()));
            }
            Record::StrokeAndFillPath(_) => {
                self.emit_path(self.fill_spec(), Some(self.stroke_spec()));
            }

            // 텍스트
            Record::ExtTextOutW(t) => self.emit_text(t),

            // 비트맵
            Record::StretchDIBits(bmp) => self.emit_bitmap(bmp),

            Record::Unknown { .. } => {}
        }
    }

    fn emit_text(&mut self, t: &ExtTextOut) {
        if t.text.is_empty() {
            return;
        }
        let dc = self.dc_stack.current();
        let color = colorref_to_rgb(dc.text_color);
        // 폰트
        let (family, size, weight, italic) = if let Some(f) = &dc.font {
            let fam = if f.face_name.is_empty() {
                "sans-serif".to_string()
            } else {
                f.face_name.clone()
            };
            // LogFontW.height: 음수=cell height, 양수=character height. |height|를 px로 사용.
            let size = f.height.unsigned_abs().max(1) as f32;
            let weight = if f.weight >= 700 { "bold" } else { "normal" };
            let italic = if f.italic != 0 { "italic" } else { "normal" };
            (fam, size, weight, italic)
        } else {
            ("sans-serif".to_string(), 12.0, "normal", "normal")
        };
        let node = format!(
            "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{:.2}\" font-weight=\"{}\" font-style=\"{}\" fill=\"{}\">{}</text>",
            t.reference.x, t.reference.y,
            escape_xml(&family), size, weight, italic, color,
            escape_xml(&t.text),
        );
        self.svg.push(&node);
    }

    fn emit_bitmap(&mut self, bmp: &StretchDIBits) {
        // DIB(BMI+bits) → BMP 파일 포맷으로 래핑 → base64 data URL.
        let data_url = dib_to_bmp_data_url(&bmp.bmi, &bmp.bits);
        let node = format!(
            "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" href=\"{data_url}\"/>",
            bmp.x_dest, bmp.y_dest, bmp.cx_dest, bmp.cy_dest,
        );
        self.svg.push(&node);
    }

    fn select_object(&mut self, handle: u32) {
        let Some(obj) = self.objects.get(handle) else {
            return;
        };
        match obj {
            GraphicsObject::Pen(p) => self.dc_stack.current_mut().pen = Some(*p),
            GraphicsObject::Brush(b) => self.dc_stack.current_mut().brush = Some(*b),
            GraphicsObject::Font(f) => self.dc_stack.current_mut().font = Some(f.clone()),
        }
    }

    fn stroke_spec(&self) -> StrokeSpec {
        if let Some(p) = self.dc_stack.current().pen {
            // PS_NULL(5) → 스트로크 없음
            let is_null = (p.style & 0x0F) == 5;
            if is_null {
                StrokeSpec {
                    color: None,
                    width: 0.0,
                }
            } else {
                StrokeSpec {
                    color: Some(colorref_to_rgb(p.color)),
                    width: p.width.max(1) as f32,
                }
            }
        } else {
            StrokeSpec {
                color: Some("black".into()),
                width: 1.0,
            }
        }
    }

    fn fill_spec(&self) -> Option<String> {
        if let Some(b) = self.dc_stack.current().brush {
            if b.style == 1 {
                None
            }
            // BS_NULL
            else {
                Some(colorref_to_rgb(b.color))
            }
        } else {
            Some("none".into())
        }
    }

    fn emit_line_to(&mut self, to: &PointL) {
        let (x1, y1) = self.dc_stack.current().current_pos;
        let s = self.stroke_spec();
        let color = s.color.as_deref().unwrap_or("none");
        let node = format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"{:.2}\" fill=\"none\"/>",
            x1, y1, to.x, to.y, s.width,
        );
        if self.path_active {
            if self.path_d.is_empty() {
                let _ = write!(self.path_d, "M{x1} {y1} ");
            }
            let _ = write!(self.path_d, "L{} {} ", to.x, to.y);
        } else {
            self.svg.push(&node);
        }
        self.dc_stack.current_mut().current_pos = (to.x, to.y);
    }

    fn emit_rect(&mut self, r: &RectL, corner: Option<(i32, i32)>) {
        let stroke = self.stroke_spec();
        let fill = self.fill_spec().unwrap_or_else(|| "none".into());
        let stroke_color = stroke.color.as_deref().unwrap_or("none");
        let (rx_attr, ry_attr) = match corner {
            Some((cw, ch)) => (format!(" rx=\"{}\"", cw / 2), format!(" ry=\"{}\"", ch / 2)),
            None => (String::new(), String::new()),
        };
        let node = format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{rx_attr}{ry_attr} fill=\"{fill}\" stroke=\"{stroke_color}\" stroke-width=\"{:.2}\"/>",
            r.left, r.top, r.width(), r.height(), stroke.width,
        );
        self.svg.push(&node);
    }

    fn emit_ellipse(&mut self, r: &RectL) {
        let stroke = self.stroke_spec();
        let fill = self.fill_spec().unwrap_or_else(|| "none".into());
        let cx = (r.left + r.right) / 2;
        let cy = (r.top + r.bottom) / 2;
        let rx = (r.right - r.left).abs() / 2;
        let ry = (r.bottom - r.top).abs() / 2;
        let stroke_color = stroke.color.as_deref().unwrap_or("none");
        let node = format!(
            "<ellipse cx=\"{cx}\" cy=\"{cy}\" rx=\"{rx}\" ry=\"{ry}\" fill=\"{fill}\" stroke=\"{stroke_color}\" stroke-width=\"{:.2}\"/>",
            stroke.width,
        );
        self.svg.push(&node);
    }

    fn emit_arc_like(&mut self, r: &RectL, start: &PointL, end: &PointL, kind: ArcKind) {
        // 근사: arc은 시작점→끝점 단순 선, chord는 같음, pie는 중심까지 삼각형 폐곡선.
        // 단계 12는 SVG arc path로 표현.
        let cx = (r.left + r.right) as f32 / 2.0;
        let cy = (r.top + r.bottom) as f32 / 2.0;
        let rx = (r.right - r.left).abs() as f32 / 2.0;
        let ry = (r.bottom - r.top).abs() as f32 / 2.0;
        let (s, e) = (start, end);
        let stroke = self.stroke_spec();
        let fill = match kind {
            ArcKind::Arc => "none".to_string(),
            ArcKind::Chord | ArcKind::Pie => self.fill_spec().unwrap_or_else(|| "none".into()),
        };
        let d = match kind {
            ArcKind::Arc => format!("M {} {} A {} {} 0 0 1 {} {}", s.x, s.y, rx, ry, e.x, e.y),
            ArcKind::Chord => format!("M {} {} A {} {} 0 0 1 {} {} Z", s.x, s.y, rx, ry, e.x, e.y),
            ArcKind::Pie => format!(
                "M {cx} {cy} L {} {} A {} {} 0 0 1 {} {} Z",
                s.x, s.y, rx, ry, e.x, e.y
            ),
        };
        let stroke_color = stroke.color.as_deref().unwrap_or("none");
        let node = format!(
            "<path d=\"{d}\" fill=\"{fill}\" stroke=\"{stroke_color}\" stroke-width=\"{:.2}\"/>",
            stroke.width,
        );
        self.svg.push(&node);
    }

    fn emit_polyline16(&mut self, points: &[(i16, i16)], close: bool) {
        if points.is_empty() {
            return;
        }
        let pts: String = points
            .iter()
            .map(|(x, y)| format!("{x},{y}"))
            .collect::<Vec<_>>()
            .join(" ");
        let stroke = self.stroke_spec();
        let fill = if close {
            self.fill_spec().unwrap_or_else(|| "none".into())
        } else {
            "none".into()
        };
        let tag = if close { "polygon" } else { "polyline" };
        let stroke_color = stroke.color.as_deref().unwrap_or("none");
        let node = format!(
            "<{tag} points=\"{pts}\" fill=\"{fill}\" stroke=\"{stroke_color}\" stroke-width=\"{:.2}\"/>",
            stroke.width,
        );
        self.svg.push(&node);
    }

    fn emit_polybezier16(&mut self, points: &[(i16, i16)]) {
        if points.is_empty() {
            return;
        }
        let mut d = format!("M{} {}", points[0].0, points[0].1);
        // EMF PolyBezier: 첫 점은 시작점, 이후 3점씩 제어1 제어2 끝점(C 커맨드).
        let mut i = 1;
        while i + 2 < points.len() + 1 && i + 2 <= points.len() {
            let (c1x, c1y) = points[i];
            let (c2x, c2y) = points[i + 1];
            let (ex, ey) = points[i + 2];
            let _ = write!(d, " C{c1x} {c1y} {c2x} {c2y} {ex} {ey}");
            i += 3;
        }
        let stroke = self.stroke_spec();
        let stroke_color = stroke.color.as_deref().unwrap_or("none");
        let node = format!(
            "<path d=\"{d}\" fill=\"none\" stroke=\"{stroke_color}\" stroke-width=\"{:.2}\"/>",
            stroke.width,
        );
        self.svg.push(&node);
    }

    fn emit_path(&mut self, fill: Option<String>, stroke: Option<StrokeSpec>) {
        if self.path_d.is_empty() {
            return;
        }
        let fill_attr = fill.as_deref().unwrap_or("none");
        let (stroke_color, stroke_width) = stroke.map_or(("none".into(), 0.0_f32), |s| {
            (s.color.unwrap_or_else(|| "none".into()), s.width)
        });
        let node = format!(
            "<path d=\"{}\" fill=\"{fill_attr}\" stroke=\"{stroke_color}\" stroke-width=\"{:.2}\"/>",
            self.path_d.trim(),
            stroke_width,
        );
        self.svg.push(&node);
        self.path_d.clear();
    }
}

#[derive(Copy, Clone)]
enum ArcKind {
    Arc,
    Chord,
    Pie,
}

#[derive(Debug, Clone)]
pub struct StrokeSpec {
    pub color: Option<String>,
    pub width: f32,
}

/// DIB(BITMAPINFO + bits)를 BMP 파일 포맷으로 래핑한 후 PNG 로 변환하여 base64 data URL로 반환.
///
/// BMP 파일 헤더(14B): `"BM"` + file_size(u32) + reserved(u32)=0 + data_offset(u32)
///
/// [Task #860] SVG renderer (rsvg-convert, 브라우저) 가 `data:image/bmp` URI 미지원 →
/// `data:image/png` 로 재인코딩 (svg.rs:1118 / shape_layout.rs:1063 과 동일 정책).
/// BMP decode 실패 시 fallback 으로 BMP URI 반환 (graceful degradation).
fn dib_to_bmp_data_url(bmi: &[u8], bits: &[u8]) -> String {
    let bmi_size = bmi.len() as u32;
    let bits_size = bits.len() as u32;
    let file_size = 14 + bmi_size + bits_size;
    let data_offset = 14 + bmi_size;

    let mut bmp = Vec::with_capacity(file_size as usize);
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&data_offset.to_le_bytes());
    bmp.extend_from_slice(bmi);
    bmp.extend_from_slice(bits);

    if let Some(png) = crate::renderer::svg::bmp_bytes_to_png_bytes(&bmp) {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
        format!("data:image/png;base64,{b64}")
    } else {
        // fallback: BMP (decode 실패 시)
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bmp);
        format!("data:image/bmp;base64,{b64}")
    }
}
