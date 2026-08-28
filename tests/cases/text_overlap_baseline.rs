//! 글자 겹침(`layout-anomaly` text-overlap) 원장 게이트 — samples 전수 렌더 래칫.
//!
//! `src/diagnostics/layout_anomaly.rs` 는 보이는 `TextRun` bbox 가 서로 교차하는
//! 사건(text-overlap)을 이미 판정한다. 그런데 그 판정을 돌리는 워크플로는
//! `.github/workflows/layout-anomaly-advisory.yml` 하나뿐이고, 그 잡은 nightly
//! advisory 다 — `pull_request` 트리거가 주석 처리돼 있고 `continue-on-error: true`
//! 라서 **PR 은 글자 겹침을 새로 만들어도 초록으로 통과한다**.
//!
//! advisory 설계문(`tools/layout_anomaly/ci_advisory.md` 2절)이 게이트 승격을
//! 미룬 이유는 "소표본에도 이미 알려진 overflow/overlap 이 있어 지금 강제 게이트로
//! 켜면 devel PR 이 한꺼번에 막힌다" 였다. 이 파일은 그 이유를 래칫으로 해소한다 —
//! 기존 겹침은 baseline 에 그대로 실어 통과시키고, **신규 발생과 증가만** 실패로
//! 잡는다. `overflow_cell_baseline.rs`(#3668)·`ir_field_sweep_baseline` 과 같은
//! 규약이고, `local_validation.md` 4.3.0.1 의 "가능하면 최소 공개 fixture 와 자동
//! 래칫을 마련한다" 를 이 축에 적용한 것이다.
//!
//! 판정 대상은 text-overlap 하나다. overflow·off-canvas·overlap 은 컨테이너 기하라
//! 정상 조판에서도 접합·장식으로 흔히 잡히지만, **보이는 글자끼리 겹치는 것**은
//! 문서로서 확정 결함이다(모듈 머리말의 `--strict` 포함 근거와 같다).
//!
//! 현재값 dump: `RHWP_TEXT_OVERLAP_DUMP=<path>` 로 실행하면 TSV 를 떨어뜨린다.
//! baseline 은 0 이 아닌 문서만 `상대경로\t건수` 사전순으로 기록한다.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
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

/// 문서 하나의 전 페이지를 스캔해 text-overlap 건수 합계를 센다.
/// 로드·렌더 실패는 이 게이트의 관심사가 아니므로 None(건너뜀)으로 처리한다 —
/// 크래시·파싱 회귀는 기존 스위트가 잡는다.
fn count_doc(path: &Path) -> Option<u64> {
    let bytes = std::fs::read(path).ok()?;
    let doc = DocumentCore::from_bytes(&bytes).ok()?;
    let opts = AnomalyOptions::default();
    let page_count = doc.page_count();
    let mut total = 0u64;
    for page in 0..page_count {
        let Ok(tree) = doc.build_page_render_tree(page) else {
            continue;
        };
        let anomalies = scan_page(page, &tree.root, page_count, &opts);
        total += anomalies.text_overlap.len() as u64;
    }
    Some(total)
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
    if !regressions.is_empty() {
        // 실패한 환경의 **전체** 현재값을 남긴다. 증가분만 찍으면 그 환경의 baseline 을
        // 다시 만들려고 실패를 여러 번 반복해야 한다 — 조판이 환경에 따라 갈리는
        // 문서가 있어(로컬과 CI 의 표 높이가 다른 사례 실측) 이 정보가 실제로 필요하다.
        // 아래 블록을 그대로 `tests/fixtures/text_overlap_baseline.tsv` 로 쓰면 된다.
        eprintln!("---8<--- 현재값 전체 (baseline TSV 형식) ---8<---");
        for (rel, n) in &nonzero {
            eprintln!("{rel}\t{n}");
        }
        eprintln!("--->8--- 현재값 전체 끝 --->8---");
    }
    assert!(
        regressions.is_empty(),
        "보이는 글자끼리 겹치는 사건(layout-anomaly text-overlap)이 늘었다.\n\
         한 글자 위에 다른 글자가 그려지면 두 글자 모두 읽을 수 없는 확정 결함이다.\n\
         원인 정정이 원칙이고, 의도된 변화만 baseline 에 반영한다(4.3.1 규약 준용).\n\
         현재값 전체는 위 `---8<---` 블록에 TSV 형식으로 찍혀 있다.\n\
         로컬 파일로 받으려면: RHWP_TEXT_OVERLAP_DUMP=<path> 로 재실행.\n\
         쪽 단위 위치 확인: rhwp layout-anomaly \"<문서>\" -p <쪽> --json\n{}",
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
