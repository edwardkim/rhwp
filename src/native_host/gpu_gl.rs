//! Owner-thread OpenGL Ganesh adapter for a host-owned current context/FBO.
//!
//! Pages are replayed strictly into an offscreen GPU surface before the host
//! target is touched. This submits work on the same GL queue; it neither swaps
//! a window nor exports a texture for another context to sample.

use std::cell::RefCell;
use std::ffi::{c_void, CStr};
use std::marker::PhantomData;
use std::rc::Rc;
use std::thread::{self, ThreadId};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use skia_safe::{
    gpu, AlphaType, BlendMode, Color, ColorSpace, ColorType, ImageInfo, Paint, Surface,
};

use super::presentation::{
    NativePresentationId, NativePresentationRegistry, NativePresentationSnapshotBorrow,
    NativePresentationStamp,
};
use super::render_snapshot::NativeRenderSnapshot;
use super::NativeHostError;
use crate::renderer::skia::{SkiaLayerRenderer, SkiaSurfaceRenderOptions};

mod errors;
mod recovery;
mod tiles;
pub use errors::{NativeGlError, NativeGlFailureInfo, NativeGlFailureKind, NativeGlResult};
use recovery::ContextCredit;
pub use recovery::{
    context_credits, NativeGlContextCredits, NativeGlRecoveryInfo, MAX_GL_CONTEXT_CREDITS,
};
use tiles::{CachedTile, ManagedTarget, TargetAllocation, TargetBudget, TileCache, TilePlan};
pub use tiles::{NativeGlTargetStats, NativeGlTileStats};

const MIB: usize = 1024 * 1024;

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}

fn limit(message: &str) -> NativeHostError {
    NativeHostError::LimitExceeded(message.into())
}

fn unsupported(message: &str) -> NativeGlError {
    NativeGlError::new(NativeGlFailureKind::BackendUnavailable, message)
}

/// Logical native target/font/cache ceilings, not total driver VRAM or RSS.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeGlPresenterConfig {
    pub max_dimension: i32,
    pub max_pixels: usize,
    pub max_pages: usize,
    pub max_target_bytes: usize,
    pub max_scratch_targets: usize,
    pub max_scratch_bytes: usize,
    pub max_renderers: usize,
    pub max_renderer_font_bytes: usize,
    pub renderer_cache_bytes: usize,
    pub resource_cache_bytes: usize,
    pub tile_size: usize,
    pub max_cached_tiles: usize,
    pub max_tile_bytes: usize,
    pub max_managed_targets: usize,
    pub max_managed_target_bytes: usize,
}

impl Default for NativeGlPresenterConfig {
    fn default() -> Self {
        Self {
            max_dimension: 4096,
            max_pixels: 16 * 1024 * 1024,
            max_pages: 32,
            max_target_bytes: 64 * MIB,
            max_scratch_targets: 2,
            max_scratch_bytes: 128 * MIB,
            max_renderers: 2,
            max_renderer_font_bytes: 64 * MIB,
            renderer_cache_bytes: 8 * MIB,
            resource_cache_bytes: 64 * MIB,
            tile_size: 512,
            max_cached_tiles: 64,
            max_tile_bytes: 64 * MIB,
            max_managed_targets: 67,
            max_managed_target_bytes: 192 * MIB,
        }
    }
}

impl NativeGlPresenterConfig {
    pub fn validate(self) -> Result<(), NativeHostError> {
        if !(1..=4096).contains(&self.max_dimension)
            || !(1..=16 * 1024 * 1024).contains(&self.max_pixels)
            || !(1..=32).contains(&self.max_pages)
            || !(1..=64 * MIB).contains(&self.max_target_bytes)
            || !(1..=2).contains(&self.max_scratch_targets)
            || !(1..=128 * MIB).contains(&self.max_scratch_bytes)
            || !(1..=8).contains(&self.max_renderers)
            || !(1..=64 * MIB).contains(&self.max_renderer_font_bytes)
            || self.renderer_cache_bytes > 8 * MIB
            || self.resource_cache_bytes > 64 * MIB
            || ![128, 256, 512, 1024].contains(&self.tile_size)
            || self.max_cached_tiles > 64
            || self.max_tile_bytes > 64 * MIB
            || !(1..=67).contains(&self.max_managed_targets)
            || !(1..=192 * MIB).contains(&self.max_managed_target_bytes)
        {
            return Err(invalid("invalid GL presenter limits"));
        }
        Ok(())
    }

    pub fn target_bytes(self, target: NativeGlTarget) -> Result<usize, NativeHostError> {
        self.validate()?;
        if target.context_id == 0
            || target.device_generation == 0
            || target.sample_count != 0
            || ![0, 8].contains(&target.stencil_bits)
        {
            return Err(invalid(
                "invalid GL target identity, size, samples, or stencil",
            ));
        }
        let (_, bytes) = super::presentation::NativePresentationViewport {
            width: target.width,
            height: target.height,
        }
        .rgba_layout(self.max_dimension, self.max_pixels)?;
        if bytes > self.max_target_bytes
            || bytes > self.max_scratch_bytes
            || bytes > self.max_managed_target_bytes
        {
            return Err(limit("GL target byte quota exceeded"));
        }
        Ok(bytes)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeGlTargetFormat {
    Rgba8,
    Srgb8Alpha8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NativeGlTargetOrigin {
    TopLeft,
    BottomLeft,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeGlTarget {
    pub context_id: u64,
    pub device_generation: u64,
    pub fbo: u32,
    pub width: i32,
    pub height: i32,
    pub format: NativeGlTargetFormat,
    pub origin: NativeGlTargetOrigin,
    pub sample_count: u32,
    pub stencil_bits: u32,
}

pub use super::presentation::NativePagePlacement as NativeGlPlacement;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeGlDeviceInfo {
    pub vendor: String,
    pub renderer: String,
    pub version: String,
    /// A known software renderer name, not hardware qualification.
    pub software_renderer: bool,
    pub hardware_acceleration_verified: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeGlFrame {
    pub backend: &'static str,
    pub context_id: u64,
    pub stamp: NativePresentationStamp,
    pub target: NativeGlTarget,
    pub page_count: usize,
    pub pixel_format: &'static str,
    pub color_space: &'static str,
    pub alpha: &'static str,
    /// GPU targets have no CPU row stride/view in this API.
    pub stride: Option<usize>,
    pub submission: u64,
    pub whole_frame_readbacks: u64,
    pub replay_micros: u64,
    pub flush_submit_micros: u64,
    pub timing_scope: &'static str,
    pub whole_frame_cpu_uploads: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeGlPresenterInfo {
    pub backend: &'static str,
    pub context_id: u64,
    pub device_generation: u64,
    pub state: &'static str,
    pub device: NativeGlDeviceInfo,
    pub config: NativeGlPresenterConfig,
    pub scratch_targets: usize,
    pub scratch_estimated_bytes: usize,
    pub renderers: usize,
    pub renderer_font_estimated_bytes: usize,
    pub resource_cache_bytes: usize,
    pub resource_cache_objects: usize,
    pub frames_submitted: u64,
    pub whole_frame_readbacks: u64,
    pub external_texture_export: bool,
    pub tile_cache: NativeGlTileStats,
    pub managed_targets: NativeGlTargetStats,
    pub renderer_cache_estimated_bytes: usize,
    pub context_credits: NativeGlContextCredits,
    pub recovery: NativeGlRecoveryInfo,
    pub last_gpu_failure: Option<NativeGlFailureInfo>,
    pub whole_frame_cpu_uploads: u64,
}

type Scratch = ManagedTarget;

struct CachedRenderer {
    renderer: SkiaLayerRenderer,
    fingerprint: String,
    font_bytes: usize,
}

/// All native objects stay on the construction thread. The explicit Rc marker
/// also prevents future Skia wrapper trait changes from making this Send/Sync.
pub struct NativeGlPresenter {
    owner: ThreadId,
    context_id: u64,
    device_generation: u64,
    config: NativeGlPresenterConfig,
    device: NativeGlDeviceInfo,
    scratch: Vec<Scratch>,
    renderers: Vec<CachedRenderer>,
    tiles: TileCache,
    target_budget: TargetBudget,
    submissions: u64,
    context: Option<gpu::DirectContext>,
    state: &'static str,
    _owner_only: PhantomData<Rc<()>>,
    context_credit: Option<ContextCredit>,
    recovery: NativeGlRecoveryInfo,
    last_gpu_failure: Option<NativeGlFailureInfo>,
}

impl NativeGlPresenter {
    /// # Safety
    /// The host's context must be current on this thread. Every returned GL
    /// function pointer must be valid for that context throughout this owner.
    /// The loader is called only during construction, never stored here.
    pub unsafe fn new(
        context_id: u64,
        device_generation: u64,
        config: NativeGlPresenterConfig,
        mut load: impl FnMut(&CStr) -> *const c_void,
    ) -> NativeGlResult<Self> {
        unsafe { Self::new_with_credit(context_id, device_generation, config, &mut load, None) }
    }

    unsafe fn new_with_credit(
        context_id: u64,
        device_generation: u64,
        config: NativeGlPresenterConfig,
        mut load: impl FnMut(&CStr) -> *const c_void,
        credit: Option<ContextCredit>,
    ) -> NativeGlResult<Self> {
        config.validate()?;
        if context_id == 0 || device_generation == 0 {
            return Err(invalid("GL context identity/generation must be nonzero").into());
        }
        let device = unsafe { read_device(&mut load)? };
        // Qt/GLX resolvers may return non-null foreign dispatch stubs for EGL
        // names. Ganesh's optional EGL query must never call those as EGL.
        let interface = gpu::gl::Interface::new_load_with_cstr(|name| {
            if name.to_bytes().starts_with(b"gl") {
                load(name)
            } else {
                std::ptr::null()
            }
        })
        .filter(|interface| interface.validate())
        .ok_or_else(|| unsupported("GL function interface is unavailable or incomplete"))?;
        // Config/string/interface probes above create no GPU object. A native
        // context factory attempt below may, even if it fails to return a handle.
        let mut credit = match credit {
            Some(credit) => credit,
            None => ContextCredit::reserve()?,
        };
        credit.touch();
        let mut context =
            gpu::direct_contexts::make_gl(interface, None).ok_or_else(|| {
                NativeGlError::new(NativeGlFailureKind::ContextCreationFailed,
                "Skia native GL context creation failed; allocation/device cause is not confirmed")
            })?;
        context.set_resource_cache_limit(config.resource_cache_bytes);
        Ok(Self {
            owner: thread::current().id(),
            context_id,
            device_generation,
            config,
            device,
            scratch: Vec::new(),
            renderers: Vec::new(),
            tiles: TileCache::default(),
            target_budget: TargetBudget::new(
                config.max_managed_targets,
                config.max_managed_target_bytes,
            ),
            submissions: 0,
            context: Some(context),
            state: "live",
            _owner_only: PhantomData,
            context_credit: Some(credit),
            recovery: NativeGlRecoveryInfo {
                last_attempted_context_id: context_id,
                last_attempted_device_generation: device_generation,
                ..Default::default()
            },
            last_gpu_failure: None,
        })
    }

    fn check_owner(&self) -> NativeGlResult<()> {
        if thread::current().id() != self.owner {
            return Err(NativeGlError::new(
                NativeGlFailureKind::WrongOwner,
                "GL presenter belongs to another thread",
            ));
        }
        Ok(())
    }

    pub fn check_context(&self, context_id: u64, device_generation: u64) -> NativeGlResult<()> {
        self.check_owner()?;
        if context_id != self.context_id || device_generation != self.device_generation {
            return Err(NativeGlError::new(
                NativeGlFailureKind::ContextMismatch,
                "GL context identity/device generation mismatch",
            ));
        }
        if self.context.is_none() {
            return Err(NativeGlError::new(NativeGlFailureKind::ContextUnavailable, "GL presenter is no longer live; explicitly recreate with a new context/device generation"));
        }
        Ok(())
    }

    pub fn info(&self) -> NativeGlResult<NativeGlPresenterInfo> {
        self.check_owner()?;
        // This queries Skia's bookkeeping only; no GL/current-context callback.
        let cache = self
            .context
            .as_ref()
            .map(|context| context.resource_cache_usage());
        Ok(NativeGlPresenterInfo {
            backend: "openGlGanesh",
            context_id: self.context_id,
            device_generation: self.device_generation,
            state: self.state,
            device: self.device.clone(),
            config: self.config,
            scratch_targets: self.scratch.len(),
            scratch_estimated_bytes: self.scratch.iter().map(|target| target.bytes()).sum(),
            renderers: self.renderers.len(),
            renderer_font_estimated_bytes: self
                .renderers
                .iter()
                .map(|renderer| renderer.font_bytes)
                .sum(),
            resource_cache_bytes: cache.as_ref().map_or(0, |usage| usage.resource_bytes),
            resource_cache_objects: cache.as_ref().map_or(0, |usage| usage.resource_count),
            frames_submitted: self.submissions,
            whole_frame_readbacks: 0,
            external_texture_export: false,
            tile_cache: self.tiles.stats(),
            managed_targets: self.target_budget.stats(),
            renderer_cache_estimated_bytes: self
                .renderers
                .iter()
                .map(|renderer| {
                    let stats = renderer.renderer.cache_stats();
                    stats.pictures.approximate_bytes
                        + stats.glyphs.approximate_bytes
                        + stats.images.approximate_bytes
                })
                .sum(),
            context_credits: context_credits(),
            recovery: self.recovery.clone(),
            last_gpu_failure: self.last_gpu_failure.clone(),
            whole_frame_cpu_uploads: 0,
        })
    }

    fn healthy_device(&mut self, message: &'static str) -> NativeGlResult<()> {
        if self
            .context
            .as_mut()
            .ok_or_else(|| {
                NativeGlError::new(
                    NativeGlFailureKind::ContextUnavailable,
                    "GL context is no longer live",
                )
            })?
            .is_device_lost()
        {
            let error = NativeGlError::new(NativeGlFailureKind::DeviceLost, message);
            self.last_gpu_failure = Some(error.info());
            self.abandon()?;
            return Err(error);
        }
        Ok(())
    }

    fn renderer(
        &mut self,
        snapshot: &NativeRenderSnapshot,
    ) -> Result<&SkiaLayerRenderer, NativeHostError> {
        let fingerprint = &snapshot.font_state().fingerprint;
        if let Some(index) = self
            .renderers
            .iter()
            .position(|entry| &entry.fingerprint == fingerprint)
        {
            let entry = self.renderers.remove(index);
            self.renderers.push(entry);
            return Ok(&self.renderers.last().expect("just inserted").renderer);
        }
        // The font loader makes per-face copies; collection sources can be
        // larger than the original explicit-font byte count in snapshot usage.
        let font_bytes = snapshot
            .font_state()
            .sources
            .iter()
            .try_fold(0usize, |sum, source| {
                source
                    .byte_length
                    .checked_mul(source.face_count as usize)
                    .and_then(|bytes| sum.checked_add(bytes))
            })
            .filter(|bytes| *bytes <= self.config.max_renderer_font_bytes)
            .ok_or_else(|| limit("GL renderer font-copy estimate exceeds its quota"))?;
        while self.renderers.len() >= self.config.max_renderers
            || self
                .renderers
                .iter()
                .map(|entry| entry.font_bytes)
                .sum::<usize>()
                + font_bytes
                > self.config.max_renderer_font_bytes
        {
            self.renderers.remove(0);
        }
        // Evict before font creation, and divide the per-renderer budget among
        // its picture/glyph/image caches (each has an independent ceiling).
        let renderer = snapshot.create_renderer()?.with_cache_limits(
            super::MAX_RENDER_CACHE_ENTRIES,
            self.config.renderer_cache_bytes / 3,
        );
        self.renderers.push(CachedRenderer {
            renderer,
            fingerprint: fingerprint.clone(),
            font_bytes,
        });
        Ok(&self.renderers.last().expect("just inserted").renderer)
    }

    fn scratch(&mut self, target: NativeGlTarget, bytes: usize) -> NativeGlResult<Scratch> {
        if let Some(index) = self
            .scratch
            .iter()
            .position(|entry| entry.width == target.width && entry.height == target.height)
        {
            return Ok(self.scratch.remove(index));
        }
        while self.scratch.len() >= self.config.max_scratch_targets
            || self
                .scratch
                .iter()
                .map(|entry| entry.bytes())
                .sum::<usize>()
                + bytes
                > self.config.max_scratch_bytes
        {
            self.scratch.remove(0);
        }
        self.ensure_target_room(1, bytes);
        let allocation = self.target_budget.reserve(bytes)?;
        self.allocate_target(target.width, target.height, allocation)
    }

    fn ensure_target_room(&mut self, count: usize, bytes: usize) -> bool {
        while !self.target_budget.can_reserve(count, bytes) {
            if self.tiles.evict_idle() {
                continue;
            }
            if self.scratch.pop().is_some() {
                continue;
            }
            return false;
        }
        true
    }

    fn allocate_target(
        &mut self,
        width: i32,
        height: i32,
        allocation: TargetAllocation,
    ) -> NativeGlResult<ManagedTarget> {
        let info = ImageInfo::new(
            (width, height),
            ColorType::RGBA8888,
            AlphaType::Premul,
            ColorSpace::new_srgb(),
        );
        let surface = gpu::surfaces::render_target(
            self.context
                .as_mut()
                .ok_or_else(|| invalid("GL presenter is no longer live"))?,
            gpu::Budgeted::Yes,
            &info,
            None,
            gpu::SurfaceOrigin::TopLeft,
            None,
            false,
            None,
        )
        .ok_or_else(|| {
            let error = NativeGlError::new(
                NativeGlFailureKind::AllocationFailed,
                "Skia could not allocate a GL target; device loss is not confirmed",
            );
            self.last_gpu_failure = Some(error.info());
            error
        })?;
        Ok(ManagedTarget::new(surface, width, height, allocation))
    }

    /// # Safety
    /// The matching host GL context must be current and target.fbo must describe
    /// a live compatible FBO for the duration of this call. The host restores any
    /// GL state it relies on afterwards and owns swap/present/context lifetime.
    pub unsafe fn render(
        &mut self,
        registry: &NativePresentationRegistry,
        stamp: NativePresentationStamp,
        target: NativeGlTarget,
        placements: &[NativeGlPlacement],
        clear_color: u32,
    ) -> NativeGlResult<NativeGlFrame> {
        self.check_context(target.context_id, target.device_generation)?;
        stamp.validate()?;
        if stamp.device_generation != self.device_generation {
            return Err(NativeGlError::new(
                NativeGlFailureKind::ContextMismatch,
                "frame/device generation mismatch",
            ));
        }
        let bytes = self.config.target_bytes(target)?;
        if placements.is_empty() || placements.len() > self.config.max_pages {
            return Err(limit("GL placement count is outside its quota").into());
        }
        let inputs: Vec<NativePresentationSnapshotBorrow> = placements
            .iter()
            .map(|placement| {
                let input = registry.borrow_snapshot_checked(placement.snapshot, stamp)?;
                placement.validate(input.snapshot())?;
                Ok(input)
            })
            .collect::<NativeGlResult<_>>()?;
        if inputs
            .iter()
            .any(|input| input.source() != inputs[0].source())
        {
            return Err(
                invalid("all GL page placements must share one presentation source").into(),
            );
        }
        if self.submissions == u64::MAX {
            return Err(limit("GL submission sequence exhausted").into());
        }
        if self
            .context
            .as_mut()
            .expect("checked live")
            .is_device_lost()
        {
            let error = NativeGlError::new(
                NativeGlFailureKind::DeviceLost,
                "Skia reported GL device/context loss; presenter abandoned",
            );
            self.last_gpu_failure = Some(error.info());
            self.abandon()?;
            return Err(error);
        }
        // Qt/host GL work may have changed bindings since the previous call.
        self.context.as_mut().expect("checked live").reset(None);
        let mut scratch = self.scratch(target, bytes)?;
        let mut result = self.replay_and_publish(
            &mut scratch,
            stamp,
            target,
            placements,
            &inputs,
            clear_color,
        );
        if result.is_err() && self.context.is_some() {
            // Failed scratch draws can still reference tiles. Drain their work
            // before a later call can evict those map entries. No failed frame
            // is copied to the visible target by this scratch-only submission.
            if let Err(error) = self.submit_surface(
                &mut scratch.surface,
                "GL failed-frame cleanup submit failed; presenter abandoned",
            ) {
                result = Err(error);
            }
        }
        if self.context.is_some() {
            self.scratch.push(scratch);
        }
        if result.is_ok() {
            self.last_gpu_failure = None;
        }
        result
    }

    fn replay_and_publish(
        &mut self,
        scratch: &mut Scratch,
        stamp: NativePresentationStamp,
        target: NativeGlTarget,
        placements: &[NativeGlPlacement],
        inputs: &[NativePresentationSnapshotBorrow],
        clear_color: u32,
    ) -> NativeGlResult<NativeGlFrame> {
        let replay_started = Instant::now();
        scratch.surface.canvas().clear(Color::from(clear_color));
        // Keep every tile referenced by this frame pinned until submission.
        // Cache admission/eviction cannot reclaim those surfaces mid-frame.
        let mut tile_pins = Vec::new();
        for (placement, input) in placements.iter().zip(inputs) {
            if !self.replay_tiles(
                &mut scratch.surface,
                stamp,
                target,
                *placement,
                input,
                clear_color,
                &mut tile_pins,
            )? {
                self.tiles.observations.viewport_fallbacks =
                    self.tiles.observations.viewport_fallbacks.saturating_add(1);
                self.renderer(input.snapshot())?.render_to_surface_strict(
                    &mut scratch.surface,
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
        let replay_micros = micros(replay_started);
        let flush_started = Instant::now();
        self.submit_surface(
            &mut scratch.surface,
            "GL scratch flush/submit failed; presenter abandoned",
        )?;
        let image = scratch.surface.image_snapshot();
        if !image.is_texture_backed() {
            return Err(unsupported("GL scratch snapshot is not GPU backed"));
        }
        let backend = gpu::backend_render_targets::make_gl(
            (target.width, target.height),
            Some(0),
            target.stencil_bits as usize,
            gpu::gl::FramebufferInfo {
                fboid: target.fbo,
                format: match target.format {
                    NativeGlTargetFormat::Rgba8 => 0x8058,
                    NativeGlTargetFormat::Srgb8Alpha8 => 0x8C43,
                },
                ..Default::default()
            },
        );
        let mut visible = gpu::surfaces::wrap_backend_render_target(
            self.context.as_mut().expect("checked live"),
            &backend,
            match target.origin {
                NativeGlTargetOrigin::TopLeft => gpu::SurfaceOrigin::TopLeft,
                NativeGlTargetOrigin::BottomLeft => gpu::SurfaceOrigin::BottomLeft,
            },
            ColorType::RGBA8888,
            ColorSpace::new_srgb(),
            None,
        )
        .ok_or_else(|| {
            NativeGlError::new(NativeGlFailureKind::TargetUnavailable,
            "Skia could not borrow the host GL framebuffer; target/resource cause is not confirmed")
        })?;
        // Resource errors above have only changed scratch. Check the source again
        // immediately before the single full-target GPU copy becomes visible.
        for input in inputs {
            input.validate_current_checked(stamp)?;
        }
        let mut copy = Paint::default();
        copy.set_blend_mode(BlendMode::Src);
        visible.canvas().draw_image(&image, (0.0, 0.0), Some(&copy));
        self.submit_surface(
            &mut visible,
            "GL host-target flush/submit failed; presenter abandoned",
        )?;
        self.submissions += 1;
        Ok(NativeGlFrame {
            backend: "openGlGanesh",
            context_id: self.context_id,
            stamp,
            target,
            page_count: placements.len(),
            pixel_format: "rgba8Premultiplied",
            color_space: "srgb",
            alpha: "premultiplied",
            stride: None,
            submission: self.submissions,
            whole_frame_readbacks: 0,
            replay_micros,
            flush_submit_micros: micros(flush_started),
            timing_scope: "hostWallClock",
            whole_frame_cpu_uploads: 0,
        })
    }

    fn submit_surface(
        &mut self,
        surface: &mut Surface,
        message: &'static str,
    ) -> NativeGlResult<()> {
        if self
            .context
            .as_mut()
            .expect("checked live")
            .flush_and_submit_surface(surface, gpu::SyncCpu::No)
            .is_err()
        {
            let lost = self
                .context
                .as_mut()
                .expect("checked live")
                .is_device_lost();
            let error = NativeGlError::new(
                if lost {
                    NativeGlFailureKind::DeviceLost
                } else {
                    NativeGlFailureKind::SubmissionFailed
                },
                message,
            );
            self.last_gpu_failure = Some(error.info());
            self.abandon()?;
            return Err(error);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn replay_tiles(
        &mut self,
        scratch: &mut Surface,
        stamp: NativePresentationStamp,
        target: NativeGlTarget,
        placement: NativeGlPlacement,
        input: &NativePresentationSnapshotBorrow,
        clear_color: u32,
        frame_pins: &mut Vec<Rc<CachedTile>>,
    ) -> NativeGlResult<bool> {
        let Some(plan) = TilePlan::new(
            self.config,
            stamp,
            target,
            placement,
            input.snapshot(),
            clear_color,
        ) else {
            return Ok(false);
        };
        let mut found: Vec<_> = plan.keys.iter().map(|key| self.tiles.get(key)).collect();
        let missing_count = found.iter().filter(|tile| tile.is_none()).count();
        if missing_count != 0 {
            let missing_bytes: usize = plan
                .keys
                .iter()
                .zip(&found)
                .filter(|(_, tile)| tile.is_none())
                .map(|(key, _)| key.region.bytes())
                .sum();
            while !self
                .tiles
                .has_room(self.config, missing_count, missing_bytes)
            {
                if !self.tiles.evict_idle() {
                    self.tiles.observations.pinned_admission_fallbacks = self
                        .tiles
                        .observations
                        .pinned_admission_fallbacks
                        .saturating_add(1);
                    return Ok(false);
                }
            }
            let working_bytes = plan
                .canonical_bytes
                .checked_add(missing_bytes)
                .ok_or_else(|| limit("GL tile working target byte overflow"))?;
            if !self.ensure_target_room(missing_count + 1, working_bytes) {
                self.tiles.observations.target_admission_fallbacks = self
                    .tiles
                    .observations
                    .target_admission_fallbacks
                    .saturating_add(1);
                return Ok(false);
            }
            let canonical_allocation = self.target_budget.reserve(plan.canonical_bytes)?;
            let allocations: Vec<_> = plan
                .keys
                .iter()
                .zip(&found)
                .filter(|(_, tile)| tile.is_none())
                .map(|(key, _)| self.target_budget.reserve(key.region.bytes()))
                .collect::<Result<_, _>>()?;
            let mut canonical =
                self.allocate_target(plan.width, plan.height, canonical_allocation)?;
            canonical.surface.canvas().clear(Color::TRANSPARENT);
            self.tiles.observations.canonical_replays =
                self.tiles.observations.canonical_replays.saturating_add(1);
            let replay = self.renderer(input.snapshot()).and_then(|renderer| {
                renderer
                    .render_to_surface_strict(
                        &mut canonical.surface,
                        input.snapshot().tree(),
                        SkiaSurfaceRenderOptions {
                            scale: placement.scale,
                            origin_x: plan.phase_x,
                            origin_y: plan.phase_y,
                            background_color: placement.background_color,
                        },
                    )
                    .map_err(NativeHostError::from)
            });
            if let Err(error) = replay {
                // Keep its reservation until every partial canonical task has
                // left Skia's unsubmitted graph, even for non-fallback errors.
                self.submit_surface(
                    &mut canonical.surface,
                    "GL failed-canonical cleanup submit failed; presenter abandoned",
                )?;
                if matches!(
                    error,
                    NativeHostError::Core(_)
                        | NativeHostError::InvalidArgument(_)
                        | NativeHostError::LimitExceeded(_)
                ) {
                    // Full-page staging must not turn an offscreen resource
                    // problem into a failure of the prior visible replay path.
                    self.tiles.observations.canonical_failures =
                        self.tiles.observations.canonical_failures.saturating_add(1);
                    return Ok(false);
                }
                return Err(error.into());
            }
            self.submit_surface(
                &mut canonical.surface,
                "GL canonical tile flush/submit failed; presenter abandoned",
            )?;
            let image = canonical.surface.image_snapshot();
            if !image.is_texture_backed() {
                return Err(unsupported("GL canonical tile input is not GPU backed"));
            }
            let mut copy = Paint::default();
            copy.set_blend_mode(BlendMode::Src);
            let mut prepared = Vec::with_capacity(missing_count);
            let mut allocations = allocations.into_iter();
            for (index, key) in plan.keys.iter().enumerate() {
                if found[index].is_some() {
                    continue;
                }
                let region = key.region;
                let mut tile = self.allocate_target(
                    region.width,
                    region.height,
                    allocations
                        .next()
                        .expect("one reservation per missing tile"),
                )?;
                tile.surface.canvas().draw_image(
                    &image,
                    (-region.x as f32, -region.y as f32),
                    Some(&copy),
                );
                self.submit_surface(
                    &mut tile.surface,
                    "GL tile copy flush/submit failed; presenter abandoned",
                )?;
                prepared.push((
                    index,
                    Rc::new(CachedTile {
                        key: key.clone(),
                        target: RefCell::new(tile),
                    }),
                ));
            }
            input.validate_current_checked(stamp)?;
            // Publish cache entries only after every strict replay and tile GPU
            // copy succeeded. The canonical target is never a hidden tile backing.
            for (index, tile) in prepared {
                found[index] = Some(Rc::clone(&tile));
                self.tiles.insert(tile);
            }
        }
        for tile in found.into_iter().flatten() {
            let image = tile.target.borrow_mut().surface.image_snapshot();
            if !image.is_texture_backed() {
                return Err(unsupported("cached GL tile is not GPU backed"));
            }
            scratch.canvas().draw_image(
                &image,
                (
                    (plan.integer_x + tile.key.region.x) as f32,
                    (plan.integer_y + tile.key.region.y) as f32,
                ),
                None,
            );
            frame_pins.push(tile);
        }
        Ok(true)
    }

    /// Rebind only to a genuinely new host context with an advanced device
    /// generation. The old native context is abandoned without GL calls.
    ///
    /// # Safety
    /// The new context must really be current, have a new identity, and keep all
    /// returned GL procedures valid. Failed admission changes no owner state.
    pub unsafe fn recreate(
        &mut self,
        context_id: u64,
        device_generation: u64,
        config: NativeGlPresenterConfig,
        load: impl FnMut(&CStr) -> *const c_void,
    ) -> NativeGlResult<()> {
        self.check_owner()?;
        config.validate()?;
        if context_id == 0
            || context_id == self.context_id
            || context_id == self.recovery.last_attempted_context_id
            || device_generation <= self.recovery.last_attempted_device_generation
        {
            return Err(NativeGlError::new(NativeGlFailureKind::RecoveryRejected,
                "recreate requires a new actual context identity and a device generation newer than the last attempt"));
        }
        if self.recovery.attempts == u64::MAX {
            return Err(limit("GL recovery attempt sequence exhausted").into());
        }
        // Reserve the new context before abandoning anything. An exhausted
        // process live/orphan budget leaves the existing handle unchanged.
        let credit = ContextCredit::reserve()?;
        self.abandon()?;
        self.recovery.attempts += 1;
        self.recovery.last_attempted_context_id = context_id;
        self.recovery.last_attempted_device_generation = device_generation;
        match unsafe {
            Self::new_with_credit(context_id, device_generation, config, load, Some(credit))
        } {
            Ok(mut replacement) => {
                replacement.recovery = self.recovery.clone();
                replacement.recovery.failure = None;
                *self = replacement;
                Ok(())
            }
            Err(error) => {
                self.state = "recoveryFailed";
                self.recovery.failure = Some(error.info());
                self.last_gpu_failure = Some(error.info());
                Err(NativeGlError {
                    kind: NativeGlFailureKind::RecoveryFailed,
                    error: error.error,
                })
            }
        }
    }

    /// # Safety
    /// Call only while the matching context is current. No render may be active.
    pub unsafe fn clear_caches(&mut self, context_id: u64, generation: u64) -> NativeGlResult<()> {
        self.check_context(context_id, generation)?;
        if self
            .context
            .as_mut()
            .expect("checked live")
            .is_device_lost()
        {
            let error = NativeGlError::new(
                NativeGlFailureKind::DeviceLost,
                "Skia reported device loss before cache cleanup",
            );
            self.last_gpu_failure = Some(error.info());
            self.abandon()?;
            return Err(error);
        }
        self.tiles.clear();
        self.scratch.clear();
        self.renderers.clear();
        self.context
            .as_mut()
            .expect("checked live")
            .free_gpu_resources();
        Ok(())
    }

    /// # Safety
    /// The matching context must be current. This explicitly frees GL resources.
    pub unsafe fn release(&mut self, context_id: u64, generation: u64) -> NativeGlResult<()> {
        self.check_context(context_id, generation)?;
        self.healthy_device("Skia reported device loss before resource retirement")?;
        self.tiles.clear();
        self.scratch.clear();
        self.renderers.clear();
        self.context
            .as_mut()
            .expect("checked live")
            .release_resources_and_abandon();
        self.context.take();
        if let Some(credit) = self.context_credit.take() {
            credit.proven_release();
        }
        self.state = "released";
        Ok(())
    }

    /// Context loss/TLS teardown fallback. Never calls the loader, host callback,
    /// or GL. Skia abandons before any surfaces/native objects are dropped.
    pub fn abandon(&mut self) -> NativeGlResult<()> {
        self.check_owner()?;
        if let Some(context) = &mut self.context {
            context.abandon();
        }
        self.tiles.clear();
        self.scratch.clear();
        self.renderers.clear();
        self.context.take();
        if let Some(mut credit) = self.context_credit.take() {
            credit.orphan();
        }
        self.state = "abandoned";
        Ok(())
    }
}

impl Drop for NativeGlPresenter {
    fn drop(&mut self) {
        // Even on a thread panic the host context may no longer be current.
        if let Some(context) = &mut self.context {
            context.abandon();
        }
        if let Some(credit) = &mut self.context_credit {
            credit.orphan();
        }
    }
}

fn micros(started: Instant) -> u64 {
    started.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}

unsafe fn read_device(
    load: &mut impl FnMut(&CStr) -> *const c_void,
) -> NativeGlResult<NativeGlDeviceInfo> {
    type GetString = unsafe extern "system" fn(u32) -> *const u8;
    let name = c"glGetString";
    let address = load(name);
    if address.is_null() {
        return Err(unsupported("glGetString is unavailable"));
    }
    // SAFETY: the typed loader contract requires a matching GL function pointer.
    let get_string = unsafe { std::mem::transmute::<*const c_void, GetString>(address) };
    let text = |kind| -> NativeGlResult<String> {
        let pointer = unsafe { get_string(kind) };
        if pointer.is_null() {
            return Err(unsupported("no current GL context/device string"));
        }
        Ok(unsafe { CStr::from_ptr(pointer.cast()) }
            .to_string_lossy()
            .chars()
            .take(512)
            .collect())
    };
    let vendor = text(0x1F00)?;
    let renderer = text(0x1F01)?;
    let version = text(0x1F02)?;
    let normalized = renderer.to_ascii_lowercase();
    let software_renderer = [
        "llvmpipe",
        "softpipe",
        "swrast",
        "swiftshader",
        "software",
        "lavapipe",
    ]
    .iter()
    .any(|name| normalized.contains(name));
    Ok(NativeGlDeviceInfo {
        vendor,
        renderer,
        version,
        software_renderer,
        hardware_acceleration_verified: false,
    })
}
