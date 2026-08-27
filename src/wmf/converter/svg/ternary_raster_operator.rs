use crate::wmf::converter::{
    svg::{node::Node, util::url_string, Fill},
    *,
};

#[derive(Clone, Debug, snafu::prelude::Snafu)]
pub enum TernaryRasterOperationError {
    #[snafu(display("no brush specified: {cause}"))]
    NoBrush { cause: String },
    #[snafu(display("no source bitmap specified: {cause}"))]
    NoSource { cause: String },
}

pub struct TernaryRasterOperator {
    operation: TernaryRasterOperation,
    x: i16,
    y: i16,
    height: i16,
    width: i16,
    brush: Option<Brush>,
    source: Option<Source>,
}

enum Source {
    Bitmap16(Bitmap16),
    Bitmap(DeviceIndependentBitmap),
}

impl TernaryRasterOperator {
    pub fn new(operation: TernaryRasterOperation, x: i16, y: i16, height: i16, width: i16) -> Self {
        Self {
            operation,
            x,
            y,
            height,
            width,
            brush: None,
            source: None,
        }
    }

    /// [#6140] 목적 사각형이 음수 폭/높이로 온 경우의 SVG 정규화.
    ///
    /// GDI 는 `dest_height`/`dest_width` 가 음수면 그 축으로 뒤집어 blit 한다.
    /// SVG 의 `width`/`height` 는 **음수를 오류로 규정**하므로 그대로 내보내면
    /// 브라우저가 그 속성을 무시하고(=절대값·비반전) 그린다 — bottom-up WMF 가
    /// 이미 y-flip 그룹 안에 있으면 그 결과가 상하 반전이 된다(156462405 7쪽
    /// 인물 사진). 원점·크기를 양수로 정규화하고, 뒤집힘은 요소 자신의
    /// `transform` 으로 표현한다.
    fn normalized_rect(&self) -> (i32, i32, i32, i32, Option<String>) {
        let (x, y) = (i32::from(self.x), i32::from(self.y));
        let (w, h) = (i32::from(self.width), i32::from(self.height));
        let x_norm = if w < 0 { x + w } else { x };
        let y_norm = if h < 0 { y + h } else { y };
        let w_abs = w.abs();
        let h_abs = h.abs();
        let mut parts = Vec::new();
        if w < 0 {
            parts.push(format!("translate({},0) scale(-1,1)", 2 * x_norm + w_abs));
        }
        if h < 0 {
            parts.push(format!("translate(0,{}) scale(1,-1)", 2 * y_norm + h_abs));
        }
        let transform = (!parts.is_empty()).then(|| parts.join(" "));
        (x_norm, y_norm, w_abs, h_abs, transform)
    }

    pub fn brush(mut self, brush: Brush) -> Self {
        self.brush = brush.into();
        self
    }

    pub fn source_bitmap16(mut self, source: Bitmap16) -> Self {
        self.source = Source::Bitmap16(source).into();
        self
    }

    pub fn source_bitmap(mut self, source: DeviceIndependentBitmap) -> Self {
        self.source = Source::Bitmap(source).into();
        self
    }

    pub fn run(
        self,
        definitions: &mut Vec<Node>,
    ) -> Result<Option<Node>, TernaryRasterOperationError> {
        if self.operation.use_selected_brush() && self.brush.is_none() {
            return Err(TernaryRasterOperationError::NoBrush {
                cause: format!(
                    "TernaryRasterOperation {:?} cannot access brush.",
                    self.operation,
                ),
            });
        }

        if self.operation.use_source() && self.source.is_none() {
            return Err(TernaryRasterOperationError::NoSource {
                cause: format!(
                    "TernaryRasterOperation {:?} cannot access source bitmap.",
                    self.operation,
                ),
            });
        }

        let result: Node = match self.operation {
            TernaryRasterOperation::BLACKNESS => Node::new("rect")
                .set("x", self.x)
                .set("y", self.y)
                .set("width", self.width)
                .set("height", self.height)
                .set("stroke", "none")
                .set("fill", "black"),
            TernaryRasterOperation::SRCCOPY => {
                let (x, y, width, height, transform) = self.normalized_rect();
                let bitmap = match self.source.unwrap() {
                    Source::Bitmap16(data) => {
                        let bitmap = crate::wmf::parser::DeviceIndependentBitmap::from(data);
                        crate::wmf::converter::Bitmap::from(bitmap)
                    }
                    Source::Bitmap(data) => Bitmap::from(data),
                };

                let image = Node::new("image")
                    .set("x", x)
                    .set("y", y)
                    .set("width", width)
                    .set("height", height)
                    .set("href", bitmap.as_data_url());
                match transform {
                    Some(transform) => image.set("transform", transform),
                    None => image,
                }
            }
            TernaryRasterOperation::PATCOPY => {
                let fill = match Fill::from(self.brush.clone().unwrap()) {
                    Fill::Pattern { pattern } => {
                        let id = Self::issue_id(definitions);
                        definitions.push(pattern.set("id", id.as_str()));
                        url_string(format!("#{id}").as_str())
                    }
                    Fill::Value { value } => value,
                };

                Node::new("rect")
                    .set("x", self.x)
                    .set("y", self.y)
                    .set("width", self.width)
                    .set("height", self.height)
                    .set("fill", fill.as_str())
            }
            TernaryRasterOperation::WHITENESS => Node::new("rect")
                .set("x", self.x)
                .set("y", self.y)
                .set("width", self.width)
                .set("height", self.height)
                .set("stroke", "none")
                .set("fill", "white"),
            operation => {
                info!(?operation, "TernaryRasterOperation is not implemented");

                return Ok(None);
            }
        };

        Ok(Some(result))
    }

    #[inline]
    fn issue_id(definitions: &[Node]) -> String {
        format!("rop_pat{}", definitions.len())
    }
}

impl From<ColorRef> for RGBQuad {
    fn from(v: ColorRef) -> Self {
        let ColorRef {
            red,
            green,
            blue,
            reserved,
        } = v;
        Self {
            red,
            green,
            blue,
            reserved,
        }
    }
}
