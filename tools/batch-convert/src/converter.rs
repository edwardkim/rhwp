use crate::config::ConversionConfig;
use crate::progress::ProgressTracker;
use anyhow::{Context, Result};
use log::*;
use rayon::prelude::*;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

/// 출력 포맷 — rhwp CLI 하위 명령과 1:1 대응한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExportFormat {
    Pdf,
    Png,
    Svg,
    Text,
}

impl ExportFormat {
    const ALL: [ExportFormat; 4] = [Self::Pdf, Self::Png, Self::Svg, Self::Text];

    fn subcommand(self) -> &'static str {
        match self {
            Self::Pdf => "export-pdf",
            Self::Png => "export-png",
            Self::Svg => "export-svg",
            Self::Text => "export-text",
        }
    }

    /// `behavior.create_format_dirs` 용 포맷별 하위 폴더 이름
    fn dir_name(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Text => "text",
        }
    }

    /// rhwp 가 페이지별 파일에 붙이는 확장자 (산출물 존재 판정용)
    fn page_extension(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Png => "png",
            Self::Svg => "svg",
            Self::Text => "txt",
        }
    }
}

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

fn run_rhwp_export(
    rhwp_bin: &Path,
    subcommand: &str,
    input: &Path,
    output: &Path,
    extra_args: &[String],
) -> Result<()> {
    let status = Command::new(rhwp_bin)
        .arg(subcommand)
        .arg(input)
        .arg("-o")
        .arg(output)
        .args(extra_args)
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

    fn format_enabled(&self, format: ExportFormat) -> bool {
        match format {
            ExportFormat::Pdf => self.config.formats.pdf,
            ExportFormat::Png => self.config.formats.png,
            ExportFormat::Svg => self.config.formats.svg,
            ExportFormat::Text => self.config.formats.text,
        }
    }

    fn enabled_formats(&self) -> Vec<ExportFormat> {
        ExportFormat::ALL
            .iter()
            .copied()
            .filter(|f| self.format_enabled(*f))
            .collect()
    }

    /// 포맷별 산출 위치. `behavior.create_format_dirs=false` 면 포맷 하위 폴더
    /// 없이 출력 루트(+입력의 상대 경로)에 바로 놓는다. PDF는 단일 파일이고
    /// 나머지 포맷은 rhwp 가 페이지별 파일을 쓰는 폴더다.
    fn output_target(&self, format: ExportFormat, rel_parent: &Path, file_stem: &str) -> PathBuf {
        let base = if self.config.behavior.create_format_dirs {
            self.output_dir.join(format.dir_name()).join(rel_parent)
        } else {
            self.output_dir.join(rel_parent)
        };
        match format {
            ExportFormat::Pdf => base.join(format!("{}.pdf", file_stem)),
            _ => base.join(file_stem),
        }
    }

    /// 산출물 존재 판정 — PDF 는 파일 존재, 나머지는 "해당 확장자 파일이 1개
    /// 이상 담긴 폴더". create_format_dirs=false 면 png/svg/text 가 같은 폴더를
    /// 공유하므로 폴더 존재가 아니라 확장자로 가른다.
    fn target_exists(format: ExportFormat, target: &Path) -> bool {
        match format {
            ExportFormat::Pdf => target.is_file(),
            _ => {
                let ext = format.page_extension();
                fs::read_dir(target).is_ok_and(|entries| {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        let path = e.path();
                        path.is_file()
                            && path
                                .extension()
                                .and_then(|x| x.to_str())
                                .is_some_and(|x| x.eq_ignore_ascii_case(ext))
                    })
                })
            }
        }
    }

    /// `behavior.skip_existing` 용 — 활성화된 포맷의 산출물이 이미 전부 있으면 true.
    fn all_outputs_exist(&self, rel_parent: &Path, file_stem: &str) -> bool {
        let formats = self.enabled_formats();
        !formats.is_empty()
            && formats
                .iter()
                .all(|f| Self::target_exists(*f, &self.output_target(*f, rel_parent, file_stem)))
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

        // `--jobs` 는 rayon 전역 풀이 아니라 **전용 풀의 worker 수**로 강제한다.
        // (전역 풀 + `with_max_len` 은 작업 분할 단위만 바꿀 뿐 동시에 도는
        // worker 수를 제한하지 않는다 — PR #4052 리뷰 지적)
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.jobs)
            .build()
            .context("Failed to build Rayon thread pool")?;

        let progress = Arc::new(Mutex::new(ProgressTracker::new(total_files)));
        let errors: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
        // `behavior.fail_fast` — 파일 실패가 확정되면 아직 시작하지 않은 파일을
        // 건너뛴다 (이미 진행 중인 파일은 마저 끝난다).
        let abort = AtomicBool::new(false);

        // Convert files in parallel (worker 상한 = 전용 풀 스레드 수)
        let results: Vec<ConversionFileResult> = pool.install(|| {
            self.files
                .par_iter()
                .map(|file| {
                    if abort.load(Ordering::SeqCst) {
                        debug!("Skipping (fail_fast abort): {}", file.path.display());
                        progress.lock().unwrap().increment();
                        return ConversionFileResult::Skipped;
                    }

                    let result = self.convert_file(&file.path, &file.relative_path, dry_run);

                    {
                        let mut prog = progress.lock().unwrap();
                        prog.increment();

                        // Print progress
                        if prog.current.is_multiple_of(10) || prog.current == total_files {
                            println!("{} | {}", prog.status_line(), file.path.display());
                        }
                    }

                    match result {
                        Ok(file_result) => file_result,
                        Err(error_msg) => {
                            if self.config.behavior.fail_fast {
                                abort.store(true, Ordering::SeqCst);
                            }
                            if self.config.behavior.collect_failed && !dry_run {
                                self.collect_failed_input(&file.path, &file.relative_path);
                            }
                            let file_name = file
                                .path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            errors.lock().unwrap().push((file_name, error_msg));
                            ConversionFileResult::Failed
                        }
                    }
                })
                .collect()
        });

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

    /// `behavior.collect_failed` — 실패한 원본을 `<출력>/failed/<상대경로>` 로
    /// 복사해 재시도 대상만 따로 모은다. 복사 실패는 변환 실패 판정에 얹지
    /// 않고 경고만 남긴다.
    fn collect_failed_input(&self, input: &Path, relative_path: &Path) {
        let dest = self.output_dir.join("failed").join(relative_path);
        let copied = (|| -> std::io::Result<()> {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(input, &dest)?;
            Ok(())
        })();
        match copied {
            Ok(()) => info!("Copied failed input to {}", dest.display()),
            Err(e) => warn!(
                "Failed to copy failed input {} to {}: {}",
                input.display(),
                dest.display(),
                e
            ),
        }
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

        let mut attempted = 0usize;
        let mut succeeded = 0usize;
        let mut format_errors: Vec<String> = Vec::new();

        for format in self.enabled_formats() {
            let target = self.output_target(format, rel_parent, file_stem);

            // `behavior.overwrite=false` — 이미 있는 산출물은 포맷 단위로
            // 건너뛰고 다시 만들지 않는다.
            if !self.config.behavior.overwrite && Self::target_exists(format, &target) {
                debug!(
                    "Skipping {} (output exists, overwrite=false): {}",
                    format.subcommand(),
                    target.display()
                );
                continue;
            }

            attempted += 1;
            match self.export_with_retry(format, input_path, &target, dry_run) {
                Ok(()) => {
                    succeeded += 1;
                    debug!(
                        "Successfully converted to {}: {}",
                        format.dir_name(),
                        target.display()
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to convert {} to {}: {:#}",
                        input_path.display(),
                        format.dir_name(),
                        e
                    );
                    format_errors.push(format!("{}: {:#}", format.subcommand(), e));
                }
            }
        }

        if attempted == 0 {
            // 활성 포맷 전부가 overwrite=false 로 건너뛰어졌다.
            return Ok(ConversionFileResult::Skipped);
        }
        if succeeded > 0 {
            Ok(ConversionFileResult::Success)
        } else {
            Err(format_errors.join("; "))
        }
    }

    /// `behavior.max_retries` — rhwp 호출이 실패하면 같은 포맷을 최대 N번 더
    /// 시도한다 (총 시도 = 1 + N).
    fn export_with_retry(
        &self,
        format: ExportFormat,
        input: &Path,
        target: &Path,
        dry_run: bool,
    ) -> Result<()> {
        let max_retries = self.config.behavior.max_retries;
        let mut attempt = 0u32;
        loop {
            match self.export_once(format, input, target, dry_run) {
                Ok(()) => return Ok(()),
                Err(e) if attempt < max_retries => {
                    attempt += 1;
                    warn!(
                        "Retrying {} for {} (attempt {}/{}): {:#}",
                        format.subcommand(),
                        input.display(),
                        attempt,
                        max_retries,
                        e
                    );
                }
                Err(e) => return Err(e),
            }
        }
    }

    fn export_once(
        &self,
        format: ExportFormat,
        input: &Path,
        target: &Path,
        dry_run: bool,
    ) -> Result<()> {
        if dry_run {
            debug!(
                "[DRY RUN] Would run rhwp {} for {} -> {}",
                format.subcommand(),
                input.display(),
                target.display()
            );
            return Ok(());
        }

        // PDF 는 단일 파일이라 부모 폴더를, 나머지는 페이지 파일이 담길 폴더
        // 자체를 만든다.
        let dir_to_create = match format {
            ExportFormat::Pdf => target.parent().unwrap_or(Path::new("")).to_path_buf(),
            _ => target.to_path_buf(),
        };
        if !dir_to_create.as_os_str().is_empty() {
            fs::create_dir_all(&dir_to_create).with_context(|| {
                format!(
                    "Failed to create output directory: {}",
                    dir_to_create.display()
                )
            })?;
        }

        run_rhwp_export(
            &self.rhwp_bin,
            format.subcommand(),
            input,
            target,
            &self.format_args(format),
        )
    }

    /// config 의 포맷 옵션을 rhwp CLI 플래그로 그대로 옮긴다. 여기서 플래그로
    /// 나가지 않는 옵션은 config 계약에도 존재하지 않는다 (config.rs 참조).
    fn format_args(&self, format: ExportFormat) -> Vec<String> {
        let mut args = Vec::new();
        match format {
            ExportFormat::Pdf => {
                let pdf = &self.config.pdf;
                if let Some(backend) = &pdf.backend {
                    args.push("--backend".to_string());
                    args.push(backend.clone());
                }
                if let Some(profile) = &pdf.profile {
                    args.push("--profile".to_string());
                    args.push(profile.clone());
                }
                if let Some(raster_dpi) = pdf.raster_dpi {
                    args.push("--raster-dpi".to_string());
                    args.push(raster_dpi.to_string());
                }
                if pdf.text_as_paths {
                    args.push("--text-as-paths".to_string());
                }
            }
            ExportFormat::Png => {
                let png = &self.config.png;
                if let Some(profile) = &png.profile {
                    args.push("--profile".to_string());
                    args.push(profile.clone());
                }
                if let Some(dpi) = png.dpi {
                    args.push("--dpi".to_string());
                    args.push(dpi.to_string());
                }
                if let Some(scale) = png.scale {
                    args.push("--scale".to_string());
                    args.push(scale.to_string());
                }
                if let Some(max_dimension) = png.max_dimension {
                    args.push("--max-dimension".to_string());
                    args.push(max_dimension.to_string());
                }
            }
            ExportFormat::Svg => {
                let svg = &self.config.svg;
                if let Some(profile) = &svg.profile {
                    args.push("--profile".to_string());
                    args.push(profile.clone());
                }
                if svg.embed_fonts {
                    args.push("--embed-fonts".to_string());
                }
            }
            ExportFormat::Text => {
                // rhwp export-text 에는 배치 변환에서 쓸 수 있는 추가 플래그가
                // 없다 (--json/--max-chars 는 파일 저장 모드와 호환되지 않음).
            }
        }
        args
    }
}

#[derive(Debug)]
enum ConversionFileResult {
    Success,
    Failed,
    Skipped,
}
