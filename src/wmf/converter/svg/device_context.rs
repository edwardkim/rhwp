use crate::wmf::converter::{svg::util::css_color_from_color_ref, *};

#[derive(Clone, Debug)]
pub struct DeviceContext {
    // graphics object
    pub object_table: GraphicsObjects,

    // structures
    pub drawing_position: PointS,
    pub text_bk_color: ColorRef,
    pub text_color: ColorRef,
    pub window: Window,

    // graphics props
    pub bk_mode: MixMode,
    pub clipping_region: Option<Rect>,
    pub poly_fill_mode: PolyFillMode,
    pub text_align_horizontal: TextAlignmentMode,
    pub text_align_vertical: VerticalTextAlignmentMode,
    pub text_align_update_cp: bool,

    pub draw_mode: Option<BinaryRasterOperation>,
    pub map_mode: MapMode,
}

impl Default for DeviceContext {
    fn default() -> Self {
        Self {
            object_table: GraphicsObjects::new(0),
            bk_mode: MixMode::TRANSPARENT,
            clipping_region: None,
            drawing_position: PointS { x: 0, y: 0 },
            draw_mode: None,
            map_mode: MapMode::MM_TEXT,
            poly_fill_mode: PolyFillMode::ALTERNATE,
            text_align_horizontal: TextAlignmentMode::TA_LEFT,
            text_align_vertical: VerticalTextAlignmentMode::VTA_BASELINE,
            text_align_update_cp: false,
            text_bk_color: ColorRef::white(),
            text_color: ColorRef::black(),
            window: Window::new(),
        }
    }
}

// mutations
impl DeviceContext {
    pub fn bk_mode(mut self, bk_mode: MixMode) -> Self {
        self.bk_mode = bk_mode;
        self
    }

    pub fn create_object_table(mut self, length: u16) -> Self {
        self.object_table = GraphicsObjects::new(length as usize);
        self
    }

    pub fn clipping_region(mut self, clipping_region: Rect) -> Self {
        let clipping_region = if let Some(ref existing) = self.clipping_region {
            if let Some(overlap_region) = existing.overlap(&clipping_region) {
                overlap_region
            } else {
                clipping_region
            }
        } else {
            clipping_region
        };

        self.clipping_region = clipping_region.into();
        self
    }

    pub fn drawing_position(mut self, drawing_position: PointS) -> Self {
        self.drawing_position = drawing_position;
        self
    }

    pub fn draw_mode(mut self, draw_mode: BinaryRasterOperation) -> Self {
        self.draw_mode = draw_mode.into();
        self
    }

    pub fn extend_window(self, p: &PointS) -> Self {
        // SetWindowExt 또는 Placeable 헤더로 명시적으로 설정된 경우 자동 확장하지 않음
        if self.window.ext_explicitly_set {
            return self;
        }

        let (mut x, mut y) = (0, 0);

        if self.window.x < p.x {
            x = p.x;
        }

        if self.window.y < p.y {
            y = p.y;
        }

        if x > 0 && y > 0 {
            self.window_ext(x, y)
        } else {
            self
        }
    }

    pub fn map_mode(mut self, map_mode: MapMode) -> Self {
        self.map_mode = map_mode;
        self
    }

    pub fn poly_fill_mode(mut self, poly_fill_mode: PolyFillMode) -> Self {
        self.poly_fill_mode = poly_fill_mode;
        self
    }

    pub fn text_align_horizontal(mut self, text_align_horizontal: TextAlignmentMode) -> Self {
        self.text_align_horizontal = text_align_horizontal;
        self
    }

    pub fn text_align_vertical(mut self, text_align_vertical: VerticalTextAlignmentMode) -> Self {
        self.text_align_vertical = text_align_vertical;
        self
    }

    pub fn text_align_update_cp(mut self, text_align_update_cp: bool) -> Self {
        self.text_align_update_cp = text_align_update_cp;
        self
    }

    pub fn text_bk_color(mut self, text_bk_color: ColorRef) -> Self {
        self.text_bk_color = text_bk_color;
        self
    }

    pub fn text_color(mut self, text_color: ColorRef) -> Self {
        self.text_color = text_color;
        self
    }

    pub fn window_ext(mut self, x: i16, y: i16) -> Self {
        self.window = self.window.ext(x, y);
        self
    }

    pub fn window_origin(mut self, x: i16, y: i16) -> Self {
        self.window = self.window.origin(x, y);
        self
    }

    pub fn window_scale(mut self, x: f32, y: f32) -> Self {
        self.window = self.window.scale(x, y);
        self
    }
}

impl DeviceContext {
    pub fn as_css_text_align(&self) -> String {
        match self.text_align_horizontal {
            TextAlignmentMode::TA_CENTER => "middle".to_owned(),
            TextAlignmentMode::TA_RIGHT => "end".to_owned(),
            _ => "start".to_owned(),
        }
    }

    pub fn as_css_text_align_vertical(&self) -> String {
        match self.text_align_vertical {
            VerticalTextAlignmentMode::VTA_BOTTOM => "text-bottom".to_owned(),
            // VTA_TOP: y 좌표에서 이미 ascent를 보정했으므로 기본 baseline 사용
            VerticalTextAlignmentMode::VTA_TOP => "auto".to_owned(),
            VerticalTextAlignmentMode::VTA_CENTER => "central".to_owned(),
            _ => "auto".to_owned(),
        }
    }

    pub fn point_s_to_absolute_point(&self, point: &PointS) -> PointS {
        let (x, y) = self
            .window
            .to_device(f32::from(point.x), f32::from(point.y));

        PointS {
            x: x as i16,
            y: y as i16,
        }
    }

    pub fn point_s_to_relative_point(&self, point: &PointS) -> PointS {
        let (x, y) = self
            .window
            .to_device(f32::from(point.x), f32::from(point.y));

        PointS {
            x: x as i16 + self.drawing_position.x,
            y: y as i16 + self.drawing_position.y,
        }
    }

    /// [#6617] 블릿(BitBlt·StretchBlt·DIB 계열) 목적 사각형을 장치 좌표로 옮긴다.
    ///
    /// 두 모서리 `(x, y)`·`(x + width, y + height)` 를 각각 창 매핑으로 보내고 작은 쪽을
    /// 원점으로 삼는다. GDI 의 뒤집기는 논리 폭/높이 부호와 창 축 방향의 **곱**으로
    /// 정해지므로, 장치 좌표에서 두 번째 모서리가 첫 모서리보다 앞이면 그 축을 뒤집는다.
    /// y-up 창(SetWindowExt y<0)의 음수 높이 DIB 는 두 부호가 상쇄돼 바로 선 그림이다
    /// (bitmap.hwp OLE 표현, 156462405 7쪽 인물 사진).
    pub fn blit_dest_rect(&self, x: i16, y: i16, width: i16, height: i16) -> BlitDestRect {
        let (x0, y0) = self.window.to_device(f32::from(x), f32::from(y));
        let (x1, y1) = self.window.to_device(
            f32::from(x) + f32::from(width),
            f32::from(y) + f32::from(height),
        );
        let (ix0, iy0, ix1, iy1) = (x0 as i32, y0 as i32, x1 as i32, y1 as i32);
        BlitDestRect {
            x: ix0.min(ix1),
            y: iy0.min(iy1),
            width: (ix1 - ix0).abs(),
            height: (iy1 - iy0).abs(),
            flip_x: ix1 < ix0,
            flip_y: iy1 < iy0,
        }
    }

    pub fn poly_fill_rule(&self) -> String {
        match self.poly_fill_mode {
            PolyFillMode::ALTERNATE => "evenodd",
            PolyFillMode::WINDING => "nonzero",
        }
        .to_owned()
    }

    pub fn text_color_as_css_color(&self) -> String {
        css_color_from_color_ref(&self.text_color)
    }
}

/// [#6617] 장치(viewBox) 좌표로 정규화한 블릿 목적 사각형. 폭·높이는 0 이상이고
/// 뒤집힘은 `flip_x`/`flip_y` 로 따로 든다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlitDestRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Clone, Debug)]
pub struct Window {
    pub x: i16,
    pub y: i16,
    pub origin_x: i16,
    pub origin_y: i16,
    pub scale_x: f32,
    pub scale_y: f32,
    /// SetWindowExt가 명시적으로 호출되었는지 여부
    pub ext_explicitly_set: bool,
    /// SetWindowExt x < 0 — 논리 x 가 커질수록 장치에서는 왼쪽으로 간다.
    pub x_inverted: bool,
    /// SetWindowExt y < 0 (Cartesian, bottom-up) — 논리 y 가 커질수록 장치에서는 위로 간다.
    /// SVG 는 top-down 이라 `to_device` 가 y 축을 뒤집는다.
    pub y_inverted: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            x: 1024,
            y: 1024,
            origin_x: 0,
            origin_y: 0,
            scale_x: 1.0,
            scale_y: 1.0,
            ext_explicitly_set: false,
            x_inverted: false,
            y_inverted: false,
        }
    }
}

impl Window {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ext(mut self, x: i16, y: i16) -> Self {
        self.x = x.abs();
        self.y = y.abs();
        self.ext_explicitly_set = true;
        self.x_inverted = x < 0;
        self.y_inverted = y < 0;
        self
    }

    /// [#6617] 논리 좌표를 창 원점 기준 장치(viewBox) 좌표로 옮긴다.
    ///
    /// viewBox 는 `(0, 0, |ext_x|, |ext_y|)` 이고(`as_view_box`), 창 범위 안의 논리 점은
    /// 여기서 `[0, |ext|]` 로 들어온다. 축이 뒤집힌 창(SetWindowExt 음수)은 그 축의 부호를
    /// 바꿔서 옮긴다 — y-up 창에서 논리 `origin_y` 가 장치 0(위), `origin_y + ext_y`
    /// (ext_y<0) 가 장치 `|ext_y|`(아래) 다. 창 밖의 점은 음수 또는 범위 밖 좌표가 되고,
    /// `generate` 의 viewBox 자동 확장이 그 점을 포함시킨다.
    pub fn to_device(&self, x: f32, y: f32) -> (f32, f32) {
        let dx = x - f32::from(self.origin_x);
        let dy = y - f32::from(self.origin_y);
        let dx = if self.x_inverted { -dx } else { dx };
        let dy = if self.y_inverted { -dy } else { dy };
        (dx / self.scale_x, dy / self.scale_y)
    }

    pub fn origin(mut self, origin_x: i16, origin_y: i16) -> Self {
        self.origin_x = origin_x;
        self.origin_y = origin_y;
        self
    }

    pub fn scale(mut self, scale_x: f32, scale_y: f32) -> Self {
        self.scale_x = scale_x;
        self.scale_y = scale_y;
        self
    }

    pub fn as_view_box(&self) -> (i16, i16, i16, i16) {
        // [Task #864] element 좌표는 모두 `point_s_to_absolute_point` 로 origin-relative
        // (device coord) 변환됨. image (TernaryRasterOperator) 도 호출 측에서 동일하게
        // 변환 (Task #864). viewBox 도 이 device 공간 (0, 0, ext_x, ext_y) 으로 정합.
        // (Task #860 Stage D 의 (origin_x, origin_y, ...) 변경 revert — image 와 text
        // 의 좌표 공간이 mismatch 였던 본질을 정정.)
        (0, 0, self.x.abs(), self.y.abs())
    }
}
