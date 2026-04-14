use crate::paint::PageLayerTree;

/// visual layer tree를 backend 출력으로 재생한다.
pub trait LayerRenderer {
    fn render_page(&mut self, tree: &PageLayerTree);
}
