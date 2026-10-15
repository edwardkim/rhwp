//! Platform-neutral continuous page viewing; no document layout is changed.
//!
//! Page sizes/positions use the engine's 96-DPI logical page coordinates.
//! Viewport, scroll and page gaps use host logical coordinates (DIPs/CSS pixels).
//! Zoom maps page to host units; device scale maps host units to physical pixels.
//! Pages are horizontally centered in the widest page/viewport and stacked from
//! y=0 with no outer padding. Gaps remain constant in host units when zooming.

use serde::{Deserialize, Serialize};

use super::NativeHostError;

pub const MAX_VIEW_PAGES: usize = 10_000;
/// Bound accumulated logical extents, not an allocation or raster-size budget.
pub const MAX_VIEW_EXTENT: f64 = 1_099_511_627_776.0; // 2^40
/// Matches the native session's maximum combined zoom/device scale.
pub const MAX_VIEW_RASTER_SCALE: f64 = 64.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeViewSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeViewPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeViewRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeViewConfig {
    /// Empty documents are valid. Supplied dimensions must be finite and positive.
    #[serde(deserialize_with = "deserialize_pages")]
    pub pages: Vec<NativeViewSize>,
    pub viewport: NativeViewSize,
    pub zoom: f64,
    pub device_scale: f64,
    /// Finite, nonnegative host logical gap between pages, including at zoom != 1.
    pub page_gap: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeViewState {
    pub config: NativeViewConfig,
    /// Host logical scroll offset. Construction/deserialization clamps this.
    pub scroll: NativeViewPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePagePoint {
    pub page: u32,
    /// Unscaled page logical coordinates.
    pub point: NativeViewPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeVisiblePage {
    pub page: u32,
    /// Whole zoomed page in viewport logical coordinates; may be off-screen.
    pub page_rect: NativeViewRect,
    /// Positive-area intersection with the viewport, in host logical units.
    pub viewport_rect: NativeViewRect,
    /// Unscaled page logical region suitable for the P52 region API.
    pub page_region: NativeViewRect,
    /// Physical pixels per page logical unit. Raster allocation budgets still apply.
    pub raster_scale: f64,
}

/// Validated, derived view state. Serialization contains only inputs and scroll;
/// deserialization validates inputs, rebuilds page bounds and clamps scroll.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", try_from = "NativeViewState")]
pub struct NativeView {
    #[serde(flatten)]
    state: NativeViewState,
    #[serde(skip)]
    page_bounds: Vec<NativeViewRect>,
    #[serde(skip)]
    content_size: NativeViewSize,
}

impl TryFrom<NativeViewState> for NativeView {
    type Error = NativeHostError;

    fn try_from(state: NativeViewState) -> Result<Self, Self::Error> {
        let mut view = Self::new(state.config)?;
        view.set_scroll(state.scroll)?;
        Ok(view)
    }
}

impl NativeView {
    pub fn new(config: NativeViewConfig) -> Result<Self, NativeHostError> {
        let (page_bounds, content_size) = layout(&config)?;
        Ok(Self {
            state: NativeViewState {
                config,
                scroll: NativeViewPoint { x: 0.0, y: 0.0 },
            },
            page_bounds,
            content_size,
        })
    }

    pub fn state(&self) -> &NativeViewState {
        &self.state
    }

    pub fn content_size(&self) -> NativeViewSize {
        self.content_size
    }

    pub fn scroll_limit(&self) -> NativeViewPoint {
        NativeViewPoint {
            x: (self.content_size.width - self.state.config.viewport.width).max(0.0),
            y: (self.content_size.height - self.state.config.viewport.height).max(0.0),
        }
    }

    pub fn raster_scale(&self) -> f64 {
        self.state.config.zoom * self.state.config.device_scale
    }

    /// Replace page sizes/viewport/DPR without changing document layout. Retains
    /// the top-left scroll offset where possible; use zoom_at for anchored zoom.
    /// Validation failure leaves the complete previous state unchanged.
    pub fn set_config(&mut self, config: NativeViewConfig) -> Result<bool, NativeHostError> {
        let mut next = Self::new(config)?;
        next.set_scroll(self.state.scroll)?;
        let changed = next.state != self.state;
        *self = next;
        Ok(changed)
    }

    /// Finite offsets outside the content are clamped, including negative values.
    pub fn set_scroll(&mut self, scroll: NativeViewPoint) -> Result<bool, NativeHostError> {
        finite_point(scroll)?;
        let limit = self.scroll_limit();
        let scroll = NativeViewPoint {
            x: scroll.x.clamp(0.0, limit.x),
            y: scroll.y.clamp(0.0, limit.y),
        };
        let changed = scroll != self.state.scroll;
        self.state.scroll = scroll;
        Ok(changed)
    }

    pub fn scroll_by(&mut self, delta: NativeViewPoint) -> Result<bool, NativeHostError> {
        finite_point(delta)?;
        self.set_scroll(NativeViewPoint {
            x: self.state.scroll.x + delta.x,
            y: self.state.scroll.y + delta.y,
        })
    }

    /// Keep the page-local point under a viewport pointer fixed, except where
    /// scroll bounds prevent it. In a gap/margin, use the nearest page by vertical
    /// distance (earlier page on a tie) and its extended local coordinate system.
    /// The pointer must be inside the closed viewport; empty views have no anchor.
    pub fn zoom_at(
        &mut self,
        zoom: f64,
        pointer: NativeViewPoint,
    ) -> Result<bool, NativeHostError> {
        finite_point(pointer)?;
        let viewport = self.state.config.viewport;
        if pointer.x < 0.0
            || pointer.y < 0.0
            || pointer.x > viewport.width
            || pointer.y > viewport.height
        {
            return Err(invalid("zoom pointer must be inside the viewport"));
        }
        if zoom == self.state.config.zoom {
            return Ok(false);
        }
        let mut config = self.state.config.clone();
        config.zoom = zoom;
        let mut next = Self::new(config)?;
        let content_pointer = NativeViewPoint {
            x: pointer.x + self.state.scroll.x,
            y: pointer.y + self.state.scroll.y,
        };
        let nearest = self
            .page_bounds
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                vertical_distance(a, content_pointer.y)
                    .total_cmp(&vertical_distance(b, content_pointer.y))
            });
        if let Some((page, old)) = nearest {
            let new = next.page_bounds[page];
            let ratio = zoom / self.state.config.zoom;
            let scroll = NativeViewPoint {
                x: new.x + (content_pointer.x - old.x) * ratio - pointer.x,
                y: new.y + (content_pointer.y - old.y) * ratio - pointer.y,
            };
            next.set_scroll(scroll)?;
        }
        let changed = next.state != self.state;
        *self = next;
        Ok(changed)
    }

    /// Hit testing excludes the right/bottom edges and inter-page gaps. A finite
    /// pointer outside the viewport yields None, useful for host drag handling.
    pub fn viewport_to_page(
        &self,
        pointer: NativeViewPoint,
    ) -> Result<Option<NativePagePoint>, NativeHostError> {
        finite_point(pointer)?;
        let viewport = self.state.config.viewport;
        if pointer.x < 0.0
            || pointer.y < 0.0
            || pointer.x >= viewport.width
            || pointer.y >= viewport.height
        {
            return Ok(None);
        }
        let point = NativeViewPoint {
            x: pointer.x + self.state.scroll.x,
            y: pointer.y + self.state.scroll.y,
        };
        Ok(self
            .page_bounds
            .iter()
            .enumerate()
            .find_map(|(page, rect)| {
                if point.x < rect.x
                    || point.x >= rect.x + rect.width
                    || point.y < rect.y
                    || point.y >= rect.y + rect.height
                {
                    return None;
                }
                let local = NativeViewPoint {
                    x: (point.x - rect.x) / self.state.config.zoom,
                    y: (point.y - rect.y) / self.state.config.zoom,
                };
                let size = self.state.config.pages[page];
                // Adding a tiny page extent to a large origin can round its far
                // edge outward. Never return a point beyond the original page.
                (local.x < size.width && local.y < size.height).then_some(NativePagePoint {
                    page: page as u32,
                    point: local,
                })
            }))
    }

    /// Closed page bounds permit caret positions at the page's right/bottom edge.
    /// The returned point may be outside the viewport for an off-screen page.
    pub fn page_to_viewport(
        &self,
        position: NativePagePoint,
    ) -> Result<NativeViewPoint, NativeHostError> {
        finite_point(position.point)?;
        let (size, rect) = self.page(position.page)?;
        if position.point.x < 0.0
            || position.point.y < 0.0
            || position.point.x > size.width
            || position.point.y > size.height
        {
            return Err(invalid("point must be inside the page"));
        }
        Ok(NativeViewPoint {
            x: rect.x + position.point.x * self.state.config.zoom - self.state.scroll.x,
            y: rect.y + position.point.y * self.state.config.zoom - self.state.scroll.y,
        })
    }

    pub fn visible_regions(&self) -> Vec<NativeVisiblePage> {
        let viewport = self.state.config.viewport;
        let scroll = self.state.scroll;
        let zoom = self.state.config.zoom;
        self.page_bounds
            .iter()
            .enumerate()
            .filter_map(|(page, rect)| {
                let left = rect.x.max(scroll.x);
                let top = rect.y.max(scroll.y);
                let right = (rect.x + rect.width).min(scroll.x + viewport.width);
                let bottom = (rect.y + rect.height).min(scroll.y + viewport.height);
                if right <= left || bottom <= top {
                    return None;
                }
                let size = self.state.config.pages[page];
                // Intersect against original page extents again after inverse
                // scaling, so f64 edge rounding cannot request outside-page ink.
                let page_left = ((left - rect.x) / zoom).clamp(0.0, size.width);
                let page_top = ((top - rect.y) / zoom).clamp(0.0, size.height);
                let page_right = ((right - rect.x) / zoom).clamp(0.0, size.width);
                let page_bottom = ((bottom - rect.y) / zoom).clamp(0.0, size.height);
                if page_right <= page_left || page_bottom <= page_top {
                    return None;
                }
                Some(NativeVisiblePage {
                    page: page as u32,
                    page_rect: NativeViewRect {
                        x: rect.x - scroll.x,
                        y: rect.y - scroll.y,
                        ..*rect
                    },
                    viewport_rect: NativeViewRect {
                        x: left - scroll.x,
                        y: top - scroll.y,
                        width: right - left,
                        height: bottom - top,
                    },
                    page_region: NativeViewRect {
                        x: page_left,
                        y: page_top,
                        width: page_right - page_left,
                        height: page_bottom - page_top,
                    },
                    raster_scale: self.raster_scale(),
                })
            })
            .collect()
    }

    /// Reveal an engine caret rectangle, with a host logical margin. Width may
    /// be zero; height must be positive and the rect must be inside its page.
    /// Scroll minimally when caret+margin fits; otherwise center that axis.
    /// Page/content bounds can prevent the complete requested margin being shown.
    pub fn ensure_caret_visible(
        &mut self,
        page: u32,
        caret: NativeViewRect,
        margin: f64,
    ) -> Result<bool, NativeHostError> {
        let (size, rect) = self.page(page)?;
        if !margin.is_finite() || margin < 0.0 || margin > MAX_VIEW_EXTENT {
            return Err(invalid(
                "caret margin must be finite and in 0..=MAX_VIEW_EXTENT",
            ));
        }
        finite_point(NativeViewPoint {
            x: caret.x,
            y: caret.y,
        })?;
        if !caret.width.is_finite()
            || !caret.height.is_finite()
            || caret.x < 0.0
            || caret.y < 0.0
            || caret.width < 0.0
            || caret.height <= 0.0
            || caret.x + caret.width > size.width
            || caret.y + caret.height > size.height
            || caret.y + caret.height <= caret.y
            || (caret.width > 0.0 && caret.x + caret.width <= caret.x)
        {
            return Err(invalid(
                "caret must be a finite nonempty-height rectangle inside the page",
            ));
        }
        let zoom = self.state.config.zoom;
        let left = rect.x + caret.x * zoom;
        let top = rect.y + caret.y * zoom;
        let right = rect.x + (caret.x + caret.width) * zoom;
        let bottom = rect.y + (caret.y + caret.height) * zoom;
        if bottom <= top || (caret.width > 0.0 && right <= left) {
            return Err(invalid(
                "caret extent is too small to represent at this zoom/origin",
            ));
        }
        let scroll = self.state.scroll;
        let viewport = self.state.config.viewport;
        self.set_scroll(NativeViewPoint {
            x: reveal_axis(scroll.x, viewport.width, left - margin, right + margin),
            y: reveal_axis(scroll.y, viewport.height, top - margin, bottom + margin),
        })
    }

    fn page(&self, page: u32) -> Result<(NativeViewSize, NativeViewRect), NativeHostError> {
        self.page_bounds
            .get(page as usize)
            .map(|rect| (self.state.config.pages[page as usize], *rect))
            .ok_or_else(|| invalid("page index out of range"))
    }
}

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}

fn finite_point(point: NativeViewPoint) -> Result<(), NativeHostError> {
    if !point.x.is_finite() || !point.y.is_finite() {
        Err(invalid(
            "coordinates must be finite; coordinate arithmetic must not overflow",
        ))
    } else {
        Ok(())
    }
}

fn positive_extent(value: f64) -> Result<(), NativeHostError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(invalid(
            "dimensions and scaled extents must be finite and positive",
        ));
    }
    if value > MAX_VIEW_EXTENT {
        return Err(NativeHostError::LimitExceeded(
            "view extent exceeds 2^40 logical units".into(),
        ));
    }
    Ok(())
}

fn layout(
    config: &NativeViewConfig,
) -> Result<(Vec<NativeViewRect>, NativeViewSize), NativeHostError> {
    if config.pages.len() > MAX_VIEW_PAGES {
        return Err(NativeHostError::LimitExceeded(
            "view page count exceeds 10000".into(),
        ));
    }
    positive_extent(config.viewport.width)?;
    positive_extent(config.viewport.height)?;
    if !config.zoom.is_finite()
        || config.zoom <= 0.0
        || !config.device_scale.is_finite()
        || config.device_scale <= 0.0
    {
        return Err(invalid("zoom and device scale must be finite and positive"));
    }
    let raster_scale = config.zoom * config.device_scale;
    if !raster_scale.is_finite() || raster_scale <= 0.0 || raster_scale > MAX_VIEW_RASTER_SCALE {
        return Err(invalid("zoom * device scale must be finite and in (0, 64]"));
    }
    positive_extent(config.viewport.width * config.device_scale)?;
    positive_extent(config.viewport.height * config.device_scale)?;
    if !config.page_gap.is_finite() || config.page_gap < 0.0 || config.page_gap > MAX_VIEW_EXTENT {
        return Err(invalid(
            "page gap must be finite and in 0..=MAX_VIEW_EXTENT",
        ));
    }
    let mut width = config.viewport.width;
    let mut height = 0.0;
    let mut bounds = Vec::with_capacity(config.pages.len());
    for page in &config.pages {
        positive_extent(page.width)?;
        positive_extent(page.height)?;
        let page_width = page.width * config.zoom;
        let page_height = page.height * config.zoom;
        positive_extent(page_width)?;
        positive_extent(page_height)?;
        if !bounds.is_empty() && config.page_gap > 0.0 {
            let next = height + config.page_gap;
            positive_extent(next)?;
            if next <= height {
                return Err(invalid("page gap is too small to represent at this extent"));
            }
            height = next;
        }
        let bottom = height + page_height;
        positive_extent(bottom)?;
        if bottom <= height {
            return Err(invalid(
                "page height is too small to represent at this extent",
            ));
        }
        bounds.push(NativeViewRect {
            x: 0.0,
            y: height,
            width: page_width,
            height: page_height,
        });
        width = width.max(page_width);
        height = bottom;
    }
    for rect in &mut bounds {
        rect.x = (width - rect.width) / 2.0;
        if rect.x + rect.width <= rect.x {
            return Err(invalid(
                "page width is too small to represent at this extent",
            ));
        }
    }
    Ok((bounds, NativeViewSize { width, height }))
}

fn vertical_distance(rect: &NativeViewRect, y: f64) -> f64 {
    (rect.y - y).max(0.0).max(y - (rect.y + rect.height))
}

fn reveal_axis(scroll: f64, viewport: f64, start: f64, end: f64) -> f64 {
    if end - start > viewport {
        start + (end - start - viewport) / 2.0
    } else if start < scroll {
        start
    } else if end > scroll + viewport {
        end - viewport
    } else {
        scroll
    }
}

fn deserialize_pages<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<NativeViewSize>, D::Error> {
    struct Pages;
    impl<'de> serde::de::Visitor<'de> for Pages {
        type Value = Vec<NativeViewSize>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("at most 10000 logical page sizes")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut sequence: A,
        ) -> Result<Self::Value, A::Error> {
            let hint = sequence.size_hint().unwrap_or(0);
            if hint > MAX_VIEW_PAGES {
                return Err(serde::de::Error::custom("view page count exceeds 10000"));
            }
            let mut pages = Vec::with_capacity(hint);
            while let Some(page) = sequence.next_element()? {
                if pages.len() == MAX_VIEW_PAGES {
                    return Err(serde::de::Error::custom("view page count exceeds 10000"));
                }
                pages.push(page);
            }
            Ok(pages)
        }
    }
    deserializer.deserialize_seq(Pages)
}
