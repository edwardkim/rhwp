//! paint 전용 원본 HFT 프로그램 등록. 측정·저장 정보는 변경하지 않는다.

use crate::{document_core::DocumentCore, error::HwpError};

impl DocumentCore {
    /// 호스트 글꼴 세대 교체 시 특정 family의 오래된 paint 공급을 폐기한다.
    pub fn clear_text_font_family_native(&mut self, family: &str) {
        self.hft_outline_fonts.clear_family(family);
    }

    /// 실제 FontFace의 cmap와 선택적 Print TTF 윤곽을 등록한다. 저장·측정 정보는 바꾸지 않는다.
    pub fn register_text_font_glyph_coverage_native(
        &mut self,
        family: &str,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, HwpError> {
        let changed = self
            .hft_outline_fonts
            .pua_coverage
            .register(family, bytes, face_index)
            .map_err(|message| HwpError::RenderError(message.into()))?;
        #[cfg(target_arch = "wasm32")]
        self.hft_outline_fonts
            .register_print_font(family, bytes, face_index);
        Ok(changed)
    }
    /// 호스트가 실제 FontFace에 적재한 바이트를 원본 HFT family에 연결한다.
    /// 지원하지 않는 face는 기존 글꼴 painter를 계속 사용한다.
    pub fn register_hft_outline_font_native(
        &mut self,
        family: &str,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, HwpError> {
        self.register_hft_outline_font_candidates_native(family, &[(bytes, face_index)])
    }

    /// 같은 표시 family의 분할 프로그램을 실제 후보 순서로 등록한다.
    pub fn register_hft_outline_font_candidates_native(
        &mut self,
        family: &str,
        candidates: &[(&[u8], u32)],
    ) -> Result<bool, HwpError> {
        let family = family.trim();
        let environment = self.resolved_font_environment();
        let is_source_hft =
            self.document
                .doc_info
                .font_faces
                .iter()
                .enumerate()
                .any(|(language, faces)| {
                    faces.iter().enumerate().any(|(index, face)| {
                        if face.alt_type != 2 {
                            return false;
                        }
                        // 저장된 이름을 바꾸지 않고 조판이 선택한 같은 슬롯의 표시 family를 쓴다.
                        let selected = u16::try_from(index).ok().and_then(|font_id| {
                            environment
                                .and_then(|env| env.replacement_for(&face.name, language, font_id))
                        });
                        selected.unwrap_or(&face.name).to_lowercase() == family.to_lowercase()
                    })
                });
        if !is_source_hft {
            return Ok(false);
        }
        self.hft_outline_fonts
            .register_candidates(family, candidates)
            .map_err(|message| HwpError::RenderError(message.into()))
    }
}
