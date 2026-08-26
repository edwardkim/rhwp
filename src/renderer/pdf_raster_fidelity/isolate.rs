//! Page-isolated raster compare. One page failure does not abort the document.

use super::catalog::{CorpusId, IsolationPolicy, PageRecord};
use super::classify::{classify_page_residual, ClassificationInput};
use super::contract::locked_page_count;
use super::fingerprint::RasterFingerprint;
use super::report::{IsolatedPageOutcome, IsolatedPageStatus, ResidualDocumentReport};
use super::wrap::WrapGeometry;
use super::ResidualClass;

#[derive(Debug, Clone)]
pub struct IsolatedRasterReport {
    pub document: ResidualDocumentReport,
    pub outcomes: Vec<IsolatedPageOutcome>,
}

#[derive(Debug, Clone)]
pub struct IsolatedPageInput<'a> {
    pub record: Option<&'a PageRecord>,
    pub oracle: Result<&'a RasterFingerprint, String>,
    pub candidate: Result<&'a RasterFingerprint, String>,
    pub wrap: Option<WrapGeometry>,
    pub face_substituted: bool,
    pub table_boxes_match: bool,
    pub line_owners_match: bool,
}

pub fn compare_document_pages(
    corpus: CorpusId,
    isolation: IsolationPolicy,
    pages: &[IsolatedPageInput<'_>],
) -> Result<IsolatedRasterReport, String> {
    if pages.is_empty() {
        return Err("raster compare requires at least one page".to_string());
    }
    let mut outcomes = Vec::with_capacity(pages.len());
    for (index, page) in pages.iter().enumerate() {
        outcomes.push(compare_one_page(index as u16, isolation, page));
    }
    if outcomes
        .iter()
        .all(|o| o.status == IsolatedPageStatus::Fatal)
    {
        return Err("every page failed fatally".to_string());
    }
    if isolation == IsolationPolicy::DocumentIfAllSkipped
        && outcomes
            .iter()
            .all(|o| o.status == IsolatedPageStatus::Skipped)
    {
        return Err("every page was skipped".to_string());
    }
    let expected = locked_page_count(corpus).filter(|&lock| pages.len() as u16 == lock);
    let document = ResidualDocumentReport::from_outcomes(corpus, expected, &outcomes, isolation);
    Ok(IsolatedRasterReport { document, outcomes })
}

fn compare_one_page(
    page_index: u16,
    isolation: IsolationPolicy,
    page: &IsolatedPageInput<'_>,
) -> IsolatedPageOutcome {
    match (&page.oracle, &page.candidate) {
        (Err(err), _) | (_, Err(err)) => isolate_decode(page_index, isolation, err),
        (Ok(oracle), Ok(candidate)) => IsolatedPageOutcome {
            page_index,
            status: IsolatedPageStatus::Compared,
            residual: classify_page_residual(ClassificationInput {
                record: page.record,
                oracle,
                candidate,
                wrap: page.wrap,
                face_substituted: page.face_substituted,
                table_boxes_match: page.table_boxes_match,
                line_owners_match: page.line_owners_match,
            }),
            isolated: false,
            message: String::new(),
        },
    }
}

fn isolate_decode(page_index: u16, isolation: IsolationPolicy, err: &str) -> IsolatedPageOutcome {
    match isolation {
        IsolationPolicy::Independent | IsolationPolicy::SkipOnDecodeFail => IsolatedPageOutcome {
            page_index,
            status: IsolatedPageStatus::IsolatedWarning,
            residual: ResidualClass::None,
            isolated: true,
            message: format!("page {} isolated: {err}", page_index + 1),
        },
        IsolationPolicy::DocumentIfAllSkipped => IsolatedPageOutcome {
            page_index,
            status: IsolatedPageStatus::Skipped,
            residual: ResidualClass::None,
            isolated: true,
            message: format!("page {} skipped: {err}", page_index + 1),
        },
    }
}

/// Public helper used by fixture-driven tests. Keeps the input type crate-visible.
pub fn page_input<'a>(
    record: Option<&'a PageRecord>,
    oracle: Result<&'a RasterFingerprint, String>,
    candidate: Result<&'a RasterFingerprint, String>,
) -> IsolatedPageInput<'a> {
    IsolatedPageInput {
        record,
        oracle,
        candidate,
        wrap: None,
        face_substituted: false,
        table_boxes_match: true,
        line_owners_match: true,
    }
}
