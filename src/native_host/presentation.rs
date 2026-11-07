//! Cross-thread presentation inputs and CPU frame leases.
//!
//! No session, Skia renderer/context, or GPU pointer lives in this registry.
//! Expensive replay happens outside locks. A lease pins immutable bytes after
//! its producer handle is released; stale sources reject new acquisition.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

use super::render_snapshot::{NativeRenderSnapshot, NativeRenderSnapshotUsage};
use super::runtime::{NativeRequestId, NativeRuntimeObserver, NativeRuntimeTarget};
use super::NativeHostError;
use crate::renderer::raster_pixels::RasterRegion;

const MAX_OBJECTS: usize = 64;
const MAX_BYTES: usize = 128 * 1024 * 1024;
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NativePresentationId(u64);

impl NativePresentationId {
    pub fn from_raw(value: u64) -> Self {
        Self(value)
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }
}

fn next_id() -> Result<NativePresentationId, NativeHostError> {
    NEXT_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map(NativePresentationId)
        .map_err(|_| NativeHostError::LimitExceeded("presentation handle space exhausted".into()))
}

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}

fn limit(message: &str) -> NativeHostError {
    NativeHostError::LimitExceeded(message.into())
}

fn lock<T>(value: &Mutex<T>) -> Result<MutexGuard<'_, T>, NativeHostError> {
    value
        .lock()
        .map_err(|_| NativeHostError::Unsupported("presentation registry is poisoned".into()))
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePresentationStamp {
    pub document_id: u64,
    pub revision: u64,
    pub font_generation: u64,
    pub view_generation: u64,
    pub device_generation: u64,
    pub device_scale: f64,
}

impl NativePresentationStamp {
    pub fn validate(self) -> Result<(), NativeHostError> {
        if self.document_id == 0
            || self.view_generation == 0
            || self.device_generation == 0
            || !self.device_scale.is_finite()
            || self.device_scale <= 0.0
            || self.device_scale > 16.0
        {
            return Err(invalid(
                "invalid presentation identity, generation, or device scale",
            ));
        }
        Ok(())
    }

    fn target(self) -> NativeRuntimeTarget {
        NativeRuntimeTarget {
            document_id: NativeRequestId::from_raw(self.document_id),
            revision: self.revision,
        }
    }
}

const MAX_COORDINATE: f64 = 16_777_216.0;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePagePlacement {
    pub snapshot: NativePresentationId,
    pub scale: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    pub background_color: Option<u32>,
}

impl NativePagePlacement {
    pub fn validate(self, snapshot: &NativeRenderSnapshot) -> Result<(), NativeHostError> {
        let tree = snapshot.tree();
        let values = [
            self.scale,
            self.origin_x,
            self.origin_y,
            tree.page_width,
            tree.page_height,
            self.origin_x + tree.page_width * self.scale,
            self.origin_y + tree.page_height * self.scale,
        ];
        if self.scale <= 0.0
            || self.scale > 16.0
            || self.scale as f32 <= 0.0
            || tree.page_width <= 0.0
            || tree.page_height <= 0.0
            || values
                .into_iter()
                .any(|value| !value.is_finite() || value.abs() > MAX_COORDINATE)
        {
            return Err(invalid("invalid page placement"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NativePresentationConfig {
    pub max_sources: usize,
    pub max_snapshots: usize,
    pub max_snapshot_bytes: usize,
    pub max_frames: usize,
    pub max_frame_bytes: usize,
    pub max_leases: usize,
    pub max_in_flight_replays: usize,
}

/// Physical presentation pixels, independent of any GL/Linux target type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativePresentationViewport {
    pub width: i32,
    pub height: i32,
}

impl NativePresentationViewport {
    /// Checked `(stride, byte_length)` for an RGBA8 target.
    pub fn rgba_layout(
        self,
        max_dimension: i32,
        max_pixels: usize,
    ) -> Result<(usize, usize), NativeHostError> {
        if self.width <= 0
            || self.height <= 0
            || self.width > max_dimension
            || self.height > max_dimension
        {
            return Err(invalid("invalid presentation viewport size"));
        }
        let pixels = (self.width as usize)
            .checked_mul(self.height as usize)
            .filter(|pixels| *pixels <= max_pixels)
            .ok_or_else(|| limit("presentation viewport pixel quota exceeded"))?;
        let stride = (self.width as usize)
            .checked_mul(4)
            .ok_or_else(|| limit("presentation stride overflow"))?;
        let bytes = pixels
            .checked_mul(4)
            .ok_or_else(|| limit("presentation byte length overflow"))?;
        Ok((stride, bytes))
    }
}

impl Default for NativePresentationConfig {
    fn default() -> Self {
        Self {
            max_sources: 16,
            max_snapshots: 32,
            max_snapshot_bytes: MAX_BYTES,
            max_frames: 32,
            max_frame_bytes: MAX_BYTES,
            max_leases: 64,
            max_in_flight_replays: 4,
        }
    }
}

impl NativePresentationConfig {
    fn validate(self) -> Result<(), NativeHostError> {
        if [
            self.max_sources,
            self.max_snapshots,
            self.max_frames,
            self.max_leases,
        ]
        .into_iter()
        .any(|value| value == 0 || value > MAX_OBJECTS)
            || !(1..=MAX_BYTES).contains(&self.max_snapshot_bytes)
            || !(1..=MAX_BYTES).contains(&self.max_frame_bytes)
            || !(1..=4).contains(&self.max_in_flight_replays)
        {
            return Err(invalid("invalid presentation registry limits"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePresentationSnapshotInfo {
    pub snapshot: NativePresentationId,
    pub source: NativePresentationId,
    pub document_id: u64,
    pub revision: u64,
    pub font_generation: u64,
    pub page: u32,
    pub profile: &'static str,
    pub font_fingerprint: String,
    pub usage: NativeRenderSnapshotUsage,
    pub current: bool,
}

/// Only the CPU variant is implemented. Future GPU descriptors must define
/// backend-specific ownership and fences before a GPU capability is advertised.
#[derive(Debug, Clone, Serialize)]
#[serde(
    tag = "backend",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum NativePresentationDescriptor {
    Cpu {
        width: i32,
        height: i32,
        stride: usize,
        byte_length: usize,
        pixel_format: &'static str,
        color_space: &'static str,
        alpha: &'static str,
        origin: &'static str,
        origin_x: i32,
        origin_y: i32,
        raster_scale: f64,
        device_scale: f64,
    },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePresentationFrameInfo {
    pub frame: NativePresentationId,
    pub source: NativePresentationId,
    pub stamp: NativePresentationStamp,
    pub descriptor: NativePresentationDescriptor,
    pub current: bool,
    pub page_count: usize,
    pub coordinate_space: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePresentationLeaseInfo {
    pub lease: NativePresentationId,
    pub frame: NativePresentationFrameInfo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePresentationStats {
    pub sources: usize,
    pub snapshot_handles: usize,
    pub retained_snapshots: usize,
    pub snapshot_estimated_bytes: usize,
    pub frame_handles: usize,
    pub retained_frames: usize,
    pub frame_bytes: usize,
    pub leases: usize,
    pub in_flight_replays: usize,
}

struct Source {
    id: NativePresentationId,
    observer: NativeRuntimeObserver,
    active: AtomicBool,
    stamp: Mutex<NativePresentationStamp>,
}

impl Source {
    fn document_current(&self, document_id: u64, revision: u64, generation: u64) -> bool {
        self.active.load(Ordering::Acquire)
            && self.observer.is_current(
                NativeRuntimeTarget {
                    document_id: NativeRequestId::from_raw(document_id),
                    revision,
                },
                generation,
            )
    }

    fn current(&self, stamp: NativePresentationStamp) -> Result<bool, NativeHostError> {
        let configured = *lock(&self.stamp)?;
        Ok(configured == stamp
            && self.document_current(stamp.document_id, stamp.revision, stamp.font_generation))
    }
}

#[derive(Default)]
struct Retention {
    snapshots: usize,
    snapshot_bytes: usize,
    frames: usize,
    frame_bytes: usize,
    replays: usize,
}

struct Budget {
    config: NativePresentationConfig,
    retained: Mutex<Retention>,
}

enum AllocationKind {
    Snapshot,
    #[cfg(feature = "native-skia")]
    Frame,
}

/// The owning Arc record keeps this reservation after removal of a handle.
/// Pixel/snapshot fields are declared before it, so payloads drop first.
struct Allocation {
    budget: Arc<Budget>,
    bytes: usize,
    kind: AllocationKind,
    #[cfg(feature = "native-skia")]
    replaying: bool,
}

impl Allocation {
    fn snapshot(budget: &Arc<Budget>, bytes: usize) -> Result<Self, NativeHostError> {
        let mut retained = lock(&budget.retained)?;
        let total = retained
            .snapshot_bytes
            .checked_add(bytes)
            .filter(|total| *total <= budget.config.max_snapshot_bytes)
            .ok_or_else(|| limit("presentation snapshot byte quota exceeded"))?;
        if retained.snapshots >= budget.config.max_snapshots {
            return Err(limit("presentation snapshot count quota exceeded"));
        }
        retained.snapshots += 1;
        retained.snapshot_bytes = total;
        Ok(Self {
            budget: Arc::clone(budget),
            bytes,
            kind: AllocationKind::Snapshot,
            #[cfg(feature = "native-skia")]
            replaying: false,
        })
    }

    #[cfg(feature = "native-skia")]
    fn frame(budget: &Arc<Budget>, bytes: usize) -> Result<Self, NativeHostError> {
        let mut retained = lock(&budget.retained)?;
        let total = retained
            .frame_bytes
            .checked_add(bytes)
            .filter(|total| *total <= budget.config.max_frame_bytes)
            .ok_or_else(|| limit("presentation CPU frame byte quota exceeded"))?;
        if retained.frames >= budget.config.max_frames
            || retained.replays >= budget.config.max_in_flight_replays
        {
            return Err(limit(
                "presentation CPU frame or replay count quota exceeded",
            ));
        }
        retained.frames += 1;
        retained.frame_bytes = total;
        retained.replays += 1;
        Ok(Self {
            budget: Arc::clone(budget),
            bytes,
            kind: AllocationKind::Frame,
            replaying: true,
        })
    }

    #[cfg(feature = "native-skia")]
    fn replay_finished(&mut self) -> Result<(), NativeHostError> {
        let mut retained = lock(&self.budget.retained)?;
        if self.replaying {
            retained.replays -= 1;
            self.replaying = false;
        }
        Ok(())
    }
}

impl Drop for Allocation {
    fn drop(&mut self) {
        // No caller code runs under this lock. On poisoning, retain the credits
        // and fail closed rather than make unverified state reusable.
        if let Ok(mut retained) = self.budget.retained.lock() {
            match self.kind {
                AllocationKind::Snapshot => {
                    retained.snapshots -= 1;
                    retained.snapshot_bytes -= self.bytes;
                }
                #[cfg(feature = "native-skia")]
                AllocationKind::Frame => {
                    retained.frames -= 1;
                    retained.frame_bytes -= self.bytes;
                    if self.replaying {
                        retained.replays -= 1;
                    }
                }
            }
        }
    }
}

struct Snapshot {
    input: Arc<NativeRenderSnapshot>,
    document_id: u64,
    source: Arc<Source>,
    _allocation: Allocation,
}

/// Pins the registry's snapshot reservation through an independent replay.
/// This never owns a session or renderer. Page input identity is independent
/// of the view/device stamp, which must be checked again before publication.
pub struct NativePresentationSnapshotBorrow {
    entry: Arc<Snapshot>,
}

#[derive(Debug)]
pub enum NativePresentationBorrowError {
    InvalidHandle,
    Stale,
    Host(NativeHostError),
}

impl From<NativePresentationBorrowError> for NativeHostError {
    fn from(error: NativePresentationBorrowError) -> Self {
        match error {
            NativePresentationBorrowError::InvalidHandle => {
                invalid("unknown presentation snapshot")
            }
            NativePresentationBorrowError::Stale => invalid("stale presentation render stamp"),
            NativePresentationBorrowError::Host(error) => error,
        }
    }
}

impl std::fmt::Display for NativePresentationBorrowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHandle => f.write_str("unknown presentation snapshot"),
            Self::Stale => f.write_str("stale presentation render stamp"),
            Self::Host(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for NativePresentationBorrowError {}

impl NativePresentationSnapshotBorrow {
    pub fn snapshot(&self) -> &NativeRenderSnapshot {
        &self.entry.input
    }

    pub fn source(&self) -> NativePresentationId {
        self.entry.source.id
    }

    pub fn validate_current(&self, stamp: NativePresentationStamp) -> Result<(), NativeHostError> {
        self.validate_current_checked(stamp).map_err(Into::into)
    }

    pub fn validate_current_checked(
        &self,
        stamp: NativePresentationStamp,
    ) -> Result<(), NativePresentationBorrowError> {
        stamp
            .validate()
            .map_err(NativePresentationBorrowError::Host)?;
        if stamp.document_id != self.entry.document_id
            || stamp.revision != self.entry.input.revision()
            || stamp.font_generation != self.entry.input.font_generation()
            || !self
                .entry
                .source
                .current(stamp)
                .map_err(NativePresentationBorrowError::Host)?
        {
            return Err(NativePresentationBorrowError::Stale);
        }
        Ok(())
    }
}

struct Frame {
    bytes: Box<[u8]>,
    id: NativePresentationId,
    source: Arc<Source>,
    stamp: NativePresentationStamp,
    descriptor: NativePresentationDescriptor,
    page_count: usize,
    coordinate_space: &'static str,
    _allocation: Allocation,
}

impl Frame {
    fn info(&self) -> Result<NativePresentationFrameInfo, NativeHostError> {
        Ok(NativePresentationFrameInfo {
            frame: self.id,
            source: self.source.id,
            stamp: self.stamp,
            descriptor: self.descriptor.clone(),
            current: self.source.current(self.stamp)?,
            page_count: self.page_count,
            coordinate_space: self.coordinate_space,
        })
    }
}

#[derive(Default)]
struct Records {
    sources: HashMap<NativePresentationId, Arc<Source>>,
    snapshots: HashMap<NativePresentationId, Arc<Snapshot>>,
    frames: HashMap<NativePresentationId, Arc<Frame>>,
    leases: HashMap<NativePresentationId, Arc<Frame>>,
}

pub struct NativePresentationRegistry {
    config: NativePresentationConfig,
    records: Mutex<Records>,
    budget: Arc<Budget>,
}

impl NativePresentationRegistry {
    pub fn new(config: NativePresentationConfig) -> Result<Self, NativeHostError> {
        config.validate()?;
        Ok(Self {
            config,
            records: Mutex::new(Records::default()),
            budget: Arc::new(Budget {
                config,
                retained: Mutex::new(Retention::default()),
            }),
        })
    }

    pub fn create_source(
        &self,
        observer: NativeRuntimeObserver,
        stamp: NativePresentationStamp,
    ) -> Result<NativePresentationId, NativeHostError> {
        stamp.validate()?;
        if !observer.is_current(stamp.target(), stamp.font_generation) {
            return Err(invalid("stale presentation source document"));
        }
        let mut records = lock(&self.records)?;
        if records.sources.len() >= self.config.max_sources {
            return Err(limit("presentation source count quota exceeded"));
        }
        let id = next_id()?;
        records.sources.insert(
            id,
            Arc::new(Source {
                id,
                observer,
                active: AtomicBool::new(true),
                stamp: Mutex::new(stamp),
            }),
        );
        Ok(id)
    }

    pub fn configure_source(
        &self,
        source: NativePresentationId,
        stamp: NativePresentationStamp,
    ) -> Result<(), NativeHostError> {
        stamp.validate()?;
        let records = lock(&self.records)?;
        let source = records
            .sources
            .get(&source)
            .ok_or_else(|| invalid("unknown presentation source"))?;
        if !source.document_current(stamp.document_id, stamp.revision, stamp.font_generation) {
            return Err(invalid("stale presentation source document"));
        }
        let mut current = lock(&source.stamp)?;
        let previous = *current;
        if stamp.view_generation < previous.view_generation
            || stamp.device_generation < previous.device_generation
            || (stamp != previous
                && stamp.view_generation == previous.view_generation
                && stamp.device_generation == previous.device_generation)
        {
            return Err(invalid(
                "presentation generations must advance before changing inputs",
            ));
        }
        *current = stamp;
        Ok(())
    }

    pub fn close_source(&self, source: NativePresentationId) -> Result<(), NativeHostError> {
        let mut records = lock(&self.records)?;
        let source = records
            .sources
            .remove(&source)
            .ok_or_else(|| invalid("unknown presentation source"))?;
        source.active.store(false, Ordering::Release);
        Ok(())
    }

    pub fn retain_snapshot(
        &self,
        source: NativePresentationId,
        document_id: u64,
        snapshot: Arc<NativeRenderSnapshot>,
    ) -> Result<NativePresentationId, NativeHostError> {
        let mut records = lock(&self.records)?;
        let source = Arc::clone(
            records
                .sources
                .get(&source)
                .ok_or_else(|| invalid("unknown presentation source"))?,
        );
        if !source.document_current(document_id, snapshot.revision(), snapshot.font_generation()) {
            return Err(invalid("stale render snapshot document"));
        }
        let allocation = Allocation::snapshot(&self.budget, snapshot.usage().estimated_bytes)?;
        let id = next_id()?;
        records.snapshots.insert(
            id,
            Arc::new(Snapshot {
                input: snapshot,
                document_id,
                source,
                _allocation: allocation,
            }),
        );
        Ok(id)
    }

    pub fn snapshot_info(
        &self,
        snapshot: NativePresentationId,
    ) -> Result<NativePresentationSnapshotInfo, NativeHostError> {
        let records = lock(&self.records)?;
        let entry = records
            .snapshots
            .get(&snapshot)
            .ok_or_else(|| invalid("unknown presentation snapshot"))?;
        Ok(NativePresentationSnapshotInfo {
            snapshot,
            source: entry.source.id,
            document_id: entry.document_id,
            revision: entry.input.revision(),
            font_generation: entry.input.font_generation(),
            page: entry.input.page(),
            profile: super::render_snapshot::profile_name(entry.input.profile()),
            font_fingerprint: entry.input.font_state().fingerprint.clone(),
            usage: entry.input.usage(),
            current: entry.source.document_current(
                entry.document_id,
                entry.input.revision(),
                entry.input.font_generation(),
            ),
        })
    }

    /// Metadata-only full-stamp query. A stale valid handle returns false;
    /// malformed stamps/unknown handles remain errors. No Core/replay runs.
    pub fn snapshot_check(
        &self,
        snapshot: NativePresentationId,
        stamp: NativePresentationStamp,
    ) -> Result<bool, NativeHostError> {
        match self.borrow_snapshot_checked(snapshot, stamp) {
            Ok(_) => Ok(true),
            Err(NativePresentationBorrowError::Stale) => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    pub fn release_snapshot(&self, snapshot: NativePresentationId) -> Result<(), NativeHostError> {
        lock(&self.records)?
            .snapshots
            .remove(&snapshot)
            .ok_or_else(|| invalid("unknown presentation snapshot"))?;
        Ok(())
    }

    pub fn borrow_snapshot(
        &self,
        snapshot: NativePresentationId,
        stamp: NativePresentationStamp,
    ) -> Result<NativePresentationSnapshotBorrow, NativeHostError> {
        self.borrow_snapshot_checked(snapshot, stamp)
            .map_err(Into::into)
    }

    pub fn borrow_snapshot_checked(
        &self,
        snapshot: NativePresentationId,
        stamp: NativePresentationStamp,
    ) -> Result<NativePresentationSnapshotBorrow, NativePresentationBorrowError> {
        let records = lock(&self.records).map_err(NativePresentationBorrowError::Host)?;
        let borrowed = NativePresentationSnapshotBorrow {
            entry: Arc::clone(
                records
                    .snapshots
                    .get(&snapshot)
                    .ok_or(NativePresentationBorrowError::InvalidHandle)?,
            ),
        };
        borrowed.validate_current_checked(stamp)?;
        Ok(borrowed)
    }

    pub fn render_cpu(
        &self,
        snapshot: NativePresentationId,
        stamp: NativePresentationStamp,
        region: RasterRegion,
        scale: f64,
    ) -> Result<NativePresentationFrameInfo, NativeHostError> {
        #[cfg(feature = "native-skia")]
        {
            use crate::renderer::layer_renderer::RasterRenderOptions;
            stamp.validate()?;
            super::NativeDocumentSession::check_scale(scale)?;
            let options = RasterRenderOptions {
                scale,
                max_pixels: super::MAX_FRAME_PIXELS,
                transparent: false,
                ..Default::default()
            };
            let window =
                region.pixel_window(scale, options.max_dimension, super::MAX_FRAME_PIXELS)?;
            let (entry, mut allocation) = {
                let records = lock(&self.records)?;
                let entry = Arc::clone(
                    records
                        .snapshots
                        .get(&snapshot)
                        .ok_or_else(|| invalid("unknown presentation snapshot"))?,
                );
                if stamp.document_id != entry.document_id
                    || stamp.revision != entry.input.revision()
                    || stamp.font_generation != entry.input.font_generation()
                    || !entry.source.current(stamp)?
                {
                    return Err(invalid("stale presentation render stamp"));
                }
                let allocation = Allocation::frame(&self.budget, window.output_bytes)?;
                (entry, allocation)
            };
            // Only immutable input/observer Arcs cross this boundary. Native
            // objects are created, used, and dropped on this calling thread.
            let rendered = entry
                .input
                .create_renderer()?
                .render_region_pixels_with_options(entry.input.tree(), region, options)?;
            if rendered.pixels.bytes.len() != window.output_bytes
                || rendered.pixels.width != window.width
                || rendered.pixels.height != window.height
                || rendered.pixels.stride != window.stride
                || rendered.pixels.format
                    != crate::renderer::raster_pixels::RasterPixelFormat::Rgba8Premultiplied
                || rendered.pixels.color_space
                    != crate::renderer::layer_renderer::RasterColorSpace::Srgb
                || rendered.origin_x != window.origin_x
                || rendered.origin_y != window.origin_y
            {
                return Err(invalid(
                    "CPU renderer output disagrees with its reserved geometry",
                ));
            }
            let bytes = rendered.pixels.bytes.into_boxed_slice();
            allocation.replay_finished()?;
            let mut records = lock(&self.records)?;
            if !entry.source.current(stamp)? {
                return Err(invalid("presentation source changed during replay"));
            }
            let id = next_id()?;
            let frame = Arc::new(Frame {
                bytes,
                id,
                source: Arc::clone(&entry.source),
                stamp,
                page_count: 1,
                coordinate_space: "pageRaster",
                descriptor: NativePresentationDescriptor::Cpu {
                    width: window.width,
                    height: window.height,
                    stride: window.stride,
                    byte_length: window.output_bytes,
                    pixel_format: "rgba8Premultiplied",
                    color_space: "srgb",
                    alpha: "premultiplied",
                    origin: "topLeft",
                    origin_x: window.origin_x,
                    origin_y: window.origin_y,
                    raster_scale: scale,
                    device_scale: stamp.device_scale,
                },
                _allocation: allocation,
            });
            let info = frame.info()?;
            if !info.current {
                return Err(invalid("presentation source changed during replay"));
            }
            records.frames.insert(id, frame);
            Ok(info)
        }
        #[cfg(not(feature = "native-skia"))]
        {
            let _ = (snapshot, stamp, region, scale);
            Err(NativeHostError::FeatureDisabled)
        }
    }

    /// Compose owned immutable pages into one physical CPU viewport. This is
    /// synchronous work for a caller-owned render thread, with no Core/layout
    /// query and no renderer transfer. Every output uses the existing leases.
    pub fn render_cpu_batch(
        &self,
        stamp: NativePresentationStamp,
        viewport: NativePresentationViewport,
        placements: &[NativePagePlacement],
        clear_color: u32,
    ) -> Result<NativePresentationFrameInfo, NativeHostError> {
        #[cfg(feature = "native-skia")]
        {
            use crate::renderer::skia::SkiaSurfaceRenderOptions;
            use skia_safe::{surfaces, AlphaType, Color, ColorSpace, ColorType, ImageInfo};
            stamp.validate()?;
            let (stride, byte_length) =
                viewport.rgba_layout(4096, super::MAX_FRAME_PIXELS as usize)?;
            if placements.is_empty() || placements.len() > 32 {
                return Err(limit("CPU batch page count is outside its quota"));
            }
            let inputs: Vec<_> = placements
                .iter()
                .map(|placement| {
                    let input = self
                        .borrow_snapshot_checked(placement.snapshot, stamp)
                        .map_err(NativeHostError::from)?;
                    placement.validate(input.snapshot())?;
                    Ok(input)
                })
                .collect::<Result<_, NativeHostError>>()?;
            if inputs
                .iter()
                .any(|input| input.source() != inputs[0].source())
            {
                return Err(invalid(
                    "CPU batch pages must share one presentation source",
                ));
            }
            let mut allocation = Allocation::frame(&self.budget, byte_length)?;
            let mut bytes = vec![0u8; byte_length];
            let image_info = ImageInfo::new(
                (viewport.width, viewport.height),
                ColorType::RGBA8888,
                AlphaType::Premul,
                ColorSpace::new_srgb(),
            );
            {
                let mut surface = surfaces::wrap_pixels(&image_info, &mut bytes, stride, None)
                    .ok_or_else(|| {
                        NativeHostError::Unsupported(
                            "could not borrow the CPU presentation pixels".into(),
                        )
                    })?;
                surface.canvas().clear(Color::from(clear_color));
                for (placement, input) in placements.iter().zip(&inputs) {
                    // At most one native renderer/font copy is live here. Its
                    // three caches together retain at most the 8 MiB estimate.
                    let renderer = input.snapshot().create_renderer()?.with_cache_limits(
                        super::MAX_RENDER_CACHE_ENTRIES,
                        super::MAX_RENDER_CACHE_BYTES / 3,
                    );
                    renderer.render_to_canvas_strict(
                        surface.canvas(),
                        input.snapshot().tree(),
                        SkiaSurfaceRenderOptions {
                            scale: placement.scale,
                            origin_x: placement.origin_x,
                            origin_y: placement.origin_y,
                            background_color: placement.background_color,
                        },
                    )?;
                }
            }
            allocation.replay_finished()?;
            let mut records = lock(&self.records)?;
            for input in &inputs {
                input.validate_current(stamp)?;
            }
            let id = next_id()?;
            let frame = Arc::new(Frame {
                bytes: bytes.into_boxed_slice(),
                id,
                source: Arc::clone(&inputs[0].entry.source),
                stamp,
                page_count: placements.len(),
                coordinate_space: "viewportPixels",
                descriptor: NativePresentationDescriptor::Cpu {
                    width: viewport.width,
                    height: viewport.height,
                    stride,
                    byte_length,
                    pixel_format: "rgba8Premultiplied",
                    color_space: "srgb",
                    alpha: "premultiplied",
                    origin: "topLeft",
                    origin_x: 0,
                    origin_y: 0,
                    raster_scale: 1.0,
                    device_scale: stamp.device_scale,
                },
                _allocation: allocation,
            });
            let info = frame.info()?;
            if !info.current {
                return Err(invalid("presentation source changed during replay"));
            }
            records.frames.insert(id, frame);
            Ok(info)
        }
        #[cfg(not(feature = "native-skia"))]
        {
            let _ = (stamp, viewport, placements, clear_color);
            Err(NativeHostError::FeatureDisabled)
        }
    }

    pub fn frame_info(
        &self,
        frame: NativePresentationId,
    ) -> Result<NativePresentationFrameInfo, NativeHostError> {
        lock(&self.records)?
            .frames
            .get(&frame)
            .ok_or_else(|| invalid("unknown presentation frame"))?
            .info()
    }

    pub fn acquire_frame(
        &self,
        frame: NativePresentationId,
        stamp: NativePresentationStamp,
    ) -> Result<NativePresentationLeaseInfo, NativeHostError> {
        stamp.validate()?;
        let mut records = lock(&self.records)?;
        let frame = Arc::clone(
            records
                .frames
                .get(&frame)
                .ok_or_else(|| invalid("unknown presentation frame"))?,
        );
        if stamp != frame.stamp || !frame.source.current(stamp)? {
            return Err(invalid("stale presentation frame"));
        }
        if records.leases.len() >= self.config.max_leases {
            return Err(limit("presentation consumer lease count quota exceeded"));
        }
        let id = next_id()?;
        let info = frame.info()?;
        if !info.current {
            return Err(invalid("stale presentation frame"));
        }
        let result = NativePresentationLeaseInfo {
            lease: id,
            frame: info,
        };
        records.leases.insert(id, frame);
        Ok(result)
    }

    pub fn release_frame(&self, frame: NativePresentationId) -> Result<(), NativeHostError> {
        lock(&self.records)?
            .frames
            .remove(&frame)
            .ok_or_else(|| invalid("unknown presentation frame"))?;
        Ok(())
    }

    pub fn lease_info(
        &self,
        lease: NativePresentationId,
    ) -> Result<NativePresentationLeaseInfo, NativeHostError> {
        let records = lock(&self.records)?;
        let frame = records
            .leases
            .get(&lease)
            .ok_or_else(|| invalid("unknown presentation lease"))?;
        Ok(NativePresentationLeaseInfo {
            lease,
            frame: frame.info()?,
        })
    }

    /// The returned CPU address is immutable and valid until this lease is
    /// released or this registry is dropped. The caller must keep the registry
    /// alive and coordinate pointer use versus lease release and library unload.
    /// Source staleness does not revoke memory from an existing lease.
    pub fn lease_data(
        &self,
        lease: NativePresentationId,
    ) -> Result<(*const u8, usize), NativeHostError> {
        let records = lock(&self.records)?;
        let frame = records
            .leases
            .get(&lease)
            .ok_or_else(|| invalid("unknown presentation lease"))?;
        Ok((frame.bytes.as_ptr(), frame.bytes.len()))
    }

    pub fn release_lease(&self, lease: NativePresentationId) -> Result<(), NativeHostError> {
        lock(&self.records)?
            .leases
            .remove(&lease)
            .ok_or_else(|| invalid("unknown presentation lease"))?;
        Ok(())
    }

    pub fn stats(&self) -> Result<NativePresentationStats, NativeHostError> {
        let records = lock(&self.records)?;
        let retained = lock(&self.budget.retained)?;
        Ok(NativePresentationStats {
            sources: records.sources.len(),
            snapshot_handles: records.snapshots.len(),
            retained_snapshots: retained.snapshots,
            snapshot_estimated_bytes: retained.snapshot_bytes,
            frame_handles: records.frames.len(),
            retained_frames: retained.frames,
            frame_bytes: retained.frame_bytes,
            leases: records.leases.len(),
            in_flight_replays: retained.replays,
        })
    }
}
