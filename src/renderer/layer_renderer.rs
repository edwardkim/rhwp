use crate::error::HwpError;
use crate::model::ColorRef;
use crate::paint::PageLayerTree;

pub type LayerRenderResult<T> = Result<T, HwpError>;

/// visual layer tree를 backend 출력으로 재생한다.
pub trait LayerRenderer {
    fn render_page(&mut self, tree: &PageLayerTree) -> LayerRenderResult<()>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RasterRenderOptions {
    pub max_dimension: i32,
    pub scale: f64,
    pub dpi: Option<f64>,
    pub transparent: bool,
    pub background_color: Option<ColorRef>,
    pub color_space: RasterColorSpace,
    pub format: RasterOutputFormat,
}

impl Default for RasterRenderOptions {
    fn default() -> Self {
        Self {
            max_dimension: 16_384,
            scale: 1.0,
            dpi: None,
            transparent: true,
            background_color: None,
            color_space: RasterColorSpace::Srgb,
            format: RasterOutputFormat::Png,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterColorSpace {
    Srgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterOutputFormat {
    Png,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RasterRenderOutput {
    pub bytes: Vec<u8>,
    pub format: RasterOutputFormat,
    pub width: i32,
    pub height: i32,
    pub dpi: Option<f64>,
    pub color_space: RasterColorSpace,
}

/// visual layer tree를 raster 결과로 직접 내보내는 backend 계약.
pub trait LayerRasterRenderer {
    fn render_png(&self, tree: &PageLayerTree) -> LayerRenderResult<Vec<u8>> {
        self.render_png_with_options(tree, RasterRenderOptions::default())
    }

    fn render_png_with_options(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<Vec<u8>> {
        let mut png_options = options;
        png_options.format = RasterOutputFormat::Png;
        self.render_raster(tree, png_options)
            .map(|output| output.bytes)
    }

    fn render_raster(
        &self,
        tree: &PageLayerTree,
        options: RasterRenderOptions,
    ) -> LayerRenderResult<RasterRenderOutput>;
}
