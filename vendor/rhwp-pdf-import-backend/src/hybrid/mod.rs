//! Hybrid classical + ML architecture.
//!
//! This module orchestrates between classical and ML-based approaches
//! based on document complexity estimation.
//!
//! # Strategy
//!
//! - Simple documents: Fast classical algorithms
//! - Moderate documents: Either approach works (prefer classical for speed)
//! - Complex documents: ML for better accuracy
//!
//! # Example
//!
//! ```ignore
//! use pdf_oxide::hybrid::SmartLayoutAnalyzer;
//!
//! let analyzer = SmartLayoutAnalyzer::new();
//! let order = analyzer.determine_reading_order(&blocks, width, height)?;
//! ```

pub mod complexity_estimator;
pub mod smart_analyzer;

pub use complexity_estimator::{Complexity, ComplexityEstimator};
pub use smart_analyzer::{AnalyzerCapabilities, SmartLayoutAnalyzer};
