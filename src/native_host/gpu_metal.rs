//! Owner-thread Metal/Ganesh producer. Host texture pointers never enter JSON.
//!
//! Strict replay finishes in a GPU scratch surface before the host-owned BGRA
//! target is touched. Successful return waits for this producer queue's work;
//! it says nothing about Flutter's later consumer queue or texture recycling.

use std::ffi::c_void;
use std::marker::PhantomData;
use std::rc::Rc;
use std::thread::{self, ThreadId};

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

const MIB: usize = 1024 * 1024;
const MAX_COORDINATE: f64 = 16_777_216.0;

fn invalid(message: &str) -> NativeHostError {
    NativeHostError::InvalidArgument(message.into())
}
fn limit(message: &str) -> NativeHostError {
    NativeHostError::LimitExceeded(message.into())
}
fn unsupported(message: &str) -> NativeHostError {
    NativeHostError::Unsupported(message.into())
}

/// Logical target/font/cache limits, not total driver VRAM or process RSS.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeMetalPresenterConfig {
    pub max_dimension: i32,
    pub max_pixels: usize,
    pub max_pages: usize,
    pub max_target_bytes: usize,
    pub max_renderer_font_bytes: usize,
    pub renderer_cache_bytes: usize,
    pub resource_cache_bytes: usize,
}

impl Default for NativeMetalPresenterConfig {
    fn default() -> Self {
        Self {
            max_dimension: 4096,
            max_pixels: 4 * 1024 * 1024,
            max_pages: 8,
            max_target_bytes: 16 * MIB,
            max_renderer_font_bytes: 64 * MIB,
            renderer_cache_bytes: 8 * MIB,
            resource_cache_bytes: 32 * MIB,
        }
    }
}

impl NativeMetalPresenterConfig {
    pub fn validate(self) -> Result<(), NativeHostError> {
        if !(1..=4096).contains(&self.max_dimension)
            || !(1..=4 * 1024 * 1024).contains(&self.max_pixels)
            || !(1..=8).contains(&self.max_pages)
            || !(1..=16 * MIB).contains(&self.max_target_bytes)
            || !(1..=64 * MIB).contains(&self.max_renderer_font_bytes)
            || self.renderer_cache_bytes > 8 * MIB
            || self.resource_cache_bytes > 32 * MIB
        {
            return Err(invalid("invalid Metal presenter limits"));
        }
        Ok(())
    }

    fn target_bytes(self, target: NativeMetalTarget) -> Result<usize, NativeHostError> {
        self.validate()?;
        if target.device_generation == 0
            || target.width <= 0
            || target.height <= 0
            || target.width > self.max_dimension
            || target.height > self.max_dimension
        {
            return Err(invalid("invalid Metal target generation or dimensions"));
        }
        (target.width as usize)
            .checked_mul(target.height as usize)
            .filter(|pixels| *pixels <= self.max_pixels)
            .and_then(|pixels| pixels.checked_mul(4))
            .filter(|bytes| *bytes <= self.max_target_bytes)
            .ok_or_else(|| limit("Metal target pixel/byte quota exceeded"))
    }
}

/// Host promises a matching-device, single-sample, BGRA8Unorm 2D render target
/// with shader-read/render-target usage and no concurrent producer/consumer.
/// Format, premultiplication and top-left origin are fixed by ABI version 1.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeMetalTarget {
    pub device_generation: u64,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeMetalPlacement {
    pub snapshot: NativePresentationId,
    /// Pixel scale, including deviceScale. Origins are target pixel positions.
    pub scale: f64,
    pub origin_x: f64,
    pub origin_y: f64,
    pub background_color: Option<u32>,
}

impl NativeMetalPlacement {
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
            return Err(invalid("invalid Metal page placement"));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeMetalFrame {
    pub backend: &'static str,
    pub stamp: NativePresentationStamp,
    pub target: NativeMetalTarget,
    pub page_count: usize,
    pub submission: u64,
    pub pixel_format: &'static str,
    pub color_space: &'static str,
    pub alpha: &'static str,
    pub origin: &'static str,
    pub producer_gpu_complete: bool,
    pub consumer_gpu_complete: bool,
    pub whole_frame_readbacks: u64,
}

struct Scratch {
    surface: Surface,
    width: i32,
    height: i32,
    bytes: usize,
}
struct Renderer {
    renderer: SkiaLayerRenderer,
    fingerprint: String,
    font_bytes: usize,
}

/// Rc marker deliberately keeps every Skia/Metal object on the creating thread.
pub struct NativeMetalPresenter {
    owner: ThreadId,
    generation: u64,
    config: NativeMetalPresenterConfig,
    scratch: Option<Scratch>,
    renderer: Option<Renderer>,
    context: Option<gpu::DirectContext>,
    submissions: u64,
    _owner_only: PhantomData<Rc<()>>,
}

impl NativeMetalPresenter {
    /// # Safety
    /// device/queue are live id<MTLDevice>/id<MTLCommandQueue> pointers, with a
    /// matching device. Skia retains both; the caller must not pass arbitrary
    /// integer addresses or share this presenter between OS threads.
    pub unsafe fn new(
        device: *const c_void,
        queue: *const c_void,
        generation: u64,
        config: NativeMetalPresenterConfig,
    ) -> Result<Self, NativeHostError> {
        config.validate()?;
        if device.is_null() || queue.is_null() || generation == 0 {
            return Err(invalid("Metal device/queue/generation must be nonzero"));
        }
        // skia-safe 0.156 BackendContext::new and make_metal each retain their
        // own references. No borrowed Objective-C pointer is stored in JSON.
        let backend = unsafe { gpu::mtl::BackendContext::new(device, queue) };
        let mut context = gpu::direct_contexts::make_metal(&backend, None)
            .ok_or_else(|| unsupported("Skia could not create a Metal Ganesh context"))?;
        context.set_resource_cache_limit(config.resource_cache_bytes);
        Ok(Self {
            owner: thread::current().id(),
            generation,
            config,
            scratch: None,
            renderer: None,
            context: Some(context),
            submissions: 0,
            _owner_only: PhantomData,
        })
    }

    fn live(&self, generation: u64) -> Result<(), NativeHostError> {
        if thread::current().id() != self.owner {
            return Err(invalid("Metal presenter belongs to another thread"));
        }
        if generation != self.generation || self.context.is_none() {
            return Err(invalid(
                "Metal presenter/device generation is no longer live",
            ));
        }
        Ok(())
    }

    pub fn info(&self) -> Result<serde_json::Value, NativeHostError> {
        self.live(self.generation)?;
        Ok(serde_json::json!({
            "backend":"metalGanesh", "deviceGeneration":self.generation,
            "config":self.config, "scratchTargets":usize::from(self.scratch.is_some()),
            "scratchEstimatedBytes":self.scratch.as_ref().map_or(0, |v|v.bytes),
            "rendererFontEstimatedBytes":self.renderer.as_ref().map_or(0, |v|v.font_bytes),
            "framesSubmitted":self.submissions, "wholeFrameReadbacks":0,
            "consumerRetirement":false, "hardwareAccelerationVerified":false,
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
                .ok_or_else(|| limit("Metal renderer font-copy estimate exceeds its quota"))?;
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
        target: NativeMetalTarget,
        bytes: usize,
    ) -> Result<Scratch, NativeHostError> {
        if let Some(scratch) = self.scratch.take() {
            if scratch.width == target.width && scratch.height == target.height {
                return Ok(scratch);
            }
            // Drop old allocation before creating its replacement.
        }
        let info = ImageInfo::new(
            (target.width, target.height),
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
        .ok_or_else(|| unsupported("Skia could not allocate a Metal scratch target"))?;
        Ok(Scratch {
            surface,
            width: target.width,
            height: target.height,
            bytes,
        })
    }

    fn finish_gpu(&mut self, surface: &mut Surface) -> Result<(), NativeHostError> {
        let context = self.context.as_mut().expect("checked live");
        // Both outcomes matter: a successful flush does not imply submission.
        let flushed = context.flush_surface(surface).is_ok();
        let submitted = context.submit(gpu::SyncCpu::Yes);
        if !flushed || !submitted || context.is_device_lost() {
            self.abandon();
            return Err(unsupported(
                "Metal flush/submit failed; presenter abandoned",
            ));
        }
        Ok(())
    }

    /// # Safety
    /// texture is a live id<MTLTexture>, matching this device and the fixed
    /// target contract above. It remains retained and exclusively writable for
    /// the entire call. Publish only a successful, still-current result. Never
    /// mutate/recycle it while an external consumer might sample it.
    pub unsafe fn render(
        &mut self,
        registry: &NativePresentationRegistry,
        stamp: NativePresentationStamp,
        texture: *const c_void,
        target: NativeMetalTarget,
        placements: &[NativeMetalPlacement],
        clear_color: u32,
    ) -> Result<NativeMetalFrame, NativeHostError> {
        self.live(target.device_generation)?;
        stamp.validate()?;
        if texture.is_null() || stamp.device_generation != self.generation {
            return Err(invalid("Metal texture/frame device generation mismatch"));
        }
        let bytes = self.config.target_bytes(target)?;
        if placements.is_empty() || placements.len() > self.config.max_pages {
            return Err(limit("Metal placement count is outside its quota"));
        }
        let inputs: Vec<NativePresentationSnapshotBorrow> = placements
            .iter()
            .map(|placement| {
                let input = registry.borrow_snapshot(placement.snapshot, stamp)?;
                placement.validate(input.snapshot())?;
                Ok(input)
            })
            .collect::<Result<_, NativeHostError>>()?;
        if inputs
            .iter()
            .any(|input| input.source() != inputs[0].source())
        {
            return Err(invalid(
                "all Metal pages must share one presentation source",
            ));
        }
        if self.submissions == u64::MAX {
            return Err(limit("Metal submission sequence exhausted"));
        }
        let mut scratch = self.scratch(target, bytes)?;
        let result = self.replay_and_publish(
            &mut scratch,
            stamp,
            texture,
            target,
            placements,
            &inputs,
            clear_color,
        );
        if self.context.is_some() {
            self.scratch = Some(scratch);
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn replay_and_publish(
        &mut self,
        scratch: &mut Scratch,
        stamp: NativePresentationStamp,
        texture: *const c_void,
        target: NativeMetalTarget,
        placements: &[NativeMetalPlacement],
        inputs: &[NativePresentationSnapshotBorrow],
        clear_color: u32,
    ) -> Result<NativeMetalFrame, NativeHostError> {
        scratch.surface.canvas().clear(Color::from(clear_color));
        for (placement, input) in placements.iter().zip(inputs) {
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
        self.finish_gpu(&mut scratch.surface)?;
        let image = scratch.surface.image_snapshot();
        if !image.is_texture_backed() {
            return Err(unsupported("Metal scratch snapshot is not GPU backed"));
        }
        // Retains the host texture until all producer GPU accesses have ended.
        let texture_info = unsafe { gpu::mtl::TextureInfo::new(texture) };
        let backend =
            gpu::backend_render_targets::make_mtl((target.width, target.height), &texture_info);
        let mut visible = gpu::surfaces::wrap_backend_render_target(
            self.context.as_mut().expect("checked live"),
            &backend,
            gpu::SurfaceOrigin::TopLeft,
            ColorType::BGRA8888,
            ColorSpace::new_srgb(),
            None,
        )
        .ok_or_else(|| unsupported("Skia could not borrow the Metal BGRA texture"))?;
        for input in inputs {
            input.validate_current(stamp)?;
        }
        let mut copy = Paint::default();
        copy.set_blend_mode(BlendMode::Src);
        visible.canvas().draw_image(&image, (0.0, 0.0), Some(&copy));
        self.finish_gpu(&mut visible)?;
        // A source invalidated during the GPU wait is never handed to Flutter.
        for input in inputs {
            input.validate_current(stamp)?;
        }
        self.submissions += 1;
        Ok(NativeMetalFrame {
            backend: "metalGanesh",
            stamp,
            target,
            page_count: placements.len(),
            submission: self.submissions,
            pixel_format: "bgra8UnormPremultiplied",
            color_space: "srgb",
            alpha: "premultiplied",
            origin: "topLeft",
            producer_gpu_complete: true,
            consumer_gpu_complete: false,
            whole_frame_readbacks: 0,
        })
    }

    pub fn release(&mut self) -> Result<(), NativeHostError> {
        self.live(self.generation)?;
        self.scratch.take();
        self.renderer.take();
        if let Some(context) = &mut self.context {
            context.release_resources_and_abandon();
        }
        self.context.take();
        Ok(())
    }

    pub fn abandon(&mut self) {
        if let Some(context) = &mut self.context {
            context.abandon();
        }
        self.scratch.take();
        self.renderer.take();
        self.context.take();
    }
}

impl Drop for NativeMetalPresenter {
    fn drop(&mut self) {
        self.abandon();
    }
}
