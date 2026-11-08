//! Request-scoped limits for the rhwp import path. Not a process-wide heap limit.

use crate::error::{Error, Result};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

/// Shared input ceiling for the frontend and this decoder policy.
pub const DEFAULT_INPUT_BYTES: usize = 64 * 1024 * 1024;

/// Stream bytes retain the originating request's budget through object clones.
#[derive(Clone, Debug, Default)]
pub struct StreamData {
    bytes: bytes::Bytes,
    budget: Option<Arc<ImportBudget>>,
}

impl StreamData {
    /// Create empty, unbound stream bytes for legacy writers.
    pub fn new() -> Self {
        Self::default()
    }
    /// Wrap static bytes without a request context.
    pub fn from_static(bytes: &'static [u8]) -> Self {
        bytes::Bytes::from_static(bytes).into()
    }
    /// Copy bytes for legacy object construction.
    pub fn copy_from_slice(bytes: &[u8]) -> Self {
        bytes::Bytes::copy_from_slice(bytes).into()
    }
    pub(crate) fn bounded(bytes: Vec<u8>, budget: Option<&Arc<ImportBudget>>) -> Self {
        Self {
            bytes: bytes.into(),
            budget: budget.cloned(),
        }
    }
    /// Original request context, shared by cloned stream objects.
    pub fn budget(&self) -> Option<&Arc<ImportBudget>> {
        self.budget.as_ref()
    }
}

impl std::ops::Deref for StreamData {
    type Target = bytes::Bytes;
    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}
impl AsRef<[u8]> for StreamData {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
impl PartialEq for StreamData {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}
impl From<bytes::Bytes> for StreamData {
    fn from(bytes: bytes::Bytes) -> Self {
        Self {
            bytes,
            budget: None,
        }
    }
}
impl From<Vec<u8>> for StreamData {
    fn from(bytes: Vec<u8>) -> Self {
        bytes::Bytes::from(bytes).into()
    }
}
impl From<String> for StreamData {
    fn from(bytes: String) -> Self {
        bytes::Bytes::from(bytes).into()
    }
}

/// Explicit limits for one PDF import; zero never means unlimited.
#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfImportLimits {
    /// Input byte limit checked before the constructor's copy.
    pub input_bytes: usize,
    /// Maximum size of any individual decoded output buffer.
    pub stream_bytes: usize,
    /// Maximum requested bytes in one private codec scratch allocation.
    pub codec_allocation_bytes: usize,
    /// Maximum cumulative work units in opted-in image decoders.
    pub codec_work_units: usize,
    /// Cumulative decoded/copy/predictor bytes, including failed attempts.
    pub decoded_bytes: u64,
    /// Maximum number of filters in one pipeline.
    pub filters: usize,
    /// Maximum lexical nesting and xref-chain depth.
    pub nesting: usize,
    /// Maximum count of parsed object values.
    pub objects: usize,
    /// Maximum cumulative collection and xref entry count.
    pub entries: usize,
    /// Maximum operators across all parses in this request.
    pub operators: usize,
    /// Maximum graphics state stack depth in a stream.
    pub graphics_stack: usize,
    /// Maximum literal or hex string size before materializing its value.
    pub string_bytes: usize,
    /// Maximum text-showing bytes across the document.
    pub text_bytes: usize,
    /// Maximum emitted spans before postprocessing.
    pub spans: usize,
    /// Maximum decoded source bytes for one ToUnicode CMap.
    pub cmap_bytes: usize,
    /// Maximum expanded CMap entries, including overwritten/temporary entries.
    pub cmap_entries: usize,
    /// Maximum conservative byte-scan work across CMap parses.
    pub cmap_scan_bytes: usize,
    /// Maximum cumulative font metric, encoding, Unicode memo and owned
    /// copy-on-write entry work. Resource/cache indices use `entries` instead.
    pub font_entries: usize,
    /// Maximum page-tree edge depth and concurrent indirect-object loads.
    pub reference_depth: usize,
    /// Maximum cumulative page-node visits and object loads, including cache hits.
    pub reference_visits: usize,
    /// Maximum nested Form invocations, with the first form at depth one.
    pub form_depth: usize,
    /// Maximum XObject invocations across the request, including cache hits.
    pub xobject_visits: usize,
}

impl Default for PdfImportLimits {
    fn default() -> Self {
        Self {
            input_bytes: DEFAULT_INPUT_BYTES,
            stream_bytes: 32 * 1024 * 1024,
            codec_allocation_bytes: 32 * 1024 * 1024,
            codec_work_units: 1_000_000_000,
            decoded_bytes: 256 * 1024 * 1024,
            filters: 8,
            nesting: 16,
            objects: 50_000,
            entries: 100_000,
            operators: 250_000,
            graphics_stack: 128,
            string_bytes: 64 * 1024,
            text_bytes: 2 * 1024 * 1024,
            spans: 40_000,
            cmap_bytes: 1024 * 1024,
            cmap_entries: 100_000,
            cmap_scan_bytes: 64 * 1024 * 1024,
            font_entries: 100_000,
            reference_depth: 32,
            reference_visits: 100_000,
            form_depth: 16,
            xobject_visits: 10_000,
        }
    }
}

struct CodecScratch(std::sync::Mutex<Option<Vec<u8>>>);
impl std::fmt::Debug for CodecScratch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodecScratch").finish_non_exhaustive()
    }
}

/// The first failed reservation; successful counters are never incremented by it.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportLimitFailure {
    /// Stable category of the first failed reservation.
    pub reason: &'static str,
    /// Successful charges at that counter before the rejected operation.
    pub used: Option<u64>,
    /// Amount rejected before it was charged or allocated.
    pub requested: Option<u64>,
    /// Applicable counter ceiling; absent for non-counter rejections.
    pub limit: Option<u64>,
}

/// Request-local work observations, not process live heap/RSS measurements.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportBudgetSnapshot {
    /// Cumulative decoded/copy/predictor work successfully charged.
    pub decoded_bytes: u64,
    /// Cumulative private codec storage requested, including released blocks.
    pub codec_allocated_bytes: u64,
    /// Work units consumed by opted-in image decoders, not CPU time.
    pub codec_work_units: usize,
    /// Parsed object values charged by this request.
    pub parsed_objects: usize,
    /// Collection and xref entries charged by this request.
    pub collection_entries: usize,
    /// Lexically parsed operators, including repeated parses.
    pub operators: usize,
    /// Bytes consumed by text-showing operators.
    pub text_bytes: usize,
    /// Emitted text spans before postprocessing.
    pub spans: usize,
    /// Page-tree and indirect-object traversal work, including cache hits.
    pub reference_visits: usize,
    /// XObject invocations, including repeated paint.
    pub xobject_visits: usize,
    /// Actual bounded parser calls, not interpreter invocations.
    pub content_parse_calls: usize,
    /// Parsed page operator cache hits.
    pub page_operator_cache_hits: usize,
    /// Decoded page stream cache hits.
    pub page_stream_cache_hits: usize,
    /// Decoded Form stream cache hits.
    pub form_stream_cache_hits: usize,
    /// First fatal reservation, if any; snapshotting does not clear it.
    pub failure: Option<ImportLimitFailure>,
}

/// An optional owner check is cooperative; it cannot preempt opaque work.
pub trait ImportWorkGuard: std::fmt::Debug + Send + Sync {
    /// Return false to reject subsequent work in the current request.
    fn permits_work(&self) -> bool;
}

/// Shared accounting with a sticky fatal state; not a global allocation tracker.
#[derive(Debug)]
pub struct ImportBudget {
    /// Immutable request policy.
    pub limits: PdfImportLimits,
    work_guard: Option<Arc<dyn ImportWorkGuard>>,
    decoded: AtomicU64,
    decoded_limit: AtomicU64,
    operator_limit: AtomicUsize,
    span_limit: AtomicUsize,
    entry_limit: AtomicUsize,
    codec_allocated: AtomicU64,
    codec_work: AtomicUsize,
    objects: AtomicUsize,
    entries: AtomicUsize,
    operators: AtomicUsize,
    text_bytes: AtomicUsize,
    spans: AtomicUsize,
    cmap_entries: AtomicUsize,
    cmap_scanned: AtomicUsize,
    font_entries: AtomicUsize,
    reference_visits: AtomicUsize,
    reference_limit: AtomicUsize,
    active_references: AtomicUsize,
    xobject_visits: AtomicUsize,
    xobject_limit: AtomicUsize,
    failed: AtomicBool,
    failure_reason: std::sync::Mutex<Option<ImportLimitFailure>>,
    content_parse_calls: AtomicUsize,
    page_operator_cache_hits: AtomicUsize,
    page_stream_cache_hits: AtomicUsize,
    form_stream_cache_hits: AtomicUsize,
    // At most one idle codec block per request; live streams own their blocks.
    codec_scratch: CodecScratch,
}

impl ImportBudget {
    /// Create independent accounting; reject disabled limits.
    pub fn new(limits: PdfImportLimits) -> Result<Self> {
        if [
            limits.input_bytes,
            limits.stream_bytes,
            limits.codec_allocation_bytes,
            limits.codec_work_units,
            limits.filters,
            limits.nesting,
            limits.objects,
            limits.entries,
            limits.operators,
            limits.graphics_stack,
            limits.string_bytes,
            limits.text_bytes,
            limits.spans,
            limits.cmap_bytes,
            limits.cmap_entries,
            limits.cmap_scan_bytes,
            limits.font_entries,
            limits.reference_depth,
            limits.reference_visits,
            limits.form_depth,
            limits.xobject_visits,
        ]
        .contains(&0)
            || limits.decoded_bytes == 0
        {
            return Err(Error::ImportResourceLimit("zero import budget"));
        }
        Ok(Self {
            limits,
            work_guard: None,
            decoded: AtomicU64::new(0),
            decoded_limit: AtomicU64::new(limits.decoded_bytes),
            operator_limit: AtomicUsize::new(limits.operators),
            span_limit: AtomicUsize::new(limits.spans),
            entry_limit: AtomicUsize::new(limits.entries),
            codec_allocated: AtomicU64::new(0),
            codec_work: AtomicUsize::new(0),
            objects: AtomicUsize::new(0),
            entries: AtomicUsize::new(0),
            operators: AtomicUsize::new(0),
            text_bytes: AtomicUsize::new(0),
            spans: AtomicUsize::new(0),
            cmap_entries: AtomicUsize::new(0),
            cmap_scanned: AtomicUsize::new(0),
            font_entries: AtomicUsize::new(0),
            reference_visits: AtomicUsize::new(0),
            reference_limit: AtomicUsize::new(limits.reference_visits),
            active_references: AtomicUsize::new(0),
            xobject_visits: AtomicUsize::new(0),
            xobject_limit: AtomicUsize::new(limits.xobject_visits),
            codec_scratch: CodecScratch(std::sync::Mutex::new(None)),
            failed: AtomicBool::new(false),
            failure_reason: std::sync::Mutex::new(None),
            content_parse_calls: AtomicUsize::new(0),
            page_operator_cache_hits: AtomicUsize::new(0),
            page_stream_cache_hits: AtomicUsize::new(0),
            form_stream_cache_hits: AtomicUsize::new(0),
        })
    }

    /// Narrow work ceilings once the host knows the selected pages. Existing
    /// consumption and the first failure remain intact; this can never raise a
    /// limit. Call before starting extraction workers.
    pub fn restrict_work_limits(&self, decoded: u64, operators: usize) -> Result<()> {
        self.check()?;
        if decoded == 0 || operators == 0 {
            return Err(self.reject("zero work budget"));
        }
        self.decoded_limit.fetch_min(decoded, Ordering::Relaxed);
        self.operator_limit.fetch_min(operators, Ordering::Relaxed);
        self.charge_decoded(0)?;
        self.charge(
            &self.operators,
            0,
            self.operator_limit.load(Ordering::Relaxed),
            "content operators",
        )
    }

    /// Install an owner check while the request is still exclusively owned.
    pub fn with_work_guard(mut self, guard: Arc<dyn ImportWorkGuard>) -> Self {
        self.work_guard = Some(guard);
        self
    }

    /// Narrow output/metadata work without clearing previously consumed values.
    pub fn restrict_structure_limits(&self, spans: usize, entries: usize) -> Result<()> {
        self.check()?;
        if spans == 0 || entries == 0 {
            return Err(self.reject("zero structure budget"));
        }
        self.span_limit.fetch_min(spans, Ordering::Relaxed);
        self.entry_limit.fetch_min(entries, Ordering::Relaxed);
        self.charge(&self.spans, 0, self.span_limit.load(Ordering::Relaxed), "emitted spans")?;
        self.charge(
            &self.entries,
            0,
            self.entry_limit.load(Ordering::Relaxed),
            "collection entries",
        )
    }

    /// Narrow traversal work after page selection without forgiving already
    /// consumed page-tree/object visits. Explicit host limits can never rise.
    pub fn restrict_traversal_limits(&self, references: usize, xobjects: usize) -> Result<()> {
        self.check()?;
        if references == 0 || xobjects == 0 {
            return Err(self.reject("zero traversal budget"));
        }
        self.reference_limit
            .fetch_min(references, Ordering::Relaxed);
        self.xobject_limit.fetch_min(xobjects, Ordering::Relaxed);
        self.charge(
            &self.reference_visits,
            0,
            self.reference_limit.load(Ordering::Relaxed),
            "reference visits",
        )?;
        self.charge(
            &self.xobject_visits,
            0,
            self.xobject_limit.load(Ordering::Relaxed),
            "XObject visits",
        )
    }

    /// Fail once any previous operation exhausted this request.
    pub fn check(&self) -> Result<()> {
        if self.failed.load(Ordering::Acquire) {
            let reason = self
                .failure_reason
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            Err(Error::ImportResourceLimit(
                reason
                    .as_ref()
                    .map_or("request budget exhausted", |failure| failure.reason),
            ))
        } else if self
            .work_guard
            .as_ref()
            .is_some_and(|guard| !guard.permits_work())
        {
            Err(self.reject("request execution boundary"))
        } else {
            Ok(())
        }
    }

    /// Mark a resource failure as fatal for the whole import.
    pub fn reject(&self, reason: &'static str) -> Error {
        self.reject_reservation(ImportLimitFailure {
            reason,
            used: None,
            requested: None,
            limit: None,
        })
    }

    fn reject_reservation(&self, failure: ImportLimitFailure) -> Error {
        let mut first = self
            .failure_reason
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let reason = first.get_or_insert(failure).reason;
        self.failed.store(true, Ordering::Release);
        Error::ImportResourceLimit(reason)
    }

    fn charge(
        &self,
        counter: &AtomicUsize,
        amount: usize,
        cap: usize,
        reason: &'static str,
    ) -> Result<()> {
        self.check()?;
        counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                used.checked_add(amount).filter(|&next| next <= cap)
            })
            .map(|_| ())
            .map_err(|used| {
                self.reject_reservation(ImportLimitFailure {
                    reason,
                    used: Some(used as u64),
                    requested: Some(amount as u64),
                    limit: Some(cap as u64),
                })
            })
    }

    /// Observe this request after an extraction step. Concurrent codec work can
    /// advance individual counters during the snapshot; this is not a heap limit.
    pub fn snapshot(&self) -> ImportBudgetSnapshot {
        let read = |value: &AtomicUsize| value.load(Ordering::Relaxed);
        ImportBudgetSnapshot {
            decoded_bytes: self.decoded.load(Ordering::Relaxed),
            codec_allocated_bytes: self.codec_allocated.load(Ordering::Relaxed),
            codec_work_units: read(&self.codec_work),
            parsed_objects: read(&self.objects),
            collection_entries: read(&self.entries),
            operators: read(&self.operators),
            text_bytes: read(&self.text_bytes),
            spans: read(&self.spans),
            reference_visits: read(&self.reference_visits),
            xobject_visits: read(&self.xobject_visits),
            content_parse_calls: read(&self.content_parse_calls),
            page_operator_cache_hits: read(&self.page_operator_cache_hits),
            page_stream_cache_hits: read(&self.page_stream_cache_hits),
            form_stream_cache_hits: read(&self.form_stream_cache_hits),
            failure: self
                .failure_reason
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
        }
    }

    pub(crate) fn record_content_parse(&self) {
        self.content_parse_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_page_operator_cache_hit(&self) {
        self.page_operator_cache_hits
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_page_stream_cache_hit(&self) {
        self.page_stream_cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn record_form_stream_cache_hit(&self) {
        self.form_stream_cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Successfully charged decoded/copy bytes so far, including failed attempts.
    pub fn decoded_bytes(&self) -> u64 {
        self.decoded.load(Ordering::Relaxed)
    }

    /// Requested private codec bytes charged before allocation, including freed buffers.
    pub fn codec_allocated_bytes(&self) -> u64 {
        self.codec_allocated.load(Ordering::Relaxed)
    }

    pub(crate) fn take_codec_scratch(&self, bytes: usize) -> Result<Option<Vec<u8>>> {
        self.check()?;
        if bytes > self.limits.codec_allocation_bytes {
            return Err(self.reject("codec allocation bytes"));
        }
        let mut slot = self
            .codec_scratch
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if slot
            .as_ref()
            .is_some_and(|buffer| buffer.capacity() >= bytes)
        {
            Ok(slot.take())
        } else {
            Ok(None)
        }
    }
    pub(crate) fn recycle_codec_scratch(&self, buffer: Vec<u8>) {
        let mut slot = self
            .codec_scratch
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if slot
            .as_ref()
            .is_none_or(|old| old.capacity() < buffer.capacity())
        {
            *slot = Some(buffer);
        }
    }

    pub(crate) fn charge_codec_work(&self, units: usize) -> Result<()> {
        self.charge(&self.codec_work, units, self.limits.codec_work_units, "codec work units")
    }

    pub(crate) fn charge_codec_allocation(&self, amount: usize) -> Result<()> {
        self.check()?;
        if amount > self.limits.codec_allocation_bytes {
            return Err(self.reject("codec allocation bytes"));
        }
        self.charge_decoded(amount)?;
        self.codec_allocated
            .fetch_add(amount as u64, Ordering::Relaxed);
        Ok(())
    }

    /// Charge before allocating or extending an output buffer.
    pub fn charge_decoded(&self, amount: usize) -> Result<()> {
        self.check()?;
        let cap = self.decoded_limit.load(Ordering::Relaxed);
        self.decoded
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                used.checked_add(amount as u64).filter(|&next| next <= cap)
            })
            .map(|_| ())
            .map_err(|used| {
                self.reject_reservation(ImportLimitFailure {
                    reason: "document decoded bytes",
                    used: Some(used),
                    requested: Some(amount as u64),
                    limit: Some(cap),
                })
            })
    }

    /// Charge the requested flat and owned payload bytes before a cache copy.
    /// Allocator overhead and process-wide live memory are not measured here.
    pub(crate) fn charge_cache_copy(&self, amount: usize) -> Result<()> {
        self.check()?;
        if amount > self.limits.stream_bytes {
            return Err(self.reject("cache copy bytes"));
        }
        self.charge_decoded(amount)
    }

    pub(crate) fn charge_object(&self) -> Result<()> {
        self.charge(&self.objects, 1, self.limits.objects, "parsed object count")
    }

    pub(crate) fn charge_entries(&self, count: usize) -> Result<()> {
        self.charge(
            &self.entries,
            count,
            self.entry_limit.load(Ordering::Relaxed),
            "collection entries",
        )
    }

    pub(crate) fn charge_operator(&self) -> Result<()> {
        self.charge(
            &self.operators,
            1,
            self.operator_limit.load(Ordering::Relaxed),
            "content operators",
        )
    }
    pub(crate) fn charge_text(&self, amount: usize) -> Result<()> {
        self.charge(&self.text_bytes, amount, self.limits.text_bytes, "shown text bytes")
    }
    pub(crate) fn charge_span(&self) -> Result<()> {
        self.charge(&self.spans, 1, self.span_limit.load(Ordering::Relaxed), "emitted spans")
    }

    pub(crate) fn cmap_source(&self, bytes: usize) -> Result<()> {
        self.check()?;
        if bytes > self.limits.cmap_bytes {
            return Err(self.reject("CMap source bytes"));
        }
        self.charge_cmap_scan(
            bytes
                .checked_mul(4)
                .ok_or_else(|| self.reject("CMap scan bytes"))?,
        )?;
        // Account for bounded source normalization and section/line scratch.
        self.charge_decoded(
            bytes
                .checked_mul(24)
                .ok_or_else(|| self.reject("CMap scratch"))?,
        )
    }

    pub(crate) fn cmap_line(&self, bytes: usize) -> Result<()> {
        self.check()?;
        if bytes > self.limits.string_bytes {
            return Err(self.reject("CMap line bytes"));
        }
        Ok(())
    }

    pub(crate) fn charge_cmap_scan(&self, bytes: usize) -> Result<()> {
        self.charge(&self.cmap_scanned, bytes, self.limits.cmap_scan_bytes, "CMap scan bytes")
    }

    pub(crate) fn charge_cmap(&self, entries: usize, text_bytes: usize) -> Result<()> {
        self.charge(&self.cmap_entries, entries, self.limits.cmap_entries, "CMap entries")?;
        let work = entries
            .checked_mul(128)
            .and_then(|n| n.checked_add(text_bytes.checked_mul(4)?))
            .ok_or_else(|| self.reject("CMap expansion bytes"))?;
        self.charge_decoded(work)
    }

    pub(crate) fn charge_font_entries(&self, entries: usize) -> Result<()> {
        self.charge(&self.font_entries, entries, self.limits.font_entries, "font metric entries")?;
        self.charge_decoded(
            entries
                .checked_mul(128)
                .ok_or_else(|| self.reject("font expansion bytes"))?,
        )
    }

    pub(crate) fn depth(&self, depth: usize) -> Result<()> {
        self.check()?;
        if depth > self.limits.nesting {
            return Err(self.reject("object nesting"));
        }
        Ok(())
    }

    pub(crate) fn page_visit(&self, depth: usize) -> Result<()> {
        self.check()?;
        if depth > self.limits.reference_depth {
            return Err(self.reject("page tree depth"));
        }
        self.charge_reference_visit()
    }

    fn charge_reference_visit(&self) -> Result<()> {
        self.charge(
            &self.reference_visits,
            1,
            self.reference_limit.load(Ordering::Relaxed),
            "reference visits",
        )
    }

    pub(crate) fn enter_reference(&self) -> Result<ReferenceGuard<'_>> {
        self.charge_reference_visit()?;
        self.charge(
            &self.active_references,
            1,
            self.limits.reference_depth,
            "active object loads",
        )?;
        Ok(ReferenceGuard(self))
    }

    pub(crate) fn xobject_visit(&self) -> Result<()> {
        self.charge(
            &self.xobject_visits,
            1,
            self.xobject_limit.load(Ordering::Relaxed),
            "XObject visits",
        )
    }

    pub(crate) fn form_depth(&self, depth: usize) -> Result<()> {
        self.check()?;
        if depth > self.limits.form_depth {
            return Err(self.reject("Form depth"));
        }
        Ok(())
    }

    pub(crate) fn output(&self) -> BoundedOutput<'_> {
        BoundedOutput {
            budget: self,
            bytes: Vec::new(),
        }
    }

    pub(crate) fn copy_bytes(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        let mut output = self.output();
        output.append(bytes)?;
        Ok(output.finish())
    }

    pub(crate) fn append_bytes(&self, bytes: &mut Vec<u8>, more: &[u8]) -> Result<()> {
        let mut output = BoundedOutput {
            budget: self,
            bytes: std::mem::take(bytes),
        };
        output.append(more)?;
        *bytes = output.finish();
        Ok(())
    }

    pub(crate) fn read(&self, mut reader: impl Read) -> Result<Vec<u8>> {
        let mut output = self.output();
        let mut chunk = [0u8; 4096];
        loop {
            self.check()?;
            let count = reader.read(&mut chunk).map_err(Error::Io)?;
            if count == 0 {
                return Ok(output.finish());
            }
            output.append(&chunk[..count])?;
        }
    }
}

pub(crate) struct ReferenceGuard<'a>(&'a ImportBudget);

impl Drop for ReferenceGuard<'_> {
    fn drop(&mut self) {
        self.0.active_references.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(crate) struct BoundedOutput<'a> {
    budget: &'a ImportBudget,
    bytes: Vec<u8>,
}

impl BoundedOutput<'_> {
    pub(crate) fn append(&mut self, bytes: &[u8]) -> Result<()> {
        let next = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .filter(|&next| next <= self.budget.limits.stream_bytes)
            .ok_or_else(|| self.budget.reject("stream output bytes"))?;
        self.budget.charge_decoded(bytes.len())?;
        if next > self.bytes.capacity() {
            let capacity = self
                .bytes
                .capacity()
                .saturating_mul(2)
                .max(4096)
                .max(next)
                .min(self.budget.limits.stream_bytes);
            self.bytes
                .try_reserve_exact(capacity - self.bytes.len())
                .map_err(|_| self.budget.reject("output allocation"))?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    pub(crate) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

impl Write for BoundedOutput<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.append(bytes)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
