//! Renderer-local, bounded replay resources. Keys never outlive this process.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::{self, Debug, Write};

use super::diagnostics::ReplayDiagnostics;
use super::glyph_replay::gradient_stop_byte_cost;
use super::replay_policy::ReplayPolicy;
use crate::paint::{
    ColorPaintGraphNodeKind, ColorPaintGraphPayload, LayerNode, LayerNodeKind, PageLayerTree,
    PaintOp, ResourceArena, LAYER_TREE_SCHEMA,
};

pub(super) type CacheKey = [u8; 32];
const MAX_KEY_BYTES: usize = 8 * 1024 * 1024;
const MAX_RESOURCE_HASH_BYTES: usize = 256 * 1024 * 1024;
pub(super) const DEFAULT_CACHE_ENTRIES: usize = 64;
pub(super) const DEFAULT_CACHE_BYTES: usize = 32 * 1024 * 1024;

/// Cumulative lookups and current retained estimates for one native cache.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeSkiaCacheUsage {
    pub entries: usize,
    pub approximate_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub skipped_oversized: u64,
}

/// Native cache diagnostics are observational and do not change replay policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeSkiaCacheStats {
    pub pictures: NativeSkiaCacheUsage,
    pub glyphs: NativeSkiaCacheUsage,
    pub images: NativeSkiaCacheUsage,
}

pub(super) struct BoundedCache<V> {
    entries: HashMap<CacheKey, (V, usize)>,
    order: VecDeque<CacheKey>,
    max_entries: usize,
    max_bytes: usize,
    usage: NativeSkiaCacheUsage,
}

impl<V: Clone> BoundedCache<V> {
    pub fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            max_entries,
            max_bytes,
            usage: NativeSkiaCacheUsage::default(),
        }
    }

    pub fn get(&mut self, key: CacheKey) -> Option<V> {
        let Some((value, _)) = self.entries.get(&key) else {
            self.usage.misses = self.usage.misses.saturating_add(1);
            return None;
        };
        let value = value.clone();
        self.order.retain(|entry| *entry != key);
        self.order.push_back(key);
        self.usage.hits = self.usage.hits.saturating_add(1);
        Some(value)
    }

    pub fn insert(&mut self, key: CacheKey, value: V, bytes: usize) {
        self.insert_observed(key, value, bytes, |_, _| {});
    }

    pub fn insert_observed(
        &mut self,
        key: CacheKey,
        value: V,
        bytes: usize,
        mut on_eviction: impl FnMut(&V, usize),
    ) -> bool {
        let Some(bytes) = bytes.checked_add(std::mem::size_of::<(CacheKey, V, usize)>()) else {
            self.usage.skipped_oversized = self.usage.skipped_oversized.saturating_add(1);
            return false;
        };
        if self.max_entries == 0 || bytes > self.max_bytes {
            self.usage.skipped_oversized = self.usage.skipped_oversized.saturating_add(1);
            return false;
        }
        if let Some((_, old_bytes)) = self.entries.remove(&key) {
            self.usage.approximate_bytes -= old_bytes;
            self.order.retain(|entry| *entry != key);
        }
        while self.entries.len() >= self.max_entries
            || self
                .usage
                .approximate_bytes
                .checked_add(bytes)
                .is_none_or(|total| total > self.max_bytes)
        {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some((old_value, old_bytes)) = self.entries.remove(&oldest) {
                on_eviction(&old_value, old_bytes);
                self.usage.approximate_bytes -= old_bytes;
                self.usage.evictions = self.usage.evictions.saturating_add(1);
            }
        }
        self.entries.insert(key, (value, bytes));
        self.order.push_back(key);
        self.usage.approximate_bytes += bytes;
        self.usage.entries = self.entries.len();
        true
    }

    pub fn matching_usage(&self, predicate: impl Fn(&V) -> bool) -> (usize, usize) {
        self.entries
            .values()
            .filter(|(value, _)| predicate(value))
            .fold((0usize, 0usize), |(entries, bytes), (_, cost)| {
                (entries.saturating_add(1), bytes.saturating_add(*cost))
            })
    }

    pub fn usage(&self) -> NativeSkiaCacheUsage {
        self.usage
    }

    pub fn enabled(&self) -> bool {
        self.max_entries != 0 && self.max_bytes != 0
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.usage = NativeSkiaCacheUsage::default();
    }
}

/// Formatting is streamed directly into BLAKE3, without materializing a giant
/// key string. Derived Debug includes all private/new paint fields and preserves
/// round-trip floating precision; unlike output JSON it does not round geometry.
/// This is deliberately not a persistent or cross-version serialization format.
struct KeyWriter {
    hasher: blake3::Hasher,
    remaining: usize,
}

impl Write for KeyWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.remaining = self.remaining.checked_sub(value.len()).ok_or(fmt::Error)?;
        self.hasher.update(value.as_bytes());
        Ok(())
    }
}

pub(super) fn debug_key(value: &impl Debug) -> Option<CacheKey> {
    bounded_debug(value).map(|(key, _)| key)
}

pub(super) fn debug_size(value: &impl Debug) -> Option<usize> {
    bounded_debug(value).map(|(_, bytes)| bytes)
}

fn bounded_debug(value: &impl Debug) -> Option<(CacheKey, usize)> {
    let mut writer = KeyWriter {
        hasher: blake3::Hasher::new(),
        remaining: MAX_KEY_BYTES,
    };
    write!(&mut writer, "{value:?}").ok()?;
    Some((
        *writer.hasher.finalize().as_bytes(),
        MAX_KEY_BYTES - writer.remaining,
    ))
}

pub(super) fn byte_key(kind: &[u8], bytes: &[u8], parameters: &impl Debug) -> Option<CacheKey> {
    if bytes.len() > MAX_RESOURCE_HASH_BYTES {
        return None;
    }
    let parameters = debug_key(parameters)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(kind.len() as u64).to_le_bytes());
    hasher.update(kind);
    hasher.update(&parameters);
    hasher.update(bytes);
    Some(*hasher.finalize().as_bytes())
}

pub(super) fn inline_resource_estimate(node: &LayerNode, scale: f32) -> usize {
    let mut pending = vec![node];
    let mut bytes = 0usize;
    while let Some(node) = pending.pop() {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => pending.extend(children),
            LayerNodeKind::ClipRect { child, .. } => pending.push(child),
            LayerNodeKind::Leaf { ops } => {
                for op in ops {
                    let payload = match op {
                        PaintOp::GlyphOutline { bbox, outline } => {
                            if let Some(graph) = outline
                                .color_layers
                                .as_ref()
                                .and_then(|payload| payload.paint_graph.as_ref())
                            {
                                let Some(total) = gradient_graph_byte_cost(graph)
                                    .and_then(|cost| bytes.checked_add(cost))
                                else {
                                    return usize::MAX;
                                };
                                bytes = total;
                            }
                            if let Some(svg) = &outline.svg_glyph {
                                let viewport = svg.font_em_box.unwrap_or(*bbox);
                                let width = (viewport.width * f64::from(scale)).ceil();
                                let height = (viewport.height * f64::from(scale)).ceil();
                                let raster = (width * height * 4.0) as usize;
                                // A font em-box can exceed the page. A Picture
                                // retains its prepared raster even if the glyph
                                // cache rejects that same oversized object.
                                bytes = bytes.saturating_add(raster);
                            }
                            None
                        }
                        PaintOp::Image {
                            bbox,
                            image,
                            resolved,
                        } => resolved
                            .as_ref()
                            .map(|resolved| resolved.data.as_slice())
                            .or(image.data.as_deref())
                            .map(|data| (*bbox, data)),
                        PaintOp::PageBackground { bbox, background } => background
                            .image
                            .as_ref()
                            .map(|image| (*bbox, image.data.as_slice())),
                        PaintOp::RawSvg { bbox, raw } => Some((*bbox, raw.svg.as_bytes())),
                        _ => None,
                    };
                    if let Some((bbox, data)) = payload {
                        let raster = (bbox.width * bbox.height * f64::from(scale).powi(2) * 4.0)
                            .abs() as usize;
                        bytes = bytes.saturating_add(if matches!(op, PaintOp::RawSvg { .. }) {
                            data.len().saturating_add(raster)
                        } else {
                            decoded_resource_cost(data, raster)
                        });
                    }
                }
            }
        }
    }
    bytes
}

// A Picture retains every shader constructed by expanded replay. Shared DAG
// nodes therefore cost once per visit, not once per stored graph node.
fn gradient_graph_byte_cost(graph: &ColorPaintGraphPayload) -> Option<usize> {
    if !graph.has_colrv1_supported_graph_contract() {
        return None;
    }
    let mut pending = vec![graph.root_node_id];
    let mut visits = 0usize;
    let mut bytes = 0usize;
    while let Some(id) = pending.pop() {
        visits += 1;
        if visits > 4096 {
            return None;
        }
        let node = graph.nodes.iter().find(|node| node.node_id == id)?;
        let stop_count = match node.kind {
            ColorPaintGraphNodeKind::Transform => {
                pending.push(node.transform.as_ref()?.child_node_id);
                continue;
            }
            ColorPaintGraphNodeKind::Clip => {
                pending.push(node.clip.as_ref()?.child_node_id);
                continue;
            }
            ColorPaintGraphNodeKind::Composite => {
                let composite = node.composite.as_ref()?;
                pending.push(composite.source_node_id);
                pending.push(composite.backdrop_node_id);
                continue;
            }
            ColorPaintGraphNodeKind::SolidPath => continue,
            ColorPaintGraphNodeKind::LinearGradientPath => {
                node.linear_gradient_path.as_ref()?.gradient.stops.len()
            }
            ColorPaintGraphNodeKind::RadialGradientPath => {
                node.radial_gradient_path.as_ref()?.gradient.stops.len()
            }
            ColorPaintGraphNodeKind::SweepGradientPath => {
                node.sweep_gradient_path.as_ref()?.gradient.stops.len()
            }
        };
        bytes = bytes.checked_add(gradient_stop_byte_cost(stop_count)?)?;
    }
    Some(bytes)
}

fn decoded_resource_cost(bytes: &[u8], svg_estimate: usize) -> usize {
    let raster_bytes = if crate::renderer::svg_fragment::is_svg_prefix(bytes) {
        svg_estimate
    } else {
        skia_safe::Image::from_encoded(skia_safe::Data::new_copy(bytes)).map_or(0, |image| {
            (image.width().max(0) as usize)
                .saturating_mul(image.height().max(0) as usize)
                .saturating_mul(4)
        })
    };
    bytes.len().saturating_add(raster_bytes)
}

#[derive(Clone, Copy)]
pub(super) struct ReplayCacheContext<'a> {
    pub seed: Option<CacheKey>,
    pub page_bounds: skia_safe::Rect,
    pub retained_resource_estimate: usize,
    pub policy: ReplayPolicy,
    pub diagnostics: &'a RefCell<ReplayDiagnostics>,
}

impl<'a> ReplayCacheContext<'a> {
    pub fn new(
        tree: &PageLayerTree,
        scale: f32,
        strict: bool,
        diagnostics: &'a RefCell<ReplayDiagnostics>,
    ) -> Self {
        let resources = resource_identity(&tree.resources);
        let seed = resources.as_ref().and_then(|(digest, _)| {
            debug_key(&(
                "native-skia-replay-v1",
                LAYER_TREE_SCHEMA,
                tree.profile,
                &tree.output_options,
                tree.page_width,
                tree.page_height,
                scale,
                strict,
                digest,
            ))
        });
        let pixels = tree.page_width * tree.page_height * f64::from(scale).powi(2);
        // Picture's own estimate excludes retained images/fonts. Include the
        // arena's decoded image estimates and a viewport per SVG resource.
        let svg_estimate = (pixels * 4.0 * tree.resources.svg_count() as f64) as usize;
        Self {
            seed,
            page_bounds: skia_safe::Rect::from_wh(tree.page_width as f32, tree.page_height as f32),
            retained_resource_estimate: resources
                .map_or(usize::MAX, |(_, bytes)| bytes)
                .saturating_add(svg_estimate),
            policy: ReplayPolicy::new(tree.profile),
            diagnostics,
        }
    }

    pub fn with_hint(mut self, hint: crate::paint::CacheHint) -> Self {
        self.policy = self.policy.with_hint(hint);
        self
    }
}

fn resource_identity(resources: &ResourceArena) -> Option<(CacheKey, usize)> {
    let metadata = debug_key(&(resources.source_image_epoch(), resources.font_resources()))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&metadata);
    let mut hashed_bytes = 0usize;
    let mut retained_bytes = 0usize;
    let mut mix = |kind: u8, id: usize, bytes: &[u8]| -> Option<()> {
        hashed_bytes = hashed_bytes.checked_add(bytes.len())?;
        if hashed_bytes > MAX_RESOURCE_HASH_BYTES {
            return None;
        }
        hasher.update(&[kind]);
        hasher.update(&(id as u64).to_le_bytes());
        hasher.update(&(bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
        Some(())
    };
    for (id, bytes) in resources.image_resources() {
        mix(0, id.0, bytes)?;
        retained_bytes = retained_bytes.saturating_add(decoded_resource_cost(bytes, 0));
    }
    for (id, fragment) in resources.svg_resources() {
        mix(1, id.0, fragment.as_bytes())?;
        retained_bytes = retained_bytes.saturating_add(fragment.len());
    }
    for (id, bytes) in resources.font_blob_resources() {
        mix(2, id.0, bytes)?;
        retained_bytes = retained_bytes.saturating_add(bytes.len());
    }
    Some((*hasher.finalize().as_bytes(), retained_bytes))
}
