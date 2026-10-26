//! Bounded LRU CPU page snapshots per native session. This cache is optional;
//! misses and rejected retention never change the region pixel contract.

use serde::Serialize;
use std::collections::VecDeque;

use super::{NativeDocumentSession, NativeFrame, NativeHostError, MAX_FRAME_PIXELS};
use crate::renderer::layer_renderer::RasterRenderOptions;
use crate::renderer::raster_pixels::{RasterRegion, RasterRegionPixels};
use crate::renderer::skia::{PreparedSkiaPage, SkiaRegionPlan, SkiaTilePlan};

pub const DEFAULT_REGION_CACHE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_REGION_CACHE_BYTES: usize = 64 * 1024 * 1024;
/// Bound metadata as well as pixel storage, including very small page rasters.
pub const MAX_REGION_CACHE_ENTRIES: usize = 4;

/// Counters accumulate for the session lifetime, including across cache clears.
/// `replays` counts successful full-page rasterizations for region requests.
/// Hits count successful requests served by the matching retained snapshot;
/// `off_page_requests` counts successful clear-only requests on a cold key.
/// Gauges count all retained N32 images, excluding output buffers, renderer
/// resources, layout data and Skia intermediates. This is not an RSS estimate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeRegionCacheStats {
    pub entries: usize,
    pub retained_bytes: usize,
    pub hits: u64,
    pub misses: u64,
    pub replays: u64,
    pub evictions: u64,
    pub invalidations: u64,
    pub bypassed_oversized: u64,
    pub off_page_requests: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct RegionKey {
    revision: u64,
    font_generation: u64,
    page: u32,
    scale: u64,
}

struct RegionEntry {
    key: RegionKey,
    page: PreparedSkiaPage,
}

pub(super) struct NativeRegionCache {
    // Least recently used first. A successful read moves its entry to the back.
    entries: VecDeque<RegionEntry>,
    max_bytes: usize,
    stats: NativeRegionCacheStats,
}

impl Default for NativeRegionCache {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            max_bytes: DEFAULT_REGION_CACHE_BYTES,
            stats: NativeRegionCacheStats::default(),
        }
    }
}

impl NativeRegionCache {
    fn evict_oldest(&mut self) {
        if self.entries.pop_front().is_some() {
            self.stats.evictions = self.stats.evictions.saturating_add(1);
        }
    }

    fn evict_all(&mut self) {
        while !self.entries.is_empty() {
            self.evict_oldest();
        }
    }

    fn retained_bytes(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| entry.page.retained_bytes())
            .sum()
    }

    fn evict_for_allocation(&mut self, allocation_bytes: u64) {
        // The region planner bounds each allocation to 128 MiB, and retained
        // images total at most 64 MiB. Include every image before allocating.
        while !self.entries.is_empty()
            && self.retained_bytes() as u64 + allocation_bytes > SkiaRegionPlan::MAX_BITMAP_BYTES
        {
            self.evict_oldest();
        }
    }

    pub(super) fn invalidate(&mut self) {
        self.evict_all();
        self.stats.invalidations = self.stats.invalidations.saturating_add(1);
    }

    fn stats(&self) -> NativeRegionCacheStats {
        let mut stats = self.stats;
        stats.entries = self.entries.len();
        stats.retained_bytes = self.retained_bytes();
        stats
    }
}

impl NativeDocumentSession {
    /// Configure total retention for up to four completed CPU page/scale
    /// snapshots, from zero (disabled) to 64 MiB. Changing the limit evicts all
    /// entries before returning. Rejected values leave the existing limit,
    /// snapshots and counters intact.
    /// The per-request 128 MiB bitmap peak remains independent of this limit.
    pub fn set_region_cache_limit(&self, max_bytes: usize) -> Result<(), NativeHostError> {
        if max_bytes > MAX_REGION_CACHE_BYTES {
            return Err(NativeHostError::LimitExceeded(
                "native region cache limit exceeds 64 MiB".into(),
            ));
        }
        let mut cache = self.region_cache.borrow_mut();
        if cache.max_bytes != max_bytes {
            cache.invalidate();
            cache.max_bytes = max_bytes;
        }
        Ok(())
    }

    pub fn region_cache_stats(&self) -> NativeRegionCacheStats {
        self.region_cache.borrow().stats()
    }

    pub(super) fn render_region_cached(
        &self,
        page: u32,
        region: RasterRegion,
        scale: f64,
    ) -> Result<NativeFrame, NativeHostError> {
        let options = RasterRenderOptions {
            scale,
            max_pixels: MAX_FRAME_PIXELS,
            transparent: false,
            ..Default::default()
        };
        let key = RegionKey {
            revision: self.revision,
            font_generation: self.font_generation,
            page,
            scale: scale.to_bits(),
        };
        let mut cache = self.region_cache.borrow_mut();
        if let Some(index) = cache.entries.iter().position(|entry| entry.key == key) {
            let plan = cache.entries[index].page.plan_region(region)?;
            cache.evict_for_allocation(plan.output_bytes() as u64);
            if let Some(index) = cache.entries.iter().position(|entry| entry.key == key) {
                let output = cache.entries[index].page.render_region(region)?;
                let entry = cache.entries.remove(index).expect("matching cache entry");
                cache.entries.push_back(entry);
                cache.stats.hits = cache.stats.hits.saturating_add(1);
                return Ok(self.region_frame(output));
            }
            // If the matching image was evicted, preserve the direct API's
            // clear-only allowance without retaining it alongside the output.
        }

        let tree = self
            .core
            .build_page_layer_tree_with_profile(page, crate::paint::RenderProfile::Screen)?;
        let plan = SkiaRegionPlan::new(tree.page_width, tree.page_height, region, options)?;
        cache.stats.misses = cache.stats.misses.saturating_add(1);
        if plan.intersection().is_none() {
            cache.evict_for_allocation(plan.output_bytes() as u64);
            let (output, prepared) = self
                .renderer()
                .prepare_page_for_region(&tree, region, options)?;
            debug_assert!(prepared.is_none());
            cache.stats.off_page_requests = cache.stats.off_page_requests.saturating_add(1);
            return Ok(self.region_frame(output));
        }

        // Reserve room before creating the new surface. Its immutable snapshot
        // shares that allocation, so working_bytes is also its retention cost.
        // Preserve the existing oversized-page bypass: release all old images.
        if plan.working_bytes() > cache.max_bytes as u64 {
            cache.evict_all();
        } else {
            while cache.entries.len() >= MAX_REGION_CACHE_ENTRIES
                || cache.retained_bytes() as u64 + plan.working_bytes() > cache.max_bytes as u64
            {
                cache.evict_oldest();
            }
        }
        cache.evict_for_allocation(plan.peak_bitmap_bytes());
        let (output, prepared) = self
            .renderer()
            .prepare_page_for_region(&tree, region, options)?;
        cache.stats.replays = cache.stats.replays.saturating_add(1);
        if let Some(prepared) = prepared {
            debug_assert_eq!(prepared.retained_bytes() as u64, plan.working_bytes());
            if prepared.retained_bytes() <= cache.max_bytes {
                cache.entries.push_back(RegionEntry {
                    key,
                    page: prepared,
                });
            } else {
                cache.stats.bypassed_oversized = cache.stats.bypassed_oversized.saturating_add(1);
            }
        }
        Ok(self.region_frame(output))
    }

    pub(super) fn render_tiled_region_uncached(
        &self,
        page: u32,
        region: RasterRegion,
        scale: f64,
    ) -> Result<NativeFrame, NativeHostError> {
        let options = RasterRenderOptions {
            scale,
            max_pixels: MAX_FRAME_PIXELS,
            transparent: false,
            ..Default::default()
        };
        let tree = self
            .core
            .build_page_layer_tree_with_profile(page, crate::paint::RenderProfile::Screen)?;
        let plan = SkiaTilePlan::new(tree.page_width, tree.page_height, region, options)?;
        // Include retained full-page snapshots in the new request's bitmap peak.
        self.region_cache
            .borrow_mut()
            .evict_for_allocation(plan.peak_bitmap_bytes());
        let output = self
            .renderer()
            .render_tiled_region_pixels_with_options(&tree, region, options)?;
        Ok(self.region_frame(output))
    }

    fn region_frame(&self, output: RasterRegionPixels) -> NativeFrame {
        NativeFrame {
            pixels: output.pixels,
            revision: self.revision,
            font_generation: self.font_generation,
            origin_x: output.origin_x,
            origin_y: output.origin_y,
        }
    }
}
