mod config;
mod converter;
mod progress;

use anyhow::Result;
use clap::Parser;
use log::*;
use std::path::PathBuf;

use config::ConversionConfig;
use converter::BatchConverter;

#[derive(Parser, Debug)]
#[command(name = "batch-convert")]
#[command(about = "Batch converter for HWP/HWPX files with parallel processing")]
#[command(version = "0.1.0")]
#[command(author = "HWP Converter Team")]
struct Args {
    /// Input directory containing HWP/HWPX files
    #[arg(short, long)]
    input_dir: PathBuf,

    /// Output directory for converted files
    #[arg(short, long)]
    output_dir: PathBuf,

    /// Configuration file (JSON) for conversion options
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Number of parallel workers
    #[arg(short = 'j', long, default_value = "4")]
    jobs: usize,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Dry run mode (no files written)
    #[arg(long)]
    dry_run: bool,

    /// File pattern filter (regex)
    #[arg(short = 'p', long)]
    pattern: Option<String>,

    /// Path to the rhwp CLI binary (default: PATH, then target/{release,debug}/rhwp)
    #[arg(long)]
    rhwp_bin: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    env_logger::Builder::from_default_env()
        .filter_level(log_level.parse()?)
        .try_init()?;

    info!("Starting batch converter");
    info!("Input directory: {}", args.input_dir.display());
    info!("Output directory: {}", args.output_dir.display());
    info!("Number of parallel workers: {}", args.jobs);

    // Load configuration
    let config = if let Some(config_path) = args.config {
        info!("Loading configuration from: {}", config_path.display());
        ConversionConfig::from_file(&config_path)?
    } else {
        info!("Using default configuration");
        ConversionConfig::default()
    };

    // Locate rhwp CLI binary
    let rhwp_bin = converter::find_rhwp_binary(args.rhwp_bin.clone()).ok_or_else(|| {
        anyhow::anyhow!(
            "rhwp binary not found — pass --rhwp-bin or build it under target/{{release,debug}}/"
        )
    })?;
    info!("Using rhwp binary: {}", rhwp_bin.display());

    // Create batch converter
    let mut converter = BatchConverter::new(
        args.input_dir.clone(),
        args.output_dir.clone(),
        config,
        args.jobs,
        rhwp_bin,
    )?;

    // Apply pattern filter if provided
    if let Some(pattern) = args.pattern {
        info!("Applying file pattern filter: {}", pattern);
        converter.set_pattern_filter(&pattern)?;
    }

    // Run batch conversion
    if args.dry_run {
        info!("Running in DRY RUN mode (no files will be written)");
    }

    let results = converter.convert_batch(args.dry_run)?;

    // Print summary
    println!("\n================== CONVERSION SUMMARY ==================");
    println!("Total files processed: {}", results.total);
    println!("Successful conversions: {}", results.successful);
    println!("Failed conversions: {}", results.failed);
    println!("Skipped files: {}", results.skipped);
    println!("Total time: {:.2}s", results.elapsed_seconds);

    if results.failed > 0 {
        println!("\nFailed files:");
        for (file, error) in &results.errors {
            println!("  - {}: {}", file, error);
        }
        std::process::exit(1);
    }

    println!("========================================================");
    println!("Batch conversion completed successfully!");

    Ok(())
}
