//! Backend-bound metric ownership. Font data is render state, never document IR.

use std::sync::Arc;

use crate::error::HwpError;
use crate::renderer::supplemental_metrics::{
    MetricBackend, MetricContext, MetricError, SupplementalMetric, SupplementalMetricSnapshot,
    SupplementalMetricStore,
};

use super::DocumentCore;

pub(crate) struct CanvasMetricSession {
    context: MetricContext,
    store: SupplementalMetricStore,
    enabled: bool,
    pub(super) pending: Option<super::canvas_metric_requests::PendingCanvasRequests>,
    pub(super) next_request: u64,
}

impl CanvasMetricSession {
    pub(crate) fn active_snapshot(&self) -> Option<Arc<SupplementalMetricSnapshot>> {
        self.enabled.then(|| self.store.snapshot()).flatten()
    }
}

impl DocumentCore {
    /// Bind the renderer's document/font generation. Changing context discards
    /// old measurements before restoring portable pagination. Repeating it is a no-op.
    pub fn begin_canvas_metric_session(
        &mut self,
        context: MetricContext,
    ) -> Result<(), MetricError> {
        if self.batch_mode {
            return Err(MetricError::EditInProgress);
        }
        if context.backend != MetricBackend::Canvas2d {
            return Err(MetricError::ContextMismatch);
        }
        if self
            .canvas_metrics
            .as_ref()
            .is_some_and(|s| s.context == context)
        {
            return Ok(());
        }
        let was_active = self.canvas_metrics_active();
        self.canvas_metrics = Some(CanvasMetricSession {
            context,
            store: SupplementalMetricStore::new(context),
            enabled: false,
            pending: None,
            next_request: 0,
        }); // dropping the old owner invalidates retained TextStyles too
        self.styles.supplemental_metrics = None;
        if was_active {
            self.rebuild_derived_state();
        }
        Ok(())
    }

    /// Validate the whole batch before publishing. Unchanged data does not cause
    /// another pagination pass. Registration alone does not select Canvas2D.
    pub fn register_canvas_metrics(
        &mut self,
        context: MetricContext,
        entries: Vec<SupplementalMetric>,
    ) -> Result<bool, MetricError> {
        if self.batch_mode {
            return Err(MetricError::EditInProgress);
        }
        let session = self
            .canvas_metrics
            .as_mut()
            .ok_or(MetricError::ContextMismatch)?;
        let before = session.store.snapshot();
        session.store.replace(context, entries)?;
        let after = session.store.snapshot();
        let changed = match (&before, &after) {
            (Some(a), Some(b)) => !Arc::ptr_eq(a, b),
            (None, None) => false,
            _ => true,
        };
        if changed && session.enabled {
            self.rebuild_derived_state();
        }
        Ok(changed)
    }

    /// Switch the whole layout context, not just the painter. Portable exports
    /// must use false; a Canvas2D view may restore true after the export ends.
    pub fn select_canvas_metrics(&mut self, enabled: bool) -> Result<bool, MetricError> {
        if self.batch_mode {
            return Err(MetricError::EditInProgress);
        }
        let Some(session) = self.canvas_metrics.as_mut() else {
            return if enabled {
                Err(MetricError::ContextMismatch)
            } else {
                Ok(false)
            };
        };
        if enabled && session.store.snapshot().is_none() {
            return Err(MetricError::ContextMismatch);
        }
        if session.enabled == enabled {
            return Ok(false);
        }
        session.enabled = enabled;
        self.rebuild_derived_state();
        Ok(true)
    }

    pub fn canvas_metrics_active(&self) -> bool {
        self.canvas_metrics.as_ref().is_some_and(|s| s.enabled)
    }

    /// Do not export positions derived from Canvas's implicit fallback to a
    /// backend that cannot paint that same source. Caller must select portable
    /// metrics (and thus rebuild pagination) before enumerating export pages.
    pub(crate) fn require_portable_metrics(&self) -> Result<(), HwpError> {
        if self.canvas_metrics_active() {
            return Err(HwpError::RenderError(
                "Canvas2D supplemental metrics are active; select portable metrics before exporting"
                    .into(),
            ));
        }
        Ok(())
    }
}
