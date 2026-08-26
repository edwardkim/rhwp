//! 백엔드 구현이 공유하는 부품 — 생명주기 상태기, op 이름표, 레이어 트리 구동기.
//!
//! 여기 있는 것들은 "모든 백엔드가 똑같이 해야 하는데 각자 구현하면 어긋나는"
//! 일들이다. 한 곳에 두어야 백엔드 간 정합 시험이 의미를 갖는다.

use crate::paint::{
    paint_op_replay_plane_with_layer, LayerNode, LayerNodeKind, PageLayerTree, PaintOp,
    PaintReplayPlane,
};
use crate::renderer::render_tree::RenderLayerInfo;

use super::traits::{PageSize, RenderBackend, RenderBackendError};

/// `PaintOp` 종류의 안정 이름.
///
/// 문자열은 `src/paint/json.rs` 의 LayerTree JSON export 가 쓰는 `"type"` 값과
/// **글자 그대로 같다**. 그래야 백엔드 추적 로그와 기존 JSON 덤프를 같은
/// 어휘로 맞대볼 수 있다.
pub fn paint_op_kind(op: &PaintOp) -> &'static str {
    match op {
        PaintOp::PageBackground { .. } => "pageBackground",
        PaintOp::TextRun { .. } => "textRun",
        PaintOp::GlyphRun { .. } => "glyphRun",
        PaintOp::GlyphOutline { .. } => "glyphOutline",
        PaintOp::CharOverlap { .. } => "charOverlap",
        PaintOp::TextControlMark { .. } => "textControlMark",
        PaintOp::TabLeader { .. } => "tabLeader",
        PaintOp::TextDecoration { .. } => "textDecoration",
        PaintOp::FootnoteMarker { .. } => "footnoteMarker",
        PaintOp::Line { .. } => "line",
        PaintOp::Rectangle { .. } => "rectangle",
        PaintOp::Ellipse { .. } => "ellipse",
        PaintOp::Path { .. } => "path",
        PaintOp::Image { .. } => "image",
        PaintOp::Equation { .. } => "equation",
        PaintOp::FormObject { .. } => "formObject",
        PaintOp::Placeholder { .. } => "placeholder",
        PaintOp::RawSvg { .. } => "rawSvg",
    }
}

/// `RenderBackend` 생명주기 불변식을 한 곳에서 판정하는 상태기.
///
/// 백엔드마다 "페이지를 안 열고 그리면 어떻게 되나"가 달라지면 계약이 아니다.
/// 이 크레이트의 백엔드는 전부 이 상태기를 품고, 외부 백엔드도 이걸 그대로
/// 쓰면 같은 오류를 같은 자리에서 낸다.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PageState {
    open: Option<PageSize>,
    pages_completed: usize,
    ops_on_page: usize,
    ops_total: usize,
}

impl PageState {
    /// 아무 페이지도 열지 않은 초기 상태.
    pub fn new() -> Self {
        Self::default()
    }

    /// `begin_page` 판정 — 치수 유효성과 중복 열기를 막는다.
    pub fn begin(&mut self, size: PageSize) -> Result<(), RenderBackendError> {
        if self.open.is_some() {
            return Err(RenderBackendError::PageAlreadyOpen);
        }
        if !size.is_valid() {
            return Err(RenderBackendError::InvalidPageSize {
                width: size.width,
                height: size.height,
            });
        }
        self.open = Some(size);
        self.ops_on_page = 0;
        Ok(())
    }

    /// `draw` 판정 — 열린 페이지가 없으면 오류다. 성공하면 op 수를 센다.
    pub fn record_draw(&mut self) -> Result<PageSize, RenderBackendError> {
        match self.open {
            Some(size) => {
                self.ops_on_page += 1;
                self.ops_total += 1;
                Ok(size)
            }
            None => Err(RenderBackendError::NoOpenPage { call: "draw" }),
        }
    }

    /// `end_page` 판정 — 닫은 페이지의 치수와 그 페이지에 그린 op 수를 돌려준다.
    pub fn end(&mut self) -> Result<(PageSize, usize), RenderBackendError> {
        match self.open.take() {
            Some(size) => {
                self.pages_completed += 1;
                Ok((size, self.ops_on_page))
            }
            None => Err(RenderBackendError::NoOpenPage { call: "end_page" }),
        }
    }

    /// `finish` 판정 — 열어둔 페이지가 남아 있으면 오류다.
    pub fn assert_finished(&self) -> Result<(), RenderBackendError> {
        if self.open.is_some() {
            return Err(RenderBackendError::UnclosedPage {
                pages_completed: self.pages_completed,
            });
        }
        Ok(())
    }

    /// 지금 열려 있는 페이지 치수.
    pub fn current_page(&self) -> Option<PageSize> {
        self.open
    }

    /// 정상으로 닫힌 페이지 수.
    pub fn pages_completed(&self) -> usize {
        self.pages_completed
    }

    /// 지금 열린 페이지에 그린 op 수.
    pub fn ops_on_page(&self) -> usize {
        self.ops_on_page
    }

    /// 지금까지 그린 op 총수.
    pub fn ops_total(&self) -> usize {
        self.ops_total
    }
}

/// 기존 `PageLayerTree` 한 장을 임의의 백엔드로 재생한다.
///
/// 이 함수가 `RenderBackend` 와 기존 IR 을 잇는 다리다. 새 백엔드는 트리 순회를
/// 다시 짤 필요 없이 [`RenderBackend`] 만 구현하면 된다.
///
/// # 재생 순서
///
/// `src/paint/replay_order.rs` 의 [`PaintReplayPlane::ORDERED`]
/// (배경 → 글 뒤 → 본문 흐름 → 글 앞)를 바깥 루프로 돌고, 각 plane 안에서는
/// 트리 전위 순회 순서를 지킨다. 이는 Skia(`src/renderer/skia/renderer.rs:496`)와
/// 웹 캔버스(`src/renderer/web_canvas.rs:492`)가 이미 쓰는 정본 순서와 같다.
/// 조상 `LayerNode::layer` 는 자손에게 상속되며, plane 판정은
/// `paint_op_replay_plane_with_layer` 에 위임한다.
pub fn replay_page<B>(backend: &mut B, tree: &PageLayerTree) -> Result<(), B::Error>
where
    B: RenderBackend + ?Sized,
{
    backend.begin_page(PageSize::from_layer_tree(tree))?;
    for plane in PaintReplayPlane::ORDERED {
        replay_node(backend, &tree.root, None, plane)?;
    }
    backend.end_page()
}

fn replay_node<B>(
    backend: &mut B,
    node: &LayerNode,
    inherited_layer: Option<RenderLayerInfo>,
    plane: PaintReplayPlane,
) -> Result<(), B::Error>
where
    B: RenderBackend + ?Sized,
{
    let active_layer = node.layer.or(inherited_layer);
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                replay_node(backend, child, active_layer, plane)?;
            }
        }
        LayerNodeKind::ClipRect { child, .. } => {
            replay_node(backend, child, active_layer, plane)?;
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if paint_op_replay_plane_with_layer(op, active_layer) == plane {
                    backend.draw(op)?;
                }
            }
        }
    }
    Ok(())
}
