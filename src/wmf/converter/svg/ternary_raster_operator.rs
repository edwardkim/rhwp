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

type BrushOnlyRopKey = (i16, i16, i16, i16, String);

#[derive(Default)]
pub struct BrushOnlyRopSequence {
    state: Option<BrushOnlyRopSequenceState>,
}

enum BrushOnlyRopSequenceState {
    AwaitDpa {
        key: BrushOnlyRopKey,
        expected_element_count: usize,
    },
    AwaitFinalPatInvert {
        key: BrushOnlyRopKey,
        expected_element_count: usize,
    },
}

impl BrushOnlyRopSequence {
    /// Returns true only for the contiguous fallback sequence
    /// PATINVERT(key) -> DPA -> PATINVERT(key).
    fn observe(
        &mut self,
        operation: TernaryRasterOperation,
        key: BrushOnlyRopKey,
        element_count: usize,
    ) -> bool {
        let state = std::mem::take(&mut self.state);
        match (state, operation) {
            (
                Some(BrushOnlyRopSequenceState::AwaitDpa {
                    key: previous_key,
                    expected_element_count,
                }),
                TernaryRasterOperation::DPA,
            ) if element_count == expected_element_count => {
                self.state = Some(BrushOnlyRopSequenceState::AwaitFinalPatInvert {
                    key: previous_key,
                    expected_element_count: element_count + 1,
                });
                false
            }
            (
                Some(BrushOnlyRopSequenceState::AwaitFinalPatInvert {
                    key: previous_key,
                    expected_element_count,
                }),
                TernaryRasterOperation::PATINVERT,
            ) if key == previous_key && element_count == expected_element_count => true,
            (_, TernaryRasterOperation::PATINVERT) => {
                self.state = Some(BrushOnlyRopSequenceState::AwaitDpa {
                    key,
                    expected_element_count: element_count + 1,
                });
                false
            }
            _ => false,
        }
    }

    fn clear_if_unrelated(&mut self, operation: TernaryRasterOperation) {
        if !matches!(
            operation,
            TernaryRasterOperation::PATINVERT | TernaryRasterOperation::DPA
        ) {
            self.state = None;
        }
    }
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
        brush_only_rop_sequence: &mut BrushOnlyRopSequence,
        element_count: usize,
    ) -> Result<Option<Node>, TernaryRasterOperationError> {
        brush_only_rop_sequence.clear_if_unrelated(self.operation);

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
            // [#6469] 미구현 ROP 을 **통째로 버리지 않는다.**
            //
            // 종전에는 여기서 `Ok(None)` 을 돌려주고 호출부가 레코드를 흔적 없이
            // 지웠다 — 156627451 2쪽 도해의 옅은 회색 패널이 그렇게 사라졌다.
            // 그 패널은 소스 없는 `DibBitBlt` 세 개가 `PATINVERT → DPa → PATINVERT`
            // 로 그리는데, 흰 바탕에서 이 조합의 최종 결과는 **브러시 색 자체**다.
            //
            //   0xFFFFFF ⊕ 0xD9D9D9 = 0x262626
            //   0x262626 ∧ 0xD9D9D9 = 0x000000
            //   0x000000 ⊕ 0xD9D9D9 = 0xD9D9D9   ← 브러시 색
            //
            // 그래서 **소스를 쓰지 않고 브러시만 쓰는** ROP 은 `PATCOPY` 로 근사한다.
            // 세 번 칠해도 결과가 같아 이 관용구를 정확히 재현하고, 진짜 XOR 하이라이트
            // 처럼 목적이 다른 쓰임은 "아무것도 안 그림"에서 "브러시 색으로 그림"이
            // 되므로 **정보가 줄지 않는다**.
            //
            // 소스를 쓰는 미구현 ROP(`SRCPAINT`·`SRCAND` 등, 투명 blit 관용구)은
            // 원본 그림을 그린다 — 마스크 패스(`SRCAND`)는 겹쳐 그려도 같은 그림이라
            // 시각 결과가 유지된다.
            //
            // **이 갈래는 종전에 아무것도 그리지 않던 경우에만 걸린다** — 이미 그려지던
            // 출력은 하나도 바뀌지 않는다.
            operation if operation.use_selected_brush() && !operation.use_source() => {
                info!(
                    ?operation,
                    "approximating brush-only TernaryRasterOperation as PATCOPY"
                );
                let fill = match Fill::from(self.brush.clone().unwrap()) {
                    Fill::Pattern { pattern } => {
                        let id = Self::issue_id(definitions);
                        definitions.push(pattern.set("id", id.as_str()));
                        url_string(format!("#{id}").as_str())
                    }
                    Fill::Value { value } => value,
                };

                // `PATINVERT`(D ⊕ P)는 확인된 연속 관용구 안에서만 상쇄한다.
                //
                //   PATINVERT(gray) → DPa(패턴) → PATINVERT(gray)
                //
                // 평면 색 근사로 셋 다 칠하면 마지막 XOR 이 가운데 패턴 blit 이 칠한
                // 그림(흰 원)을 덮는다. 다만 전역 이력에서 같은 키를 찾으면 독립된
                // 후속 draw까지 지워질 수 있으므로, 출력 요소 순서상 연속한
                // PATINVERT → DPA → PATINVERT만 상쇄한다.
                let key = (self.x, self.y, self.width, self.height, fill.clone());
                if brush_only_rop_sequence.observe(operation, key, element_count) {
                    return Ok(None);
                }

                Node::new("rect")
                    .set("x", self.x)
                    .set("y", self.y)
                    .set("width", self.width)
                    .set("height", self.height)
                    .set("fill", fill.as_str())
            }
            operation if operation.use_source() => {
                info!(
                    ?operation,
                    "approximating source TernaryRasterOperation as SRCCOPY"
                );
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
