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

    /// 원 HFT 또는 선택 신원이 검증된 CFF 조각을 실제 후보 순서로 등록한다.
    /// 유형 미상 원본을 HFT로 재분류하지 않으며 공급된 paint 프로그램만 보관한다.
    pub fn register_hft_outline_font_candidates_native(
        &mut self,
        family: &str,
        candidates: &[(&[u8], u32)],
    ) -> Result<bool, HwpError> {
        use sha2::{Digest, Sha256};
        let family = family.trim();
        let environment = self.resolved_font_environment();
        // 실제 선택 신원과 첫 공급 face가 같은 경우에만 유형 미상의 CFF도 허용한다.
        // 원본 유형·조판 폭을 다시 분류하지 않으며 기존 프로그램 바이트 한도를 먼저 확인한다.
        let supplied_identity = candidates
            .first()
            .filter(|(bytes, _)| {
                !bytes.is_empty() && bytes.len() <= crate::renderer::hft_outline::MAX_SOURCE_BYTES
            })
            .map(|(bytes, index)| {
                let mut hash = Sha256::new();
                hash.update(bytes);
                let source: [u8; 32] = hash.clone().finalize().into();
                hash.update(index.to_be_bytes());
                let indexed: [u8; 32] = hash.finalize().into();
                (source, indexed, *index)
            });
        let owns_supplied_outline =
            self.document
                .doc_info
                .font_faces
                .iter()
                .enumerate()
                .any(|(language, faces)| {
                    faces.iter().enumerate().any(|(index, face)| {
                        // 저장된 이름을 바꾸지 않고 조판이 선택한 같은 슬롯의 표시 family를 쓴다.
                        let Ok(font_id) = u16::try_from(index) else {
                            return false;
                        };
                        let selected = environment
                            .and_then(|env| env.replacement_for(&face.name, language, font_id));
                        let matches_selected =
                            selected.unwrap_or(&face.name).to_lowercase() == family.to_lowercase();
                        if face.alt_type == 2 && matches_selected {
                            return true;
                        }
                        let Some(env) = environment else {
                            return false;
                        };
                        let matches_query = env
                            .selected_program_query_for(&face.name, language, font_id)
                            .is_some_and(|query| query.to_lowercase() == family.to_lowercase());
                        (matches_selected || matches_query)
                            && supplied_identity.is_some_and(|(source, indexed, index)| {
                                env.selected_program_identity_for(&face.name, language, font_id)
                                    .is_some_and(|identity| {
                                        identity.matches(&source, &indexed, index)
                                    })
                            })
                    })
                });
        if !owns_supplied_outline {
            return Ok(false);
        }
        self.hft_outline_fonts
            .register_candidates(family, candidates)
            .map_err(|message| HwpError::RenderError(message.into()))
    }
}
