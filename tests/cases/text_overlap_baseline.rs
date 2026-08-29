//! [#6315] 글자 겹침(text-overlap) 원장 게이트 — samples 전수 렌더 래칫.
//!
//! `layout_anomaly` 의 text-overlap 판정(#5372)은 이미 있고 정확하다. 문제는 그
//! 판정을 **PR 에서 아무도 돌리지 않는 것**이었다 —
//! `.github/workflows/layout-anomaly-advisory.yml` 은 nightly advisory 이고
//! `continue-on-error: true` 라, 보이는 글자끼리 겹치는 회귀를 새로 만들어도 PR 은
//! 초록으로 통과했다. PR #6083 이 실제 사례다(편람 69쪽: devel 0건 → head 7건,
//! 상자 하단이 넘쳐 다음 본문 줄과 겹침). 검토자가 PNG 를 눈으로 대조한 뒤에야
//! 발견됐다.
//!
//! `ci_advisory.md` §2 가 게이트 승격을 미룬 이유는 "소표본에도 이미 알려진
//! overflow/overlap 이 있어 지금 강제하면 PR 이 한꺼번에 막힌다" 였다. 이 저장소는
//! 같은 문제를 `LAYOUT_OVERFLOW_CELL`(#3668)·`ir_field_sweep_baseline` 에서
//! **기존 발생은 baseline 에 싣고 신규·증가만 실패**시키는 래칫으로 이미 풀었다.
//! 이 게이트는 그 규약을 그대로 따른다.
//!
//! **판정 대상은 text-overlap 하나다.** overflow·off-canvas·overlap 은 컨테이너
//! 기하라 정상 조판의 접합·장식으로도 흔히 잡히지만, **보이는 글자끼리 겹치는 것은
//! 두 글자 모두 읽을 수 없게 되는 확정 결함**이다(모듈 머리말이 `--strict` 에
//! text-overlap 을 포함한 근거와 같다).
//!
//! ## baseline 재생성
//!
//! ```text
//! RHWP_TEXT_OVERLAP_DUMP=tests/fixtures/text_overlap_baseline.tsv \
//!   cargo test --profile release-test text_overlap_baseline -- --nocapture
//! ```
//!
//! 0 이 아닌 문서만 `상대경로\t건수` 로 사전순 기록한다. **감소는 통과**이므로,
//! 결함을 고쳤으면 dump 로 확인한 뒤 래칫을 조인다.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rhwp::diagnostics::layout_anomaly::{scan_document, AnomalyOptions};
use rhwp::document_core::DocumentCore;

const SAMPLES_ROOT: &str = "samples";
const BASELINE_PATH: &str = "tests/fixtures/text_overlap_baseline.tsv";

/// 확장자로 샘플을 재귀 수집해 루트 기준 상대 경로(슬래시)로 돌려준다.
fn collect_samples() -> Vec<(PathBuf, String)> {
    fn walk(dir: &Path, root: &Path, acc: &mut Vec<(PathBuf, String)>) {
        let entries = std::fs::read_dir(dir).expect("samples 읽기 실패");
        for entry in entries {
            let path = entry.expect("dir entry").path();
            if path.is_dir() {
                walk(&path, root, acc);
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("hwp") | Some("hwpx")
            ) {
                let rel = path
                    .strip_prefix(root)
                    .expect("strip_prefix")
                    .to_string_lossy()
                    .replace('\\', "/");
                acc.push((path, rel));
            }
        }
    }
    let mut acc = Vec::new();
    walk(Path::new(SAMPLES_ROOT), Path::new(SAMPLES_ROOT), &mut acc);
    acc.sort_by(|a, b| a.1.cmp(&b.1));
    assert!(!acc.is_empty(), "samples 에 hwp/hwpx 샘플이 없음");
    acc
}

fn load_baseline() -> BTreeMap<String, u64> {
    let text = std::fs::read_to_string(BASELINE_PATH)
        .unwrap_or_else(|e| panic!("baseline 읽기 실패 {BASELINE_PATH}: {e}"));
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (rel, n) = l.rsplit_once('\t').expect("baseline TSV 행 형식");
            (rel.to_string(), n.parse::<u64>().expect("baseline 수치"))
        })
        .collect()
}

/// 문서 하나의 전 페이지 text-overlap 건수.
///
/// 로드·렌더 실패는 이 게이트의 관심사가 아니므로 None(건너뜀)으로 처리한다 —
/// 크래시·파싱 회귀는 기존 스위트가 잡는다(overflow_cell_baseline 과 같은 규약).
fn count_doc(path: &Path) -> Option<u64> {
    let bytes = std::fs::read(path).ok()?;
    let doc = DocumentCore::from_bytes(&bytes).ok()?;
    let anomalies = scan_document(&doc, &AnomalyOptions::default()).ok()?;
    Some(anomalies.text_overlap_count() as u64)
}

#[test]
fn text_overlaps_do_not_grow() {
    let samples = collect_samples();
    let baseline = load_baseline();

    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(8);
    let queue = std::sync::Mutex::new(samples.into_iter());
    let results = std::sync::Mutex::new(BTreeMap::<String, u64>::new());
    let skipped = std::sync::atomic::AtomicUsize::new(0);

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let item = queue.lock().unwrap().next();
                let Some((path, rel)) = item else { break };
                match count_doc(&path) {
                    Some(n) => {
                        results.lock().unwrap().insert(rel, n);
                    }
                    None => {
                        skipped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
            });
        }
    });

    let results = results.into_inner().unwrap();
    let nonzero: BTreeMap<&String, u64> = results
        .iter()
        .filter(|(_, &n)| n > 0)
        .map(|(k, &v)| (k, v))
        .collect();

    eprintln!(
        "text-overlap 스윕: 샘플 {}건(스킵 {}) / 0 아닌 문서 {}종 / 총 {}건",
        results.len(),
        skipped.load(std::sync::atomic::Ordering::Relaxed),
        nonzero.len(),
        nonzero.values().sum::<u64>(),
    );

    if let Ok(dump) = std::env::var("RHWP_TEXT_OVERLAP_DUMP") {
        let mut out = String::new();
        for (rel, n) in &nonzero {
            out.push_str(&format!("{rel}\t{n}\n"));
        }
        std::fs::write(&dump, out).expect("dump 쓰기 실패");
        eprintln!("현재값 dump → {dump}");
    }

    // 래칫 판정: 신규 발생 또는 증가만 실패. 감소·해소는 통과(dump 대조로 조인다).
    let mut regressions = Vec::new();
    for (rel, &n) in &nonzero {
        match baseline.get(*rel) {
            None => regressions.push(format!("신규 발생: {rel} — {n}건 (baseline 없음)")),
            Some(&base) if n > base => regressions.push(format!("증가: {rel} — {base} → {n}건")),
            _ => {}
        }
    }
    assert!(
        regressions.is_empty(),
        "보이는 글자끼리 겹치는 회귀(text-overlap)가 늘었다.\n\
         겹친 두 글자는 **모두** 읽을 수 없게 되므로 확정 결함이다(#5372 판정).\n\
         `rhwp layout-anomaly <파일> -p <쪽> --json` 으로 짝을 확인할 수 있다.\n\
         원인 정정이 원칙이고, 의도된 변화만 baseline 에 반영한다(4.3.1 규약 준용).\n{}",
        regressions.join("\n")
    );

    // baseline 부패 감지: 기록된 문서가 코퍼스에서 사라지면 행을 정리해야 한다.
    let missing: Vec<&String> = baseline
        .keys()
        .filter(|rel| !results.contains_key(*rel))
        .collect();
    assert!(
        missing.is_empty(),
        "baseline 에 있으나 샘플에 없는 문서 — 행을 정리할 것: {missing:?}"
    );
}
