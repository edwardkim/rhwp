//! Request ownership is separate from the editable document's lifetime.
use super::DocumentCore;
use crate::error::HwpError;
use crate::parser::pdf::{PdfImportError, PdfImportOptions, PdfImportPolicy, PdfImportProgress};
use pdf_oxide::import_limits::{ImportBudget, ImportBudgetSnapshot};
use std::sync::Arc;

struct OwnerGuard<F>(F);
impl<F> std::fmt::Debug for OwnerGuard<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OwnerGuard")
    }
}
impl<F: Fn() -> bool + Send + Sync> pdf_oxide::import_limits::ImportWorkGuard for OwnerGuard<F> {
    fn permits_work(&self) -> bool {
        (self.0)()
    }
}

#[derive(Debug)]
struct ReopenReservation {
    active: Arc<std::sync::atomic::AtomicBool>,
    inner: Arc<dyn crate::storage::Reservation>,
}
impl crate::storage::Reservation for ReopenReservation {
    fn hash_payload(&self, element_bytes: usize, capacity: usize) -> Result<(), String> {
        if self.active.load(std::sync::atomic::Ordering::Acquire) {
            self.inner.hash_payload(element_bytes, capacity)
        } else {
            Ok(())
        }
    }
    fn reject(&self, reason: &'static str) -> String {
        if self.active.load(std::sync::atomic::Ordering::Acquire) {
            self.inner.reject(reason)
        } else {
            reason.into()
        }
    }
}
struct ReopenLease(Arc<std::sync::atomic::AtomicBool>);
impl Drop for ReopenLease {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}

/// One PDF initialization request with shared parser and Core storage accounting.
///
/// The session never becomes part of the returned document. Ordinary editing
/// therefore does not consume the initial import's cumulative counters. Hosts
/// still supervise process/worker memory and execution time; these counters do
/// not measure every renderer allocation or the process heap.
pub struct PdfImportSession {
    request: Arc<ImportBudget>,
    policy: PdfImportPolicy,
    started: bool,
    failed: bool,
}

impl PdfImportSession {
    pub fn new(policy: PdfImportPolicy) -> Result<Self, HwpError> {
        Self::new_inner(policy, None)
    }

    /// The owner supplies a cooperative check, not preemption of opaque work.
    pub fn new_with_work_guard<F: Fn() -> bool + Send + Sync + 'static>(
        policy: PdfImportPolicy,
        guard: F,
    ) -> Result<Self, HwpError> {
        Self::new_inner(policy, Some(Arc::new(OwnerGuard(guard))))
    }

    fn new_inner(
        policy: PdfImportPolicy,
        guard: Option<Arc<dyn pdf_oxide::import_limits::ImportWorkGuard>>,
    ) -> Result<Self, HwpError> {
        let fail =
            |error| HwpError::InvalidFile(crate::parser::ParseError::PdfError(error).to_string());
        policy.validate().map_err(fail)?;
        let request = ImportBudget::new(policy.backend)
            .map_err(crate::parser::pdf::PdfImportError::backend)
            .map_err(fail)?;
        let request = if let Some(guard) = guard {
            request.with_work_guard(guard)
        } else {
            request
        };
        Ok(Self {
            request: Arc::new(request),
            policy,
            started: false,
            failed: false,
        })
    }

    /// Reserve a caller-owned transport copy before its backing is allocated.
    /// This counts logical payload work, not JavaScript or process live heap.
    pub fn reserve_transfer(&mut self, bytes: usize) -> Result<(), HwpError> {
        if self.failed {
            return Err(HwpError::InvalidFile(
                "PDF_IMPORT_SESSION: failed transfer request".into(),
            ));
        }
        let result = self.request.charge_decoded(bytes).map_err(|error| {
            self.storage_error(
                "transport backing",
                crate::parser::pdf::PdfImportError::backend(error),
            )
        });
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    /// Reopen the generated HWPX handoff under the owner's remaining policy.
    /// Archive/decoded backing and shared Core initialization consume this
    /// request. The legacy XML parser's object allocations and process heap are
    /// not thereby certified. Lazy assets detach from the request on return.
    pub fn reopen_hwpx_export(
        &mut self,
        data: &[u8],
        maximum: usize,
    ) -> Result<DocumentCore, HwpError> {
        if self.failed || std::mem::replace(&mut self.started, true) {
            return Err(HwpError::InvalidFile(
                "PDF_IMPORT_SESSION: request already started".into(),
            ));
        }
        let output = (|| {
            if maximum == 0
                || maximum > crate::pdf_runtime::PdfTransferLimits::standard().output_bytes
                || data.len() > maximum
            {
                return Err(HwpError::InvalidFile(
                    "PDF_RESOURCE_LIMIT: editor input byte limit".into(),
                ));
            }
            self.request.check().map_err(|e| {
                self.storage_error(
                    "editor reopen",
                    crate::parser::pdf::PdfImportError::backend(e),
                )
            })?;
            if crate::parser::detect_format(data) != crate::parser::FileFormat::Hwpx {
                return Err(HwpError::InvalidFile(
                    "PDF_EDITOR_REOPEN: expected generated HWPX".into(),
                ));
            }
            self.request
                .charge_decoded(
                    std::mem::size_of::<ReopenReservation>()
                        + std::mem::size_of::<std::sync::atomic::AtomicBool>()
                        + 4 * std::mem::size_of::<usize>(),
                )
                .map_err(|e| {
                    self.storage_error(
                        "editor reopen",
                        crate::parser::pdf::PdfImportError::backend(e),
                    )
                })?;
            let active = Arc::new(std::sync::atomic::AtomicBool::new(true));
            let _lease = ReopenLease(Arc::clone(&active));
            let reservation = crate::parser::pdf::allocation::payload_reservation(&self.request)
                .map_err(|e| self.storage_error("editor input", e))?;
            let reservation: Arc<dyn crate::storage::Reservation> = Arc::new(ReopenReservation {
                active,
                inner: reservation,
            });
            let parsed = crate::parser::parse_pdf_editor_hwpx(data, reservation)
                .map_err(|e| HwpError::InvalidFile(e.to_string()))?;
            // A permissive legacy XML branch cannot hide a fatal reservation.
            self.request.check().map_err(|e| {
                self.storage_error(
                    "editor reopen",
                    crate::parser::pdf::PdfImportError::backend(e),
                )
            })?;
            let mut core = DocumentCore::from_parsed_document_with_storage(
                parsed,
                crate::parser::FileFormat::Hwpx,
                &crate::parser::pdf::allocation::Allocation(&self.request),
            )?;
            // HWPX is the worker's transfer container, not the imported source.
            // Keep its load semantics, but use PDF export adaptation thereafter:
            // HWPX-only picture-extra normalization would erase the PDF crop's
            // explicit image reference dimensions on the next HWP save (#8403).
            core.source_format = crate::parser::FileFormat::Pdf;
            self.request.check().map_err(|e| {
                self.storage_error(
                    "editor reopen",
                    crate::parser::pdf::PdfImportError::backend(e),
                )
            })?;
            Ok(core)
        })();
        if output.is_err() {
            self.failed = true;
        }
        output
    }

    /// A failed request cannot be reused. Create a fresh session for recovery.
    pub fn open(
        &mut self,
        bytes: &[u8],
        options: &PdfImportOptions,
    ) -> Result<DocumentCore, HwpError> {
        self.open_with_progress(bytes, options, &mut |_| Ok(()))
    }

    /// Reports actual selected-page conversion boundaries. Final Core creation
    /// and serialization follow the last page; these are not elapsed-time estimates.
    /// A callback error aborts and invalidates this request like a parser error.
    pub fn open_with_progress(
        &mut self,
        bytes: &[u8],
        options: &PdfImportOptions,
        progress: &mut dyn FnMut(PdfImportProgress) -> Result<(), PdfImportError>,
    ) -> Result<DocumentCore, HwpError> {
        if self.failed || std::mem::replace(&mut self.started, true) {
            return Err(HwpError::InvalidFile(
                "PDF_IMPORT_SESSION: request already started; create a fresh session".into(),
            ));
        }
        let result = self.open_inner(bytes, options, progress);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn open_inner(
        &self,
        bytes: &[u8],
        options: &PdfImportOptions,
        progress: &mut dyn FnMut(PdfImportProgress) -> Result<(), PdfImportError>,
    ) -> Result<DocumentCore, HwpError> {
        let parsed = crate::parser::parse_pdf_with_metadata_and_request(
            bytes,
            options,
            self.policy,
            Arc::clone(&self.request),
            progress,
        )
        .map_err(|error| HwpError::InvalidFile(error.to_string()))?;
        let mut core = DocumentCore::from_parsed_document_with_storage(
            parsed,
            crate::parser::FileFormat::Pdf,
            &crate::parser::pdf::allocation::Allocation(&self.request),
        )?;
        self.request
            .check()
            .map_err(|error| HwpError::InvalidFile(error.to_string()))?;
        if let Some(metadata) = core.pdf_metadata.as_mut() {
            metadata.resource_usage = self.resource_usage();
        }
        Ok(core)
    }

    pub(crate) fn prepare_hwp_snapshot(
        &self,
        core: &DocumentCore,
    ) -> Result<crate::document_core::commands::HwpExportSnapshot, HwpError> {
        if !self.started || self.failed {
            return Err(HwpError::InvalidFile(
                "PDF_IMPORT_SESSION: invalid or failed snapshot request".into(),
            ));
        }
        core.try_prepare_hwp_export_snapshot(&crate::parser::pdf::allocation::Allocation(
            &self.request,
        ))
        .map_err(|error| self.storage_error("HWP snapshot", error))
    }

    /// Serialize HWP using the remaining request budget and an output byte cap.
    /// Format adaptation and opaque scratch also require host memory/time supervision.
    pub fn export_hwp(
        &mut self,
        core: &mut DocumentCore,
        maximum: usize,
    ) -> Result<Vec<u8>, HwpError> {
        if !self.started || self.failed || maximum < 512 {
            self.failed = true;
            return Err(HwpError::InvalidFile(
                "PDF_IMPORT_SESSION: invalid or failed export request".into(),
            ));
        }
        let output = self.prepare_hwp_snapshot(core).and_then(|snapshot| {
            crate::serializer::cfb_writer::serialize_hwp_with_storage(
                snapshot.document(),
                &crate::parser::pdf::allocation::Allocation(&self.request),
                maximum,
            )
            .map_err(|error| HwpError::RenderError(error.to_string()))
        });
        match output {
            Ok(bytes) => {
                if let Some(metadata) = core.pdf_metadata.as_mut() {
                    metadata.resource_usage = self.resource_usage();
                }
                Ok(bytes)
            }
            Err(error) => {
                self.failed = true;
                Err(error)
            }
        }
    }

    /// Serialize HWPX with the same request and a bounded ZIP output backing.
    /// XML/context construction also requires host memory/time supervision.
    pub fn export_hwpx(
        &mut self,
        core: &mut DocumentCore,
        maximum: usize,
    ) -> Result<Vec<u8>, HwpError> {
        if !self.started || self.failed || maximum == 0 {
            self.failed = true;
            return Err(HwpError::InvalidFile(
                "PDF_IMPORT_SESSION: invalid or failed export request".into(),
            ));
        }
        let output = core.hwpx_document_for_export(|doc| {
            let reservation = crate::parser::pdf::allocation::shared_reservation(&self.request)
                .map_err(|error| {
                    crate::serializer::SerializeError::XmlError(format!("HWPX context: {error}"))
                })?;
            crate::serializer::hwpx::serialize_hwpx_with_storage(
                doc,
                &crate::parser::pdf::allocation::Allocation(&self.request),
                maximum,
                reservation,
            )
        });
        match output {
            Ok(bytes) => {
                if let Some(metadata) = core.pdf_metadata.as_mut() {
                    metadata.resource_usage = self.resource_usage();
                }
                Ok(bytes)
            }
            Err(error) => {
                self.failed = true;
                Err(error)
            }
        }
    }

    /// Encode the worker response before allocating its output backing. The
    /// metadata snapshot describes import/export before this transport encoding;
    /// `resource_usage()` also includes the diagnostic buffer reservation.
    pub fn diagnostics_json(
        &mut self,
        core: &DocumentCore,
        maximum: usize,
    ) -> Result<Vec<u8>, HwpError> {
        if !self.started || self.failed || maximum == 0 {
            self.failed = true;
            return Err(HwpError::InvalidFile(
                "PDF_IMPORT_SESSION: invalid or failed diagnostics request".into(),
            ));
        }
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Response<'a> {
            page_count: u32,
            diagnostics: Option<&'a crate::parser::pdf::PdfImportMetadata>,
        }
        let response = Response {
            page_count: core.page_count(),
            diagnostics: core.pdf_metadata(),
        };
        let output = crate::parser::pdf::allocation::Allocation(&self.request)
            .json_exact(&response, maximum)
            .map_err(|error| self.storage_error("diagnostic encoding", error));
        if output.is_err() {
            self.failed = true;
        }
        output
    }

    pub fn resource_usage(&self) -> ImportBudgetSnapshot {
        self.request.snapshot()
    }

    /// The shared SVG serializer refuses final syntax backing before growth.
    /// Layer construction and renderer scratch still require host supervision.
    pub fn render_page_svg(
        &mut self,
        core: &DocumentCore,
        page: u32,
        maximum: usize,
    ) -> Result<String, HwpError> {
        if !self.started || self.failed || maximum == 0 {
            return Err(HwpError::InvalidFile(
                "PDF_IMPORT_SESSION: invalid SVG request".into(),
            ));
        }
        if page >= core.page_count() {
            return Err(HwpError::InvalidFile(
                "PDF_PAGE_SELECTION: output page is out of range".into(),
            ));
        }
        let output = crate::parser::pdf::allocation::payload_reservation(&self.request)
            .map_err(|error| self.storage_error("SVG backing", error))
            .and_then(|reservation| core.render_page_svg_bounded(page, maximum, reservation));
        if output.is_err() {
            self.failed = true;
        }
        output
    }

    fn storage_error(&self, stage: &str, error: crate::parser::pdf::PdfImportError) -> HwpError {
        let usage = self.resource_usage();
        let details = match &usage.failure {
            Some(failure) => format!(
                "used {:?}, requested {:?}, limit {:?}",
                failure.used, failure.requested, failure.limit,
            ),
            None => format!("decoded bytes {}", usage.decoded_bytes),
        };
        HwpError::InvalidFile(format!(
            "{} (during {stage}; {details})",
            crate::parser::ParseError::PdfError(error),
        ))
    }
}
