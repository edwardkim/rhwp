use super::DocumentCore;
use crate::{error::HwpError, renderer::font_environment::FontEnvironment};

impl DocumentCore {
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
        if self.font_environment == environment {
            return Ok(false);
        }
        self.font_environment = environment;
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
