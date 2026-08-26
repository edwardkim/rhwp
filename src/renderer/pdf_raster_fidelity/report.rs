//! Isolated residual report. One page never wipes the document ledger.

use super::catalog::{CorpusId, IsolationPolicy};
use super::ResidualClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolatedPageStatus {
    Compared,
    IsolatedWarning,
    Skipped,
    Fatal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolatedPageOutcome {
    pub page_index: u16,
    pub status: IsolatedPageStatus,
    pub residual: ResidualClass,
    pub isolated: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResidualPageReport {
    pub page_index: u16,
    pub human_page: u16,
    pub status: IsolatedPageStatus,
    pub residual: ResidualClass,
    pub isolated: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResidualDocumentReport {
    pub corpus: CorpusId,
    pub expected_page_count: Option<u16>,
    pub observed_page_count: u16,
    pub compared_pages: u16,
    pub isolated_pages: u16,
    pub layout_defects: u16,
    pub font_env_pages: u16,
    pub skipped_pages: u16,
    pub page_count_ok: bool,
    pub document_ok: bool,
    pub pages: Vec<ResidualPageReport>,
}

impl ResidualDocumentReport {
    pub fn from_outcomes(
        corpus: CorpusId,
        expected_page_count: Option<u16>,
        outcomes: &[IsolatedPageOutcome],
        isolation: IsolationPolicy,
    ) -> Self {
        let observed_page_count = outcomes.len() as u16;
        let mut compared_pages = 0;
        let mut isolated_pages = 0;
        let mut layout_defects = 0;
        let mut font_env_pages = 0;
        let mut skipped_pages = 0;
        let mut pages = Vec::with_capacity(outcomes.len());
        for outcome in outcomes {
            match outcome.status {
                IsolatedPageStatus::Compared => compared_pages += 1,
                IsolatedPageStatus::IsolatedWarning | IsolatedPageStatus::Skipped => {
                    isolated_pages += 1;
                    if outcome.status == IsolatedPageStatus::Skipped {
                        skipped_pages += 1;
                    }
                }
                IsolatedPageStatus::Fatal => {}
            }
            if outcome.residual == ResidualClass::FontEnv {
                font_env_pages += 1;
            }
            if outcome.residual.is_layout_defect() {
                layout_defects += 1;
            }
            pages.push(ResidualPageReport {
                page_index: outcome.page_index,
                human_page: outcome.page_index.saturating_add(1),
                status: outcome.status,
                residual: outcome.residual,
                isolated: outcome.isolated,
                message: outcome.message.clone(),
            });
        }
        let page_count_ok = match expected_page_count {
            Some(locked) => observed_page_count == locked,
            None => observed_page_count > 0,
        };
        let all_skipped = observed_page_count > 0 && skipped_pages == observed_page_count;
        let document_ok = page_count_ok
            && !outcomes
                .iter()
                .any(|o| o.status == IsolatedPageStatus::Fatal)
            && !(all_skipped && isolation == IsolationPolicy::DocumentIfAllSkipped);
        Self {
            corpus,
            expected_page_count,
            observed_page_count,
            compared_pages,
            isolated_pages,
            layout_defects,
            font_env_pages,
            skipped_pages,
            page_count_ok,
            document_ok,
            pages,
        }
    }
}
