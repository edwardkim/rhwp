//! Streaming-table scaling benchmark (issue #393 release gate).
//!
//! Validates that `StreamingTable::push_row` is **O(1) amortized per row**
//! — i.e. the Rust implementation does not regress to MigraDoc's O(rows²)
//! failure mode at 30 k rows. The decision doc
//! (`docs/v0.3.39/design/393_tables_decision.md`) commits to this as the
//! release gate.
//!
//! We run `StreamingTable` at 1 k, 5 k, 10 k, 30 k rows and expose each
//! measurement via criterion. A follow-up analysis step (not enforced
//! here — CI gate is the fact that all four scale points complete in
//! bounded time; the ratio is verified by `#[test]` in
//! `src/writer/streaming_table.rs::tests::test_streaming_table_*`) can
//! read the target JSON and assert scaling ratio < ~1.5× per 10 k rows.
//!
//! Row payload: 5 columns (SKU text, name text, qty integer right-aligned,
//! price currency right-aligned, status text). Widths are explicit
//! (`TableMode::Fixed`) so no content-driven autofit pass runs.
//!
//! Designed to complete locally in <10 s and in CI (bench smoke job) in
//! <30 s at the 5 k size. The 30 k size is `--bench`-only; CI runs the
//! 5 k variant for regression gating.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pdf_oxide::writer::{CellAlign, DocumentBuilder, StreamingColumn, StreamingTableConfig};
use std::hint::black_box;

/// Build a 5-column `StreamingTable` with `n` rows, return the produced
/// PDF size. Returning the size prevents the optimizer from eliding the
/// build pipeline.
fn build_streaming_table_pdf(n: usize) -> usize {
    let mut doc = DocumentBuilder::new();
    {
        let page = doc.letter_page().font("Helvetica", 8.0).at(72.0, 720.0);
        let mut t = page.streaming_table(
            StreamingTableConfig::new()
                .column(StreamingColumn::new("SKU").width_pt(72.0))
                .column(StreamingColumn::new("Name").width_pt(160.0))
                .column(
                    StreamingColumn::new("Qty")
                        .width_pt(48.0)
                        .align(CellAlign::Right),
                )
                .column(
                    StreamingColumn::new("Price")
                        .width_pt(72.0)
                        .align(CellAlign::Right),
                )
                .column(StreamingColumn::new("Status").width_pt(60.0))
                .repeat_header(true),
        );
        for i in 0..n {
            // Pre-computed strings; the benchmark measures pdf_oxide
            // code, not format!(). But we do need realistic content to
            // stress wrapping + measurement.
            let sku = format!("SKU-{:06}", i);
            let name = "Generic product line";
            let qty = (i % 100).to_string();
            let price = format!("{}.{:02}", i / 100, i % 100);
            let status = if i % 3 == 0 { "OK" } else { "PENDING" };
            t.push_row(|r| {
                r.cell(sku);
                r.cell(name);
                r.cell(qty);
                r.cell(price);
                r.cell(status);
            })
            .expect("push_row must not fail on bounded-memory streaming");
        }
        t.finish().done();
    }
    doc.build().expect("document must build").len()
}

fn bench_streaming_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_table_scaling");
    // Report throughput in rows/sec so the criterion report is
    // interpretable against the #393 acceptance gate.
    group.sample_size(10);

    for size in [1_000usize, 5_000, 10_000, 30_000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let produced = build_streaming_table_pdf(black_box(size));
                black_box(produced);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_streaming_scaling);
criterion_main!(benches);
