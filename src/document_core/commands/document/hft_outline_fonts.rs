//! paint 전용 원본 HFT 프로그램 등록. 측정·저장 정보는 변경하지 않는다.

use crate::{document_core::DocumentCore, error::HwpError};

impl DocumentCore {
    /// 실제 적재한 FontFace의 cmap만 paint용으로 등록한다. 저장·측정 정보는 바꾸지 않는다.
    pub fn register_text_font_glyph_coverage_native(
        &mut self,
        family: &str,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, HwpError> {
        self.hft_outline_fonts
            .pua_coverage
            .register(family, bytes, face_index)
            .map_err(|message| HwpError::RenderError(message.into()))
    }
    /// 호스트가 실제 FontFace에 적재한 바이트를 원본 HFT family에 연결한다.
    /// 지원하지 않는 face는 기존 글꼴 painter를 계속 사용한다.
    pub fn register_hft_outline_font_native(
        &mut self,
        family: &str,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, HwpError> {
        let family = family.trim();
        let is_source_hft = self
            .document
            .doc_info
            .font_faces
            .iter()
            .flatten()
            .any(|face| face.alt_type == 2 && face.name.to_lowercase() == family.to_lowercase());
        if !is_source_hft {
            return Ok(false);
        }
        self.hft_outline_fonts
            .register(family, bytes, face_index)
            .map_err(|message| HwpError::RenderError(message.into()))
    }
}
