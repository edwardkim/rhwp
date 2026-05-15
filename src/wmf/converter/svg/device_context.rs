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
    /// [Task #902 v2 Stage 4] MM_ANISOTROPIC 에서 Window/Viewport ratio 로
    /// device 좌표 계산. ViewportExt 미호출 시 Task #860 의 viewBox 자동 확장
    /// 동작 보존을 위해 ext_explicitly_set 플래그로 분기.
    pub viewport: Viewport,

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
            viewport: Viewport::new(),
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

    pub fn text_align_horizontal(
        mut self,
        text_align_horizontal: TextAlignmentMode,
    ) -> Self {
        self.text_align_horizontal = text_align_horizontal;
        self
    }

    pub fn text_align_vertical(
        mut self,
        text_align_vertical: VerticalTextAlignmentMode,
    ) -> Self {
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

    // [Task #902 v2 Stage 4] Viewport mutators
    pub fn viewport_ext(mut self, x: i16, y: i16) -> Self {
        self.viewport = self.viewport.ext(x, y);
        self
    }

    pub fn viewport_origin(mut self, x: i16, y: i16) -> Self {
        self.viewport = self.viewport.origin(x, y);
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
        let (dx, dy) = self.logical_to_device_delta(point);
        PointS {
            x: dx + self.viewport.origin_x,
            y: dy + self.viewport.origin_y,
        }
    }

    pub fn point_s_to_relative_point(&self, point: &PointS) -> PointS {
        let (dx, dy) = self.logical_to_device_delta(point);
        PointS {
            x: dx + self.viewport.origin_x + self.drawing_position.x,
            y: dy + self.viewport.origin_y + self.drawing_position.y,
        }
    }

    /// [Task #902 v2 Stage 4] logical 좌표 → device 좌표 delta 변환.
    /// MM_ANISOTROPIC 의 Window/Viewport ratio 정합:
    ///   device = (logical - WindowOrg) × (ViewportExt / WindowExt) + ViewportOrg
    /// ViewportExt 미설정 시 기존 동작 (window.scale 또는 1:1) 유지하여
    /// Task #860 의 viewBox 자동 확장 fixture 회귀 방지.
    fn logical_to_device_delta(&self, point: &PointS) -> (i16, i16) {
        let dx_logical = f32::from((point.x - self.window.origin_x).abs());
        let dy_logical = f32::from((point.y - self.window.origin_y).abs());

        let x = if self.viewport.ext_explicitly_set {
            // MM_ANISOTROPIC with explicit ViewportExt: 정확 ratio
            let ratio =
                f32::from(self.viewport.x) / f32::from(self.window.x.max(1));
            (dx_logical * ratio) as i16
        } else {
            (dx_logical / self.window.scale_x) as i16
        };

        let y = if self.viewport.ext_explicitly_set {
            let ratio =
                f32::from(self.viewport.y) / f32::from(self.window.y.max(1));
            (dy_logical * ratio) as i16
        } else {
            (dy_logical / self.window.scale_y) as i16
        };

        (x, y)
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
    /// [Task #860 Stage D] WMF 의 SetWindowExt y < 0 (Cartesian, bottom-up) 인 경우 true.
    /// SVG renderer 는 top-down (y 아래 증가). y < 0 처리를 위해 element y 좌표 flip 필요.
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
        // [Task #860 Stage D] y < 0 = Cartesian 좌표계 (bottom-up) — 일부 application
        // 이 WMF 에 SetWindowExt(width, -height) 로 bottom-up 설정. SVG 변환 시
        // y-flip transform 필요. 현재 sample 들에서는 미발견.
        if y < 0 {
            self.y_inverted = true;
        }
        self
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

/// [Task #902 v2 Stage 4] MM_ANISOTROPIC 의 Viewport (device coord) 정보.
/// SetViewportExt / SetViewportOrg 의 명시 호출 추적.
#[derive(Clone, Debug)]
pub struct Viewport {
    pub x: i16,
    pub y: i16,
    pub origin_x: i16,
    pub origin_y: i16,
    /// SetViewportExt 가 명시적으로 호출되었는지 여부. true 이면 정확한
    /// MM_ANISOTROPIC ratio 적용; false 면 기존 Task #860 자동 확장 동작 유지.
    pub ext_explicitly_set: bool,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 1,
            y: 1,
            origin_x: 0,
            origin_y: 0,
            ext_explicitly_set: false,
        }
    }
}

impl Viewport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ext(mut self, x: i16, y: i16) -> Self {
        self.x = x;
        self.y = y;
        self.ext_explicitly_set = true;
        self
    }

    pub fn origin(mut self, origin_x: i16, origin_y: i16) -> Self {
        self.origin_x = origin_x;
        self.origin_y = origin_y;
        self
    }
}
