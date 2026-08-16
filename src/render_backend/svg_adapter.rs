//! 레퍼런스 어댑터 — 기존 SVG 렌더러를 `RenderBackend` 뒤에 세운다.
//!
//! 이 파일은 `src/renderer/**` 를 **한 줄도 고치지 않는다**. 기존 공개 API
//! (`SvgLayerRenderer::new` / `LayerRenderer::render_page` / `SvgLayerRenderer::output`)
//! 를 호출만 한다. 계약이 실제 백엔드를 감쌀 수 있는지 증명하는 것이 목적이다.
//!
//! # 얇음의 대가 (알려진 한계)
//!
//! 어댑터는 받은 op 들을 `LayerNode::leaf` 하나로 묶어 기존 렌더러에 넘긴다.
//! 따라서 원본 트리의 그룹·클립 구조는 이 경로에서 평탄해진다. 진짜 이관은
//! `SvgRenderer` 안에서 이 trait 을 직접 구현하는 것이며(그쪽엔 이미
//! `begin_page`/`end_page` 내부 함수가 있다), 그 작업은 이 PR 의 범위가 아니다.
//! 자세한 채택 시나리오는 `mydocs/tech/render_backend.md` 를 본다.

use crate::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
use crate::renderer::layer_renderer::LayerRenderer;
use crate::renderer::render_tree::BoundingBox;
use crate::renderer::svg_layer::SvgLayerRenderer;

use super::caps::BackendCapabilities;
use super::traits::{PageSize, RenderBackend, RenderBackendError};
use super::util::PageState;

/// 기존 SVG 렌더러를 `RenderBackend` 계약으로 감싼 어댑터.
///
/// 산출물은 페이지마다 독립된 `<svg>` 문서이며, `finish` 는 그것들을 줄바꿈으로
/// 이어 붙인 하나의 문자열을 돌려준다. 페이지별로 따로 쓰려면 [`SvgBackend::pages`]
/// 를 본다.
#[derive(Debug)]
pub struct SvgBackend {
    state: PageState,
    profile: RenderProfile,
    pending: Vec<PaintOp>,
    pages: Vec<String>,
}

impl SvgBackend {
    /// 화면 프로파일(`RenderProfile::Screen`)로 어댑터를 만든다.
    pub fn new() -> Self {
        Self::with_profile(RenderProfile::Screen)
    }

    /// 렌더 프로파일을 지정해 어댑터를 만든다.
    ///
    /// 프로파일은 편집기 전용 시각 요소(문단 부호 등)를 낼지를 가른다 —
    /// 기존 `PageLayerTree::with_profile` 이 쓰는 값과 같은 것이다.
    pub fn with_profile(profile: RenderProfile) -> Self {
        Self {
            state: PageState::new(),
            profile,
            pending: Vec::new(),
            pages: Vec::new(),
        }
    }

    /// 지금까지 완성된 페이지별 SVG 문서들.
    pub fn pages(&self) -> &[String] {
        &self.pages
    }
}

impl Default for SvgBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderBackend for SvgBackend {
    type Output = String;
    type Error = RenderBackendError;

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            // 어댑터 자신은 폰트 바이트를 싣지 않는다. SVG 에 폰트를 내장하는
            // 일은 상위(`document_core`)가 `generate_embedded_font_style` 로
            // 따로 한다.
            embedded_fonts: false,
            ..BackendCapabilities::vector("svg")
        }
    }

    fn begin_page(&mut self, size: PageSize) -> Result<(), Self::Error> {
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
        let bounds = BoundingBox::new(0.0, 0.0, size.width, size.height);
        let root = LayerNode::leaf(bounds, None, std::mem::take(&mut self.pending));
        let tree = PageLayerTree::with_profile(size.width, size.height, root, self.profile);

        let mut renderer = SvgLayerRenderer::new();
        renderer
            .render_page(&tree)
            .map_err(RenderBackendError::from)?;
        self.pages.push(renderer.output().to_string());
        Ok(())
    }

    fn finish(self) -> Result<Self::Output, Self::Error> {
        self.state.assert_finished()?;
        Ok(self.pages.join("\n"))
    }

    fn finish_boxed(self: Box<Self>) -> Result<Self::Output, Self::Error> {
        (*self).finish()
    }
}
