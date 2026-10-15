//! 브라우저 없이 문서와 렌더러의 수명을 유지하는 opt-in host 계층.
//!
//! 문서 엔진의 편집 정책은 바꾸지 않는다. 세션은 생성한 스레드에서 사용하며
//! 페이지 좌표는 기존 layer tree의 96-DPI 논리 좌표를 그대로 사용한다.

pub mod editor;
pub mod fonts;

use std::marker::PhantomData;
use std::rc::Rc;

use serde::Serialize;

use crate::{DocumentCore, HwpError};

pub const MAX_DOCUMENT_BYTES: usize = 128 * 1024 * 1024;
pub const MAX_FRAME_PIXELS: u64 = 16 * 1024 * 1024;
pub const MAX_RENDER_CACHE_ENTRIES: usize = 64;
pub const MAX_RENDER_CACHE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug)]
pub enum NativeHostError {
    InvalidArgument(String),
    LimitExceeded(String),
    Unsupported(String),
    FeatureDisabled,
    StaleRevision { expected: u64, actual: u64 },
    Core(HwpError),
}

impl From<crate::document_core::native_interaction::NativeInteractionError> for NativeHostError {
    fn from(error: crate::document_core::native_interaction::NativeInteractionError) -> Self {
        use crate::document_core::native_interaction::NativeInteractionError;
        match error {
            NativeInteractionError::InvalidInput(message) => Self::InvalidArgument(message.into()),
            NativeInteractionError::InvalidPosition(message) => Self::InvalidArgument(message),
            NativeInteractionError::Unsupported(message) => Self::Unsupported(message.into()),
            NativeInteractionError::InvalidCoreResult(message) => {
                Self::Core(HwpError::RenderError(message))
            }
            NativeInteractionError::Core(error) => Self::Core(error),
        }
    }
}

impl From<HwpError> for NativeHostError {
    fn from(error: HwpError) -> Self {
        match error {
            HwpError::ResourceLimit(message) => Self::LimitExceeded(message),
            other => Self::Core(other),
        }
    }
}

impl std::fmt::Display for NativeHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArgument(message)
            | Self::LimitExceeded(message)
            | Self::Unsupported(message) => f.write_str(message),
            Self::FeatureDisabled => f.write_str("native-skia feature is required"),
            Self::StaleRevision { expected, actual } => {
                write!(f, "expected revision {expected}, current revision {actual}")
            }
            Self::Core(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for NativeHostError {}

impl NativeHostError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidArgument(_) => "invalidArgument",
            Self::LimitExceeded(_) => "limitExceeded",
            Self::Unsupported(_) => "unsupported",
            Self::FeatureDisabled => "featureDisabled",
            Self::StaleRevision { .. } => "staleRevision",
            Self::Core(_) => "documentError",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePageSize {
    pub width: f64,
    pub height: f64,
}

/// RGBA8 premultiplied, sRGB, 위에서 아래로 저장한 프레임.
/// `pixels.stride`를 사용하며 픽셀은 세션보다 오래 소유할 수 있다.
pub struct NativeFrame {
    pub pixels: crate::renderer::raster_pixels::RasterPixels,
    pub revision: u64,
    pub font_generation: u64,
    /// 확대된 전체 페이지 픽셀 격자에서 이 버퍼의 왼쪽 위 원점.
    pub origin_x: i32,
    pub origin_y: i32,
}

pub struct NativeDocumentSession {
    pub(super) core: DocumentCore,
    revision: u64,
    font_generation: u64,
    fonts: fonts::NativeFontState,
    #[cfg(feature = "native-skia")]
    renderer: std::cell::OnceCell<crate::renderer::skia::SkiaLayerRenderer>,
    // feature가 꺼진 빌드도 같은 스레드 소유권 계약을 지킨다.
    _thread_affinity: PhantomData<Rc<()>>,
}

impl NativeDocumentSession {
    pub fn from_core(core: DocumentCore) -> Self {
        let fonts = fonts::NativeFontState::from_environment(core.font_environment().cloned());
        Self {
            core,
            revision: 0,
            font_generation: 0,
            fonts,
            #[cfg(feature = "native-skia")]
            renderer: std::cell::OnceCell::new(),
            _thread_affinity: PhantomData,
        }
    }

    pub fn new_blank() -> Result<Self, NativeHostError> {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native()?;
        Ok(Self::from_core(core))
    }

    pub fn open_bytes(bytes: &[u8], password: Option<&[u8]>) -> Result<Self, NativeHostError> {
        if bytes.is_empty() {
            return Err(NativeHostError::InvalidArgument("empty document".into()));
        }
        if bytes.len() > MAX_DOCUMENT_BYTES || password.is_some_and(|p| p.len() > 4096) {
            return Err(NativeHostError::LimitExceeded(
                "document or password exceeds limit".into(),
            ));
        }
        let core = match password {
            Some(password) => DocumentCore::from_bytes_with_password(bytes, password)?,
            None => DocumentCore::from_bytes(bytes)?,
        };
        Ok(Self::from_core(core))
    }

    pub fn document(&self) -> &DocumentCore {
        &self.core
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn font_generation(&self) -> u64 {
        self.font_generation
    }

    pub fn is_frame_current(&self, revision: u64, font_generation: u64) -> bool {
        self.revision == revision && self.font_generation == font_generation
    }

    pub fn clear_caches(&self) {
        #[cfg(feature = "native-skia")]
        if let Some(renderer) = self.renderer.get() {
            renderer.clear_caches();
        }
    }

    #[cfg(feature = "native-skia")]
    pub fn cache_stats(&self) -> crate::renderer::skia::NativeSkiaCacheStats {
        self.renderer
            .get()
            .map(|renderer| renderer.cache_stats())
            .unwrap_or_default()
    }

    pub(super) fn did_mutate(&mut self) -> Result<u64, NativeHostError> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| NativeHostError::LimitExceeded("document revision exhausted".into()))?;
        self.clear_caches();
        Ok(self.revision)
    }

    pub fn check_revision(&self, expected: u64) -> Result<(), NativeHostError> {
        if expected != self.revision {
            return Err(NativeHostError::StaleRevision {
                expected,
                actual: self.revision,
            });
        }
        Ok(())
    }

    pub fn page_size(&self, page: u32) -> Result<NativePageSize, NativeHostError> {
        self.check_page(page)?;
        let tree = self
            .core
            .build_page_layer_tree_with_profile(page, crate::paint::RenderProfile::Screen)?;
        Ok(NativePageSize {
            width: tree.page_width,
            height: tree.page_height,
        })
    }

    fn check_page(&self, page: u32) -> Result<(), NativeHostError> {
        if page >= self.core.page_count() {
            return Err(NativeHostError::InvalidArgument(
                "page index out of range".into(),
            ));
        }
        Ok(())
    }

    pub fn render_page(&self, page: u32, scale: f64) -> Result<NativeFrame, NativeHostError> {
        self.check_page(page)?;
        Self::check_scale(scale)?;
        #[cfg(feature = "native-skia")]
        {
            use crate::renderer::layer_renderer::RasterRenderOptions;
            let tree = self
                .core
                .build_page_layer_tree_with_profile(page, crate::paint::RenderProfile::Screen)?;
            let pixels = self.renderer().render_pixels_with_options(
                &tree,
                RasterRenderOptions {
                    scale,
                    max_pixels: MAX_FRAME_PIXELS,
                    transparent: false,
                    ..Default::default()
                },
            )?;
            Ok(NativeFrame {
                pixels,
                revision: self.revision,
                font_generation: self.font_generation,
                origin_x: 0,
                origin_y: 0,
            })
        }
        #[cfg(not(feature = "native-skia"))]
        Err(NativeHostError::FeatureDisabled)
    }

    pub fn render_region(
        &self,
        page: u32,
        region: crate::renderer::raster_pixels::RasterRegion,
        scale: f64,
    ) -> Result<NativeFrame, NativeHostError> {
        self.check_page(page)?;
        Self::check_scale(scale)?;
        #[cfg(feature = "native-skia")]
        {
            let tree = self
                .core
                .build_page_layer_tree_with_profile(page, crate::paint::RenderProfile::Screen)?;
            let output = self.renderer().render_region_pixels_with_options(
                &tree,
                region,
                crate::renderer::layer_renderer::RasterRenderOptions {
                    scale,
                    max_pixels: MAX_FRAME_PIXELS,
                    transparent: false,
                    ..Default::default()
                },
            )?;
            Ok(NativeFrame {
                pixels: output.pixels,
                revision: self.revision,
                font_generation: self.font_generation,
                origin_x: output.origin_x,
                origin_y: output.origin_y,
            })
        }
        #[cfg(not(feature = "native-skia"))]
        {
            let _ = region;
            Err(NativeHostError::FeatureDisabled)
        }
    }

    #[cfg(feature = "native-skia")]
    fn renderer(&self) -> &crate::renderer::skia::SkiaLayerRenderer {
        self.renderer.get_or_init(|| {
            crate::renderer::skia::SkiaLayerRenderer::new()
                .with_cache_limits(MAX_RENDER_CACHE_ENTRIES, MAX_RENDER_CACHE_BYTES)
        })
    }

    fn check_scale(scale: f64) -> Result<(), NativeHostError> {
        if !scale.is_finite() || scale <= 0.0 || scale > 64.0 {
            return Err(NativeHostError::InvalidArgument(
                "scale must be finite and in (0, 64]".into(),
            ));
        }
        Ok(())
    }
}
