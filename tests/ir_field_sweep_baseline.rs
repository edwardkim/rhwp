//! [#2740] IR 필드 전수 스윕 회귀 관문 — 왕복 불변식의 래칫.
//!
//! ## 이 게이트가 기존 게이트와 다른 점
//!
//! `tests/hwp5_roundtrip_baseline.rs`·`tests/hwpx_roundtrip_baseline.rs` 는 이미
//! 코퍼스를 전수로 돌린다. 다만 두 게이트가 쓰는 `diff_documents` 는 사건 대응으로
//! 누적된 수작업 화이트리스트라, `CharShape` 내용·`Paragraph.text`·표/셀 속성·
//! `CommonObjAttr` 배치 모델 같은 큰 영역을 **아예 보지 않는다**. 그래서 두 게이트가
//! 초록인 채로 1속성 왕복 소실이 계속 나왔다.
//!
//! 본 게이트는 `diagnostics::ir_field_sweep` 의 `Debug` 기반 전수 비교기를 코퍼스에
//! 걸고, 그 결과를 baseline 파일로 **동결**한다. 목적은 "지금 전부 무손실"이 아니라
//! **"오늘보다 나빠지면 즉시 실패"** 다.
//!
//! baseline v2는 정규화 경로별 **완전한 건수**만 받는다. 상세 예시 payload 상한과
//! 건수 집계를 분리하므로, 앞 경로의 개선이 뒤 경로의 기존 발산을 새 회귀처럼
//! 재배치하지 않는다. schema marker가 없는 예전 전역-cap baseline은 거부한다.
//!
//! ## 왜 baseline(래칫) 인가
//!
//! 스윕은 첫날부터 빨갛다(실측 결과는 이슈 #2740 §4). 첫날 빨간 게이트는 결국
//! 꺼지므로, 현 상태를 baseline 으로 고정하고 **증가분만** 실패시킨다. 그래야
//! CI 에 오늘 넣을 수 있고, 이후 모든 신규 왕복 소실이 기계적으로 걸린다.
//!
//! ## 래칫 방향 — 회귀만 실패
//!
//! `tests/opengov_corpus_snapshot.rs` 는 개선도 실패시켜 스냅샷 승격을 강제한다.
//! 본 게이트는 **회귀만** 실패시키고 개선은 메시지로만 알린다. 이유: 스윕 범위가
//! 직렬화기 전반이라 다른 작업의 무관한 수정도 수치를 낮추는데, 그때마다 남의 PR 이
//! 깨지면 게이트가 미움받아 꺼진다. 대신 baseline 재생성 절차를 아래에 고정한다.
//!
//! ## baseline 재생성
//!
//! ```text
//! RHWP_IR_SWEEP_DUMP=tests/fixtures/ir_field_sweep_baseline.tsv \
//!   cargo test --profile release-test --test ir_field_sweep_baseline -- --nocapture
//! ```
//!
//! ## CI 범위 — 전체 코퍼스가 기본값
//!
//! 코퍼스 794회 왕복 실측 **56.7초**(release-test, 단일 스레드)로 CI 예산 안에 들어와
//! 전체를 기본으로 돌린다. 부분집합만 돌리면 대형 실문서의 회귀를 놓치므로 굳이
//! 줄이지 않는다. 로컬에서 빠르게 확인할 때만 `RHWP_IR_SWEEP_FAST=1` 로
//! [`FAST_LIMIT`] 이하 샘플로 좁힌다(이때는 자기 몫의 행만 검사한다).
//!
//! ## 진단
//!
//! `RHWP_IR_SWEEP_DETAIL="샘플명조각;다른조각"` 을 주면 해당 샘플의 완전한
//! 경로별 건수와 상한 이내의 경로·값 예시를 출력한다 — 잔여 항목 분류용.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rhwp::diagnostics::ir_field_sweep::{
    sweep_hwp5_rebuild_roundtrip, sweep_hwp5_roundtrip, sweep_hwpx_roundtrip, tally,
    DivergenceReport,
};
use rhwp::parser::{detect_format, parse_document, FileFormat};

const BASELINE: &str = "tests/fixtures/ir_field_sweep_baseline.tsv";
const BASELINE_SCHEMA: &str = "# rhwp-ir-field-sweep-baseline-v2 complete-normalized-path-counts";
const BASELINE_HEADER: &str = "lane\tsample\tpath\tcount";
const HWP5_ROOT: &str = "samples";
const HWPX_ROOT: &str = "samples/hwpx";

/// `RHWP_IR_SWEEP_FAST=1` 일 때만 쓰는 샘플 크기 상한(바이트) — 로컬 반복 확인용.
/// 기본(CI 포함)은 전체 코퍼스다.
const FAST_LIMIT: u64 = 300 * 1024;

/// baseline 한 행 — (포맷, 샘플 상대경로, 정규화 경로) → 건수.
type Baseline = BTreeMap<(String, String, String), usize>;

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| !v.is_empty() && v != "0")
}

/// 기본은 전체 코퍼스. `RHWP_IR_SWEEP_FAST=1` 일 때만 부분집합으로 좁힌다.
fn full_mode() -> bool {
    !env_flag("RHWP_IR_SWEEP_FAST")
}

/// 확장자로 샘플을 재귀 수집해 루트 기준 상대 경로(슬래시)로 돌려준다.
/// `skip_dirs` 에 든 디렉터리 이름은 건너뛴다.
fn collect(root: &Path, ext: &str, skip_dirs: &[&str]) -> Vec<(PathBuf, String)> {
    fn walk(
        dir: &Path,
        root: &Path,
        ext: &str,
        skip_dirs: &[&str],
        acc: &mut Vec<(PathBuf, String)>,
    ) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skip = path
                    .file_name()
                    .is_some_and(|n| skip_dirs.iter().any(|s| n == *s));
                if !skip {
                    walk(&path, root, ext, skip_dirs, acc);
                }
            } else if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(ext))
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
    let mut acc = Vec::new();
    walk(root, root, ext, skip_dirs, &mut acc);
    acc.sort_by(|a, b| a.1.cmp(&b.1));
    acc
}

/// HWP5 범위 밖(자동 제외): HWP5 실체가 아니거나 배포용 문서.
/// `tests/hwp5_roundtrip_baseline.rs::out_of_scope` 와 같은 규칙.
fn hwp5_out_of_scope(bytes: &[u8]) -> bool {
    if detect_format(bytes) != FileFormat::Hwp {
        return true;
    }
    matches!(parse_document(bytes), Ok(doc) if doc.header.distribution)
}

/// 상세 출력 대상 필터 — `RHWP_IR_SWEEP_DETAIL` 의 `;` 구분 문자열 중 하나가
/// 샘플명에 있으면 그 샘플의 발산을 경로·값까지 전부 출력한다(메인테이너 분류용).
fn detail_filter() -> Option<Vec<String>> {
    let raw = std::env::var("RHWP_IR_SWEEP_DETAIL").ok()?;
    let parts: Vec<String> = raw
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    (!parts.is_empty()).then_some(parts)
}

fn print_detail(lane: &str, rel: &str, report: &DivergenceReport) {
    let Some(filters) = detail_filter() else {
        return;
    };
    if !filters.iter().any(|f| rel.contains(f.as_str())) {
        return;
    }
    println!("--- [{lane}] {rel} — 발산 {}건 ---", report.total());
    println!("  [정규화 경로별 완전한 건수]");
    for (path, count) in report.counts() {
        println!("  {count}\t{path}");
    }
    println!("  [상세 예시]");
    for d in report.examples() {
        println!("  {d}");
    }
    let omitted = report.total().saturating_sub(report.examples().len());
    if omitted > 0 {
        println!("  ... 상세 예시 상한 이후 {omitted}건 생략");
    }
}

/// 코퍼스를 스윕해 (레인, 샘플, 정규화경로) → 건수 로 집계한다.
/// 파싱/직렬화 자체가 실패하는 샘플은 기존 baseline 게이트의 몫이라 건너뛴다.
///
/// 레인은 3개다:
/// - `hwp5` — HWP5 왕복(원본 `raw_stream` 재생 경로)
/// - `hwp5rb` — HWP5 **레코드 재생성** 경로 (편집 후 저장과 동일; 진짜 직렬화기 측정)
/// - `hwpx` — HWPX 왕복
fn sweep_corpus(size_filter: &dyn Fn(u64) -> bool) -> (Baseline, usize, usize) {
    let mut acc: Baseline = BTreeMap::new();
    let mut swept = 0usize;
    let mut skipped = 0usize;

    let mut record = |lane: &str, rel: &str, report: &DivergenceReport| {
        print_detail(lane, rel, report);
        for (p, n) in tally(report) {
            acc.insert((lane.to_string(), rel.to_string(), p.clone()), *n);
        }
    };

    for (path, rel) in collect(Path::new(HWP5_ROOT), "hwp", &[]) {
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if !size_filter(size) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        if hwp5_out_of_scope(&bytes) {
            continue;
        }
        match sweep_hwp5_roundtrip(&bytes) {
            Ok(report) => {
                swept += 1;
                record("hwp5", &rel, &report);
            }
            Err(error) if error.is_capacity_error() => {
                panic!("[hwp5] {rel}: {error}")
            }
            Err(_) => skipped += 1,
        }
        match sweep_hwp5_rebuild_roundtrip(&bytes) {
            Ok(report) => {
                swept += 1;
                record("hwp5rb", &rel, &report);
            }
            Err(error) if error.is_capacity_error() => {
                panic!("[hwp5rb] {rel}: {error}")
            }
            Err(_) => skipped += 1,
        }
    }

    for (path, rel) in collect(Path::new(HWPX_ROOT), "hwpx", &["opengov"]) {
        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if !size_filter(size) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        match sweep_hwpx_roundtrip(&bytes) {
            Ok(report) => {
                swept += 1;
                record("hwpx", &rel, &report);
            }
            Err(error) if error.is_capacity_error() => {
                panic!("[hwpx] {rel}: {error}")
            }
            Err(_) => skipped += 1,
        }
    }

    (acc, swept, skipped)
}

fn parse_baseline(text: &str, source: &str) -> Result<Baseline, String> {
    let mut lines = text.lines();
    let schema = lines
        .next()
        .ok_or_else(|| format!("baseline 이 비어 있음: {source}"))?;
    if schema != BASELINE_SCHEMA {
        return Err(format!(
            "baseline schema 불일치: {source}: {schema:?} (기대: {BASELINE_SCHEMA:?})"
        ));
    }
    let header = lines
        .next()
        .ok_or_else(|| format!("baseline 헤더 없음: {source}"))?;
    if header != BASELINE_HEADER {
        return Err(format!(
            "baseline 헤더 불일치: {source}: {header:?} (기대: {BASELINE_HEADER:?})"
        ));
    }

    let mut map = Baseline::new();
    for (i, line) in lines.enumerate() {
        let line_no = i + 3;
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 4 {
            return Err(format!("baseline {source} {line_no}행 열 수 이상: {line}"));
        }
        let count: usize = cols[3]
            .trim()
            .parse()
            .map_err(|_| format!("baseline {source} {line_no}행 건수 파싱 실패"))?;
        let key = (
            cols[0].to_string(),
            cols[1].to_string(),
            cols[2].to_string(),
        );
        if map.insert(key, count).is_some() {
            return Err(format!("baseline {source} {line_no}행 키 중복: {line}"));
        }
    }
    Ok(map)
}

fn load_baseline() -> Baseline {
    let text = std::fs::read_to_string(BASELINE)
        .unwrap_or_else(|e| panic!("baseline 읽기 실패 {BASELINE}: {e}"));
    parse_baseline(&text, BASELINE).unwrap_or_else(|e| panic!("{e}"))
}

fn write_dump(path: &str, rows: &Baseline) {
    let mut s = format!("{BASELINE_SCHEMA}\n{BASELINE_HEADER}\n");
    for ((lane, sample, p), n) in rows {
        s.push_str(&format!("{lane}\t{sample}\t{p}\t{n}\n"));
    }
    if let Some(parent) = Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, s).unwrap_or_else(|e| panic!("덤프 쓰기 실패 {path}: {e}"));
}

#[derive(Debug, PartialEq, Eq)]
struct BaselineDelta {
    regressions: Vec<String>,
    improvements: usize,
}

fn compare_baseline(current: &Baseline, baseline: &Baseline, full: bool) -> BaselineDelta {
    let mut regressions = Vec::new();
    let mut improvements = 0usize;

    for (key, &now) in current {
        let was = baseline.get(key).copied().unwrap_or(0);
        if now > was {
            regressions.push(format!(
                "  [{}] {} — {} : {was} → {now}",
                key.0, key.1, key.2
            ));
        } else if now < was {
            improvements += 1;
        }
    }
    // baseline 에 있었는데 이번에 사라진 경로도 개선이다 (빠른 부분집합에서는
    // 대상 밖 샘플이 많으므로 전체 모드에서만 센다).
    if full {
        for key in baseline.keys() {
            if !current.contains_key(key) {
                improvements += 1;
            }
        }
    }

    BaselineDelta {
        regressions,
        improvements,
    }
}

/// 회귀 관문 — baseline 대비 **증가분**만 실패시킨다.
#[test]
fn ir_field_sweep_does_not_regress() {
    let full = full_mode();
    let started = std::time::Instant::now();
    let (current, swept, skipped) = if full {
        sweep_corpus(&|_| true)
    } else {
        sweep_corpus(&|sz| sz <= FAST_LIMIT)
    };
    let elapsed = started.elapsed();

    if let Ok(dump) = std::env::var("RHWP_IR_SWEEP_DUMP") {
        if !dump.is_empty() {
            write_dump(&dump, &current);
            println!("덤프 저장: {dump} ({}행)", current.len());
        }
    }

    let total: usize = current.values().sum();
    println!(
        "IR 필드 스윕: 샘플 {swept}건(스킵 {skipped}) / 발산 경로 {}종 / 총 {total}건 / {:.1}s{}",
        current.len(),
        elapsed.as_secs_f64(),
        if full {
            " [전체]"
        } else {
            " [빠른 부분집합]"
        }
    );

    assert!(swept > 0, "스윕 대상 샘플이 없음");

    let baseline = load_baseline();
    let delta = compare_baseline(&current, &baseline, full);

    if delta.improvements > 0 {
        println!(
            "개선 {}건 — baseline 재생성 권장 (RHWP_IR_SWEEP_DUMP={BASELINE})",
            delta.improvements
        );
    }

    assert!(
        delta.regressions.is_empty(),
        "IR 필드 왕복 발산이 baseline 보다 늘었다 ({}건). 직렬화기가 필드를 잃고 있다는 뜻이므로 \
         원인을 고치거나, 의도된 정규화라면 baseline 을 갱신하고 사유를 PR 에 남겨라:\n{}",
        delta.regressions.len(),
        delta.regressions.join("\n")
    );
}

/// baseline 정합 — 기록된 샘플이 실제로 존재해야 한다(목록 부패 방지).
#[test]
fn baseline_samples_exist() {
    let baseline = load_baseline();
    assert!(!baseline.is_empty(), "baseline 이 비어 있음: {BASELINE}");
    let mut missing = Vec::new();
    for (lane, sample, _) in baseline.keys() {
        let root = if lane == "hwpx" { HWPX_ROOT } else { HWP5_ROOT };
        let path = Path::new(root).join(sample);
        if !path.exists() {
            missing.push(format!("  [{lane}] {sample}"));
        }
    }
    assert!(
        missing.is_empty(),
        "baseline 에 실종 샘플 {}건 — 목록 정비 필요:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn legacy_capped_baseline_schema_is_rejected() {
    let legacy = "lane\tsample\tpath\tcount\nhwpx\tfixture.hwpx\tpath\t1\n";
    let error = parse_baseline(legacy, "legacy.tsv").unwrap_err();
    assert!(
        error.contains("schema 불일치"),
        "old capped counts must not be read as complete: {error}"
    );
}

#[test]
fn later_path_regression_is_not_offset_by_earlier_improvement() {
    let early = (
        "hwpx".to_string(),
        "fixture.hwpx".to_string(),
        "early.path".to_string(),
    );
    let later = (
        "hwpx".to_string(),
        "fixture.hwpx".to_string(),
        "later.path".to_string(),
    );
    let baseline = Baseline::from([(early.clone(), 1), (later.clone(), 2000)]);
    let current = Baseline::from([(later, 2001)]);

    let delta = compare_baseline(&current, &baseline, true);
    assert_eq!(delta.improvements, 1);
    assert_eq!(delta.regressions.len(), 1);
    assert!(delta.regressions[0].contains("later.path : 2000 → 2001"));
}
