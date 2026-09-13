//! File boundary for the shared native import engine. Never reopen source after hashing.
use rhwp::document_core::{DocumentCore, ImportParagraphBlockRequest};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs::File, io::Read, path::Path};

// Transport ceiling, distinct from the native decoded resource budgets.
const MAX_SOURCE_BYTES: u64 = 64 * 1024 * 1024;

pub(super) fn is_action(step: &Value) -> bool {
    step["action"] == "import_paragraph_block"
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    path: String,
    sha256: String,
}

pub(super) struct PreparedImport {
    source: DocumentCore,
    request: ImportParagraphBlockRequest,
    path: String,
    sha256: String,
}

pub(super) struct Failure(pub Value, pub i32);

fn usage(reason: impl ToString) -> Failure {
    Failure(
        json!({"invalid":[{"step":0,"action":"import_paragraph_block","reason":reason.to_string()}]}),
        2,
    )
}
fn runtime(reason: impl ToString) -> Failure {
    Failure(json!({"error":reason.to_string()}), 1)
}

impl PreparedImport {
    pub(super) fn load(step: &Value, input: &str, output: &str) -> Result<Self, Failure> {
        if step.to_string().len() > 8 * 1024 * 1024 {
            return Err(usage("import step exceeds 8 MiB"));
        }
        if step.as_object().is_none_or(|obj| {
            obj.keys()
                .any(|k| !["action", "source", "request", "if"].contains(&k.as_str()))
        }) {
            return Err(usage(
                "import step permits only action, source, request and if",
            ));
        }
        let source: Source = serde_json::from_value(step["source"].clone()).map_err(usage)?;
        let request = serde_json::from_value(step["request"].clone()).map_err(usage)?;
        if source.path.is_empty()
            || source.sha256.len() != 64
            || !source.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(usage(
                "source requires a nonempty path and a 64-digit hexadecimal sha256",
            ));
        }
        for destination in [input, output] {
            if crate::paths_refer_to_same_file(Path::new(&source.path), Path::new(destination)) {
                return Err(usage(
                    "source must be distinct from target input and output",
                ));
            }
        }
        if !std::fs::metadata(&source.path).map_err(runtime)?.is_file() {
            return Err(usage("source must be a regular file"));
        }
        let file = File::open(&source.path).map_err(runtime)?;
        let meta = file.metadata().map_err(runtime)?;
        if !meta.is_file() || meta.len() > MAX_SOURCE_BYTES {
            return Err(usage("source must be a regular file of at most 64 MiB"));
        }
        let mut bytes = Vec::new();
        file.take(MAX_SOURCE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(runtime)?;
        if bytes.len() as u64 > MAX_SOURCE_BYTES {
            return Err(usage("source exceeds 64 MiB"));
        }
        let actual = crate::cli::protocol::sha256_hex_of(&bytes);
        if !source.sha256.eq_ignore_ascii_case(&actual) {
            return Err(Failure(
                json!({"invalid":[],"error":"source SHA-256 mismatch; no execution or save",
                "preconditionFailed":{"kind":"sourceSha256","step":0,"expected":source.sha256.to_ascii_lowercase(),"actual":actual},
                "nextCall":{"name":"digest","arguments":[source.path,"--json"],"why":"Inspect the changed source and explicitly replan its range and fingerprint."}}),
                3,
            ));
        }
        let document = DocumentCore::from_bytes(&bytes).map_err(runtime)?;
        Ok(Self {
            source: document,
            request,
            path: source.path,
            sha256: actual,
        })
    }

    pub(super) fn protect_output(&self, output: &str) -> Result<(), String> {
        if crate::paths_refer_to_same_file(Path::new(&self.path), Path::new(output)) {
            Err("output aliases the import source; no save".into())
        } else {
            Ok(())
        }
    }

    pub(super) fn preview(
        &self,
        doc: &rhwp::wasm_api::HwpDocument,
        index: usize,
    ) -> Result<Value, String> {
        let result = doc
            .preview_paragraph_block_import_native(self.source.document(), &self.request)
            .map_err(|e| e.to_string())?;
        Ok(self.journal(index, result))
    }

    pub(super) fn execute(
        &self,
        doc: &mut rhwp::wasm_api::HwpDocument,
        index: usize,
    ) -> Result<Value, String> {
        let result = doc
            .import_paragraph_block_native(self.source.document(), &self.request)
            .map_err(|e| e.to_string())?;
        Ok(self.journal(index, result))
    }

    fn journal(&self, index: usize, result: impl serde::Serialize) -> Value {
        json!({"step":index,"action":"import_paragraph_block",
            "source":{"path":self.path,"sha256":self.sha256},
            "operationResult":{"action":"import_paragraph_block","result":result}})
    }
}
