//! EMF 공통 구조체 — RECTL/POINTL/SIZEL, Header 등. 단계 10은 헤더에 필요한 것만.

pub mod header;
pub mod logbrush;
pub mod logfont;
pub mod logpen;
pub mod rectl;
pub mod xform;

pub use header::Header;
pub use logbrush::LogBrush;
pub use logfont::LogFontW;
pub use logpen::{LogPen, PenWidthKind};
pub use rectl::{PointL, RectL, SizeL};
pub use xform::XForm;
