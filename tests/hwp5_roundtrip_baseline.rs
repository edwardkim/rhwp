//! Task #1552 — `samples/*.hwp` 전수 HWP5 roundtrip 무손실 baseline 게이트.
//!
//! `mydocs/manual/hwp5_roundtrip_baseline.md` 의 등급 체계를 코드로 고정한다.
//! `hwpx_roundtrip_baseline.rs`(Task #1315)의 HWP5 대응.
//!
//! 검사(C1~C5): IR 뼈대 diff 0 + BinData 스트림 보존(decompressed 내용) +
//! CFB 구조 + 페이지수 복원(rhwp 자기 일관) + 2-round 안정성.
//!
//! - **A (baseline)**: 위 전부 통과. 목록에 없는 신규 HWP5 샘플도 자동 포함.
//! - **B (xfail)**: 식별된 결함으로 제외(사유 필수). 통과하게 되면 `xfail_entries_still_fail`
//!   가 실패 → baseline 승격.
//! - **자동 제외**: HWP5(`FileFormat::Hwp`)가 아닌 `.hwp`(HWP3 등)와 배포용 문서
//!   (`header.distribution`) — serializer 결함이 아니라 범위 밖.
//!
//! 주의: 구조(뼈대)+BinData+페이지(자기일관) 보존 검증이며 시각 충실도 보장이 아니다.
//! 외부 한글-only 페이지 붕괴(convert 경로 등)는 `output/poc/fidelity/` 한글 harness 보조.

use std::path::{Path, PathBuf};

use rhwp::diagnostics::hwp5_roundtrip_batch::baseline_check;
use rhwp::parser::{detect_format, parse_document, FileFormat};

const SAMPLES_ROOT: &str = "samples";

/// 대형 분리 기준(바이트). 이상은 `baseline_large_samples_roundtrip` 로 분리해
/// 하네스 병렬 실행을 활용한다.
const LARGE_THRESHOLD: u64 = 3 * 1024 * 1024;

/// B등급 (xfail) — (상대 경로, 사유). 사유 없는 등록 금지.
///
/// 과거 `serialize_document`(HWP5 직렬화)의 **BinData 그림 스트림 드롭**(F1, Task #1552
/// 조사) 9건이 등록되어 있었으나, Task #1554 에서 대응 BinData 레코드 없는 고아 /BinData
/// 스트림을 `extra_streams` 로 보존하도록 수정하여 전건 baseline 승격(목록에서 제거).
const XFAIL: &[(&str, &str)] = &[];

fn rel_of(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .expect("strip_prefix")
        .to_string_lossy()
        .replace('\\', "/")
}

/// `samples/` 에서 `.hwp` 를 재귀 수집해 루트 기준 상대 경로(슬래시 구분)로 반환.
fn collect_samples() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, root: &Path, acc: &mut Vec<(PathBuf, String)>) {
        let entries = std::fs::read_dir(dir).expect("samples 읽기 실패");
        for entry in entries {
            let path = entry.expect("디렉토리 항목 읽기 실패").path();
            if path.is_dir() {
                walk(&path, root, acc);
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("hwp"))
            {
                let rel = rel_of(&path, root);
                acc.push((path, rel));
            }
        }
    }
    let root = Path::new(SAMPLES_ROOT);
    let mut acc = Vec::new();
    walk(root, root, &mut acc);
    acc.sort_by(|a, b| a.1.cmp(&b.1));
    assert!(!acc.is_empty(), "samples 에 .hwp 샘플이 없음");
    acc
}

fn in_list(list: &[(&str, &str)], rel: &str) -> bool {
    list.iter().any(|(name, _)| *name == rel)
}

/// 범위 밖(자동 제외) 여부: HWP5 가 아니거나(HWP3 등) 배포용 문서.
/// `Some(사유)` 면 제외 대상.
fn out_of_scope(bytes: &[u8]) -> Option<&'static str> {
    if detect_format(bytes) != FileFormat::Hwp {
        return Some("HWP5 아님(HWP3/HWPML 등)");
    }
    match parse_document(bytes) {
        Ok(doc) if doc.header.distribution => Some("배포용 문서"),
        _ => None,
    }
}

/// baseline 대상(범위 밖/XFAIL 제외)을 검사하고 실패 목록을 단언한다.
fn run_baseline(size_filter: impl Fn(u64) -> bool) {
    let mut failures = Vec::new();
    let mut eligible = 0usize;

    for (path, rel) in collect_samples() {
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if !size_filter(size) || in_list(XFAIL, &rel) {
            continue;
        }
        let bytes = std::fs::read(&path).expect("읽기 실패");
        if out_of_scope(&bytes).is_some() {
            continue;
        }
        eligible += 1;
        if let Err(reason) = baseline_check(&bytes) {
            failures.push(format!("  {rel}: {reason}"));
        }
    }

    assert!(eligible > 0, "baseline 검사 대상이 없음");
    assert!(
        failures.is_empty(),
        "baseline 샘플 {}건 중 {}건 실패 — 결함 수정 또는 사유와 함께 XFAIL 등록 필요:\n{}",
        eligible,
        failures.len(),
        failures.join("\n")
    );
}

/// A등급 전수 게이트 (소형) — 신규 샘플은 자동 포함.
#[test]
fn baseline_all_samples_roundtrip() {
    run_baseline(|sz| sz <= LARGE_THRESHOLD);
}

/// A등급 전수 게이트 (대형) — 하네스 병렬 실행으로 wall time 단축.
#[test]
fn baseline_large_samples_roundtrip() {
    run_baseline(|sz| sz > LARGE_THRESHOLD);
}

/// B등급(xfail) 샘플은 여전히 실패해야 한다 — 통과하게 되면 baseline 승격 필요.
#[test]
fn xfail_entries_still_fail() {
    for (name, reason) in XFAIL {
        let path = Path::new(SAMPLES_ROOT).join(name);
        assert!(path.exists(), "XFAIL 샘플 실종: {name} (목록 정비 필요)");
        let bytes = std::fs::read(&path).expect("읽기 실패");
        assert!(
            out_of_scope(&bytes).is_none(),
            "XFAIL 은 범위 내(HWP5·편집가능) 샘플이어야 함: {name}"
        );
        assert!(
            baseline_check(&bytes).is_err(),
            "XFAIL 샘플이 통과함: {name} — baseline 으로 승격하고 XFAIL 에서 제거하라 (사유였던 결함: {reason})"
        );
    }
}
