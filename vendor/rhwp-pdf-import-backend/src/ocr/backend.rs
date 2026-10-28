//! Inference-backend seam for OCR (#524).
//!
//! `TextDetector` / `TextRecognizer` used to call `ort` (native ONNX
//! Runtime) inline, which has no `wasm32` story. This module isolates
//! the single "run an ONNX graph" operation behind [`InferenceBackend`]
//! so the same detector/recognizer + pre/post-processing drive either:
//!
//! * [`OrtBackend`] — native ONNX Runtime (`ocr` feature), the
//!   default everywhere it is available; unchanged behaviour.
//! * [`TractBackend`] — pure-Rust `tract` (`ocr-tract` feature, which
//!   `ml` implies), the path the browser/Deno/edge `wasm32` build uses
//!   since it needs no native library and no JS bridge. Validated to
//!   load + run the PaddleOCR det/rec graphs (issue #524 Approach-B
//!   gate).
//!
//! Both consume ONNX model **bytes** and expose one call: a single
//! `f32` input tensor named `"x"` in, the first `f32` output tensor
//! out, as a dynamic-rank `ndarray`. All image normalization, box
//! extraction and CTC decoding stays shared in the sibling modules, so
//! the two backends are numerically comparable by construction.

use super::error::{OcrError, OcrResult};

/// One ONNX graph evaluation: `[N,C,H,W] f32` ("x") → first `f32`
/// output as a dynamic-rank array. Implementors must be `Send + Sync`
/// (the detector/recognizer are shared across threads on native).
pub(crate) trait InferenceBackend: Send + Sync {
    /// Run the graph on `input` and return the first output tensor.
    fn run(&self, input: &ndarray::Array4<f32>) -> OcrResult<ndarray::ArrayD<f32>>;
}

/// Build the backend appropriate for the current build: native ONNX
/// Runtime when the `ocr` feature is on, otherwise the pure-Rust
/// `tract` backend (`ocr-tract`, which `ml` implies and `wasm-ocr`
/// uses). `num_threads` is honoured only by the native backend.
#[allow(unused_variables)]
pub(crate) fn build_backend(
    model_bytes: &[u8],
    num_threads: usize,
) -> OcrResult<Box<dyn InferenceBackend>> {
    // Exactly one of these cfg blocks is compiled, and it is the
    // function's tail expression — no `return` needed (clippy-clean).
    #[cfg(feature = "ocr")]
    {
        Ok(Box::new(OrtBackend::from_bytes(model_bytes, num_threads)?))
    }
    #[cfg(all(not(feature = "ocr"), feature = "ocr-tract"))]
    {
        Ok(Box::new(TractBackend::from_bytes(model_bytes)?))
    }
    #[cfg(all(not(feature = "ocr"), not(feature = "ocr-tract")))]
    {
        Err(OcrError::ModelLoadError(
            "no OCR inference backend compiled in (enable `ocr` or `ocr-tract`)".to_string(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Native ONNX Runtime backend (`ort`).
// ---------------------------------------------------------------------------

#[cfg(feature = "ocr")]
pub(crate) struct OrtBackend {
    // `Mutex` because `ort::Session::run` needs `&mut` while the
    // detector/recognizer are shared `&self` across threads — exactly
    // the prior `TextDetector`/`TextRecognizer` ownership model.
    session: std::sync::Mutex<ort::session::Session>,
}

#[cfg(feature = "ocr")]
impl OrtBackend {
    pub(crate) fn from_bytes(model_bytes: &[u8], num_threads: usize) -> OcrResult<Self> {
        // wrap `ort::Session::builder()` (and the
        // builder chain) in `std::panic::catch_unwind` so a missing
        // `libonnxruntime.so` / `.dylib` / `.dll` (the default-wheel
        // case where ORT is not bundled) does NOT propagate as an
        // uncatchable `PanicException` across the PyO3 / JNI / N-API /
        // cgo / FFI boundary. The catch produces a clean
        // `OcrError::ModelLoadError` instead, which each binding's
        // wrapper translates to the appropriate language-native
        // exception (`OcrUnavailable` in Python / Java / Ruby / PHP /
        // Go / C# / Node / WASM per
        // `research-typed-signals-cross-lang.md` §13).
        //
        // Without this, ORT's lazy dylib load fires through
        // `ort::api()` inside `Session::builder()` and panics at
        // `.../ort-2.0.0-rc.11/src/lib.rs:191:41` with
        // "Failed to load ONNX Runtime dylib".
        // `&[u8]` is already `UnwindSafe`, and `AssertUnwindSafe`
        // additionally allows borrowing it through the closure without
        // an owned copy. A previous revision called `.to_vec()` first,
        // which copied the full 50-100 MB PaddleOCR detection model on
        // the OCR hot path for no safety benefit.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ort::session::Session::builder()
                .map_err(|e| {
                    OcrError::ModelLoadError(format!("Failed to create session builder: {}", e))
                })?
                .with_intra_threads(num_threads)
                .map_err(|e| OcrError::ModelLoadError(format!("Failed to set threads: {}", e)))?
                .commit_from_memory(model_bytes)
                .map_err(|e| OcrError::ModelLoadError(format!("Failed to load model: {}", e)))
        }));
        let session = match result {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => return Err(e),
            Err(panic_payload) => {
                let detail = panic_payload
                    .downcast::<String>()
                    .map(|b| *b)
                    .unwrap_or_else(|p| {
                        p.downcast::<&'static str>()
                            .map(|b| (*b).to_string())
                            .unwrap_or_else(|_| "unknown panic".to_string())
                    });
                return Err(OcrError::ModelLoadError(format!(
                    "ONNX Runtime initialisation panicked (typically: \
                     libonnxruntime failed to load). Install onnxruntime, then set \
                     ORT_DYLIB_PATH to the library FILE (e.g. \
                     .../libonnxruntime.dylib) — not a directory — or add its \
                     directory to DYLD_LIBRARY_PATH (macOS) / LD_LIBRARY_PATH \
                     (Linux). Detail: {}",
                    detail
                )));
            },
        };
        Ok(Self {
            session: std::sync::Mutex::new(session),
        })
    }
}

#[cfg(feature = "ocr")]
impl InferenceBackend for OrtBackend {
    fn run(&self, input: &ndarray::Array4<f32>) -> OcrResult<ndarray::ArrayD<f32>> {
        use ort::value::TensorRef;

        let mut session = self.session.lock().map_err(|e| {
            OcrError::InferenceError(format!("Failed to acquire session lock: {}", e))
        })?;

        let input_tensor = TensorRef::from_array_view(input).map_err(|e| {
            OcrError::InferenceError(format!("Failed to create input tensor: {}", e))
        })?;

        let outputs = session
            .run(ort::inputs!["x" => input_tensor])
            .map_err(|e| OcrError::InferenceError(format!("Inference failed: {}", e)))?;

        let (_, output_tensor) = outputs
            .iter()
            .next()
            .ok_or_else(|| OcrError::InferenceError("No output tensor found".to_string()))?;

        let view = output_tensor
            .try_extract_array::<f32>()
            .map_err(|e| OcrError::InferenceError(format!("Failed to extract output: {}", e)))?;

        // Own the data: the `outputs` (and its borrow of `session`) are
        // dropped at function end, so hand back an owned `ArrayD`.
        Ok(view.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Pure-Rust `tract` backend — the wasm32 path.
// ---------------------------------------------------------------------------

// When both `ocr` and `ocr-tract` are on (e.g. `--features ocr,ml`),
// the native `ort` backend wins in `build_backend`, so this type is
// compiled but unconstructed — intentional, not dead code. In a real
// `wasm-ocr` build (`ocr` off) it *is* constructed, so the allow is
// scoped to the combined-feature case only.
#[cfg(feature = "ocr-tract")]
#[cfg_attr(feature = "ocr", allow(dead_code))]
pub(crate) struct TractBackend {
    // The unoptimized inference graph. PaddleOCR det/rec have dynamic
    // H/W, so a plan is specialised + cached per concrete input shape
    // on first use (recognizer height is fixed at 48; detector pads to
    // /32, so distinct shapes are bounded in practice).
    model: tract_onnx::prelude::InferenceModel,
    plans: std::sync::Mutex<std::collections::HashMap<Vec<usize>, std::sync::Arc<TractPlan>>>,
}

#[cfg(feature = "ocr-tract")]
#[cfg_attr(feature = "ocr", allow(dead_code))]
type TractPlan = tract_onnx::prelude::TypedRunnableModel<tract_onnx::prelude::TypedModel>;

#[cfg(feature = "ocr-tract")]
#[cfg_attr(feature = "ocr", allow(dead_code))]
impl TractBackend {
    pub(crate) fn from_bytes(model_bytes: &[u8]) -> OcrResult<Self> {
        use tract_onnx::prelude::*;
        let model = tract_onnx::onnx()
            .model_for_read(&mut std::io::Cursor::new(model_bytes))
            .map_err(|e| OcrError::ModelLoadError(format!("tract: parse ONNX: {}", e)))?;
        Ok(Self {
            model,
            plans: std::sync::Mutex::new(std::collections::HashMap::new()),
        })
    }

    /// Specialise (or fetch a cached) runnable plan for `shape`.
    fn plan_for(&self, shape: &[usize]) -> OcrResult<std::sync::Arc<TractPlan>> {
        use tract_onnx::prelude::*;

        let key = shape.to_vec();
        let mut plans = self
            .plans
            .lock()
            .map_err(|e| OcrError::InferenceError(format!("tract: plan lock: {}", e)))?;
        if let Some(p) = plans.get(&key) {
            return Ok(p.clone());
        }
        // `into_optimized()` is mandatory, not a nicety: with only
        // `into_typed()` the DBNet detector graph is so slow on a
        // full-page image that a single inference effectively hangs
        // (empirically >5 min vs sub-second optimized — #524 task 5).
        let runnable = self
            .model
            .clone()
            .with_input_fact(0, f32::fact(shape).into())
            .map_err(|e| OcrError::InferenceError(format!("tract: input fact: {}", e)))?
            .into_optimized()
            .map_err(|e| OcrError::InferenceError(format!("tract: optimize: {}", e)))?
            .into_runnable()
            .map_err(|e| OcrError::InferenceError(format!("tract: runnable: {}", e)))?;
        let arc = std::sync::Arc::new(runnable);
        plans.insert(key, arc.clone());
        Ok(arc)
    }
}

#[cfg(feature = "ocr-tract")]
impl InferenceBackend for TractBackend {
    fn run(&self, input: &ndarray::Array4<f32>) -> OcrResult<ndarray::ArrayD<f32>> {
        use tract_onnx::prelude::*;

        let shape: Vec<usize> = input.shape().to_vec();
        let plan = self.plan_for(&shape)?;

        // Bridge via flat data + shape rather than ndarray types:
        // tract bundles its own `ndarray` version, so array types are
        // not interchangeable with this crate's `ndarray`. `.iter()`
        // yields logical C-order, matching `shape`, regardless of the
        // input's memory layout.
        let data: Vec<f32> = input.iter().copied().collect();
        let tensor = Tensor::from_shape(&shape, &data)
            .map_err(|e| OcrError::InferenceError(format!("tract: input tensor: {}", e)))?;

        let result = plan
            .run(tvec!(tensor.into()))
            .map_err(|e| OcrError::InferenceError(format!("tract: run: {}", e)))?;

        let out = result
            .into_iter()
            .next()
            .ok_or_else(|| OcrError::InferenceError("tract: no output tensor".to_string()))?;

        let out_shape: Vec<usize> = out.shape().to_vec();
        let out_data = out
            .as_slice::<f32>()
            .map_err(|e| OcrError::InferenceError(format!("tract: extract output: {}", e)))?;
        ndarray::ArrayD::from_shape_vec(ndarray::IxDyn(&out_shape), out_data.to_vec())
            .map_err(|e| OcrError::InferenceError(format!("tract: reshape output: {}", e)))
    }
}

// ---------------------------------------------------------------------------
// #524 task 5 — ort↔tract numerical-equivalence harness.
//
// Feeds an *identical* deterministic input tensor through both backends
// for the real PaddleOCR graphs and reports the output divergence. A
// large diff localizes the recognition garble to a tract inference op
// (vs. shared preprocessing/CTC, which cannot differ between backends).
// `#[ignore]`d: needs the model files and the ONNX Runtime dylib —
//   ORT_DYLIB_PATH=/path/libonnxruntime.so \
//   cargo test --features ocr,ml --lib backend::parity -- --ignored --nocapture
// ---------------------------------------------------------------------------
#[cfg(all(test, feature = "ocr", feature = "ocr-tract"))]
mod parity {
    use super::*;

    fn models_dir() -> std::path::PathBuf {
        std::env::var_os("PDF_OXIDE_MODEL_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var("HOME").expect("HOME"))
                    .join(".cache/pdf_oxide/models")
            })
    }

    /// Reproducible input; values span a typical normalized range so the
    /// graphs exercise real arithmetic (we test engine *agreement*, not
    /// OCR correctness — identical bytes go to both backends).
    fn deterministic_input(shape: [usize; 4]) -> ndarray::Array4<f32> {
        let n: usize = shape.iter().product();
        let v: Vec<f32> = (0..n).map(|i| (i as f32 * 0.013).sin() * 2.0).collect();
        ndarray::Array4::from_shape_vec(shape, v).expect("input shape")
    }

    fn diff(a: &ndarray::ArrayD<f32>, b: &ndarray::ArrayD<f32>) -> (f32, f64) {
        assert_eq!(
            a.shape(),
            b.shape(),
            "ort/tract output SHAPES differ: {:?} vs {:?}",
            a.shape(),
            b.shape()
        );
        let mut max = 0f32;
        let mut sum = 0f64;
        for (x, y) in a.iter().zip(b.iter()) {
            let d = (x - y).abs();
            max = max.max(d);
            sum += d as f64;
        }
        (max, sum / a.len().max(1) as f64)
    }

    #[test]
    #[ignore = "needs PDF_OXIDE_MODEL_DIR models + ORT_DYLIB_PATH"]
    fn ort_vs_tract_detector() {
        let m = std::fs::read(models_dir().join("det.onnx")).expect("det.onnx");
        let ort = OrtBackend::from_bytes(&m, 1).expect("ort det");
        let tract = TractBackend::from_bytes(&m).expect("tract det");
        let inp = deterministic_input([1, 3, 640, 640]);
        let o = ort.run(&inp).expect("ort run");
        let t = tract.run(&inp).expect("tract run");
        let (mx, mean) = diff(&o, &t);
        println!("DET  shape={:?}  max_abs_diff={mx:.6}  mean_abs_diff={mean:.6}", o.shape());
        assert!(mx < 1e-2, "detector ort/tract diverge: max_abs_diff={mx}");
    }

    #[test]
    #[ignore = "needs PDF_OXIDE_MODEL_DIR models + ORT_DYLIB_PATH"]
    fn ort_vs_tract_recognizer() {
        let m = std::fs::read(models_dir().join("rec.onnx")).expect("rec.onnx");
        let ort = OrtBackend::from_bytes(&m, 1).expect("ort rec");
        let tract = TractBackend::from_bytes(&m).expect("tract rec");
        let inp = deterministic_input([1, 3, 48, 320]);
        let o = ort.run(&inp).expect("ort run");
        let t = tract.run(&inp).expect("tract run");
        let (mx, mean) = diff(&o, &t);
        println!("REC  shape={:?}  max_abs_diff={mx:.6}  mean_abs_diff={mean:.6}", o.shape());
        assert!(mx < 1e-2, "recognizer ort/tract diverge: max_abs_diff={mx}");
    }
}
