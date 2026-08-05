use crate::config::ConversionConfig;
use crate::progress::ProgressTracker;
use anyhow::{Context, Result};
use log::*;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

/// `rhwp` 바이너리를 찾는다: 명시 경로 > PATH > `target/{release,debug}/rhwp`.
///
/// 별도 crate로 빌드되는 batch-convert는 rhwp 라이브러리의 렌더링/내보내기
/// 내부 구현에 직접 링크하지 않고 이미 검증된 CLI 계약(export-pdf/png/svg/text)에
/// 위임한다 — 내부 API 변경에 따로 맞춰 낡는 것을 막기 위해서다.
pub fn find_rhwp_binary(explicit: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(path) = explicit {
        return Some(path);
    }

    if let Ok(path) = which::which("rhwp") {
        return Some(path);
    }

    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    // batch-convert 실행 파일은 보통 target/{profile}/batch-convert 에 있으므로
    // 같은 디렉터리에 sibling으로 빌드된 rhwp를 먼저 찾는다.
    for name in ["rhwp.exe", "rhwp"] {
        let candidate = exe_dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let repo_root = exe_dir
        .ancestors()
        .find(|p| p.join("Cargo.toml").is_file())?;
    for profile in ["release", "debug"] {
        for name in ["rhwp.exe", "rhwp"] {
            let candidate = repo_root.join("target").join(profile).join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

fn run_rhwp_export(rhwp_bin: &Path, subcommand: &str, input: &Path, output: &Path) -> Result<()> {
    let status = Command::new(rhwp_bin)
        .arg(subcommand)
        .arg(input)
        .arg("-o")
        .arg(output)
        .status()
        .with_context(|| format!("failed to spawn rhwp {}", subcommand))?;

    if !status.success() {
        anyhow::bail!(
            "rhwp {} exited with {} for {}",
            subcommand,
            status,
            input.display()
        );
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub skipped: usize,
    pub elapsed_seconds: f64,
    pub errors: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
struct FileEntry {
    path: PathBuf,
    relative_path: PathBuf,
}

pub struct BatchConverter {
    input_dir: PathBuf,
    output_dir: PathBuf,
    config: ConversionConfig,
    jobs: usize,
    rhwp_bin: PathBuf,
    pattern_filter: Option<Regex>,
    files: Vec<FileEntry>,
}

impl BatchConverter {
    pub fn new(
        input_dir: PathBuf,
        output_dir: PathBuf,
        config: ConversionConfig,
        jobs: usize,
        rhwp_bin: PathBuf,
    ) -> Result<Self> {
        // Validate input directory
        if !input_dir.exists() {
            anyhow::bail!("Input directory does not exist: {}", input_dir.display());
        }

        // Create output directory if it doesn't exist
        if !output_dir.exists() {
            fs::create_dir_all(&output_dir).context(format!(
                "Failed to create output directory: {}",
                output_dir.display()
            ))?;
        }

        let mut converter = BatchConverter {
            input_dir,
            output_dir,
            config,
            jobs,
            rhwp_bin,
            pattern_filter: None,
            files: Vec::new(),
        };

        // Discover files
        converter.discover_files()?;

        Ok(converter)
    }

    pub fn set_pattern_filter(&mut self, pattern: &str) -> Result<()> {
        self.pattern_filter = Some(Regex::new(pattern)?);
        self.discover_files()?;
        Ok(())
    }

    /// `behavior.skip_existing` 용 — 활성화된 포맷의 출력이 이미 전부 있으면 true.
    /// PDF는 단일 파일, PNG/SVG/텍스트는 페이지별 파일이 담기는 디렉터리라
    /// 디렉터리 존재만 확인한다(빈 디렉터리는 미완료로 간주해 재변환한다).
    fn all_outputs_exist(&self, rel_parent: &Path, file_stem: &str) -> bool {
        let formats = &self.config.formats;

        if formats.pdf {
            let path = self
                .output_dir
                .join("pdf")
                .join(rel_parent)
                .join(format!("{}.pdf", file_stem));
            if !path.is_file() {
                return false;
            }
        }
        for (enabled, name) in [
            (formats.png, "png"),
            (formats.svg, "svg"),
            (formats.text, "text"),
        ] {
            if !enabled {
                continue;
            }
            let dir = self.output_dir.join(name).join(rel_parent).join(file_stem);
            let has_output = dir.is_dir()
                && fs::read_dir(&dir).is_ok_and(|mut entries| entries.next().is_some());
            if !has_output {
                return false;
            }
        }

        formats.pdf || formats.png || formats.svg || formats.text
    }

    fn discover_files(&mut self) -> Result<()> {
        self.files.clear();

        for entry in WalkDir::new(&self.input_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Check if it's a file
            if !path.is_file() {
                continue;
            }

            // Check file extension
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());

            if !matches!(extension.as_deref(), Some("hwp") | Some("hwpx")) {
                continue;
            }

            // Apply pattern filter if set
            if let Some(ref filter) = self.pattern_filter {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if !filter.is_match(file_name) {
                        continue;
                    }
                }
            }

            let relative_path = path
                .strip_prefix(&self.input_dir)
                .unwrap_or(Path::new(""))
                .to_path_buf();

            self.files.push(FileEntry {
                path: path.to_path_buf(),
                relative_path,
            });
        }

        info!("Discovered {} HWP/HWPX files", self.files.len());
        Ok(())
    }

    pub fn convert_batch(&self, dry_run: bool) -> Result<ConversionResult> {
        let start_time = std::time::Instant::now();
        let total_files = self.files.len();

        info!("Starting parallel conversion with {} workers", self.jobs);

        let progress = Arc::new(Mutex::new(ProgressTracker::new(total_files)));
        let errors: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));

        // Convert files in parallel
        let results: Vec<ConversionFileResult> = self
            .files
            .par_iter()
            .with_max_len(self.jobs)
            .map(|file| {
                let result = self.convert_file(&file.path, &file.relative_path, dry_run);

                let mut prog = progress.lock().unwrap();
                prog.increment();

                // Print progress
                if prog.current.is_multiple_of(10) || prog.current == total_files {
                    println!("{} | {}", prog.status_line(), file.path.display());
                }

                // Handle errors
                if let Err(ref e) = result {
                    let file_name = file
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let error_msg = format!("{:?}", e);
                    errors.lock().unwrap().push((file_name, error_msg));
                }

                result.unwrap_or(ConversionFileResult::Failed)
            })
            .collect();

        // Count results
        let successful = results
            .iter()
            .filter(|r| matches!(r, ConversionFileResult::Success))
            .count();
        let failed = results
            .iter()
            .filter(|r| matches!(r, ConversionFileResult::Failed))
            .count();
        let skipped = results
            .iter()
            .filter(|r| matches!(r, ConversionFileResult::Skipped))
            .count();

        let elapsed = start_time.elapsed();
        let elapsed_seconds = elapsed.as_secs_f64();

        let error_list = errors.lock().unwrap().clone();

        Ok(ConversionResult {
            total: total_files,
            successful,
            failed,
            skipped,
            elapsed_seconds,
            errors: error_list,
        })
    }

    fn convert_file(
        &self,
        input_path: &Path,
        relative_path: &Path,
        dry_run: bool,
    ) -> Result<ConversionFileResult, String> {
        // Get file stem (name without extension)
        let file_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "Invalid file name".to_string())?;

        // 입력 디렉터리 하위 구조를 출력에도 그대로 반영한다 (예: input/2026/a.hwp
        // → output/pdf/2026/a.pdf).
        let rel_parent = relative_path.parent().unwrap_or(Path::new(""));

        if self.config.behavior.skip_existing && self.all_outputs_exist(rel_parent, file_stem) {
            debug!("Skipping (outputs already exist): {}", input_path.display());
            return Ok(ConversionFileResult::Skipped);
        }

        // Convert to each enabled format
        let mut any_success = false;

        // PDF conversion
        if self.config.formats.pdf {
            let output_subdir = self.output_dir.join("pdf").join(rel_parent);
            if !dry_run && !output_subdir.exists() {
                fs::create_dir_all(&output_subdir).ok();
            }

            let output_path = output_subdir.join(format!("{}.pdf", file_stem));
            match self.convert_to_pdf(input_path, &output_path, dry_run) {
                Ok(()) => {
                    any_success = true;
                    debug!("Successfully converted to PDF: {}", output_path.display());
                }
                Err(e) => {
                    warn!("Failed to convert to PDF: {}", e);
                }
            }
        }

        // PNG conversion — rhwp export-png writes one file per page into a
        // directory (unlike export-pdf, which writes a single merged file).
        if self.config.formats.png {
            let output_subdir = self.output_dir.join("png").join(rel_parent).join(file_stem);
            if !dry_run && !output_subdir.exists() {
                fs::create_dir_all(&output_subdir).ok();
            }

            match self.convert_to_png(input_path, &output_subdir, dry_run) {
                Ok(()) => {
                    any_success = true;
                    debug!("Successfully converted to PNG: {}", output_subdir.display());
                }
                Err(e) => {
                    warn!("Failed to convert to PNG: {}", e);
                }
            }
        }

        // SVG conversion — same per-page-directory convention as PNG/text.
        if self.config.formats.svg {
            let output_subdir = self.output_dir.join("svg").join(rel_parent).join(file_stem);
            if !dry_run && !output_subdir.exists() {
                fs::create_dir_all(&output_subdir).ok();
            }

            match self.convert_to_svg(input_path, &output_subdir, dry_run) {
                Ok(()) => {
                    any_success = true;
                    debug!("Successfully converted to SVG: {}", output_subdir.display());
                }
                Err(e) => {
                    warn!("Failed to convert to SVG: {}", e);
                }
            }
        }

        // Text conversion — same per-page-directory convention as PNG/SVG.
        if self.config.formats.text {
            let output_subdir = self
                .output_dir
                .join("text")
                .join(rel_parent)
                .join(file_stem);
            if !dry_run && !output_subdir.exists() {
                fs::create_dir_all(&output_subdir).ok();
            }

            match self.convert_to_text(input_path, &output_subdir, dry_run) {
                Ok(()) => {
                    any_success = true;
                    debug!(
                        "Successfully converted to text: {}",
                        output_subdir.display()
                    );
                }
                Err(e) => {
                    warn!("Failed to convert to text: {}", e);
                }
            }
        }

        if any_success {
            Ok(ConversionFileResult::Success)
        } else {
            Ok(ConversionFileResult::Failed)
        }
    }

    fn convert_to_pdf(&self, input: &Path, output: &Path, dry_run: bool) -> Result<()> {
        if dry_run {
            debug!(
                "[DRY RUN] Would convert {} to PDF at {}",
                input.display(),
                output.display()
            );
            return Ok(());
        }
        debug!("Converting {} to PDF...", input.display());
        run_rhwp_export(&self.rhwp_bin, "export-pdf", input, output)
    }

    fn convert_to_png(&self, input: &Path, output: &Path, dry_run: bool) -> Result<()> {
        if dry_run {
            debug!(
                "[DRY RUN] Would convert {} to PNG at {}",
                input.display(),
                output.display()
            );
            return Ok(());
        }
        debug!("Converting {} to PNG...", input.display());
        run_rhwp_export(&self.rhwp_bin, "export-png", input, output)
    }

    fn convert_to_svg(&self, input: &Path, output: &Path, dry_run: bool) -> Result<()> {
        if dry_run {
            debug!(
                "[DRY RUN] Would convert {} to SVG at {}",
                input.display(),
                output.display()
            );
            return Ok(());
        }
        debug!("Converting {} to SVG...", input.display());
        run_rhwp_export(&self.rhwp_bin, "export-svg", input, output)
    }

    fn convert_to_text(&self, input: &Path, output: &Path, dry_run: bool) -> Result<()> {
        if dry_run {
            debug!(
                "[DRY RUN] Would convert {} to text at {}",
                input.display(),
                output.display()
            );
            return Ok(());
        }
        debug!("Converting {} to text...", input.display());
        run_rhwp_export(&self.rhwp_bin, "export-text", input, output)
    }
}

#[derive(Debug)]
enum ConversionFileResult {
    Success,
    Failed,
    Skipped,
}
