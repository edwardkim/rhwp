//! GPU 가속 SVG 래스터화 경로 (`feature = "gpu"`, 네이티브 전용).
//!
//! # 무엇을 GPU로 옮기는가 — 그리고 무엇은 아닌가
//!
//! rhwp의 파싱·레이아웃은 분기 지배적(branch-bound) 이라 GPU로 가속되지 않는다. 이 모듈은
//! 그 경계를 넘지 않는다 — 레이아웃은 손대지 않고, 기존 SVG 산출(`render_page_svg_native`)이
//! 이미 만들어 놓은 벡터 표현을 **픽셀로 굽는 단계**(rasterization) 만 GPU로 옮긴다. 이 단계는
//! 픽셀마다 독립적이라 데이터 병렬성이 크고, 문서 코퍼스를 대량으로 이미지화해 비전 모델
//! (VLM)에 먹이는 에이전트 파이프라인에서 실제 병목이 되는 곳이다.
//!
//! # 파이프라인
//!
//! ```text
//! HWP/HWPX ──(파싱·레이아웃, CPU)──▶ SVG 문자열 ──(usvg 파싱)──▶ usvg::Tree
//!                                                                     │
//!                    ┌────────────────────────────────────────────────┤
//!                    ▼                                                 ▼
//!        vello_svg → vello::Scene                            resvg (CPU 기준선)
//!        → wgpu 컴퓨트 래스터 → 텍스처 → 리드백 → RGBA          → tiny_skia Pixmap → RGBA
//! ```
//!
//! `usvg::Tree` 는 한 번만 파싱해 GPU·CPU 두 경로에 **동일 입력**으로 넣는다. resvg 0.45 와
//! vello_svg 0.7 이 같은 usvg 0.45 를 공유하므로(카고가 단일 노드로 통합) 벤치마크가
//! 순수 래스터화 단계만 비교하게 되고, 픽셀 비교도 의미를 가진다(같은 벡터 → 두 래스터라이저).
//!
//! # 정직성
//!
//! GPU 컨텍스트(어댑터·디바이스·셰이더 컴파일) 생성은 수백 ms가 드는 **일회성 비용**이다.
//! 따라서 [`GpuContext`] 는 한 번만 만들어 배치 전체에서 재사용해야 하며, 벤치마크는 이
//! 일회성 비용을 별도로 보고한다. 소규모 문서 한두 장이라면 이 초기화 비용이 지배해 GPU가
//! 오히려 느릴 수 있다 — 이득은 대량 배치·고해상도에서 나온다. 이 모듈은 그 사실을 숨기지
//! 않고 측정해 드러낸다.

use std::path::PathBuf;

// Cargo 에서 `resvg-gpu`(package = resvg, 0.45)로 이름을 바꿔 가져온다 — native-skia 의
// resvg 0.47 과 버전이 다르기 때문이다. 0.45 로 핀하는 이유는 vello_svg 0.7 과 usvg 0.45 를
// **공유**하기 위함이다(카고가 단일 노드로 통합). 그래야 하나의 `usvg::Tree` 를 GPU·CPU 두
// 래스터라이저에 동일 입력으로 넣을 수 있다. `tiny_skia`·`usvg` 는 resvg 가 재수출한다.
use resvg_gpu::{tiny_skia, usvg};
use vello::kurbo::Affine;
use vello::peniko::Color;
use vello::wgpu;
use vello::{AaConfig, AaSupport, Renderer, RendererOptions, Scene};

/// 래스터화 결과 한 장 — straight(비-프리멀티플라이) RGBA8, 행 우선, `width*height*4` 바이트.
///
/// 페이지 배경을 불투명 흰색으로 채우므로 전 픽셀의 alpha 는 255 이고, 이 경우
/// premultiplied 와 straight 표현이 동일해 두 래스터라이저의 출력을 바로 비교할 수 있다.
pub struct RasterImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl RasterImage {
    /// PNG 로 인코딩한다(저장소 공용 `image` 크레이트, png feature).
    pub fn encode_png(&self) -> Result<Vec<u8>, String> {
        let buf = image::RgbaImage::from_raw(self.width, self.height, self.rgba.clone())
            .ok_or_else(|| "RGBA 버퍼 크기가 이미지 치수와 맞지 않습니다".to_string())?;
        let mut out = std::io::Cursor::new(Vec::new());
        buf.write_to(&mut out, image::ImageFormat::Png)
            .map_err(|e| format!("PNG 인코딩 실패: {e}"))?;
        Ok(out.into_inner())
    }
}

/// SVG 문자열을 하나의 `usvg::Tree` 로 파싱한다. 텍스트→글리프 변환에 시스템 폰트와
/// (선택적으로) 지정 폰트 경로를 쓴다. 이 트리를 GPU·CPU 두 경로가 공유한다.
pub fn parse_svg(svg: &str, font_paths: &[PathBuf]) -> Result<usvg::Tree, String> {
    let mut opt = usvg::Options::default();
    {
        let db = opt.fontdb_mut();
        db.load_system_fonts();
        for path in font_paths {
            if path.is_dir() {
                db.load_fonts_dir(path);
            } else if let Err(e) = db.load_font_file(path) {
                eprintln!(
                    "경고: 폰트 파일을 불러오지 못했습니다 - {}: {e}",
                    path.display()
                );
            }
        }
    }
    usvg::Tree::from_str(svg, &opt).map_err(|e| format!("SVG 파싱 실패: {e}"))
}

/// usvg 트리의 픽셀 치수를 배율에 맞춰 계산한다(각 변 최소 1px).
fn scaled_dims(tree: &usvg::Tree, scale: f64) -> (u32, u32) {
    let size = tree.size();
    let w = ((size.width() as f64) * scale).ceil().max(1.0) as u32;
    let h = ((size.height() as f64) * scale).ceil().max(1.0) as u32;
    (w, h)
}

/// CPU 기준선: 동일한 `usvg::Tree` 를 resvg(tiny-skia 백엔드)로 래스터화한다.
///
/// GPU 경로와 완전히 같은 벡터 입력을 소비하므로, 두 결과의 픽셀 차이는 레이아웃 차이가
/// 아니라 **래스터라이저 차이**(안티에일리어싱 방식 등)만 반영한다.
pub fn cpu_rasterize(tree: &usvg::Tree, scale: f64) -> Result<RasterImage, String> {
    let (width, height) = scaled_dims(tree, scale);
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| format!("tiny-skia Pixmap 생성 실패 ({width}x{height})"))?;
    // 페이지 배경을 불투명 흰색으로 — GPU 경로(base_color=WHITE)와 동일 조건.
    pixmap.fill(tiny_skia::Color::WHITE);
    resvg_gpu::render(
        tree,
        tiny_skia::Transform::from_scale(scale as f32, scale as f32),
        &mut pixmap.as_mut(),
    );
    Ok(RasterImage {
        width,
        height,
        rgba: pixmap.data().to_vec(),
    })
}

/// GPU 래스터화 컨텍스트 — wgpu 디바이스/큐 + vello 렌더러를 **한 번** 만들어 배치 전체에서
/// 재사용한다. 헤드리스(서피스 없음)로 동작하며, 기본 어댑터를 자동 선택한다.
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
    /// 선택된 어댑터 정보(백엔드·디바이스명) — 보고·진단용.
    pub adapter_info: wgpu::AdapterInfo,
}

impl GpuContext {
    /// 헤드리스 GPU 컨텍스트를 만든다. 어댑터가 없거나 vello 초기화가 실패하면 `Err`.
    ///
    /// 이 함수 호출 비용(수백 ms 수준)이 곧 "일회성 초기화 비용" 이다 — 배치가 클수록
    /// 페이지당으로 분할 상각된다.
    pub fn new() -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            // GL 백엔드는 vello 의 컴퓨트 파이프라인을 지원하지 않으므로 PRIMARY(Vulkan/DX12/Metal)만.
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .ok_or_else(|| {
            "GPU 어댑터를 찾을 수 없습니다 (Vulkan/DX12/Metal 지원 GPU·드라이버 필요)".to_string()
        })?;

        let adapter_info = adapter.get_info();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("rhwp-gpu-raster"),
                required_features: wgpu::Features::empty(),
                // 어댑터가 실제로 지원하는 한도를 그대로 요청 — vello 의 스토리지 텍스처/버퍼
                // 요구를 다운레벨 기본값이 못 맞추는 경우를 피한다.
                required_limits: adapter.limits(),
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ))
        .map_err(|e| format!("GPU 디바이스 요청 실패: {e}"))?;

        let renderer = Renderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                // Area AA만 초기화 — 셰이더 순열 컴파일을 줄여 초기화를 빠르게.
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )
        .map_err(|e| format!("vello 렌더러 생성 실패: {e:?}"))?;

        Ok(Self {
            device,
            queue,
            renderer,
            adapter_info,
        })
    }

    /// 사람이 읽는 어댑터 요약("DX12 / NVIDIA GeForce ... (DiscreteGpu)").
    pub fn adapter_summary(&self) -> String {
        format!(
            "{:?} / {} ({:?})",
            self.adapter_info.backend, self.adapter_info.name, self.adapter_info.device_type
        )
    }

    /// 공유 `usvg::Tree` 를 GPU에서 래스터화해 straight RGBA8 로 리드백한다.
    pub fn rasterize(&mut self, tree: &usvg::Tree, scale: f64) -> Result<RasterImage, String> {
        let (width, height) = scaled_dims(tree, scale);

        // vello_svg 로 트리를 Scene 으로 변환하고, 배율은 Affine 으로 전체에 적용한다.
        let svg_scene = vello_svg::render_tree(tree);
        let mut scene = Scene::new();
        scene.append(&svg_scene, Some(Affine::scale(scale)));

        // vello 는 STORAGE_BINDING 텍스처(Rgba8Unorm)에 컴퓨트로 쓴다. 리드백을 위해
        // COPY_SRC 도 함께 요구한다.
        let target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rhwp-gpu-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());

        self.renderer
            .render_to_texture(
                &self.device,
                &self.queue,
                &scene,
                &view,
                &vello::RenderParams {
                    // 불투명 흰 배경 — CPU 기준선과 동일 조건, alpha 전부 255.
                    base_color: Color::WHITE,
                    width,
                    height,
                    antialiasing_method: AaConfig::Area,
                },
            )
            .map_err(|e| format!("vello 렌더 실패: {e:?}"))?;

        self.read_back(&target, width, height)
    }

    /// 텍스처 → 매핑 버퍼로 복사하고, 256바이트 정렬 패딩을 제거해 조밀 RGBA 로 되돌린다.
    fn read_back(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<RasterImage, String> {
        let unpadded_bpr = width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        // wgpu 는 copy_texture_to_buffer 의 bytes_per_row 가 256의 배수이길 요구한다.
        let padded_bpr = unpadded_bpr.div_ceil(align) * align;

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rhwp-gpu-readback"),
            size: (padded_bpr as u64) * (height as u64),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bpr),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        // 헤드리스라 이벤트 루프가 없으므로 블로킹 폴로 매핑 완료를 기다린다.
        self.device.poll(wgpu::Maintain::Wait);

        let mapped = slice.get_mapped_range();
        let mut rgba = vec![0u8; (unpadded_bpr as usize) * (height as usize)];
        for y in 0..height as usize {
            let src = y * padded_bpr as usize;
            let dst = y * unpadded_bpr as usize;
            rgba[dst..dst + unpadded_bpr as usize]
                .copy_from_slice(&mapped[src..src + unpadded_bpr as usize]);
        }
        drop(mapped);
        buffer.unmap();

        Ok(RasterImage {
            width,
            height,
            rgba,
        })
    }
}

/// 두 래스터 결과의 저비용 픽셀 비교 통계.
#[derive(Debug, Clone, Copy)]
pub struct DiffStats {
    pub dims_match: bool,
    pub width_a: u32,
    pub height_a: u32,
    pub width_b: u32,
    pub height_b: u32,
    /// 채널당 평균 절대 차이(0..255).
    pub mean_abs: f64,
    /// 채널당 최대 절대 차이(0..255).
    pub max_abs: u8,
    /// 어떤 채널이든 16 이상 차이 나는 픽셀의 비율(0..1).
    pub pct_pixels_over_thresh: f64,
}

/// 같은 치수의 두 RGBA 이미지를 비교한다. 치수가 다르면 `dims_match=false` 만 채워 돌려준다.
pub fn diff(a: &RasterImage, b: &RasterImage) -> DiffStats {
    let dims_match = a.width == b.width && a.height == b.height;
    if !dims_match {
        return DiffStats {
            dims_match: false,
            width_a: a.width,
            height_a: a.height,
            width_b: b.width,
            height_b: b.height,
            mean_abs: f64::NAN,
            max_abs: 255,
            pct_pixels_over_thresh: f64::NAN,
        };
    }

    let n = a.rgba.len().min(b.rgba.len());
    let mut sum_abs: u64 = 0;
    let mut max_abs: u8 = 0;
    let mut pixels_over: u64 = 0;
    let px_count = (a.width as u64) * (a.height as u64);

    for px in 0..(n / 4) {
        let mut over = false;
        for c in 0..4 {
            let i = px * 4 + c;
            let d = a.rgba[i].abs_diff(b.rgba[i]);
            sum_abs += d as u64;
            if d > max_abs {
                max_abs = d;
            }
            if d >= 16 {
                over = true;
            }
        }
        if over {
            pixels_over += 1;
        }
    }

    DiffStats {
        dims_match: true,
        width_a: a.width,
        height_a: a.height,
        width_b: b.width,
        height_b: b.height,
        mean_abs: sum_abs as f64 / n as f64,
        max_abs,
        pct_pixels_over_thresh: if px_count == 0 {
            0.0
        } else {
            pixels_over as f64 / px_count as f64
        },
    }
}

/// 사용 가능한 GPU 어댑터를 열거한다(`gpu-info` 하위명령용). 각 원소는
/// "(backend) name — type" 요약 문자열이다.
pub fn probe_adapters() -> Vec<String> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    instance
        .enumerate_adapters(wgpu::Backends::all())
        .into_iter()
        .map(|adapter| {
            let info = adapter.get_info();
            format!(
                "({:?}) {} — {:?} [driver: {}]",
                info.backend, info.name, info.device_type, info.driver
            )
        })
        .collect()
}
