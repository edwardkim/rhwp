use crate::paint::PageLayerTree;

/// visual layer tree를 stateful backend 출력으로 재생하는 전환기 trait.
///
/// 현재는 내부 출력 버퍼나 장면 상태를 누적하는 backend, 예를 들어 layered SVG bridge가
/// 이 trait를 직접 구현한다. native Skia처럼 최종 결과를 바이트로 반환하는 raster
/// backend는 아직 별도의 명시적 API를 유지한다.
pub trait LayerRenderer {
    fn render_page(&mut self, tree: &PageLayerTree);
}
