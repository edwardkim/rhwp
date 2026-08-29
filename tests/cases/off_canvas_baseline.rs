//! 용지 밖 그리기(`layout-anomaly` off-canvas) 원장 게이트 — samples 전수 래칫.
//!
//! # 왜 필요한가
//!
//! `layout-anomaly` 는 요소가 **페이지 상자 밖**(또는 `y < 0`)에 놓인 것을 off-canvas 로
//! 판정한다. 본문 여백을 넘은 overflow 와 달리 이쪽은 **종이 밖**이라 사용자에게 그 내용이
//! 아예 보이지 않는다.
//!
//! 그런데 그 판정을 돌리는 워크플로는 nightly advisory 하나뿐이라
//! (`.github/workflows/layout-anomaly-advisory.yml` — `pull_request` 트리거 주석 처리,
//! `continue-on-error: true`), PR 이 새 off-canvas 를 만들어도 초록으로 통과한다.
//!
//! 글자 겹침 축은 #6315 가 같은 방식으로 막았다. 이 시험은 **그 게이트가 보지 않는**
//! off-canvas 축을 맡는다 — 둘은 겹치지 않는 결함군이다.
//!
//! # 실측이 말하는 것 (devel `f6a6bee8f3`, samples 945 건)
//!
//! | 신호 | 건수 | 문서 |
//! | --- | ---: | ---: |
//! | text-overlap (#6315 가 담당) | 4,408 | 153 |
//! | **off-canvas (이 시험)** | **437** | **78** |
//!
//! off-canvas 437 건의 성격이다.
//!
//! - 노드 종류: `TextLine` 321 · `Table` 96 · `Image` 18 · `Group` 2
//! - 밖으로 나간 정도: 중앙값 **103.7px**, 90 퍼센타일 363.2px, 최대 4,043.6px
//! - 2px 이하(경계 스침)는 6 건뿐이고 **397 건이 10px 초과**다
//!
//! 즉 경계 반올림 잡음이 아니라 내용이 실제로 종이 밖에 있다. 최다 문서
//! `basic/sungeo.hwp` 는 105 건이고, 같은 문서가 한글 정답지 대비 8 쪽 부족하다
//! (#6337 원장의 최대 격차). 두 신호가 같은 뿌리 — 한 쪽에 너무 많이 담는 것 — 를 가리킨다.
//!
//! # 판정 규약 (`overflow_cell_baseline`·`text_overlap_baseline` 과 같은 래칫)
//!
//! 기존 발생은 baseline 에 싣고 **신규 발생·증가만** 실패로 잡는다. 감소는 통과다.
//! `local_validation.md` §4.3.0.1 의 "가능하면 최소 공개 fixture 와 자동 래칫을 마련한다"
//! 를 이 축에 적용한 것이다.
//!
//! 현재값 dump: `RHWP_OFF_CANVAS_DUMP=<path>`. 실패 시에는 전체 현재값을 stderr 에
//! TSV 로 남긴다 — 조판이 환경에 따라 갈리는 문서가 있어(#6325) 다른 환경의 baseline 을
//! 만들려면 그 정보가 필요하다.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::document_core::DocumentCore;

const SAMPLES_ROOT: &str = "samples";
const BASELINE_PATH: &str = "tests/fixtures/off_canvas_baseline.tsv";
const PARTITIONS: usize = 16;
const SLOW_SAMPLE_LOG_THRESHOLD: Duration = Duration::from_secs(30);

/// 전용 장기 sentinel 이 담당하는 fixture.
///
/// `issue2063_huge_cellbreak_table.hwp` 는 5만+ 셀 CellBreak 표로, layout-anomaly
/// 전수 래칫에서도 단일 문서가 수 분을 차지한다. `tests/issue_2063.rs`가 해당 문서의
/// 완주 성능과 page-count pin을 전담하므로 여기서는 중복 스캔하지 않는다.
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

/// 문서 하나의 전 페이지를 스캔해 off-canvas 건수 합계를 센다.
/// 로드·렌더 실패는 이 게이트의 관심사가 아니다(파싱 회귀는 기존 스위트가 잡는다).
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
        total += scan_page(page, &tree.root, page_count, &opts)
            .off_canvas
            .len() as u64;
    }
    Some(total)
}

fn off_canvas_does_not_grow_partition(part: usize) {
    let buckets = partition_samples(collect_samples(), PARTITIONS);
    let samples = buckets
        .into_iter()
        .nth(part)
        .unwrap_or_else(|| panic!("없는 partition: {part}"));
    assert!(
        !samples.is_empty(),
        "off-canvas partition {part} 이 비어 있음"
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
                        "off-canvas slow sample partition {part}/{PARTITIONS}: {:.3}s {rel}",
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
        "off-canvas 스윕 partition {part}/{PARTITIONS}: 샘플 {}건(스킵 {}) / 0 아닌 문서 {}종 / 총 {}건",
        results.len(),
        skipped.load(std::sync::atomic::Ordering::Relaxed),
        nonzero.len(),
        nonzero.values().sum::<u64>(),
    );

    if let Ok(dump) = std::env::var("RHWP_OFF_CANVAS_DUMP") {
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
        // 만들려고 실패를 여러 번 반복해야 한다. 조판이 환경에 따라 갈리는 문서가
        // 있어(#6325) 이 정보가 실제로 필요하다.
        eprintln!("---8<--- 현재값 전체 (baseline TSV 형식) ---8<---");
        for (rel, n) in &nonzero {
            eprintln!("{rel}\t{n}");
        }
        eprintln!("--->8--- 현재값 전체 끝 --->8---");
    }
    assert!(
        regressions.is_empty(),
        "용지 밖에 그려지는 요소(layout-anomaly off-canvas)가 늘었다.\n\
         페이지 상자 밖이나 y<0 에 놓인 내용은 사용자에게 아예 보이지 않는다.\n\
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
        "baseline 에 있으나 샘플에 없는 문서 — 행을 정리할 것: {missing:?}"
    );
}

macro_rules! off_canvas_partition_tests {
    ($($name:ident => $part:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                off_canvas_does_not_grow_partition($part);
            }
        )+
    };
}

off_canvas_partition_tests!(
    off_canvas_does_not_grow_partition_0 => 0,
    off_canvas_does_not_grow_partition_1 => 1,
    off_canvas_does_not_grow_partition_2 => 2,
    off_canvas_does_not_grow_partition_3 => 3,
    off_canvas_does_not_grow_partition_4 => 4,
    off_canvas_does_not_grow_partition_5 => 5,
    off_canvas_does_not_grow_partition_6 => 6,
    off_canvas_does_not_grow_partition_7 => 7,
    off_canvas_does_not_grow_partition_8 => 8,
    off_canvas_does_not_grow_partition_9 => 9,
    off_canvas_does_not_grow_partition_10 => 10,
    off_canvas_does_not_grow_partition_11 => 11,
    off_canvas_does_not_grow_partition_12 => 12,
    off_canvas_does_not_grow_partition_13 => 13,
    off_canvas_does_not_grow_partition_14 => 14,
    off_canvas_does_not_grow_partition_15 => 15,
);
