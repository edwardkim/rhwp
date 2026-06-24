//! Task #1499 — `samples/hwpx/` 전수 **렌더 기하 정합성** baseline 게이트.
//!
//! 기존 `hwpx_roundtrip_baseline` (Task #1315) 은 IR 뼈대(구조) 보존만 검증한다.
//! 본 게이트는 그 상위 단계로, 라운드트립(parse→serialize→reparse)이 **렌더 결과**
//! (페이지 수, 렌더 노드 삽입/삭제, bbox 변위)를 바꾸는지 검증한다 — `render-diff` 코어
//! (`diagnostics::render_geom_diff`) 의 자기 라운드트립 비교를 사용한다.
//!
//! 등급:
//! - **PASS (baseline)**: 페이지 수 보존 ∧ 렌더 노드 삽입/삭제 0 ∧ 매칭 노드 최대 변위 ≤ 임계.
//!   목록에 없는 신규 샘플도 자동 포함 — 통과 못 하면 사유와 함께 `XFAIL` 에 등록해야 한다.
//! - **XFAIL**: 식별된 라운드트립 렌더 회귀(이미지/노드 누락, 좌표 변위 등)로 baseline 제외.
//!   결함이 해소되어 PASS 하게 되면 `xfail_entries_still_pass_promote` 가 실패하므로 승격해야 한다.
//! - **EXCLUDED**: 샘플 자체가 HWPX 패키지가 아님 (serializer 결함 아님).
//!
//! 주의: 본 게이트는 "원본 IR 렌더 vs 라운드트립 IR 렌더" 의 **내부 정합성(회귀 방지)** 만
//! 보장한다. 자기 roundtrip PASS ≠ 한컴 정답지 시각 충실도 (별개 축).
//!
//! 임계: 실측 결과 PASS 샘플의 매칭 노드 변위는 전부 0.0px → 부동소수 재계산 잡음 여유를
//! 두어 `DEFAULT_THRESHOLD_PX = 0.5` 사용 (`task_m100_1499_report.md` 측정).

use std::path::{Path, PathBuf};

use rhwp::diagnostics::render_geom_diff::{self_roundtrip_diff, DEFAULT_THRESHOLD_PX};

const SAMPLES_ROOT: &str = "samples/hwpx";

/// XFAIL — (상대 경로, 사유). 사유 없는 등록 금지. 모두 IrDiff 0(구조 보존)이지만
/// 라운드트립이 렌더 결과를 바꾸는, hwpx_roundtrip_baseline 이 못 잡는 시각 회귀다.
/// (측정 근거: `output/poc/task1499/geom_inventory.tsv`, `task_m100_1499_report.md`)
const XFAIL: &[(&str, &str)] = &[
    // 임베디드 이미지/노드 누락 (단순 문서이므로 후속 우선 수정 후보)
    (
        "el-school-001.hwpx",
        "라운드트립이 임베디드 Image 노드 1개 누락 (IrDiff 0, render del=1)",
    ),
    (
        "143E433F503322BD33.hwpx",
        "라운드트립 렌더 노드 변동 (render ins=1 del=5, IrDiff 0)",
    ),
    (
        "expense_report.hwpx",
        "라운드트립 렌더 노드 누락 (render del=4, IrDiff 0)",
    ),
    (
        "hy-002.hwpx",
        "라운드트립 렌더 노드 누락 + 변위 (render del=1 disp=603px, IrDiff 0)",
    ),
    (
        "2026_oss_rst.hwpx",
        "라운드트립 렌더 노드 누락 + 변위 (render del=1 disp=459px, IrDiff 0)",
    ),
    (
        "exam_social-p1.hwpx",
        "라운드트립 렌더 노드 누락 (render del=13, IrDiff 0)",
    ),
    (
        "footnote-01.hwpx",
        "라운드트립 렌더 노드 누락 + 변위 (render del=35 disp=613px, IrDiff 0)",
    ),
    // 좌표 변위 (DISP_OVER)
    (
        "shape-001.hwpx",
        "라운드트립 도형 Path 변위 6.8px (> 0.5px 임계, IrDiff 0)",
    ),
    ("hwpx-h-01.hwpx", "라운드트립 Image 변위 514px (IrDiff 0)"),
    // 복합 실문서 (ORACLE_UNFIT) — 표·그림·도형·다단 혼합, 라운드트립 렌더 변화 known
    (
        "exam_social.hwpx",
        "ORACLE_UNFIT 복합 실문서 — render del=52 (IrDiff 0)",
    ),
    (
        "exam-kor-2p.hwpx",
        "ORACLE_UNFIT 시험지 — render del=43 (IrDiff 0)",
    ),
    (
        "exam-kor-3p.hwpx",
        "ORACLE_UNFIT 시험지 — render del=63 (IrDiff 0)",
    ),
    (
        "exam-kor-4p.hwpx",
        "ORACLE_UNFIT 시험지 — render del=83 (IrDiff 0)",
    ),
    (
        "exam_kor.hwpx",
        "ORACLE_UNFIT 시험지(대형) — render del=421 (IrDiff 0)",
    ),
    (
        "aift.hwpx",
        "ORACLE_UNFIT 복합 실문서(대형) — render del=34 disp=621px (IrDiff 0)",
    ),
    (
        "k-water-rfp.hwpx",
        "ORACLE_UNFIT 복합 실문서(대형) — render ins=13 disp=622px (IrDiff 0)",
    ),
    (
        "[2027] 온새미로 1 본교재.hwpx",
        "ORACLE_UNFIT 교재(대형) — render del=1281 (IrDiff 0)",
    ),
    (
        "2024년 1분기 해외직접투자 보도자료 ff.hwpx",
        "ORACLE_UNFIT 보도자료 — Image 변위 514px (IrDiff 0)",
    ),
    (
        "2024년 2분기 해외직접투자 보도자료ff.hwpx",
        "ORACLE_UNFIT 보도자료 — Image 변위 514px (IrDiff 0)",
    ),
    (
        "2024년 연간 해외직접투자 보도자료 _ ff.hwpx",
        "ORACLE_UNFIT 보도자료 — Image 변위 514px (IrDiff 0)",
    ),
];

/// 검사 제외 — 샘플 자체가 HWPX 패키지가 아님 (hwpx_roundtrip_baseline 과 동일 판정).
const EXCLUDED: &[(&str, &str)] = &[(
    "hwpx-01.hwpx",
    "HWP5(OLE CFB) 파일이 .hwpx 확장자로 저장된 샘플 — 파일 매직 d0cf11e0, serializer 결함 아님",
)];

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

/// 자기 라운드트립 렌더 기하 검사. PASS 면 Ok, 아니면 사유 반환.
fn visual_check(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("읽기 실패: {e}"))?;
    let diff = self_roundtrip_diff(&bytes, DEFAULT_THRESHOLD_PX)?;
    if diff.verdict().is_pass() {
        Ok(())
    } else {
        Err(format!(
            "{} (pages {}→{}, ins={} del={} max_disp={:.3}px)",
            diff.verdict().as_str(),
            diff.pages_a,
            diff.pages_b,
            diff.total_inserted,
            diff.total_deleted,
            diff.max_disp
        ))
    }
}

/// PASS baseline 전수 게이트 — 신규 샘플은 자동으로 포함된다.
#[test]
fn visual_baseline_all_samples() {
    let mut failures = Vec::new();
    let mut eligible = 0usize;

    for (path, rel) in collect_samples() {
        if in_list(XFAIL, &rel) || in_list(EXCLUDED, &rel) {
            continue;
        }
        eligible += 1;
        if let Err(reason) = visual_check(&path) {
            failures.push(format!("  {rel}: {reason}"));
        }
    }

    assert!(eligible > 0, "baseline 검사 대상이 없음");
    assert!(
        failures.is_empty(),
        "render-diff baseline {}건 중 {}건 실패 — 결함 수정 또는 사유와 함께 XFAIL 등록 필요:\n{}",
        eligible,
        failures.len(),
        failures.join("\n")
    );
}

/// XFAIL 샘플은 여전히 실패해야 한다 — PASS 하게 되면 baseline 으로 승격해야 한다.
#[test]
fn xfail_entries_still_pass_promote() {
    let mut promotable = Vec::new();
    for (name, reason) in XFAIL {
        let path = Path::new(SAMPLES_ROOT).join(name);
        assert!(path.exists(), "XFAIL 샘플 실종: {name} (목록 정비 필요)");
        if visual_check(&path).is_ok() {
            promotable.push(format!("  {name} (사유였던 결함: {reason})"));
        }
    }
    assert!(
        promotable.is_empty(),
        "XFAIL 샘플이 PASS 함 — baseline 으로 승격하고 XFAIL 에서 제거하라:\n{}",
        promotable.join("\n")
    );
}

/// 목록 정합 가드 — XFAIL/EXCLUDED 항목이 실제로 존재해야 한다.
#[test]
fn grade_lists_are_consistent() {
    for (name, _) in XFAIL {
        assert!(
            Path::new(SAMPLES_ROOT).join(name).exists(),
            "XFAIL 샘플 실종: {name} (목록 정비 필요)"
        );
    }
    for (name, _) in EXCLUDED {
        assert!(
            Path::new(SAMPLES_ROOT).join(name).exists(),
            "EXCLUDED 샘플 실종: {name} (목록 정비 필요)"
        );
    }
}
