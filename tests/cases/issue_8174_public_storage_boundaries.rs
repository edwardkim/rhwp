//! Observable storage boundaries; no private Storage or budget mutation API.
#![cfg(feature = "pdf-import")]
use rhwp::{parser::pdf::PdfImportPolicy, PdfImportSession};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
const PDF: &[u8] = include_bytes!("../../pdf/issue8174/fonts/cff-cid.pdf");

#[derive(Debug)]
struct Resolver {
    calls: Arc<AtomicUsize>,
    queries: Arc<AtomicUsize>,
    hint: Option<usize>,
}
impl rhwp::model::bin_data::BinDataResolver for Resolver {
    fn stable_size_bound(&self, _: &str) -> Option<usize> {
        self.queries.fetch_add(1, Ordering::SeqCst);
        self.hint
    }
    fn resolve(&self, _: &str) -> Vec<u8> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        vec![0x42; 4096]
    }
    fn resolve_limited(&self, key: &str, maximum: usize) -> Option<Vec<u8>> {
        (maximum >= 4096).then(|| self.resolve(key))
    }
}
fn attach(
    core: &mut rhwp::DocumentCore,
    hint: Option<usize>,
) -> (Arc<AtomicUsize>, Arc<AtomicUsize>) {
    use rhwp::model::bin_data::{BinDataBytes, BinDataContent};
    let calls = Arc::new(AtomicUsize::new(0));
    let queries = Arc::new(AtomicUsize::new(0));
    core.document_mut().bin_data_content.push(BinDataContent {
        id: 77,
        extension: "bin".into(),
        data: BinDataBytes::Lazy {
            resolver: Arc::new(Resolver {
                calls: calls.clone(),
                queries: queries.clone(),
                hint,
            }),
            key: "fixture".into(),
        },
    });
    (calls, queries)
}

#[test]
fn lazy_asset_reservation_precedes_resolution_for_known_and_opaque_bounds() {
    let mut healthy = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = healthy.open(PDF, &Default::default()).unwrap();
    let initial = healthy.resource_usage().decoded_bytes;
    let (calls, queries) = attach(&mut core, Some(4096));
    let expected = healthy.export_hwp(&mut core, 1024 * 1024).unwrap();
    assert!(queries.load(Ordering::SeqCst) > 0 && calls.load(Ordering::SeqCst) > 0);
    let full = healthy.resource_usage().decoded_bytes;
    let mut exercised = None;
    for cap in (initial..full).step_by(256) {
        let mut policy = PdfImportPolicy::default();
        policy.backend.decoded_bytes = cap;
        let mut request = PdfImportSession::new(policy).unwrap();
        let Ok(mut core) = request.open(PDF, &Default::default()) else {
            continue;
        };
        let (calls, queries) = attach(&mut core, Some(4096));
        if request.export_hwp(&mut core, 1024 * 1024).is_err()
            && queries.load(Ordering::SeqCst) > 0
            && calls.load(Ordering::SeqCst) == 0
        {
            assert!(request.resource_usage().failure.is_some());
            assert!(request.resource_usage().decoded_bytes <= cap);
            assert!(request.export_hwp(&mut core, 1024 * 1024).is_err());
            assert_eq!(calls.load(Ordering::SeqCst), 0);
            exercised = Some(cap);
            break;
        }
    }
    let cap = exercised.expect("real first-save asset reservation was not exercised");
    let mut policy = PdfImportPolicy::default();
    policy.backend.decoded_bytes = cap;
    let mut request = PdfImportSession::new(policy).unwrap();
    let mut core = request.open(PDF, &Default::default()).unwrap();
    let (calls, queries) = attach(&mut core, None);
    assert!(request.export_hwp(&mut core, 1024 * 1024).is_err());
    assert!(queries.load(Ordering::SeqCst) > 0);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let mut fresh = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let mut core = fresh.open(PDF, &Default::default()).unwrap();
    let (calls, _) = attach(&mut core, None);
    assert_eq!(fresh.export_hwp(&mut core, 1024 * 1024).unwrap(), expected);
    assert!(calls.load(Ordering::SeqCst) > 0);
}

#[test]
fn actual_diagnostics_keep_value_numeric_semantics_and_exact_reserved_backing() {
    let mut request = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let core = request
        .open(
            PDF,
            &rhwp::parser::pdf::PdfImportOptions {
                include_table_inference: true,
                ..Default::default()
            },
        )
        .unwrap();
    let expected =
        serde_json::json!({"pageCount":core.page_count(),"diagnostics":core.pdf_metadata()});
    let before = request.resource_usage().decoded_bytes;
    let bytes = request.diagnostics_json(&core, 1024 * 1024).unwrap();
    // Compare two independently encoded JSON values after the same JSON
    // parser. Its float_roundtrip feature is off; an in-memory f64 can differ
    // by one ULP from that parser's result even for an identical wire number.
    let expected: serde_json::Value =
        serde_json::from_slice(&serde_json::to_vec(&expected).unwrap()).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&bytes).unwrap(),
        expected
    );
    assert_eq!(
        request.resource_usage().decoded_bytes,
        before + bytes.len() as u64
    );
    let mut low = PdfImportSession::new(PdfImportPolicy::default()).unwrap();
    let core = low
        .open(
            PDF,
            &rhwp::parser::pdf::PdfImportOptions {
                include_table_inference: true,
                ..Default::default()
            },
        )
        .unwrap();
    let before = low.resource_usage().decoded_bytes;
    assert!(low.diagnostics_json(&core, bytes.len() - 1).is_err());
    assert_eq!(low.resource_usage().decoded_bytes, before);
    assert!(low.diagnostics_json(&core, 1024 * 1024).is_err());
}
