//! WMF 이미지 내부 글꼴의 실제 공급과 paint 세대를 문서 측정 계약에서 분리한다.

use crate::{document_core::DocumentCore, error::HwpError};

impl DocumentCore {
    /// 호스트가 선택한 실제 SFNT/TTC face를 메타파일 paint에 공급한다.
    pub fn register_metafile_font_native(
        &mut self,
        family: &str,
        bytes: &[u8],
        face_index: u32,
    ) -> Result<bool, HwpError> {
        let changed = self
            .metafile_fonts
            .register(family, bytes, face_index)
            .map_err(|message| HwpError::RenderError(message.into()))?;
        if changed {
            // 조판·원본·페이지네이션은 그대로이며 해당 글꼴을 소비한 paint만 다시 만든다.
            self.page_layer_tree_cache.borrow_mut().clear();
            self.layer_tree_json_cache.borrow_mut().clear();
        }
        Ok(changed)
    }

    /// 호스트 글꼴 세대가 교체되면 이전 문서의 paint font와 surface도 보존하지 않는다.
    pub fn clear_metafile_fonts_native(&mut self) {
        self.metafile_fonts = Default::default();
        self.page_layer_tree_cache.borrow_mut().clear();
        self.layer_tree_json_cache.borrow_mut().clear();
    }

    /// Canvas font 요청으로는 관측할 수 없는 SVG 이미지 내부의 family·문자 요구다.
    /// 조판 전에 원본 BinData를 읽으며, unsupported WMF를 성공한 변환으로 바꾸지 않는다.
    pub fn get_metafile_font_requests_native(&self) -> Result<String, HwpError> {
        if let Some(json) = self.metafile_fonts.cached_requests(self.bin_data_epoch) {
            return Ok(json);
        }
        let mut requests = Vec::new();
        {
            for item in &self.document.bin_data_content {
                let Some(data) = item
                    .data
                    .load_limited_shared(crate::model::bin_data::MAX_BIN_DATA_BYTES)
                else {
                    continue;
                };
                if crate::renderer::image_resolver::detect_image_mime_type(&data) != "image/x-wmf" {
                    continue;
                }
                let Some(svg) = crate::renderer::svg::convert_wmf_to_svg(&data) else {
                    continue;
                };
                if let Some(mut items) = crate::paint::metafile_fonts::requests(&svg) {
                    requests.append(&mut items);
                }
            }
        }
        let json =
            serde_json::to_string(&requests).map_err(|e| HwpError::RenderError(e.to_string()))?;
        self.metafile_fonts
            .cache_requests(self.bin_data_epoch, &json);
        Ok(json)
    }
}
