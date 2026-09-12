//! 본문 바닥 넘김(`layout-anomaly` overflow 의 `over_bottom` 축) 원장 게이트 — samples 전수 래칫.
//!
//! # 왜 필요한가 (`#6976` 단계 0)
//!
//! "쪽 경계를 못 보고 본문 밖으로 그린다"는 결함이 갈래를 바꿔 가며 계속 들어온다.
//! 표 조각(`#6935` `#6973` `#6599`) · 예산과 실측의 분열(`#6923` `#6574`) · 글줄 이월
//! 누락(`#6920` `#6761` `#6970` `#6802`) 이 전부 같은 결과를 낸다 — **본문 바닥 아래에
//! 내용이 그려진다.**
//!
//! 그런데 이 축은 어느 래칫도 잠그지 않는다.
//!
//! | 기존 래칫 | 무엇을 보나 | 이 축을 잡나 |
//! | --- | --- | --- |
//! | `overflow_cell_baseline` | **셀 안** 줄의 윗변이 쪽 하단 밖 | 최상위 표·본문 글줄은 못 봄 |
//! | `off_canvas_baseline` | **용지** 상자 밖·`y < 0` | 본문만 넘고 용지 안이면 못 봄 |
//! | `text_overlap_baseline` | 글자 런 bbox 교차 | 겹치지 않고 넘기만 하면 못 봄 |
//!
//! `#6973` 이 그 사각지대의 실례다 — 표가 본문 바닥을 108.9px 넘었는데 용지는 0.5px 만
//! 넘어 off-canvas 공차(1.0px) 아래로 빠졌고, 자동 신호가 하나도 뜨지 않았다.
//!
//! # 실측 (devel `0d36da409`, samples 1,028건)
//!
//! ```text
//!   넘침 문서 319 (31%) · 넘침 쪽 2,058 · 넘침 노드 2,848
//! ```
//!
//! | 노드 | 건수 | 초과 중앙값 | ≤2px | >10px |
//! | --- | ---: | ---: | ---: | ---: |
//! | Table | 1,689 | 0.0px | 1,259 | 288 |
//! | TextLine | 1,059 | 18.3px | 110 | 708 |
//!
//! 표의 1,259건은 **테두리 반올림 잡음**이라 공차 밖으로 둔다(그 축은 `#6913` 이 따로
//! 다룬다). 그래서 이 게이트의 공차는 기본값 1.0px 이 아니라 **2.0px** 이고, 네 방향 중
//! **아래쪽(`over_bottom`)만** 센다 — 쪽 경계 축이 이 이슈의 대상이기 때문이다.
//!
//! # 판정 규약 (`off_canvas_baseline`·`overflow_cell_baseline` 과 같은 래칫)
//!
//! 기존 발생은 baseline 에 싣고 **신규 발생·증가만** 실패로 잡는다. 감소는 통과다 —
//! `#6976` 의 단계 1~4 가 줄여 나가면 dump 로 확인한 뒤 래칫을 조인다.
//!
//! 현재값 dump: `RHWP_BODY_OVERFLOW_DUMP=<path>`. 실패 시에는 전체 현재값을 stderr 에
//! TSV 로 남긴다 — 조판이 환경에 따라 갈리는 문서가 있어(`#6325`) 다른 환경의 baseline 을
//! 만들려면 그 정보가 필요하다.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;

const SAMPLES_ROOT: &str = "samples";
const BASELINE_PATH: &str = "tests/fixtures/body_overflow_baseline.tsv";
const PARTITIONS: usize = 16;
const SLOW_SAMPLE_LOG_THRESHOLD: Duration = Duration::from_secs(30);

/// 테두리 반올림 잡음을 거르는 공차(px). 모듈 주석의 분포 표가 근거다.
const BODY_OVERFLOW_TOLERANCE_PX: f64 = 2.0;

/// 전용 장기 sentinel 이 담당하는 fixture.
///
/// `issue2063_huge_cellbreak_table.hwp` 는 5만+ 셀 CellBreak 표로, layout-anomaly
/// 전수 래칫에서도 단일 문서가 수 분을 차지한다. `tests/issue_2063.rs` 가 해당 문서의
/// 완주 성능과 page-count pin 을 전담하므로 여기서는 중복 스캔하지 않는다.
const DEDICATED_SLOW_FIXTURES: &[&str] = &["issue2063_huge_cellbreak_table.hwp"];

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

/// 문서 하나의 전 페이지를 스캔해 **본문 바닥을 공차 넘게 벗어난** 노드 수를 센다.
///
/// 읽지 못한 문서는 None이다. 페이지 렌더 실패를 0건으로 세어 감소로 위장하지 않는다.
fn count_doc(path: &Path) -> Option<u64> {
    let bytes = std::fs::read(path).ok()?;
    let doc = DocumentCore::from_bytes(&bytes).ok()?;
    let opts = AnomalyOptions {
        overflow_tolerance_px: BODY_OVERFLOW_TOLERANCE_PX,
        ..AnomalyOptions::default()
    };
    let page_count = doc.page_count();
    let mut total = 0u64;
    for page in 0..page_count {
        let tree = doc.build_page_render_tree(page).ok()?;
        total += scan_page(page, &tree.root, page_count, &opts)
            .overflow
            .iter()
            .filter(|o| o.over_bottom > BODY_OVERFLOW_TOLERANCE_PX)
            .count() as u64;
    }
    Some(total)
}

fn body_overflow_does_not_grow_partition(part: usize) {
    let all_samples = collect_samples();
    let baseline = load_baseline();
    let all_rels: BTreeSet<_> = all_samples.iter().map(|(_, rel)| rel.as_str()).collect();
    let absent: Vec<_> = baseline
        .keys()
        .filter(|rel| !all_rels.contains(rel.as_str()))
        .collect();
    assert!(
        absent.is_empty(),
        "baseline 샘플이 삭제되거나 누락됨: {absent:?}"
    );
    let buckets = partition_samples(all_samples, PARTITIONS);
    let samples = buckets
        .into_iter()
        .nth(part)
        .unwrap_or_else(|| panic!("없는 partition: {part}"));
    assert!(
        !samples.is_empty(),
        "body-overflow partition {part} 이 비어 있음"
    );
    let selected_rels: BTreeSet<String> = samples.iter().map(|(_, rel)| rel.clone()).collect();

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
                        "body-overflow slow sample partition {part}/{PARTITIONS}: {:.3}s {rel}",
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
        "body-overflow 스윕 partition {part}/{PARTITIONS}: 샘플 {}건(스킵 {}) / 0 아닌 문서 {}종 / 총 {}건",
        results.len(),
        skipped.load(std::sync::atomic::Ordering::Relaxed),
        nonzero.len(),
        nonzero.values().sum::<u64>(),
    );

    if let Ok(dump) = std::env::var("RHWP_BODY_OVERFLOW_DUMP") {
        let dump = partition_dump_path(&dump, part);
        let mut out = String::new();
        for (rel, n) in &nonzero {
            out.push_str(&format!("{rel}\t{n}\n"));
        }
        std::fs::write(&dump, out).expect("dump 쓰기 실패");
        eprintln!("현재값 dump → {dump}");
    }

    let mut regressions = Vec::new();
    for (rel, &n) in &nonzero {
        match baseline.get(*rel) {
            None => regressions.push(format!("신규 발생: {rel} — {n}건 (baseline 없음)")),
            Some(&base) if n > base => regressions.push(format!("증가: {rel} — {base} → {n}건")),
            _ => {}
        }
    }
    if !regressions.is_empty() {
        // 실패한 환경의 전체 현재값을 남긴다 — 증가분만 찍으면 그 환경의 baseline 을
        // 만들려고 실패를 여러 번 반복해야 한다(#6325 와 같은 이유).
        eprintln!("---8<--- 현재값 전체 (baseline TSV 형식) ---8<---");
        for (rel, n) in &nonzero {
            eprintln!("{rel}\t{n}");
        }
        eprintln!("--->8--- 현재값 전체 끝 --->8---");
    }
    assert!(
        regressions.is_empty(),
        "본문 바닥 아래에 그려지는 요소가 늘었다(layout-anomaly overflow 의 over_bottom, 공차 {BODY_OVERFLOW_TOLERANCE_PX}px).\n\
         본문 밖으로 나간 내용은 꼬리말·쪽번호와 겹치거나 용지 밖으로 잘려 나간다.\n\
         원인 정정이 원칙이고, 의도된 변화만 baseline 에 반영한다(4.3.1 규약 준용).\n\
         현재값 전체는 위 `---8<---` 블록에 TSV 형식으로 찍혀 있다.\n\
         쪽 단위 위치 확인: rhwp layout-anomaly \"<문서>\" -p <쪽> --json\n{}",
        regressions.join("\n")
    );

    let missing: Vec<&String> = baseline
        .keys()
        .filter(|rel| selected_rels.contains(*rel) && !results.contains_key(*rel))
        .collect();
    assert!(
        missing.is_empty(),
        "baseline 샘플의 로드 또는 페이지 렌더 실패: {missing:?}"
    );
}

macro_rules! body_overflow_partition_tests {
    ($($name:ident => $part:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                body_overflow_does_not_grow_partition($part);
            }
        )+
    };
}

body_overflow_partition_tests!(
    body_overflow_does_not_grow_partition_0 => 0,
    body_overflow_does_not_grow_partition_1 => 1,
    body_overflow_does_not_grow_partition_2 => 2,
    body_overflow_does_not_grow_partition_3 => 3,
    body_overflow_does_not_grow_partition_4 => 4,
    body_overflow_does_not_grow_partition_5 => 5,
    body_overflow_does_not_grow_partition_6 => 6,
    body_overflow_does_not_grow_partition_7 => 7,
    body_overflow_does_not_grow_partition_8 => 8,
    body_overflow_does_not_grow_partition_9 => 9,
    body_overflow_does_not_grow_partition_10 => 10,
    body_overflow_does_not_grow_partition_11 => 11,
    body_overflow_does_not_grow_partition_12 => 12,
    body_overflow_does_not_grow_partition_13 => 13,
    body_overflow_does_not_grow_partition_14 => 14,
    body_overflow_does_not_grow_partition_15 => 15,
);
