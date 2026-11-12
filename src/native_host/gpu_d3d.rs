//! Owner-thread D3D12/Ganesh producer for an immutable D3D11/DXGI bridge.
//!
//! Strict replay finishes in scratch before the shared BGRA target is touched.
//! Successful return includes a checked D3D12 queue fence and COMMON decay; it
//! is not a Flutter D3D11 consumer-completion or target-recycling fence.

use std::cell::RefCell;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use windows::core::{IUnknown, Interface};
use windows::Win32::Graphics::{Direct3D12::*, Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM};

use serde::{Deserialize, Serialize};
use skia_safe::{
    gpu, AlphaType, BlendMode, Color, ColorSpace, ColorType, ImageInfo, Paint, Surface,
};

use super::gpu_tiles::frame_resources::FrameResources;
use super::gpu_tiles::{
    CachedTile, ManagedTarget, TargetAllocation, TargetBudget, TileCache, TileConfig,
    TilePlacement, TilePlan, TileTarget,
};
use super::presentation::{
    NativePresentationId, NativePresentationRegistry, NativePresentationSnapshotBorrow,
    NativePresentationStamp,
};
use super::render_snapshot::NativeRenderSnapshot;
use super::NativeHostError;
use crate::renderer::skia::{SkiaLayerRenderer, SkiaSurfaceRenderOptions};

const MIB: usize = 1024 * 1024;
const MAX_COORDINATE: f64 = 16_777_216.0;
const MAX_CONTEXTS: usize = 4;
static CONTEXTS: AtomicUsize = AtomicUsize::new(0);
static QUARANTINED_CONTEXTS: AtomicUsize = AtomicUsize::new(0);

struct ContextCredit {
    returnable: bool,
}
impl ContextCredit {
    fn reserve() -> Result<Self, NativeHostError> {
        CONTEXTS
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                (count < MAX_CONTEXTS).then_some(count + 1)
            })
            .map_err(|_| limit("D3D process context/quarantine budget exhausted"))?;
        Ok(Self { returnable: true })
    }
    fn quarantine(&mut self) {
        if self.returnable {
            self.returnable = false;
            QUARANTINED_CONTEXTS.fetch_add(1, Ordering::Relaxed);
        }
    }
}
impl Drop for ContextCredit {
    fn drop(&mut self) {
        if self.returnable {
            CONTEXTS.fetch_sub(1, Ordering::Relaxed);
        }
    }
}
pub fn context_credits() -> serde_json::Value {
    serde_json::json!({"inUse":CONTEXTS.load(Ordering::Relaxed),
        "quarantined":QUARANTINED_CONTEXTS.load(Ordering::Relaxed),"limit":MAX_CONTEXTS})
}

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}
fn limit(message: &str) -> NativeHostError {
    NativeHostError::LimitExceeded(message.into())
}
fn unsupported(message: &str) -> NativeHostError {
    NativeHostError::Unsupported(message.into())
}

unsafe fn retain_com<T: Interface>(pointer: *const c_void) -> Result<T, NativeHostError> {
    if pointer.is_null() {
        return Err(invalid("D3D interface pointer is null"));
    }
    let raw = pointer as *mut c_void;
    unsafe { T::from_raw_borrowed(&raw) }
        .cloned()
        .ok_or_else(|| invalid("D3D interface is null"))
}

fn same_device(left: &ID3D12Device, right: &ID3D12Device) -> Result<bool, NativeHostError> {
    let left: IUnknown = left
        .cast()
        .map_err(|_| invalid("D3D device identity query failed"))?;
    let right: IUnknown = right
        .cast()
        .map_err(|_| invalid("D3D device identity query failed"))?;
    Ok(left.as_raw() == right.as_raw())
}

/// Logical target/font/cache limits, not total driver VRAM or process RSS.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeD3dPresenterConfig {
    pub max_dimension: i32,
    pub max_pixels: usize,
    pub max_pages: usize,
    pub max_target_bytes: usize,
    pub max_renderer_font_bytes: usize,
    pub renderer_cache_bytes: usize,
    pub resource_cache_bytes: usize,
    pub tile_size: usize,
    pub max_cached_tiles: usize,
    pub max_tile_bytes: usize,
    pub max_managed_targets: usize,
    pub max_managed_target_bytes: usize,
}

impl Default for NativeD3dPresenterConfig {
    fn default() -> Self {
        Self {
            max_dimension: 4096,
            max_pixels: 4 * 1024 * 1024,
            max_pages: 8,
            max_target_bytes: 16 * MIB,
            max_renderer_font_bytes: 64 * MIB,
            renderer_cache_bytes: 8 * MIB,
            resource_cache_bytes: 32 * MIB,
            tile_size: 512,
            max_cached_tiles: 64,
            max_tile_bytes: 16 * MIB,
            max_managed_targets: 66,
            max_managed_target_bytes: 48 * MIB,
        }
    }
}

impl NativeD3dPresenterConfig {
    pub fn validate(self) -> Result<(), NativeHostError> {
        if !(1..=4096).contains(&self.max_dimension)
            || !(1..=4 * 1024 * 1024).contains(&self.max_pixels)
            || !(1..=8).contains(&self.max_pages)
            || !(1..=16 * MIB).contains(&self.max_target_bytes)
            || !(1..=64 * MIB).contains(&self.max_renderer_font_bytes)
            || self.renderer_cache_bytes > 8 * MIB
            || self.resource_cache_bytes > 32 * MIB
            || ![128, 256, 512, 1024].contains(&self.tile_size)
            || self.max_cached_tiles > 64
            || self.max_tile_bytes > 64 * MIB
            || !(1..=66).contains(&self.max_managed_targets)
            || !(1..=96 * MIB).contains(&self.max_managed_target_bytes)
        {
            return Err(invalid("invalid D3d presenter limits"));
        }
        Ok(())
    }

    fn target_bytes(self, target: NativeD3dTarget) -> Result<usize, NativeHostError> {
        self.validate()?;
        if target.device_generation == 0
            || target.width <= 0
            || target.height <= 0
            || target.width > self.max_dimension
            || target.height > self.max_dimension
        {
            return Err(invalid("invalid D3d target generation or dimensions"));
        }
        (target.width as usize)
            .checked_mul(target.height as usize)
            .filter(|pixels| *pixels <= self.max_pixels)
            .and_then(|pixels| pixels.checked_mul(4))
            .filter(|bytes| {
                *bytes <= self.max_target_bytes && *bytes <= self.max_managed_target_bytes
            })
            .ok_or_else(|| limit("D3d target pixel/byte quota exceeded"))
    }

    fn tile_config(self) -> TileConfig {
        TileConfig {
            max_dimension: self.max_dimension,
            max_pixels: self.max_pixels,
            max_target_bytes: self.max_target_bytes,
            tile_size: self.tile_size,
            max_cached_tiles: self.max_cached_tiles,
            max_tile_bytes: self.max_tile_bytes,
        }
    }
}

/// Shared DEFAULT-heap, BGRA8_UNORM 2D target, single sample/mip/layer, COMMON
/// initially, with ALLOW_RENDER_TARGET | ALLOW_SIMULTANEOUS_ACCESS and shader
/// reading enabled. It is unpublished and exclusively writable during this call.
/// Format, premultiplication and top-left origin are fixed by ABI version 1.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeD3dTarget {
    pub device_generation: u64,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeD3dPlacement {
    pub snapshot: NativePresentationId,
    /// Pixel scale, including deviceScale. Origins are target pixel positions.
    pub scale: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    pub background_color: Option<u32>,
}

impl NativeD3dPlacement {
    fn validate(self, snapshot: &NativeRenderSnapshot) -> Result<(), NativeHostError> {
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
                .any(|v| !v.is_finite() || v.abs() > MAX_COORDINATE)
        {
            return Err(invalid("invalid D3d page placement"));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeD3dFrame {
    pub backend: &'static str,
    pub stamp: NativePresentationStamp,
    pub target: NativeD3dTarget,
    pub page_count: usize,
    pub submission: u64,
    pub pixel_format: &'static str,
    pub color_space: &'static str,
    pub alpha: &'static str,
    pub origin: &'static str,
    pub producer_gpu_complete: bool,
    pub resource_common: bool,
    pub producer_fence_value: u64,
    pub consumer_gpu_complete: bool,
    pub whole_frame_readbacks: u64,
}

struct Renderer {
    renderer: SkiaLayerRenderer,
    fingerprint: String,
    font_bytes: usize,
}

/// Rc marker deliberately keeps every Skia/D3D12 object on the creating thread.
pub struct NativeD3dPresenter {
    owner: ThreadId,
    generation: u64,
    backend: gpu::d3d::BackendContext,
    fence: ID3D12Fence,
    fence_value: u64,
    credit: ContextCredit,
    pending_gpu: bool,
    frame_active: bool,
    suspended: bool,
    quarantine_leaked: bool,
    quarantine_surface: Option<Rc<RefCell<Surface>>>,
    frame_inputs: Option<Rc<Vec<NativePresentationSnapshotBorrow>>>,
    config: NativeD3dPresenterConfig,
    resources: FrameResources<RefCell<ManagedTarget>, CachedTile>,
    target_budget: TargetBudget,
    tiles: TileCache,
    renderer: Option<Renderer>,
    context: Option<gpu::DirectContext>,
    submissions: u64,
    _owner_only: PhantomData<Rc<()>>,
}

impl NativeD3dPresenter {
    /// # Safety
    /// Pointers are live IDXGIAdapter1/ID3D12Device/ID3D12CommandQueue interfaces
    /// on the same single-node adapter. This dedicated direct queue is not used
    /// concurrently. Cloned COM references and the backend context remain owned
    /// until Skia is released/abandoned on this same producer OS thread.
    pub unsafe fn new(
        adapter: *const c_void,
        device: *const c_void,
        queue: *const c_void,
        generation: u64,
        config: NativeD3dPresenterConfig,
    ) -> Result<Self, NativeHostError> {
        config.validate()?;
        if generation == 0 {
            return Err(invalid("D3D device generation must be nonzero"));
        }
        let adapter: gpu::d3d::IDXGIAdapter1 = unsafe { retain_com(adapter)? };
        let device: ID3D12Device = unsafe { retain_com(device)? };
        let queue: ID3D12CommandQueue = unsafe { retain_com(queue)? };
        let desc = unsafe { adapter.GetDesc1() }
            .map_err(|_| unsupported("DXGI adapter description failed"))?;
        let luid = unsafe { device.GetAdapterLuid() };
        let mut queue_device: Option<ID3D12Device> = None;
        unsafe { queue.GetDevice(&mut queue_device) }
            .map_err(|_| invalid("D3D queue device query failed"))?;
        if luid.LowPart != desc.AdapterLuid.LowPart
            || luid.HighPart != desc.AdapterLuid.HighPart
            || unsafe { device.GetNodeCount() } != 1
            || unsafe { queue.GetDesc() }.Type != D3D12_COMMAND_LIST_TYPE_DIRECT
            || !same_device(
                &device,
                &queue_device.ok_or_else(|| invalid("D3D queue has no device"))?,
            )?
        {
            return Err(invalid(
                "D3D adapter LUID, queue device/type or node count mismatch",
            ));
        }
        let credit = ContextCredit::reserve()?;
        let fence: ID3D12Fence = unsafe { device.CreateFence(0, D3D12_FENCE_FLAG_NONE) }
            .map_err(|_| unsupported("D3D producer fence allocation failed"))?;
        let backend = gpu::d3d::BackendContext {
            adapter,
            device,
            queue,
            memory_allocator: None,
            protected_context: gpu::Protected::No,
        };
        let mut context = unsafe { gpu::direct_contexts::make_d3d(&backend, None) }
            .ok_or_else(|| unsupported("Skia could not create a D3D12 Ganesh context"))?;
        context.set_resource_cache_limit(config.resource_cache_bytes);
        Ok(Self {
            owner: thread::current().id(),
            generation,
            backend,
            fence,
            fence_value: 0,
            credit,
            pending_gpu: false,
            frame_active: false,
            suspended: false,
            quarantine_leaked: false,
            quarantine_surface: None,
            frame_inputs: None,
            config,
            resources: FrameResources::default(),
            target_budget: TargetBudget::new(
                config.max_managed_targets,
                config.max_managed_target_bytes,
            ),
            tiles: TileCache::default(),
            renderer: None,
            context: Some(context),
            submissions: 0,
            _owner_only: PhantomData,
        })
    }

    fn live(&self, generation: u64) -> Result<(), NativeHostError> {
        if thread::current().id() != self.owner {
            return Err(invalid("D3d presenter belongs to another thread"));
        }
        if generation != self.generation
            || self.context.is_none()
            || self.suspended
            || self.frame_active
            || self.pending_gpu
        {
            return Err(invalid("D3d presenter/device generation is no longer live"));
        }
        Ok(())
    }

    pub fn info(&self) -> Result<serde_json::Value, NativeHostError> {
        if thread::current().id() != self.owner {
            return Err(invalid("D3D presenter belongs to another thread"));
        }
        Ok(serde_json::json!({
            "backend":"d3d12Ganesh", "deviceGeneration":self.generation,
            "config":self.config, "scratchTargets":usize::from(self.resources.scratch.is_some()),
            "scratchEstimatedBytes":self.resources.scratch.as_ref().map_or(0, |v|v.borrow().bytes()),
            "managedTargets":self.target_budget.stats(), "tileCache":self.tiles.stats(),
            "workingTargets":self.resources.working_count(),
            "frameTilePins":self.resources.tile_count(),
            "rendererFontEstimatedBytes":self.renderer.as_ref().map_or(0, |v|v.font_bytes),
            "framesSubmitted":self.submissions, "wholeFrameReadbacks":0,
            "consumerRetirement":false, "hardwareAccelerationVerified":false,
            "state":if self.suspended {"quarantined"} else {"active"},
            "contextCredits":context_credits(),
            "quarantinedSnapshots":self.frame_inputs.as_ref().map_or(0, |v|v.len()),
        }))
    }

    fn renderer(
        &mut self,
        snapshot: &NativeRenderSnapshot,
    ) -> Result<&SkiaLayerRenderer, NativeHostError> {
        let fingerprint = &snapshot.font_state().fingerprint;
        if self
            .renderer
            .as_ref()
            .is_none_or(|v| &v.fingerprint != fingerprint)
        {
            let bytes = snapshot
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
                .ok_or_else(|| limit("D3d renderer font-copy estimate exceeds its quota"))?;
            self.renderer.take();
            let renderer = snapshot.create_renderer()?.with_cache_limits(
                super::MAX_RENDER_CACHE_ENTRIES,
                self.config.renderer_cache_bytes / 3,
            );
            self.renderer = Some(Renderer {
                renderer,
                fingerprint: fingerprint.clone(),
                font_bytes: bytes,
            });
        }
        Ok(&self
            .renderer
            .as_ref()
            .expect("initialized renderer")
            .renderer)
    }

    fn scratch(
        &mut self,
        target: NativeD3dTarget,
        bytes: usize,
    ) -> Result<Rc<RefCell<ManagedTarget>>, NativeHostError> {
        if let Some(scratch) = &self.resources.scratch {
            if scratch.borrow().width == target.width && scratch.borrow().height == target.height {
                return Ok(Rc::clone(scratch));
            }
        }
        // The previous frame completed before another render is admitted.
        // Drop its old dimensions before charging the replacement.
        self.resources.scratch.take();
        if !self.ensure_target_room(1, bytes) {
            return Err(limit("D3D scratch managed target quota exceeded"));
        }
        let allocation = self.target_budget.reserve(bytes)?;
        let scratch = Rc::new(RefCell::new(self.allocate_target(
            target.width,
            target.height,
            allocation,
        )?));
        self.resources.scratch = Some(Rc::clone(&scratch));
        Ok(scratch)
    }

    fn ensure_target_room(&mut self, count: usize, bytes: usize) -> bool {
        while !self.target_budget.can_reserve(count, bytes) {
            if !self.tiles.evict_idle() {
                return false;
            }
        }
        true
    }

    fn allocate_target(
        &mut self,
        width: i32,
        height: i32,
        allocation: TargetAllocation,
    ) -> Result<ManagedTarget, NativeHostError> {
        let info = ImageInfo::new(
            (width, height),
            ColorType::BGRA8888,
            AlphaType::Premul,
            ColorSpace::new_srgb(),
        );
        let surface = gpu::surfaces::render_target(
            self.context.as_mut().expect("checked live"),
            gpu::Budgeted::Yes,
            &info,
            None,
            gpu::SurfaceOrigin::TopLeft,
            None,
            false,
            None,
        )
        .ok_or_else(|| unsupported("Skia could not allocate a D3d managed target"))?;
        Ok(ManagedTarget::new(surface, width, height, allocation))
    }

    fn finish_gpu(&mut self, surface: &mut Surface) -> Result<(), NativeHostError> {
        self.pending_gpu = true;
        let context = self.context.as_mut().expect("checked live");
        // Both outcomes matter: a successful flush does not imply submission.
        let flushed = context.flush_surface(surface).is_ok();
        let submitted = context.submit(gpu::SyncCpu::Yes);
        if !flushed || !submitted || context.is_device_lost() {
            self.suspend();
            return Err(unsupported("D3d flush/submit failed; presenter abandoned"));
        }
        self.pending_gpu = false;
        Ok(())
    }

    unsafe fn validate_resource(
        &self,
        pointer: *const c_void,
        target: NativeD3dTarget,
    ) -> Result<(), NativeHostError> {
        let resource: ID3D12Resource = unsafe { retain_com(pointer)? };
        let desc = unsafe { resource.GetDesc() };
        let mut device: Option<ID3D12Device> = None;
        unsafe { resource.GetDevice(&mut device) }
            .map_err(|_| invalid("D3D resource device query failed"))?;
        let mut properties = D3D12_HEAP_PROPERTIES::default();
        let mut flags = D3D12_HEAP_FLAGS::default();
        unsafe { resource.GetHeapProperties(Some(&mut properties), Some(&mut flags)) }
            .map_err(|_| invalid("D3D resource heap query failed"))?;
        let required =
            D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET | D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS;
        if !same_device(
            &self.backend.device,
            &device.ok_or_else(|| invalid("D3D resource has no device"))?,
        )? || desc.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE2D
            || desc.Width != target.width as u64
            || desc.Height != target.height as u32
            || desc.DepthOrArraySize != 1
            || desc.MipLevels != 1
            || desc.SampleDesc.Count != 1
            || desc.SampleDesc.Quality != 0
            || desc.Format != DXGI_FORMAT_B8G8R8A8_UNORM
            || desc.Layout != D3D12_TEXTURE_LAYOUT_UNKNOWN
            || desc.Flags != required
            || properties.Type != D3D12_HEAP_TYPE_DEFAULT
            || properties.CreationNodeMask != 1
            || properties.VisibleNodeMask != 1
            || flags.0 & D3D12_HEAP_FLAG_SHARED.0 == 0
            || flags.0 & D3D12_HEAP_FLAG_SHARED_CROSS_ADAPTER.0 != 0
        {
            return Err(invalid(
                "D3D target device/format/flags/shared heap contract mismatch",
            ));
        }
        Ok(())
    }

    fn wait_producer_fence(&mut self) -> Result<(), NativeHostError> {
        let value = self
            .fence_value
            .checked_add(1)
            .ok_or_else(|| limit("D3D fence exhausted"))?;
        self.fence_value = value;
        self.pending_gpu = true;
        if unsafe { self.backend.queue.Signal(&self.fence, value) }.is_err() {
            self.suspend();
            return Err(unsupported("D3D producer queue fence signal failed"));
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let completed = unsafe { self.fence.GetCompletedValue() };
            if completed == u64::MAX
                || unsafe { self.backend.device.GetDeviceRemovedReason() }.is_err()
                || Instant::now() >= deadline
            {
                self.suspend();
                return Err(unsupported(
                    "D3D producer completion unknown; target must be quarantined",
                ));
            }
            if completed >= value {
                self.pending_gpu = false;
                return Ok(());
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    /// # Safety
    /// texture is a live ID3D12Resource, matching this device and the fixed
    /// target contract above. It remains retained and exclusively writable for
    /// the entire call. Publish only a successful, still-current result. Never
    /// mutate/recycle it while an external consumer might sample it.
    pub unsafe fn render(
        &mut self,
        registry: &NativePresentationRegistry,
        stamp: NativePresentationStamp,
        texture: *const c_void,
        target: NativeD3dTarget,
        placements: &[NativeD3dPlacement],
        clear_color: u32,
    ) -> Result<NativeD3dFrame, NativeHostError> {
        self.live(target.device_generation)?;
        stamp.validate()?;
        if texture.is_null() || stamp.device_generation != self.generation {
            return Err(invalid("D3d texture/frame device generation mismatch"));
        }
        let bytes = self.config.target_bytes(target)?;
        unsafe { self.validate_resource(texture, target)? };
        if placements.is_empty() || placements.len() > self.config.max_pages {
            return Err(limit("D3d placement count is outside its quota"));
        }
        let inputs: Vec<NativePresentationSnapshotBorrow> = placements
            .iter()
            .map(|placement| {
                let input = registry.borrow_snapshot(placement.snapshot, stamp)?;
                if input.snapshot().profile() != crate::paint::RenderProfile::Screen {
                    return Err(invalid(
                        "D3D presentation snapshots require the screen profile",
                    ));
                }
                placement.validate(input.snapshot())?;
                Ok(input)
            })
            .collect::<Result<_, NativeHostError>>()?;
        if inputs
            .iter()
            .any(|input| input.source() != inputs[0].source())
        {
            return Err(invalid("all D3d pages must share one presentation source"));
        }
        if self.submissions == u64::MAX || self.fence_value == u64::MAX {
            return Err(limit("D3d submission sequence exhausted"));
        }
        // Keep immutable input and every GPU surface in the owner before any
        // work is recorded, including through a caught Rust panic.
        let inputs = Rc::new(inputs);
        self.frame_inputs = Some(Rc::clone(&inputs));
        self.frame_active = true;
        self.pending_gpu = true;
        let scratch = match self.scratch(target, bytes) {
            Ok(scratch) => scratch,
            Err(error) => {
                self.suspend();
                return Err(error);
            }
        };
        let mut scratch = scratch.borrow_mut();
        let mut result = self.replay_and_publish(
            &mut scratch,
            stamp,
            texture,
            target,
            placements,
            &inputs,
            clear_color,
        );
        if result.is_err() && !self.suspended {
            // A later canonical completion can clear pending_gpu while an
            // earlier tile is still recorded on scratch. Always drain failed
            // scratch work before releasing its pins; never publish it.
            if let Err(error) = self.finish_gpu(&mut scratch.surface) {
                // Never return staleFrame's safe-discard receipt when the
                // producer's cleanup completion became unknown.
                result = Err(error);
            }
        }
        if !self.suspended && !self.pending_gpu {
            self.resources.complete_frame();
            self.frame_inputs.take();
            self.quarantine_surface.take();
            self.frame_active = false;
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn replay_and_publish(
        &mut self,
        scratch: &mut ManagedTarget,
        stamp: NativePresentationStamp,
        texture: *const c_void,
        target: NativeD3dTarget,
        placements: &[NativeD3dPlacement],
        inputs: &[NativePresentationSnapshotBorrow],
        clear_color: u32,
    ) -> Result<NativeD3dFrame, NativeHostError> {
        self.pending_gpu = true;
        scratch.surface.canvas().clear(Color::from(clear_color));
        for (placement, input) in placements.iter().zip(inputs) {
            if !self.replay_tiles(
                &mut scratch.surface,
                stamp,
                target,
                *placement,
                input,
                clear_color,
            )? {
                self.tiles.observations.viewport_fallbacks =
                    self.tiles.observations.viewport_fallbacks.saturating_add(1);
                self.pending_gpu = true;
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
        self.finish_gpu(&mut scratch.surface)?;
        let image = scratch.surface.image_snapshot();
        if !image.is_texture_backed() {
            return Err(unsupported("D3d scratch snapshot is not GPU backed"));
        }
        // Ganesh refs the D3D12 resource; no raw pointer is encoded in JSON.
        let resource: ID3D12Resource = unsafe { retain_com(texture)? };
        let info = gpu::d3d::TextureResourceInfo {
            resource,
            alloc: None,
            resource_state: D3D12_RESOURCE_STATE_COMMON,
            format: DXGI_FORMAT_B8G8R8A8_UNORM,
            sample_count: 1,
            level_count: 1,
            sample_quality_pattern: 0,
            protected: gpu::Protected::No,
        };
        let backend = gpu::backend_render_targets::make_d3d((target.width, target.height), &info);
        let visible = gpu::surfaces::wrap_backend_render_target(
            self.context.as_mut().expect("checked live"),
            &backend,
            gpu::SurfaceOrigin::TopLeft,
            ColorType::BGRA8888,
            ColorSpace::new_srgb(),
            None,
        )
        .ok_or_else(|| unsupported("Skia could not borrow the D3d BGRA texture"))?;
        for input in inputs {
            input.validate_current(stamp)?;
        }
        let visible = Rc::new(RefCell::new(visible));
        self.quarantine_surface = Some(Rc::clone(&visible));
        let mut copy = Paint::default();
        copy.set_blend_mode(BlendMode::Src);
        visible
            .borrow_mut()
            .canvas()
            .draw_image(&image, (0.0, 0.0), Some(&copy));
        self.finish_gpu(&mut visible.borrow_mut())?;
        self.wait_producer_fence()?;
        // ALLOW_SIMULTANEOUS_ACCESS textures decay to COMMON when ExecuteCommandLists
        // completes, even after explicit transitions. The queue fence proves that
        // completion before another D3D11 device opens/reads this shared resource.
        // A source invalidated during the GPU wait is never handed to Flutter.
        for input in inputs {
            input.validate_current(stamp)?;
        }
        self.submissions += 1;
        Ok(NativeD3dFrame {
            backend: "d3d12Ganesh",
            stamp,
            target,
            page_count: placements.len(),
            submission: self.submissions,
            pixel_format: "bgra8UnormPremultiplied",
            color_space: "srgb",
            alpha: "premultiplied",
            origin: "topLeft",
            producer_gpu_complete: true,
            resource_common: true,
            producer_fence_value: self.fence_value,
            consumer_gpu_complete: false,
            whole_frame_readbacks: 0,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn replay_tiles(
        &mut self,
        scratch: &mut Surface,
        stamp: NativePresentationStamp,
        target: NativeD3dTarget,
        placement: NativeD3dPlacement,
        input: &NativePresentationSnapshotBorrow,
        clear_color: u32,
    ) -> Result<bool, NativeHostError> {
        let Some(plan) = TilePlan::new(
            self.config.tile_config(),
            stamp,
            TileTarget {
                context_id: self.generation,
                width: target.width,
                height: target.height,
            },
            TilePlacement {
                scale: placement.scale,
                origin_x: placement.origin_x,
                origin_y: placement.origin_y,
                background_color: placement.background_color,
            },
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
            while !self.tiles.has_room(
                self.config.max_cached_tiles,
                self.config.max_tile_bytes,
                missing_count,
                missing_bytes,
            ) {
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
                .ok_or_else(|| limit("D3D tile working target byte overflow"))?;
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
            let canonical = Rc::new(RefCell::new(self.allocate_target(
                plan.width,
                plan.height,
                canonical_allocation,
            )?));
            self.resources.pin_working(&canonical);
            self.pending_gpu = true;
            canonical
                .borrow_mut()
                .surface
                .canvas()
                .clear(Color::TRANSPARENT);
            self.tiles.observations.canonical_replays =
                self.tiles.observations.canonical_replays.saturating_add(1);
            let replay = self.renderer(input.snapshot()).and_then(|renderer| {
                renderer
                    .render_to_surface_strict(
                        &mut canonical.borrow_mut().surface,
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
                // A full-page staging failure must not break the previously
                // valid viewport path. Drain before allowing that fallback.
                self.finish_gpu(&mut canonical.borrow_mut().surface)?;
                self.resources.complete_working();
                if matches!(
                    error,
                    NativeHostError::Core(_)
                        | NativeHostError::InvalidArgument(_)
                        | NativeHostError::LimitExceeded(_)
                ) {
                    self.tiles.observations.canonical_failures =
                        self.tiles.observations.canonical_failures.saturating_add(1);
                    return Ok(false);
                }
                return Err(error);
            }
            self.finish_gpu(&mut canonical.borrow_mut().surface)?;
            let image = canonical.borrow_mut().surface.image_snapshot();
            if !image.is_texture_backed() {
                return Err(unsupported("D3D canonical tile input is not GPU backed"));
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
                let tile = Rc::new(RefCell::new(
                    self.allocate_target(
                        region.width,
                        region.height,
                        allocations
                            .next()
                            .expect("one reservation per missing tile"),
                    )?,
                ));
                self.resources.pin_working(&tile);
                self.pending_gpu = true;
                tile.borrow_mut().surface.canvas().draw_image(
                    &image,
                    (-region.x as f32, -region.y as f32),
                    Some(&copy),
                );
                self.finish_gpu(&mut tile.borrow_mut().surface)?;
                prepared.push((
                    index,
                    Rc::new(CachedTile {
                        key: key.clone(),
                        target: tile,
                    }),
                ));
            }
            input.validate_current(stamp)?;
            // Neither an incomplete canonical replay nor one copied tile may
            // become a cache hit. All entries share the same current input.
            for (index, tile) in prepared {
                found[index] = Some(Rc::clone(&tile));
                self.tiles.insert(tile);
            }
            self.resources.complete_working();
        }
        // A cache key intentionally survives view-generation changes for
        // integer scrolling; the full current stamp is still mandatory.
        input.validate_current(stamp)?;
        for tile in found.into_iter().flatten() {
            self.resources.pin_tile(&tile);
            let image = tile.target.borrow_mut().surface.image_snapshot();
            if !image.is_texture_backed() {
                return Err(unsupported("cached D3D tile is not GPU backed"));
            }
            self.pending_gpu = true;
            scratch.canvas().draw_image(
                &image,
                (
                    (plan.integer_x + tile.key.region.x) as f32,
                    (plan.integer_y + tile.key.region.y) as f32,
                ),
                None,
            );
        }
        Ok(true)
    }

    pub fn release(&mut self) -> Result<(), NativeHostError> {
        self.live(self.generation)?;
        self.resources.clear_completed();
        self.tiles.clear();
        self.renderer.take();
        if let Some(context) = &mut self.context {
            context.release_resources_and_abandon();
        }
        self.context.take();
        Ok(())
    }

    pub fn abandon(&mut self) {
        if self.pending_gpu || self.suspended || self.frame_active {
            self.suspend();
            if self.quarantine_leaked {
                return;
            }
            self.quarantine_leaked = true;
            // D3D12 does not retain application resources until GPU completion.
            // Keep this bounded opaque state alive through process lifetime.
            std::mem::forget(std::mem::take(&mut self.tiles));
            std::mem::forget(self.renderer.take());
            std::mem::forget(self.quarantine_surface.take());
            std::mem::forget(self.frame_inputs.take());
            std::mem::forget(self.context.take());
            std::mem::forget(self.backend.clone());
            std::mem::forget(self.fence.clone());
            return;
        }
        if let Some(context) = &mut self.context {
            context.abandon();
        }
        self.resources.clear_completed();
        self.tiles.clear();
        self.renderer.take();
        self.context.take();
    }

    fn suspend(&mut self) {
        self.suspended = true;
        self.resources.quarantine();
        self.credit.quarantine();
    }
}

impl Drop for NativeD3dPresenter {
    fn drop(&mut self) {
        self.abandon();
    }
}
