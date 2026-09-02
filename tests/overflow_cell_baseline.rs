//! [#3668] `LAYOUT_OVERFLOW_CELL` 원장 게이트 — samples 전수 렌더 래칫.
//!
//! 셀 안 줄의 윗변이 쪽 하단 밖에 그려지면 그 줄은 어느 부분도 보이지 않는다
//! (판정 근거: `paragraph_layout.rs` 진단 주석, MATCH 80건 오탐 0). #3236 에서 이
//! 신호가 stderr 로만 나가 7일간 아무도 보지 못했다. 이 게이트는 samples 전수를
//! 렌더해 문서별 발생 줄 수를 세고, baseline(`tests/fixtures/overflow_cell_baseline.tsv`)
//! 대비 **신규 발생·증가만** 실패로 처리한다. 감소는 통과 — dump 로 확인 후 래칫을
//! 조인다 (ir_field_sweep_baseline 과 같은 규약).
//!
//! 현재값 dump: `RHWP_OVERFLOW_CELL_DUMP=<path>` 로 실행하면 TSV 를 떨어뜨린다.
//! baseline 은 0 이 아닌 문서만 `상대경로\t줄수` 사전순으로 기록한다.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rhwp::document_core::DocumentCore;

const SAMPLES_ROOT: &str = "samples";
const BASELINE_PATH: &str = "tests/fixtures/overflow_cell_baseline.tsv";
const PARTITIONS: usize = 16;
const SLOW_SAMPLE_LOG_THRESHOLD: Duration = Duration::from_secs(30);

/// 전용 장기 sentinel 이 담당하는 fixture.
///
/// `issue2063_huge_cellbreak_table.hwp` 는 5만+ 셀 CellBreak 표로, 이 원장에서도
/// 단일 문서가 수 분을 차지한다. `tests/issue_2063.rs`가 해당 문서의 완주 성능과
/// page-count pin을 전담하므로 여기서는 중복 스캔하지 않는다.
const DEDICATED_SLOW_FIXTURES: &[&str] = &["issue2063_huge_cellbreak_table.hwp"];

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
    acc.retain(|(_, rel)| !DEDICATED_SLOW_FIXTURES.contains(&rel.as_str()));
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

fn partition_samples(
    mut samples: Vec<(PathBuf, String)>,
    partitions: usize,
) -> Vec<Vec<(PathBuf, String)>> {
    samples.sort_by_key(|(path, rel)| {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        (std::cmp::Reverse(size), rel.clone())
    });
    let mut buckets: Vec<(u64, Vec<(PathBuf, String)>)> =
        (0..partitions).map(|_| (0, Vec::new())).collect();
    for (path, rel) in samples {
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        let index = buckets
            .iter()
            .enumerate()
            .min_by_key(|(i, (total, rows))| (*total, rows.len(), *i))
            .map(|(i, _)| i)
            .expect("partition bucket");
        buckets[index].0 += size.max(1);
        buckets[index].1.push((path, rel));
    }
    buckets.into_iter().map(|(_, rows)| rows).collect()
}

fn partition_dump_path(path: &str, part: usize) -> String {
    if PARTITIONS == 1 {
        path.to_string()
    } else {
        format!("{path}.part{part:02}-of{PARTITIONS:02}")
    }
}

/// 문서 하나의 전 페이지를 렌더해 `LAYOUT_OVERFLOW_CELL` 줄 수 합계를 센다.
/// 로드·렌더 실패는 이 게이트의 관심사가 아니므로 None(건너뜀)으로 처리한다 —
/// 크래시·파싱 회귀는 기존 스위트가 잡는다.
fn count_doc(path: &Path) -> Option<u64> {
    let bytes = std::fs::read(path).ok()?;
    let doc = DocumentCore::from_bytes(&bytes).ok()?;
    // 로드 잔여 카운트가 있다면 비우고 시작한다.
    let _ = doc.take_overflow_cell_lines();
    let mut total = 0u64;
    for page in 0..doc.page_count() {
        // overflow-cell 카운터는 레이아웃 중 증가한다. SVG 문자열 생성은 필요 없으므로
        // page render tree까지만 만들어 전수 gate의 wall time을 줄인다.
        if doc.build_page_render_tree(page).is_err() {
            let _ = doc.take_overflow_cell_lines();
            continue;
        }
        total += u64::from(doc.take_overflow_cell_lines());
    }
    Some(total)
}

fn overflow_cell_lines_do_not_grow_partition(part: usize) {
    let buckets = partition_samples(collect_samples(), PARTITIONS);
    let samples = buckets
        .into_iter()
        .nth(part)
        .unwrap_or_else(|| panic!("없는 partition: {part}"));
    assert!(
        !samples.is_empty(),
        "overflow-cell partition {part} 이 비어 있음"
    );
    let selected_rels: BTreeSet<String> = samples.iter().map(|(_, rel)| rel.clone()).collect();
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
                let started = Instant::now();
                match count_doc(&path) {
                    Some(n) => {
                        results.lock().unwrap().insert(rel.clone(), n);
                    }
                    None => {
                        skipped.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                let elapsed = started.elapsed();
                if elapsed >= SLOW_SAMPLE_LOG_THRESHOLD {
                    eprintln!(
                        "overflow-cell slow sample partition {part}/{PARTITIONS}: {:.3}s {rel}",
                        elapsed.as_secs_f64()
                    );
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
        "overflow-cell 스윕 partition {part}/{PARTITIONS}: 샘플 {}건(스킵 {}) / 0 아닌 문서 {}종 / 총 {}줄",
        results.len(),
        skipped.load(std::sync::atomic::Ordering::Relaxed),
        nonzero.len(),
        nonzero.values().sum::<u64>(),
    );

    if let Ok(dump) = std::env::var("RHWP_OVERFLOW_CELL_DUMP") {
        let dump = partition_dump_path(&dump, part);
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
            None => regressions.push(format!("신규 발생: {rel} — {n}줄 (baseline 없음)")),
            Some(&base) if n > base => regressions.push(format!("증가: {rel} — {base} → {n}줄")),
            _ => {}
        }
    }
    assert!(
        regressions.is_empty(),
        "쪽 밖 소실 줄(LAYOUT_OVERFLOW_CELL)이 늘었다.\n\
         셀 콘텐츠가 쪽 하단 밖에 그려져 사용자에게 보이지 않는 회귀다(#3236 계열).\n\
         원인 정정이 원칙이고, 의도된 변화만 baseline 에 반영한다(4.3.1 규약 준용).\n{}",
        regressions.join("\n")
    );

    // baseline 부패 감지: 기록된 문서가 코퍼스에서 사라지면 행을 정리해야 한다.
    let missing: Vec<&String> = baseline
        .keys()
        .filter(|rel| selected_rels.contains(*rel) && !results.contains_key(*rel))
        .collect();
    assert!(
        missing.is_empty(),
        "baseline 에 있으나 샘플에 없는 문서 — 행을 정리할 것: {missing:?}"
    );
}

macro_rules! overflow_cell_partition_tests {
    ($($name:ident => $part:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                overflow_cell_lines_do_not_grow_partition($part);
            }
        )+
    };
}

overflow_cell_partition_tests!(
    overflow_cell_lines_do_not_grow_partition_0 => 0,
    overflow_cell_lines_do_not_grow_partition_1 => 1,
    overflow_cell_lines_do_not_grow_partition_2 => 2,
    overflow_cell_lines_do_not_grow_partition_3 => 3,
    overflow_cell_lines_do_not_grow_partition_4 => 4,
    overflow_cell_lines_do_not_grow_partition_5 => 5,
    overflow_cell_lines_do_not_grow_partition_6 => 6,
    overflow_cell_lines_do_not_grow_partition_7 => 7,
    overflow_cell_lines_do_not_grow_partition_8 => 8,
    overflow_cell_lines_do_not_grow_partition_9 => 9,
    overflow_cell_lines_do_not_grow_partition_10 => 10,
    overflow_cell_lines_do_not_grow_partition_11 => 11,
    overflow_cell_lines_do_not_grow_partition_12 => 12,
    overflow_cell_lines_do_not_grow_partition_13 => 13,
    overflow_cell_lines_do_not_grow_partition_14 => 14,
    overflow_cell_lines_do_not_grow_partition_15 => 15,
);
