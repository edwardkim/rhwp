use crate::wmf::converter::{
    svg::{device_context::BlitDestRect, node::Node, util::url_string, Fill},
    *,
};

#[derive(Clone, Debug, snafu::prelude::Snafu)]
pub enum TernaryRasterOperationError {
    #[snafu(display("no brush specified: {cause}"))]
    NoBrush { cause: String },
    #[snafu(display("no source bitmap specified: {cause}"))]
    NoSource { cause: String },
}

#[derive(PartialEq)]
struct BrushOnlyRopKey {
    rect: [i32; 4],
    fill: String,
    clip_id: Option<String>,
    clip_rect: Option<[i32; 4]>,
}

#[derive(Default)]
pub struct BrushOnlyRopSequence {
    state: Option<BrushOnlyRopSequenceState>,
    suppressed_elements: Vec<usize>,
    vector_masks: Vec<(usize, Node, String, [i32; 4])>,
    bitmap_masks: Vec<(usize, String, [i32; 4])>,
}

enum BrushOnlyRopSequenceState {
    AwaitDpa {
        key: BrushOnlyRopKey,
        expected_element_count: usize,
    },
    AwaitFinalPatInvert {
        key: BrushOnlyRopKey,
        expected_element_count: usize,
        mask_definition_index: Option<usize>,
        mask_rect: [i32; 4],
    },
    AwaitFinalVectorInvert {
        key: BrushOnlyRopKey,
        expected_element_count: usize,
        mask: Node,
    },
}

impl BrushOnlyRopSequence {
    /// 마지막 XOR까지 확인한 뒤에만 중간 연산을 역흑백 마스크로 바꾼다.
    /// 마스크 밖에서는 두 XOR이 상쇄되어 바탕이 남고 검정 부분에는 브러시 색이 남는다.
    fn observe(
        &mut self,
        operation: TernaryRasterOperation,
        key: BrushOnlyRopKey,
        element_count: usize,
        mask_definition_index: Option<usize>,
    ) -> bool {
        let state = std::mem::take(&mut self.state);
        match (state, operation) {
            (
                Some(BrushOnlyRopSequenceState::AwaitFinalVectorInvert {
                    key: previous_key,
                    expected_element_count,
                    mask,
                }),
                TernaryRasterOperation::PATINVERT,
            ) if key == previous_key && element_count == expected_element_count => {
                // (D ⊕ P) ∧ M ⊕ P는 흰 마스크에서 D, 검은 마스크에서 P다.
                // 두 바깥 XOR 사각형 대신 같은 윤곽의 역흑백 마스크만 표시한다.
                self.suppressed_elements.push(element_count - 2);
                self.vector_masks.push((
                    element_count - 1,
                    mask,
                    previous_key.fill,
                    previous_key.rect,
                ));
                true
            }
            (
                Some(BrushOnlyRopSequenceState::AwaitDpa {
                    key: previous_key,
                    expected_element_count,
                }),
                TernaryRasterOperation::DPA,
            ) if element_count == expected_element_count
                && key.clip_id == previous_key.clip_id
                && key.clip_rect == previous_key.clip_rect =>
            {
                // 중간 B 영역은 바깥 XOR의 A와 달라도 된다. 흰 마스크에서는
                // 이전 바탕, 검은 마스크에서는 A 안의 P 또는 A 밖의 검정이 남는다.
                let mask_definition_index =
                    mask_definition_index.filter(|_| key.rect[2] > 0 && key.rect[3] > 0);
                self.state = Some(BrushOnlyRopSequenceState::AwaitFinalPatInvert {
                    key: previous_key,
                    expected_element_count: element_count + 1,
                    mask_definition_index,
                    mask_rect: key.rect,
                });
                false
            }
            (
                Some(BrushOnlyRopSequenceState::AwaitFinalPatInvert {
                    key: previous_key,
                    expected_element_count,
                    mask_definition_index,
                    mask_rect,
                }),
                TernaryRasterOperation::PATINVERT,
            ) if key == previous_key && element_count == expected_element_count => {
                if mask_definition_index.is_some() {
                    // (D ⊕ P) ∧ M ⊕ P는 M=흰색에서 D, M=검정에서 P다.
                    // 첫 단색 칠을 지우고 중간 DPa의 실제 패턴으로 P를 마스킹한다.
                    // 팔레트·비트를 버리면 반투명 패널과 바탕의 색이 달라진다.
                    self.suppressed_elements.push(element_count - 2);
                    self.bitmap_masks
                        .push((element_count - 1, previous_key.fill, mask_rect));
                }
                true
            }
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

    pub fn observe_vector_mask(
        &mut self,
        mask: &Node,
        brush: &Brush,
        draw_mode: Option<BinaryRasterOperation>,
        bounds: [i32; 4],
        clip: (Option<&str>, Option<&Rect>),
        element_count: usize,
    ) {
        let Some(BrushOnlyRopSequenceState::AwaitDpa {
            key,
            expected_element_count,
        }) = self.state.take()
        else {
            return;
        };
        let (clip_id, clip_rect) = clip;
        let clip_rect = clip_rect.map(|r| {
            [
                i32::from(r.left),
                i32::from(r.top),
                i32::from(r.right),
                i32::from(r.bottom),
            ]
        });
        let [x, y, width, height] = key.rect;
        if expected_element_count != element_count
            || draw_mode != Some(BinaryRasterOperation::R2_MASKPEN)
            || !Self::brush_is_monochrome_mask(Some(brush))
            || mask.attr("stroke") != Some("none")
            || !key.fill.starts_with('#')
            || key.clip_id.as_deref() != clip_id
            || key.clip_rect != clip_rect
            || width <= 0
            || height <= 0
            || bounds[0] < x
            || bounds[1] < y
            || i64::from(bounds[2]) > i64::from(x) + i64::from(width)
            || i64::from(bounds[3]) > i64::from(y) + i64::from(height)
        {
            return;
        }
        self.state = Some(BrushOnlyRopSequenceState::AwaitFinalVectorInvert {
            key,
            expected_element_count: element_count + 1,
            mask: mask.clone(),
        });
    }

    pub fn finish(
        self,
        mut definitions: Vec<Node>,
        mut elements: Vec<Node>,
    ) -> (Vec<Node>, Vec<Node>) {
        let bitmap_masks: Vec<_> = self
            .bitmap_masks
            .into_iter()
            .map(|(index, color, rect)| {
                let mask = elements[index]
                    .clone()
                    .set("id", format!("rop_bitmap_source{index}"));
                let paint = elements[index - 1].clone().set("fill", &color);
                (index, mask, color, rect, Some(paint))
            })
            .collect();
        for (kind, (index, mask, color, [x, y, width, height], paint)) in self
            .vector_masks
            .into_iter()
            .map(|(index, mask, color, rect)| ("vector", (index, mask, color, rect, None)))
            .chain(bitmap_masks.into_iter().map(|mask| ("bitmap", mask)))
        {
            let filter_id = format!("rop_{kind}_inverse{}", definitions.len());
            let mask_id = format!("rop_{kind}_mask{}", definitions.len());
            let transfer = Node::new("feComponentTransfer")
                .add(
                    Node::new("feFuncR")
                        .set("type", "table")
                        .set("tableValues", "1 0"),
                )
                .add(
                    Node::new("feFuncG")
                        .set("type", "table")
                        .set("tableValues", "1 0"),
                )
                .add(
                    Node::new("feFuncB")
                        .set("type", "table")
                        .set("tableValues", "1 0"),
                );
            definitions.push(
                Node::new("filter")
                    .set("id", &filter_id)
                    .set("color-interpolation-filters", "sRGB")
                    .add(transfer),
            );
            definitions.push(
                Node::new("mask")
                    .set("id", &mask_id)
                    .set("maskUnits", "userSpaceOnUse")
                    .set("mask-type", "luminance")
                    .set("x", x)
                    .set("y", y)
                    .set("width", width)
                    .set("height", height)
                    .add(mask.set("filter", url_string(format!("#{filter_id}").as_str()))),
            );
            let mask_url = url_string(format!("#{mask_id}").as_str());
            if let Some(paint) = paint {
                // B의 검정과 A의 브러시 색을 먼저 합성한 뒤 마스크를 한 번 적용한다.
                // 개별 칠에 따로 적용하면 보간된 마스크 경계의 alpha가 중복된다.
                // B의 viewport와 같은 viewBox이므로 원문 장치 좌표는 그대로다.
                let black = elements[index].clone().set("fill", "black");
                elements[index] = Node::new("svg")
                    .set("x", x)
                    .set("y", y)
                    .set("width", width)
                    .set("height", height)
                    .set("viewBox", format!("{x} {y} {width} {height}"))
                    .set("mask", &mask_url)
                    .add(black)
                    .add(paint);
            } else {
                elements[index] = elements[index]
                    .clone()
                    .set("fill", color)
                    .set("mask", &mask_url);
            }
        }
        // Indices are recorded in emission order. Do not remove nodes while
        // collecting: doing so would change IDs or invalidate later indices.
        let elements = elements
            .into_iter()
            .enumerate()
            .filter_map(|(index, node)| {
                self.suppressed_elements
                    .binary_search(&index)
                    .is_err()
                    .then_some(node)
            })
            .collect();
        (definitions, elements)
    }

    /// One bit is a palette index, not proof of a black/white mask.
    /// Palette-indexed brushes cannot be classified without resolving the DC.
    fn brush_is_monochrome_mask(brush: Option<&Brush>) -> bool {
        let Some(Brush::DIBPatternPT {
            color_usage: ColorUsage::DIB_RGB_COLORS,
            brush_hatch,
        }) = brush
        else {
            return false;
        };
        if !matches!(
            brush_hatch.dib_header_info.bit_count(),
            BitCount::BI_BITCOUNT_1
        ) {
            return false;
        }
        let colors = match &brush_hatch.colors {
            Colors::RGBQuad(colors) if colors.len() == 2 => [
                [colors[0].red, colors[0].green, colors[0].blue],
                [colors[1].red, colors[1].green, colors[1].blue],
            ],
            Colors::RGBTriple(colors) if colors.len() == 2 => [
                [colors[0].red, colors[0].green, colors[0].blue],
                [colors[1].red, colors[1].green, colors[1].blue],
            ],
            _ => return false,
        };
        matches!(
            colors,
            [[0, 0, 0], [255, 255, 255]] | [[255, 255, 255], [0, 0, 0]]
        )
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
    /// [#6617] 장치 좌표로 정규화한 목적 사각형(`DeviceContext::blit_dest_rect`).
    rect: BlitDestRect,
    brush: Option<Brush>,
    source: Option<Source>,
    clip_id: Option<String>,
    clip_rect: Option<[i32; 4]>,
}

enum Source {
    Bitmap16(Bitmap16),
    Bitmap(DeviceIndependentBitmap),
}

impl TernaryRasterOperator {
    pub fn new(operation: TernaryRasterOperation, rect: BlitDestRect) -> Self {
        Self {
            operation,
            rect,
            brush: None,
            source: None,
            clip_id: None,
            clip_rect: None,
        }
    }

    /// 목적 사각형과, 뒤집힌 축이 있으면 그 축을 되돌리는 `transform`.
    ///
    /// [#6140] SVG 의 `width`/`height` 는 음수를 오류로 규정하므로 사각형은 항상 양수 크기로
    /// 두고 뒤집힘을 요소 자신의 `transform` 으로 표현한다. [#6617] 어느 축이 뒤집히는지는
    /// 논리 폭/높이 부호가 아니라 장치 좌표에서 정한다(`DeviceContext::blit_dest_rect`) —
    /// y-up 창의 음수 높이 DIB 는 뒤집히지 않는다.
    fn normalized_rect(&self) -> (i32, i32, i32, i32, Option<String>) {
        let BlitDestRect {
            x,
            y,
            width,
            height,
            flip_x,
            flip_y,
        } = self.rect;
        let mut parts = Vec::new();
        if flip_x {
            parts.push(format!("translate({},0) scale(-1,1)", 2 * x + width));
        }
        if flip_y {
            parts.push(format!("translate(0,{}) scale(1,-1)", 2 * y + height));
        }
        let transform = (!parts.is_empty()).then(|| parts.join(" "));
        (x, y, width, height, transform)
    }

    pub fn brush(mut self, brush: Brush) -> Self {
        self.brush = brush.into();
        self
    }

    pub fn clip(mut self, clip_id: Option<&str>, clip_rect: Option<&Rect>) -> Self {
        self.clip_id = clip_id.map(str::to_owned);
        // INTERSECTCLIPRECT changes the DC without issuing an SVG clip ID.
        // Track both representations before cancelling any raster draw.
        self.clip_rect = clip_rect.map(|rect| {
            [
                i32::from(rect.left),
                i32::from(rect.top),
                i32::from(rect.right),
                i32::from(rect.bottom),
            ]
        });
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
                .set("x", self.rect.x)
                .set("y", self.rect.y)
                .set("width", self.rect.width)
                .set("height", self.rect.height)
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
                    // WMF blit은 원본 비율과 무관하게 목적 사각형을 채운다.
                    // SVG 기본 meet는 여백을 만들고 비트맵을 가운데로 옮긴다.
                    .set("preserveAspectRatio", "none")
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
                    .set("x", self.rect.x)
                    .set("y", self.rect.y)
                    .set("width", self.rect.width)
                    .set("height", self.rect.height)
                    .set("fill", fill.as_str())
            }
            TernaryRasterOperation::WHITENESS => Node::new("rect")
                .set("x", self.rect.x)
                .set("y", self.rect.y)
                .set("width", self.rect.width)
                .set("height", self.rect.height)
                .set("stroke", "none")
                .set("fill", "white"),
            // 미지원 연산의 칠을 버리지 않는 기존 PATCOPY 폴백이다(#6469).
            // 완전한 XOR → 흑백 DPa → XOR는 상태 기계가 실제 비트맵/벡터
            // 마스크로 확정한다. 세 단색 칠과 같은 연산이라고 가정하지 않는다.
            // 다른 팔레트·불완전한 순서 등은 아직 이 근사 경로에 남아 있다.
            operation if operation.use_selected_brush() && !operation.use_source() => {
                info!(
                    ?operation,
                    "approximating brush-only TernaryRasterOperation as PATCOPY"
                );
                let is_mask = BrushOnlyRopSequence::brush_is_monochrome_mask(self.brush.as_ref());
                let mut mask_definition_index = None;
                let fill = match Fill::from(self.brush.clone().unwrap()) {
                    Fill::Pattern { pattern } => {
                        let id = Self::issue_id(definitions);
                        if is_mask {
                            mask_definition_index = Some(definitions.len());
                        }
                        definitions.push(pattern.set("id", id.as_str()));
                        url_string(format!("#{id}").as_str())
                    }
                    Fill::Value { value } => value,
                };

                // `PATINVERT`(D ⊕ P)는 확인된 연속 관용구 안에서만 상쇄한다.
                //
                //   PATINVERT(gray) → DPa(패턴) → PATINVERT(gray)
                //
                // 출력 요소 순서상 연속한 세 연산만 확정한다. 전역 이력에서
                // 같은 키를 찾아 독립된 후속 칠까지 지우지 않는다.
                let key = BrushOnlyRopKey {
                    rect: [self.rect.x, self.rect.y, self.rect.width, self.rect.height],
                    fill: fill.clone(),
                    clip_id: self.clip_id,
                    clip_rect: self.clip_rect,
                };
                if brush_only_rop_sequence.observe(
                    operation,
                    key,
                    element_count,
                    mask_definition_index,
                ) {
                    return Ok(None);
                }

                Node::new("rect")
                    .set("x", self.rect.x)
                    .set("y", self.rect.y)
                    .set("width", self.rect.width)
                    .set("height", self.rect.height)
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
                    .set("preserveAspectRatio", "none")
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
