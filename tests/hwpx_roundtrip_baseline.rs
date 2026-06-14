//! Task #1315 — `samples/hwpx/` 전수 HWPX roundtrip baseline 게이트.
//!
//! `mydocs/manual/hwpx_roundtrip_baseline.md` 의 등급 체계를 코드로 고정한다.
//!
//! - **A (baseline)**: parse → serialize → 재parse + IrDiff 0 + 패키지 검사 + 2-round 안정성.
//!   목록에 없는 신규 샘플도 자동 포함 — 통과 못 하면 사유와 함께 `XFAIL` 에 등록해야 한다.
//! - **B (xfail)**: 식별된 결함/미지원으로 baseline 제외. 결함이 해소되어 통과하게 되면
//!   `xfail_entries_still_fail` 테스트가 실패하므로 baseline 으로 승격해야 한다.
//! - **제외**: 샘플 자체가 HWPX 가 아닌 경우 (serializer 결함 아님).
//!
//! 주의: 이 게이트는 **구조(뼈대) 보존** 검증이다. 통과 = 시각 충실도 보장이 아니며,
//! 복합 실문서를 full visual fidelity oracle 로 쓰면 안 된다 (`ORACLE_UNFIT` 참고).

use std::path::{Path, PathBuf};

use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::package_check::check_package;
use rhwp::serializer::hwpx::roundtrip::diff_documents;
use rhwp::serializer::hwpx::serialize_hwpx;

const SAMPLES_ROOT: &str = "samples/hwpx";

/// B등급 (xfail) — (상대 경로, 사유). 사유 없는 등록 금지.
///
/// #1379 2~3단계에서 subList(셀·글상자) 컨트롤 보존이 해소되어 25건이 baseline 으로
/// 승격됨 (task_m100_1379_stage3.md 측정). #1382 해소로 143E433F503322BD33 승격
/// (autoNum 폭 축 일관화 — task_m100_1382_stage2.md). #1384 해소(borderFill 등록
/// 1-based 정정 — task_m100_1384_stage2.md)로 잔존 4건(exam_kor/exam_social/
/// exam_social-p1/issue_1133) 승격 → **xfail 0**.
const XFAIL: &[(&str, &str)] = &[];

/// 검사 제외 — 샘플 자체가 HWPX 패키지가 아님.
const EXCLUDED: &[(&str, &str)] = &[(
    "hwpx-01.hwpx",
    "HWP5(OLE CFB) 파일이 .hwpx 확장자로 저장된 샘플 — 파일 매직 d0cf11e0, serializer 결함 아님",
)];

/// 시각 oracle 부적합 — baseline(구조) 은 통과하지만, 표·그림·도형·다단이 혼합된
/// 복합 실문서이므로 full visual fidelity oracle 로 쓰면 안 되는 샘플 (이슈 #1315 주의사항).
const ORACLE_UNFIT: &[&str] = &[
    "2024년 1분기 해외직접투자 보도자료 ff.hwpx",
    "2024년 2분기 해외직접투자 보도자료ff.hwpx",
    "2024년 연간 해외직접투자 보도자료 _ ff.hwpx",
    "2025년 1분기 해외직접투자 보도자료f.hwpx",
    "2025년 2분기 해외직접투자 (최종).hwpx",
    "[2027] 온새미로 1 본교재.hwpx",
    "aift.hwpx",
    "exam_kor.hwpx",
    "exam-kor-1p.hwpx",
    "exam-kor-2p.hwpx",
    "exam-kor-3p.hwpx",
    "exam-kor-4p.hwpx",
    "k-water-rfp.hwpx",
];

/// `samples/hwpx/` 에서 `.hwpx` 를 재귀 수집해 루트 기준 상대 경로(슬래시 구분)로 반환.
fn collect_samples() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, root: &Path, acc: &mut Vec<(PathBuf, String)>) {
        let entries = std::fs::read_dir(dir).expect("samples/hwpx 읽기 실패");
        for entry in entries {
            let path = entry.expect("디렉토리 항목 읽기 실패").path();
            if path.is_dir() {
                walk(&path, root, acc);
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("hwpx"))
            {
                let rel = path
                    .strip_prefix(root)
                    .expect("strip_prefix")
                    .to_string_lossy()
                    .replace('\\', "/");
                acc.push((path, rel));
            }
        }
    }
    let root = Path::new(SAMPLES_ROOT);
    let mut acc = Vec::new();
    walk(root, root, &mut acc);
    acc.sort_by(|a, b| a.1.cmp(&b.1));
    assert!(!acc.is_empty(), "samples/hwpx 에 .hwpx 샘플이 없음");
    acc
}

fn in_list(list: &[(&str, &str)], rel: &str) -> bool {
    list.iter().any(|(name, _)| *name == rel)
}

/// baseline 전체 검사: roundtrip + IrDiff 0 + 패키지 + 2-round. 실패 시 사유 반환.
fn baseline_check(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("읽기 실패: {e}"))?;
    let doc1 = parse_hwpx(&bytes).map_err(|e| format!("파싱 실패: {e}"))?;
    let out = serialize_hwpx(&doc1).map_err(|e| format!("직렬화 실패: {e}"))?;
    let doc2 = parse_hwpx(&out).map_err(|e| format!("재파싱 실패: {e}"))?;

    let diff = diff_documents(&doc1, &doc2);
    if !diff.is_empty() {
        return Err(format!(
            "IrDiff {}건: {}",
            diff.differences.len(),
            diff.differences
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }

    let pkg = check_package(&out, &doc1);
    if !pkg.is_ok() {
        return Err(format!("패키지 검사 실패: {}", pkg.summary()));
    }

    let out2 = serialize_hwpx(&doc2).map_err(|e| format!("2-round 직렬화 실패: {e}"))?;
    let doc3 = parse_hwpx(&out2).map_err(|e| format!("2-round 재파싱 실패: {e}"))?;
    let diff2 = diff_documents(&doc2, &doc3);
    if !diff2.is_empty() {
        return Err(format!(
            "2-round 불안정: IrDiff {}건",
            diff2.differences.len()
        ));
    }

    Ok(())
}

/// 처리 시간이 긴 대형 샘플 — 별도 테스트로 분리해 하네스 병렬 실행을 활용한다.
const LARGE: &[&str] = &["exam_kor.hwpx", "aift.hwpx", "k-water-rfp.hwpx"];

/// baseline 대상(XFAIL/EXCLUDED 제외)을 검사하고 실패 목록을 단언한다.
fn run_baseline(filter: impl Fn(&str) -> bool) {
    let mut failures = Vec::new();
    let mut eligible = 0usize;

    for (path, rel) in collect_samples() {
        if in_list(XFAIL, &rel) || in_list(EXCLUDED, &rel) || !filter(&rel) {
            continue;
        }
        eligible += 1;
        if let Err(reason) = baseline_check(&path) {
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

/// A등급 전수 게이트 (대형 샘플 제외분) — 신규 샘플은 자동으로 이 게이트에 포함된다.
#[test]
fn baseline_all_samples_roundtrip() {
    run_baseline(|rel| !LARGE.contains(&rel));
}

/// A등급 전수 게이트 (대형 샘플) — `LARGE` 목록만 검사.
#[test]
fn baseline_large_samples_roundtrip() {
    run_baseline(|rel| LARGE.contains(&rel));
}

/// B등급(xfail) 샘플은 여전히 실패해야 한다 — 통과하게 되면 baseline 승격 필요.
#[test]
fn xfail_entries_still_fail() {
    for (name, reason) in XFAIL {
        let path = Path::new(SAMPLES_ROOT).join(name);
        assert!(path.exists(), "XFAIL 샘플 실종: {name} (목록 정비 필요)");
        assert!(
            baseline_check(&path).is_err(),
            "XFAIL 샘플이 통과함: {name} — baseline 으로 승격하고 XFAIL 에서 제거하라 (사유였던 결함: {reason})"
        );
    }
}

/// 목록 정합 가드 — EXCLUDED/ORACLE_UNFIT 항목이 실제로 존재하고,
/// ORACLE_UNFIT 은 HWPX 샘플(EXCLUDED 아님)이어야 한다.
/// (XFAIL 과의 중복은 허용 — xfail 샘플도 시각 oracle 부적합 표시는 유효하며,
/// 해소 후 승격 시 표시가 그대로 살아 있어야 한다. 예: exam_kor #1384)
#[test]
fn grade_lists_are_consistent() {
    for (name, _) in EXCLUDED {
        assert!(
            Path::new(SAMPLES_ROOT).join(name).exists(),
            "EXCLUDED 샘플 실종: {name} (목록 정비 필요)"
        );
    }
    for name in ORACLE_UNFIT {
        assert!(
            Path::new(SAMPLES_ROOT).join(name).exists(),
            "ORACLE_UNFIT 샘플 실종: {name} (목록 정비 필요)"
        );
        assert!(
            !in_list(EXCLUDED, name),
            "ORACLE_UNFIT 은 HWPX 샘플이어야 함 (EXCLUDED 금지): {name}"
        );
    }
}
