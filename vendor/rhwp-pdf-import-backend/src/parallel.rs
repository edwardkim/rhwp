//! Parallel page extraction using rayon.
//!
//! This module provides parallel text and markdown extraction from PDF documents.
//! Because [`PdfDocument`] uses `BufReader<File>` and `&mut self` throughout,
//! it is neither `Send` nor `Sync`. Rather than redesigning the core struct, this
//! module takes a pragmatic approach: each rayon task opens its own `PdfDocument`
//! instance (with its own `BufReader`). The global font cache (keyed by font
//! identity hash) means fonts are parsed only once and shared across all instances.
//!
//! # Feature flag
//!
//! This module is gated behind the `parallel` feature:
//!
//! ```toml
//! [dependencies]
//! pdf_oxide = { version = "0.3", features = ["parallel"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use std::path::Path;
//! use pdf_oxide::parallel::ParallelExtractor;
//!
//! let pages = ParallelExtractor::extract_all_text(Path::new("large.pdf"))?;
//! for (i, text) in pages.iter().enumerate() {
//!     println!("--- Page {} ---\n{}", i + 1, text);
//! }
//! ```

use std::path::Path;

use rayon::prelude::*;

use crate::converters::ConversionOptions;
use crate::document::PdfDocument;
use crate::error::{Error, Result};

/// Parallel extractor that processes PDF pages across multiple threads.
///
/// Uses a batched strategy: divides pages into chunks and each rayon worker
/// opens a single [`PdfDocument`] instance to process its chunk sequentially.
/// This amortizes the cost of document opening, xref parsing, page tree walks,
/// and font loading across many pages instead of paying it per-page.
///
/// All results are returned in page order regardless of which thread processed
/// each page.
pub struct ParallelExtractor;

impl ParallelExtractor {
    /// Extract plain text from every page of a PDF in parallel.
    ///
    /// Opens the document once on the calling thread to determine the page count,
    /// then divides pages into batches distributed across rayon worker threads.
    /// Each worker opens a single `PdfDocument` and extracts all pages in its batch.
    ///
    /// Returns a `Vec<String>` with one entry per page, in page order.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered by any worker. If multiple workers
    /// fail, only one error is propagated (rayon semantics).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::path::Path;
    /// use pdf_oxide::parallel::ParallelExtractor;
    ///
    /// let pages = ParallelExtractor::extract_all_text(Path::new("report.pdf"))?;
    /// assert_eq!(pages.len(), 42);
    /// ```
    pub fn extract_all_text(path: &Path) -> Result<Vec<String>> {
        let page_count = Self::get_page_count(path)?;
        if page_count == 0 {
            return Ok(Vec::new());
        }
        let path_buf = path.to_path_buf();

        // Divide pages into batches for parallel processing.
        // Each batch is processed by a single PdfDocument instance, amortizing
        // the cost of document open, xref parse, page tree walk, and font loading.
        let num_threads = rayon::current_num_threads().max(1);
        let batch_size = page_count.div_ceil(num_threads);
        let batches: Vec<(usize, usize)> = (0..page_count)
            .step_by(batch_size)
            .map(|start| (start, (start + batch_size).min(page_count)))
            .collect();

        let batch_results: std::result::Result<Vec<Vec<(usize, String)>>, Error> = batches
            .into_par_iter()
            .map(|(start, end)| {
                let doc = PdfDocument::open(&path_buf)?;
                let mut results = Vec::with_capacity(end - start);
                for page_index in start..end {
                    let text = doc.extract_text(page_index)?;
                    results.push((page_index, text));
                }
                Ok(results)
            })
            .collect();

        // Flatten and sort by page index to guarantee order
        let mut all_results: Vec<(usize, String)> = batch_results?.into_iter().flatten().collect();
        all_results.sort_unstable_by_key(|(idx, _)| *idx);
        Ok(all_results.into_iter().map(|(_, text)| text).collect())
    }

    /// Extract Markdown from every page of a PDF in parallel.
    ///
    /// Behaves like [`extract_all_text`](Self::extract_all_text) but converts
    /// each page to Markdown using the supplied [`ConversionOptions`].
    ///
    /// Returns a `Vec<String>` with one entry per page, in page order.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered by any worker.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::path::Path;
    /// use pdf_oxide::parallel::ParallelExtractor;
    /// use pdf_oxide::converters::ConversionOptions;
    ///
    /// let opts = ConversionOptions::default();
    /// let pages = ParallelExtractor::extract_all_markdown(
    ///     Path::new("report.pdf"),
    ///     &opts,
    /// )?;
    /// ```
    pub fn extract_all_markdown(path: &Path, options: &ConversionOptions) -> Result<Vec<String>> {
        let page_count = Self::get_page_count(path)?;
        if page_count == 0 {
            return Ok(Vec::new());
        }
        let path_buf = path.to_path_buf();
        let options = options.clone();

        let num_threads = rayon::current_num_threads().max(1);
        let batch_size = page_count.div_ceil(num_threads);
        let batches: Vec<(usize, usize)> = (0..page_count)
            .step_by(batch_size)
            .map(|start| (start, (start + batch_size).min(page_count)))
            .collect();

        let batch_results: std::result::Result<Vec<Vec<(usize, String)>>, Error> = batches
            .into_par_iter()
            .map(|(start, end)| {
                let doc = PdfDocument::open(&path_buf)?;
                let mut results = Vec::with_capacity(end - start);
                for page_index in start..end {
                    let md = doc.to_markdown(page_index, &options)?;
                    results.push((page_index, md));
                }
                Ok(results)
            })
            .collect();

        let mut all_results: Vec<(usize, String)> = batch_results?.into_iter().flatten().collect();
        all_results.sort_unstable_by_key(|(idx, _)| *idx);
        Ok(all_results.into_iter().map(|(_, md)| md).collect())
    }

    /// Determine the page count by opening the document once.
    fn get_page_count(path: &Path) -> Result<usize> {
        let doc = PdfDocument::open(path)?;
        doc.page_count()
    }
}

/// Convenience function: extract plain text from all pages in parallel.
///
/// This is a free function wrapper around
/// [`ParallelExtractor::extract_all_text`].
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use pdf_oxide::parallel::extract_all_text_parallel;
///
/// let pages = extract_all_text_parallel(Path::new("report.pdf"))?;
/// ```
pub fn extract_all_text_parallel(path: &Path) -> Result<Vec<String>> {
    ParallelExtractor::extract_all_text(path)
}

/// Convenience function: extract Markdown from all pages in parallel.
///
/// This is a free function wrapper around
/// [`ParallelExtractor::extract_all_markdown`].
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use pdf_oxide::parallel::{extract_all_markdown_parallel};
/// use pdf_oxide::converters::ConversionOptions;
///
/// let opts = ConversionOptions::default();
/// let pages = extract_all_markdown_parallel(Path::new("report.pdf"), &opts)?;
/// ```
pub fn extract_all_markdown_parallel(
    path: &Path,
    options: &ConversionOptions,
) -> Result<Vec<String>> {
    ParallelExtractor::extract_all_markdown(path, options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn test_parallel_text_extraction_simple() {
        let path = fixture_path("simple.pdf");
        if !path.exists() {
            eprintln!("Skipping test: fixture not found at {:?}", path);
            return;
        }

        let pages = ParallelExtractor::extract_all_text(&path).expect("extraction should succeed");

        // Verify we got at least one page
        assert!(!pages.is_empty(), "should extract at least one page");

        // Verify page count matches serial extraction
        let doc = PdfDocument::open(&path).unwrap();
        let expected_count = doc.page_count().unwrap();
        assert_eq!(pages.len(), expected_count);
    }

    #[test]
    fn test_parallel_text_matches_serial() {
        let path = fixture_path("simple.pdf");
        if !path.exists() {
            eprintln!("Skipping test: fixture not found at {:?}", path);
            return;
        }

        // Extract serially
        let doc = PdfDocument::open(&path).unwrap();
        let page_count = doc.page_count().unwrap();
        let serial: Vec<String> = (0..page_count)
            .map(|i| doc.extract_text(i).unwrap())
            .collect();

        // Extract in parallel
        let parallel =
            ParallelExtractor::extract_all_text(&path).expect("parallel extraction should succeed");

        assert_eq!(serial.len(), parallel.len());
        for (i, (s, p)) in serial.iter().zip(parallel.iter()).enumerate() {
            assert_eq!(s, p, "page {} text differs between serial and parallel", i);
        }
    }

    #[test]
    fn test_parallel_markdown_extraction() {
        let path = fixture_path("simple.pdf");
        if !path.exists() {
            eprintln!("Skipping test: fixture not found at {:?}", path);
            return;
        }

        let opts = ConversionOptions::default();
        let pages = ParallelExtractor::extract_all_markdown(&path, &opts)
            .expect("markdown extraction should succeed");

        assert!(!pages.is_empty(), "should extract at least one page");

        let doc = PdfDocument::open(&path).unwrap();
        let expected_count = doc.page_count().unwrap();
        assert_eq!(pages.len(), expected_count);
    }

    #[test]
    fn test_parallel_markdown_matches_serial() {
        let path = fixture_path("simple.pdf");
        if !path.exists() {
            eprintln!("Skipping test: fixture not found at {:?}", path);
            return;
        }

        let opts = ConversionOptions::default();

        // Extract serially
        let doc = PdfDocument::open(&path).unwrap();
        let page_count = doc.page_count().unwrap();
        let serial: Vec<String> = (0..page_count)
            .map(|i| doc.to_markdown(i, &opts).unwrap())
            .collect();

        // Extract in parallel
        let parallel = ParallelExtractor::extract_all_markdown(&path, &opts)
            .expect("parallel extraction should succeed");

        assert_eq!(serial.len(), parallel.len());
        for (i, (s, p)) in serial.iter().zip(parallel.iter()).enumerate() {
            assert_eq!(s, p, "page {} markdown differs between serial and parallel", i);
        }
    }

    #[test]
    fn test_parallel_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/file.pdf");
        let result = ParallelExtractor::extract_all_text(&path);
        assert!(result.is_err(), "should error on missing file");
    }

    #[test]
    fn test_convenience_functions() {
        let path = fixture_path("simple.pdf");
        if !path.exists() {
            eprintln!("Skipping test: fixture not found at {:?}", path);
            return;
        }

        let text_pages = extract_all_text_parallel(&path).expect("convenience fn should work");
        assert!(!text_pages.is_empty());

        let opts = ConversionOptions::default();
        let md_pages =
            extract_all_markdown_parallel(&path, &opts).expect("convenience fn should work");
        assert!(!md_pages.is_empty());
    }

    #[test]
    fn test_parallel_preserves_page_order() {
        let path = fixture_path("outline.pdf");
        if !path.exists() {
            eprintln!("Skipping test: fixture not found at {:?}", path);
            return;
        }

        // Extract serially
        let doc = PdfDocument::open(&path).unwrap();
        let page_count = doc.page_count().unwrap();
        if page_count < 2 {
            eprintln!("Skipping order test: need multi-page PDF");
            return;
        }
        let serial: Vec<String> = (0..page_count)
            .map(|i| doc.extract_text(i).unwrap())
            .collect();

        // Run parallel extraction several times to test ordering stability
        for _ in 0..5 {
            let parallel = ParallelExtractor::extract_all_text(&path).unwrap();
            assert_eq!(serial.len(), parallel.len());
            for (i, (s, p)) in serial.iter().zip(parallel.iter()).enumerate() {
                assert_eq!(s, p, "page {} text differs on repeated parallel run", i);
            }
        }
    }
}
