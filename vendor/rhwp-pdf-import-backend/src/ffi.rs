//! C Foreign Function Interface (FFI) for pdf_oxide
//!
//! Provides `#[no_mangle] pub extern "C"` functions that Go (CGO), Node.js (N-API),
//! and C# (P/Invoke) bindings can link against. The compiled `libpdf_oxide.so` / `.dylib` / `.dll`
//! exports these symbols when built as a cdylib.
#![allow(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::too_many_arguments)]
//!
//! # Error Convention
//! Most functions accept an `error_code: *mut i32` out-parameter:
//! - 0 = success
//! - 1 = invalid argument / path
//! - 2 = file not found / IO error
//! - 3 = parse error / invalid PDF
//! - 4 = extraction failed
//! - 5 = internal error
//! - 6 = invalid page index
//! - 7 = search error
//! - 8 = unsupported feature
//!
//! # Memory Convention
//! - Strings returned as `*mut c_char` are owned by the caller and must be freed with `free_string`.
//! - Byte buffers returned as `*mut u8` must be freed with `free_bytes`.
//! - Opaque handles (Box pointers) must be freed with their corresponding `*_free` function.
//!
//! # Handle Ownership Contract
//! Every opaque handle (`*mut PdfDocument`, `*mut DocumentEditor`, `*mut Pdf`, etc.) is a
//! `Box<T>` cast to a raw pointer. Functions dereference these pointers via `handle_mut`
//! or `handle_ref`. The following invariants **must** be upheld by all callers:
//!
//! 1. **Exclusive ownership** — at any point in time, at most one logical caller holds a given
//!    handle. Passing the same handle to two threads concurrently (without external
//!    synchronisation) is undefined behaviour.
//! 2. **Validity** — the handle must be non-null and point to a live, fully-initialised `T`
//!    allocated by the corresponding `*_open` / `*_new` function.
//! 3. **No use after free** — callers must not call any function on a handle after the
//!    corresponding `*_free` function returns.
//!
//! All language bindings (Go/CGO, C#/P-Invoke, Node.js/N-API) serialise access to handles
//! at the binding layer and satisfy these invariants. A future release will enforce constraint 1
//! inside the library via `Mutex<T>`; see issue #410.

use crate::converters::ConversionOptions;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

use crate::annotations::Annotation as RustAnnotation;
use crate::api::Pdf;
use crate::document::PdfDocument;
use crate::editor::DocumentEditor;
use crate::editor::EditableDocument;
use crate::search::{SearchOptions, SearchResult as RustSearchResult, TextSearcher};

// ─── Handle dereference helpers ──────────────────────────────────────────────
//
// Use handle_mut() for every pointer deref that calls a &mut self method.
// Use handle_ref() for every pointer deref that calls a &self method.
// This centralises the unsafe block so a future Mutex<T> migration only
// needs to change these two functions — see issue #410.

/// Dereference a raw handle pointer as a shared reference.
///
/// Callers must uphold the Handle Ownership Contract (see module docs):
/// non-null, valid, and no concurrent `&mut` alias exists.
#[allow(unsafe_code)]
#[inline(always)]
fn handle_ref<'a, T>(ptr: *const T) -> &'a T {
    // NonNull::new checks null via safe code; expect panics if null.
    let nn = std::ptr::NonNull::new(ptr as *mut T).expect("FFI: handle pointer must be non-null");
    // SAFETY: non-null proven by NonNull above; Handle Ownership Contract — see module docs.
    unsafe { nn.as_ref() }
}

/// Dereference a raw handle pointer as a unique (mutable) reference.
///
/// Callers must uphold the Handle Ownership Contract (see module docs):
/// non-null, valid, exclusively owned, and no other reference to `*$ptr` exists.
#[allow(unsafe_code)]
#[inline(always)]
fn handle_mut<'a, T>(ptr: *mut T) -> &'a mut T {
    // NonNull::new validates the pointer is non-null before any dereference.
    let mut nn = std::ptr::NonNull::new(ptr).expect("FFI: handle pointer must be non-null");
    // SAFETY: non-null proven by NonNull above; Handle Ownership Contract — see module docs.
    unsafe { nn.as_mut() }
}

// ─── Error helpers ───────────────────────────────────────────────────────────

const ERR_SUCCESS: i32 = 0;
const ERR_INVALID_ARG: i32 = 1;
const ERR_IO: i32 = 2;
const ERR_PARSE: i32 = 3;
const ERR_EXTRACTION: i32 = 4;
const ERR_INTERNAL: i32 = 5;
const ERR_INVALID_PAGE: i32 = 6;
const ERR_SEARCH: i32 = 7;
const _ERR_UNSUPPORTED: i32 = 8;

fn set_error(ptr: *mut i32, code: i32) {
    if !ptr.is_null() {
        unsafe {
            *ptr = code;
        }
    }
}

fn classify_error(e: &crate::error::Error) -> i32 {
    use crate::error::Error;
    match e {
        Error::Io(_) => ERR_IO,
        Error::InvalidHeader(_)
        | Error::UnsupportedVersion(_)
        | Error::ParseError { .. }
        | Error::ParseWarning { .. }
        | Error::InvalidXref
        | Error::ObjectNotFound(_, _)
        | Error::InvalidObjectType { .. }
        | Error::UnexpectedEof
        | Error::InvalidPdf(_)
        | Error::Decode(_) => ERR_PARSE,
        Error::LayoutAnalysis(_) => ERR_EXTRACTION,
        Error::InvalidOperation(_) => ERR_INVALID_ARG,
        Error::Unsupported(_) | Error::UnsupportedFilter(_) | Error::Barcode(_) => _ERR_UNSUPPORTED,
        _ => ERR_INTERNAL,
    }
}

/// Copy `bytes` into a `malloc`-allocated buffer so callers can free it with
/// `free_bytes(ptr)` using no length argument. All FFI byte-buffer returns
/// go through this helper.
fn vec_to_ffi_bytes(bytes: Vec<u8>) -> *mut u8 {
    let len = bytes.len();
    if len == 0 {
        return ptr::null_mut();
    }
    let ptr = unsafe { libc::malloc(len) as *mut u8 };
    if !ptr.is_null() {
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len) };
    }
    ptr
}

fn to_c_string(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(cs) => cs.into_raw(),
        Err(_) => {
            // Fallback: replace NUL bytes
            let cleaned: String = s.replace('\0', "");
            CString::new(cleaned)
                .map(|c| c.into_raw())
                .unwrap_or(ptr::null_mut())
        },
    }
}

fn to_c_string_opt(s: Option<String>) -> *mut c_char {
    match s {
        Some(s) => to_c_string(&s),
        None => ptr::null_mut(),
    }
}

/// Convert a C string pointer to `&str`. Returns `Err(())` if null or invalid UTF-8.
///
/// Callers must ensure `ptr`, when non-null, points to a valid null-terminated C string.
#[allow(unsafe_code)]
fn c_str<'a>(ptr: *const c_char) -> Result<&'a str, ()> {
    if ptr.is_null() {
        return Err(());
    }
    // SAFETY: non-null, caller ensures valid null-terminated string per FFI contract.
    unsafe { CStr::from_ptr(ptr) }.to_str().map_err(|_| ())
}

/// Convert a C string pointer to an owned `String` using lossy UTF-8 conversion.
/// Returns `None` if null.
///
/// Callers must ensure `ptr`, when non-null, points to a valid null-terminated C string.
#[allow(unsafe_code)]
fn c_str_lossy(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: non-null, caller ensures valid null-terminated string per FFI contract.
    Some(
        unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// Convert a C pointer+length pair into a Rust slice.
/// Returns an empty slice if `ptr` is null or `len` is zero.
///
/// Callers must ensure `ptr..ptr+len` is a valid, aligned, initialized memory region.
#[allow(unsafe_code)]
fn raw_slice<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if ptr.is_null() || len == 0 {
        return &[];
    }
    // SAFETY: non-null, len is caller-supplied and correct per FFI contract.
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

/// Write a value through a raw out-parameter pointer. No-op if null.
///
/// Callers must ensure `ptr`, when non-null, is a valid writable pointer.
#[allow(unsafe_code)]
#[inline]
fn write_out<T: Copy>(ptr: *mut T, val: T) {
    if !ptr.is_null() {
        // SAFETY: non-null, caller ensures valid writable pointer per FFI contract.
        unsafe { *ptr = val }
    }
}

// ─── Logging ───────────────────────────────────────────────────────────────

/// Set the global log level for the library.
/// Levels: 0 = Off, 1 = Error, 2 = Warn, 3 = Info, 4 = Debug, 5 = Trace
#[no_mangle]
pub extern "C" fn pdf_oxide_set_log_level(level: i32) {
    let filter = match level {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    log::set_max_level(filter);
}

/// Get the current log level. Returns 0-5 matching the set_log_level values.
#[no_mangle]
pub extern "C" fn pdf_oxide_get_log_level() -> i32 {
    match log::max_level() {
        log::LevelFilter::Off => 0,
        log::LevelFilter::Error => 1,
        log::LevelFilter::Warn => 2,
        log::LevelFilter::Info => 3,
        log::LevelFilter::Debug => 4,
        log::LevelFilter::Trace => 5,
    }
}

/// Set the global content-stream operator cap.
/// `limit < 0` restores the default (1,000,000); any non-negative
/// value (including 0) is used as the explicit cap. Returns the
/// previous cap (or -1 if the default was active).
///
/// Bindings (Java JNI, Ruby FFI, PHP FFI, Go cgo / purego, C# P/Invoke,
/// Node N-API, WASM) call this via the cdylib's exported symbol.
#[no_mangle]
pub extern "C" fn pdf_oxide_set_max_ops_per_stream(limit: i64) -> i64 {
    let new_limit = if limit < 0 {
        None
    } else {
        Some(limit as usize)
    };
    let prev = crate::content::parser::set_max_ops_per_stream(new_limit);
    prev.map(|v| v as i64).unwrap_or(-1)
}

/// Toggle the global U+FFFD preservation flag for
/// the high-level extract_text / extract_words / extract_spans
/// accessors. `1` = preserve FFFD chars; `0` = filter (v0.3.54
/// default). Returns the previous value as `0` or `1`.
#[no_mangle]
pub extern "C" fn pdf_oxide_set_preserve_unmapped_glyphs(preserve: i32) -> i32 {
    let prev = crate::extractors::text::set_preserve_unmapped_glyphs(preserve != 0);
    if prev {
        1
    } else {
        0
    }
}

// ─── Crypto provider ──────────────────────────────────────────

/// Returns the name of the active cryptographic provider as a
/// caller-owned C string. Free with [`free_string`].
///
/// **Non-initializing** — if no provider has been installed yet, this
/// returns `"rust-crypto (default, lazy)"` *without* committing the
/// `OnceLock`, so a subsequent [`pdf_oxide_crypto_use_fips`] call can
/// still succeed. If a provider has already been installed (either
/// explicitly via `pdf_oxide_crypto_use_fips`, or lazily by an earlier
/// crypto operation on this process), the active provider's actual
/// name is returned (e.g. `"rust-crypto"` or `"aws-lc-rs"`).
///
/// Bindings (Python / Node / C# / Go) call this for display/audit
/// before optionally calling `crypto_use_fips`; it is intentionally
/// safe to call first.
#[no_mangle]
pub extern "C" fn pdf_oxide_crypto_active_provider() -> *mut c_char {
    use crate::crypto::CryptoProvider;
    let name = if crate::crypto::is_set() {
        crate::crypto::active().name().to_string()
    } else {
        // Don't initialize the OnceLock — peek at the default
        // provider's compile-time name and keep `crypto_use_fips`
        // available for the rest of the call site.
        format!("{} (default, lazy)", crate::crypto::RustCryptoProvider.name())
    };
    std::ffi::CString::new(name)
        .map(std::ffi::CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

/// Whether the FIPS-validated `aws-lc-rs` provider was compiled
/// into this binary. Returns 1 if available (built with
/// `--features fips`), 0 otherwise.
///
/// Bindings query this to expose the right list to user code (Python
/// `available_providers()`, etc.).
#[no_mangle]
pub extern "C" fn pdf_oxide_crypto_fips_available() -> i32 {
    if cfg!(feature = "fips") {
        1
    } else {
        0
    }
}

/// Install the FIPS-validated `aws-lc-rs` provider as the
/// process-wide active cryptographic backend. Set-once: returns
/// non-zero error code if a provider is already installed (either
/// explicitly or lazily via a prior crypto operation).
///
/// Error codes:
///   0 = success
///   1 = FIPS feature not compiled in
///   2 = provider already set
#[no_mangle]
pub extern "C" fn pdf_oxide_crypto_use_fips() -> i32 {
    #[cfg(feature = "fips")]
    {
        use std::sync::Arc;
        match crate::crypto::set_provider(Arc::new(crate::crypto::AwsLcProvider::new())) {
            Ok(()) => 0,
            Err(_) => 2,
        }
    }
    #[cfg(not(feature = "fips"))]
    {
        1
    }
}

// ─── Crypto governance policy (#230) ───────────────────────────────────────

/// Install the process-wide runtime crypto policy from its grammar
/// string (`"compat"|"strict"|"fips-strict"[;<allow|deny>:<alg>@<use>]`).
///
/// Error codes (fail-closed):
///   0 = success
///   1 = invalid argument (null/non-UTF-8 spec)
///   2 = parse error (spec rejected — policy NOT installed)
///   3 = a policy is already set (set-once)
#[no_mangle]
pub extern "C" fn pdf_oxide_crypto_set_policy(spec: *const c_char) -> i32 {
    let s = match c_str(spec) {
        Ok(s) => s,
        Err(_) => return 1,
    };
    let policy: crate::crypto::SecurityPolicy = match s.parse() {
        Ok(p) => p,
        Err(_) => return 2,
    };
    match crate::crypto::set_policy(policy) {
        Ok(()) => 0,
        Err(_) => 3,
    }
}

/// The active crypto policy as its canonical grammar string. Caller
/// frees via [`free_string`]. Never null on success.
#[no_mangle]
pub extern "C" fn pdf_oxide_crypto_policy() -> *mut c_char {
    to_c_string(&crate::crypto::active_policy().to_string())
}

/// Comma-joined lowercase tokens of the algorithms exercised so far
/// this process (governance inventory). Caller frees via
/// [`free_string`]; `""` when nothing has been exercised.
#[no_mangle]
pub extern "C" fn pdf_oxide_crypto_inventory() -> *mut c_char {
    let joined = crate::crypto::inventory()
        .into_iter()
        .map(|a| a.token())
        .collect::<Vec<_>>()
        .join(",");
    to_c_string(&joined)
}

/// A CycloneDX 1.6 Cryptographic Bill of Materials (JSON) of the
/// algorithms exercised so far this process (#230 Phase F). Caller
/// frees via [`free_string`].
#[no_mangle]
pub extern "C" fn pdf_oxide_crypto_cbom() -> *mut c_char {
    to_c_string(&crate::crypto::cbom_json())
}

// ─── Memory management ──────────────────────────────────────────────────────

/// Free a string returned by any FFI function.
#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

/// Free a byte buffer returned by any FFI function.
///
/// Byte buffers are allocated with the C system allocator (malloc) so that
/// callers can free them with a plain pointer and no length argument.
#[no_mangle]
pub extern "C" fn free_bytes(ptr: *mut u8) {
    if !ptr.is_null() {
        unsafe {
            libc::free(ptr as *mut libc::c_void);
        }
    }
}

// ─── PdfDocument ────────────────────────────────────────────────────────────

/// Open a PDF document from a file path. Returns an opaque handle.
///
/// # Safety
/// `path` must be a valid, null-terminated UTF-8 C string.
/// The returned handle must be freed with [`pdf_document_free`].
#[no_mangle]
pub extern "C" fn pdf_document_open(path: *const c_char, error_code: *mut i32) -> *mut PdfDocument {
    if path.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let path_str = match c_str(path) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    match PdfDocument::open(path_str) {
        Ok(doc) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(doc))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Free a PdfDocument handle.
///
/// # Safety
/// `handle` must have been returned by [`pdf_document_open`] and must not be used after this call.
/// Passing null is safe (no-op).
#[no_mangle]
pub extern "C" fn pdf_document_free(handle: *mut PdfDocument) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Get the page count.
#[no_mangle]
pub extern "C" fn pdf_document_get_page_count(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let doc = handle_ref(handle);
    match doc.page_count() {
        Ok(count) => {
            set_error(error_code, ERR_SUCCESS);
            count as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Get the PDF version as (major, minor).
#[no_mangle]
pub extern "C" fn pdf_document_get_version(
    handle: *const PdfDocument,
    major: *mut u8,
    minor: *mut u8,
) {
    if handle.is_null() || major.is_null() || minor.is_null() {
        return;
    }
    let doc = handle_ref(handle);
    let (maj, min) = doc.version();
    write_out(major, maj);
    write_out(minor, min);
}

/// Check if the document has a structure tree (tagged PDF).
#[no_mangle]
pub extern "C" fn pdf_document_has_structure_tree(handle: *mut PdfDocument) -> bool {
    if handle.is_null() {
        return false;
    }
    let doc = handle_ref(handle);
    doc.structure_tree().ok().flatten().is_some()
}

/// Extract text from a single page.
#[no_mangle]
pub extern "C" fn pdf_document_extract_text(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_text(page_index as usize) {
        Ok(text) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&text)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Extract a page as structured typed regions (issue #536), returned as a JSON
/// string (a serialized `StructuredPage`: `page_index`, `page_width`,
/// `page_height`, and `regions[]` with `kind` / `text` / `bbox` / `spans` /
/// `column_index`). Bindings deserialize the JSON into their native types.
///
/// Returns NULL on error (see `error_code`); the returned string must be freed
/// with `pdf_free_string`.
#[no_mangle]
pub extern "C" fn pdf_document_extract_structured_to_json(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_structured(page_index as usize) {
        Ok(structured) => match serde_json::to_string(&structured) {
            Ok(json) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&json)
            },
            Err(_) => {
                set_error(error_code, ERR_INTERNAL);
                ptr::null_mut()
            },
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Convert a page to Markdown.
#[no_mangle]
pub extern "C" fn pdf_document_to_markdown(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    let opts = ConversionOptions::default();
    match doc.to_markdown(page_index as usize, &opts) {
        Ok(text) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&text)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Convert a page to HTML.
#[no_mangle]
pub extern "C" fn pdf_document_to_html(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    let opts = ConversionOptions::default();
    match doc.to_html(page_index as usize, &opts) {
        Ok(text) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&text)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Convert a page to plain text.
#[no_mangle]
pub extern "C" fn pdf_document_to_plain_text(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    let opts = ConversionOptions::default();
    match doc.to_plain_text(page_index as usize, &opts) {
        Ok(text) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&text)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Convert all pages to Markdown.
#[no_mangle]
pub extern "C" fn pdf_document_to_markdown_all(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    let opts = ConversionOptions::default();
    match doc.to_markdown_all(&opts) {
        Ok(text) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&text)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Convert the entire PDF to DOCX bytes.
///
/// On success `out_len` is set to the number of bytes and a heap-allocated
/// buffer is returned — the caller must free it with [`free_bytes`].
#[no_mangle]
pub extern "C" fn pdf_document_to_docx(
    handle: *mut PdfDocument,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if handle.is_null() || out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.to_docx_bytes() {
        Ok(bytes) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(out_len, bytes.len());
            vec_to_ffi_bytes(bytes)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Convert the entire PDF to PPTX bytes.
///
/// On success `out_len` is set to the number of bytes and a heap-allocated
/// buffer is returned — the caller must free it with [`free_bytes`].
#[no_mangle]
pub extern "C" fn pdf_document_to_pptx(
    handle: *mut PdfDocument,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if handle.is_null() || out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.to_pptx_bytes() {
        Ok(bytes) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(out_len, bytes.len());
            vec_to_ffi_bytes(bytes)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Convert the entire PDF to XLSX bytes.
///
/// On success `out_len` is set to the number of bytes and a heap-allocated
/// buffer is returned — the caller must free it with [`free_bytes`].
#[no_mangle]
pub extern "C" fn pdf_document_to_xlsx(
    handle: *mut PdfDocument,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if handle.is_null() || out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.to_xlsx_bytes() {
        Ok(bytes) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(out_len, bytes.len());
            vec_to_ffi_bytes(bytes)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Open a PDF document from DOCX bytes. Returns an opaque PdfDocument handle.
#[no_mangle]
pub extern "C" fn pdf_document_open_from_docx_bytes(
    data: *const u8,
    len: usize,
    error_code: *mut i32,
) -> *mut PdfDocument {
    if data.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let bytes = raw_slice(data, len);
    match crate::converters::office::OfficeConverter::new().convert_docx_bytes(bytes) {
        Ok(pdf_bytes) => match PdfDocument::from_bytes(pdf_bytes) {
            Ok(doc) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(doc))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Open a PDF document from PPTX bytes. Returns an opaque PdfDocument handle.
#[no_mangle]
pub extern "C" fn pdf_document_open_from_pptx_bytes(
    data: *const u8,
    len: usize,
    error_code: *mut i32,
) -> *mut PdfDocument {
    if data.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let bytes = raw_slice(data, len);
    match crate::converters::office::OfficeConverter::new().convert_pptx_bytes(bytes) {
        Ok(pdf_bytes) => match PdfDocument::from_bytes(pdf_bytes) {
            Ok(doc) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(doc))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Open a PDF document from XLSX bytes. Returns an opaque PdfDocument handle.
#[no_mangle]
pub extern "C" fn pdf_document_open_from_xlsx_bytes(
    data: *const u8,
    len: usize,
    error_code: *mut i32,
) -> *mut PdfDocument {
    if data.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let bytes = raw_slice(data, len);
    match crate::converters::office::OfficeConverter::new().convert_xlsx_bytes(bytes) {
        Ok(pdf_bytes) => match PdfDocument::from_bytes(pdf_bytes) {
            Ok(doc) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(doc))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

// ─── DocumentEditor ─────────────────────────────────────────────────────────

/// Open a PDF for editing. Returns an opaque DocumentEditor handle.
///
/// # Safety
/// `path` must be a valid, null-terminated UTF-8 C string.
/// The returned handle must be freed with [`document_editor_free`].
/// All calls on the same handle must be serialised — see the module-level Handle Ownership Contract.
#[no_mangle]
pub extern "C" fn document_editor_open(
    path: *const c_char,
    error_code: *mut i32,
) -> *mut DocumentEditor {
    if path.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let path_str = match c_str(path) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    match DocumentEditor::open(path_str) {
        Ok(editor) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(editor))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Free a DocumentEditor handle.
///
/// # Safety
/// `handle` must have been returned by [`document_editor_open`] or [`document_editor_open_from_bytes`]
/// and must not be used after this call. Passing null is safe (no-op).
#[no_mangle]
pub extern "C" fn document_editor_free(handle: *mut DocumentEditor) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Check if the editor has modifications.
#[no_mangle]
pub extern "C" fn document_editor_is_modified(handle: *const DocumentEditor) -> bool {
    if handle.is_null() {
        return false;
    }
    let editor = handle_ref(handle);
    editor.is_modified()
}

/// Get the source path of the editor.
#[no_mangle]
pub extern "C" fn document_editor_get_source_path(
    handle: *const DocumentEditor,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let editor = handle_ref(handle);
    set_error(error_code, ERR_SUCCESS);
    to_c_string(editor.source_path())
}

/// Get PDF version from the editor.
#[no_mangle]
pub extern "C" fn document_editor_get_version(
    handle: *const DocumentEditor,
    major: *mut u8,
    minor: *mut u8,
) {
    if handle.is_null() || major.is_null() || minor.is_null() {
        return;
    }
    let editor = handle_ref(handle);
    let (maj, min) = editor.version();
    write_out(major, maj);
    write_out(minor, min);
}

/// Get page count from editor.
#[no_mangle]
pub extern "C" fn document_editor_get_page_count(
    handle: *mut DocumentEditor,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_ref(handle);
    set_error(error_code, ERR_SUCCESS);
    editor.current_page_count() as i32
}

macro_rules! editor_get_string_field {
    ($fn_name:ident, $method:ident) => {
        #[no_mangle]
        pub extern "C" fn $fn_name(
            handle: *const DocumentEditor,
            error_code: *mut i32,
        ) -> *mut c_char {
            if handle.is_null() {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            }
            // We need &mut self for these methods, but we only have *const
            // This is safe because we hold the only reference from Go side
            let editor = handle_mut(handle as *mut DocumentEditor);
            match editor.$method() {
                Ok(Some(val)) => {
                    set_error(error_code, ERR_SUCCESS);
                    to_c_string(&val)
                },
                Ok(None) => {
                    set_error(error_code, ERR_SUCCESS);
                    ptr::null_mut()
                },
                Err(e) => {
                    set_error(error_code, classify_error(&e));
                    ptr::null_mut()
                },
            }
        }
    };
}

macro_rules! editor_set_string_field {
    ($fn_name:ident, $method:ident) => {
        #[no_mangle]
        pub extern "C" fn $fn_name(
            handle: *mut DocumentEditor,
            value: *const c_char,
            error_code: *mut i32,
        ) -> i32 {
            if handle.is_null() || value.is_null() {
                set_error(error_code, ERR_INVALID_ARG);
                return -1;
            }
            let editor = handle_mut(handle);
            let val = match c_str(value) {
                Ok(s) => s,
                Err(_) => {
                    set_error(error_code, ERR_INVALID_ARG);
                    return -1;
                },
            };
            editor.$method(val);
            set_error(error_code, ERR_SUCCESS);
            0
        }
    };
}

editor_get_string_field!(document_editor_get_title, title);
editor_set_string_field!(document_editor_set_title, set_title);
editor_get_string_field!(document_editor_get_author, author);
editor_set_string_field!(document_editor_set_author, set_author);
editor_get_string_field!(document_editor_get_subject, subject);
editor_set_string_field!(document_editor_set_subject, set_subject);

/// Producer — reads from `/Info.Producer`. Returns a C string owned by
/// the caller (must be freed with `free_string`); returns null if no
/// producer is set.
#[no_mangle]
pub extern "C" fn document_editor_get_producer(
    handle: *mut DocumentEditor,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let editor = handle_mut(handle);
    match editor.producer() {
        Ok(Some(s)) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&s)
        },
        Ok(None) => {
            set_error(error_code, ERR_SUCCESS);
            ptr::null_mut()
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_set_producer(
    handle: *mut DocumentEditor,
    value: *const c_char,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || value.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let s = match c_str(value) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    handle_mut(handle).set_producer(s);
    set_error(error_code, ERR_SUCCESS);
    0
}

/// Creation date — reads from `/Info.CreationDate` as a raw PDF
/// date string (e.g. `D:20260421120000Z`).
#[no_mangle]
pub extern "C" fn document_editor_get_creation_date(
    handle: *mut DocumentEditor,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let editor = handle_mut(handle);
    match editor.creation_date() {
        Ok(Some(s)) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&s)
        },
        Ok(None) => {
            set_error(error_code, ERR_SUCCESS);
            ptr::null_mut()
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_set_creation_date(
    handle: *mut DocumentEditor,
    date_str: *const c_char,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || date_str.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let s = match c_str(date_str) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    handle_mut(handle).set_creation_date(s);
    set_error(error_code, ERR_SUCCESS);
    0
}

editor_get_string_field!(document_editor_get_keywords, keywords);
editor_set_string_field!(document_editor_set_keywords, set_keywords);

/// Save the edited document.
#[no_mangle]
pub extern "C" fn document_editor_save(
    handle: *mut DocumentEditor,
    path: *const c_char,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || path.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    let path_str = match c_str(path) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    match editor.save(path_str) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

// ─── DocumentEditor: new functions ─────────────────────────────────────────

/// Open a DocumentEditor from an in-memory byte buffer.
/// Returns an opaque handle, or null on error.
#[no_mangle]
pub extern "C" fn document_editor_open_from_bytes(
    data: *const u8,
    len: usize,
    error_code: *mut i32,
) -> *mut DocumentEditor {
    if data.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let bytes = raw_slice(data, len).to_vec();
    match DocumentEditor::from_bytes(bytes) {
        Ok(editor) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(editor))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Save the editor contents to an in-memory byte buffer.
/// The returned pointer is a boxed slice; free with `free_bytes`.
/// `out_len` receives the number of bytes written.
#[no_mangle]
pub extern "C" fn document_editor_save_to_bytes(
    handle: *mut DocumentEditor,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if handle.is_null() || out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let editor = handle_mut(handle);
    match editor.save_to_bytes() {
        Ok(bytes) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(out_len, bytes.len());
            vec_to_ffi_bytes(bytes)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Save the editor contents to bytes with compression / GC / linearize options.
#[no_mangle]
pub extern "C" fn document_editor_save_to_bytes_with_options(
    handle: *mut DocumentEditor,
    compress: bool,
    garbage_collect: bool,
    linearize: bool,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if handle.is_null() || out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let editor = handle_mut(handle);
    let mut opts = crate::editor::SaveOptions::full_rewrite();
    opts.compress = compress;
    opts.garbage_collect = garbage_collect;
    opts.linearize = linearize;
    match editor.save_to_bytes_with_options(opts) {
        Ok(bytes) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(out_len, bytes.len());
            vec_to_ffi_bytes(bytes)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Extract a subset of pages to a new in-memory PDF. `pages` is a
/// C array of 0-based page indices; `count` is its length.
#[no_mangle]
pub extern "C" fn document_editor_extract_pages_to_bytes(
    handle: *mut DocumentEditor,
    pages: *const i32,
    count: usize,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if handle.is_null() || pages.is_null() || out_len.is_null() || count == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let page_indices: Vec<usize> = unsafe { std::slice::from_raw_parts(pages, count) }
        .iter()
        .map(|&i| i as usize)
        .collect();
    let editor = handle_mut(handle);
    match editor.extract_pages_to_bytes(&page_indices) {
        Ok(bytes) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(out_len, bytes.len());
            vec_to_ffi_bytes(bytes)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Convert the edited document to PDF/A in-place.
/// level: 0=A1b 1=A1a 2=A2b 3=A2a 4=A2u 5=A3b 6=A3a 7=A3u
#[no_mangle]
pub extern "C" fn document_editor_convert_to_pdf_a(
    handle: *mut DocumentEditor,
    level: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    use crate::compliance::convert_to_pdf_a;
    use crate::compliance::types::PdfALevel;
    let pdf_level = match level {
        0 => PdfALevel::A1b,
        1 => PdfALevel::A1a,
        2 => PdfALevel::A2b,
        3 => PdfALevel::A2a,
        4 => PdfALevel::A2u,
        5 => PdfALevel::A3b,
        6 => PdfALevel::A3a,
        7 => PdfALevel::A3u,
        _ => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    let editor = handle_mut(handle);
    match convert_to_pdf_a(editor.source_mut(), pdf_level) {
        Ok(_) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Save the edited document with AES-256 encryption and return bytes.
/// The returned pointer must be freed with `free_bytes`.
#[no_mangle]
pub extern "C" fn document_editor_save_encrypted_to_bytes(
    handle: *mut DocumentEditor,
    user_password: *const c_char,
    owner_password: *const c_char,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if handle.is_null() || user_password.is_null() || out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let user_pwd = match c_str(user_password) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    let owner_pwd = if owner_password.is_null() {
        user_pwd
    } else {
        match c_str(owner_password) {
            Ok(s) => s,
            Err(_) => user_pwd,
        }
    };
    let enc_config = crate::editor::EncryptionConfig::new(user_pwd, owner_pwd)
        .with_algorithm(crate::editor::EncryptionAlgorithm::Aes256);
    let save_opts = crate::editor::SaveOptions::with_encryption(enc_config);
    let editor = handle_mut(handle);
    match editor.save_to_bytes_with_options(save_opts) {
        Ok(bytes) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(out_len, bytes.len());
            vec_to_ffi_bytes(bytes)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Merge pages from an in-memory PDF byte buffer into this document.
#[no_mangle]
pub extern "C" fn document_editor_merge_from_bytes(
    handle: *mut DocumentEditor,
    data: *const u8,
    len: usize,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || data.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    let bytes = raw_slice(data, len);
    match editor.merge_from_bytes(bytes) {
        Ok(n) => {
            set_error(error_code, ERR_SUCCESS);
            n as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Embed a file attachment into the document.
#[no_mangle]
pub extern "C" fn document_editor_embed_file(
    handle: *mut DocumentEditor,
    name: *const std::os::raw::c_char,
    data: *const u8,
    len: usize,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || name.is_null() || data.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    let name_str = match c_str(name) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    let bytes = raw_slice(data, len).to_vec();
    match editor.embed_file(name_str, bytes) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Apply redactions on a single page (burn them in).
#[no_mangle]
pub extern "C" fn document_editor_apply_page_redactions(
    handle: *mut DocumentEditor,
    page: usize,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.apply_page_redactions(page) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Apply all pending redactions across the entire document.
#[no_mangle]
pub extern "C" fn document_editor_apply_all_redactions(
    handle: *mut DocumentEditor,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.apply_all_redactions() {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Queue an explicit destructive redaction rectangle on `page`
/// (#231). Coordinates and colour are page user-space / DeviceRGB.
/// Returns 0 on success, -1 on error. Matches the symbol the Node
/// `editing-manager` already calls.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn pdf_redaction_add(
    handle: *mut DocumentEditor,
    page: usize,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    r: f64,
    g: f64,
    b: f64,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    let rect = [x1 as f32, y1 as f32, x2 as f32, y2 as f32];
    let fill = Some([r as f32, g as f32, b as f32]);
    match editor.add_redaction(page, rect, fill) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Number of queued redaction regions for `page` (annotations +
/// programmatic). Returns the count, or -1 on error.
#[no_mangle]
pub extern "C" fn pdf_redaction_count(
    handle: *mut DocumentEditor,
    page: usize,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.redaction_count(page) {
        Ok(n) => {
            set_error(error_code, ERR_SUCCESS);
            n as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Destructively apply all queued redactions (true content removal +
/// opaque overlay, ISO 32000-1 §12.5.6.23). `scrub_metadata` reserved
/// for the document-scrub pass. Returns the number of glyphs physically
/// removed, or -1 on error (e.g. composite-font content — fail closed).
#[no_mangle]
pub extern "C" fn pdf_redaction_apply(
    handle: *mut DocumentEditor,
    scrub_metadata: bool,
    r: f64,
    g: f64,
    b: f64,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    let opts = crate::redaction::RedactionOptions {
        scrub_metadata,
        default_fill: [r as f32, g as f32, b as f32],
        ..crate::redaction::RedactionOptions::default()
    };
    match editor.apply_redactions_destructive(opts) {
        Ok(report) => {
            set_error(error_code, ERR_SUCCESS);
            report.glyphs_removed.min(i32::MAX as usize) as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Standalone document sanitization without geometric redaction
/// (#231 T10): strips `/Info`, catalog XMP `/Metadata`, document
/// JavaScript (`/OpenAction`, `/AA`, `/Names/JavaScript`)
/// `/Names/EmbeddedFiles`; the removed subtrees are hard-excluded from
/// the output (G6). Returns the number of top-level constructs removed,
/// or -1 on error.
#[no_mangle]
pub extern "C" fn pdf_redaction_scrub_metadata(
    handle: *mut DocumentEditor,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.sanitize_document(crate::redaction::RedactionOptions::default()) {
        Ok(report) => {
            set_error(error_code, ERR_SUCCESS);
            report.annotations_removed.min(i32::MAX as usize) as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Rotate all pages by `degrees` (relative, added to existing rotation).
#[no_mangle]
pub extern "C" fn document_editor_rotate_all_pages(
    handle: *mut DocumentEditor,
    degrees: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.rotate_all_pages(degrees) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Rotate a single page by `degrees` (additive, not absolute).
#[no_mangle]
pub extern "C" fn document_editor_rotate_page_by(
    handle: *mut DocumentEditor,
    page: usize,
    degrees: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.rotate_page_by(page, degrees) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Get the MediaBox for a page. Returns [x, y, width, height] in f64.
#[no_mangle]
pub extern "C" fn document_editor_get_page_media_box(
    handle: *mut DocumentEditor,
    page: usize,
    x: *mut f64,
    y: *mut f64,
    w: *mut f64,
    h: *mut f64,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || x.is_null() || y.is_null() || w.is_null() || h.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.get_page_media_box(page) {
        Ok(b) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(x, b[0] as f64);
            write_out(y, b[1] as f64);
            write_out(w, b[2] as f64);
            write_out(h, b[3] as f64);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Set the MediaBox for a page.
#[no_mangle]
pub extern "C" fn document_editor_set_page_media_box(
    handle: *mut DocumentEditor,
    page: usize,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.set_page_media_box(page, [x as f32, y as f32, w as f32, h as f32]) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Get the CropBox for a page. Writes 0,0,0,0 if no CropBox is set.
#[no_mangle]
pub extern "C" fn document_editor_get_page_crop_box(
    handle: *mut DocumentEditor,
    page: usize,
    x: *mut f64,
    y: *mut f64,
    w: *mut f64,
    h: *mut f64,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || x.is_null() || y.is_null() || w.is_null() || h.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.get_page_crop_box(page) {
        Ok(Some(b)) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(x, b[0] as f64);
            write_out(y, b[1] as f64);
            write_out(w, b[2] as f64);
            write_out(h, b[3] as f64);
            0
        },
        Ok(None) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(x, 0.0f64);
            write_out(y, 0.0f64);
            write_out(w, 0.0f64);
            write_out(h, 0.0f64);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Set the CropBox for a page.
#[no_mangle]
pub extern "C" fn document_editor_set_page_crop_box(
    handle: *mut DocumentEditor,
    page: usize,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.set_page_crop_box(page, [x as f32, y as f32, w as f32, h as f32]) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Erase multiple rectangular regions on a page.
/// `rects` is a flat array of [x, y, w, h] quads (f64).
/// `rects_count` is the number of rectangles (not the array length).
#[no_mangle]
pub extern "C" fn document_editor_erase_regions(
    handle: *mut DocumentEditor,
    page: usize,
    rects: *const f64,
    rects_count: usize,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || rects.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    let flat = raw_slice(rects, rects_count * 4);
    let boxes: Vec<[f32; 4]> = flat
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| [c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32])
        .collect();
    match editor.erase_regions(page, &boxes) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Clear all pending erase-region entries for a page.
#[no_mangle]
pub extern "C" fn document_editor_clear_erase_regions(
    handle: *mut DocumentEditor,
    page: usize,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    editor.clear_erase_regions(page);
    set_error(error_code, ERR_SUCCESS);
    0
}

/// Returns 1 if the page is marked for annotation-flatten, 0 if not, -1 on error.
#[no_mangle]
pub extern "C" fn document_editor_is_page_marked_for_flatten(
    handle: *const DocumentEditor,
    page: usize,
) -> i32 {
    if handle.is_null() {
        return -1;
    }
    let editor = handle_ref(handle);
    if editor.is_page_marked_for_flatten(page) {
        1
    } else {
        0
    }
}

/// Remove the flatten mark from a page.
#[no_mangle]
pub extern "C" fn document_editor_unmark_page_for_flatten(
    handle: *mut DocumentEditor,
    page: usize,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    editor.unmark_page_for_flatten(page);
    set_error(error_code, ERR_SUCCESS);
    0
}

/// Returns 1 if the page is marked for redaction, 0 if not, -1 on error.
#[no_mangle]
pub extern "C" fn document_editor_is_page_marked_for_redaction(
    handle: *const DocumentEditor,
    page: usize,
) -> i32 {
    if handle.is_null() {
        return -1;
    }
    let editor = handle_ref(handle);
    if editor.is_page_marked_for_redaction(page) {
        1
    } else {
        0
    }
}

/// Remove the redaction mark from a page.
#[no_mangle]
pub extern "C" fn document_editor_unmark_page_for_redaction(
    handle: *mut DocumentEditor,
    page: usize,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    editor.unmark_page_for_redaction(page);
    set_error(error_code, ERR_SUCCESS);
    0
}

// ─── PDF Creator (Pdf type) ────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn pdf_from_markdown(markdown: *const c_char, error_code: *mut i32) -> *mut Pdf {
    if markdown.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let md = match c_str(markdown) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    match Pdf::from_markdown(md) {
        Ok(pdf) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(pdf))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_from_html(html: *const c_char, error_code: *mut i32) -> *mut Pdf {
    if html.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let html_str = match c_str(html) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    match Pdf::from_html(html_str) {
        Ok(pdf) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(pdf))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_from_text(text: *const c_char, error_code: *mut i32) -> *mut Pdf {
    if text.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let text_str = match c_str(text) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    match Pdf::from_text(text_str) {
        Ok(pdf) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(pdf))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_save(handle: *mut Pdf, path: *const c_char, error_code: *mut i32) -> i32 {
    if handle.is_null() || path.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let path_str = match c_str(path) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    // Borrow the Pdf mutably — save must NOT consume it, otherwise the
    // subsequent `pdf_free` call in the caller (Go/JS/C#) is a double-free.
    let pdf = handle_mut(handle);
    match pdf.save(path_str) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_save_to_bytes(
    handle: *mut Pdf,
    data_len: *mut i32,
    error_code: *mut i32,
) -> *mut u8 {
    if handle.is_null() || data_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    // Borrow mutably — must NOT consume the handle (would cause double-free
    // when caller runs pdf_free).
    let pdf = handle_mut(handle);
    match pdf.save_to_bytes() {
        Ok(bytes) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(data_len, bytes.len() as i32);
            vec_to_ffi_bytes(bytes)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_get_page_count(handle: *mut Pdf, error_code: *mut i32) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let pdf = handle_mut(handle);
    match pdf.page_count() {
        Ok(count) => {
            set_error(error_code, ERR_SUCCESS);
            count as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_free(handle: *mut Pdf) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ─── Search ─────────────────────────────────────────────────────────────────

/// Opaque search results handle.
pub struct FfiSearchResults {
    results: Vec<RustSearchResult>,
}

#[no_mangle]
pub extern "C" fn pdf_document_search_page(
    handle: *mut PdfDocument,
    page_index: i32,
    search_term: *const c_char,
    case_sensitive: bool,
    error_code: *mut i32,
) -> *mut FfiSearchResults {
    if handle.is_null() || search_term.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    let term = match c_str(search_term) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    let opts = SearchOptions::new()
        .with_case_insensitive(!case_sensitive)
        .with_page_range(page_index as usize, page_index as usize + 1);
    match TextSearcher::search(doc, term, &opts) {
        Ok(results) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiSearchResults { results }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_search_all(
    handle: *mut PdfDocument,
    search_term: *const c_char,
    case_sensitive: bool,
    error_code: *mut i32,
) -> *mut FfiSearchResults {
    if handle.is_null() || search_term.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    let term = match c_str(search_term) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    let opts = SearchOptions::new().with_case_insensitive(!case_sensitive);
    match TextSearcher::search(doc, term, &opts) {
        Ok(results) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiSearchResults { results }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Build the search index for every page up front, instead of the lazy
/// per-page build `pdf_document_search_page`/`pdf_document_search_all`
/// otherwise do on first use. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn pdf_document_prepare_search(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let doc = handle_ref(handle);
    match doc.prepare_search() {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Drop the cached search index, if any, freeing its memory.
/// `pdf_document_search_page`/`pdf_document_search_all` rebuild it lazily
/// on next use.
#[no_mangle]
pub extern "C" fn pdf_document_clear_search_index(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let doc = handle_ref(handle);
    doc.clear_search_index();
    set_error(error_code, ERR_SUCCESS);
    0
}

#[no_mangle]
pub extern "C" fn pdf_oxide_search_result_count(results: *const FfiSearchResults) -> i32 {
    if results.is_null() {
        return 0;
    }
    let r = handle_ref(results);
    r.results.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_search_result_get_text(
    results: *const FfiSearchResults,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if results.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let r = handle_ref(results);
    if index < 0 || (index as usize) >= r.results.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&r.results[index as usize].text)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_search_result_get_page(
    results: *const FfiSearchResults,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if results.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let r = handle_ref(results);
    if index < 0 || (index as usize) >= r.results.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return -1;
    }
    set_error(error_code, ERR_SUCCESS);
    r.results[index as usize].page as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_search_result_get_bbox(
    results: *const FfiSearchResults,
    index: i32,
    x: *mut f32,
    y: *mut f32,
    width: *mut f32,
    height: *mut f32,
    error_code: *mut i32,
) {
    if results.is_null() || x.is_null() || y.is_null() || width.is_null() || height.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let r = handle_ref(results);
    if index < 0 || (index as usize) >= r.results.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    let bbox = &r.results[index as usize].bbox;
    write_out(x, bbox.x);
    write_out(y, bbox.y);
    write_out(width, bbox.width);
    write_out(height, bbox.height);
    set_error(error_code, ERR_SUCCESS);
}

#[no_mangle]
pub extern "C" fn pdf_oxide_search_result_free(handle: *mut FfiSearchResults) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ─── Font extraction ────────────────────────────────────────────────────────

/// Simple FFI-friendly font info
pub struct FfiFont {
    name: String,
    subtype: String,
    encoding: String,
    is_embedded: bool,
    is_subset: bool,
}

pub struct FfiFontList {
    fonts: Vec<FfiFont>,
}

#[no_mangle]
pub extern "C" fn pdf_document_get_embedded_fonts(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut FfiFontList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    // Extract spans to discover font names used on this page
    let fonts = match doc.extract_spans(page_index as usize) {
        Ok(spans) => {
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();
            for span in &spans {
                let name = &span.font_name;
                if !name.is_empty() && seen.insert(name.clone()) {
                    let is_subset = name.len() > 7 && name.as_bytes().get(6) == Some(&b'+');
                    let is_embedded = span.font_name.contains('+') || !span.font_name.is_empty();
                    result.push(FfiFont {
                        name: name.clone(),
                        subtype: String::from("Unknown"),
                        encoding: String::from("Unknown"),
                        is_embedded,
                        is_subset,
                    });
                }
            }
            result
        },
        Err(_) => Vec::new(),
    };
    set_error(error_code, ERR_SUCCESS);
    Box::into_raw(Box::new(FfiFontList { fonts }))
}

#[no_mangle]
pub extern "C" fn pdf_oxide_font_count(fonts: *const FfiFontList) -> i32 {
    if fonts.is_null() {
        return 0;
    }
    handle_ref(fonts).fonts.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_font_get_name(
    fonts: *const FfiFontList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if fonts.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(fonts);
    if (index as usize) >= list.fonts.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.fonts[index as usize].name)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_font_get_type(
    fonts: *const FfiFontList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if fonts.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(fonts);
    if (index as usize) >= list.fonts.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.fonts[index as usize].subtype)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_font_get_encoding(
    fonts: *const FfiFontList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if fonts.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(fonts);
    if (index as usize) >= list.fonts.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.fonts[index as usize].encoding)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_font_is_embedded(
    fonts: *const FfiFontList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if fonts.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(fonts);
    if (index as usize) >= list.fonts.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    if list.fonts[index as usize].is_embedded {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_font_is_subset(
    fonts: *const FfiFontList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if fonts.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(fonts);
    if (index as usize) >= list.fonts.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    if list.fonts[index as usize].is_subset {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_font_get_size(
    _fonts: *const FfiFontList,
    _index: i32,
    error_code: *mut i32,
) -> f32 {
    set_error(error_code, ERR_SUCCESS);
    0.0
}

#[no_mangle]
pub extern "C" fn pdf_oxide_font_list_free(handle: *mut FfiFontList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ─── Image extraction ───────────────────────────────────────────────────────

use crate::extractors::PdfImage;

pub struct FfiImageList {
    images: Vec<PdfImage>,
}

#[no_mangle]
pub extern "C" fn pdf_document_get_embedded_images(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut FfiImageList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_images(page_index as usize) {
        Ok(images) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiImageList { images }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_image_count(images: *const FfiImageList) -> i32 {
    if images.is_null() {
        return 0;
    }
    handle_ref(images).images.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_image_get_width(
    images: *const FfiImageList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if images.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(images);
    if (index as usize) >= list.images.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.images[index as usize].width() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_image_get_height(
    images: *const FfiImageList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if images.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(images);
    if (index as usize) >= list.images.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.images[index as usize].height() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_image_get_format(
    images: *const FfiImageList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if images.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(images);
    if (index as usize) >= list.images.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&format!("{:?}", list.images[index as usize].color_space()))
}

#[no_mangle]
pub extern "C" fn pdf_oxide_image_get_colorspace(
    images: *const FfiImageList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if images.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(images);
    if (index as usize) >= list.images.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&format!("{:?}", list.images[index as usize].color_space()))
}

#[no_mangle]
pub extern "C" fn pdf_oxide_image_get_bits_per_component(
    images: *const FfiImageList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if images.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(images);
    if (index as usize) >= list.images.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.images[index as usize].bits_per_component() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_image_get_data(
    images: *const FfiImageList,
    index: i32,
    data_len: *mut i32,
    error_code: *mut i32,
) -> *mut u8 {
    if images.is_null() || index < 0 || data_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(images);
    if (index as usize) >= list.images.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    let img = &list.images[index as usize];
    let data = match img.data() {
        crate::extractors::ImageData::Jpeg(bytes) => bytes.clone(),
        crate::extractors::ImageData::Raw { pixels, .. } => pixels.clone(),
    };
    write_out(data_len, data.len() as i32);
    set_error(error_code, ERR_SUCCESS);
    vec_to_ffi_bytes(data)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_image_list_free(handle: *mut FfiImageList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ─── Annotations ────────────────────────────────────────────────────────────

pub struct FfiAnnotationList {
    annotations: Vec<RustAnnotation>,
}

#[no_mangle]
pub extern "C" fn pdf_document_get_page_annotations(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut FfiAnnotationList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.get_annotations(page_index as usize) {
        Ok(annotations) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiAnnotationList { annotations }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_count(annotations: *const FfiAnnotationList) -> i32 {
    if annotations.is_null() {
        return 0;
    }
    handle_ref(annotations).annotations.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_get_type(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.annotations[index as usize].annotation_type)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_get_content(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string_opt(list.annotations[index as usize].contents.clone())
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_get_rect(
    annotations: *const FfiAnnotationList,
    index: i32,
    x: *mut f32,
    y: *mut f32,
    width: *mut f32,
    height: *mut f32,
    error_code: *mut i32,
) {
    if annotations.is_null() || x.is_null() || y.is_null() || width.is_null() || height.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let list = handle_ref(annotations);
    if index < 0 || (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    if let Some(rect) = &list.annotations[index as usize].rect {
        write_out(x, rect[0] as f32);
        write_out(y, rect[1] as f32);
        write_out(width, (rect[2] - rect[0]) as f32);
        write_out(height, (rect[3] - rect[1]) as f32);
    } else {
        write_out(x, 0.0f32);
        write_out(y, 0.0f32);
        write_out(width, 0.0f32);
        write_out(height, 0.0f32);
    }
    set_error(error_code, ERR_SUCCESS);
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_list_free(handle: *mut FfiAnnotationList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// Advanced annotation accessors
#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_get_subtype(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&format!("{:?}", list.annotations[index as usize].subtype_enum))
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_is_marked_deleted(
    _annotations: *const FfiAnnotationList,
    _index: i32,
    error_code: *mut i32,
) -> bool {
    set_error(error_code, ERR_SUCCESS);
    false
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_get_creation_date(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> i64 {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    // Return 0 — dates are stored as strings in the annotation, not timestamps
    0
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_get_modification_date(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> i64 {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    0
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_get_author(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string_opt(list.annotations[index as usize].author.clone())
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_get_border_width(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> f32 {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0.0;
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0.0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.annotations[index as usize]
        .border
        .map(|b| b[2] as f32)
        .unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_get_color(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> u32 {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    if let Some(color) = &list.annotations[index as usize].color {
        if color.len() >= 3 {
            let r = (color[0] * 255.0) as u32;
            let g = (color[1] * 255.0) as u32;
            let b = (color[2] * 255.0) as u32;
            (r << 16) | (g << 8) | b
        } else {
            0
        }
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_is_hidden(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> bool {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    list.annotations[index as usize].flags.is_hidden()
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_is_printable(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> bool {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    list.annotations[index as usize].flags.is_printable()
}

#[no_mangle]
pub extern "C" fn pdf_oxide_annotation_is_read_only(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> bool {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    list.annotations[index as usize].flags.is_read_only()
}

#[no_mangle]
pub extern "C" fn pdf_oxide_link_annotation_get_uri(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    if let Some(LinkAction::Uri(uri)) = &list.annotations[index as usize].action {
        to_c_string(uri)
    } else {
        ptr::null_mut()
    }
}

use crate::annotations::LinkAction;

#[no_mangle]
pub extern "C" fn pdf_oxide_text_annotation_get_icon_name(
    _annotations: *const FfiAnnotationList,
    _index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    set_error(error_code, ERR_SUCCESS);
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn pdf_oxide_highlight_annotation_get_quad_points_count(
    annotations: *const FfiAnnotationList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if annotations.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.annotations[index as usize]
        .quad_points
        .as_ref()
        .map(|q| q.len() as i32)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_highlight_annotation_get_quad_point(
    annotations: *const FfiAnnotationList,
    index: i32,
    quad_index: i32,
    x1: *mut f32,
    y1: *mut f32,
    x2: *mut f32,
    y2: *mut f32,
    x3: *mut f32,
    y3: *mut f32,
    x4: *mut f32,
    y4: *mut f32,
    error_code: *mut i32,
) {
    if annotations.is_null() || index < 0 || quad_index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let list = handle_ref(annotations);
    if (index as usize) >= list.annotations.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    if let Some(quads) = &list.annotations[index as usize].quad_points {
        if (quad_index as usize) < quads.len() {
            let q = &quads[quad_index as usize];
            write_out(x1, q[0] as f32);
            write_out(y1, q[1] as f32);
            write_out(x2, q[2] as f32);
            write_out(y2, q[3] as f32);
            write_out(x3, q[4] as f32);
            write_out(y3, q[5] as f32);
            write_out(x4, q[6] as f32);
            write_out(y4, q[7] as f32);
            set_error(error_code, ERR_SUCCESS);
            return;
        }
    }
    set_error(error_code, ERR_INVALID_PAGE);
}

// ─── Page operations ────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn pdf_page_get_width(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> f32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return 0.0;
    }
    let doc = handle_ref(handle);
    match doc.get_page_media_box(page_index as usize) {
        Ok((_, _, w, _)) => {
            set_error(error_code, ERR_SUCCESS);
            w
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            0.0
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_page_get_height(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> f32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return 0.0;
    }
    let doc = handle_ref(handle);
    match doc.get_page_media_box(page_index as usize) {
        Ok((_, _, _, h)) => {
            set_error(error_code, ERR_SUCCESS);
            h
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            0.0
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_page_get_rotation(
    _handle: *mut PdfDocument,
    _page_index: i32,
    error_code: *mut i32,
) -> i32 {
    set_error(error_code, ERR_SUCCESS);
    0 // Rotation is not directly exposed in PdfDocument; would need page dict access
}

macro_rules! page_box_fn {
    ($fn_name:ident) => {
        #[no_mangle]
        pub extern "C" fn $fn_name(
            handle: *mut PdfDocument,
            page_index: i32,
            x: *mut f32,
            y: *mut f32,
            width: *mut f32,
            height: *mut f32,
            error_code: *mut i32,
        ) {
            if handle.is_null() || x.is_null() || y.is_null() || width.is_null() || height.is_null()
            {
                set_error(error_code, ERR_INVALID_ARG);
                return;
            }
            let doc = handle_ref(handle);
            match doc.get_page_media_box(page_index as usize) {
                Ok((bx, by, bw, bh)) => {
                    write_out(x, bx);
                    write_out(y, by);
                    write_out(width, bw);
                    write_out(height, bh);
                    set_error(error_code, ERR_SUCCESS);
                },
                Err(e) => {
                    set_error(error_code, classify_error(&e));
                },
            }
        }
    };
}

page_box_fn!(pdf_page_get_media_box);
page_box_fn!(pdf_page_get_crop_box);
page_box_fn!(pdf_page_get_art_box);
page_box_fn!(pdf_page_get_bleed_box);
page_box_fn!(pdf_page_get_trim_box);

// ─── Page elements ──────────────────────────────────────────────────────────

use crate::layout::TextSpan;

pub struct FfiElementList {
    spans: Vec<TextSpan>,
}

#[no_mangle]
pub extern "C" fn pdf_page_get_elements(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut FfiElementList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_spans(page_index as usize) {
        Ok(spans) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiElementList { spans }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_element_count(elements: *const FfiElementList) -> i32 {
    if elements.is_null() {
        return 0;
    }
    handle_ref(elements).spans.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_element_get_type(
    _elements: *const FfiElementList,
    _index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    set_error(error_code, ERR_SUCCESS);
    to_c_string("text")
}

#[no_mangle]
pub extern "C" fn pdf_oxide_element_get_text(
    elements: *const FfiElementList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if elements.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(elements);
    if (index as usize) >= list.spans.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.spans[index as usize].text)
}

/// The span's Unicode-mapping provenance label (ISO 32000-1 §9.10.2 tier):
/// `"to_unicode"`, `"encoding"`, `"predefined_cmap"`, `"embedded_cmap"`,
/// `"actual_text"`, or `"fallback"` (the text is a fabricated glyph-index echo,
/// not read from the file). Returns null when the font could not be resolved.
/// Free the returned string with `pdf_oxide_free_string`.
#[no_mangle]
pub extern "C" fn pdf_oxide_element_get_provenance(
    elements: *const FfiElementList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if elements.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(elements);
    if (index as usize) >= list.spans.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    match list.spans[index as usize].provenance {
        Some(p) => to_c_string(p.as_str()),
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_element_get_rect(
    elements: *const FfiElementList,
    index: i32,
    x: *mut f32,
    y: *mut f32,
    width: *mut f32,
    height: *mut f32,
    error_code: *mut i32,
) {
    if elements.is_null() || x.is_null() || y.is_null() || width.is_null() || height.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let list = handle_ref(elements);
    if index < 0 || (index as usize) >= list.spans.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    let span = &list.spans[index as usize];
    write_out(x, span.bbox.x);
    write_out(y, span.bbox.y);
    write_out(width, span.bbox.width);
    write_out(height, span.bbox.height);
    set_error(error_code, ERR_SUCCESS);
}

/// Page-space extents of the span: the rect from `pdf_oxide_element_get_rect`
/// with any text-matrix rotation resolved into an axis-aligned page-space
/// hull. Identical to `pdf_oxide_element_get_rect` for upright runs.
#[no_mangle]
pub extern "C" fn pdf_oxide_element_get_page_rect(
    elements: *const FfiElementList,
    index: i32,
    x: *mut f32,
    y: *mut f32,
    width: *mut f32,
    height: *mut f32,
    error_code: *mut i32,
) {
    if elements.is_null() || x.is_null() || y.is_null() || width.is_null() || height.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let list = handle_ref(elements);
    if index < 0 || (index as usize) >= list.spans.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    let b = list.spans[index as usize].page_bbox();
    write_out(x, b.x);
    write_out(y, b.y);
    write_out(width, b.width);
    write_out(height, b.height);
    set_error(error_code, ERR_SUCCESS);
}

#[no_mangle]
pub extern "C" fn pdf_oxide_elements_free(handle: *mut FfiElementList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ─── Barcodes ───────────────────────────────────────────────────────────────

use crate::writer::barcode::{
    BarcodeGenerator, BarcodeOptions, BarcodeType, QrCodeOptions, QrErrorCorrection,
};

/// Opaque handle for generated barcode image (PNG bytes)
pub struct FfiBarcodeImage {
    data: Vec<u8>,
    format: i32, // 0=QR, 1=Code128, etc.
    source_data: String,
}

#[no_mangle]
pub extern "C" fn pdf_generate_qr_code(
    data: *const c_char,
    error_correction: i32,
    size_px: i32,
    error_code: *mut i32,
) -> *mut FfiBarcodeImage {
    if data.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let data_str = match c_str(data) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    let ec = match error_correction {
        0 => QrErrorCorrection::Low,
        1 => QrErrorCorrection::Medium,
        2 => QrErrorCorrection::Quartile,
        3 => QrErrorCorrection::High,
        _ => QrErrorCorrection::Medium,
    };
    let opts = QrCodeOptions::new()
        .size(size_px.max(1) as u32)
        .error_correction(ec);
    match BarcodeGenerator::generate_qr(data_str, &opts) {
        Ok(png_bytes) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiBarcodeImage {
                data: png_bytes,
                format: 100, // 100 = QR; avoids collision with 1D barcode format 0 (Code128)
                source_data: data_str.to_string(),
            }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_generate_barcode(
    data: *const c_char,
    format: i32,
    size_px: i32,
    error_code: *mut i32,
) -> *mut FfiBarcodeImage {
    if data.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let data_str = match c_str(data) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    let barcode_type = match format {
        0 => BarcodeType::Code128,
        1 => BarcodeType::Code39,
        2 => BarcodeType::Ean13,
        3 => BarcodeType::Ean8,
        4 => BarcodeType::UpcA,
        5 => BarcodeType::Itf,
        _ => BarcodeType::Code128,
    };
    let opts = BarcodeOptions::new()
        .width(size_px.max(1) as u32)
        .height((size_px.max(1) / 3).max(30) as u32);
    match BarcodeGenerator::generate_1d(barcode_type, data_str, &opts) {
        Ok(png_bytes) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiBarcodeImage {
                data: png_bytes,
                format,
                source_data: data_str.to_string(),
            }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_barcode_get_image_png(
    barcode_handle: *const FfiBarcodeImage,
    _size_px: i32,
    out_len: *mut i32,
    error_code: *mut i32,
) -> *mut u8 {
    if barcode_handle.is_null() || out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        write_out(out_len, 0i32);
        return ptr::null_mut();
    }
    let bc = handle_ref(barcode_handle);
    let data = bc.data.clone();
    let len = data.len() as i32;
    write_out(out_len, len);
    set_error(error_code, ERR_SUCCESS);
    vec_to_ffi_bytes(data)
}

#[no_mangle]
pub extern "C" fn pdf_barcode_get_svg(
    barcode_handle: *const FfiBarcodeImage,
    _size_px: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if barcode_handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    #[cfg(feature = "barcodes")]
    {
        let bc = handle_ref(barcode_handle);
        let result = if bc.format == 100 {
            BarcodeGenerator::generate_qr_svg(&bc.source_data, &QrCodeOptions::default())
        } else {
            let barcode_type = match bc.format {
                0 => BarcodeType::Code128,
                1 => BarcodeType::Code39,
                2 => BarcodeType::Ean13,
                3 => BarcodeType::Ean8,
                4 => BarcodeType::UpcA,
                5 => BarcodeType::Itf,
                6 => BarcodeType::Code93,
                7 => BarcodeType::Codabar,
                _ => BarcodeType::Code128,
            };
            BarcodeGenerator::generate_1d_svg(
                barcode_type,
                &bc.source_data,
                &BarcodeOptions::default(),
            )
        };
        match result {
            Ok(svg) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&svg)
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "barcodes"))]
    {
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_add_barcode_to_page(
    document_handle: *mut DocumentEditor,
    page_index: i32,
    barcode_handle: *const FfiBarcodeImage,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    error_code: *mut i32,
) -> i32 {
    if document_handle.is_null() || barcode_handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    if page_index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(document_handle);
    let barcode = handle_ref(barcode_handle);
    match editor.add_image_bytes_to_page(page_index as usize, &barcode.data, x, y, width, height) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(_) => {
            set_error(error_code, ERR_INTERNAL);
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_barcode_get_format(
    barcode_handle: *const FfiBarcodeImage,
    error_code: *mut i32,
) -> i32 {
    if barcode_handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    set_error(error_code, ERR_SUCCESS);
    handle_ref(barcode_handle).format
}

#[no_mangle]
pub extern "C" fn pdf_barcode_get_data(
    barcode_handle: *const FfiBarcodeImage,
    error_code: *mut i32,
) -> *mut c_char {
    if barcode_handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&handle_ref(barcode_handle).source_data)
}

#[no_mangle]
pub extern "C" fn pdf_barcode_get_confidence(
    _barcode_handle: *const FfiBarcodeImage,
    error_code: *mut i32,
) -> f32 {
    set_error(error_code, ERR_SUCCESS);
    1.0 // generated barcodes have perfect confidence
}

#[no_mangle]
pub extern "C" fn pdf_barcode_free(handle: *mut FfiBarcodeImage) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ─── Signatures ─────────────────────────────────────────────────────────────

/// Opaque handle for SignatureInfo extracted from a PDF
#[cfg(feature = "signatures")]
pub struct FfiSignatureInfo {
    info: crate::signatures::SignatureInfo,
}

#[cfg(not(feature = "signatures"))]
pub struct FfiSignatureInfo {
    _dummy: (),
}

#[no_mangle]
pub extern "C" fn pdf_certificate_load_from_bytes(
    cert_bytes: *const u8,
    cert_len: i32,
    password: *const c_char,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "signatures")]
    {
        if cert_bytes.is_null() || cert_len <= 0 {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let data = raw_slice(cert_bytes, cert_len as usize);
        let pwd = if password.is_null() {
            ""
        } else {
            c_str(password).unwrap_or("")
        };
        // Try PKCS#12 first (has a private key + cert chain). If that
        // fails — today `from_pkcs12` is still stubbed — fall back to
        // raw DER parsing so Certificate accessors work even without
        // PKCS#12 support in Rust core.
        if let Ok(creds) = crate::signatures::SigningCredentials::from_pkcs12(data, pwd) {
            set_error(error_code, ERR_SUCCESS);
            return Box::into_raw(Box::new(creds)) as *mut std::ffi::c_void;
        }
        match crate::signatures::SigningCredentials::from_der(data.to_vec()) {
            Ok(creds) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(creds)) as *mut std::ffi::c_void
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (cert_bytes, cert_len, password);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Load signing credentials (certificate + private key) from PEM-encoded strings.
/// `cert_pem` and `key_pem` are NUL-terminated UTF-8 strings.
/// Returns an opaque handle on success, NULL on failure.
#[no_mangle]
pub unsafe extern "C" fn pdf_certificate_load_from_pem(
    cert_pem: *const c_char,
    key_pem: *const c_char,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "signatures")]
    {
        let Some(cert_str) = read_cstr_or_fail(cert_pem, error_code) else {
            return ptr::null_mut();
        };
        let Some(key_str) = read_cstr_or_fail(key_pem, error_code) else {
            return ptr::null_mut();
        };
        match crate::signatures::SigningCredentials::from_pem(&cert_str, &key_str) {
            Ok(creds) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(creds)) as *mut std::ffi::c_void
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (cert_pem, key_pem);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Sign the document loaded at `document_handle` in-place. The signature is
/// appended as a PDF incremental update so that subsequent saves (via
/// `pdf_save` / `pdf_save_to_bytes`) emit the signed file.
///
/// `certificate_handle` must be a `SigningCredentials` pointer obtained from
/// one of the `pdf_certificate_load_*` functions.
///
/// Returns 0 on success, -1 on failure (check `*error_code`).
#[no_mangle]
pub extern "C" fn pdf_document_sign(
    document_handle: *mut std::ffi::c_void,
    certificate_handle: *const std::ffi::c_void,
    reason: *const c_char,
    location: *const c_char,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "signatures")]
    {
        use crate::signatures::{sign_pdf_bytes, SignOptions};
        if document_handle.is_null() || certificate_handle.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let doc = handle_mut(document_handle as *mut PdfDocument);
        let creds = handle_ref(certificate_handle as *const crate::signatures::SigningCredentials);
        if doc.source_bytes.is_empty() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let reason_str = c_str_lossy(reason);
        let location_str = c_str_lossy(location);
        let opts = SignOptions {
            reason: reason_str,
            location: location_str,
            ..Default::default()
        };
        match sign_pdf_bytes(&doc.source_bytes.clone(), creds, opts) {
            Ok(signed) => {
                doc.source_bytes = signed;
                set_error(error_code, ERR_SUCCESS);
                0
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                -1
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (document_handle, certificate_handle, reason, location);
        set_error(error_code, _ERR_UNSUPPORTED);
        -1
    }
}

/// Sign raw PDF bytes and return the signed PDF as a newly allocated buffer.
///
/// The caller is responsible for freeing the returned buffer via `free_bytes`.
/// `*out_len` is set to the number of bytes in the returned buffer.
///
/// Returns `NULL` on failure (check `*error_code`).
#[no_mangle]
pub unsafe extern "C" fn pdf_sign_bytes(
    pdf_data: *const u8,
    pdf_len: usize,
    certificate_handle: *const std::ffi::c_void,
    reason: *const c_char,
    location: *const c_char,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    #[cfg(feature = "signatures")]
    {
        use crate::signatures::{sign_pdf_bytes, SignOptions};
        if pdf_data.is_null() || certificate_handle.is_null() || out_len.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let data = raw_slice(pdf_data, pdf_len);
        let creds = handle_ref(certificate_handle as *const crate::signatures::SigningCredentials);
        let reason_str = c_str_lossy(reason);
        let location_str = c_str_lossy(location);
        let opts = SignOptions {
            reason: reason_str,
            location: location_str,
            ..Default::default()
        };
        match sign_pdf_bytes(data, creds, opts) {
            Ok(signed) => {
                set_error(error_code, ERR_SUCCESS);
                write_out(out_len, signed.len());
                vec_to_ffi_bytes(signed)
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (pdf_data, pdf_len, certificate_handle, reason, location, out_len);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_get_signature_count(
    document_handle: *const std::ffi::c_void,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "signatures")]
    {
        if document_handle.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let doc = handle_mut(document_handle as *mut PdfDocument);
        match crate::signatures::count_signatures(doc) {
            Ok(n) => {
                set_error(error_code, ERR_SUCCESS);
                n as i32
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                -1
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = document_handle;
        set_error(error_code, _ERR_UNSUPPORTED);
        -1
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_get_signature(
    document_handle: *const std::ffi::c_void,
    index: i32,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "signatures")]
    {
        if document_handle.is_null() || index < 0 {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let doc = handle_mut(document_handle as *mut PdfDocument);
        match crate::signatures::enumerate_signatures(doc) {
            Ok(list) => match list.into_iter().nth(index as usize) {
                Some(info) => {
                    set_error(error_code, ERR_SUCCESS);
                    Box::into_raw(Box::new(FfiSignatureInfo { info })) as *mut std::ffi::c_void
                },
                None => {
                    set_error(error_code, ERR_INVALID_ARG);
                    ptr::null_mut()
                },
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (document_handle, index);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Run the signer-attributes crypto check (RSA-PKCS#1 v1.5 over
/// `signed_attrs`) on the CMS blob carried by a signature handle.
///
/// Returns:
/// - `1` — Valid: signer held the private key matching the embedded
///           certificate. Callers still need to verify the
///           `messageDigest` attribute against their document content
///           hash for a full detached-signature claim — use
///           `pdf_signature_verify_detached` which runs both checks.
/// - `0` — Invalid: CMS parsed but the RSA check failed (tampered
///           attributes or wrong key).
/// - `-1` — Unknown or not supported: PSS / ECDSA / unrecognised
///           digest OID / missing signed_attrs / structurally
///           unparseable / feature not compiled.
#[no_mangle]
pub extern "C" fn pdf_signature_verify(
    signature_handle: *const std::ffi::c_void,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "signatures")]
    {
        if signature_handle.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let ffi = handle_ref(signature_handle as *const FfiSignatureInfo);
        let Some(contents) = ffi.info.contents() else {
            set_error(error_code, _ERR_UNSUPPORTED);
            return -1;
        };
        match crate::signatures::verify_signer(contents) {
            Ok(crate::signatures::SignerVerify::Valid) => {
                set_error(error_code, ERR_SUCCESS);
                1
            },
            Ok(crate::signatures::SignerVerify::Invalid) => {
                set_error(error_code, ERR_SUCCESS);
                0
            },
            Ok(crate::signatures::SignerVerify::Unknown) => {
                set_error(error_code, _ERR_UNSUPPORTED);
                -1
            },
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                -1
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = signature_handle;
        set_error(error_code, _ERR_UNSUPPORTED);
        -1
    }
}

/// Verify a PDF signature end-to-end: run the signer-attributes crypto
/// check and the RFC 5652 §11.2 `messageDigest` attribute check against
/// the caller-provided document bytes. `pdf_data` must be the full PDF
/// file — the signature handle's stored `/ByteRange` is used to extract
/// the segments that were actually signed.
///
/// Returns:
/// - `1` — Valid: both the RSA-PKCS#1 v1.5 check and the messageDigest
///           check passed. The signer is authentic and the document has
///           not been tampered with since signing.
/// - `0` — Invalid: either the signer check or the messageDigest check
///           failed. Callers can't distinguish "wrong signer" from
///           "document tampered after signing" from this code alone.
/// - `-1` — Unknown or not supported: signer uses PSS / ECDSA / unknown
///           digest, blob lacks `signed_attrs` / `messageDigest`,
///           `/ByteRange` is malformed, or the feature is not compiled.
#[no_mangle]
pub extern "C" fn pdf_signature_verify_detached(
    signature_handle: *const std::ffi::c_void,
    pdf_data: *const u8,
    pdf_len: usize,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "signatures")]
    {
        if signature_handle.is_null() || pdf_data.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let ffi = handle_ref(signature_handle as *const FfiSignatureInfo);
        let Some(contents) = ffi.info.contents() else {
            set_error(error_code, _ERR_UNSUPPORTED);
            return -1;
        };
        let br = ffi.info.byte_range();
        if br.len() != 4 {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let byte_range: [i64; 4] = [br[0], br[1], br[2], br[3]];
        let pdf_slice = raw_slice(pdf_data, pdf_len);
        let signed_bytes = match crate::signatures::ByteRangeCalculator::extract_signed_bytes(
            pdf_slice,
            &byte_range,
        ) {
            Ok(b) => b,
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return -1;
            },
        };
        match crate::signatures::verify_signer_detached(contents, &signed_bytes) {
            Ok(crate::signatures::SignerVerify::Valid) => {
                set_error(error_code, ERR_SUCCESS);
                1
            },
            Ok(crate::signatures::SignerVerify::Invalid) => {
                set_error(error_code, ERR_SUCCESS);
                0
            },
            Ok(crate::signatures::SignerVerify::Unknown) => {
                set_error(error_code, _ERR_UNSUPPORTED);
                -1
            },
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                -1
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (signature_handle, pdf_data, pdf_len);
        set_error(error_code, _ERR_UNSUPPORTED);
        -1
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_verify_all_signatures(
    _document_handle: *const std::ffi::c_void,
    error_code: *mut i32,
) -> i32 {
    set_error(error_code, _ERR_UNSUPPORTED);
    -1
}

#[no_mangle]
pub extern "C" fn pdf_signature_get_signer_name(
    sig: *const FfiSignatureInfo,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "signatures")]
    {
        if sig.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let info = handle_ref(sig);
        set_error(error_code, ERR_SUCCESS);
        to_c_string_opt(info.info.signer_name.clone())
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = sig;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_signature_get_signing_time(
    sig: *const FfiSignatureInfo,
    error_code: *mut i32,
) -> i64 {
    #[cfg(feature = "signatures")]
    {
        if sig.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return 0;
        }
        let info = handle_ref(sig);
        set_error(error_code, ERR_SUCCESS);
        info.info
            .signing_time
            .as_deref()
            .and_then(crate::signatures::parse_pdf_date_to_epoch)
            .unwrap_or(0)
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = sig;
        set_error(error_code, _ERR_UNSUPPORTED);
        0
    }
}

#[no_mangle]
pub extern "C" fn pdf_signature_get_signing_reason(
    sig: *const FfiSignatureInfo,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "signatures")]
    {
        if sig.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let info = handle_ref(sig);
        set_error(error_code, ERR_SUCCESS);
        to_c_string_opt(info.info.reason.clone())
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = sig;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_signature_get_signing_location(
    sig: *const FfiSignatureInfo,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "signatures")]
    {
        if sig.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let info = handle_ref(sig);
        set_error(error_code, ERR_SUCCESS);
        to_c_string_opt(info.info.location.clone())
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = sig;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_signature_get_certificate(
    sig: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "signatures")]
    {
        if sig.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let info = handle_ref(sig as *const FfiSignatureInfo);
        let contents = match info.info.contents.as_ref() {
            Some(c) => c,
            None => {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            },
        };
        let cert_der = match crate::signatures::extract_signer_certificate_der(contents) {
            Ok(d) => d,
            Err(e) => {
                set_error(error_code, classify_error(&e));
                return ptr::null_mut();
            },
        };
        match crate::signatures::SigningCredentials::from_der(cert_der) {
            Ok(creds) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(creds)) as *mut std::ffi::c_void
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = sig;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_certificate_get_subject(
    cert: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "signatures")]
    {
        if cert.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let creds = handle_ref(cert as *const crate::signatures::SigningCredentials);
        match creds.subject() {
            Ok(s) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&s)
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = cert;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_certificate_get_issuer(
    cert: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "signatures")]
    {
        if cert.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let creds = handle_ref(cert as *const crate::signatures::SigningCredentials);
        match creds.issuer() {
            Ok(s) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&s)
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = cert;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_certificate_get_serial(
    cert: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "signatures")]
    {
        if cert.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let creds = handle_ref(cert as *const crate::signatures::SigningCredentials);
        match creds.serial() {
            Ok(s) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&s)
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = cert;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_certificate_get_validity(
    cert: *const std::ffi::c_void,
    not_before: *mut i64,
    not_after: *mut i64,
    error_code: *mut i32,
) {
    #[cfg(feature = "signatures")]
    {
        if cert.is_null() || not_before.is_null() || not_after.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return;
        }
        let creds = handle_ref(cert as *const crate::signatures::SigningCredentials);
        match creds.validity() {
            Ok((nb, na)) => {
                write_out(not_before, nb);
                write_out(not_after, na);
                set_error(error_code, ERR_SUCCESS);
            },
            Err(e) => set_error(error_code, classify_error(&e)),
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (cert, not_before, not_after);
        set_error(error_code, _ERR_UNSUPPORTED);
    }
}

#[no_mangle]
pub extern "C" fn pdf_certificate_is_valid(
    cert: *const std::ffi::c_void,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "signatures")]
    {
        if cert.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return 0;
        }
        let creds = handle_ref(cert as *const crate::signatures::SigningCredentials);
        match creds.is_valid() {
            Ok(v) => {
                set_error(error_code, ERR_SUCCESS);
                if v {
                    1
                } else {
                    0
                }
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                0
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = cert;
        set_error(error_code, _ERR_UNSUPPORTED);
        0
    }
}

#[no_mangle]
pub extern "C" fn pdf_signature_free(handle: *mut FfiSignatureInfo) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ─── PAdES LTV (#235) ───────────────────────────────────────────────────────
//
// FROZEN ENUM CONTRACT — `PadesLevel` integer mapping is part of the
// stable C ABI and every binding depends on it (#235 plan §7.1).
// Mirrors the discipline of `pdf_timestamp_get_hash_algorithm`:
//
//     0 = B-B 1 = B-T 2 = B-LT 3 = B-LTA
//
// NEVER renumber. Adding a level later must append a new code only.
// The single source of truth is `signatures::PadesLevel::code` /
// `from_code` (asserted frozen in `pades/mod.rs` tests).

/// Opaque handle for a parsed Document Security Store (`/DSS`), freed
/// with [`pdf_dss_free`].
#[cfg(feature = "signatures")]
pub struct FfiDss {
    dss: crate::signatures::DocumentSecurityStore,
}
#[cfg(not(feature = "signatures"))]
pub struct FfiDss {
    _private: [u8; 0],
}

/// Gather a parallel-array of DER blobs (`ptrs[i]` of length `lens[i]`)
/// into owned `Vec<Vec<u8>>`. Null inner pointers are skipped.
///
/// Callers must ensure `ptrs`/`lens`, when non-null, point to `n` valid
/// entries (mirrors the safe-wrapper convention of [`raw_slice`]).
#[cfg(feature = "signatures")]
#[allow(unsafe_code)]
fn gather_der_blobs(ptrs: *const *const u8, lens: *const usize, n: usize) -> Vec<Vec<u8>> {
    if ptrs.is_null() || lens.is_null() || n == 0 {
        return Vec::new();
    }
    // SAFETY: non-null, caller guarantees `n` valid entries per FFI contract.
    let p = unsafe { std::slice::from_raw_parts(ptrs, n) };
    let l = unsafe { std::slice::from_raw_parts(lens, n) };
    p.iter()
        .zip(l)
        .filter(|(pp, _)| !pp.is_null())
        .map(|(pp, &ll)| raw_slice(*pp, ll).to_vec())
        .collect()
}

/// Sign raw PDF bytes at a PAdES baseline level. `level`: 0=B-B 1=B-T
/// 2=B-LT (3=B-LTA ⇒ `ERR_UNSUPPORTED`). `tsa_url` must be non-null for
/// `level >= 1` (the RFC 3161 timestamp source); requires the
/// `tsa-client` feature, else `ERR_UNSUPPORTED`. The three parallel
/// arrays carry the B-LT revocation material (DER certs/CRLs/OCSPs).
///
/// Returns a `malloc`'d buffer (free with [`free_bytes`]) / `NULL` on
/// failure (check `*error_code`).
#[no_mangle]
pub unsafe extern "C" fn pdf_sign_bytes_pades(
    pdf_data: *const u8,
    pdf_len: usize,
    certificate_handle: *const std::ffi::c_void,
    level: i32,
    tsa_url: *const c_char,
    reason: *const c_char,
    location: *const c_char,
    certs: *const *const u8,
    cert_lens: *const usize,
    n_certs: usize,
    crls: *const *const u8,
    crl_lens: *const usize,
    n_crls: usize,
    ocsps: *const *const u8,
    ocsp_lens: *const usize,
    n_ocsps: usize,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    #[cfg(feature = "signatures")]
    {
        use crate::signatures::{sign_pdf_bytes_pades, RevocationMaterial, SignOptions};
        if pdf_data.is_null() || certificate_handle.is_null() || out_len.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let Some(level_enum) = crate::signatures::PadesLevel::from_code(level) else {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        };
        let data = raw_slice(pdf_data, pdf_len);
        let creds = handle_ref(certificate_handle as *const crate::signatures::SigningCredentials);
        let opts = SignOptions {
            reason: c_str_lossy(reason),
            location: c_str_lossy(location),
            ..Default::default()
        };
        let material = RevocationMaterial {
            certificates: gather_der_blobs(certs, cert_lens, n_certs),
            crls: gather_der_blobs(crls, crl_lens, n_crls),
            ocsp_responses: gather_der_blobs(ocsps, ocsp_lens, n_ocsps),
            ..Default::default()
        };

        // Build a TSA-fetch timestamper from `tsa_url` (B-T/B-LT). The
        // core fail-closes with `Unsupported` if a timestamper is
        // required but absent, so a missing URL / missing `tsa-client`
        // feature surfaces as a clear error rather than a silent B-B.
        #[cfg(feature = "tsa-client")]
        let ts_closure: Option<Box<dyn Fn(&[u8]) -> crate::error::Result<Vec<u8>>>> =
            c_str_lossy(tsa_url).map(|u| {
                let client =
                    crate::signatures::TsaClient::new(crate::signatures::TsaClientConfig::new(u));
                Box::new(move |sig: &[u8]| {
                    client
                        .request_timestamp(sig)
                        .map(|t| t.token_bytes().to_vec())
                }) as Box<dyn Fn(&[u8]) -> crate::error::Result<Vec<u8>>>
            });
        #[cfg(not(feature = "tsa-client"))]
        let ts_closure: Option<Box<dyn Fn(&[u8]) -> crate::error::Result<Vec<u8>>>> = {
            let _ = tsa_url;
            None
        };

        match sign_pdf_bytes_pades(data, creds, opts, level_enum, ts_closure.as_deref(), &material)
        {
            Ok(signed) => {
                set_error(error_code, ERR_SUCCESS);
                write_out(out_len, signed.len());
                vec_to_ffi_bytes(signed)
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (
            pdf_data,
            pdf_len,
            certificate_handle,
            level,
            tsa_url,
            reason,
            location,
            certs,
            cert_lens,
            n_certs,
            crls,
            crl_lens,
            n_crls,
            ocsps,
            ocsp_lens,
            n_ocsps,
            out_len,
        );
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Options for [`pdf_sign_bytes_pades_opts`] — the same parameters as
/// [`pdf_sign_bytes_pades`], collapsed into one `#[repr(C)]` struct.
///
/// `pdf_sign_bytes_pades` takes 18 scalar arguments, which exceeds the
/// argument-marshalling limit of some FFI clients (notably the pure-Go
/// `purego` backend on the SysV/AMD64 ABI, which panics at registration
/// above ~9 arguments). This struct-pointer variant keeps the call
/// surface at 5 arguments so every binding — including `CGO_ENABLED=0`
/// Go — can invoke PAdES signing. Field order/types mirror the scalar
/// function exactly; `#[repr(C)]` makes the layout match a C struct of
/// the same fields on every supported platform.
#[repr(C)]
pub struct PadesSignOptionsC {
    pub certificate_handle: *const std::ffi::c_void,
    pub certs: *const *const u8,
    pub cert_lens: *const usize,
    pub n_certs: usize,
    pub crls: *const *const u8,
    pub crl_lens: *const usize,
    pub n_crls: usize,
    pub ocsps: *const *const u8,
    pub ocsp_lens: *const usize,
    pub n_ocsps: usize,
    pub tsa_url: *const c_char,
    pub reason: *const c_char,
    pub location: *const c_char,
    pub level: i32,
}

/// Struct-options variant of [`pdf_sign_bytes_pades`] (5 arguments).
/// Functionally identical — it unpacks `options` and delegates to
/// `pdf_sign_bytes_pades`, so behaviour, error codes and the returned
/// buffer are byte-for-byte the same. Additive: the 18-argument
/// function is unchanged for existing C/C++/C#/Node callers.
///
/// # Safety
/// `options` must be a valid pointer to a `PadesSignOptionsC` whose
/// pointer fields are valid for the duration of the call (same
/// contract as the scalar function).
#[no_mangle]
pub unsafe extern "C" fn pdf_sign_bytes_pades_opts(
    pdf_data: *const u8,
    pdf_len: usize,
    options: *const PadesSignOptionsC,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if options.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let o = &*options;
    pdf_sign_bytes_pades(
        pdf_data,
        pdf_len,
        o.certificate_handle,
        o.level,
        o.tsa_url,
        o.reason,
        o.location,
        o.certs,
        o.cert_lens,
        o.n_certs,
        o.crls,
        o.crl_lens,
        o.n_crls,
        o.ocsps,
        o.ocsp_lens,
        o.n_ocsps,
        out_len,
        error_code,
    )
}

/// Classify a signature's PAdES level from its CMS attributes alone
/// (B-B vs B-T). B-LT additionally needs the document `/DSS` — read it
/// via [`pdf_document_get_dss`] and inspect its VRI. Returns the frozen
/// `PadesLevel` code, or `-1` on error.
#[no_mangle]
pub extern "C" fn pdf_signature_get_pades_level(
    signature_handle: *const std::ffi::c_void,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "signatures")]
    {
        if signature_handle.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let ffi = handle_ref(signature_handle as *const FfiSignatureInfo);
        set_error(error_code, ERR_SUCCESS);
        crate::signatures::classify_pades_level(&ffi.info, None).code()
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = signature_handle;
        set_error(error_code, _ERR_UNSUPPORTED);
        -1
    }
}

/// Read the document `/DSS` into an opaque handle. Returns `NULL` with
/// `*error_code == ERR_SUCCESS` when the document simply has no DSS
/// (not an error); `NULL` with a non-zero code on a real failure. Free
/// the handle with [`pdf_dss_free`].
#[no_mangle]
pub extern "C" fn pdf_document_get_dss(
    document_handle: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "signatures")]
    {
        if document_handle.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let doc = handle_ref(document_handle as *const PdfDocument);
        match crate::signatures::read_dss(doc) {
            Ok(Some(dss)) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(FfiDss { dss })) as *mut std::ffi::c_void
            },
            Ok(None) => {
                set_error(error_code, ERR_SUCCESS);
                ptr::null_mut()
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = document_handle;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Whether the document carries a document-scoped RFC 3161
/// `/DocTimeStamp` archival timestamp (PAdES-B-LTA, ISO 32000-2:2020
/// §12.8.5). This is the document-level reader signal that
/// `pdf_signature_get_pades_level` cannot report — that call is
/// signature-scoped and tops out at B-LT by design. Returns `1` when a
/// document timestamp is present, `0` when not, `-1` on a null handle
/// or when the `signatures` feature is disabled.
#[no_mangle]
pub extern "C" fn pdf_document_has_timestamp(
    document_handle: *const std::ffi::c_void,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "signatures")]
    {
        if document_handle.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let doc = handle_ref(document_handle as *const PdfDocument);
        set_error(error_code, ERR_SUCCESS);
        i32::from(crate::signatures::has_document_timestamp(&doc.source_bytes))
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = document_handle;
        set_error(error_code, _ERR_UNSUPPORTED);
        -1
    }
}

#[cfg(feature = "signatures")]
macro_rules! dss_count_fn {
    ($name:ident, $field:ident) => {
        /// Number of entries; `-1` if the handle is null.
        #[no_mangle]
        pub extern "C" fn $name(dss: *const std::ffi::c_void) -> i32 {
            if dss.is_null() {
                return -1;
            }
            handle_ref(dss as *const FfiDss).dss.$field.len() as i32
        }
    };
}
#[cfg(feature = "signatures")]
dss_count_fn!(pdf_dss_cert_count, certificates);
#[cfg(feature = "signatures")]
dss_count_fn!(pdf_dss_crl_count, crls);
#[cfg(feature = "signatures")]
dss_count_fn!(pdf_dss_ocsp_count, ocsp_responses);
#[cfg(feature = "signatures")]
dss_count_fn!(pdf_dss_vri_count, vri);

#[cfg(not(feature = "signatures"))]
#[no_mangle]
pub extern "C" fn pdf_dss_cert_count(_dss: *const std::ffi::c_void) -> i32 {
    -1
}
#[cfg(not(feature = "signatures"))]
#[no_mangle]
pub extern "C" fn pdf_dss_crl_count(_dss: *const std::ffi::c_void) -> i32 {
    -1
}
#[cfg(not(feature = "signatures"))]
#[no_mangle]
pub extern "C" fn pdf_dss_ocsp_count(_dss: *const std::ffi::c_void) -> i32 {
    -1
}
#[cfg(not(feature = "signatures"))]
#[no_mangle]
pub extern "C" fn pdf_dss_vri_count(_dss: *const std::ffi::c_void) -> i32 {
    -1
}

#[cfg(feature = "signatures")]
macro_rules! dss_get_fn {
    ($name:ident, $field:ident) => {
        /// Copy the i-th DER blob into a `malloc`'d buffer (free with
        /// [`free_bytes`]); `NULL` + error on out-of-range/null.
        #[no_mangle]
        pub extern "C" fn $name(
            dss: *const std::ffi::c_void,
            index: i32,
            out_len: *mut usize,
            error_code: *mut i32,
        ) -> *mut u8 {
            if dss.is_null() || index < 0 {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            }
            let d = handle_ref(dss as *const FfiDss);
            match d.dss.$field.get(index as usize) {
                Some(der) => {
                    set_error(error_code, ERR_SUCCESS);
                    write_out(out_len, der.len());
                    vec_to_ffi_bytes(der.clone())
                },
                None => {
                    set_error(error_code, ERR_INVALID_ARG);
                    ptr::null_mut()
                },
            }
        }
    };
}
#[cfg(feature = "signatures")]
dss_get_fn!(pdf_dss_get_cert, certificates);
#[cfg(feature = "signatures")]
dss_get_fn!(pdf_dss_get_crl, crls);
#[cfg(feature = "signatures")]
dss_get_fn!(pdf_dss_get_ocsp, ocsp_responses);

#[cfg(not(feature = "signatures"))]
#[no_mangle]
pub extern "C" fn pdf_dss_get_cert(
    _dss: *const std::ffi::c_void,
    _index: i32,
    _out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    set_error(error_code, _ERR_UNSUPPORTED);
    ptr::null_mut()
}
#[cfg(not(feature = "signatures"))]
#[no_mangle]
pub extern "C" fn pdf_dss_get_crl(
    _dss: *const std::ffi::c_void,
    _index: i32,
    _out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    set_error(error_code, _ERR_UNSUPPORTED);
    ptr::null_mut()
}
#[cfg(not(feature = "signatures"))]
#[no_mangle]
pub extern "C" fn pdf_dss_get_ocsp(
    _dss: *const std::ffi::c_void,
    _index: i32,
    _out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    set_error(error_code, _ERR_UNSUPPORTED);
    ptr::null_mut()
}

/// Free a DSS handle from [`pdf_document_get_dss`].
#[no_mangle]
pub extern "C" fn pdf_dss_free(dss: *mut std::ffi::c_void) {
    if !dss.is_null() {
        unsafe {
            drop(Box::from_raw(dss as *mut FfiDss));
        }
    }
}
#[no_mangle]
pub extern "C" fn pdf_certificate_free(handle: *mut std::ffi::c_void) {
    #[cfg(feature = "signatures")]
    {
        if !handle.is_null() {
            unsafe {
                drop(Box::from_raw(handle as *mut crate::signatures::SigningCredentials));
            }
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = handle;
    }
}

// ─── Rendering ──────────────────────────────────────────────────────────────

#[cfg(feature = "rendering")]
use crate::rendering::{
    self, ImageFormat as RenderImageFormat, RenderOptions as RustRenderOptions,
    RenderedImage as RustRenderedImage, DEFAULT_MAX_OUTPUT_PIXELS as RENDER_MAX_OUTPUT_PIXELS,
};

#[cfg(feature = "rendering")]
pub struct FfiRenderedImage {
    inner: RustRenderedImage,
}

#[cfg(not(feature = "rendering"))]
pub struct FfiRenderedImage {
    _dummy: (),
}

#[no_mangle]
pub extern "C" fn pdf_estimate_render_time(
    _doc: *const std::ffi::c_void,
    _page_index: i32,
    error_code: *mut i32,
) -> i32 {
    set_error(error_code, ERR_SUCCESS);
    100 // estimate 100ms
}

#[no_mangle]
pub extern "C" fn pdf_create_renderer(
    _dpi: i32,
    _format: i32,
    _quality: i32,
    _anti_alias: bool,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    // Rendering uses stateless render_page() function, no persistent renderer needed
    set_error(error_code, _ERR_UNSUPPORTED);
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn pdf_render_page(
    doc: *mut PdfDocument,
    page_index: i32,
    format: i32,
    error_code: *mut i32,
) -> *mut FfiRenderedImage {
    #[cfg(feature = "rendering")]
    {
        if doc.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let d = handle_ref(doc as *const _);
        let fmt = if format == 1 {
            RenderImageFormat::Jpeg
        } else {
            RenderImageFormat::Png
        };
        let opts = RustRenderOptions {
            dpi: 150,
            format: fmt,
            ..Default::default()
        };
        match rendering::render_page(d, page_index as usize, &opts) {
            Ok(img) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(FfiRenderedImage { inner: img }))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (doc, page_index, format);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Render a page with the full RenderOptions surface exposed to C callers.
///
/// All four background channels are 0.0..=1.0; set `transparent_background`
/// to 1 to drop the fill entirely (matches Rust's
/// `RenderOptions { background: None, .. }`).
///
/// Mirrors the Python surface added in gap O and the C# RenderOptions class
/// added in gap B. Rust implementation is the single source of truth.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn pdf_render_page_with_options(
    doc: *mut PdfDocument,
    page_index: i32,
    dpi: i32,
    format: i32,
    bg_r: f32,
    bg_g: f32,
    bg_b: f32,
    bg_a: f32,
    transparent_background: i32,
    render_annotations: i32,
    jpeg_quality: i32,
    error_code: *mut i32,
) -> *mut FfiRenderedImage {
    #[cfg(feature = "rendering")]
    {
        if doc.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        if dpi <= 0 || !(1..=100).contains(&jpeg_quality) {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let d = handle_ref(doc as *const _);
        let fmt = if format == 1 {
            RenderImageFormat::Jpeg
        } else {
            RenderImageFormat::Png
        };
        let background = if transparent_background != 0 {
            None
        } else {
            Some([bg_r, bg_g, bg_b, bg_a])
        };
        // OCG layer filtering is exposed on the variant
        // `pdf_render_page_with_options_ex` below — keeping this ABI stable.
        let opts = RustRenderOptions {
            max_output_pixels: RENDER_MAX_OUTPUT_PIXELS,
            dpi: dpi as u32,
            format: fmt,
            background,
            render_annotations: render_annotations != 0,
            jpeg_quality: jpeg_quality as u8,
            excluded_layers: std::collections::HashSet::new(),
            scale_override: None,
        };
        match rendering::render_page(d, page_index as usize, &opts) {
            Ok(img) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(FfiRenderedImage { inner: img }))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (
            doc,
            page_index,
            dpi,
            format,
            bg_r,
            bg_g,
            bg_b,
            bg_a,
            transparent_background,
            render_annotations,
            jpeg_quality,
        );
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Render a page with the full RenderOptions surface plus OCG layer filtering.
///
/// `excluded_layers` is a pointer to an array of `excluded_layers_count` null-
/// terminated UTF-8 C strings. Each string is the `/Name` of an Optional
/// Content Group (OCG) to suppress. Pass a null pointer or zero count to
/// disable filtering (matches `pdf_render_page_with_options` behaviour).
///
/// The renderer also honours OCMD references that resolve to any of the named
/// OCGs, per ISO 32000-1 §8.11.2.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn pdf_render_page_with_options_ex(
    doc: *mut PdfDocument,
    page_index: i32,
    dpi: i32,
    format: i32,
    bg_r: f32,
    bg_g: f32,
    bg_b: f32,
    bg_a: f32,
    transparent_background: i32,
    render_annotations: i32,
    jpeg_quality: i32,
    excluded_layers: *const *const c_char,
    excluded_layers_count: usize,
    error_code: *mut i32,
) -> *mut FfiRenderedImage {
    #[cfg(feature = "rendering")]
    {
        if doc.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        if dpi <= 0 || !(1..=100).contains(&jpeg_quality) {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let d = handle_ref(doc as *const _);
        let fmt = if format == 1 {
            RenderImageFormat::Jpeg
        } else {
            RenderImageFormat::Png
        };
        let background = if transparent_background != 0 {
            None
        } else {
            Some([bg_r, bg_g, bg_b, bg_a])
        };

        let mut layers: std::collections::HashSet<String> = std::collections::HashSet::new();
        if !excluded_layers.is_null() && excluded_layers_count > 0 {
            for i in 0..excluded_layers_count {
                // Safety: caller-supplied pointer must point to an array of
                // at least `excluded_layers_count` C strings or be null.
                let entry: *const c_char = unsafe { *excluded_layers.add(i) };
                if entry.is_null() {
                    continue;
                }
                if let Some(s) = c_str_lossy(entry) {
                    layers.insert(s);
                }
            }
        }

        let opts = RustRenderOptions {
            max_output_pixels: RENDER_MAX_OUTPUT_PIXELS,
            dpi: dpi as u32,
            format: fmt,
            background,
            render_annotations: render_annotations != 0,
            jpeg_quality: jpeg_quality as u8,
            excluded_layers: layers,
            scale_override: None,
        };
        match rendering::render_page(d, page_index as usize, &opts) {
            Ok(img) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(FfiRenderedImage { inner: img }))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (
            doc,
            page_index,
            dpi,
            format,
            bg_r,
            bg_g,
            bg_b,
            bg_a,
            transparent_background,
            render_annotations,
            jpeg_quality,
            excluded_layers,
            excluded_layers_count,
        );
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Render a rectangular region of a page. `crop_*` are in PDF user-space
/// points (origin bottom-left). Format: 0=PNG, 1=JPEG.
#[no_mangle]
pub extern "C" fn pdf_render_page_region(
    doc: *mut PdfDocument,
    page_index: i32,
    crop_x: f32,
    crop_y: f32,
    crop_width: f32,
    crop_height: f32,
    format: i32,
    error_code: *mut i32,
) -> *mut FfiRenderedImage {
    #[cfg(feature = "rendering")]
    {
        if doc.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let d = handle_ref(doc as *const _);
        let fmt = if format == 1 {
            RenderImageFormat::Jpeg
        } else {
            RenderImageFormat::Png
        };
        let opts = RustRenderOptions {
            dpi: 150,
            format: fmt,
            ..Default::default()
        };
        match rendering::render_page_region(
            d,
            page_index as usize,
            (crop_x, crop_y, crop_width, crop_height),
            &opts,
        ) {
            Ok(img) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(FfiRenderedImage { inner: img }))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (doc, page_index, crop_x, crop_y, crop_width, crop_height, format);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_render_page_zoom(
    doc: *mut PdfDocument,
    page_index: i32,
    zoom: f32,
    format: i32,
    error_code: *mut i32,
) -> *mut FfiRenderedImage {
    #[cfg(feature = "rendering")]
    {
        if doc.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let d = handle_ref(doc as *const _);
        let dpi = (72.0 * zoom) as u32;
        let fmt = if format == 1 {
            RenderImageFormat::Jpeg
        } else {
            RenderImageFormat::Png
        };
        let opts = RustRenderOptions {
            dpi,
            format: fmt,
            ..Default::default()
        };
        match rendering::render_page(d, page_index as usize, &opts) {
            Ok(img) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(FfiRenderedImage { inner: img }))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (doc, page_index, zoom, format);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Render a page to fit inside `w`×`h` pixels, preserving aspect ratio.
#[no_mangle]
pub extern "C" fn pdf_render_page_fit(
    doc: *mut PdfDocument,
    page_index: i32,
    w: i32,
    h: i32,
    format: i32,
    error_code: *mut i32,
) -> *mut FfiRenderedImage {
    #[cfg(feature = "rendering")]
    {
        if doc.is_null() || w <= 0 || h <= 0 {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let d = handle_ref(doc as *const _);
        let fmt = if format == 1 {
            RenderImageFormat::Jpeg
        } else {
            RenderImageFormat::Png
        };
        let opts = RustRenderOptions {
            dpi: 150,
            format: fmt,
            ..Default::default()
        };
        match rendering::render_page_fit(d, page_index as usize, w as u32, h as u32, &opts) {
            Ok(img) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(FfiRenderedImage { inner: img }))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (doc, page_index, w, h, format);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_render_page_thumbnail(
    doc: *mut PdfDocument,
    page_index: i32,
    _size: i32,
    format: i32,
    error_code: *mut i32,
) -> *mut FfiRenderedImage {
    #[cfg(feature = "rendering")]
    {
        if doc.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let d = handle_ref(doc as *const _);
        let fmt = if format == 1 {
            RenderImageFormat::Jpeg
        } else {
            RenderImageFormat::Png
        };
        let opts = RustRenderOptions {
            dpi: 72,
            format: fmt,
            ..Default::default()
        };
        match rendering::render_page(d, page_index as usize, &opts) {
            Ok(img) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(FfiRenderedImage { inner: img }))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (doc, page_index, _size, format);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Render a page and return the raw premultiplied RGBA8888 pixel buffer.
///
/// The caller retrieves the pixel bytes via `pdf_get_rendered_image_data`
/// the dimensions via `out_width`/`out_height` (set on success). Pixels are
/// row-major, top-left origin; `data_len == *out_width * *out_height * 4`.
/// Free the returned handle with `pdf_rendered_image_free`.
///
/// Returns null on error; check `error_code`. `out_width` and `out_height`
/// are only written when the return value is non-null.
#[no_mangle]
pub extern "C" fn pdf_render_page_raw(
    doc: *mut PdfDocument,
    page_index: i32,
    dpi: i32,
    out_width: *mut i32,
    out_height: *mut i32,
    error_code: *mut i32,
) -> *mut FfiRenderedImage {
    #[cfg(feature = "rendering")]
    {
        if doc.is_null() || out_width.is_null() || out_height.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let effective_dpi = if dpi <= 0 { 150 } else { dpi as u32 };
        let d = handle_ref(doc as *const _);
        let opts = RustRenderOptions {
            dpi: effective_dpi,
            format: RenderImageFormat::RawRgba8,
            ..Default::default()
        };
        match rendering::render_page(d, page_index as usize, &opts) {
            Ok(img) => {
                write_out(out_width, img.width as i32);
                write_out(out_height, img.height as i32);
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(FfiRenderedImage { inner: img }))
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (doc, page_index, dpi, out_width, out_height);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_get_rendered_image_width(
    img: *const FfiRenderedImage,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "rendering")]
    {
        if img.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return 0;
        }
        set_error(error_code, ERR_SUCCESS);
        handle_ref(img).inner.width as i32
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = img;
        set_error(error_code, _ERR_UNSUPPORTED);
        0
    }
}

#[no_mangle]
pub extern "C" fn pdf_get_rendered_image_height(
    img: *const FfiRenderedImage,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "rendering")]
    {
        if img.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return 0;
        }
        set_error(error_code, ERR_SUCCESS);
        handle_ref(img).inner.height as i32
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = img;
        set_error(error_code, _ERR_UNSUPPORTED);
        0
    }
}

#[no_mangle]
pub extern "C" fn pdf_get_rendered_image_data(
    img: *const FfiRenderedImage,
    data_len: *mut i32,
    error_code: *mut i32,
) -> *mut u8 {
    #[cfg(feature = "rendering")]
    {
        if img.is_null() || data_len.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let i = handle_ref(img);
        let bytes = i.inner.as_bytes().to_vec();
        write_out(data_len, bytes.len() as i32);
        set_error(error_code, ERR_SUCCESS);
        vec_to_ffi_bytes(bytes)
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (img, data_len);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_save_rendered_image(
    img: *const FfiRenderedImage,
    file_path: *const c_char,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "rendering")]
    {
        if img.is_null() || file_path.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let path = match c_str(file_path) {
            Ok(s) => s,
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return -1;
            },
        };
        let i = handle_ref(img);
        match i.inner.save(path) {
            Ok(()) => {
                set_error(error_code, ERR_SUCCESS);
                0
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                -1
            },
        }
    }
    #[cfg(not(feature = "rendering"))]
    {
        let _ = (img, file_path);
        set_error(error_code, _ERR_UNSUPPORTED);
        -1
    }
}

#[no_mangle]
pub extern "C" fn pdf_rendered_image_free(handle: *mut FfiRenderedImage) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}
#[no_mangle]
pub extern "C" fn pdf_renderer_free(_handle: *mut std::ffi::c_void) {}

// ─── TSA (Time Stamp Authority) ────────────────────────────────────────────
// TSA is integrated into signatures via SignOptions::with_timestamp()
// No standalone TSA client in the Rust library — these remain stubs

#[no_mangle]
pub extern "C" fn pdf_tsa_client_create(
    url: *const c_char,
    username: *const c_char,
    password: *const c_char,
    timeout: i32,
    hash_algo: i32,
    use_nonce: bool,
    cert_req: bool,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "tsa-client")]
    {
        if url.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let url_str = c_str_lossy(url).unwrap_or_default();
        let user_opt = c_str_lossy(username);
        let pw_opt = c_str_lossy(password);
        let algo = hash_algo_from_i32(hash_algo);
        let cfg = crate::signatures::TsaClientConfig {
            url: url_str,
            username: user_opt,
            password: pw_opt,
            timeout: if timeout > 0 {
                std::time::Duration::from_secs(timeout as u64)
            } else {
                std::time::Duration::from_secs(30)
            },
            hash_algorithm: algo,
            use_nonce,
            cert_req,
        };
        let client = crate::signatures::TsaClient::new(cfg);
        set_error(error_code, ERR_SUCCESS);
        Box::into_raw(Box::new(client)) as *mut std::ffi::c_void
    }
    #[cfg(not(feature = "tsa-client"))]
    {
        let _ = (url, username, password, timeout, hash_algo, use_nonce, cert_req);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_tsa_client_free(client: *mut std::ffi::c_void) {
    #[cfg(feature = "tsa-client")]
    {
        if !client.is_null() {
            unsafe {
                drop(Box::from_raw(client as *mut crate::signatures::TsaClient));
            }
        }
    }
    #[cfg(not(feature = "tsa-client"))]
    {
        let _ = client;
    }
}

#[no_mangle]
pub extern "C" fn pdf_tsa_request_timestamp(
    client: *const std::ffi::c_void,
    data: *const u8,
    data_len: usize,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "tsa-client")]
    {
        if client.is_null() || data.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let c = handle_ref(client as *const crate::signatures::TsaClient);
        let slice = raw_slice(data, data_len);
        match c.request_timestamp(slice) {
            Ok(ts) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(ts)) as *mut std::ffi::c_void
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "tsa-client"))]
    {
        let _ = (client, data, data_len);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_tsa_request_timestamp_hash(
    client: *const std::ffi::c_void,
    hash: *const u8,
    hash_len: usize,
    hash_algo: i32,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "tsa-client")]
    {
        if client.is_null() || hash.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let c = handle_ref(client as *const crate::signatures::TsaClient);
        let slice = raw_slice(hash, hash_len);
        let algo = hash_algo_from_i32(hash_algo);
        match c.request_timestamp_hash(slice, algo) {
            Ok(ts) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(ts)) as *mut std::ffi::c_void
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "tsa-client"))]
    {
        let _ = (client, hash, hash_len, hash_algo);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[cfg(feature = "tsa-client")]
fn hash_algo_from_i32(code: i32) -> crate::signatures::HashAlgorithm {
    match code {
        1 => crate::signatures::HashAlgorithm::Sha1,
        2 => crate::signatures::HashAlgorithm::Sha256,
        3 => crate::signatures::HashAlgorithm::Sha384,
        4 => crate::signatures::HashAlgorithm::Sha512,
        _ => crate::signatures::HashAlgorithm::Sha256,
    }
}

/// Parse a DER-encoded RFC 3161 TimeStampToken (or bare TSTInfo) into
/// an owned `Timestamp` handle. Every `pdf_timestamp_*` accessor below
/// is cheap O(1) once the handle exists.
#[no_mangle]
pub extern "C" fn pdf_timestamp_parse(
    bytes: *const u8,
    len: usize,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "signatures")]
    {
        if bytes.is_null() || len == 0 {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let slice = raw_slice(bytes, len);
        match crate::signatures::Timestamp::from_der(slice) {
            Ok(ts) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(ts)) as *mut std::ffi::c_void
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (bytes, len);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_timestamp_get_token(
    ts: *const std::ffi::c_void,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *const u8 {
    #[cfg(feature = "signatures")]
    {
        if ts.is_null() || out_len.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null();
        }
        let t = handle_ref(ts as *const crate::signatures::Timestamp);
        let bytes = t.token_bytes();
        write_out(out_len, bytes.len());
        set_error(error_code, ERR_SUCCESS);
        bytes.as_ptr()
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (ts, out_len);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn pdf_timestamp_get_time(ts: *const std::ffi::c_void, error_code: *mut i32) -> i64 {
    #[cfg(feature = "signatures")]
    {
        if ts.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return 0;
        }
        let t = handle_ref(ts as *const crate::signatures::Timestamp);
        set_error(error_code, ERR_SUCCESS);
        t.time()
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = ts;
        set_error(error_code, _ERR_UNSUPPORTED);
        0
    }
}

#[no_mangle]
pub extern "C" fn pdf_timestamp_get_serial(
    ts: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "signatures")]
    {
        if ts.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let t = handle_ref(ts as *const crate::signatures::Timestamp);
        set_error(error_code, ERR_SUCCESS);
        to_c_string(&t.serial())
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = ts;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_timestamp_get_tsa_name(
    ts: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "signatures")]
    {
        if ts.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let t = handle_ref(ts as *const crate::signatures::Timestamp);
        set_error(error_code, ERR_SUCCESS);
        to_c_string(&t.tsa_name())
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = ts;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_timestamp_get_policy_oid(
    ts: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "signatures")]
    {
        if ts.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let t = handle_ref(ts as *const crate::signatures::Timestamp);
        set_error(error_code, ERR_SUCCESS);
        to_c_string(&t.policy_oid())
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = ts;
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

#[no_mangle]
pub extern "C" fn pdf_timestamp_get_hash_algorithm(
    ts: *const std::ffi::c_void,
    error_code: *mut i32,
) -> i32 {
    #[cfg(feature = "signatures")]
    {
        if ts.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let t = handle_ref(ts as *const crate::signatures::Timestamp);
        set_error(error_code, ERR_SUCCESS);
        t.hash_algorithm() as i32
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = ts;
        set_error(error_code, _ERR_UNSUPPORTED);
        -1
    }
}

#[no_mangle]
pub extern "C" fn pdf_timestamp_get_message_imprint(
    ts: *const std::ffi::c_void,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *const u8 {
    // Returns a pointer into the owned `Timestamp` — lives as long as
    // the caller holds the Timestamp handle; must NOT be freed
    // separately. `out_len` receives the imprint byte length.
    #[cfg(feature = "signatures")]
    {
        if ts.is_null() || out_len.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null();
        }
        let t = handle_ref(ts as *const crate::signatures::Timestamp);
        // Stash the imprint on the handle so the returned pointer
        // remains valid for the handle's lifetime. The getter on
        // Timestamp clones, which would invalidate the pointer on
        // return — we need an API that returns a borrowed slice.
        // Compromise: allocate a new leaked Box each call and rely on
        // the caller to NOT free. Instead we stash on the handle:
        let imprint = t.message_imprint_ref();
        write_out(out_len, imprint.len());
        set_error(error_code, ERR_SUCCESS);
        imprint.as_ptr()
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = (ts, out_len);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn pdf_timestamp_verify(ts: *const std::ffi::c_void, error_code: *mut i32) -> bool {
    #[cfg(feature = "signatures")]
    {
        if ts.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return false;
        }
        let t = handle_ref(ts as *const crate::signatures::Timestamp);
        match t.verify() {
            Ok(valid) => {
                set_error(error_code, ERR_SUCCESS);
                valid
            },
            Err(_) => {
                set_error(error_code, _ERR_UNSUPPORTED);
                false
            },
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = ts;
        set_error(error_code, _ERR_UNSUPPORTED);
        false
    }
}

#[no_mangle]
pub extern "C" fn pdf_timestamp_free(ts: *mut std::ffi::c_void) {
    #[cfg(feature = "signatures")]
    {
        if !ts.is_null() {
            unsafe {
                drop(Box::from_raw(ts as *mut crate::signatures::Timestamp));
            }
        }
    }
    #[cfg(not(feature = "signatures"))]
    {
        let _ = ts;
    }
}

#[no_mangle]
pub extern "C" fn pdf_signature_add_timestamp(
    _sig: *const std::ffi::c_void,
    _ts: *const std::ffi::c_void,
    error_code: *mut i32,
) -> bool {
    set_error(error_code, _ERR_UNSUPPORTED);
    false
}

#[no_mangle]
pub extern "C" fn pdf_signature_has_timestamp(
    _sig: *const std::ffi::c_void,
    error_code: *mut i32,
) -> bool {
    set_error(error_code, _ERR_UNSUPPORTED);
    false
}

#[no_mangle]
pub extern "C" fn pdf_signature_get_timestamp(
    _sig: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    set_error(error_code, _ERR_UNSUPPORTED);
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn pdf_add_timestamp(
    _pdf_data: *const u8,
    _pdf_len: usize,
    _sig_index: i32,
    _tsa_url: *const c_char,
    _out_data: *mut *mut u8,
    _out_len: *mut usize,
    error_code: *mut i32,
) -> bool {
    set_error(error_code, _ERR_UNSUPPORTED);
    false
}

// ─── PDF/UA Validation (always available) ──────────────────────────────────

use crate::compliance::pdf_ua::{PdfUaLevel, PdfUaValidator, UaValidationResult};

pub struct FfiUaResults {
    result: UaValidationResult,
}

#[no_mangle]
pub extern "C" fn pdf_validate_pdf_ua(
    document: *mut PdfDocument,
    level: i32,
    error_code: *mut i32,
) -> *mut FfiUaResults {
    if document.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_mut(document);
    let ua_level = if level == 2 {
        PdfUaLevel::Ua2
    } else {
        PdfUaLevel::Ua1
    };
    match PdfUaValidator::new().validate(doc, ua_level) {
        Ok(result) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiUaResults { result }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_pdf_ua_is_accessible(
    results: *const FfiUaResults,
    error_code: *mut i32,
) -> bool {
    if results.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    handle_ref(results).result.is_compliant
}

#[no_mangle]
pub extern "C" fn pdf_pdf_ua_error_count(results: *const FfiUaResults) -> i32 {
    if results.is_null() {
        return 0;
    }
    handle_ref(results).result.errors.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_pdf_ua_get_error(
    results: *const FfiUaResults,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if results.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let r = handle_ref(results);
    if (index as usize) >= r.result.errors.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&r.result.errors[index as usize].message)
}

#[no_mangle]
pub extern "C" fn pdf_pdf_ua_warning_count(results: *const FfiUaResults) -> i32 {
    if results.is_null() {
        return 0;
    }
    handle_ref(results).result.warnings.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_pdf_ua_get_warning(
    results: *const FfiUaResults,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if results.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let r = handle_ref(results);
    if (index as usize) >= r.result.warnings.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&r.result.warnings[index as usize].message)
}

#[no_mangle]
pub extern "C" fn pdf_pdf_ua_get_stats(
    results: *const FfiUaResults,
    out_struct: *mut i32,
    out_images: *mut i32,
    out_tables: *mut i32,
    out_forms: *mut i32,
    out_annotations: *mut i32,
    out_pages: *mut i32,
    error_code: *mut i32,
) -> bool {
    if results.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let r = handle_ref(results);
    let s = &r.result.stats;
    write_out(out_struct, s.structure_elements_checked as i32);
    write_out(out_images, s.images_checked as i32);
    write_out(out_tables, s.tables_checked as i32);
    write_out(out_forms, s.form_fields_checked as i32);
    write_out(out_annotations, s.annotations_checked as i32);
    write_out(out_pages, s.pages_checked as i32);
    set_error(error_code, ERR_SUCCESS);
    true
}

#[no_mangle]
pub extern "C" fn pdf_pdf_ua_results_free(results: *mut FfiUaResults) {
    if !results.is_null() {
        unsafe {
            drop(Box::from_raw(results));
        }
    }
}

// ─── FDF/XFDF Import/Export (always available) ─────────────────────────────

use crate::extractors::FormExtractor;
use crate::fdf::{FdfWriter, XfdfWriter};

#[no_mangle]
pub extern "C" fn pdf_form_import_from_file(
    _document: *const std::ffi::c_void,
    _filename: *const c_char,
    error_code: *mut i32,
) -> bool {
    // Import requires DocumentEditor — not supported via PdfDocument handle
    set_error(error_code, _ERR_UNSUPPORTED);
    false
}

#[no_mangle]
pub extern "C" fn pdf_document_import_form_data(
    _document: *const std::ffi::c_void,
    _data_path: *const c_char,
    error_code: *mut i32,
) -> i32 {
    set_error(error_code, _ERR_UNSUPPORTED);
    -1
}

#[no_mangle]
pub extern "C" fn pdf_editor_import_fdf_bytes(
    _document: *const std::ffi::c_void,
    _data: *const u8,
    _data_len: usize,
    error_code: *mut i32,
) -> i32 {
    set_error(error_code, _ERR_UNSUPPORTED);
    -1
}

#[no_mangle]
pub extern "C" fn pdf_editor_import_xfdf_bytes(
    _document: *const std::ffi::c_void,
    _data: *const u8,
    _data_len: usize,
    error_code: *mut i32,
) -> i32 {
    set_error(error_code, _ERR_UNSUPPORTED);
    -1
}

/// Export form data from a PdfDocument. format_type: 0=FDF, 1=XFDF
#[no_mangle]
pub extern "C" fn pdf_document_export_form_data_to_bytes(
    document: *mut PdfDocument,
    format_type: i32,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if document.is_null() || out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_mut(document);
    let fields = match FormExtractor::extract_fields(doc) {
        Ok(f) => f,
        Err(e) => {
            set_error(error_code, classify_error(&e));
            return ptr::null_mut();
        },
    };
    let bytes = if format_type == 1 {
        // XFDF (XML)
        let writer = XfdfWriter::from_fields(fields);
        writer.to_bytes()
    } else {
        // FDF
        let writer = FdfWriter::from_fields(fields);
        match writer.to_bytes() {
            Ok(b) => b,
            Err(e) => {
                set_error(error_code, classify_error(&e));
                return ptr::null_mut();
            },
        }
    };
    write_out(out_len, bytes.len());
    set_error(error_code, ERR_SUCCESS);
    vec_to_ffi_bytes(bytes)
}

// ─── Open from bytes / password ─────────────────────────────────────────────

/// Open a PDF document from a byte buffer.
#[no_mangle]
pub extern "C" fn pdf_document_open_from_bytes(
    data: *const u8,
    len: usize,
    error_code: *mut i32,
) -> *mut PdfDocument {
    if data.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let bytes = raw_slice(data, len).to_vec();
    match PdfDocument::from_bytes(bytes) {
        Ok(doc) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(doc))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Open a PDF with password.
#[no_mangle]
pub extern "C" fn pdf_document_open_with_password(
    path: *const c_char,
    password: *const c_char,
    error_code: *mut i32,
) -> *mut PdfDocument {
    if path.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let path_str = match c_str(path) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    match PdfDocument::open(path_str) {
        Ok(mut doc) => {
            if !password.is_null() {
                if let Ok(pwd) = c_str(password) {
                    let _ = doc.authenticate(pwd.as_bytes());
                }
            }
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(doc))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Check if the document is encrypted.
#[no_mangle]
pub extern "C" fn pdf_document_is_encrypted(handle: *const PdfDocument) -> bool {
    if handle.is_null() {
        return false;
    }
    handle_ref(handle).is_encrypted()
}

/// Authenticate with password.
#[no_mangle]
pub extern "C" fn pdf_document_authenticate(
    handle: *mut PdfDocument,
    password: *const c_char,
    error_code: *mut i32,
) -> bool {
    if handle.is_null() || password.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let doc = handle_ref(handle);
    let pwd = match c_str(password) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return false;
        },
    };
    match doc.authenticate(pwd.as_bytes()) {
        Ok(ok) => {
            set_error(error_code, ERR_SUCCESS);
            ok
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            false
        },
    }
}

// ─── Extract all text / all HTML / all plain text ───────────────────────────

#[no_mangle]
pub extern "C" fn pdf_document_extract_all_text(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_all_text() {
        Ok(t) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&t)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_to_html_all(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    let opts = ConversionOptions::default();
    match doc.to_html_all(&opts) {
        Ok(t) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&t)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_to_plain_text_all(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    let opts = ConversionOptions::default();
    match doc.to_plain_text_all(&opts) {
        Ok(t) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&t)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

// ─── Granular extraction: chars, words, lines, tables ───────────────────────

use crate::layout::{TextChar, TextLine as RustTextLine, Word};

// --- Chars ---

pub struct FfiCharList {
    chars: Vec<TextChar>,
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_chars(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut FfiCharList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_chars(page_index as usize) {
        Ok(chars) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiCharList { chars }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_char_count(chars: *const FfiCharList) -> i32 {
    if chars.is_null() {
        return 0;
    }
    handle_ref(chars).chars.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_char_get_char(
    chars: *const FfiCharList,
    index: i32,
    error_code: *mut i32,
) -> u32 {
    if chars.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(chars);
    if (index as usize) >= list.chars.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.chars[index as usize].char as u32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_char_get_bbox(
    chars: *const FfiCharList,
    index: i32,
    x: *mut f32,
    y: *mut f32,
    w: *mut f32,
    h: *mut f32,
    error_code: *mut i32,
) {
    if chars.is_null() || index < 0 || x.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let list = handle_ref(chars);
    if (index as usize) >= list.chars.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    let b = &list.chars[index as usize].bbox;
    write_out(x, b.x);
    write_out(y, b.y);
    write_out(w, b.width);
    write_out(h, b.height);
    set_error(error_code, ERR_SUCCESS);
}

#[no_mangle]
pub extern "C" fn pdf_oxide_char_get_font_name(
    chars: *const FfiCharList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if chars.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(chars);
    if (index as usize) >= list.chars.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.chars[index as usize].font_name)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_char_get_font_size(
    chars: *const FfiCharList,
    index: i32,
    error_code: *mut i32,
) -> f32 {
    if chars.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0.0;
    }
    let list = handle_ref(chars);
    if (index as usize) >= list.chars.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0.0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.chars[index as usize].font_size
}

#[no_mangle]
pub extern "C" fn pdf_oxide_char_list_free(handle: *mut FfiCharList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// --- Words ---

pub struct FfiWordList {
    words: Vec<Word>,
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_words(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut FfiWordList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_words(page_index as usize) {
        Ok(words) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiWordList { words }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_word_count(words: *const FfiWordList) -> i32 {
    if words.is_null() {
        return 0;
    }
    handle_ref(words).words.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_word_get_text(
    words: *const FfiWordList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if words.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(words);
    if (index as usize) >= list.words.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.words[index as usize].text)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_word_get_bbox(
    words: *const FfiWordList,
    index: i32,
    x: *mut f32,
    y: *mut f32,
    w: *mut f32,
    h: *mut f32,
    error_code: *mut i32,
) {
    if words.is_null() || index < 0 || x.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let list = handle_ref(words);
    if (index as usize) >= list.words.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    let b = &list.words[index as usize].bbox;
    write_out(x, b.x);
    write_out(y, b.y);
    write_out(w, b.width);
    write_out(h, b.height);
    set_error(error_code, ERR_SUCCESS);
}

#[no_mangle]
pub extern "C" fn pdf_oxide_word_get_font_name(
    words: *const FfiWordList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if words.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(words);
    if (index as usize) >= list.words.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.words[index as usize].dominant_font)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_word_get_font_size(
    words: *const FfiWordList,
    index: i32,
    error_code: *mut i32,
) -> f32 {
    if words.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0.0;
    }
    let list = handle_ref(words);
    if (index as usize) >= list.words.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0.0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.words[index as usize].avg_font_size
}

#[no_mangle]
pub extern "C" fn pdf_oxide_word_is_bold(
    words: *const FfiWordList,
    index: i32,
    error_code: *mut i32,
) -> bool {
    if words.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let list = handle_ref(words);
    if (index as usize) >= list.words.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    list.words[index as usize].is_bold
}

/// Content-stream emission order of the word's originating span: the order in
/// which spans were drawn during the Tj/TJ walk. Words with adjacent values
/// were drawn consecutively, distinguishing genuinely-consecutive draw calls
/// from spatially-close-but-stream-distant ones, independent of reading order.
/// Returns -1 (with an error code) on a null handle or out-of-range index.
#[no_mangle]
pub extern "C" fn pdf_oxide_word_get_sequence(
    words: *const FfiWordList,
    index: i32,
    error_code: *mut i32,
) -> i64 {
    if words.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let list = handle_ref(words);
    if (index as usize) >= list.words.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return -1;
    }
    set_error(error_code, ERR_SUCCESS);
    list.words[index as usize].sequence as i64
}

/// Rotation of the word's glyph run in degrees, snapped to a quadrant
/// (0 / 90 / 180 / -90) from the composed text-rendering matrix
/// (ISO 32000-1:2008 section 9.4.4). 90 means the text reads bottom-to-top
/// on an unrotated page — a landscape table typeset on a portrait page —
/// so callers can transform coordinates into the reading frame.
/// Returns 0.0 (with an error code) on a null handle or out-of-range index.
#[no_mangle]
pub extern "C" fn pdf_oxide_word_get_rotation(
    words: *const FfiWordList,
    index: i32,
    error_code: *mut i32,
) -> f32 {
    if words.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0.0;
    }
    let list = handle_ref(words);
    if (index as usize) >= list.words.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0.0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.words[index as usize].rotation_degrees
}

#[no_mangle]
pub extern "C" fn pdf_oxide_word_list_free(handle: *mut FfiWordList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// --- Text Lines ---

pub struct FfiTextLineList {
    lines: Vec<RustTextLine>,
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_text_lines(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut FfiTextLineList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_text_lines(page_index as usize) {
        Ok(lines) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiTextLineList { lines }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_line_count(lines: *const FfiTextLineList) -> i32 {
    if lines.is_null() {
        return 0;
    }
    handle_ref(lines).lines.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_line_get_text(
    lines: *const FfiTextLineList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if lines.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(lines);
    if (index as usize) >= list.lines.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.lines[index as usize].text)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_line_get_bbox(
    lines: *const FfiTextLineList,
    index: i32,
    x: *mut f32,
    y: *mut f32,
    w: *mut f32,
    h: *mut f32,
    error_code: *mut i32,
) {
    if lines.is_null() || index < 0 || x.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let list = handle_ref(lines);
    if (index as usize) >= list.lines.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    let b = &list.lines[index as usize].bbox;
    write_out(x, b.x);
    write_out(y, b.y);
    write_out(w, b.width);
    write_out(h, b.height);
    set_error(error_code, ERR_SUCCESS);
}

#[no_mangle]
pub extern "C" fn pdf_oxide_line_get_word_count(
    lines: *const FfiTextLineList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if lines.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(lines);
    if (index as usize) >= list.lines.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.lines[index as usize].words.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_line_list_free(handle: *mut FfiTextLineList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// --- Tables ---

use crate::structure::table_extractor::Table;

pub struct FfiTableList {
    tables: Vec<Table>,
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_tables(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut FfiTableList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_tables(page_index as usize) {
        Ok(tables) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiTableList { tables }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_table_count(tables: *const FfiTableList) -> i32 {
    if tables.is_null() {
        return 0;
    }
    handle_ref(tables).tables.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_table_get_row_count(
    tables: *const FfiTableList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if tables.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(tables);
    if (index as usize) >= list.tables.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.tables[index as usize].rows.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_table_get_col_count(
    tables: *const FfiTableList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if tables.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(tables);
    if (index as usize) >= list.tables.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.tables[index as usize].col_count as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_table_get_cell_text(
    tables: *const FfiTableList,
    table_index: i32,
    row: i32,
    col: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if tables.is_null() || table_index < 0 || row < 0 || col < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(tables);
    if (table_index as usize) >= list.tables.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    let table = &list.tables[table_index as usize];
    if (row as usize) >= table.rows.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    let row_data = &table.rows[row as usize];
    if (col as usize) >= row_data.cells.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&row_data.cells[col as usize].text)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_table_has_header(
    tables: *const FfiTableList,
    index: i32,
    error_code: *mut i32,
) -> bool {
    if tables.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let list = handle_ref(tables);
    if (index as usize) >= list.tables.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    list.tables[index as usize].has_header
}

#[no_mangle]
pub extern "C" fn pdf_oxide_table_list_free(handle: *mut FfiTableList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ─── Region extraction ──────────────────────────────────────────────────────

use crate::geometry::Rect;
use crate::layout::RectFilterMode;

fn make_rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(x, y, w, h)
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_text_in_rect(
    handle: *mut PdfDocument,
    page_index: i32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_text_in_rect(
        page_index as usize,
        make_rect(x, y, w, h),
        RectFilterMode::Intersects,
    ) {
        Ok(t) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&t)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_words_in_rect(
    handle: *mut PdfDocument,
    page_index: i32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> *mut FfiWordList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_words_in_rect(
        page_index as usize,
        make_rect(x, y, w, h),
        RectFilterMode::Intersects,
    ) {
        Ok(words) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiWordList { words }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_lines_in_rect(
    handle: *mut PdfDocument,
    page_index: i32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> *mut FfiTextLineList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_text_lines_in_rect(
        page_index as usize,
        make_rect(x, y, w, h),
        RectFilterMode::Intersects,
    ) {
        Ok(lines) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiTextLineList { lines }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_tables_in_rect(
    handle: *mut PdfDocument,
    page_index: i32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> *mut FfiTableList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_tables_in_rect(page_index as usize, make_rect(x, y, w, h)) {
        Ok(tables) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiTableList { tables }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_images_in_rect(
    handle: *mut PdfDocument,
    page_index: i32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> *mut FfiImageList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_images_in_rect(page_index as usize, make_rect(x, y, w, h)) {
        Ok(images) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiImageList { images }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

// ─── Form fields ────────────────────────────────────────────────────────────

use crate::extractors::FormField as RustFormField;

pub struct FfiFormFieldList {
    fields: Vec<RustFormField>,
}

#[no_mangle]
pub extern "C" fn pdf_document_get_form_fields(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut FfiFormFieldList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match FormExtractor::extract_fields(doc) {
        Ok(fields) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiFormFieldList { fields }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_form_field_count(fields: *const FfiFormFieldList) -> i32 {
    if fields.is_null() {
        return 0;
    }
    handle_ref(fields).fields.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_form_field_get_name(
    fields: *const FfiFormFieldList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if fields.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(fields);
    if (index as usize) >= list.fields.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&list.fields[index as usize].full_name)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_form_field_get_type(
    fields: *const FfiFormFieldList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if fields.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(fields);
    if (index as usize) >= list.fields.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&format!("{:?}", list.fields[index as usize].field_type))
}

#[no_mangle]
pub extern "C" fn pdf_oxide_form_field_get_value(
    fields: *const FfiFormFieldList,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if fields.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(fields);
    if (index as usize) >= list.fields.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&format!("{:?}", list.fields[index as usize].value))
}

#[no_mangle]
pub extern "C" fn pdf_oxide_form_field_is_readonly(
    fields: *const FfiFormFieldList,
    index: i32,
    error_code: *mut i32,
) -> bool {
    if fields.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let list = handle_ref(fields);
    if (index as usize) >= list.fields.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    list.fields[index as usize]
        .flags
        .map(|f| (f & 0x1) != 0)
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_form_field_is_required(
    fields: *const FfiFormFieldList,
    index: i32,
    error_code: *mut i32,
) -> bool {
    if fields.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let list = handle_ref(fields);
    if (index as usize) >= list.fields.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    list.fields[index as usize]
        .flags
        .map(|f| (f & 0x2) != 0)
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn pdf_oxide_form_field_list_free(handle: *mut FfiFormFieldList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Check if document has XFA forms.
#[no_mangle]
pub extern "C" fn pdf_document_has_xfa(handle: *mut PdfDocument) -> bool {
    if handle.is_null() {
        return false;
    }
    let doc = handle_ref(handle);
    if let Ok(catalog) = doc.catalog() {
        if let crate::object::Object::Dictionary(dict) = &catalog {
            if let Some(acroform) = dict.get("AcroForm") {
                if let crate::object::Object::Dictionary(form_dict) = acroform {
                    return form_dict.contains_key("XFA");
                }
            }
        }
    }
    false
}

// ─── Artifact removal ───────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn pdf_document_remove_headers(
    handle: *mut PdfDocument,
    threshold: f32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let doc = handle_ref(handle);
    match doc.remove_headers(threshold) {
        Ok(n) => {
            set_error(error_code, ERR_SUCCESS);
            n as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_remove_footers(
    handle: *mut PdfDocument,
    threshold: f32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let doc = handle_ref(handle);
    match doc.remove_footers(threshold) {
        Ok(n) => {
            set_error(error_code, ERR_SUCCESS);
            n as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_remove_artifacts(
    handle: *mut PdfDocument,
    threshold: f32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let doc = handle_ref(handle);
    match doc.remove_artifacts(threshold) {
        Ok(n) => {
            set_error(error_code, ERR_SUCCESS);
            n as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_erase_header(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let doc = handle_ref(handle);
    match doc.erase_header(page_index as usize) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_erase_footer(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let doc = handle_ref(handle);
    match doc.erase_footer(page_index as usize) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_document_erase_artifacts(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let doc = handle_ref(handle);
    match doc.erase_artifacts(page_index as usize) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

// ─── Editor: page operations ────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn document_editor_delete_page(
    handle: *mut DocumentEditor,
    page_index: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.remove_page(page_index as usize) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_move_page(
    handle: *mut DocumentEditor,
    from: i32,
    to: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.move_page(from as usize, to as usize) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_get_page_rotation(
    handle: *mut DocumentEditor,
    page: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let editor = handle_mut(handle);
    match editor.get_page_rotation(page as usize) {
        Ok(r) => {
            set_error(error_code, ERR_SUCCESS);
            r
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            0
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_set_page_rotation(
    handle: *mut DocumentEditor,
    page: i32,
    degrees: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.set_page_rotation(page as usize, degrees) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_erase_region(
    handle: *mut DocumentEditor,
    page: i32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.erase_region(page as usize, [x, y, x + w, y + h]) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_flatten_annotations(
    handle: *mut DocumentEditor,
    page: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.flatten_page_annotations(page as usize) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_flatten_all_annotations(
    handle: *mut DocumentEditor,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.flatten_all_annotations() {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_crop_margins(
    handle: *mut DocumentEditor,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.crop_margins(left, right, top, bottom) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

#[no_mangle]
pub extern "C" fn document_editor_merge_from(
    handle: *mut DocumentEditor,
    source_path: *const c_char,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || source_path.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    let path = match c_str(source_path) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    match editor.merge_from(path) {
        Ok(n) => {
            set_error(error_code, ERR_SUCCESS);
            n as i32
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

// ─── PDF Creation extras ────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn pdf_from_image(path: *const c_char, error_code: *mut i32) -> *mut Pdf {
    if path.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let p = match c_str(path) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    match Pdf::from_image(p) {
        Ok(pdf) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(pdf))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_from_image_bytes(
    data: *const u8,
    data_len: i32,
    error_code: *mut i32,
) -> *mut Pdf {
    if data.is_null() || data_len <= 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let bytes = raw_slice(data, data_len as usize);
    match Pdf::from_image_bytes(bytes) {
        Ok(pdf) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(pdf))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Merge multiple PDFs. paths is a null-terminated array of C strings.
#[no_mangle]
pub extern "C" fn pdf_merge(
    paths: *const *const c_char,
    path_count: i32,
    data_len: *mut i32,
    error_code: *mut i32,
) -> *mut u8 {
    if paths.is_null() || path_count <= 0 || data_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let mut path_strs: Vec<String> = Vec::new();
    for &p in raw_slice(paths, path_count as usize) {
        if p.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        match c_str(p) {
            Ok(s) => path_strs.push(s.to_string()),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            },
        }
    }
    let path_refs: Vec<&str> = path_strs.iter().map(|s| s.as_str()).collect();
    match crate::api::merge_pdfs(&path_refs) {
        Ok(bytes) => {
            set_error(error_code, ERR_SUCCESS);
            write_out(data_len, bytes.len() as i32);
            vec_to_ffi_bytes(bytes)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

// ─── PDF/A Validation ──────────────────────────────────────────────────────

use crate::compliance::{
    convert_to_pdf_a, validate_pdf_a, PdfALevel, ValidationResult as PdfAValidationResult,
};

pub struct FfiPdfAResults {
    result: PdfAValidationResult,
}

#[no_mangle]
pub extern "C" fn pdf_validate_pdf_a_level(
    document: *mut PdfDocument,
    level: i32,
    error_code: *mut i32,
) -> *mut FfiPdfAResults {
    if document.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_mut(document);
    let pdf_a_level = match level {
        0 => PdfALevel::A1b,
        1 => PdfALevel::A1a,
        2 => PdfALevel::A2b,
        3 => PdfALevel::A2a,
        4 => PdfALevel::A2u,
        5 => PdfALevel::A3b,
        6 => PdfALevel::A3a,
        7 => PdfALevel::A3u,
        _ => PdfALevel::A2b,
    };
    match validate_pdf_a(doc, pdf_a_level) {
        Ok(result) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiPdfAResults { result }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_convert_to_pdf_a(
    document: *mut PdfDocument,
    level: i32,
    error_code: *mut i32,
) -> bool {
    if document.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let pdf_level = match level {
        0 => PdfALevel::A1b,
        1 => PdfALevel::A1a,
        2 => PdfALevel::A2b,
        3 => PdfALevel::A2a,
        4 => PdfALevel::A2u,
        5 => PdfALevel::A3b,
        6 => PdfALevel::A3a,
        7 => PdfALevel::A3u,
        _ => {
            set_error(error_code, ERR_INVALID_ARG);
            return false;
        },
    };
    let doc = handle_mut(document);
    match convert_to_pdf_a(doc, pdf_level) {
        Ok(result) => {
            set_error(error_code, ERR_SUCCESS);
            result.success
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            false
        },
    }
}

/// Return a copy of the document's current source bytes (after any in-place
/// conversion). `out_len` receives the byte count. Free with `free_bytes`.
#[no_mangle]
pub extern "C" fn pdf_document_get_source_bytes(
    document: *mut PdfDocument,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    if document.is_null() || out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_mut(document);
    let bytes = doc.source_bytes.clone();
    set_error(error_code, ERR_SUCCESS);
    write_out(out_len, bytes.len());
    vec_to_ffi_bytes(bytes)
}

#[no_mangle]
pub extern "C" fn pdf_pdf_a_is_compliant(
    results: *const FfiPdfAResults,
    error_code: *mut i32,
) -> bool {
    if results.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    handle_ref(results).result.is_compliant
}

#[no_mangle]
pub extern "C" fn pdf_pdf_a_error_count(results: *const FfiPdfAResults) -> i32 {
    if results.is_null() {
        return 0;
    }
    handle_ref(results).result.errors.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_pdf_a_get_error(
    results: *const FfiPdfAResults,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if results.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let r = handle_ref(results);
    if (index as usize) >= r.result.errors.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&r.result.errors[index as usize].message)
}

#[no_mangle]
pub extern "C" fn pdf_pdf_a_warning_count(results: *const FfiPdfAResults) -> i32 {
    if results.is_null() {
        return 0;
    }
    handle_ref(results).result.warnings.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_pdf_a_results_free(results: *mut FfiPdfAResults) {
    if !results.is_null() {
        unsafe {
            drop(Box::from_raw(results));
        }
    }
}

// ─── PDF/X Validation ──────────────────────────────────────────────────────

use crate::compliance::pdf_x::{validate_pdf_x, PdfXLevel, XValidationResult};

pub struct FfiPdfXResults {
    result: XValidationResult,
}

#[no_mangle]
pub extern "C" fn pdf_validate_pdf_x_level(
    document: *mut PdfDocument,
    level: i32,
    error_code: *mut i32,
) -> *mut FfiPdfXResults {
    if document.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_mut(document);
    let x_level = match level {
        0 => PdfXLevel::X1a2001,
        1 => PdfXLevel::X32002,
        2 => PdfXLevel::X4,
        _ => PdfXLevel::X4,
    };
    match validate_pdf_x(doc, x_level) {
        Ok(result) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiPdfXResults { result }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_pdf_x_is_compliant(
    results: *const FfiPdfXResults,
    error_code: *mut i32,
) -> bool {
    if results.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    handle_ref(results).result.is_compliant
}

#[no_mangle]
pub extern "C" fn pdf_pdf_x_error_count(results: *const FfiPdfXResults) -> i32 {
    if results.is_null() {
        return 0;
    }
    handle_ref(results).result.errors.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_pdf_x_get_error(
    results: *const FfiPdfXResults,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if results.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let r = handle_ref(results);
    if (index as usize) >= r.result.errors.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return ptr::null_mut();
    }
    set_error(error_code, ERR_SUCCESS);
    to_c_string(&r.result.errors[index as usize].message)
}

#[no_mangle]
pub extern "C" fn pdf_pdf_x_results_free(results: *mut FfiPdfXResults) {
    if !results.is_null() {
        unsafe {
            drop(Box::from_raw(results));
        }
    }
}

// ─── Encryption ─────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn document_editor_save_encrypted(
    handle: *mut DocumentEditor,
    path: *const c_char,
    user_password: *const c_char,
    owner_password: *const c_char,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || path.is_null() || user_password.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    let path_str = match c_str(path) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    let user_pwd = match c_str(user_password) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    let owner_pwd = if owner_password.is_null() {
        user_pwd
    } else {
        match c_str(owner_password) {
            Ok(s) => s,
            Err(_) => user_pwd,
        }
    };
    let enc_config = crate::editor::EncryptionConfig::new(user_pwd, owner_pwd)
        .with_algorithm(crate::editor::EncryptionAlgorithm::Aes256);
    let save_opts = crate::editor::SaveOptions::with_encryption(enc_config);
    match editor.save_to_bytes_with_options(save_opts) {
        Ok(bytes) => match std::fs::write(path_str, &bytes) {
            Ok(()) => {
                set_error(error_code, ERR_SUCCESS);
                0
            },
            Err(_) => {
                set_error(error_code, ERR_IO);
                -1
            },
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

// ─── Extract paths ──────────────────────────────────────────────────────────

use crate::elements::PathContent;

pub struct FfiPathList {
    paths: Vec<PathContent>,
}

#[no_mangle]
pub extern "C" fn pdf_document_extract_paths(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut FfiPathList {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_paths(page_index as usize) {
        Ok(paths) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(FfiPathList { paths }))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[no_mangle]
pub extern "C" fn pdf_oxide_path_count(paths: *const FfiPathList) -> i32 {
    if paths.is_null() {
        return 0;
    }
    handle_ref(paths).paths.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_path_get_bbox(
    paths: *const FfiPathList,
    index: i32,
    x: *mut f32,
    y: *mut f32,
    w: *mut f32,
    h: *mut f32,
    error_code: *mut i32,
) {
    if paths.is_null() || index < 0 || x.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let list = handle_ref(paths);
    if (index as usize) >= list.paths.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    let b = &list.paths[index as usize].bbox;
    write_out(x, b.x);
    write_out(y, b.y);
    write_out(w, b.width);
    write_out(h, b.height);
    set_error(error_code, ERR_SUCCESS);
}

/// Rendered extents of the path: the geometric bbox inflated by the stroke
/// (ISO 32000-1:2008 section 8.4.3.2 — half the line width straddles each
/// side of the path). A table rule drawn as a ~1pt segment stroked as wide
/// as the table is tall reports the bar the reader sees instead of a 1x0pt
/// speck. Identical to pdf_oxide_path_get_bbox for unstroked
/// paths.
#[no_mangle]
pub extern "C" fn pdf_oxide_path_get_rendered_bbox(
    paths: *const FfiPathList,
    index: i32,
    x: *mut f32,
    y: *mut f32,
    w: *mut f32,
    h: *mut f32,
    error_code: *mut i32,
) {
    if paths.is_null() || index < 0 || x.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return;
    }
    let list = handle_ref(paths);
    if (index as usize) >= list.paths.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return;
    }
    let b = list.paths[index as usize].rendered_bbox();
    write_out(x, b.x);
    write_out(y, b.y);
    write_out(w, b.width);
    write_out(h, b.height);
    set_error(error_code, ERR_SUCCESS);
}

#[no_mangle]
pub extern "C" fn pdf_oxide_path_get_stroke_width(
    paths: *const FfiPathList,
    index: i32,
    error_code: *mut i32,
) -> f32 {
    if paths.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0.0;
    }
    let list = handle_ref(paths);
    if (index as usize) >= list.paths.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0.0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.paths[index as usize].stroke_width
}

#[no_mangle]
pub extern "C" fn pdf_oxide_path_has_stroke(
    paths: *const FfiPathList,
    index: i32,
    error_code: *mut i32,
) -> bool {
    if paths.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let list = handle_ref(paths);
    if (index as usize) >= list.paths.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    list.paths[index as usize].stroke_color.is_some()
}

#[no_mangle]
pub extern "C" fn pdf_oxide_path_has_fill(
    paths: *const FfiPathList,
    index: i32,
    error_code: *mut i32,
) -> bool {
    if paths.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return false;
    }
    let list = handle_ref(paths);
    if (index as usize) >= list.paths.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return false;
    }
    set_error(error_code, ERR_SUCCESS);
    list.paths[index as usize].fill_color.is_some()
}

#[no_mangle]
pub extern "C" fn pdf_oxide_path_get_operation_count(
    paths: *const FfiPathList,
    index: i32,
    error_code: *mut i32,
) -> i32 {
    if paths.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return 0;
    }
    let list = handle_ref(paths);
    if (index as usize) >= list.paths.len() {
        set_error(error_code, ERR_INVALID_PAGE);
        return 0;
    }
    set_error(error_code, ERR_SUCCESS);
    list.paths[index as usize].operations.len() as i32
}

#[no_mangle]
pub extern "C" fn pdf_oxide_path_list_free(handle: *mut FfiPathList) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ─── Page labels ────────────────────────────────────────────────────────────

use crate::extractors::PageLabelExtractor;

/// Get page labels as JSON: [{"start":0,"prefix":"","style":"Decimal","first":1}, ...]
#[no_mangle]
pub extern "C" fn pdf_document_get_page_labels(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match PageLabelExtractor::extract(doc) {
        Ok(ranges) => {
            let page_count = doc.page_count().unwrap_or(0);
            let labels = PageLabelExtractor::get_all_labels(&ranges, page_count);
            let json: Vec<String> = labels
                .iter()
                .enumerate()
                .map(|(i, l)| format!(r#"{{"page":{},"label":"{}"}}"#, i, l.replace('"', "\\\"")))
                .collect();
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&format!("[{}]", json.join(",")))
        },
        Err(_) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string("[]")
        },
    }
}

// ─── XMP Metadata ───────────────────────────────────────────────────────────

use crate::extractors::XmpExtractor;

/// Get XMP metadata as JSON
#[no_mangle]
pub extern "C" fn pdf_document_get_xmp_metadata(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match XmpExtractor::extract(doc) {
        Ok(Some(xmp)) => {
            let title = xmp.dc_title.as_deref().unwrap_or("").replace('"', "\\\"");
            let desc = xmp
                .dc_description
                .as_deref()
                .unwrap_or("")
                .replace('"', "\\\"");
            let creator_tool = xmp
                .xmp_creator_tool
                .as_deref()
                .unwrap_or("")
                .replace('"', "\\\"");
            let create_date = xmp
                .xmp_create_date
                .as_deref()
                .unwrap_or("")
                .replace('"', "\\\"");
            let modify_date = xmp
                .xmp_modify_date
                .as_deref()
                .unwrap_or("")
                .replace('"', "\\\"");
            let producer = xmp
                .pdf_producer
                .as_deref()
                .unwrap_or("")
                .replace('"', "\\\"");
            let creators: Vec<String> = xmp
                .dc_creator
                .iter()
                .map(|c| format!(r#""{}""#, c.replace('"', "\\\"")))
                .collect();
            let subjects: Vec<String> = xmp
                .dc_subject
                .iter()
                .map(|s| format!(r#""{}""#, s.replace('"', "\\\"")))
                .collect();
            let json = format!(
                r#"{{"title":"{}","description":"{}","creators":[{}],"subjects":[{}],"creatorTool":"{}","createDate":"{}","modifyDate":"{}","producer":"{}"}}"#,
                title,
                desc,
                creators.join(","),
                subjects.join(","),
                creator_tool,
                create_date,
                modify_date,
                producer
            );
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&json)
        },
        Ok(None) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string("{}")
        },
        Err(_) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string("{}")
        },
    }
}

// ─── Document outline ───────────────────────────────────────────────────────

fn outline_to_json(items: &[crate::outline::OutlineItem]) -> String {
    let json: Vec<String> = items
        .iter()
        .map(|item| {
            let dest = match &item.dest {
                Some(crate::outline::Destination::PageIndex(p)) => format!("{}", p),
                Some(crate::outline::Destination::Named(n)) => {
                    format!(r#""{}""#, n.replace('"', "\\\""))
                },
                None => "null".to_string(),
            };
            let children_json = if item.children.is_empty() {
                "[]".to_string()
            } else {
                outline_to_json(&item.children)
            };
            format!(
                r#"{{"title":"{}","dest":{},"children":{}}}"#,
                item.title.replace('"', "\\\""),
                dest,
                children_json
            )
        })
        .collect();
    format!("[{}]", json.join(","))
}

/// Get document outline (bookmarks) as JSON
#[no_mangle]
pub extern "C" fn pdf_document_get_outline(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.get_outline() {
        Ok(Some(items)) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&outline_to_json(&items))
        },
        Ok(None) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string("[]")
        },
        Err(_) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string("[]")
        },
    }
}

// ─── Split by bookmarks (#482) — JSON in, JSON out ─────────────────────────

#[derive(serde::Deserialize)]
struct FfiSplitOpts {
    #[serde(default)]
    title_prefix: Option<String>,
    #[serde(default)]
    ignore_case: bool,
    #[serde(default = "ffi_split_level_default")]
    level: u32,
    #[serde(default = "ffi_split_true")]
    include_front_matter: bool,
}
fn ffi_split_level_default() -> u32 {
    1
}
fn ffi_split_true() -> bool {
    true
}
impl Default for FfiSplitOpts {
    fn default() -> Self {
        Self {
            title_prefix: None,
            ignore_case: false,
            level: 1,
            include_front_matter: true,
        }
    }
}

/// Plan a bookmark split. `options_json` is `{}`-tolerant
/// (`title_prefix`, `ignore_case`, `level` [0=all,1=top],
/// `include_front_matter`). Returns a malloc'd JSON array of segment
/// objects (free via the existing string-free). On any failure sets
/// `error_code` (non-zero) and returns null — errors are surfaced, not
/// swallowed (foundation §3.3; this is not pure read-extraction).
#[no_mangle]
pub extern "C" fn pdf_document_plan_split_by_bookmarks(
    handle: *mut PdfDocument,
    options_json: *const c_char,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let opts_in: FfiSplitOpts = match c_str(options_json) {
        Ok(s) if !s.trim().is_empty() => match serde_json::from_str(s) {
            Ok(o) => o,
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            },
        },
        Ok(_) => FfiSplitOpts::default(),
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    let opts = crate::split_bookmarks::SplitByBookmarksOptions {
        title_prefix: opts_in.title_prefix,
        ignore_case: opts_in.ignore_case,
        level: crate::split_bookmarks::BookmarkLevel::from_u32(opts_in.level),
        include_front_matter: opts_in.include_front_matter,
        ..Default::default()
    };
    let doc = handle_ref(handle);
    match crate::split_bookmarks::plan_split_by_bookmarks(doc, &opts) {
        Ok(segs) => match serde_json::to_string(&segs) {
            Ok(j) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&j)
            },
            Err(e) => {
                set_error(
                    error_code,
                    classify_error(&crate::error::Error::InvalidOperation(e.to_string())),
                );
                ptr::null_mut()
            },
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

// ─── Comprehensive auto extraction (#517) ──────────────────────────────────
//
// JSON-string result at the C-ABI (matches the shipped split-by-bookmarks
// idiom —; the frozen parity contract for all bindings).
// Enums serialise as stable snake_case tokens. Read-only inspection;
// encrypted-unauthenticated fails closed (case L) via `classify_error`.

/// Cheap per-page text-vs-OCR classification (no OCR, no rasterisation).
/// Returns a malloc'd JSON `PageClassification`
/// (`{page,kind,confidence,reason,signals}`); free via the string-free.
/// On failure sets `error_code` non-zero and returns null.
#[no_mangle]
pub extern "C" fn pdf_document_classify_page(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.classify_page(page_index as usize) {
        Ok(c) => match serde_json::to_string(&c) {
            Ok(j) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&j)
            },
            Err(e) => {
                set_error(
                    error_code,
                    classify_error(&crate::error::Error::InvalidOperation(e.to_string())),
                );
                ptr::null_mut()
            },
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// The document's structured diagnostics as a malloc'd JSON array; free via
/// the string-free. `"[]"` when there are none.
///
/// Non-destructive: a later call returns the same entries plus any raised
/// since. Use `pdf_document_take_structured_warnings` to drain.
///
/// ```json
/// [{"category":"no_text_layer","page":0,
///   "message":"page 1 has no extractable text layer…","spec_section":null}]
/// ```
///
/// `category` is a stable snake_case token. **Consumers must tolerate tokens
/// they do not know** — categories are added in minor releases, so a binding
/// that models this as a closed enum turns a routine release into a
/// deserialisation failure for its users. Keep it a string, or give the enum
/// an unknown-value arm.
///
/// This is the channel the library reports *about* extraction on. Nothing is
/// written into the extracted content itself: a page with no text extracts as
/// nothing and says so here, carrying the page index, so a caller can decide
/// whether to surface it, where, and in what language.
#[no_mangle]
pub extern "C" fn pdf_document_structured_warnings(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match serde_json::to_string(&doc.structured_warnings()) {
        Ok(j) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&j)
        },
        Err(e) => {
            set_error(
                error_code,
                classify_error(&crate::error::Error::InvalidOperation(e.to_string())),
            );
            ptr::null_mut()
        },
    }
}

/// As `pdf_document_structured_warnings`, but drains: the returned entries are
/// removed, so a batch pipeline can read per document without the sink growing
/// across the run.
#[no_mangle]
pub extern "C" fn pdf_document_take_structured_warnings(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match serde_json::to_string(&doc.take_structured_warnings()) {
        Ok(j) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&j)
        },
        Err(e) => {
            set_error(
                error_code,
                classify_error(&crate::error::Error::InvalidOperation(e.to_string())),
            );
            ptr::null_mut()
        },
    }
}

/// Cheap whole-document classification (per-page kinds +
/// `pages_needing_ocr` + aggregate summary). Malloc'd JSON
/// `DocumentClassification`; free via the string-free.
#[no_mangle]
pub extern "C" fn pdf_document_classify_document(
    handle: *mut PdfDocument,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.classify_document() {
        Ok(c) => match serde_json::to_string(&c) {
            Ok(j) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&j)
            },
            Err(e) => {
                set_error(
                    error_code,
                    classify_error(&crate::error::Error::InvalidOperation(e.to_string())),
                );
                ptr::null_mut()
            },
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// One-shot auto text extraction (`AutoExtractor::new()`): per-page,
/// auto-routed text-vs-OCR with graceful native fallback (never the
/// opaque OCR error — #513). Malloc'd UTF-8 string; free via the
/// string-free.
#[no_mangle]
pub extern "C" fn pdf_document_extract_text_auto(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let doc = handle_ref(handle);
    match doc.extract_text_auto(page_index as usize) {
        Ok(text) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&text)
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Rich per-page extraction. `options_json` is `{}`-tolerant
/// (`AutoExtractOptions`: `mode`, `reconstruct_image_tables`,
/// `emit_placeholders`, `ocr_languages`, `min_text_confidence`,
/// `table_confidence`, `force_ocr_pages`); NULL/empty → defaults.
/// Returns a malloc'd JSON `PageExtraction` (per-region bbox + typed
/// reason — never bare-empty). Free via the string-free.
#[no_mangle]
pub extern "C" fn pdf_document_extract_page_auto(
    handle: *mut PdfDocument,
    page_index: i32,
    options_json: *const c_char,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    // Per the doc contract a NULL or empty `options_json` selects the
    // defaults. `c_str` returns `Err` on NULL, so handle NULL the same
    // as an empty string instead of failing with ERR_INVALID_ARG
    // (PR #519 review).
    let opts: crate::extractors::auto::AutoExtractOptions = if options_json.is_null() {
        crate::extractors::auto::AutoExtractOptions::default()
    } else {
        match c_str(options_json) {
            Ok(s) if !s.trim().is_empty() => match serde_json::from_str(s) {
                Ok(o) => o,
                Err(_) => {
                    set_error(error_code, ERR_INVALID_ARG);
                    return ptr::null_mut();
                },
            },
            Ok(_) => crate::extractors::auto::AutoExtractOptions::default(),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            },
        }
    };
    let doc = handle_ref(handle);
    let ae = crate::extractors::auto::AutoExtractor::with(opts);
    match ae.extract_page(doc, page_index as usize) {
        Ok(pe) => match serde_json::to_string(&pe) {
            Ok(j) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&j)
            },
            Err(e) => {
                set_error(
                    error_code,
                    classify_error(&crate::error::Error::InvalidOperation(e.to_string())),
                );
                ptr::null_mut()
            },
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Build-time OCR model provisioning (#519, cross-binding). Downloads
/// the shared detector + the recognition model/dict for each requested
/// language into the model cache dir (`$PDF_OXIDE_MODEL_DIR` / the
/// platform cache) and returns that dir as a malloc'd UTF-8 string
/// (free via `free_string`). `languages_csv` is comma-separated
/// language codes (e.g. `"english,chinese,arabic"`); NULL/empty →
/// English. Unknown codes are skipped. Idempotent. Actual download
/// requires the library built with the `ocr` feature; without it the
/// cache dir is still created (no fetch) — query
/// `pdf_oxide_prefetch_available()`.
#[no_mangle]
pub extern "C" fn pdf_oxide_prefetch_models(
    languages_csv: *const c_char,
    error_code: *mut i32,
) -> *mut c_char {
    use crate::extractors::auto::{AutoExtractor, OcrLanguage};
    let langs: Vec<OcrLanguage> = match c_str(languages_csv) {
        Ok(s) if !s.trim().is_empty() => s
            .split(',')
            .filter_map(|t| OcrLanguage::from_code(t.trim()))
            .collect(),
        _ => Vec::new(),
    };
    let want = if langs.is_empty() {
        vec![OcrLanguage::English]
    } else {
        langs
    };
    match AutoExtractor::prefetch_models(&want) {
        Ok(dir) => {
            set_error(error_code, ERR_SUCCESS);
            to_c_string(&dir.to_string_lossy())
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Air-gapped OCR model manifest — JSON (detector + every supported
/// language's cache filenames and source URLs). Malloc'd; free via
/// `free_string`. Never errors.
#[no_mangle]
pub extern "C" fn pdf_oxide_model_manifest() -> *mut c_char {
    to_c_string(&crate::extractors::auto::AutoExtractor::model_manifest())
}

/// Whether this build can actually download models (compiled with the
/// `ocr` feature). `1` = downloads work; `0` =
/// `pdf_oxide_prefetch_models` only creates the cache dir (no fetch).
#[no_mangle]
pub extern "C" fn pdf_oxide_prefetch_available() -> i32 {
    i32::from(crate::extractors::auto::AutoExtractor::prefetch_available())
}

// ─── Form field mutation (via DocumentEditor) ──────────────────────────────

/// Set a form field value on a DocumentEditor. Value is a UTF-8 string.
#[no_mangle]
pub extern "C" fn document_editor_set_form_field_value(
    handle: *mut DocumentEditor,
    name: *const c_char,
    value: *const c_char,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() || name.is_null() || value.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    let field_name = match c_str(name) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    let field_value = match c_str(value) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    match editor.set_form_field_value(
        field_name,
        crate::editor::form_fields::FormFieldValue::Text(field_value.to_string()),
    ) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Flatten all forms in the document (bake form values into page content).
#[no_mangle]
pub extern "C" fn document_editor_flatten_forms(
    handle: *mut DocumentEditor,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.flatten_forms() {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Flatten forms on a specific page.
#[no_mangle]
pub extern "C" fn document_editor_flatten_forms_on_page(
    handle: *mut DocumentEditor,
    page_index: i32,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let editor = handle_mut(handle);
    match editor.flatten_forms_on_page(page_index as usize) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Return the number of warnings collected during the last form-flattening save.
///
/// Each warning names a widget field that had no `/AP` appearance stream.
/// Returns -1 if `handle` is null.
#[no_mangle]
pub extern "C" fn document_editor_flatten_warnings_count(handle: *const DocumentEditor) -> i32 {
    if handle.is_null() {
        return -1;
    }
    let editor = handle_ref(handle);
    editor.flatten_warnings().len() as i32
}

/// Return the `index`-th flatten warning as a C string owned by the caller.
///
/// The returned string must be freed with `free_string`. Returns null if
/// `handle` is null or `index` is out of range.
#[no_mangle]
pub extern "C" fn document_editor_flatten_warning(
    handle: *const DocumentEditor,
    index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    if handle.is_null() || index < 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return std::ptr::null_mut();
    }
    let editor = handle_ref(handle);
    let warnings = editor.flatten_warnings();
    match warnings.get(index as usize) {
        Some(w) => match CString::new(w.as_str()) {
            Ok(cs) => {
                set_error(error_code, ERR_SUCCESS);
                cs.into_raw()
            },
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                std::ptr::null_mut()
            },
        },
        None => {
            set_error(error_code, ERR_INVALID_ARG);
            std::ptr::null_mut()
        },
    }
}

// ─── C# compatibility aliases ──────────────────────────────────────────────
// C# P/Invoke uses PascalCase names. These re-export the snake_case functions.

#[no_mangle]
pub extern "C" fn PdfDocumentOpen(path: *const c_char, error_code: *mut i32) -> *mut PdfDocument {
    pdf_document_open(path, error_code)
}

#[no_mangle]
pub extern "C" fn PdfDocumentFree(handle: *mut PdfDocument) {
    pdf_document_free(handle)
}

#[no_mangle]
pub extern "C" fn PdfDocumentGetPageCount(handle: *mut PdfDocument, error_code: *mut i32) -> i32 {
    pdf_document_get_page_count(handle, error_code)
}

#[no_mangle]
pub extern "C" fn PdfDocumentExtractText(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    pdf_document_extract_text(handle, page_index, error_code)
}

#[no_mangle]
pub extern "C" fn PdfDocumentToMarkdown(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    pdf_document_to_markdown(handle, page_index, error_code)
}

#[no_mangle]
pub extern "C" fn PdfDocumentToHtml(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    pdf_document_to_html(handle, page_index, error_code)
}

#[no_mangle]
pub extern "C" fn PdfDocumentToPlainText(
    handle: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> *mut c_char {
    pdf_document_to_plain_text(handle, page_index, error_code)
}

#[no_mangle]
pub extern "C" fn PdfFromMarkdown(markdown: *const c_char, error_code: *mut i32) -> *mut Pdf {
    pdf_from_markdown(markdown, error_code)
}

#[no_mangle]
pub extern "C" fn PdfFromHtml(html: *const c_char, error_code: *mut i32) -> *mut Pdf {
    pdf_from_html(html, error_code)
}

#[no_mangle]
pub extern "C" fn PdfFromText(text: *const c_char, error_code: *mut i32) -> *mut Pdf {
    pdf_from_text(text, error_code)
}

#[no_mangle]
pub extern "C" fn PdfSave(handle: *mut Pdf, path: *const c_char, error_code: *mut i32) -> i32 {
    pdf_save(handle, path, error_code)
}

#[no_mangle]
pub extern "C" fn PdfSaveToBytes(
    handle: *mut Pdf,
    data_len: *mut i32,
    error_code: *mut i32,
) -> *mut u8 {
    pdf_save_to_bytes(handle, data_len, error_code)
}

#[no_mangle]
pub extern "C" fn PdfFree(handle: *mut Pdf) {
    pdf_free(handle)
}

#[no_mangle]
pub extern "C" fn DocumentEditorOpen(
    path: *const c_char,
    error_code: *mut i32,
) -> *mut DocumentEditor {
    document_editor_open(path, error_code)
}

#[no_mangle]
pub extern "C" fn DocumentEditorFree(handle: *mut DocumentEditor) {
    document_editor_free(handle)
}

#[no_mangle]
pub extern "C" fn DocumentEditorSave(
    handle: *mut DocumentEditor,
    path: *const c_char,
    error_code: *mut i32,
) -> i32 {
    document_editor_save(handle, path, error_code)
}

#[no_mangle]
pub extern "C" fn DocumentEditorSetTitle(
    handle: *mut DocumentEditor,
    value: *const c_char,
    error_code: *mut i32,
) -> i32 {
    document_editor_set_title(handle, value, error_code)
}

#[no_mangle]
pub extern "C" fn DocumentEditorSetAuthor(
    handle: *mut DocumentEditor,
    value: *const c_char,
    error_code: *mut i32,
) -> i32 {
    document_editor_set_author(handle, value, error_code)
}

#[no_mangle]
pub extern "C" fn FreeString(ptr: *mut c_char) {
    free_string(ptr)
}

#[no_mangle]
pub extern "C" fn FreeBytes(ptr: *mut u8) {
    free_bytes(ptr)
}

#[no_mangle]
pub extern "C" fn AllocString(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    match c_str(s).ok().and_then(|s| CString::new(s).ok()) {
        Some(cs) => cs.into_raw(),
        None => ptr::null_mut(),
    }
}

// ─── JSON bulk extractors ───────────────────────────────────────────────────
//
// These functions serialize an entire list handle to a single JSON string so
// that language bindings can make one FFI crossing per list instead of N*M
// per-field calls. The returned pointer is a UTF-8 C string that the caller
// must free with `free_string()`.
//
// The JSON schemas are stable contracts — downstream bindings depend on them.
// Field names use lowerCamelCase to match the idioms of the primary consumer
// languages (Go, JS, C#).
//
// Image data is NOT exposed via JSON because base64 overhead makes it worse
// than the per-item binary extraction path already available.

use serde::Serialize;

#[derive(Serialize)]
struct JsonFont<'a> {
    name: &'a str,
    r#type: &'a str,
    encoding: &'a str,
    isEmbedded: bool,
    isSubset: bool,
    size: f32,
}

#[derive(Serialize)]
struct JsonAnnotation<'a> {
    r#type: &'a str,
    subtype: &'a str,
    content: &'a str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    author: &'a str,
    borderWidth: f32,
    color: u32,
    creationDate: i64,
    modificationDate: i64,
    linkURI: &'a str,
    textIconName: &'a str,
    isHidden: bool,
    isPrintable: bool,
    isReadOnly: bool,
    isMarkedDeleted: bool,
}

#[derive(Serialize)]
struct JsonElement<'a> {
    r#type: &'a str,
    text: &'a str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    /// Page-space extents: the x/y/width/height rect with any text-matrix
    /// rotation resolved into an axis-aligned page-space hull. Identical to
    /// it for upright runs.
    #[serde(rename = "pageX")]
    page_x: f32,
    #[serde(rename = "pageY")]
    page_y: f32,
    #[serde(rename = "pageWidth")]
    page_width: f32,
    #[serde(rename = "pageHeight")]
    page_height: f32,
    /// §9.10.2 mapping-provenance label; omitted when the font is unresolved.
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<&'static str>,
}

#[derive(Serialize)]
struct JsonSearchResult<'a> {
    text: &'a str,
    page: i32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn string_to_c(s: String) -> *mut c_char {
    CString::new(s)
        .map(|c| c.into_raw())
        .unwrap_or(ptr::null_mut())
}

/// Serializes an entire font list to JSON. Returns a UTF-8 C string owned by
/// the caller (free with `free_string`). The schema is
/// `[{"name": "...", "type": "...", "encoding": "...", "isEmbedded": bool,
/// "isSubset": bool, "size": number}, ...]`.
#[no_mangle]
pub extern "C" fn pdf_oxide_fonts_to_json(
    fonts: *const FfiFontList,
    error_code: *mut i32,
) -> *mut c_char {
    if fonts.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(fonts);
    let items: Vec<JsonFont> = list
        .fonts
        .iter()
        .map(|f| JsonFont {
            name: &f.name,
            r#type: &f.subtype,
            encoding: &f.encoding,
            isEmbedded: f.is_embedded,
            isSubset: f.is_subset,
            size: 0.0,
        })
        .collect();
    match serde_json::to_string(&items) {
        Ok(s) => {
            set_error(error_code, ERR_SUCCESS);
            string_to_c(s)
        },
        Err(_) => {
            set_error(error_code, ERR_INTERNAL);
            ptr::null_mut()
        },
    }
}

/// Serializes an entire annotation list to JSON. Returns a UTF-8 C string
/// owned by the caller (free with `free_string`). The schema matches the Go
/// `Annotation` struct fields.
#[no_mangle]
pub extern "C" fn pdf_oxide_annotations_to_json(
    annotations: *const FfiAnnotationList,
    error_code: *mut i32,
) -> *mut c_char {
    if annotations.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(annotations);

    // Pre-compute owned strings so the closure can borrow them.
    let rows: Vec<(
        String,
        String,
        String,
        f32,
        f32,
        f32,
        f32,
        String,
        f32,
        u32,
        i64,
        i64,
        String,
        String,
        bool,
        bool,
        bool,
        bool,
    )> = list
        .annotations
        .iter()
        .map(|a| {
            let rect = a.rect.unwrap_or([0.0, 0.0, 0.0, 0.0]);
            let (x, y, w, h) = (
                rect[0] as f32,
                rect[1] as f32,
                (rect[2] - rect[0]) as f32,
                (rect[3] - rect[1]) as f32,
            );
            (
                a.annotation_type.clone(),
                a.subtype.clone().unwrap_or_default(),
                a.contents.clone().unwrap_or_default(),
                x,
                y,
                w,
                h,
                a.author.clone().unwrap_or_default(),
                a.border.map(|b| b[2] as f32).unwrap_or(0.0),
                0,             // Color — deferred; RustAnnotation stores Vec<f64>
                0,             // CreationDate — string in Rust, 0 until parsed
                0,             // ModificationDate — same
                String::new(), // LinkURI — deferred
                String::new(), // TextIconName — deferred
                false,
                false,
                false,
                false, // flags — extracted from a.flags if needed
            )
        })
        .collect();

    let items: Vec<JsonAnnotation> = rows
        .iter()
        .map(|r| JsonAnnotation {
            r#type: &r.0,
            subtype: &r.1,
            content: &r.2,
            x: r.3,
            y: r.4,
            width: r.5,
            height: r.6,
            author: &r.7,
            borderWidth: r.8,
            color: r.9,
            creationDate: r.10,
            modificationDate: r.11,
            linkURI: &r.12,
            textIconName: &r.13,
            isHidden: r.14,
            isPrintable: r.15,
            isReadOnly: r.16,
            isMarkedDeleted: r.17,
        })
        .collect();

    match serde_json::to_string(&items) {
        Ok(s) => {
            set_error(error_code, ERR_SUCCESS);
            string_to_c(s)
        },
        Err(_) => {
            set_error(error_code, ERR_INTERNAL);
            ptr::null_mut()
        },
    }
}

/// Serializes an entire element list (text spans) to JSON. Returns a UTF-8 C
/// string owned by the caller (free with `free_string`).
#[no_mangle]
pub extern "C" fn pdf_oxide_elements_to_json(
    elements: *const FfiElementList,
    error_code: *mut i32,
) -> *mut c_char {
    if elements.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(elements);
    let items: Vec<JsonElement> = list
        .spans
        .iter()
        .map(|s| {
            let pb = s.page_bbox();
            JsonElement {
                r#type: "text",
                text: &s.text,
                x: s.bbox.x,
                y: s.bbox.y,
                width: s.bbox.width,
                height: s.bbox.height,
                page_x: pb.x,
                page_y: pb.y,
                page_width: pb.width,
                page_height: pb.height,
                provenance: s.provenance.map(|p| p.as_str()),
            }
        })
        .collect();
    match serde_json::to_string(&items) {
        Ok(s) => {
            set_error(error_code, ERR_SUCCESS);
            string_to_c(s)
        },
        Err(_) => {
            set_error(error_code, ERR_INTERNAL);
            ptr::null_mut()
        },
    }
}

/// Serializes an entire search-results list to JSON. Returns a UTF-8 C string
/// owned by the caller (free with `free_string`).
#[no_mangle]
pub extern "C" fn pdf_oxide_search_results_to_json(
    results: *const FfiSearchResults,
    error_code: *mut i32,
) -> *mut c_char {
    if results.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let list = handle_ref(results);
    let items: Vec<JsonSearchResult> = list
        .results
        .iter()
        .map(|r| JsonSearchResult {
            text: &r.text,
            page: r.page as i32,
            x: r.bbox.x,
            y: r.bbox.y,
            width: r.bbox.width,
            height: r.bbox.height,
        })
        .collect();
    match serde_json::to_string(&items) {
        Ok(s) => {
            set_error(error_code, ERR_SUCCESS);
            string_to_c(s)
        },
        Err(_) => {
            set_error(error_code, ERR_INTERNAL);
            ptr::null_mut()
        },
    }
}

// ─── OCR ────────────────────────────────────────────────────────────────────

/// Create an OCR engine from model/dictionary file paths.
/// Returns an opaque handle (`Box<OcrEngine>`) that must be freed with
/// `pdf_ocr_engine_free`. On failure returns null and sets `error_code`.
#[no_mangle]
pub extern "C" fn pdf_ocr_engine_create(
    det_model_path: *const c_char,
    rec_model_path: *const c_char,
    dict_path: *const c_char,
    error_code: *mut i32,
) -> *mut std::ffi::c_void {
    #[cfg(feature = "ocr")]
    {
        use crate::ocr::{OcrConfig, OcrEngine};

        if det_model_path.is_null() || rec_model_path.is_null() || dict_path.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let det = match c_str(det_model_path) {
            Ok(s) => s,
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            },
        };
        let rec = match c_str(rec_model_path) {
            Ok(s) => s,
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            },
        };
        let dict = match c_str(dict_path) {
            Ok(s) => s,
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            },
        };
        match OcrEngine::new(det, rec, dict, OcrConfig::default()) {
            Ok(engine) => {
                set_error(error_code, ERR_SUCCESS);
                Box::into_raw(Box::new(engine)) as *mut std::ffi::c_void
            },
            Err(_e) => {
                set_error(error_code, ERR_INTERNAL);
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "ocr"))]
    {
        let _ = (det_model_path, rec_model_path, dict_path);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

/// Free an OCR engine handle created by `pdf_ocr_engine_create`.
#[no_mangle]
pub extern "C" fn pdf_ocr_engine_free(engine: *mut std::ffi::c_void) {
    #[cfg(feature = "ocr")]
    {
        use crate::ocr::OcrEngine;
        if !engine.is_null() {
            unsafe {
                drop(Box::from_raw(engine as *mut OcrEngine));
            }
        }
    }
    #[cfg(not(feature = "ocr"))]
    {
        let _ = engine;
    }
}

/// Check whether a page needs OCR (i.e. is scanned/hybrid).
/// Returns false and sets `error_code` to `_ERR_UNSUPPORTED` when the `ocr`
/// feature is disabled.
#[no_mangle]
pub extern "C" fn pdf_ocr_page_needs_ocr(
    doc: *mut PdfDocument,
    page_index: i32,
    error_code: *mut i32,
) -> bool {
    #[cfg(feature = "ocr")]
    {
        if doc.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return false;
        }
        let d = handle_mut(doc);
        match crate::ocr::needs_ocr(d, page_index as usize) {
            Ok(v) => {
                set_error(error_code, ERR_SUCCESS);
                v
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                false
            },
        }
    }
    #[cfg(not(feature = "ocr"))]
    {
        let _ = (doc, page_index);
        set_error(error_code, _ERR_UNSUPPORTED);
        false
    }
}

/// Extract text from a page using OCR. `engine` may be null (will use native
/// text extraction only). The returned C string must be freed with `free_string`.
#[no_mangle]
pub extern "C" fn pdf_ocr_extract_text(
    doc: *mut PdfDocument,
    page_index: i32,
    engine: *const std::ffi::c_void,
    error_code: *mut i32,
) -> *mut c_char {
    #[cfg(feature = "ocr")]
    {
        use crate::ocr::{OcrEngine, OcrExtractOptions};

        if doc.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let d = handle_mut(doc);
        let ocr_engine: Option<&OcrEngine> = if engine.is_null() {
            None
        } else {
            Some(handle_ref(engine as *const OcrEngine))
        };
        match crate::ocr::extract_text_with_ocr(
            d,
            page_index as usize,
            ocr_engine,
            OcrExtractOptions::default(),
        ) {
            Ok(text) => {
                set_error(error_code, ERR_SUCCESS);
                to_c_string(&text)
            },
            Err(e) => {
                set_error(error_code, classify_error(&e));
                ptr::null_mut()
            },
        }
    }
    #[cfg(not(feature = "ocr"))]
    {
        let _ = (doc, page_index, engine);
        set_error(error_code, _ERR_UNSUPPORTED);
        ptr::null_mut()
    }
}

// =============================================================================
// Write-side API: EmbeddedFont / DocumentBuilder / PageBuilder
// =============================================================================
//
// C-FFI mirror of the pyo3 / wasm-bindgen exposure, with FFI-appropriate
// idioms: opaque handles, out-parameter error codes, explicit free
// functions, no fluent chaining (each method returns an int status, not
// a handle, so C / C# / Go / Node wrappers rebuild fluency above this).
//
// **Handle-lifetime contract** — read carefully before wrapping this FFI:
//
//   1. `pdf_document_builder_create` returns a builder handle. The
//      caller owns it until one of
//        * `pdf_document_builder_free`
//        * `pdf_document_builder_save` / `_save_encrypted`
//        * `pdf_document_builder_build` / `_to_bytes_encrypted`
//      Each terminal method **consumes** the handle — subsequent use
//      is undefined behaviour.
//
//   2. `pdf_document_builder_a4_page` / `_letter_page` / `_page`
//      returns a page handle. While any page handle is outstanding,
//      its parent builder's open-page slot is held; a second
//      `pdf_document_builder_*_page` call on the same builder before
//      the prior `pdf_page_builder_done` returns `ERR_INVALID_ARG (1)`.
//
//  3. `pdf_page_builder_done` commits the buffered operations
//      clears the parent's open-page slot. The page handle becomes
//      invalid. `pdf_page_builder_free` drops without committing
//      (error recovery only).
//
//   4. `pdf_document_builder_register_embedded_font` **consumes** the
//      `font` handle. Caller must NOT call `pdf_embedded_font_free`
//      afterward.
//
//   5. Byte buffers returned from `_build` / `_to_bytes_encrypted`
//      must be freed with `free_bytes`.

const _ERR_FONT: i32 = 9; // reserved for future fine-grained font errors

/// FFI wrapper around [`crate::writer::DocumentBuilder`] that adds a
/// reentrancy guard around the open-page slot.
pub struct FfiDocumentBuilder {
    inner: Option<crate::writer::DocumentBuilder>,
    open_page: bool,
}

/// Buffered page operations that are replayed against a real Rust
/// `FluentPageBuilder` on `pdf_page_builder_done`.
enum FfiPageOp {
    Font(String, f32),
    At(f32, f32),
    Text(String),
    Heading(u8, String),
    Paragraph(String),
    Space(f32),
    HorizontalRule,
    LinkUrl(String),
    LinkPage(usize),
    LinkNamed(String),
    Highlight(f32, f32, f32),
    Underline(f32, f32, f32),
    Strikeout(f32, f32, f32),
    Squiggly(f32, f32, f32),
    StickyNote(String),
    StickyNoteAt(f32, f32, String),
    Watermark(String),
    WatermarkConfidential,
    WatermarkDraft,
    Stamp(String),
    FreeText {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: String,
    },
    TextField {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        default_value: Option<String>,
    },
    Checkbox {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        checked: bool,
    },
    ComboBox {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        options: Vec<String>,
        selected: Option<String>,
    },
    RadioGroup {
        name: String,
        buttons: Vec<(String, f32, f32, f32, f32)>,
        selected: Option<String>,
    },
    PushButton {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        caption: String,
    },
    SignatureField {
        name: String,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    Footnote {
        ref_mark: String,
        note_text: String,
    },
    Columns {
        count: u32,
        gap_pt: f32,
        text: String,
    },
    Inline(String),
    InlineBold(String),
    InlineItalic(String),
    InlineColor {
        r: f32,
        g: f32,
        b: f32,
        text: String,
    },
    Newline,
    Rect(f32, f32, f32, f32),
    FilledRect(f32, f32, f32, f32, f32, f32, f32),
    Line(f32, f32, f32, f32),
    StrokeRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
    },
    StrokeRectDashed {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
        dash: Vec<f32>,
        phase: f32,
    },
    StrokeLine {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
    },
    StrokeLineDashed {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        r: f32,
        g: f32,
        b: f32,
        dash: Vec<f32>,
        phase: f32,
    },
    TextInRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text: String,
        align: i32, // 0=Left 1=Center 2=Right
    },
    NewPageSameSize,
    /// Buffered `Table` — n_cells = n_rows × n_cols. Cells are stored as
    /// a flat row-major `Vec<String>`; aligns is per column.
    Table {
        widths: Vec<f32>,
        aligns: Vec<i32>, // per column: 0=Left 1=Center 2=Right
        rows: Vec<Vec<String>>,
        has_header: bool,
    },
    /// Open a streaming table. Pending ops from this point until a
    /// matching `StreamingTableFinish` are fed into the live
    /// `StreamingTable` on replay, giving true row-at-a-time drawing.
    StreamingTableOpen {
        /// Per-column (header, width_pt, align) — align is 0/1/2.
        columns: Vec<(String, f32, i32)>,
        repeat_header: bool,
        /// 0=fixed, 1=sample, 2=auto_all
        mode: i32,
        sample_rows: usize,
        min_col_width_pt: f32,
        max_col_width_pt: f32,
        /// Maximum rowspan. Default 1 (rowspan disabled).
        max_rowspan: usize,
    },
    /// Push one row into the currently-open streaming table. Each cell
    /// carries (text, rowspan); rowspan == 1 is a normal cell.
    StreamingTableRow(Vec<(String, usize)>),
    /// Close the current streaming table. Subsequent ops are normal
    /// page ops again.
    StreamingTableFinish,
    /// Pre-rendered barcode PNG bytes (generated at record time so errors
    /// surface at the caller, not during replay).
    BarcodeImage {
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    /// Plain image embed — no alt text, no /Artifact wrapper.
    Image {
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    /// Image with accessibility alt text (PDF/UA-1 /Figure).
    ImageWithAlt {
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        alt_text: String,
    },
    /// Decorative image — no alt text, wrapped in /Artifact (PDF/UA-1).
    ImageArtifact {
        bytes: Vec<u8>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    /// JavaScript link annotation on the last text element.
    LinkJavaScript(String),
    /// JavaScript to run when the page is opened (`/AA /O`).
    OnOpen(String),
    /// JavaScript to run when the page is closed (`/AA /C`).
    OnClose(String),
    /// /AA /K keystroke JS on the last form field.
    FieldKeystroke(String),
    /// /AA /F format JS on the last form field.
    FieldFormat(String),
    /// /AA /V validate JS on the last form field.
    FieldValidate(String),
    /// /AA /C calculate JS on the last form field.
    FieldCalculate(String),
}

/// Parse a stamp-type name into the Rust `StampType` enum. Unknown names
/// become `StampType::Custom`.
fn ffi_parse_stamp_type(name: &str) -> crate::writer::StampType {
    use crate::writer::StampType;
    match name {
        "Approved" => StampType::Approved,
        "Experimental" => StampType::Experimental,
        "NotApproved" => StampType::NotApproved,
        "AsIs" => StampType::AsIs,
        "Expired" => StampType::Expired,
        "NotForPublicRelease" => StampType::NotForPublicRelease,
        "Confidential" => StampType::Confidential,
        "Final" => StampType::Final,
        "Sold" => StampType::Sold,
        "Departmental" => StampType::Departmental,
        "ForComment" => StampType::ForComment,
        "TopSecret" => StampType::TopSecret,
        "Draft" => StampType::Draft,
        "ForPublicRelease" => StampType::ForPublicRelease,
        other => StampType::Custom(other.to_string()),
    }
}

/// Page sub-handle. Holds a raw back-pointer to the parent; the parent
/// must outlive this handle (enforced by the one-page-at-a-time
/// invariant documented above).
pub struct FfiPageBuilder {
    parent: *mut FfiDocumentBuilder,
    page_size: Option<crate::writer::PageSize>,
    custom_width: f32,
    custom_height: f32,
    ops: Vec<FfiPageOp>,
    done_called: bool,
    // Streaming-table batch tracking (valid while a table is open).
    // Auto-flush triggers when pending reaches batch_size (0 = disabled).
    st_batch_size: usize,
    st_batch_count: usize,
    st_pending_count: usize,
}

// ───────────────────────────────────────────────────────────────────────────
// EmbeddedFont
// ───────────────────────────────────────────────────────────────────────────

/// Load a TTF / OTF font from a file path. Returns an opaque handle or
/// NULL on error.
#[no_mangle]
pub extern "C" fn pdf_embedded_font_from_file(
    path: *const c_char,
    error_code: *mut i32,
) -> *mut crate::writer::EmbeddedFont {
    if path.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let path_str = match c_str(path) {
        Ok(s) => s,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        },
    };
    match crate::writer::EmbeddedFont::from_file(path_str) {
        Ok(font) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(font))
        },
        Err(_) => {
            set_error(error_code, ERR_IO);
            ptr::null_mut()
        },
    }
}

/// Load a font from a byte buffer. `name` may be NULL to use the
/// PostScript name from the font face.
#[no_mangle]
pub extern "C" fn pdf_embedded_font_from_bytes(
    data: *const u8,
    len: usize,
    name: *const c_char,
    error_code: *mut i32,
) -> *mut crate::writer::EmbeddedFont {
    if data.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let bytes = raw_slice(data, len).to_vec();
    let name_opt = if name.is_null() {
        None
    } else {
        match c_str(name) {
            Ok(s) => Some(s.to_string()),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            },
        }
    };
    match crate::writer::EmbeddedFont::from_data(name_opt, bytes) {
        Ok(font) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(font))
        },
        Err(_) => {
            set_error(error_code, ERR_PARSE);
            ptr::null_mut()
        },
    }
}

/// Free an `EmbeddedFont` handle. No-op on NULL. Do not call after
/// a successful `pdf_document_builder_register_embedded_font` —
/// the builder has taken ownership.
#[no_mangle]
pub extern "C" fn pdf_embedded_font_free(handle: *mut crate::writer::EmbeddedFont) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// DocumentBuilder
// ───────────────────────────────────────────────────────────────────────────

/// Create a new `DocumentBuilder`. Never fails but keeps the
/// error_code signature for uniformity.
#[no_mangle]
pub extern "C" fn pdf_document_builder_create(error_code: *mut i32) -> *mut FfiDocumentBuilder {
    set_error(error_code, ERR_SUCCESS);
    Box::into_raw(Box::new(FfiDocumentBuilder {
        inner: Some(crate::writer::DocumentBuilder::new()),
        open_page: false,
    }))
}

/// Free a `DocumentBuilder` handle without building. Safe to call on
/// an already-consumed handle — it'll just drop the (empty) wrapper.
#[no_mangle]
pub extern "C" fn pdf_document_builder_free(handle: *mut FfiDocumentBuilder) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

fn ffi_builder_mut<'a>(
    handle: *mut FfiDocumentBuilder,
    error_code: *mut i32,
) -> Option<&'a mut FfiDocumentBuilder> {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return None;
    }
    let wrapper = handle_mut(handle);
    if wrapper.inner.is_none() {
        set_error(error_code, ERR_INVALID_ARG);
        return None;
    }
    Some(wrapper)
}

fn ffi_builder_apply<F>(handle: *mut FfiDocumentBuilder, error_code: *mut i32, f: F) -> i32
where
    F: FnOnce(crate::writer::DocumentBuilder) -> crate::writer::DocumentBuilder,
{
    let Some(wrapper) = ffi_builder_mut(handle, error_code) else {
        return -1;
    };
    if wrapper.open_page {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let taken = wrapper.inner.take().unwrap();
    wrapper.inner = Some(f(taken));
    set_error(error_code, ERR_SUCCESS);
    0
}

fn read_cstr_or_fail(ptr: *const c_char, error_code: *mut i32) -> Option<String> {
    if ptr.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return None;
    }
    match c_str(ptr) {
        Ok(s) => Some(s.to_string()),
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            None
        },
    }
}

/// Set the document title. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn pdf_document_builder_set_title(
    handle: *mut FfiDocumentBuilder,
    title: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(title) = read_cstr_or_fail(title, error_code) else {
        return -1;
    };
    ffi_builder_apply(handle, error_code, |b| b.title(title))
}

/// Set the document author. Returns 0 on success, -1 on error.
#[no_mangle]
pub extern "C" fn pdf_document_builder_set_author(
    handle: *mut FfiDocumentBuilder,
    author: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(author) = read_cstr_or_fail(author, error_code) else {
        return -1;
    };
    ffi_builder_apply(handle, error_code, |b| b.author(author))
}

/// Set the document subject.
#[no_mangle]
pub extern "C" fn pdf_document_builder_set_subject(
    handle: *mut FfiDocumentBuilder,
    subject: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(subject) = read_cstr_or_fail(subject, error_code) else {
        return -1;
    };
    ffi_builder_apply(handle, error_code, |b| b.subject(subject))
}

/// Set the document keywords (comma-separated).
#[no_mangle]
pub extern "C" fn pdf_document_builder_set_keywords(
    handle: *mut FfiDocumentBuilder,
    keywords: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(keywords) = read_cstr_or_fail(keywords, error_code) else {
        return -1;
    };
    ffi_builder_apply(handle, error_code, |b| b.keywords(keywords))
}

/// Set the creator application name.
#[no_mangle]
pub extern "C" fn pdf_document_builder_set_creator(
    handle: *mut FfiDocumentBuilder,
    creator: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(creator) = read_cstr_or_fail(creator, error_code) else {
        return -1;
    };
    ffi_builder_apply(handle, error_code, |b| b.creator(creator))
}

/// Run JavaScript when the document is opened (`/OpenAction`).
#[no_mangle]
pub extern "C" fn pdf_document_builder_on_open(
    handle: *mut FfiDocumentBuilder,
    script: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(script, error_code) else {
        return -1;
    };
    ffi_builder_apply(handle, error_code, |b| b.on_open(s))
}

/// Enable PDF/UA-1 tagged PDF mode. Returns 0 on success, -1 on error.
///
/// When enabled, `pdf_document_builder_build` emits `/MarkInfo`,
/// `/StructTreeRoot`, `/Lang`, and `/ViewerPreferences` in the catalog.
/// Opt-in — no effect unless called. Bundle F-1/F-2.
#[no_mangle]
pub extern "C" fn pdf_document_builder_tagged_pdf_ua1(
    handle: *mut FfiDocumentBuilder,
    error_code: *mut i32,
) -> i32 {
    ffi_builder_apply(handle, error_code, |b| b.tagged_pdf_ua1())
}

/// Set the document's natural language tag (e.g. "en-US"). Returns 0 on success.
///
/// Emitted as `/Lang` in the catalog when `pdf_document_builder_tagged_pdf_ua1`
/// has been called. Bundle F-2.
#[no_mangle]
pub extern "C" fn pdf_document_builder_language(
    handle: *mut FfiDocumentBuilder,
    lang: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(lang) = read_cstr_or_fail(lang, error_code) else {
        return -1;
    };
    ffi_builder_apply(handle, error_code, |b| b.language(lang))
}

/// Add a role-map entry: custom structure type → standard PDF structure type.
/// Returns 0 on success, -1 on error.
///
/// Emitted in `/RoleMap` inside the StructTreeRoot when
/// `pdf_document_builder_tagged_pdf_ua1` is set. Multiple calls accumulate
/// entries. Bundle F-4.
#[no_mangle]
pub extern "C" fn pdf_document_builder_role_map(
    handle: *mut FfiDocumentBuilder,
    custom: *const c_char,
    standard: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(custom) = read_cstr_or_fail(custom, error_code) else {
        return -1;
    };
    let Some(standard) = read_cstr_or_fail(standard, error_code) else {
        return -1;
    };
    ffi_builder_apply(handle, error_code, |b| b.role_map(custom, standard))
}

/// Register a TTF / OTF font. **Consumes** the `font` handle — on
/// success, callers must not call `pdf_embedded_font_free` on it.
/// On error the font handle is NOT consumed and remains valid.
#[no_mangle]
pub extern "C" fn pdf_document_builder_register_embedded_font(
    handle: *mut FfiDocumentBuilder,
    name: *const c_char,
    font: *mut crate::writer::EmbeddedFont,
    error_code: *mut i32,
) -> i32 {
    let Some(name_s) = read_cstr_or_fail(name, error_code) else {
        return -1;
    };
    if font.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    // Validate builder without consuming the font.
    let Some(wrapper) = ffi_builder_mut(handle, error_code) else {
        return -1;
    };
    if wrapper.open_page {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    // All validation passed — take ownership of the font.
    let font_owned = unsafe { Box::from_raw(font) };
    let taken = wrapper.inner.take().unwrap();
    wrapper.inner = Some(taken.register_embedded_font(name_s, *font_owned));
    set_error(error_code, ERR_SUCCESS);
    0
}

fn open_page(
    handle: *mut FfiDocumentBuilder,
    page_size: Option<crate::writer::PageSize>,
    width: f32,
    height: f32,
    error_code: *mut i32,
) -> *mut FfiPageBuilder {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let Some(wrapper) = ffi_builder_mut(handle, error_code) else {
        return ptr::null_mut();
    };
    if wrapper.open_page {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    wrapper.open_page = true;
    set_error(error_code, ERR_SUCCESS);
    // SAFETY: handle checked non-null above; parent is only dereferenced when
    // the page is finalised (page_builder_done), which re-validates via ffi_builder_mut.
    Box::into_raw(Box::new(FfiPageBuilder {
        parent: handle,
        page_size,
        custom_width: width,
        custom_height: height,
        ops: Vec::new(),
        done_called: false,
        st_batch_size: 0,
        st_batch_count: 0,
        st_pending_count: 0,
    }))
}

/// Start an A4 page. Only one page at a time may be open per builder —
/// a second call while a page handle is outstanding returns NULL with
/// `ERR_INVALID_ARG`.
#[no_mangle]
pub extern "C" fn pdf_document_builder_a4_page(
    handle: *mut FfiDocumentBuilder,
    error_code: *mut i32,
) -> *mut FfiPageBuilder {
    open_page(handle, Some(crate::writer::PageSize::A4), 0.0, 0.0, error_code)
}

/// Start a US Letter page.
#[no_mangle]
pub extern "C" fn pdf_document_builder_letter_page(
    handle: *mut FfiDocumentBuilder,
    error_code: *mut i32,
) -> *mut FfiPageBuilder {
    open_page(handle, Some(crate::writer::PageSize::Letter), 0.0, 0.0, error_code)
}

/// Start a page with custom dimensions in PDF points (72 pt = 1 inch).
#[no_mangle]
pub extern "C" fn pdf_document_builder_page(
    handle: *mut FfiDocumentBuilder,
    width: f32,
    height: f32,
    error_code: *mut i32,
) -> *mut FfiPageBuilder {
    open_page(handle, None, width, height, error_code)
}

// ───────────────────────────────────────────────────────────────────────────
// PageBuilder operations
// ───────────────────────────────────────────────────────────────────────────

fn push_page_op(handle: *mut FfiPageBuilder, error_code: *mut i32, op: FfiPageOp) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let page = handle_mut(handle);
    if page.done_called {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    page.ops.push(op);
    set_error(error_code, ERR_SUCCESS);
    0
}

/// Set the font + size for subsequent text on this page.
#[no_mangle]
pub extern "C" fn pdf_page_builder_font(
    handle: *mut FfiPageBuilder,
    name: *const c_char,
    size: f32,
    error_code: *mut i32,
) -> i32 {
    let Some(name_s) = read_cstr_or_fail(name, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::Font(name_s, size))
}

/// Move the cursor to absolute coordinates (in PDF points, from lower-left).
#[no_mangle]
pub extern "C" fn pdf_page_builder_at(
    handle: *mut FfiPageBuilder,
    x: f32,
    y: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::At(x, y))
}

/// Emit a line of text at the current cursor position, then advance
/// the cursor down by one line-height.
#[no_mangle]
pub extern "C" fn pdf_page_builder_text(
    handle: *mut FfiPageBuilder,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(text_s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::Text(text_s))
}

/// Emit a heading with the given level (1–6) and text.
#[no_mangle]
pub extern "C" fn pdf_page_builder_heading(
    handle: *mut FfiPageBuilder,
    level: u8,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(text_s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::Heading(level, text_s))
}

/// Emit a paragraph with automatic line wrapping.
#[no_mangle]
pub extern "C" fn pdf_page_builder_paragraph(
    handle: *mut FfiPageBuilder,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(text_s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::Paragraph(text_s))
}

/// Advance the cursor down by `points`.
#[no_mangle]
pub extern "C" fn pdf_page_builder_space(
    handle: *mut FfiPageBuilder,
    points: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::Space(points))
}

/// Draw a horizontal rule across the page.
#[no_mangle]
pub extern "C" fn pdf_page_builder_horizontal_rule(
    handle: *mut FfiPageBuilder,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::HorizontalRule)
}

// ── Annotations (direct-method variant) ───────────────────────────────────

/// Attach a URL link to the previously-emitted text element.
#[no_mangle]
pub extern "C" fn pdf_page_builder_link_url(
    handle: *mut FfiPageBuilder,
    url: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(url_s) = read_cstr_or_fail(url, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::LinkUrl(url_s))
}

/// Link the previous text to an internal page index (zero-based).
#[no_mangle]
pub extern "C" fn pdf_page_builder_link_page(
    handle: *mut FfiPageBuilder,
    page: usize,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::LinkPage(page))
}

/// Link the previous text to a named destination.
#[no_mangle]
pub extern "C" fn pdf_page_builder_link_named(
    handle: *mut FfiPageBuilder,
    destination: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(dest_s) = read_cstr_or_fail(destination, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::LinkNamed(dest_s))
}

/// Link the previous text to a JavaScript action.
#[no_mangle]
pub extern "C" fn pdf_page_builder_link_javascript(
    handle: *mut FfiPageBuilder,
    script: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(script, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::LinkJavaScript(s))
}

/// Run JavaScript when this page is opened (`/AA /O`).
#[no_mangle]
pub extern "C" fn pdf_page_builder_on_open(
    handle: *mut FfiPageBuilder,
    script: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(script, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::OnOpen(s))
}

/// Run JavaScript when this page is closed (`/AA /C`).
#[no_mangle]
pub extern "C" fn pdf_page_builder_on_close(
    handle: *mut FfiPageBuilder,
    script: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(script, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::OnClose(s))
}

/// Set a keystroke JS action (`/AA /K`) on the most-recently-added form field.
#[no_mangle]
pub extern "C" fn pdf_page_builder_field_keystroke(
    handle: *mut FfiPageBuilder,
    script: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(script, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::FieldKeystroke(s))
}

/// Set a format JS action (`/AA /F`) on the most-recently-added form field.
#[no_mangle]
pub extern "C" fn pdf_page_builder_field_format(
    handle: *mut FfiPageBuilder,
    script: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(script, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::FieldFormat(s))
}

/// Set a validate JS action (`/AA /V`) on the most-recently-added form field.
#[no_mangle]
pub extern "C" fn pdf_page_builder_field_validate(
    handle: *mut FfiPageBuilder,
    script: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(script, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::FieldValidate(s))
}

/// Set a calculate JS action (`/AA /C`) on the most-recently-added form field.
#[no_mangle]
pub extern "C" fn pdf_page_builder_field_calculate(
    handle: *mut FfiPageBuilder,
    script: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(script, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::FieldCalculate(s))
}

/// Highlight the previous text with an RGB colour (channels in 0.0–1.0).
#[no_mangle]
pub extern "C" fn pdf_page_builder_highlight(
    handle: *mut FfiPageBuilder,
    r: f32,
    g: f32,
    b: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::Highlight(r, g, b))
}

/// Underline the previous text.
#[no_mangle]
pub extern "C" fn pdf_page_builder_underline(
    handle: *mut FfiPageBuilder,
    r: f32,
    g: f32,
    b: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::Underline(r, g, b))
}

/// Strikeout the previous text.
#[no_mangle]
pub extern "C" fn pdf_page_builder_strikeout(
    handle: *mut FfiPageBuilder,
    r: f32,
    g: f32,
    b: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::Strikeout(r, g, b))
}

/// Squiggly-underline the previous text.
#[no_mangle]
pub extern "C" fn pdf_page_builder_squiggly(
    handle: *mut FfiPageBuilder,
    r: f32,
    g: f32,
    b: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::Squiggly(r, g, b))
}

/// Attach a sticky-note annotation to the previous text.
#[no_mangle]
pub extern "C" fn pdf_page_builder_sticky_note(
    handle: *mut FfiPageBuilder,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(text_s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::StickyNote(text_s))
}

/// Place a free-standing sticky note at an absolute position on the page.
#[no_mangle]
pub extern "C" fn pdf_page_builder_sticky_note_at(
    handle: *mut FfiPageBuilder,
    x: f32,
    y: f32,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(text_s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::StickyNoteAt(x, y, text_s))
}

/// Apply a text watermark to the entire page.
#[no_mangle]
pub extern "C" fn pdf_page_builder_watermark(
    handle: *mut FfiPageBuilder,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(text_s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::Watermark(text_s))
}

/// Apply the standard "CONFIDENTIAL" diagonal watermark.
#[no_mangle]
pub extern "C" fn pdf_page_builder_watermark_confidential(
    handle: *mut FfiPageBuilder,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::WatermarkConfidential)
}

/// Apply the standard "DRAFT" diagonal watermark.
#[no_mangle]
pub extern "C" fn pdf_page_builder_watermark_draft(
    handle: *mut FfiPageBuilder,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::WatermarkDraft)
}

/// Attach a standard stamp annotation at the current cursor position
/// (default 150×50 pt box). `type_name` matches the PDF spec's stamp
/// names (Approved, NotApproved, Draft, Confidential, Final,
/// Experimental, Expired, ForPublicRelease, NotForPublicRelease, AsIs,
/// Sold, Departmental, ForComment, TopSecret) — anything else becomes
/// a custom stamp with that text.
#[no_mangle]
pub extern "C" fn pdf_page_builder_stamp(
    handle: *mut FfiPageBuilder,
    type_name: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(name) = read_cstr_or_fail(type_name, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::Stamp(name))
}

/// Place a free-flowing text annotation inside the given rectangle.
#[no_mangle]
pub extern "C" fn pdf_page_builder_freetext(
    handle: *mut FfiPageBuilder,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(text_s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::FreeText {
            x,
            y,
            w,
            h,
            text: text_s,
        },
    )
}

// ── Form-field widget creation ─────────────────────────────────────────

/// Add a single-line text form field. `default_value` may be NULL for
/// a blank field; the initial value otherwise.
#[no_mangle]
pub extern "C" fn pdf_page_builder_text_field(
    handle: *mut FfiPageBuilder,
    name: *const c_char,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    default_value: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(name_s) = read_cstr_or_fail(name, error_code) else {
        return -1;
    };
    let default_s = if default_value.is_null() {
        None
    } else {
        match c_str(default_value) {
            Ok(s) => Some(s.to_string()),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return -1;
            },
        }
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::TextField {
            name: name_s,
            x,
            y,
            w,
            h,
            default_value: default_s,
        },
    )
}

/// Add a checkbox form field. `checked` is non-zero for initially-ticked.
#[no_mangle]
pub extern "C" fn pdf_page_builder_checkbox(
    handle: *mut FfiPageBuilder,
    name: *const c_char,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    checked: i32,
    error_code: *mut i32,
) -> i32 {
    let Some(name_s) = read_cstr_or_fail(name, error_code) else {
        return -1;
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::Checkbox {
            name: name_s,
            x,
            y,
            w,
            h,
            checked: checked != 0,
        },
    )
}

/// Helper — collect a C string array into a `Vec<String>`.
fn read_cstring_array(
    array: *const *const c_char,
    count: usize,
    error_code: *mut i32,
) -> Option<Vec<String>> {
    if array.is_null() || count == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return None;
    }
    let mut out = Vec::with_capacity(count);
    for &ptr in raw_slice(array, count) {
        if ptr.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return None;
        }
        match c_str(ptr) {
            Ok(s) => out.push(s.to_string()),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return None;
            },
        }
    }
    Some(out)
}

/// Add a dropdown combo-box with a fixed list of string options.
/// `options` is an array of C-strings of length `options_count`.
/// `selected` may be NULL for no initial selection.
#[no_mangle]
pub extern "C" fn pdf_page_builder_combo_box(
    handle: *mut FfiPageBuilder,
    name: *const c_char,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    options: *const *const c_char,
    options_count: usize,
    selected: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(name_s) = read_cstr_or_fail(name, error_code) else {
        return -1;
    };
    let Some(opts) = read_cstring_array(options, options_count, error_code) else {
        return -1;
    };
    let selected_s = if selected.is_null() {
        None
    } else {
        match c_str(selected) {
            Ok(s) => Some(s.to_string()),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return -1;
            },
        }
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::ComboBox {
            name: name_s,
            x,
            y,
            w,
            h,
            options: opts,
            selected: selected_s,
        },
    )
}

/// Add a radio-button group. `values` / `xs` / `ys` / `ws` / `hs` are
/// parallel arrays of length `count` describing each button's export
/// value and rect. `selected` may be NULL.
#[no_mangle]
pub extern "C" fn pdf_page_builder_radio_group(
    handle: *mut FfiPageBuilder,
    name: *const c_char,
    values: *const *const c_char,
    xs: *const f32,
    ys: *const f32,
    ws: *const f32,
    hs: *const f32,
    count: usize,
    selected: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(name_s) = read_cstr_or_fail(name, error_code) else {
        return -1;
    };
    if count == 0 || xs.is_null() || ys.is_null() || ws.is_null() || hs.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let Some(vs) = read_cstring_array(values, count, error_code) else {
        return -1;
    };
    let xs_slice = raw_slice(xs, count);
    let ys_slice = raw_slice(ys, count);
    let ws_slice = raw_slice(ws, count);
    let hs_slice = raw_slice(hs, count);
    let buttons: Vec<(String, f32, f32, f32, f32)> = vs
        .into_iter()
        .zip(xs_slice.iter().copied())
        .zip(ys_slice.iter().copied())
        .zip(ws_slice.iter().copied())
        .zip(hs_slice.iter().copied())
        .map(|((((v, x), y), w), h)| (v, x, y, w, h))
        .collect();
    let selected_s = if selected.is_null() {
        None
    } else {
        match c_str(selected) {
            Ok(s) => Some(s.to_string()),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return -1;
            },
        }
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::RadioGroup {
            name: name_s,
            buttons,
            selected: selected_s,
        },
    )
}

/// Add a clickable push button with a visible caption.
#[no_mangle]
pub extern "C" fn pdf_page_builder_push_button(
    handle: *mut FfiPageBuilder,
    name: *const c_char,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    caption: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(name_s) = read_cstr_or_fail(name, error_code) else {
        return -1;
    };
    let Some(caption_s) = read_cstr_or_fail(caption, error_code) else {
        return -1;
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::PushButton {
            name: name_s,
            x,
            y,
            w,
            h,
            caption: caption_s,
        },
    )
}

/// Add an unsigned signature placeholder field to the page.
#[no_mangle]
pub extern "C" fn pdf_page_builder_signature_field(
    handle: *mut FfiPageBuilder,
    name: *const c_char,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> i32 {
    let Some(name_s) = read_cstr_or_fail(name, error_code) else {
        return -1;
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::SignatureField {
            name: name_s,
            x,
            y,
            w,
            h,
        },
    )
}

/// Add a footnote reference mark at the current cursor position and record the
/// note body for page-end placement.
///
/// `ref_mark` — superscript label emitted inline (e.g. "¹" or `[1]`).
/// `note_text` — body text placed near the page bottom with a separator line.
///
/// # Safety
/// `handle`, `ref_mark`, and `note_text` must be valid non-null pointers.
#[no_mangle]
pub extern "C" fn pdf_page_builder_footnote(
    handle: *mut FfiPageBuilder,
    ref_mark: *const c_char,
    note_text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(ref_mark_s) = read_cstr_or_fail(ref_mark, error_code) else {
        return -1;
    };
    let Some(note_text_s) = read_cstr_or_fail(note_text, error_code) else {
        return -1;
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::Footnote {
            ref_mark: ref_mark_s,
            note_text: note_text_s,
        },
    )
}

/// Lay out `text` as wrapped lines distributed across `column_count` balanced
/// columns at the current cursor position, with `gap_pt` points between columns.
///
/// # Safety
/// `handle` and `text` must be valid non-null pointers.
#[no_mangle]
pub extern "C" fn pdf_page_builder_columns(
    handle: *mut FfiPageBuilder,
    column_count: u32,
    gap_pt: f32,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(text_s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::Columns {
            count: column_count,
            gap_pt,
            text: text_s,
        },
    )
}

// ── Rich text inline runs ─────────────────────────────────────────────

/// Emit `text` inline at the cursor — advances cursor_x, not cursor_y.
/// Call `pdf_page_builder_newline` to advance to the next line.
///
/// # Safety
/// `handle` and `text` must be valid non-null pointers.
#[no_mangle]
pub extern "C" fn pdf_page_builder_inline(
    handle: *mut FfiPageBuilder,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::Inline(s))
}

/// Inline bold run.
///
/// # Safety
/// `handle` and `text` must be valid non-null pointers.
#[no_mangle]
pub extern "C" fn pdf_page_builder_inline_bold(
    handle: *mut FfiPageBuilder,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::InlineBold(s))
}

/// Inline italic run.
///
/// # Safety
/// `handle` and `text` must be valid non-null pointers.
#[no_mangle]
pub extern "C" fn pdf_page_builder_inline_italic(
    handle: *mut FfiPageBuilder,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::InlineItalic(s))
}

/// Inline colored run (RGB 0.0–1.0).
///
/// # Safety
/// `handle` and `text` must be valid non-null pointers.
#[no_mangle]
pub extern "C" fn pdf_page_builder_inline_color(
    handle: *mut FfiPageBuilder,
    r: f32,
    g: f32,
    b: f32,
    text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(s) = read_cstr_or_fail(text, error_code) else {
        return -1;
    };
    push_page_op(handle, error_code, FfiPageOp::InlineColor { r, g, b, text: s })
}

/// Advance cursor_y by one line-height and reset cursor_x to 72 pt.
///
/// # Safety
/// `handle` must be a valid non-null pointer.
#[no_mangle]
pub extern "C" fn pdf_page_builder_newline(
    handle: *mut FfiPageBuilder,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::Newline)
}

// ── Barcode helpers ───────────────────────────────────────────────────

/// Map FFI barcode-type integer to the Rust enum.
fn ffi_barcode_type(n: i32) -> Option<crate::writer::BarcodeType> {
    use crate::writer::BarcodeType;
    match n {
        0 => Some(BarcodeType::Code128),
        1 => Some(BarcodeType::Code39),
        2 => Some(BarcodeType::Ean13),
        3 => Some(BarcodeType::Ean8),
        4 => Some(BarcodeType::UpcA),
        5 => Some(BarcodeType::Itf),
        6 => Some(BarcodeType::Code93),
        7 => Some(BarcodeType::Codabar),
        _ => None,
    }
}

/// Place a 1-D barcode image on the page.
/// `barcode_type`: 0=Code128 1=Code39 2=EAN13 3=EAN8 4=UPCA 5=ITF 6=Code93 7=Codabar.
#[no_mangle]
pub extern "C" fn pdf_page_builder_barcode_1d(
    handle: *mut FfiPageBuilder,
    barcode_type: i32,
    data: *const c_char,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> i32 {
    let Some(bt) = ffi_barcode_type(barcode_type) else {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    };
    let Some(data_s) = read_cstr_or_fail(data, error_code) else {
        return -1;
    };
    let opts = crate::writer::BarcodeOptions::new()
        .width(w as u32)
        .height(h as u32);
    let bytes = match crate::writer::BarcodeGenerator::generate_1d(bt, &data_s, &opts) {
        Ok(b) => b,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    push_page_op(handle, error_code, FfiPageOp::BarcodeImage { bytes, x, y, w, h })
}

/// Place a QR-code image on the page (square: `size × size` points).
#[no_mangle]
pub extern "C" fn pdf_page_builder_barcode_qr(
    handle: *mut FfiPageBuilder,
    data: *const c_char,
    x: f32,
    y: f32,
    size: f32,
    error_code: *mut i32,
) -> i32 {
    let Some(data_s) = read_cstr_or_fail(data, error_code) else {
        return -1;
    };
    let opts = crate::writer::QrCodeOptions::new().size(size as u32);
    let bytes = match crate::writer::BarcodeGenerator::generate_qr(&data_s, &opts) {
        Ok(b) => b,
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::BarcodeImage {
            bytes,
            x,
            y,
            w: size,
            h: size,
        },
    )
}

/// Embed an image at `(x, y, w, h)` with an accessibility alt text.
/// `bytes` must point to raw JPEG/PNG/WebP image data of `len` bytes.
/// Embed an image at `(x, y, w, h)` in PDF points (no accessibility wrapper).
/// `bytes` must point to raw JPEG/PNG image data of `len` bytes.
/// Returns 0 on success, −1 on error.
#[no_mangle]
pub unsafe extern "C" fn pdf_page_builder_image(
    handle: *mut FfiPageBuilder,
    bytes: *const u8,
    len: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> i32 {
    if bytes.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let data = raw_slice(bytes, len).to_vec();
    push_page_op(
        handle,
        error_code,
        FfiPageOp::Image {
            bytes: data,
            x,
            y,
            w,
            h,
        },
    )
}

/// `alt_text` is a NUL-terminated UTF-8 string.
/// Returns 0 on success, −1 on error.
#[no_mangle]
pub unsafe extern "C" fn pdf_page_builder_image_with_alt(
    handle: *mut FfiPageBuilder,
    bytes: *const u8,
    len: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    alt_text: *const c_char,
    error_code: *mut i32,
) -> i32 {
    if bytes.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let data = raw_slice(bytes, len).to_vec();
    let Some(alt) = read_cstr_or_fail(alt_text, error_code) else {
        return -1;
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::ImageWithAlt {
            bytes: data,
            x,
            y,
            w,
            h,
            alt_text: alt,
        },
    )
}

/// Embed a decorative image at `(x, y, w, h)` as an /Artifact (no alt text).
/// `bytes` must point to raw JPEG/PNG/WebP image data of `len` bytes.
/// Returns 0 on success, −1 on error.
#[no_mangle]
pub unsafe extern "C" fn pdf_page_builder_image_artifact(
    handle: *mut FfiPageBuilder,
    bytes: *const u8,
    len: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> i32 {
    if bytes.is_null() || len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let data = raw_slice(bytes, len).to_vec();
    push_page_op(
        handle,
        error_code,
        FfiPageOp::ImageArtifact {
            bytes: data,
            x,
            y,
            w,
            h,
        },
    )
}

// ── Low-level graphics primitives (PdfWriter exposure) ────────────────

/// Draw a stroked rectangle outline (1pt black).
#[no_mangle]
pub extern "C" fn pdf_page_builder_rect(
    handle: *mut FfiPageBuilder,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::Rect(x, y, w, h))
}

/// Draw a filled rectangle in RGB colour (channels 0–1).
#[no_mangle]
pub extern "C" fn pdf_page_builder_filled_rect(
    handle: *mut FfiPageBuilder,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    r: f32,
    g: f32,
    b: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::FilledRect(x, y, w, h, r, g, b))
}

/// Draw a line from `(x1, y1)` to `(x2, y2)` with 1pt black stroke.
#[no_mangle]
pub extern "C" fn pdf_page_builder_line(
    handle: *mut FfiPageBuilder,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::Line(x1, y1, x2, y2))
}

/// Buffer a `stroke_rect(x, y, w, h, LineStyle)` call. `width` is the
/// stroke width in points; `r`/`g`/`b` are 0..1 colour channels.
#[no_mangle]
pub extern "C" fn pdf_page_builder_stroke_rect(
    handle: *mut FfiPageBuilder,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    width: f32,
    r: f32,
    g: f32,
    b: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(
        handle,
        error_code,
        FfiPageOp::StrokeRect {
            x,
            y,
            w,
            h,
            width,
            r,
            g,
            b,
        },
    )
}

/// Buffer a `stroke_line(p1, p2, LineStyle)` call.
#[no_mangle]
pub extern "C" fn pdf_page_builder_stroke_line(
    handle: *mut FfiPageBuilder,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    width: f32,
    r: f32,
    g: f32,
    b: f32,
    error_code: *mut i32,
) -> i32 {
    push_page_op(
        handle,
        error_code,
        FfiPageOp::StrokeLine {
            x1,
            y1,
            x2,
            y2,
            width,
            r,
            g,
            b,
        },
    )
}

/// Buffer a `stroke_rect` with a dash pattern.
///
/// `dash_array` is a pointer to `n_dash` alternating on/off lengths in points.
/// `phase` is the starting offset into the pattern. Pass `n_dash == 0` for solid.
///
/// # Safety
/// `handle` must satisfy the Handle Ownership Contract. If `n_dash > 0`,
/// `dash_array` must point to `n_dash` valid `f32` values.
#[no_mangle]
pub extern "C" fn pdf_page_builder_stroke_rect_dashed(
    handle: *mut FfiPageBuilder,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    width: f32,
    r: f32,
    g: f32,
    b: f32,
    dash_array: *const f32,
    n_dash: usize,
    phase: f32,
    error_code: *mut i32,
) -> i32 {
    let dash = if n_dash == 0 || dash_array.is_null() {
        vec![]
    } else {
        raw_slice(dash_array, n_dash).to_vec()
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::StrokeRectDashed {
            x,
            y,
            w,
            h,
            width,
            r,
            g,
            b,
            dash,
            phase,
        },
    )
}

/// Buffer a `stroke_line` with a dash pattern.
///
/// # Safety
/// `handle` must satisfy the Handle Ownership Contract. If `n_dash > 0`,
/// `dash_array` must point to `n_dash` valid `f32` values.
#[no_mangle]
pub extern "C" fn pdf_page_builder_stroke_line_dashed(
    handle: *mut FfiPageBuilder,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    width: f32,
    r: f32,
    g: f32,
    b: f32,
    dash_array: *const f32,
    n_dash: usize,
    phase: f32,
    error_code: *mut i32,
) -> i32 {
    let dash = if n_dash == 0 || dash_array.is_null() {
        vec![]
    } else {
        raw_slice(dash_array, n_dash).to_vec()
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::StrokeLineDashed {
            x1,
            y1,
            x2,
            y2,
            width,
            r,
            g,
            b,
            dash,
            phase,
        },
    )
}

/// Buffer a `text_in_rect(rect, text, align)` call. `align` encodes
/// 0=Left, 1=Center, 2=Right — anything else is treated as Left.
#[no_mangle]
pub extern "C" fn pdf_page_builder_text_in_rect(
    handle: *mut FfiPageBuilder,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: *const c_char,
    align: i32,
    error_code: *mut i32,
) -> i32 {
    if text.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let text_s = match c_str(text) {
        Ok(s) => s.to_string(),
        Err(_) => {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        },
    };
    push_page_op(
        handle,
        error_code,
        FfiPageOp::TextInRect {
            x,
            y,
            w,
            h,
            text: text_s,
            align,
        },
    )
}

/// Buffer a `new_page_same_size` transition. Subsequent ops buffered on
/// this same page handle land on the new page.
#[no_mangle]
pub extern "C" fn pdf_page_builder_new_page_same_size(
    handle: *mut FfiPageBuilder,
    error_code: *mut i32,
) -> i32 {
    push_page_op(handle, error_code, FfiPageOp::NewPageSameSize)
}

/// Buffer a buffered `table(Table)` call — the whole row matrix is
/// stored in memory until `pdf_page_builder_done` replays it against the
/// real FluentPageBuilder.
///
/// Cell array is row-major: `cell_strings[row * n_columns + col]`. Each
/// pointer must be a valid null-terminated UTF-8 C string. `widths`
/// `aligns` are both length `n_columns`; `aligns` encodes 0/1/2 (see
/// `pdf_page_builder_text_in_rect`). `has_header != 0` promotes the
/// first row to a header (bold + default background).
///
/// The streaming variant (row-at-a-time, O(cols) memory) is tracked
/// separately — see issue #393 step 6.5.
#[no_mangle]
pub extern "C" fn pdf_page_builder_table(
    handle: *mut FfiPageBuilder,
    n_columns: usize,
    widths: *const f32,
    aligns: *const i32,
    n_rows: usize,
    cell_strings: *const *const c_char,
    has_header: i32,
    error_code: *mut i32,
) -> i32 {
    if n_columns == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    if (aligns.is_null() || widths.is_null()) && n_columns > 0
        || (cell_strings.is_null() && n_rows > 0)
    {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let widths_vec: Vec<f32> = raw_slice(widths, n_columns).to_vec();
    let aligns_vec: Vec<i32> = raw_slice(aligns, n_columns).to_vec();

    let total = n_rows.saturating_mul(n_columns);
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(n_rows);
    if total > 0 {
        let strings = raw_slice(cell_strings, total);
        for row_idx in 0..n_rows {
            let mut row = Vec::with_capacity(n_columns);
            for col_idx in 0..n_columns {
                let ptr = strings[row_idx * n_columns + col_idx];
                if ptr.is_null() {
                    row.push(String::new());
                    continue;
                }
                match c_str(ptr) {
                    Ok(s) => row.push(s.to_string()),
                    Err(_) => {
                        set_error(error_code, ERR_INVALID_ARG);
                        return -1;
                    },
                }
            }
            rows.push(row);
        }
    }

    push_page_op(
        handle,
        error_code,
        FfiPageOp::Table {
            widths: widths_vec,
            aligns: aligns_vec,
            rows,
            has_header: has_header != 0,
        },
    )
}

/// Open a streaming table on this page. Columns are supplied as three
/// parallel arrays — `headers` is a length-`n_columns` array of
/// null-terminated C strings; `widths` and `aligns` are length-
/// `n_columns` numeric arrays (aligns: 0=Left, 1=Center, 2=Right).
///
/// Subsequent calls to `pdf_page_builder_streaming_table_push_row` feed
/// rows into the open table. Call `pdf_page_builder_streaming_table_finish`
/// to close it; the table is also auto-closed on `_done`.
///
/// In v0.3.39 the FFI layer buffers rows until `_done` replays them
/// against the live Rust `StreamingTable` — true O(cols) memory lives
/// in the Rust core; the FFI replay holds `Vec<Vec<String>>` temporarily.
/// Row-by-row FFI streaming is tracked under #400 v0.3.40.
#[no_mangle]
pub extern "C" fn pdf_page_builder_streaming_table_begin(
    handle: *mut FfiPageBuilder,
    n_columns: usize,
    headers: *const *const c_char,
    widths: *const f32,
    aligns: *const i32,
    repeat_header: i32,
    error_code: *mut i32,
) -> i32 {
    if n_columns == 0 || headers.is_null() || widths.is_null() || aligns.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let widths_slice = raw_slice(widths, n_columns);
    let aligns_slice = raw_slice(aligns, n_columns);
    let headers_slice = raw_slice(headers, n_columns);

    let mut cols: Vec<(String, f32, i32)> = Vec::with_capacity(n_columns);
    for i in 0..n_columns {
        let ptr = headers_slice[i];
        if ptr.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let header = match c_str(ptr) {
            Ok(s) => s.to_string(),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return -1;
            },
        };
        cols.push((header, widths_slice[i], aligns_slice[i]));
    }

    let pb = handle_mut(handle);
    pb.st_batch_size = 256;
    pb.st_batch_count = 0;
    pb.st_pending_count = 0;
    push_page_op(
        handle,
        error_code,
        FfiPageOp::StreamingTableOpen {
            columns: cols,
            repeat_header: repeat_header != 0,
            mode: 0,
            sample_rows: 50,
            min_col_width_pt: 20.0,
            max_col_width_pt: 400.0,
            max_rowspan: 1,
        },
    )
}

/// Begin a streaming table with a column-width mode.
///
/// `mode`: 0 = Fixed (default), 1 = Sample(sample_rows, min_w, max_w),
///         2 = AutoAll (rejected — see `pdf_page_builder_streaming_table_begin`).
/// `max_rowspan`: 0 or 1 disables rowspan; ≥2 enables cells to span that many rows.
#[no_mangle]
pub extern "C" fn pdf_page_builder_streaming_table_begin_v2(
    handle: *mut FfiPageBuilder,
    n_columns: usize,
    headers: *const *const c_char,
    widths: *const f32,
    aligns: *const i32,
    repeat_header: i32,
    mode: i32,
    sample_rows: usize,
    min_col_width_pt: f32,
    max_col_width_pt: f32,
    max_rowspan: usize,
    error_code: *mut i32,
) -> i32 {
    if n_columns == 0 || headers.is_null() || widths.is_null() || aligns.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let widths_slice = raw_slice(widths, n_columns);
    let aligns_slice = raw_slice(aligns, n_columns);
    let headers_slice = raw_slice(headers, n_columns);

    let mut cols: Vec<(String, f32, i32)> = Vec::with_capacity(n_columns);
    for i in 0..n_columns {
        let ptr = headers_slice[i];
        if ptr.is_null() {
            set_error(error_code, ERR_INVALID_ARG);
            return -1;
        }
        let header = match c_str(ptr) {
            Ok(s) => s.to_string(),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return -1;
            },
        };
        cols.push((header, widths_slice[i], aligns_slice[i]));
    }

    let pb = handle_mut(handle);
    pb.st_batch_size = 256;
    pb.st_batch_count = 0;
    pb.st_pending_count = 0;
    push_page_op(
        handle,
        error_code,
        FfiPageOp::StreamingTableOpen {
            columns: cols,
            repeat_header: repeat_header != 0,
            mode,
            sample_rows,
            min_col_width_pt,
            max_col_width_pt,
            max_rowspan,
        },
    )
}

/// Set the batch size for the currently-open streaming table. When
/// `push_row` accumulates this many pending rows, a batch boundary is
/// recorded automatically (auto-flush). Call after
/// `pdf_page_builder_streaming_table_begin[_v2]` and before the first
/// `push_row`. A value of 0 defaults to 256.
#[no_mangle]
pub extern "C" fn pdf_page_builder_streaming_table_set_batch_size(
    handle: *mut FfiPageBuilder,
    batch_size: usize,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let pb = handle_mut(handle);
    pb.st_batch_size = if batch_size == 0 { 256 } else { batch_size };
    pb.st_batch_count = 0;
    pb.st_pending_count = 0;
    set_error(error_code, ERR_SUCCESS);
    0
}

/// Return the number of rows pushed since the last batch boundary
/// (or since the table was opened). Returns 0 if no table is open.
#[no_mangle]
pub extern "C" fn pdf_page_builder_streaming_table_pending_row_count(
    handle: *mut FfiPageBuilder,
) -> usize {
    if handle.is_null() {
        return 0;
    }
    handle_ref(handle).st_pending_count
}

/// Return the number of complete batches recorded so far.
/// Returns 0 if no table is open.
#[no_mangle]
pub extern "C" fn pdf_page_builder_streaming_table_batch_count(
    handle: *mut FfiPageBuilder,
) -> usize {
    if handle.is_null() {
        return 0;
    }
    handle_ref(handle).st_batch_count
}

/// Explicitly mark a batch boundary: increment the batch counter
/// reset the pending-row counter. A no-op if there are no pending rows.
#[no_mangle]
pub extern "C" fn pdf_page_builder_streaming_table_flush(
    handle: *mut FfiPageBuilder,
    error_code: *mut i32,
) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let pb = handle_mut(handle);
    if pb.st_pending_count > 0 {
        pb.st_batch_count += 1;
        pb.st_pending_count = 0;
    }
    set_error(error_code, ERR_SUCCESS);
    0
}

/// Push one row into the currently-open streaming table. `cells` is
/// a length-`n_cells` array of null-terminated UTF-8 C strings; its
/// length must equal the column count supplied to `_begin`.
/// All cells are treated as rowspan=1. For rowspan > 1 use
/// `pdf_page_builder_streaming_table_push_row_v2`.
#[no_mangle]
pub extern "C" fn pdf_page_builder_streaming_table_push_row(
    handle: *mut FfiPageBuilder,
    n_cells: usize,
    cells: *const *const c_char,
    error_code: *mut i32,
) -> i32 {
    pdf_page_builder_streaming_table_push_row_v2(
        handle,
        n_cells,
        cells,
        std::ptr::null(), // rowspans=NULL → all 1
        error_code,
    )
}

/// Push one row with per-cell rowspan values. `cells` is a length-`n_cells`
/// array of null-terminated UTF-8 C strings. `rowspans` is a length-`n_cells`
/// array of `size_t` values (1 = normal cell, ≥2 = span). Pass NULL for
/// `rowspans` to treat all cells as rowspan=1 (same as `_push_row`).
#[no_mangle]
pub extern "C" fn pdf_page_builder_streaming_table_push_row_v2(
    handle: *mut FfiPageBuilder,
    n_cells: usize,
    cells: *const *const c_char,
    rowspans: *const usize,
    error_code: *mut i32,
) -> i32 {
    if n_cells == 0 || cells.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let cells_slice = raw_slice(cells, n_cells);
    let rowspans_slice = if rowspans.is_null() {
        None
    } else {
        Some(raw_slice(rowspans, n_cells))
    };
    let mut row: Vec<(String, usize)> = Vec::with_capacity(n_cells);
    for i in 0..n_cells {
        let ptr = cells_slice[i];
        let text = if ptr.is_null() {
            String::new()
        } else {
            match c_str(ptr) {
                Ok(s) => s.to_string(),
                Err(_) => {
                    set_error(error_code, ERR_INVALID_ARG);
                    return -1;
                },
            }
        };
        let span = rowspans_slice.map_or(1, |rs| rs[i].max(1));
        row.push((text, span));
    }
    let rc = push_page_op(handle, error_code, FfiPageOp::StreamingTableRow(row));
    if rc == 0 {
        let pb = handle_mut(handle);
        pb.st_pending_count += 1;
        if pb.st_batch_size > 0 && pb.st_pending_count >= pb.st_batch_size {
            pb.st_batch_count += 1;
            pb.st_pending_count = 0;
        }
    }
    rc
}

/// Close the currently-open streaming table. Records a final batch
/// boundary if any rows are pending, then resets all batch state.
/// Returns -1 + ERR_INVALID_ARG if no table is open.
#[no_mangle]
pub extern "C" fn pdf_page_builder_streaming_table_finish(
    handle: *mut FfiPageBuilder,
    error_code: *mut i32,
) -> i32 {
    if !handle.is_null() {
        let pb = handle_mut(handle);
        if pb.st_pending_count > 0 {
            pb.st_batch_count += 1;
            pb.st_pending_count = 0;
        }
        pb.st_batch_size = 0;
        pb.st_batch_count = 0;
    }
    push_page_op(handle, error_code, FfiPageOp::StreamingTableFinish)
}

/// Commit this page's buffered operations to its parent builder
/// **consume** the page handle. After a successful call the handle is
/// invalid; do not call `_free`.
#[no_mangle]
pub extern "C" fn pdf_page_builder_done(handle: *mut FfiPageBuilder, error_code: *mut i32) -> i32 {
    if handle.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    // Take ownership of the page handle.
    let mut page = unsafe { Box::from_raw(handle) };
    if page.done_called {
        set_error(error_code, ERR_INVALID_ARG);
        // Re-leak so later `_free` is still safe.
        Box::leak(page);
        return -1;
    }
    page.done_called = true;

    // Access parent — SAFETY: one-page-at-a-time invariant means no
    // other FfiPageBuilder references this parent, and the caller has
    // promised not to free the parent while this page is outstanding.
    if page.parent.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    }
    let parent = handle_mut(page.parent);
    let Some(builder) = parent.inner.as_mut() else {
        set_error(error_code, ERR_INVALID_ARG);
        parent.open_page = false;
        return -1;
    };

    let page_size = page
        .page_size
        .unwrap_or(crate::writer::PageSize::Custom(page.custom_width, page.custom_height));
    let mut rust_page_opt: Option<crate::writer::FluentPageBuilder<'_>> =
        Some(builder.page(page_size));
    let mut streaming_opt: Option<crate::writer::StreamingTable<'_>> = None;

    for op in page.ops.drain(..) {
        // Streaming mode handles its own row / finish variants. Other
        // variants in streaming mode are a programmer error at the
        // binding layer; we flush them straight through to avoid
        // silently swallowing them.
        match op {
            FfiPageOp::StreamingTableOpen {
                columns,
                repeat_header,
                mode,
                sample_rows,
                min_col_width_pt,
                max_col_width_pt,
                max_rowspan,
            } => {
                let Some(rp) = rust_page_opt.take() else {
                    // Already streaming — reject reopening.
                    set_error(error_code, ERR_INVALID_ARG);
                    return -1;
                };
                let mut config = crate::writer::StreamingTableConfig::new()
                    .repeat_header(repeat_header)
                    .max_rowspan(max_rowspan);
                config = match mode {
                    1 => config.mode_sample(sample_rows, min_col_width_pt, max_col_width_pt),
                    2 => config.mode_auto_all(),
                    _ => config.mode_fixed(),
                };
                for (header, width, align) in columns {
                    let cell_align = match align {
                        1 => crate::writer::CellAlign::Center,
                        2 => crate::writer::CellAlign::Right,
                        _ => crate::writer::CellAlign::Left,
                    };
                    config = config.column(
                        crate::writer::StreamingColumn::new(header)
                            .width_pt(width)
                            .align(cell_align),
                    );
                }
                streaming_opt = Some(rp.streaming_table(config));
                continue;
            },
            FfiPageOp::StreamingTableRow(cells) => {
                if let Some(st) = streaming_opt.as_mut() {
                    let _ = st.push_row(|r| {
                        for (text, span) in cells {
                            if span > 1 {
                                r.span_cell(text, span);
                            } else {
                                r.cell(text);
                            }
                        }
                    });
                } else {
                    set_error(error_code, ERR_INVALID_ARG);
                    return -1;
                }
                continue;
            },
            FfiPageOp::StreamingTableFinish => {
                let Some(st) = streaming_opt.take() else {
                    set_error(error_code, ERR_INVALID_ARG);
                    return -1;
                };
                rust_page_opt = Some(st.finish());
                continue;
            },
            FfiPageOp::BarcodeImage { bytes, x, y, w, h } => {
                let rp = match rust_page_opt.take() {
                    Some(p) => p,
                    None => {
                        set_error(error_code, ERR_INVALID_ARG);
                        return -1;
                    },
                };
                match rp.image_from_bytes(&bytes, crate::geometry::Rect::new(x, y, w, h)) {
                    Ok(p) => rust_page_opt = Some(p),
                    Err(_) => {
                        set_error(error_code, ERR_INTERNAL);
                        return -1;
                    },
                }
                continue;
            },
            FfiPageOp::Image { bytes, x, y, w, h } => {
                let rp = match rust_page_opt.take() {
                    Some(p) => p,
                    None => {
                        set_error(error_code, ERR_INVALID_ARG);
                        return -1;
                    },
                };
                match rp.image_from_bytes(&bytes, crate::geometry::Rect::new(x, y, w, h)) {
                    Ok(p) => rust_page_opt = Some(p),
                    Err(_) => {
                        set_error(error_code, ERR_INTERNAL);
                        return -1;
                    },
                }
                continue;
            },
            FfiPageOp::ImageWithAlt {
                bytes,
                x,
                y,
                w,
                h,
                alt_text,
            } => {
                let rp = match rust_page_opt.take() {
                    Some(p) => p,
                    None => {
                        set_error(error_code, ERR_INVALID_ARG);
                        return -1;
                    },
                };
                match rp.image_from_bytes_with_alt(
                    &bytes,
                    crate::geometry::Rect::new(x, y, w, h),
                    &alt_text,
                ) {
                    Ok(p) => rust_page_opt = Some(p),
                    Err(_) => {
                        set_error(error_code, ERR_INTERNAL);
                        return -1;
                    },
                }
                continue;
            },
            FfiPageOp::ImageArtifact { bytes, x, y, w, h } => {
                let rp = match rust_page_opt.take() {
                    Some(p) => p,
                    None => {
                        set_error(error_code, ERR_INVALID_ARG);
                        return -1;
                    },
                };
                match rp
                    .image_from_bytes_as_artifact(&bytes, crate::geometry::Rect::new(x, y, w, h))
                {
                    Ok(p) => rust_page_opt = Some(p),
                    Err(_) => {
                        set_error(error_code, ERR_INTERNAL);
                        return -1;
                    },
                }
                continue;
            },
            _ => {},
        }

        // Non-streaming ops: must have rust_page, not streaming.
        let rust_page = match rust_page_opt.take() {
            Some(rp) => rp,
            None => {
                // We're mid-stream but got a non-stream op. That's a
                // binding-level bug; reject to avoid silent corruption.
                set_error(error_code, ERR_INVALID_ARG);
                return -1;
            },
        };
        rust_page_opt = Some(match op {
            FfiPageOp::Font(name, size) => rust_page.font(&name, size),
            FfiPageOp::At(x, y) => rust_page.at(x, y),
            FfiPageOp::Text(text) => rust_page.text(&text),
            FfiPageOp::Heading(level, text) => rust_page.heading(level, &text),
            FfiPageOp::Paragraph(text) => rust_page.paragraph(&text),
            FfiPageOp::Space(points) => rust_page.space(points),
            FfiPageOp::HorizontalRule => rust_page.horizontal_rule(),
            FfiPageOp::LinkUrl(url) => rust_page.link_url(&url),
            FfiPageOp::LinkPage(p) => rust_page.link_page(p),
            FfiPageOp::LinkNamed(dest) => rust_page.link_named(&dest),
            FfiPageOp::LinkJavaScript(script) => rust_page.link_javascript(&script),
            FfiPageOp::OnOpen(script) => rust_page.on_open(&script),
            FfiPageOp::OnClose(script) => rust_page.on_close(&script),
            FfiPageOp::FieldKeystroke(s) => rust_page.field_keystroke(&s),
            FfiPageOp::FieldFormat(s) => rust_page.field_format(&s),
            FfiPageOp::FieldValidate(s) => rust_page.field_validate(&s),
            FfiPageOp::FieldCalculate(s) => rust_page.field_calculate(&s),
            FfiPageOp::Highlight(r, g, b) => rust_page.highlight((r, g, b)),
            FfiPageOp::Underline(r, g, b) => rust_page.underline((r, g, b)),
            FfiPageOp::Strikeout(r, g, b) => rust_page.strikeout((r, g, b)),
            FfiPageOp::Squiggly(r, g, b) => rust_page.squiggly((r, g, b)),
            FfiPageOp::StickyNote(text) => rust_page.sticky_note(&text),
            FfiPageOp::StickyNoteAt(x, y, text) => rust_page.sticky_note_at(x, y, &text),
            FfiPageOp::Watermark(text) => rust_page.watermark(&text),
            FfiPageOp::WatermarkConfidential => rust_page.watermark_confidential(),
            FfiPageOp::WatermarkDraft => rust_page.watermark_draft(),
            FfiPageOp::Stamp(name) => rust_page.stamp(ffi_parse_stamp_type(&name)),
            FfiPageOp::FreeText { x, y, w, h, text } => {
                rust_page.freetext(crate::geometry::Rect::new(x, y, w, h), &text)
            },
            FfiPageOp::TextField {
                name,
                x,
                y,
                w,
                h,
                default_value,
            } => rust_page.text_field(name, x, y, w, h, default_value),
            FfiPageOp::Checkbox {
                name,
                x,
                y,
                w,
                h,
                checked,
            } => rust_page.checkbox(name, x, y, w, h, checked),
            FfiPageOp::ComboBox {
                name,
                x,
                y,
                w,
                h,
                options,
                selected,
            } => rust_page.combo_box(name, x, y, w, h, options, selected),
            FfiPageOp::RadioGroup {
                name,
                buttons,
                selected,
            } => rust_page.radio_group(name, buttons, selected),
            FfiPageOp::PushButton {
                name,
                x,
                y,
                w,
                h,
                caption,
            } => rust_page.push_button(name, x, y, w, h, caption),
            FfiPageOp::SignatureField { name, x, y, w, h } => {
                rust_page.signature_field(name, x, y, w, h)
            },
            FfiPageOp::Footnote {
                ref_mark,
                note_text,
            } => rust_page.footnote(&ref_mark, &note_text),
            FfiPageOp::Columns {
                count,
                gap_pt,
                text,
            } => rust_page.columns(count, gap_pt, &text),
            FfiPageOp::Inline(text) => rust_page.inline(&text),
            FfiPageOp::InlineBold(text) => rust_page.inline_bold(&text),
            FfiPageOp::InlineItalic(text) => rust_page.inline_italic(&text),
            FfiPageOp::InlineColor { r, g, b, text } => rust_page.inline_color(r, g, b, &text),
            FfiPageOp::Newline => rust_page.newline(),
            FfiPageOp::Rect(x, y, w, h) => rust_page.rect(x, y, w, h),
            FfiPageOp::FilledRect(x, y, w, h, r, g, b) => {
                rust_page.filled_rect(x, y, w, h, r, g, b)
            },
            FfiPageOp::Line(x1, y1, x2, y2) => rust_page.line(x1, y1, x2, y2),
            FfiPageOp::StrokeRect {
                x,
                y,
                w,
                h,
                width,
                r,
                g,
                b,
            } => rust_page.stroke_rect(x, y, w, h, crate::writer::LineStyle::new(width, r, g, b)),
            FfiPageOp::StrokeRectDashed {
                x,
                y,
                w,
                h,
                width,
                r,
                g,
                b,
                dash,
                phase,
            } => {
                let style = if dash.is_empty() {
                    crate::writer::LineStyle::new(width, r, g, b)
                } else {
                    crate::writer::LineStyle::new(width, r, g, b).with_dash(&dash, phase)
                };
                rust_page.stroke_rect(x, y, w, h, style)
            },
            FfiPageOp::StrokeLine {
                x1,
                y1,
                x2,
                y2,
                width,
                r,
                g,
                b,
            } => {
                rust_page.stroke_line(x1, y1, x2, y2, crate::writer::LineStyle::new(width, r, g, b))
            },
            FfiPageOp::StrokeLineDashed {
                x1,
                y1,
                x2,
                y2,
                width,
                r,
                g,
                b,
                dash,
                phase,
            } => {
                let style = if dash.is_empty() {
                    crate::writer::LineStyle::new(width, r, g, b)
                } else {
                    crate::writer::LineStyle::new(width, r, g, b).with_dash(&dash, phase)
                };
                rust_page.stroke_line(x1, y1, x2, y2, style)
            },
            FfiPageOp::TextInRect {
                x,
                y,
                w,
                h,
                text,
                align,
            } => {
                let align_e = match align {
                    1 => crate::writer::TextAlign::Center,
                    2 => crate::writer::TextAlign::Right,
                    _ => crate::writer::TextAlign::Left,
                };
                rust_page.text_in_rect(crate::geometry::Rect::new(x, y, w, h), &text, align_e)
            },
            FfiPageOp::NewPageSameSize => rust_page.new_page_same_size(),
            FfiPageOp::Table {
                widths,
                aligns,
                rows,
                has_header,
            } => {
                let cells: Vec<Vec<crate::writer::TableCell>> = rows
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(crate::writer::TableCell::text)
                            .collect()
                    })
                    .collect();
                let mut table = crate::writer::Table::new(cells);
                let col_widths: Vec<crate::writer::ColumnWidth> = widths
                    .iter()
                    .map(|&w| crate::writer::ColumnWidth::Fixed(w))
                    .collect();
                table = table.with_column_widths(col_widths);
                let col_aligns: Vec<crate::writer::CellAlign> = aligns
                    .iter()
                    .map(|&a| match a {
                        1 => crate::writer::CellAlign::Center,
                        2 => crate::writer::CellAlign::Right,
                        _ => crate::writer::CellAlign::Left,
                    })
                    .collect();
                table.column_aligns = col_aligns;
                if has_header {
                    table = table.with_header_row();
                }
                rust_page.table(table)
            },
            FfiPageOp::StreamingTableOpen { .. }
            | FfiPageOp::StreamingTableRow(_)
            | FfiPageOp::StreamingTableFinish
            | FfiPageOp::BarcodeImage { .. }
            | FfiPageOp::Image { .. }
            | FfiPageOp::ImageWithAlt { .. }
            | FfiPageOp::ImageArtifact { .. } => {
                unreachable!("streaming ops handled above; reaching here is a replay-loop bug")
            },
        });
    }

    // If the table was left open (missing finish), close it to avoid
    // silently dropping the page.
    if let Some(st) = streaming_opt.take() {
        rust_page_opt = Some(st.finish());
    }

    let Some(rust_page) = rust_page_opt else {
        set_error(error_code, ERR_INVALID_ARG);
        return -1;
    };
    rust_page.done();
    parent.open_page = false;
    // page box drops here, releasing the buffered ops (already drained).
    set_error(error_code, ERR_SUCCESS);
    0
}

/// Drop an uncommitted page handle (error-recovery path). Does NOT
/// apply the buffered operations. If the parent builder is still
/// alive, also clears the open-page slot so the next `_page` call
/// succeeds.
#[no_mangle]
pub extern "C" fn pdf_page_builder_free(handle: *mut FfiPageBuilder) {
    if !handle.is_null() {
        unsafe {
            let page = Box::from_raw(handle);
            if !page.parent.is_null() && !page.done_called {
                (*page.parent).open_page = false;
            }
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Finalisation
// ───────────────────────────────────────────────────────────────────────────

fn consume_builder(
    handle: *mut FfiDocumentBuilder,
    error_code: *mut i32,
) -> Option<crate::writer::DocumentBuilder> {
    let wrapper = ffi_builder_mut(handle, error_code)?;
    if wrapper.open_page {
        set_error(error_code, ERR_INVALID_ARG);
        return None;
    }
    wrapper.inner.take()
}

fn bytes_to_ffi(bytes: Vec<u8>, out_len: *mut usize, error_code: *mut i32) -> *mut u8 {
    if out_len.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    write_out(out_len, bytes.len());
    set_error(error_code, ERR_SUCCESS);
    vec_to_ffi_bytes(bytes)
}

/// Build the PDF and return the bytes. Consumes the builder *state*
/// — the handle wrapper is still allocated and MUST be released with
/// `pdf_document_builder_free`. Returns NULL on error. Output buffer
/// must be freed with `free_bytes`.
#[no_mangle]
pub extern "C" fn pdf_document_builder_build(
    handle: *mut FfiDocumentBuilder,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    let Some(builder) = consume_builder(handle, error_code) else {
        return ptr::null_mut();
    };
    match builder.build() {
        Ok(bytes) => bytes_to_ffi(bytes, out_len, error_code),
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Build and save the PDF to `path`. Consumes the builder state;
/// caller must still call `pdf_document_builder_free`.
#[no_mangle]
pub extern "C" fn pdf_document_builder_save(
    handle: *mut FfiDocumentBuilder,
    path: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(path_s) = read_cstr_or_fail(path, error_code) else {
        return -1;
    };
    let Some(builder) = consume_builder(handle, error_code) else {
        return -1;
    };
    match builder.save(&path_s) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Build and save with AES-256 encryption. Consumes the builder
/// state; caller must still call `pdf_document_builder_free`.
#[no_mangle]
pub extern "C" fn pdf_document_builder_save_encrypted(
    handle: *mut FfiDocumentBuilder,
    path: *const c_char,
    user_password: *const c_char,
    owner_password: *const c_char,
    error_code: *mut i32,
) -> i32 {
    let Some(path_s) = read_cstr_or_fail(path, error_code) else {
        return -1;
    };
    let Some(user_pw) = read_cstr_or_fail(user_password, error_code) else {
        return -1;
    };
    let Some(owner_pw) = read_cstr_or_fail(owner_password, error_code) else {
        return -1;
    };
    let Some(builder) = consume_builder(handle, error_code) else {
        return -1;
    };
    match builder.save_encrypted(&path_s, &user_pw, &owner_pw) {
        Ok(()) => {
            set_error(error_code, ERR_SUCCESS);
            0
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            -1
        },
    }
}

/// Build encrypted bytes. Consumes the builder state; caller must
/// still call `pdf_document_builder_free`. Output buffer must be
/// freed with `free_bytes`.
#[no_mangle]
pub extern "C" fn pdf_document_builder_to_bytes_encrypted(
    handle: *mut FfiDocumentBuilder,
    user_password: *const c_char,
    owner_password: *const c_char,
    out_len: *mut usize,
    error_code: *mut i32,
) -> *mut u8 {
    let Some(user_pw) = read_cstr_or_fail(user_password, error_code) else {
        return ptr::null_mut();
    };
    let Some(owner_pw) = read_cstr_or_fail(owner_password, error_code) else {
        return ptr::null_mut();
    };
    let Some(builder) = consume_builder(handle, error_code) else {
        return ptr::null_mut();
    };
    match builder.to_bytes_encrypted(&user_pw, &owner_pw) {
        Ok(bytes) => bytes_to_ffi(bytes, out_len, error_code),
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

// =============================================================================
// HTML+CSS pipeline
// =============================================================================

/// Build a PDF from HTML + CSS + a single embedded font. Returns a
/// `Pdf` handle (usable with `pdf_save` / `pdf_save_to_bytes`) or NULL
/// on error.
#[no_mangle]
pub extern "C" fn pdf_from_html_css(
    html: *const c_char,
    css: *const c_char,
    font_bytes: *const u8,
    font_len: usize,
    error_code: *mut i32,
) -> *mut Pdf {
    let Some(html_s) = read_cstr_or_fail(html, error_code) else {
        return ptr::null_mut();
    };
    let Some(css_s) = read_cstr_or_fail(css, error_code) else {
        return ptr::null_mut();
    };
    if font_bytes.is_null() || font_len == 0 {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let font_vec = raw_slice(font_bytes, font_len).to_vec();
    match Pdf::from_html_css(&html_s, &css_s, font_vec) {
        Ok(pdf) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(pdf))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

/// Build a PDF from HTML+CSS with a multi-font cascade. `families`,
/// `font_bytes`, and `font_lens` are parallel arrays of length `count`.
/// Caller-owned; the FFI copies the bytes.
#[no_mangle]
pub extern "C" fn pdf_from_html_css_with_fonts(
    html: *const c_char,
    css: *const c_char,
    families: *const *const c_char,
    font_bytes: *const *const u8,
    font_lens: *const usize,
    count: usize,
    error_code: *mut i32,
) -> *mut Pdf {
    let Some(html_s) = read_cstr_or_fail(html, error_code) else {
        return ptr::null_mut();
    };
    let Some(css_s) = read_cstr_or_fail(css, error_code) else {
        return ptr::null_mut();
    };
    if count == 0 || families.is_null() || font_bytes.is_null() || font_lens.is_null() {
        set_error(error_code, ERR_INVALID_ARG);
        return ptr::null_mut();
    }
    let mut fonts: Vec<(String, Vec<u8>)> = Vec::with_capacity(count);
    let names_slice = raw_slice(families, count);
    let bytes_ptrs_slice = raw_slice(font_bytes, count);
    let bytes_lens_slice = raw_slice(font_lens, count);
    for i in 0..count {
        let name_ptr = names_slice[i];
        let bytes_ptr = bytes_ptrs_slice[i];
        let bytes_len = bytes_lens_slice[i];
        if name_ptr.is_null() || bytes_ptr.is_null() || bytes_len == 0 {
            set_error(error_code, ERR_INVALID_ARG);
            return ptr::null_mut();
        }
        let name = match c_str(name_ptr) {
            Ok(s) => s.to_string(),
            Err(_) => {
                set_error(error_code, ERR_INVALID_ARG);
                return ptr::null_mut();
            },
        };
        let bytes = raw_slice(bytes_ptr, bytes_len).to_vec();
        fonts.push((name, bytes));
    }
    match Pdf::from_html_css_with_fonts(&html_s, &css_s, fonts) {
        Ok(pdf) => {
            set_error(error_code, ERR_SUCCESS);
            Box::into_raw(Box::new(pdf))
        },
        Err(e) => {
            set_error(error_code, classify_error(&e));
            ptr::null_mut()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    // classify_error must map each error variant to the correct FFI code.
    // Previously Error::Barcode fell through to ERR_INTERNAL (5) via string
    // matching; now variant matching guarantees _ERR_UNSUPPORTED (8).

    #[test]
    fn classify_barcode_error_returns_unsupported() {
        let e = Error::Barcode("requires barcodes feature".into());
        assert_eq!(classify_error(&e), _ERR_UNSUPPORTED);
    }

    #[test]
    fn classify_unsupported_error_returns_unsupported() {
        let e = Error::Unsupported("not supported".into());
        assert_eq!(classify_error(&e), _ERR_UNSUPPORTED);
    }

    #[test]
    fn classify_unsupported_filter_returns_unsupported() {
        let e = Error::UnsupportedFilter("JBIG2".into());
        assert_eq!(classify_error(&e), _ERR_UNSUPPORTED);
    }

    #[test]
    fn classify_io_error_returns_io() {
        let e = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
        assert_eq!(classify_error(&e), ERR_IO);
    }

    #[test]
    fn classify_parse_error_returns_parse() {
        let e = Error::ParseError {
            offset: 0,
            reason: "bad token".into(),
        };
        assert_eq!(classify_error(&e), ERR_PARSE);
    }

    #[test]
    fn classify_invalid_pdf_returns_parse() {
        let e = Error::InvalidPdf("corrupt".into());
        assert_eq!(classify_error(&e), ERR_PARSE);
    }

    #[test]
    fn classify_invalid_operation_returns_invalid_arg() {
        let e = Error::InvalidOperation("called on closed doc".into());
        assert_eq!(classify_error(&e), ERR_INVALID_ARG);
    }

    #[test]
    fn classify_layout_analysis_returns_extraction() {
        let e = Error::LayoutAnalysis("layout failed".into());
        assert_eq!(classify_error(&e), ERR_EXTRACTION);
    }

    // ── FFI builder/page helpers ─────────────────────────────────────────

    unsafe fn new_test_builder() -> *mut FfiDocumentBuilder {
        let mut ec = 0i32;
        let b = pdf_document_builder_create(&mut ec);
        assert!(!b.is_null(), "pdf_document_builder_create ec={ec}");
        b
    }

    unsafe fn new_test_page(b: *mut FfiDocumentBuilder) -> *mut FfiPageBuilder {
        let mut ec = 0i32;
        let p = pdf_document_builder_a4_page(b, &mut ec);
        assert!(!p.is_null(), "pdf_document_builder_a4_page ec={ec}");
        p
    }

    /// Free without replaying — suitable for tests that don't need the PDF bytes.
    unsafe fn free_test_pair(b: *mut FfiDocumentBuilder, p: *mut FfiPageBuilder) {
        pdf_page_builder_free(p);
        pdf_document_builder_free(b);
    }

    // ── Dashed stroke tests ──────────────────────────────────────────────

    #[test]
    fn stroke_rect_dashed_returns_success() {
        unsafe {
            let b = new_test_builder();
            let p = new_test_page(b);
            let dash = [5.0f32, 3.0f32];
            let mut ec = 0i32;
            let rc = pdf_page_builder_stroke_rect_dashed(
                p,
                10.0,
                10.0,
                200.0,
                100.0,
                2.0,
                0.0,
                0.0,
                1.0,
                dash.as_ptr(),
                dash.len(),
                0.0,
                &mut ec,
            );
            assert_eq!(rc, 0, "expected 0, ec={ec}");
            assert_eq!(ec, ERR_SUCCESS);
            free_test_pair(b, p);
        }
    }

    #[test]
    fn stroke_line_dashed_returns_success() {
        unsafe {
            let b = new_test_builder();
            let p = new_test_page(b);
            let dash = [4.0f32, 2.0f32];
            let mut ec = 0i32;
            let rc = pdf_page_builder_stroke_line_dashed(
                p,
                10.0,
                10.0,
                200.0,
                10.0,
                1.5,
                0.0,
                0.5,
                0.5,
                dash.as_ptr(),
                dash.len(),
                1.0,
                &mut ec,
            );
            assert_eq!(rc, 0, "expected 0, ec={ec}");
            assert_eq!(ec, ERR_SUCCESS);
            free_test_pair(b, p);
        }
    }

    #[test]
    fn stroke_rect_dashed_null_dash_array_is_solid() {
        // NULL dash array + zero length = solid stroke (no error).
        unsafe {
            let b = new_test_builder();
            let p = new_test_page(b);
            let mut ec = 0i32;
            let rc = pdf_page_builder_stroke_rect_dashed(
                p,
                0.0,
                0.0,
                100.0,
                50.0,
                1.0,
                0.0,
                0.0,
                0.0,
                std::ptr::null(),
                0,
                0.0,
                &mut ec,
            );
            assert_eq!(rc, 0, "ec={ec}");
            free_test_pair(b, p);
        }
    }

    // ── StreamingTable batch-tracking tests ──────────────────────────────

    unsafe fn begin_single_col_table(p: *mut FfiPageBuilder) {
        let hdr = b"Col\0";
        let hdrs = [hdr.as_ptr() as *const c_char];
        let widths = [100.0f32];
        let aligns = [0i32];
        let mut ec = 0i32;
        pdf_page_builder_streaming_table_begin_v2(
            p,
            1,
            hdrs.as_ptr(),
            widths.as_ptr(),
            aligns.as_ptr(),
            1,
            0,
            20,
            0.0,
            9999.0,
            1,
            &mut ec,
        );
        assert_eq!(ec, ERR_SUCCESS, "streaming_table_begin_v2 failed");
    }

    unsafe fn push_str_row(p: *mut FfiPageBuilder, val: &str) {
        let s = format!("{val}\0");
        let ptr = s.as_ptr() as *const c_char;
        let cells = [ptr];
        let mut ec = 0i32;
        pdf_page_builder_streaming_table_push_row(p, 1, cells.as_ptr(), &mut ec);
        assert_eq!(ec, ERR_SUCCESS, "push_row failed for {val:?}");
    }

    #[test]
    fn streaming_table_set_batch_size_resets_counts() {
        unsafe {
            let b = new_test_builder();
            let p = new_test_page(b);
            begin_single_col_table(p);
            let mut ec = 0i32;
            pdf_page_builder_streaming_table_set_batch_size(p, 5, &mut ec);
            assert_eq!(ec, ERR_SUCCESS);
            assert_eq!(pdf_page_builder_streaming_table_pending_row_count(p), 0);
            assert_eq!(pdf_page_builder_streaming_table_batch_count(p), 0);
            pdf_page_builder_streaming_table_finish(p, &mut ec);
            free_test_pair(b, p);
        }
    }

    #[test]
    fn streaming_table_auto_flush_at_batch_boundary() {
        // batch_size=3, push 7 rows → 2 auto-flushes (at row 3 and row 6),
        // 1 row pending.
        unsafe {
            let b = new_test_builder();
            let p = new_test_page(b);
            begin_single_col_table(p);
            let mut ec = 0i32;
            pdf_page_builder_streaming_table_set_batch_size(p, 3, &mut ec);
            assert_eq!(ec, ERR_SUCCESS);

            for i in 0..7 {
                push_str_row(p, &format!("r{i}"));
            }
            assert_eq!(pdf_page_builder_streaming_table_batch_count(p), 2);
            assert_eq!(pdf_page_builder_streaming_table_pending_row_count(p), 1);

            pdf_page_builder_streaming_table_finish(p, &mut ec);
            free_test_pair(b, p);
        }
    }

    #[test]
    fn streaming_table_explicit_flush_marks_boundary() {
        unsafe {
            let b = new_test_builder();
            let p = new_test_page(b);
            begin_single_col_table(p);
            let mut ec = 0i32;
            pdf_page_builder_streaming_table_set_batch_size(p, 100, &mut ec);

            push_str_row(p, "a");
            push_str_row(p, "b");
            assert_eq!(pdf_page_builder_streaming_table_batch_count(p), 0);
            assert_eq!(pdf_page_builder_streaming_table_pending_row_count(p), 2);

            pdf_page_builder_streaming_table_flush(p, &mut ec);
            assert_eq!(ec, ERR_SUCCESS);
            assert_eq!(pdf_page_builder_streaming_table_batch_count(p), 1);
            assert_eq!(pdf_page_builder_streaming_table_pending_row_count(p), 0);

            pdf_page_builder_streaming_table_finish(p, &mut ec);
            free_test_pair(b, p);
        }
    }

    #[test]
    fn streaming_table_finish_records_final_batch() {
        // finish() should bump batch_count if rows are still pending.
        // After finish the counts reset (st_batch_size = 0).
        unsafe {
            let b = new_test_builder();
            let p = new_test_page(b);
            begin_single_col_table(p);
            let mut ec = 0i32;
            pdf_page_builder_streaming_table_set_batch_size(p, 100, &mut ec);
            push_str_row(p, "only");
            assert_eq!(pdf_page_builder_streaming_table_pending_row_count(p), 1);
            pdf_page_builder_streaming_table_finish(p, &mut ec);
            assert_eq!(ec, ERR_SUCCESS);
            // After finish the state is reset; pending=0
            assert_eq!(pdf_page_builder_streaming_table_pending_row_count(p), 0);
            free_test_pair(b, p);
        }
    }
}
