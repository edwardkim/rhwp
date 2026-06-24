//! P0-1 — `samples/hwpx/` 전수 **렌더 시각 정합성** baseline 게이트.
//!
//! 기존 `hwpx_roundtrip_baseline` 이 IR 뼈대 보존만 보는 한계를 보완해, 라운드트립
//! (parse→serialize→reparse)이 **렌더 기하(페이지별 RenderNode bbox)** 를 바꾸는지 검사한다.
//! IrDiff 0 이어도 페이지 수 변화·노드 삽입/삭제·좌표 변위가 발생할 수 있으므로(autoNum 등),
//! 그 시각 축을 이 게이트가 잠근다.
//!
//! - **A (baseline)**: status PASS = 페이지 수 보존 + 구조 불일치 없음 + 최대 변위 ≤ 임계.
//!   목록에 없는 신규 샘플도 자동 포함 — 통과 못 하면 사유와 함께 `VISUAL_XFAIL` 에 등록.
//! - **B (visual_xfail)**: 현재 라운드트립 시각 드리프트가 있는 샘플(측정값 기록). 드리프트가
//!   해소되어 PASS 가 되면 `visual_xfail_entries_still_fail` 이 실패하므로 baseline 으로 승격.
//! - **제외**: HWPX 가 아닌 샘플.
//!
//! 주의(정직성): 이 게이트는 "rhwp 가 그린 원본 IR vs 라운드트립 IR" 의 **내부 정합성(회귀
//! 방지)** 만 본다. 한컴 정답지 충실도와는 별개다.
//!
//! 임계값 근거: 2026-06-24 전수 측정 결과 PASS 샘플의 매칭 노드 변위는 **정확히 0.0px**
//! (잡음 바닥 0). 0.5px 는 향후 서브픽셀 반올림 여유분이며, 실제 드리프트(≥6.8px)와 충분히 분리.

use std::path::{Path, PathBuf};

use rhwp::diagnostics::render_geom_diff::{roundtrip_geom, Via};

const SAMPLES_ROOT: &str = "samples/hwpx";

/// 변위 임계값(px). 측정 잡음 바닥 0.0px + 서브픽셀 여유.
const MAX_DISP: f64 = 0.5;

/// B등급(visual_xfail) — (상대 경로, 사유/측정값). 사유 없는 등록 금지.
/// 측정 기준: `rhwp render-diff --batch samples/hwpx` (2026-06-24).
/// 이들 드리프트의 본질 정정은 별도 이슈(직렬화/레이아웃 회귀 위험으로 분리).
const VISUAL_XFAIL: &[(&str, &str)] = &[
    // 승격 이력: 514.12px 그룹 자식(보도자료 ×3 + hwpx-h-01) — 컨테이너 shape_attr + pic
    // renderingInfo. 도형 회전 전치(shape-001) — 레거시 도형 shape_attr 블록. 쪽 테두리
    // (expense_report) — pageBorderFill borderFillIDRef. 바탕쪽(exam 6종 + 온새미로) —
    // masterPage 직렬화. 잔여는 아래 (차트/표/각주/이미지 등 별개 원인).
    ("143E433F503322BD33.hwpx", "차트 RawSvg→Placeholder 1페이지"),
    (
        "2026_oss_rst.hwpx",
        "459px 변위 + 구조 불일치 1페이지(TextRun)",
    ),
    (
        "el-school-001.hwpx",
        "구조 불일치 1페이지(도형 변위는 #shape 정정으로 0)",
    ),
    ("hy-002.hwpx", "603px 변위 + 구조 불일치 1페이지"),
    ("footnote-01.hwpx", "613px 변위 + 구조 불일치 4페이지(각주)"),
    ("aift.hwpx", "621px 변위 + 구조 불일치 5페이지(대형)"),
    ("k-water-rfp.hwpx", "622px 변위 + 구조 불일치 2페이지(대형)"),
];

/// 검사 제외 — 샘플 자체가 HWPX 패키지가 아님(HWP5 가 .hwpx 확장자로 저장됨).
const EXCLUDED: &[(&str, &str)] = &[(
    "hwpx-01.hwpx",
    "HWP5(OLE CFB) 파일이 .hwpx 확장자로 저장된 샘플 — serializer 결함 아님",
)];

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

/// 시각 정합성 검사: 페이지 수 보존 + 구조 불일치 없음 + 최대 변위 ≤ 임계. 실패 시 사유.
fn visual_check(path: &Path) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("읽기 실패: {e}"))?;
    let diff = roundtrip_geom(&bytes, Via::Hwpx).map_err(|e| format!("라운드트립 실패: {e:?}"))?;

    if diff.page_count_mismatch() {
        return Err(format!(
            "페이지 수 불일치: {} → {}",
            diff.page_count_a, diff.page_count_b
        ));
    }
    if diff.any_structure_mismatch() {
        let n = diff.pages.iter().filter(|p| p.structure_mismatch).count();
        return Err(format!("구조 불일치 {n}페이지(노드 삽입/삭제)"));
    }
    if diff.max_disp > MAX_DISP {
        return Err(format!(
            "최대 변위 {:.2}px > 임계 {:.2}px",
            diff.max_disp, MAX_DISP
        ));
    }
    Ok(())
}

/// A등급 전수 게이트 — VISUAL_XFAIL/EXCLUDED 외 전 샘플. 신규 샘플 자동 포함.
#[test]
fn visual_baseline_all_samples() {
    let mut failures = Vec::new();
    let mut eligible = 0usize;

    for (path, rel) in collect_samples() {
        if in_list(VISUAL_XFAIL, &rel) || in_list(EXCLUDED, &rel) {
            continue;
        }
        eligible += 1;
        if let Err(reason) = visual_check(&path) {
            failures.push(format!("  {rel}: {reason}"));
        }
    }

    assert!(eligible > 0, "시각 baseline 검사 대상이 없음");
    assert!(
        failures.is_empty(),
        "시각 baseline 샘플 {}건 중 {}건 실패 — 정정하거나 사유와 함께 VISUAL_XFAIL 등록 필요:\n{}",
        eligible,
        failures.len(),
        failures.join("\n")
    );
}

/// B등급(visual_xfail) 은 여전히 실패해야 한다 — PASS 가 되면 baseline 승격 필요.
#[test]
fn visual_xfail_entries_still_fail() {
    for (name, reason) in VISUAL_XFAIL {
        let path = Path::new(SAMPLES_ROOT).join(name);
        assert!(
            path.exists(),
            "VISUAL_XFAIL 샘플 실종: {name} (목록 정비 필요)"
        );
        assert!(
            visual_check(&path).is_err(),
            "VISUAL_XFAIL 샘플이 PASS 함: {name} — baseline 으로 승격하고 VISUAL_XFAIL 에서 제거하라 (기록 사유: {reason})"
        );
    }
}

/// 목록 정합 가드 — VISUAL_XFAIL/EXCLUDED 항목이 실제로 존재해야 한다.
#[test]
fn visual_grade_lists_are_consistent() {
    for (name, _) in VISUAL_XFAIL {
        assert!(
            Path::new(SAMPLES_ROOT).join(name).exists(),
            "VISUAL_XFAIL 샘플 실종: {name} (목록 정비 필요)"
        );
        assert!(
            !in_list(EXCLUDED, name),
            "VISUAL_XFAIL 과 EXCLUDED 중복 금지: {name}"
        );
    }
    for (name, _) in EXCLUDED {
        assert!(
            Path::new(SAMPLES_ROOT).join(name).exists(),
            "EXCLUDED 샘플 실종: {name} (목록 정비 필요)"
        );
    }
}
