//! Batch processing API for extracting text from multiple PDFs.
//!
//! Provides a high-level API for processing directories or lists of PDF files
//! with progress reporting and error collection. When the `parallel` feature
//! is enabled, uses rayon for parallel document processing.
//!
//! # Example
//!
//! ```no_run
//! use pdf_oxide::batch::{BatchProcessor, BatchResult};
//! use std::path::Path;
//!
//! let processor = BatchProcessor::new();
//! let results = processor.extract_text_from_files(&[
//!     Path::new("doc1.pdf"),
//!     Path::new("doc2.pdf"),
//! ]);
//!
//! for result in &results {
//!     match &result.text {
//!         Ok(text) => println!("{}: {} chars", result.path.display(), text.len()),
//!         Err(e) => eprintln!("{}: ERROR: {}", result.path.display(), e),
//!     }
//! }
//! ```

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::document::PdfDocument;
use crate::error::Error;

/// Result of processing a single PDF file.
#[derive(Debug)]
pub struct BatchResult {
    /// Path to the PDF file that was processed.
    pub path: PathBuf,
    /// Extracted text or error.
    pub text: Result<String, Error>,
    /// Processing time in milliseconds.
    pub time_ms: u64,
    /// Number of pages in the document (0 if open failed).
    pub page_count: usize,
}

/// Progress callback type for batch processing.
///
/// Called after each file is processed with `(completed, total)` counts.
pub type ProgressCallback = Box<dyn Fn(usize, usize) + Send + Sync>;

/// Batch processor for extracting text from multiple PDF files.
///
/// Supports both sequential and parallel processing (with the `parallel` feature).
/// Collects results from all files without stopping on individual failures.
pub struct BatchProcessor {
    /// Optional progress callback
    progress: Option<ProgressCallback>,
}

impl Default for BatchProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchProcessor {
    /// Create a new batch processor.
    pub fn new() -> Self {
        Self { progress: None }
    }

    /// Set a progress callback that is invoked after each file is processed.
    ///
    /// The callback receives `(completed_count, total_count)`.
    pub fn with_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress = Some(callback);
        self
    }

    /// Extract text from a list of PDF file paths.
    ///
    /// Processes each file and collects results. Does not stop on individual
    /// file failures — errors are captured in the [`BatchResult`].
    ///
    /// When the `parallel` feature is enabled, files are processed in parallel
    /// using rayon. Otherwise, processing is sequential.
    pub fn extract_text_from_files(&self, paths: &[&Path]) -> Vec<BatchResult> {
        let total = paths.len();

        #[cfg(feature = "parallel")]
        {
            use rayon::prelude::*;
            use std::sync::atomic::{AtomicUsize, Ordering};

            let completed = AtomicUsize::new(0);

            let results: Vec<BatchResult> = paths
                .par_iter()
                .map(|path| {
                    let result = Self::process_single_file(path);
                    let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    if let Some(ref cb) = self.progress {
                        cb(done, total);
                    }
                    result
                })
                .collect();

            results
        }

        #[cfg(not(feature = "parallel"))]
        {
            let mut results = Vec::with_capacity(total);
            for (i, path) in paths.iter().enumerate() {
                results.push(Self::process_single_file(path));
                if let Some(ref cb) = self.progress {
                    cb(i + 1, total);
                }
            }
            results
        }
    }

    /// Extract text from all PDF files in a directory.
    ///
    /// Scans the directory for files with `.pdf` extension (case-insensitive)
    /// and processes them all.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read. Individual file
    /// failures are captured in the [`BatchResult`] entries.
    pub fn extract_text_from_directory(&self, dir: &Path) -> Result<Vec<BatchResult>, Error> {
        let mut pdf_paths: Vec<PathBuf> = Vec::new();

        let entries = std::fs::read_dir(dir).map_err(Error::Io)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("pdf") {
                        pdf_paths.push(path);
                    }
                }
            }
        }

        // Sort for deterministic order
        pdf_paths.sort();

        let path_refs: Vec<&Path> = pdf_paths.iter().map(|p| p.as_path()).collect();
        Ok(self.extract_text_from_files(&path_refs))
    }

    /// Process a single PDF file and return a BatchResult.
    fn process_single_file(path: &Path) -> BatchResult {
        let start = Instant::now();

        let result = (|| -> Result<(String, usize), Error> {
            let doc = PdfDocument::open(path)?;
            let page_count = doc.page_count()?;
            let text = doc.extract_all_text()?;
            Ok((text, page_count))
        })();

        let time_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok((text, page_count)) => BatchResult {
                path: path.to_path_buf(),
                text: Ok(text),
                time_ms,
                page_count,
            },
            Err(e) => BatchResult {
                path: path.to_path_buf(),
                text: Err(e),
                time_ms,
                page_count: 0,
            },
        }
    }
}

/// Summary statistics from a batch run.
#[derive(Debug)]
pub struct BatchSummary {
    /// Total files processed.
    pub total: usize,
    /// Files that succeeded.
    pub succeeded: usize,
    /// Files that failed.
    pub failed: usize,
    /// Total characters extracted.
    pub total_chars: usize,
    /// Total pages across all documents.
    pub total_pages: usize,
    /// Total processing time in milliseconds.
    pub total_time_ms: u64,
}

impl BatchSummary {
    /// Compute summary statistics from batch results.
    pub fn from_results(results: &[BatchResult]) -> Self {
        let mut succeeded = 0;
        let mut failed = 0;
        let mut total_chars = 0;
        let mut total_pages = 0;
        let mut total_time_ms = 0;

        for r in results {
            total_time_ms += r.time_ms;
            total_pages += r.page_count;
            match &r.text {
                Ok(text) => {
                    succeeded += 1;
                    total_chars += text.len();
                },
                Err(_) => {
                    failed += 1;
                },
            }
        }

        BatchSummary {
            total: results.len(),
            succeeded,
            failed,
            total_chars,
            total_pages,
            total_time_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_processor_creation() {
        let processor = BatchProcessor::new();
        assert!(processor.progress.is_none());
    }

    #[test]
    fn test_batch_processor_with_progress() {
        let processor = BatchProcessor::new().with_progress(Box::new(|done, total| {
            assert!(done <= total);
        }));
        assert!(processor.progress.is_some());
    }

    #[test]
    fn test_batch_processor_empty_list() {
        let processor = BatchProcessor::new();
        let results = processor.extract_text_from_files(&[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_batch_processor_nonexistent_file() {
        let processor = BatchProcessor::new();
        let results = processor.extract_text_from_files(&[Path::new("/nonexistent.pdf")]);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.is_err());
        assert_eq!(results[0].page_count, 0);
    }

    #[test]
    fn test_batch_summary_empty() {
        let summary = BatchSummary::from_results(&[]);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.succeeded, 0);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_batch_summary_from_results() {
        let results = vec![
            BatchResult {
                path: PathBuf::from("good.pdf"),
                text: Ok("Hello world".to_string()),
                time_ms: 100,
                page_count: 5,
            },
            BatchResult {
                path: PathBuf::from("bad.pdf"),
                text: Err(Error::InvalidPdf("test".to_string())),
                time_ms: 50,
                page_count: 0,
            },
        ];
        let summary = BatchSummary::from_results(&results);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.succeeded, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.total_chars, 11);
        assert_eq!(summary.total_pages, 5);
        assert_eq!(summary.total_time_ms, 150);
    }

    #[test]
    fn test_batch_result_fixture() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("simple.pdf");
        if !fixture.exists() {
            return;
        }

        let processor = BatchProcessor::new();
        let results = processor.extract_text_from_files(&[fixture.as_path()]);
        assert_eq!(results.len(), 1);
        assert!(results[0].text.is_ok());
        assert!(results[0].page_count > 0);
        assert!(results[0].time_ms < 10_000);
    }
}
