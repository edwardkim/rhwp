//! Bounded source-side preparation. No font I/O and no paint-time document scan.
use super::DocumentCore;
use crate::model::{control::Control, paragraph::Paragraph, shape::ShapeObject};
use crate::renderer::{
    canvas_text_font::CanvasTextFont,
    composer::compose_paragraph_for_metric_requests,
    layout::{resolved_to_text_style, trace_char_width_decisions},
    supplemental_metrics::{
        MetricContext, MetricError, SupplementalMetric, MAX_SUPPLEMENTAL_ENTRIES,
        MAX_SUPPLEMENTAL_KEY_BYTES,
    },
    TextStyle,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Serialize)]
pub struct CanvasMetricRequest {
    pub key: String,
    pub cluster: String,
    pub font: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasMetricBatch {
    pub revision: u64,
    pub requests: Vec<CanvasMetricRequest>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CanvasMetricReply {
    pub key: String,
    pub cluster: String,
    pub font: String,
    pub resolved_font: String,
    pub measured_advance_px: f64,
}

pub(super) struct PendingCanvasRequests {
    context: MetricContext,
    revision: u64,
    source: (u64, Vec<u64>, u64),
    entries: BTreeMap<String, (CanvasMetricRequest, TextStyle)>,
}

struct Collector<'a> {
    core: &'a DocumentCore,
    entries: BTreeMap<String, (CanvasMetricRequest, TextStyle)>,
    work: usize,
    bytes: usize,
}
impl Collector<'_> {
    fn budget(&mut self, work: usize, depth: usize) -> Result<(), MetricError> {
        self.work = self.work.saturating_add(work);
        if self.work > 4 * 1024 * 1024 || depth > 128 {
            return Err(MetricError::LimitExceeded);
        }
        Ok(())
    }
    fn paragraphs(&mut self, paragraphs: &[Paragraph], depth: usize) -> Result<(), MetricError> {
        for para in paragraphs {
            self.budget(1 + para.text.len() + para.controls.len(), depth)?;
            for line in compose_paragraph_for_metric_requests(para).lines {
                for run in line.runs {
                    // Markers/overlap have their own paint contracts, not ordinary text.
                    if run.footnote_marker.is_some()
                        || run.char_overlap.is_some()
                        || run.display_text.is_some()
                        || run.supplemental_metrics_blocked
                    {
                        continue;
                    }
                    let mut style = run.text_style(&self.core.styles);
                    style.supplemental_metrics = None;
                    let decisions = trace_char_width_decisions(&run.text, &style);
                    let mut index = 0;
                    for cluster in run.text.graphemes(true) {
                        let count = cluster.chars().count();
                        let decision = &decisions[index];
                        index += count;
                        let ch = cluster.chars().next().unwrap();
                        if count != 1
                            || decision.width_source != "heuristicHalfwidth"
                            || decision.dash_leader
                            || ch.is_whitespace()
                            || ch.is_control()
                            || crate::renderer::boxed_pua_number(ch).is_some()
                        {
                            continue;
                        }
                        let font = CanvasTextFont::for_positioned_text(&style, 0.0)
                            .descriptor()
                            .to_owned();
                        // Validate the same style/scalar/descriptor contract before exposing a request.
                        let metric = SupplementalMetric::from_canvas_measurement(
                            &style,
                            cluster,
                            &font,
                            font.clone(),
                            0.0,
                        )?;
                        let key = metric.request_key();
                        if self.entries.contains_key(&key) {
                            continue;
                        }
                        self.bytes = self.bytes.saturating_add(
                            key.len() + font.len() + cluster.len() + style.font_family.len(),
                        );
                        if self.entries.len() >= MAX_SUPPLEMENTAL_ENTRIES
                            || self.bytes > MAX_SUPPLEMENTAL_KEY_BYTES
                        {
                            return Err(MetricError::LimitExceeded);
                        }
                        self.entries.insert(
                            key.clone(),
                            (
                                CanvasMetricRequest {
                                    key,
                                    cluster: cluster.into(),
                                    font,
                                },
                                style.clone(),
                            ),
                        );
                    }
                }
            }
            for control in &para.controls {
                match control {
                    Control::Table(t) => {
                        if let Some(c) = &t.caption {
                            self.paragraphs(&c.paragraphs, depth + 1)?;
                        }
                        for cell in &t.cells {
                            self.paragraphs(&cell.paragraphs, depth + 1)?;
                        }
                    }
                    Control::Shape(s) => self.shape(s, depth + 1)?,
                    Control::Picture(p) => {
                        if let Some(c) = &p.caption {
                            self.paragraphs(&c.paragraphs, depth + 1)?;
                        }
                    }
                    Control::Header(h) => self.paragraphs(&h.paragraphs, depth + 1)?,
                    Control::Footer(h) => self.paragraphs(&h.paragraphs, depth + 1)?,
                    Control::Footnote(n) => self.paragraphs(&n.paragraphs, depth + 1)?,
                    Control::Endnote(n) => self.paragraphs(&n.paragraphs, depth + 1)?,
                    Control::HiddenComment(c) => self.paragraphs(&c.paragraphs, depth + 1)?,
                    Control::Field(f) => self.paragraphs(&f.memo_paragraphs, depth + 1)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }
    fn shape(&mut self, shape: &ShapeObject, depth: usize) -> Result<(), MetricError> {
        self.budget(1, depth)?;
        if let Some(drawing) = shape.drawing() {
            if let Some(t) = &drawing.text_box {
                self.paragraphs(&t.paragraphs, depth + 1)?;
            }
            if let Some(c) = &drawing.caption {
                self.paragraphs(&c.paragraphs, depth + 1)?;
            }
        }
        match shape {
            ShapeObject::Group(g) => {
                if let Some(c) = &g.caption {
                    self.paragraphs(&c.paragraphs, depth + 1)?;
                }
                for child in &g.children {
                    self.shape(child, depth + 1)?;
                }
            }
            ShapeObject::Picture(p) => {
                if let Some(c) = &p.caption {
                    self.paragraphs(&c.paragraphs, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl DocumentCore {
    /// Collect once per settled document revision, outside layout/paint.
    pub fn collect_canvas_metric_requests(
        &mut self,
        context: MetricContext,
    ) -> Result<CanvasMetricBatch, MetricError> {
        self.begin_canvas_metric_session(context)?;
        let mut collector = Collector {
            core: self,
            entries: BTreeMap::new(),
            work: 0,
            bytes: 0,
        };
        for section in &self.document.sections {
            collector.paragraphs(&section.paragraphs, 0)?;
            for master in &section.section_def.master_pages {
                collector.paragraphs(&master.paragraphs, 0)?;
            }
        }
        let entries = collector.entries;
        let requests = entries.values().map(|(r, _)| r.clone()).collect();
        let source = self.canvas_metric_source_revision();
        let session = self.canvas_metrics.as_mut().unwrap();
        session.next_request = session
            .next_request
            .checked_add(1)
            .ok_or(MetricError::LimitExceeded)?;
        let revision = session.next_request;
        self.canvas_metrics.as_mut().unwrap().pending = Some(PendingCanvasRequests {
            context,
            revision,
            source,
            entries,
        });
        Ok(CanvasMetricBatch { revision, requests })
    }

    pub fn register_canvas_metric_replies(
        &mut self,
        context: MetricContext,
        revision: u64,
        replies: Vec<CanvasMetricReply>,
    ) -> Result<bool, MetricError> {
        if self.batch_mode {
            return Err(MetricError::EditInProgress);
        }
        let pending = self
            .canvas_metrics
            .as_ref()
            .and_then(|s| s.pending.as_ref())
            .ok_or(MetricError::ContextMismatch)?;
        if pending.context != context
            || pending.revision != revision
            || pending.source != self.canvas_metric_source_revision()
        {
            return Err(MetricError::ContextMismatch);
        }
        if replies.len() != pending.entries.len() {
            return Err(MetricError::ContextMismatch);
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut entries = Vec::with_capacity(replies.len());
        for reply in replies {
            if !seen.insert(reply.key.clone()) {
                return Err(MetricError::DuplicateKey);
            }
            let (request, style) = pending
                .entries
                .get(&reply.key)
                .ok_or(MetricError::ContextMismatch)?;
            if reply.font != request.font || reply.cluster != request.cluster {
                return Err(MetricError::InvalidDescriptor);
            }
            entries.push(SupplementalMetric::from_canvas_measurement(
                style,
                &reply.cluster,
                &reply.font,
                reply.resolved_font,
                reply.measured_advance_px,
            )?);
        }
        let changed = self.register_canvas_metrics(context, entries)?;
        self.canvas_metrics.as_mut().unwrap().pending = None;
        Ok(changed)
    }

    fn canvas_metric_source_revision(&self) -> (u64, Vec<u64>, u64) {
        (
            self.render_normalization.document_epoch,
            self.render_normalization.section_revisions.clone(),
            self.dpi.to_bits(),
        )
    }
}
