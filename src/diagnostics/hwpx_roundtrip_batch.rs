//! HWPX roundtrip 배치 검증 (Task #1315).
//!
//! `samples/hwpx/` 등의 HWPX 파일을 `parse → serialize → 재parse` 경로로 돌려
//! 파일별 성공 여부와 IR diff 건수를 측정하고, 재조립 `.hwpx`를 출력 폴더에 남긴다.
//!
//! ```text
//! rhwp hwpx-roundtrip sample.hwpx -o output/poc/task1315/
//! rhwp hwpx-roundtrip --batch samples/hwpx -o output/poc/task1315/
//! ```
//!
//! 출력:
//! - `{out}/{상대경로 stem}.rt.hwpx` — 재조립 HWPX
//! - `{out}/inventory.tsv` — 배치 측정 결과 (배치 모드)

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::parser::hwpx::parse_hwpx;
use crate::serializer::hwpx::package_check::check_package;
use crate::serializer::hwpx::roundtrip::{
    diff_documents, diff_linesegs, LinesegDiff, LinesegDiffKind,
};
use crate::serializer::hwpx::serialize_hwpx;

#[derive(Debug)]
struct Options {
    input: PathBuf,
    batch: bool,
    out_dir: PathBuf,
    /// #1380 측정 전용 — 문단별 lineseg diff 를 `lineseg_diff.tsv` 로 산출.
    lineseg_report: bool,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut input: Option<PathBuf> = None;
    let mut batch = false;
    let mut out_dir = PathBuf::from("output/poc/task1315");
    let mut lineseg_report = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--batch" => batch = true,
            "--lineseg-report" => lineseg_report = true,
            "-o" | "--out" => {
                i += 1;
                let v = args
                    .get(i)
                    .ok_or_else(|| "-o 다음에 출력 폴더가 필요합니다".to_string())?;
                out_dir = PathBuf::from(v);
            }
            other if other.starts_with('-') => {
                return Err(format!("알 수 없는 옵션: {other}"));
            }
            other => {
                if input.is_some() {
                    return Err(format!("입력 경로가 중복 지정됨: {other}"));
                }
                input = Some(PathBuf::from(other));
            }
        }
        i += 1;
    }

    let input = input.ok_or_else(|| {
        "사용법: rhwp hwpx-roundtrip <입력.hwpx | --batch 폴더> [-o 출력폴더]".to_string()
    })?;
    Ok(Options {
        input,
        batch,
        out_dir,
        lineseg_report,
    })
}

/// 파일 1건의 roundtrip 측정 결과.
#[derive(Debug)]
struct RoundtripRow {
    /// 배치 루트 기준 상대 경로 (단일 모드는 파일명).
    rel_path: String,
    parse_ok: bool,
    serialize_ok: bool,
    reparse_ok: bool,
    ir_diff_count: Option<usize>,
    ir_diff_summary: String,
    /// 패키지 구조 검사 (2단계). `None` = 미실행 (선행 단계 실패).
    pkg_ok: Option<bool>,
    pkg_problems: String,
    /// 2-round 안정성: round1 IR vs round2 IR 의 diff 건수. `None` = 미실행/실패.
    round2_diff_count: Option<usize>,
    round2_error: String,
    elapsed_ms: u128,
    error: String,
    /// [#1914] 매직 바이트 실체가 HWPX(ZIP)가 아닌 확장자 위장 파일 — 검출된
    /// 실체 포맷명. HWPX 구조 게이트의 범위 밖이므로 실패가 아닌 스킵으로 분류.
    format_skip: Option<&'static str>,
    /// [#1946] ODF 암호화(비밀번호 보호) HWPX — 복호화 미지원. 게이트 범위 밖
    /// 이므로 실패가 아닌 스킵으로 분류(FORMAT_SKIP 과 동형).
    encrypted_skip: bool,
    /// #1380 측정 전용 — round1(원본 vs RT) lineseg diff. `--lineseg-report` 시에만 수집.
    lineseg_r1: Vec<LinesegDiff>,
    /// #1380 측정 전용 — round2(RT vs RT²) lineseg diff.
    lineseg_r2: Vec<LinesegDiff>,
}

impl RoundtripRow {
    fn status(&self) -> &'static str {
        if self.format_skip.is_some() {
            "FORMAT_SKIP"
        } else if self.encrypted_skip {
            "ENCRYPTED_SKIP"
        } else if !self.parse_ok {
            "PARSE_FAIL"
        } else if !self.serialize_ok {
            "SERIALIZE_FAIL"
        } else if !self.reparse_ok {
            "REPARSE_FAIL"
        } else if self.ir_diff_count.is_some_and(|c| c > 0) {
            "IR_DIFF"
        } else if self.pkg_ok == Some(false) {
            "PKG_FAIL"
        } else if !self.round2_error.is_empty() || self.round2_diff_count.is_none() {
            "ROUND2_FAIL"
        } else if self.round2_diff_count.is_some_and(|c| c > 0) {
            "ROUND2_DIFF"
        } else {
            "PASS"
        }
    }

    /// 회귀 검출용 하드 실패 (등급화 대상 분류와 별개).
    fn is_hard_fail(&self) -> bool {
        if self.format_skip.is_some() || self.encrypted_skip {
            // [#1914/#1946] 확장자 위장(실체 비-HWPX)·암호화 문서는 게이트 범위 밖 — 실패 아님.
            return false;
        }
        !(self.parse_ok && self.serialize_ok && self.reparse_ok)
            || self.pkg_ok == Some(false)
            || !self.round2_error.is_empty()
    }
}

/// [#1914] 매직 바이트 실체 포맷명 (FORMAT_SKIP 사유 표기용).
fn detected_format_name(fmt: crate::parser::FileFormat) -> Option<&'static str> {
    use crate::parser::FileFormat;
    match fmt {
        FileFormat::Hwp => Some("HWP5(OLE/CFB)"),
        FileFormat::Hwp3 => Some("HWP3"),
        FileFormat::Hml => Some("HWPML(구 XML)"),
        FileFormat::DrmProtected => Some("DRM 보호"),
        FileFormat::Empty => Some("빈 파일"),
        FileFormat::Hwpx | FileFormat::Unknown => None,
    }
}

/// 단일 HWPX 파일 roundtrip 실행. 재조립 파일을 `rt_path`에 기록.
fn roundtrip_one(
    path: &Path,
    rel_path: &str,
    rt_path: &Path,
    lineseg_report: bool,
) -> RoundtripRow {
    let started = Instant::now();
    let mut row = RoundtripRow {
        rel_path: rel_path.to_string(),
        parse_ok: false,
        serialize_ok: false,
        reparse_ok: false,
        ir_diff_count: None,
        ir_diff_summary: String::new(),
        pkg_ok: None,
        pkg_problems: String::new(),
        round2_diff_count: None,
        round2_error: String::new(),
        elapsed_ms: 0,
        error: String::new(),
        format_skip: None,
        encrypted_skip: false,
        lineseg_r1: Vec::new(),
        lineseg_r2: Vec::new(),
    };

    let finish = |mut row: RoundtripRow, started: Instant| -> RoundtripRow {
        row.elapsed_ms = started.elapsed().as_millis();
        row
    };

    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            row.error = format!("읽기 실패: {e}");
            return finish(row, started);
        }
    };

    // [#1914] 확장자 아닌 매직 바이트로 실체 판별 — .hwpx 로 유통되는 HWP5(OLE)
    // 실체 파일(정부 보도자료 계열 다수)은 HWPX 구조 게이트의 범위 밖이다.
    // 종전 PARSE_FAIL("ZIP EOCD") 오분류를 FORMAT_SKIP 으로 정정하고 실체와
    // 올바른 게이트를 안내한다. Unknown(빈 파일/DRM 래퍼 등)은 종전대로
    // 파싱 실패가 정당하므로 그대로 진행한다.
    if let Some(actual) = detected_format_name(crate::parser::detect_format(&bytes)) {
        row.format_skip = Some(actual);
        row.error = format!(
            "확장자 위장: 실체 {actual} — HWPX 게이트 범위 밖 (HWP5 는 hwp5-roundtrip 사용)"
        );
        return finish(row, started);
    }

    let doc1 = match parse_hwpx(&bytes) {
        Ok(d) => d,
        Err(e) => {
            // [#1946] 암호화 HWPX 는 게이트 범위 밖 — PARSE_FAIL 대신 ENCRYPTED_SKIP.
            if e.is_encrypted() {
                row.encrypted_skip = true;
            }
            row.error = format!("파싱 실패: {e}");
            return finish(row, started);
        }
    };
    row.parse_ok = true;

    let out = match serialize_hwpx(&doc1) {
        Ok(o) => o,
        Err(e) => {
            row.error = format!("직렬화 실패: {e}");
            return finish(row, started);
        }
    };
    row.serialize_ok = true;

    if let Some(parent) = rt_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            row.error = format!("출력 폴더 생성 실패: {e}");
            return finish(row, started);
        }
    }
    if let Err(e) = fs::write(rt_path, &out) {
        row.error = format!("재조립 파일 쓰기 실패: {e}");
        return finish(row, started);
    }

    let doc2 = match parse_hwpx(&out) {
        Ok(d) => d,
        Err(e) => {
            row.error = format!("재파싱 실패: {e}");
            return finish(row, started);
        }
    };
    row.reparse_ok = true;

    let diff = diff_documents(&doc1, &doc2);
    row.ir_diff_count = Some(diff.differences.len());
    row.ir_diff_summary = diff
        .differences
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("; ");

    // 패키지 구조 검사 (2단계) — 재조립 ZIP 을 원본 IR 기준으로 검사
    let pkg = check_package(&out, &doc1);
    row.pkg_ok = Some(pkg.is_ok());
    row.pkg_problems = pkg.summary();

    // #1380 측정 전용 — round1 lineseg diff (원본 IR vs RT IR)
    if lineseg_report {
        row.lineseg_r1 = diff_linesegs(&doc1, &doc2);
    }

    // 2-round 안정성 (LINE_SEG reflow 류 드리프트 검출):
    // round1 IR(doc2) 을 다시 직렬화→파싱한 IR(doc3) 과 비교해 0 이어야 안정.
    match serialize_hwpx(&doc2) {
        Ok(out2) => match parse_hwpx(&out2) {
            Ok(doc3) => {
                let diff2 = diff_documents(&doc2, &doc3);
                row.round2_diff_count = Some(diff2.differences.len());
                if lineseg_report {
                    row.lineseg_r2 = diff_linesegs(&doc2, &doc3);
                }
            }
            Err(e) => row.round2_error = format!("2-round 재파싱 실패: {e}"),
        },
        Err(e) => row.round2_error = format!("2-round 직렬화 실패: {e}"),
    }

    finish(row, started)
}

/// 폴더에서 `.hwpx` 파일을 재귀 수집 (정렬된 순서).
fn collect_hwpx_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            fs::read_dir(&dir).map_err(|e| format!("폴더 읽기 실패 {}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("폴더 항목 읽기 실패: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("hwpx"))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn tsv_escape(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

fn write_tsv(out_dir: &Path, rows: &[RoundtripRow]) -> Result<PathBuf, String> {
    let tsv_path = out_dir.join("inventory.tsv");
    let opt_to_str = |o: Option<usize>| o.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
    let mut tsv = String::from(
        "sample\tstatus\tparse_ok\tserialize_ok\treparse_ok\tir_diff_count\tpkg_ok\tround2_diff\telapsed_ms\terror\tir_diff_summary\tpkg_problems\tround2_error\n",
    );
    for row in rows {
        let pkg = match row.pkg_ok {
            Some(true) => "true",
            Some(false) => "false",
            None => "-",
        };
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            tsv_escape(&row.rel_path),
            row.status(),
            row.parse_ok,
            row.serialize_ok,
            row.reparse_ok,
            opt_to_str(row.ir_diff_count),
            pkg,
            opt_to_str(row.round2_diff_count),
            row.elapsed_ms,
            tsv_escape(&row.error),
            tsv_escape(&row.ir_diff_summary),
            tsv_escape(&row.pkg_problems),
            tsv_escape(&row.round2_error),
        ));
    }
    fs::write(&tsv_path, tsv).map_err(|e| format!("TSV 쓰기 실패: {e}"))?;
    Ok(tsv_path)
}

/// #1380 측정 전용 — 문단별 lineseg diff 를 `lineseg_diff.tsv` 로 기록.
///
/// 컬럼: sample / round(1=원본vsRT, 2=RTvsRT²) / section / paragraph / path /
/// kind(count|value) / index / field / expected / actual.
/// 기존 `inventory.tsv` 13컬럼은 변경하지 않는다.
fn write_lineseg_tsv(out_dir: &Path, rows: &[RoundtripRow]) -> Result<PathBuf, String> {
    let tsv_path = out_dir.join("lineseg_diff.tsv");
    let mut tsv = String::from(
        "sample\tround\tsection\tparagraph\tpath\tkind\tindex\tfield\texpected\tactual\n",
    );
    for row in rows {
        for (round, diffs) in [(1, &row.lineseg_r1), (2, &row.lineseg_r2)] {
            for d in diffs {
                let (kind, index, field, expected, actual) = match &d.kind {
                    LinesegDiffKind::CountMismatch { expected, actual } => (
                        "count",
                        "-".to_string(),
                        "-",
                        expected.to_string(),
                        actual.to_string(),
                    ),
                    LinesegDiffKind::ValueMismatch {
                        index,
                        field,
                        expected,
                        actual,
                    } => (
                        "value",
                        index.to_string(),
                        *field,
                        expected.to_string(),
                        actual.to_string(),
                    ),
                };
                tsv.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    tsv_escape(&row.rel_path),
                    round,
                    d.section,
                    d.paragraph,
                    tsv_escape(&d.path),
                    kind,
                    index,
                    field,
                    expected,
                    actual,
                ));
            }
        }
    }
    fs::create_dir_all(out_dir).map_err(|e| format!("출력 폴더 생성 실패: {e}"))?;
    fs::write(&tsv_path, tsv).map_err(|e| format!("lineseg TSV 쓰기 실패: {e}"))?;
    Ok(tsv_path)
}

fn print_summary(rows: &[RoundtripRow]) {
    let count = |s: &str| rows.iter().filter(|r| r.status() == s).count();
    println!();
    println!("=== hwpx-roundtrip 요약 ===");
    println!("  총 파일        : {}", rows.len());
    println!("  PASS           : {}", count("PASS"));
    println!("  IR_DIFF        : {}", count("IR_DIFF"));
    println!("  PKG_FAIL       : {}", count("PKG_FAIL"));
    println!("  ROUND2_DIFF    : {}", count("ROUND2_DIFF"));
    println!("  ROUND2_FAIL    : {}", count("ROUND2_FAIL"));
    println!("  ENCRYPTED_SKIP : {}", count("ENCRYPTED_SKIP"));
    println!("  PARSE_FAIL     : {}", count("PARSE_FAIL"));
    println!("  SERIALIZE_FAIL : {}", count("SERIALIZE_FAIL"));
    println!("  REPARSE_FAIL   : {}", count("REPARSE_FAIL"));
}

/// `rt.hwpx` 출력 경로 — 배치 루트 기준 상대 구조를 출력 폴더 아래에 유지.
fn rt_output_path(out_dir: &Path, rel_path: &str) -> PathBuf {
    let rel = Path::new(rel_path);
    let stem = rel
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let mut out = out_dir.to_path_buf();
    if let Some(parent) = rel.parent() {
        out.push(parent);
    }
    out.push(format!("{stem}.rt.hwpx"));
    out
}

pub fn run(args: &[String]) {
    let opts = match parse_args(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("오류: {e}");
            std::process::exit(2);
        }
    };

    let inputs: Vec<(PathBuf, String)> = if opts.batch {
        match collect_hwpx_files(&opts.input) {
            Ok(files) => files
                .into_iter()
                .map(|p| {
                    let rel = p
                        .strip_prefix(&opts.input)
                        .map(|r| r.to_string_lossy().to_string())
                        .unwrap_or_else(|_| p.to_string_lossy().to_string());
                    (p, rel)
                })
                .collect(),
            Err(e) => {
                eprintln!("오류: {e}");
                std::process::exit(2);
            }
        }
    } else {
        let rel = opts
            .input
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| opts.input.to_string_lossy().to_string());
        vec![(opts.input.clone(), rel)]
    };

    if inputs.is_empty() {
        eprintln!(
            "오류: 처리할 .hwpx 파일이 없습니다: {}",
            opts.input.display()
        );
        std::process::exit(2);
    }

    let mut rows = Vec::with_capacity(inputs.len());
    for (path, rel) in &inputs {
        let rt_path = rt_output_path(&opts.out_dir, rel);
        let row = roundtrip_one(path, rel, &rt_path, opts.lineseg_report);
        let fmt_opt =
            |o: Option<usize>| o.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
        println!(
            "[{:>14}] diff={:>3} r2={:>3} {:>6}ms  {}",
            row.status(),
            fmt_opt(row.ir_diff_count),
            fmt_opt(row.round2_diff_count),
            row.elapsed_ms,
            row.rel_path
        );
        for detail in [&row.error, &row.pkg_problems, &row.round2_error] {
            if !detail.is_empty() {
                println!("                 └ {}", detail);
            }
        }
        rows.push(row);
    }

    if opts.batch {
        match write_tsv(&opts.out_dir, &rows) {
            Ok(p) => println!("\nTSV 저장: {}", p.display()),
            Err(e) => {
                eprintln!("오류: {e}");
                std::process::exit(1);
            }
        }
        print_summary(&rows);
    }

    if opts.lineseg_report {
        match write_lineseg_tsv(&opts.out_dir, &rows) {
            Ok(p) => {
                let r1: usize = rows.iter().map(|r| r.lineseg_r1.len()).sum();
                let r2: usize = rows.iter().map(|r| r.lineseg_r2.len()).sum();
                let touched = rows
                    .iter()
                    .filter(|r| !r.lineseg_r1.is_empty() || !r.lineseg_r2.is_empty())
                    .count();
                println!(
                    "lineseg TSV 저장: {} (round1={} round2={} 파일={})",
                    p.display(),
                    r1,
                    r2,
                    touched
                );
            }
            Err(e) => {
                eprintln!("오류: {e}");
                std::process::exit(1);
            }
        }
    }

    // 하드 실패(파싱/직렬화/재파싱/패키지/2-round 오류)가 있으면 비정상 종료 코드 (회귀 검출용)
    if rows.iter().any(|r| r.is_hard_fail()) {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_single_file() {
        let args = vec!["sample.hwpx".to_string()];
        let o = parse_args(&args).unwrap();
        assert_eq!(o.input, PathBuf::from("sample.hwpx"));
        assert!(!o.batch);
        assert_eq!(o.out_dir, PathBuf::from("output/poc/task1315"));
    }

    #[test]
    fn parse_args_batch_with_out() {
        let args = vec![
            "--batch".to_string(),
            "samples/hwpx".to_string(),
            "-o".to_string(),
            "output/poc/x".to_string(),
        ];
        let o = parse_args(&args).unwrap();
        assert!(o.batch);
        assert_eq!(o.input, PathBuf::from("samples/hwpx"));
        assert_eq!(o.out_dir, PathBuf::from("output/poc/x"));
    }

    #[test]
    fn parse_args_rejects_unknown_option() {
        let args = vec!["--nope".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parse_args_requires_input() {
        let args: Vec<String> = vec![];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn rt_output_path_keeps_subdir() {
        let p = rt_output_path(Path::new("out"), "ref/ref_text.hwpx");
        assert_eq!(p, PathBuf::from("out/ref/ref_text.rt.hwpx"));
    }

    #[test]
    fn rt_output_path_flat_file() {
        let p = rt_output_path(Path::new("out"), "blank_hwpx.hwpx");
        assert_eq!(p, PathBuf::from("out/blank_hwpx.rt.hwpx"));
    }

    #[test]
    fn tsv_escape_strips_tabs_newlines() {
        assert_eq!(tsv_escape("a\tb\nc"), "a b c");
    }

    #[test]
    fn roundtrip_one_blank_sample_passes() {
        let sample = Path::new("samples/hwpx/blank_hwpx.hwpx");
        if !sample.exists() {
            return; // 샘플 미존재 환경에서는 건너뜀
        }
        let tmp = std::env::temp_dir().join("rhwp_task1315_test_blank.rt.hwpx");
        let row = roundtrip_one(sample, "blank_hwpx.hwpx", &tmp, false);
        assert_eq!(row.status(), "PASS", "error={}", row.error);
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn parse_args_lineseg_report() {
        let args = vec![
            "--batch".to_string(),
            "samples/hwpx".to_string(),
            "--lineseg-report".to_string(),
        ];
        let o = parse_args(&args).unwrap();
        assert!(o.lineseg_report);
        // 기본은 비활성 — inventory.tsv 13컬럼 불변 보장.
        let o2 = parse_args(&["sample.hwpx".to_string()]).unwrap();
        assert!(!o2.lineseg_report);
    }
}
