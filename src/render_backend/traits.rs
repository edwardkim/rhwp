//! `RenderBackend` 계약 본체 — trait, 페이지 치수 타입, 공통 오류 타입.
//!
//! 이 파일이 정의하는 것은 **출력 백엔드가 지켜야 할 최소 계약**이다.
//! 기존 SVG·Skia·PDF 백엔드는 각자 다른 입력(`PageRenderTree` / `PageLayerTree`)과
//! 다른 출력(`String` / `Vec<u8>` / 부수효과)을 가지므로, 이 trait 은 그 차이를
//! 연관 타입 `Output` 으로 흡수하고 **호출 순서(생명주기)와 좌표 계약만** 고정한다.

use crate::error::HwpError;
use crate::paint::{PageLayerTree, PaintOp};

use super::caps::BackendCapabilities;

/// 한 페이지의 출력 표면 치수.
///
/// # 단위 계약
///
/// - 단위는 **px** 이다. HWPUNIT(1/7200 inch)이 아니다.
/// - 이는 `PageLayerTree` 가 선언한 단위(`crate::paint::PAGE_LAYER_TREE_UNIT == "px"`)와
///   `BoundingBox` 필드 주석(`src/renderer/render_tree.rs`)이 이미 쓰는 단위와 같다.
/// - 백엔드는 이 값을 자기 형식의 단위로 바꿀 책임이 있다(예: PDF 는 pt, Skia 는 device px).
///   **변환은 백엔드 안에서 일어나고, 이 trait 을 통과하는 값은 언제나 px 이다.**
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageSize {
    /// 페이지 폭 (px).
    pub width: f64,
    /// 페이지 높이 (px).
    pub height: f64,
}

impl PageSize {
    /// px 단위 폭·높이로 페이지 치수를 만든다.
    ///
    /// 유효성(양수·유한)은 여기서 막지 않는다. `begin_page` 가
    /// [`PageSize::is_valid`] 로 판정해 [`RenderBackendError::InvalidPageSize`] 를 낸다.
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// 페이지 치수가 그릴 수 있는 값인가 — 유한하고 0 보다 큰가.
    pub fn is_valid(&self) -> bool {
        self.width.is_finite() && self.height.is_finite() && self.width > 0.0 && self.height > 0.0
    }

    /// 기존 `PageLayerTree` 에서 페이지 치수를 그대로 가져온다.
    ///
    /// `PageLayerTree::page_width` / `page_height` 는 이미 px 이므로 환산이 없다.
    pub fn from_layer_tree(tree: &PageLayerTree) -> Self {
        Self::new(tree.page_width, tree.page_height)
    }
}

/// 백엔드 공통 오류.
///
/// 각 백엔드는 `RenderBackend::Error` 를 자유롭게 고를 수 있지만, **생명주기 위반**
/// (페이지를 열지 않고 그리기 등)은 백엔드마다 다르게 판정되면 안 된다. 그래서
/// 이 크레이트가 제공하는 백엔드는 모두 이 타입을 쓰고, 외부 백엔드도 자기 오류
/// 타입에 `From<RenderBackendError>` 를 달아 같은 판정을 재사용하도록 권한다.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderBackendError {
    /// `begin_page` 없이 `draw`/`end_page` 를 불렀다.
    NoOpenPage {
        /// 위반한 호출 이름(`"draw"` / `"end_page"`).
        call: &'static str,
    },
    /// 이미 열린 페이지가 있는데 `begin_page` 를 또 불렀다.
    PageAlreadyOpen,
    /// 페이지를 닫지 않고 `finish` 를 불렀다.
    UnclosedPage {
        /// `finish` 시점까지 정상으로 닫힌 페이지 수.
        pages_completed: usize,
    },
    /// 페이지 치수가 유한한 양수가 아니다.
    InvalidPageSize {
        /// 문제의 폭 (px).
        width: f64,
        /// 문제의 높이 (px).
        height: f64,
    },
    /// 백엔드가 그 op 를 표현할 수 없다 — capabilities 로 미리 질의해 피할 수 있는 실패다.
    UnsupportedOp {
        /// 백엔드 이름(`BackendCapabilities::name`).
        backend: &'static str,
        /// op 종류 이름(`super::paint_op_kind`).
        op: &'static str,
    },
    /// 감싼 기존 백엔드가 낸 오류를 문자열로 옮긴 것.
    ///
    /// 어댑터가 기존 `HwpError` 를 잃지 않고 전달하는 통로다.
    Backend(String),
}

impl std::fmt::Display for RenderBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoOpenPage { call } => {
                write!(f, "열린 페이지가 없습니다: {call} 앞에 begin_page 가 필요합니다")
            }
            Self::PageAlreadyOpen => {
                write!(f, "이미 열린 페이지가 있습니다: end_page 없이 begin_page 를 다시 부를 수 없습니다")
            }
            Self::UnclosedPage { pages_completed } => write!(
                f,
                "닫지 않은 페이지가 있습니다: 완료된 페이지 {pages_completed}개 뒤에 end_page 가 필요합니다"
            ),
            Self::InvalidPageSize { width, height } => {
                write!(f, "유효하지 않은 페이지 치수: {width}x{height} px")
            }
            Self::UnsupportedOp { backend, op } => {
                write!(f, "백엔드 {backend}가 지원하지 않는 op 입니다: {op}")
            }
            Self::Backend(msg) => write!(f, "백엔드 오류: {msg}"),
        }
    }
}

impl std::error::Error for RenderBackendError {}

impl From<HwpError> for RenderBackendError {
    fn from(value: HwpError) -> Self {
        Self::Backend(value.to_string())
    }
}

impl From<RenderBackendError> for HwpError {
    fn from(value: RenderBackendError) -> Self {
        HwpError::RenderError(value.to_string())
    }
}

/// 한 페이지 분량의 paint op 를 받아 자기 형식으로 그리는 출력 백엔드.
///
/// # 생명주기 (불변식)
///
/// 호출 순서는 다음 정규식과 같아야 한다.
///
/// ```text
/// ( begin_page  draw*  end_page )*  finish
/// ```
///
/// 이를 어기면 백엔드는 **오류를 내야 하고, 조용히 넘어가면 안 된다**.
/// `super::PageState` 가 이 판정을 한 곳에 모아두므로 새 백엔드는 그걸 쓰면 된다.
///
/// # 좌표·단위 계약
///
/// - 모든 좌표와 치수는 **px** 다. `PaintOp::bounds()` 의 `BoundingBox` 도 px 다.
/// - 원점은 **페이지 왼쪽 위**, **y 는 아래로 증가**한다.
///   (`crate::paint::PAGE_LAYER_TREE_COORDINATE_SYSTEM == "page-top-left-y-down"`)
/// - 좌표는 **페이지 절대 좌표**다. 그룹·클립 조상에 따른 누적 변환이 없다 —
///   `PaintOp` 는 이미 평탄화된 leaf op 이기 때문이다.
/// - 백엔드가 자기 형식 단위(pt, device px, mm)로 바꾸는 것은 백엔드 내부 일이며,
///   그 변환 계수는 이 trait 표면에 드러나지 않는다.
///
/// # 왜 `finish(self)` 인가
///
/// 출력물(문자열·바이트열)의 소유권을 넘기기 위해서다. 대신 trait object 로는
/// `finish` 를 부를 수 없으므로, `Box<dyn RenderBackend<..>>` 를 위해
/// [`RenderBackend::finish_boxed`] 를 함께 둔다.
pub trait RenderBackend {
    /// 이 백엔드가 최종적으로 내놓는 산출물 타입(예: SVG `String`, PDF `Vec<u8>`).
    type Output;
    /// 이 백엔드가 내는 오류 타입. 생명주기 위반은 [`RenderBackendError`] 와 같은
    /// 판정이어야 하므로 `From<RenderBackendError>` 를 다는 것을 권한다.
    type Error;

    /// 이 백엔드가 무엇을 할 수 있는지 스스로 밝힌다.
    ///
    /// 소비자는 백엔드 종류로 `match` 하지 말고 이 값을 **질의**해서 분기한다.
    fn capabilities(&self) -> BackendCapabilities;

    /// 새 페이지를 연다. 이미 열린 페이지가 있으면 오류다.
    fn begin_page(&mut self, size: PageSize) -> Result<(), Self::Error>;

    /// 열린 페이지에 leaf paint op 하나를 그린다.
    ///
    /// 페이지가 열려 있지 않으면 오류다. op 순서는 **그리기 순서**이며,
    /// 뒤에 온 op 가 앞의 op 위에 그려진다.
    fn draw(&mut self, op: &PaintOp) -> Result<(), Self::Error>;

    /// 열린 페이지를 닫는다. 열린 페이지가 없으면 오류다.
    fn end_page(&mut self) -> Result<(), Self::Error>;

    /// 백엔드를 소비해 산출물을 낸다. 닫지 않은 페이지가 있으면 오류다.
    fn finish(self) -> Result<Self::Output, Self::Error>
    where
        Self: Sized;

    /// `Box<dyn RenderBackend<..>>` 에서도 산출물을 꺼낼 수 있게 하는 통로.
    ///
    /// 구현은 거의 언제나 `(*self).finish()` 한 줄이다. `finish(self)` 는
    /// `Self: Sized` 를 요구해 vtable 에 올라가지 못하지만, `self: Box<Self>`
    /// 수신자는 object safe 이므로 이 메서드는 trait object 에서 호출된다.
    fn finish_boxed(self: Box<Self>) -> Result<Self::Output, Self::Error>;
}
