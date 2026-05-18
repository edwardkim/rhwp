//! EMF → SVG 컨버터. 단계 11은 DeviceContext/DcStack/ObjectTable 구조만 제공하고,
//! 실제 Player 로직(레코드 순회 → SVG 출력)은 단계 12~13에서 구현한다.

pub mod device_context;
pub mod player;
pub mod svg;

pub use device_context::{DcStack, DeviceContext, GraphicsObject, ObjectTable};
pub use player::Player;
pub use svg::{colorref_to_rgb, escape_xml, SvgBuilder};
