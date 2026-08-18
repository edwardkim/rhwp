//! native-skia 어댑터 — 기존 레이어 래스터 경로를 `RenderBackend` 뒤에 세운다.
//!
//! 이 파일은 `src/renderer/**` 를 **한 줄도 고치지 않는다**. `native-skia` 가 켜져 있고
//! wasm 이 아니면 기존 공개 API (`SkiaLayerRenderer::new` /
//! `LayerRasterRenderer::render_raster`) 를 호출만 한다. `render_png` /
//! `render_png_with_options` 는 쓰지 않는다 — PNG 바이트열만 내는 어댑터는
//! M06-1 `PngBackend` 의 몫이다. 이 어댑터의 산출물은 레이어 래스터 문서
//! (`RasterRenderOutput`: 바이트 + 폭·높이·dpi·색공간·형식)다.
//!
//! 그 피처가 없으면 어댑터는 그대로 컴파일되고 생명주기는 지키지만 래스터는
//! **건너뛴다** (`finish` 산출물은 빈 문서 — 바이트 없음, 0×0). 능력 선언은 그
//! 사실을 숨기지 않는다.
//!
//! # 얇음의 대가 (알려진 한계)
//!
//! 어댑터는 받은 op 들을 `LayerNode::leaf` 하나로 묶어 기존 렌더러에 넘긴다.
//! 따라서 원본 트리의 그룹·클립 구조는 이 경로에서 평탄해진다. SVG 어댑터와
//! 같은 대가이며, `capabilities().clipping` 을 켜지 않는다.

use crate::paint::{PaintOp, RenderProfile};
use crate::renderer::layer_renderer::{
    RasterColorSpace, RasterOutputFormat, RasterRenderOptions, RasterRenderOutput,
};

use super::caps::BackendCapabilities;
use super::traits::{PageSize, RenderBackend, RenderBackendError};
use super::util::PageState;

/// 기존 native-skia 레이어 래스터 경로를 `RenderBackend` 계약으로 감싼 어댑터.
///
/// 산출물은 [`RasterRenderOutput`] 한 장이다. 래스터 문서 여러 장을 한 값으로
/// 이어 붙일 형식이 없으므로, 이 어댑터는 두 번째 페이지를 거절한다. 여러
/// 페이지를 내려면 페이지마다 새 `SkiaBackend` 를 만든다.
#[derive(Debug)]
pub struct SkiaBackend {
    state: PageState,
    profile: RenderProfile,
    options: RasterRenderOptions,
    pending: Vec<PaintOp>,
    pages: Vec<RasterRenderOutput>,
}

impl SkiaBackend {
    /// 화면 프로파일과 기본 래스터 옵션으로 어댑터를 만든다.
    pub fn new() -> Self {
        Self::with_options(RenderProfile::Screen, RasterRenderOptions::default())
    }

    /// 렌더 프로파일을 지정해 어댑터를 만든다.
    pub fn with_profile(profile: RenderProfile) -> Self {
        Self::with_options(profile, RasterRenderOptions::default())
    }

    /// 래스터 옵션만 지정해 어댑터를 만든다.
    ///
    /// 옵션은 페이지마다 바뀌지 않고 생성자에 고정된다. 기존
    /// `LayerRasterRenderer::render_raster` 가 받는 값과 같다.
    pub fn with_raster_options(options: RasterRenderOptions) -> Self {
        Self::with_options(RenderProfile::Screen, options)
    }

    /// 렌더 프로파일과 래스터 옵션을 지정해 어댑터를 만든다.
    ///
    /// `native-skia` 가 꺼져 있으면 이 옵션은 쓰이지 않는다.
    pub fn with_options(profile: RenderProfile, options: RasterRenderOptions) -> Self {
        Self {
            state: PageState::new(),
            profile,
            options,
            pending: Vec::new(),
            pages: Vec::new(),
        }
    }

    /// 지금까지 완성된 페이지별 래스터 문서.
    ///
    /// 래스터를 건너뛴 페이지는 빈 문서(0×0, 바이트 없음)다.
    pub fn pages(&self) -> &[RasterRenderOutput] {
        &self.pages
    }

    /// 이 빌드가 실제 native-skia 래스터를 돌릴 수 있는가.
    ///
    /// `native-skia` 피처가 켜진 네이티브 빌드에서만 `true` 다. wasm 과
    /// 기본 CI 빌드에서는 `false` 이며, 그때 능력 선언도 그에 맞춘다.
    pub const fn raster_available() -> bool {
        cfg!(all(not(target_arch = "wasm32"), feature = "native-skia"))
    }
}

impl Default for SkiaBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackend for SkiaBackend {
    type Output = RasterRenderOutput;
    type Error = RenderBackendError;

    fn capabilities(&self) -> BackendCapabilities {
        // 래스터 전용 산출물이므로 vector_text 는 끈다 (`raster("skia")` 기본값).
        // 얇은 어댑터는 PaintOp 만 받아 새 leaf 로 평탄화하므로 클립을 보존하지 못한다.
        // RasterRenderOutput 한 장은 한 페이지 — 여러 장을 한 산출물로 묶을 형식이 없다.
        // 실제 래스터가 없으면 이미지·그라디언트도 산출물에 남지 않는다.
        let live = Self::raster_available();
        BackendCapabilities {
            gradients: live,
            clipping: false,
            images: live,
            multi_page: false,
            ..BackendCapabilities::raster("skia")
        }
    }

    fn begin_page(&mut self, size: PageSize) -> Result<(), Self::Error> {
        if !self.pages.is_empty() {
            return Err(RenderBackendError::MultiplePagesUnsupported { backend: "skia" });
        }
        self.state.begin(size)?;
        self.pending.clear();
        Ok(())
    }

    fn draw(&mut self, op: &PaintOp) -> Result<(), Self::Error> {
        self.state.record_draw()?;
        self.pending.push(op.clone());
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), Self::Error> {
        let (size, _) = self.state.end()?;
        let document = raster_document(
            size,
            self.profile,
            self.options,
            std::mem::take(&mut self.pending),
        )?;
        self.pages.push(document);
        Ok(())
    }

    fn finish(self) -> Result<Self::Output, Self::Error> {
        self.state.assert_finished()?;
        Ok(self.pages.into_iter().next().unwrap_or_else(empty_document))
    }

    fn finish_boxed(self: Box<Self>) -> Result<Self::Output, Self::Error> {
        (*self).finish()
    }
}

fn empty_document() -> RasterRenderOutput {
    RasterRenderOutput {
        bytes: Vec::new(),
        format: RasterOutputFormat::Png,
        width: 0,
        height: 0,
        dpi: None,
        color_space: RasterColorSpace::Srgb,
    }
}

fn raster_document(
    size: PageSize,
    profile: RenderProfile,
    options: RasterRenderOptions,
    pending: Vec<PaintOp>,
) -> Result<RasterRenderOutput, RenderBackendError> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
    {
        use crate::paint::{LayerNode, PageLayerTree};
        use crate::renderer::layer_renderer::LayerRasterRenderer;
        use crate::renderer::render_tree::BoundingBox;
        use crate::renderer::skia::SkiaLayerRenderer;

        let bounds = BoundingBox::new(0.0, 0.0, size.width, size.height);
        let root = LayerNode::leaf(bounds, None, pending);
        let tree = PageLayerTree::with_profile(size.width, size.height, root, profile);
        SkiaLayerRenderer::new()
            .render_raster(&tree, options)
            .map_err(RenderBackendError::from)
    }
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native-skia")))]
    {
        let _ = (size, profile, options, pending);
        Ok(empty_document())
    }
}
