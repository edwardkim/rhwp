//! Opt-in headless Vulkan host: exercise borrowed GPU surfaces, without a window.
//!
//! Follows rust-skia's Vulkan backend-context/resolver examples. Readback below
//! is a verification operation performed by this host, not by the renderer API.

use std::error::Error;
use std::ffi::CStr;
use std::io;

use ash::vk::{self, Handle};
use rhwp::paint::{LayerNode, PageLayerTree, PaintOp};
use rhwp::renderer::render_tree::{BoundingBox, RectangleNode};
use rhwp::renderer::skia::{SkiaLayerRenderer, SkiaSurfaceRenderOptions};
use rhwp::renderer::ShapeStyle;
use skia_safe::{gpu, AlphaType, Color, ColorType, ImageInfo};

type ProbeResult<T> = Result<T, Box<dyn Error>>;

struct VulkanHost {
    entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,
    physical: vk::PhysicalDevice,
    queue: vk::Queue,
    family: u32,
    name: String,
    software_device: bool,
}

impl VulkanHost {
    fn new() -> ProbeResult<Self> {
        // SAFETY: ash owns the loader and keeps its library loaded in `entry`.
        let entry = unsafe { ash::Entry::load()? };
        // Ganesh's documented minimum, also passed to Skia's backend builder.
        let app = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_1);
        let create = vk::InstanceCreateInfo::default().application_info(&app);
        // SAFETY: no extension/layer pointers are supplied; app lives through call.
        let instance = unsafe { entry.create_instance(&create, None)? };
        let initialized = (|| -> ProbeResult<_> {
            // SAFETY: the instance is valid and remains alive below.
            let physicals = unsafe { instance.enumerate_physical_devices()? };
            let mut selected = None;
            for physical in physicals {
                // SAFETY: enumeration returned this physical device for instance.
                let properties = unsafe { instance.get_physical_device_properties(physical) };
                if properties.api_version < vk::API_VERSION_1_1 {
                    continue;
                }
                // SAFETY: same live instance and enumerated physical device.
                let families =
                    unsafe { instance.get_physical_device_queue_family_properties(physical) };
                if let Some(family) = families.iter().position(|family| {
                    family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                }) {
                    selected = Some((physical, family as u32, properties));
                    break;
                }
            }
            let (physical, family, properties) = selected.ok_or_else(|| {
                io::Error::other("no Vulkan 1.1 graphics device; probe did not run")
            })?;
            let priorities = [1.0];
            let queues = [vk::DeviceQueueCreateInfo::default()
                .queue_family_index(family)
                .queue_priorities(&priorities)];
            let create = vk::DeviceCreateInfo::default().queue_create_infos(&queues);
            // SAFETY: selected device/family are valid; backing slices live through call.
            let device = unsafe { instance.create_device(physical, &create, None)? };
            // SAFETY: queue zero was created above for this family.
            let queue = unsafe { device.get_device_queue(family, 0) };
            // SAFETY: Vulkan guarantees a nul-terminated device_name array.
            let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            Ok((
                device,
                physical,
                queue,
                family,
                name,
                properties.device_type == vk::PhysicalDeviceType::CPU,
            ))
        })();
        match initialized {
            Ok((device, physical, queue, family, name, software_device)) => Ok(Self {
                entry,
                instance,
                device,
                physical,
                queue,
                family,
                name,
                software_device,
            }),
            Err(error) => {
                // SAFETY: device initialization failed; no dependent object survives.
                unsafe { instance.destroy_instance(None) };
                Err(error)
            }
        }
    }
}

impl Drop for VulkanHost {
    fn drop(&mut self) {
        // SAFETY: verify() has returned and all Skia objects are already dropped.
        // These handles are owned by this host and destroyed exactly once.
        unsafe {
            let _ = self.device.device_wait_idle();
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn rectangle(bounds: BoundingBox, color: u32) -> PaintOp {
    PaintOp::rectangle(
        bounds,
        RectangleNode::new(
            0.0,
            ShapeStyle {
                fill_color: Some(color),
                ..Default::default()
            },
            None,
        ),
    )
}

fn verify(host: &VulkanHost) -> ProbeResult<()> {
    // The host, resolver and Vulkan handles outlive every Skia object below.
    let resolver = |of: gpu::vk::GetProcOf| -> *const std::os::raw::c_void {
        // SAFETY: Skia passes handles from this backend and valid symbol names.
        unsafe {
            match of {
                gpu::vk::GetProcOf::Instance(instance, name) => host
                    .entry
                    .get_instance_proc_addr(vk::Instance::from_raw(instance as u64), name),
                gpu::vk::GetProcOf::Device(device, name) => host
                    .instance
                    .get_device_proc_addr(vk::Device::from_raw(device as u64), name),
            }
            .map_or(std::ptr::null(), |function| function as *const _)
        }
    };
    // SAFETY: host handles and resolver outlive backend, context, surface.
    let backend = unsafe {
        gpu::vk::BackendContext::new_builder(
            host.instance.handle().as_raw() as gpu::vk::Instance,
            host.physical.as_raw() as gpu::vk::PhysicalDevice,
            host.device.handle().as_raw() as gpu::vk::Device,
            (host.queue.as_raw() as gpu::vk::Queue, host.family as usize),
            &resolver,
            Some(vk::API_VERSION_1_1.into()),
        )
        .build()
    };
    let mut context = gpu::direct_contexts::make_vulkan(&backend, None).ok_or_else(|| {
        io::Error::other("Skia Vulkan context creation failed; probe did not run")
    })?;
    assert_eq!(context.backend(), gpu::BackendAPI::Vulkan);
    let info = ImageInfo::new((64, 48), ColorType::RGBA8888, AlphaType::Premul, None);
    let mut surface = gpu::surfaces::render_target(
        &mut context,
        gpu::Budgeted::Yes,
        &info,
        Some(0),
        Some(gpu::SurfaceOrigin::TopLeft),
        None,
        false,
        false,
    )
    .ok_or_else(|| io::Error::other("Skia Vulkan render target creation failed"))?;
    assert_eq!(
        surface.recording_context().map(|context| context.backend()),
        Some(gpu::BackendAPI::Vulkan)
    );
    surface.canvas().clear(Color::WHITE);
    surface.canvas().save();
    surface.canvas().translate((3.0, 2.0));
    surface.canvas().clip_rect(
        skia_safe::Rect::from_xywh(0.0, 0.0, 46.0, 40.0),
        None,
        false,
    );
    let before = (
        surface.canvas().save_count(),
        surface.canvas().local_to_device(),
        surface.canvas().device_clip_bounds(),
    );
    let tree = PageLayerTree::new(
        32.0,
        24.0,
        LayerNode::leaf(
            BoundingBox::new(0.0, 0.0, 32.0, 24.0),
            None,
            vec![
                rectangle(BoundingBox::new(4.0, 4.0, 8.0, 6.0), 0x0000ff),
                rectangle(BoundingBox::new(12.0, 8.0, 12.0, 8.0), 0xff0000),
            ],
        ),
    );
    let renderer = SkiaLayerRenderer::new();
    let options = SkiaSurfaceRenderOptions {
        scale: 1.25,
        origin_x: 5.0,
        origin_y: 6.0,
        background_color: Some(0x00ff00),
    };
    renderer
        .render_to_surface_strict(&mut surface, &tree, options)
        .map_err(|error| io::Error::other(error.to_string()))?;
    assert_eq!(
        before,
        (
            surface.canvas().save_count(),
            surface.canvas().local_to_device(),
            surface.canvas().device_clip_bounds()
        )
    );
    // Synchronization and readback are explicitly owned by this verification host.
    context.flush_and_submit_surface(&mut surface, gpu::SyncCpu::Yes);
    let mut bytes = vec![0; info.min_row_bytes() * info.height() as usize];
    assert!(surface.read_pixels(&info, &mut bytes, info.min_row_bytes(), (0, 0)));
    for (x, y, expected) in [
        (16, 15, [255, 0, 0, 255]),
        (30, 22, [0, 0, 255, 255]),
        (10, 10, [0, 255, 0, 255]),
        (4, 4, [255, 255, 255, 255]),
    ] {
        let offset = y * info.min_row_bytes() + x * 4;
        assert_eq!(&bytes[offset..offset + 4], &expected, "pixel ({x}, {y})");
    }
    assert!(renderer
        .render_to_surface_strict(
            &mut surface,
            &tree,
            SkiaSurfaceRenderOptions {
                scale: f64::NAN,
                ..options
            },
        )
        .is_err());
    let mut after_error = vec![0; bytes.len()];
    context.flush_and_submit_surface(&mut surface, gpu::SyncCpu::Yes);
    assert!(surface.read_pixels(&info, &mut after_error, info.min_row_bytes(), (0, 0)));
    assert_eq!(
        bytes, after_error,
        "invalid arguments must not clear a GPU target"
    );
    // Local declaration order ensures surface/context/backend drop before host.
    Ok(())
}

fn main() -> ProbeResult<()> {
    let host = VulkanHost::new()?;
    verify(&host)?;
    println!(
        "{}",
        serde_json::json!({
            "status": "passed",
            "backend": "Vulkan",
            "device": &host.name,
            "softwareDevice": host.software_device,
            "hardwareAccelerationVerified": false,
            "borrowedGpuSurface": true,
            "scale": 1.25,
            "readback": "verification host only",
            "checks": ["known interior colors", "host state restoration", "invalid arguments preserve pixels"],
        })
    );
    Ok(())
}
