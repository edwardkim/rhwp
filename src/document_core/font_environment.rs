use super::DocumentCore;
use crate::{error::HwpError, renderer::font_environment::FontEnvironment};

impl DocumentCore {
    /// 호스트가 실제 프로그램을 준비할 문서 후보와 현재 명시 환경을 반환한다.
    pub fn get_document_font_selection_plan(&self) -> String {
        serde_json::json!({
            "requests": FontEnvironment::document_requests(&self.document.doc_info),
            "environmentId": self.font_environment.as_ref().map(FontEnvironment::id),
            "embeddedFamilies": FontEnvironment::embedded_program_names(&self.document.doc_info),
        })
        .to_string()
    }

    /// 설치 조회 정책을 복제하지 않고 실제 공급 snapshot으로 공통 환경을 계산한다.
    pub fn build_font_environment_from_available_programs(
        &self,
        json: &str,
    ) -> Result<String, HwpError> {
        let environment = FontEnvironment::from_available_programs(&self.document.doc_info, json)?;
        serde_json::to_string(&environment).map_err(|e| HwpError::RenderError(e.to_string()))
    }

    /// Change the rendering environment without modifying document fonts or saved lines.
    /// None restores the portable default. Repeating an environment is a no-op.
    pub fn set_font_environment(
        &mut self,
        environment: Option<FontEnvironment>,
    ) -> Result<bool, HwpError> {
        if self.batch_mode {
            return Err(HwpError::RenderError(
                "font environment cannot change during an edit batch".into(),
            ));
        }
        if let Some(environment) = &environment {
            environment.validate()?;
        }
        if self.font_environment == environment && self.host_font_environment.is_none() {
            return Ok(false);
        }
        self.font_environment = environment;
        self.host_font_environment = None;
        // Measurements and pending requests from the former font generation must not survive.
        self.canvas_metrics = None;
        self.pending_pagination_job = None;
        self.deferred_pagination_descriptor = None;
        self.rebuild_derived_state();
        Ok(true)
    }

    pub fn font_environment(&self) -> Option<&FontEnvironment> {
        self.font_environment.as_ref()
    }
}
