//! Internal bounded pipe protocol; only the supervisor commits the output file.
use rhwp::parser::pdf::{PdfImportLayout, PdfImportOptions, PdfImportPolicy};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Request {
    pub nonce: String,
    pub input: String,
    pub hwpx: bool,
    #[serde(default)]
    pub password: Option<String>,
    pub source_pages: Option<Vec<usize>>,
    pub table_inference: bool,
    pub layout: String,
    pub policy: PdfImportPolicy,
    pub output_bytes: u64,
    pub diagnostics_bytes: u64,
}

pub(super) fn main() -> i32 {
    match execute() {
        Ok(()) => crate::EXIT_OK,
        Err((code, error)) => {
            eprintln!("오류: {error}");
            code
        }
    }
}

fn execute() -> Result<(), (i32, String)> {
    let runtime = |message: &str| (crate::EXIT_RUNTIME, message.to_owned());
    let nonce = std::env::var("RHWP_PDF_WORKER_NONCE")
        .ok()
        .filter(|value| value.len() == 64)
        .ok_or_else(|| runtime("PDF_PROCESS_PROTOCOL: supervisor handshake required"))?;
    std::env::remove_var("RHWP_PDF_WORKER_NONCE");
    let mut request = Vec::new();
    std::io::stdin()
        .take(64 * 1024 + 1)
        .read_to_end(&mut request)
        .map_err(|_| runtime("PDF_PROCESS_PROTOCOL"))?;
    if request.len() > 64 * 1024 {
        return Err(runtime("PDF_PROCESS_PROTOCOL"));
    }
    let request: Request =
        serde_json::from_slice(&request).map_err(|_| runtime("PDF_PROCESS_PROTOCOL"))?;
    if request.nonce != nonce {
        return Err(runtime(
            "PDF_PROCESS_PROTOCOL: supervisor handshake mismatch",
        ));
    }
    request
        .policy
        .validate()
        .map_err(|e| runtime(&e.to_string()))?;
    let ceiling = super::supervisor::ExecutionLimits::default();
    if request.output_bytes == 0
        || request.output_bytes > ceiling.output_bytes
        || request.diagnostics_bytes == 0
        || request.diagnostics_bytes > ceiling.diagnostics_bytes
    {
        return Err(runtime("PDF_RESOURCE_POLICY"));
    }
    if !request.hwpx && request.output_bytes < 512 {
        return Err(runtime(
            "PDF_PROCESS_OUTPUT_LIMIT: HWP requires a 512-byte header",
        ));
    }
    let output_error = |error: rhwp::error::HwpError| {
        let message = error.to_string();
        // Preserve the CLI's stable output-limit code when a bounded producer
        // refuses before the pipe transport sees any bytes. Other request
        // budget failures retain their own resource code and details.
        if message.contains("SERIALIZE_RESOURCE_LIMIT: output byte limit")
            || message.contains("SERIALIZE_RESOURCE_LIMIT: output seek limit")
            || message.contains("SERIALIZE_RESOURCE_LIMIT: CFB output byte limit")
            || message.contains("PDF_RESOURCE_LIMIT: frontend JSON output bytes")
        {
            runtime(&format!("PDF_PROCESS_OUTPUT_LIMIT: {message}"))
        } else {
            runtime(&message)
        }
    };
    let (isolation, input_file) = rhwp::resource_supervisor::pdf_confinement::confine_pdf_worker(
        std::path::Path::new(&request.input),
    )
    .map_err(|error| runtime(&format!("PDF_PROCESS_SANDBOX_UNAVAILABLE: {error}")))?;
    let input = input_file;
    let mut bytes = Vec::new();
    input
        .take(request.policy.max_input_bytes() as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| runtime(&e.to_string()))?;
    let layout = PdfImportLayout::parse(&request.layout)
        .ok_or_else(|| runtime("PDF_IMPORT_OPTIONS: invalid layout"))?;
    let mut session =
        rhwp::PdfImportSession::new(request.policy).map_err(|error| runtime(&error.to_string()))?;
    let mut document = session
        .open(
            &bytes,
            &PdfImportOptions {
                password: request.password,
                source_pages: request.source_pages,
                include_table_inference: request.table_inference,
                layout,
            },
        )
        .map_err(|error| {
            let usage = matches!(&error, rhwp::error::HwpError::InvalidFile(message)
            if message.starts_with("PDF 오류: PDF_PAGE_SELECTION:")
                || message.starts_with("PDF 오류: PDF_PASSWORD_REQUIRED:"));
            (
                if usage {
                    crate::EXIT_USAGE
                } else {
                    crate::EXIT_RUNTIME
                },
                error.to_string(),
            )
        })?;
    let output = if request.hwpx {
        session.export_hwpx(&mut document, request.output_bytes as usize)
    } else {
        session.export_hwp(&mut document, request.output_bytes as usize)
    }
    .map_err(output_error)?;
    if output.len() as u64 > request.output_bytes {
        return Err(runtime(
            "PDF_PROCESS_OUTPUT_LIMIT: serialized output exceeds request limit",
        ));
    }
    let metadata = session
        .diagnostics_json(&document, request.diagnostics_bytes as usize)
        .map_err(output_error)?;
    let isolation = serde_json::to_vec(&isolation).map_err(|_| runtime("PDF_PROCESS_PROTOCOL"))?;
    const PREFIX: &[u8] = b"{\"isolation\":";
    let metadata_tail = metadata
        .strip_prefix(b"{")
        .ok_or_else(|| runtime("PDF_PROCESS_PROTOCOL"))?;
    let metadata_length = metadata
        .len()
        .checked_add(PREFIX.len())
        .and_then(|length| length.checked_add(isolation.len()))
        .ok_or_else(|| runtime("PDF_PROCESS_OUTPUT_LIMIT"))?;
    if metadata_length as u64 > request.diagnostics_bytes {
        return Err(runtime(
            "PDF_PROCESS_OUTPUT_LIMIT: diagnostics exceed request limit",
        ));
    }
    let mut stdout = std::io::stdout().lock();
    stdout
        .write_all(&(metadata_length as u64).to_le_bytes())
        .and_then(|_| stdout.write_all(PREFIX))
        .and_then(|_| stdout.write_all(&isolation))
        .and_then(|_| stdout.write_all(b","))
        .and_then(|_| stdout.write_all(metadata_tail))
        .and_then(|_| stdout.write_all(&output))
        .and_then(|_| stdout.flush())
        .map_err(|_| runtime("PDF_PROCESS_IO_FAILED"))
}

pub(super) fn decode_response(
    bytes: &[u8],
    limits: super::supervisor::ExecutionLimits,
) -> Result<(ResponseMetadata<'_>, &[u8]), &'static str> {
    let length: [u8; 8] = bytes
        .get(..8)
        .ok_or("PDF_PROCESS_PROTOCOL")?
        .try_into()
        .map_err(|_| "PDF_PROCESS_PROTOCOL")?;
    let length = u64::from_le_bytes(length);
    if length > limits.diagnostics_bytes {
        return Err("PDF_PROCESS_OUTPUT_LIMIT");
    }
    let length = usize::try_from(length).map_err(|_| "PDF_PROCESS_PROTOCOL")?;
    let metadata = bytes.get(8..8 + length).ok_or("PDF_PROCESS_PROTOCOL")?;
    let output = bytes.get(8 + length..).ok_or("PDF_PROCESS_PROTOCOL")?;
    if output.len() as u64 > limits.output_bytes {
        return Err("PDF_PROCESS_OUTPUT_LIMIT");
    }
    if metadata.first() != Some(&b'{') {
        return Err("PDF_PROCESS_PROTOCOL");
    }
    // The large document-derived diagnostic stays a validated borrowed JSON
    // slice. Rebuilding it as Value in the unsupervised parent would multiply
    // its node/string allocations outside the worker's address-space limit.
    let metadata: ResponseMetadata<'_> =
        serde_json::from_slice(metadata).map_err(|_| "PDF_PROCESS_PROTOCOL")?;
    if metadata.page_count == 0 {
        return Err("PDF_PROCESS_PROTOCOL");
    }
    Ok((metadata, output))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ResponseMetadata<'a> {
    pub page_count: u32,
    #[serde(borrow)]
    pub diagnostics: &'a serde_json::value::RawValue,
    #[serde(default)]
    pub isolation: rhwp::resource_supervisor::pdf_confinement::Confinement,
}
