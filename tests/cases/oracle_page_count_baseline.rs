//! 한글 정답지 PDF 대비 쪽수 원장 — 저장소 자산만으로 재현하는 v1.0 조판 판정.
//!
//! # 왜 필요한가
//!
//! v1.0 의 목표는 "한컴오피스와 같은 조판" 이고, 그 1차 지표는 **같은 문서를 몇 쪽으로
//! 조판하는가** 다. #5585 가 그 축을 10,000 문서로 재어 462 건(4.6%) 불일치를 보고했는데,
//! 그 측정은 **비공개 코퍼스와 한글 2022 설치본**이 있어야 재현된다. 외부 기여자는 자기
//! 변경이 그 축을 건드렸는지 확인할 방법이 없다.
//!
//! 이 저장소는 `pdf/` 에 한글이 직접 뽑은 출력 573 장을 갖고 있다. 그 쪽수는 **한글이 이
//! 문서를 몇 쪽으로 조판했는가** 의 정답이다. 이 시험은 그 정답지와 rhwp `page_count()` 를
//! 전수 대조해 **같은 축을 저장소 자산만으로** 판정한다 — 클론만 있으면 누구나 돌린다.
//!
//! # 판정 규약 (`overflow_cell_baseline` 과 같은 래칫)
//!
//! 픽스처 한 줄은 `상대경로 <TAB> 정답지쪽수(쉼표구분) <TAB> 기준선의 rhwp쪽수` 다.
//!
//! - rhwp 값이 **정답지와 일치** 하면 통과. 기준선이 무엇이든 상관없다(개선은 언제나 통과).
//! - 일치하지 않으면, 정답지와의 **차이가 기준선보다 커지지 않아야** 통과.
//! - 픽스처에 없는 문서가 정답지와 어긋나면 신규 발생으로 실패.
//!
//! 즉 현재 맞는 문서는 **하드 게이트**(어긋나는 순간 실패)이고, 지금 어긋난 문서는
//! 더 나빠질 때만 실패한다. 이 규약은 `local_validation.md` §4.3.0.1 의 "가능하면 최소
//! 공개 fixture 와 자동 래칫을 마련한다" 를 이 축에 적용한 것이다.
//!
//! # 모아 찍기 문서는 대상이 아니다
//!
//! `print_method` 가 모아 찍기(4·5)면 한글이 한 장에 여러 쪽을 실어 뽑으므로 장 수가 애초에
//! 다르다(`model::document::print_method_implies_nup` 주석의 실측표). 픽스처 생성 단계에서
//! 제외하며, 이 시험도 같은 값을 확인해 이중으로 막는다.
//!
//! **정답지의 용지 방향 같은 간접 신호로 추측하지 않는다.** 세로로 뽑힌 정답지를 2-up 으로
//! 오인하면 진짜 불일치를 삼킨다 — 픽스처를 만들며 실제로 겪었다(`hancom-hwp/hwpx-02.hwp`
//! 가 그렇게 가려졌다). 문서가 스스로 선언한 값만 쓴다.
//!
//! # 픽스처 재생성
//!
//! ```bash
//! python tools/oracle_page_count/regenerate.py --rhwp target/release-test/rhwp.exe
//! ```
#![cfg(not(target_arch = "wasm32"))]

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/deferred_exam_scope.rs"
));
include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/support/exam_fixtures.rs"
));

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rhwp::document_core::DocumentCore;
use rhwp::model::document::print_method_implies_nup;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const BASELINE_PATH: &str = "tests/fixtures/oracle_page_count_baseline.tsv";
const PARTITIONS: usize = 16;
const SLOW_SAMPLE_LOG_THRESHOLD: Duration = Duration::from_secs(30);
const DEDICATED_PAGE_COUNT_FIXTURES: &[(&str, &str)] = &[(
    "samples/issue2063_huge_cellbreak_table.hwp",
    "tests/issue_2063.rs::huge_cellbreak_table_paginates_without_quadratic_blowup",
)];

#[derive(Clone)]
struct Row {
    /// 한글이 뽑은 쪽수. 원본 형식·한컴 엔진이 확인된 canonical PDF만 모은다.
    /// 미표기·kopub/no-ttf 쪽수는 섞지 않는다 (#6374).
    oracle: Vec<u32>,
    /// 이 기준선을 찍을 때의 rhwp 쪽수.
    baseline: u32,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn dedicated_page_count_fixture(rel: &str) -> Option<&'static str> {
    DEDICATED_PAGE_COUNT_FIXTURES
        .iter()
        .find_map(|(path, owner)| (*path == rel).then_some(*owner))
}

fn load_baseline() -> BTreeMap<String, Row> {
    let path = repo_root().join(BASELINE_PATH);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("기준선 읽기 실패 {}: {e}", path.display()));
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split('\t');
        let rel = parts.next().expect("기준선 열 1(경로)");
        if is_exam_fixture(Path::new(rel)) || is_deferred_exam_document(rel) {
            continue;
        }
        let oracle_raw = parts.next().expect("기준선 열 2(정답지 쪽수)");
        let baseline_raw = parts.next().expect("기준선 열 3(기준 rhwp 쪽수)");
        let oracle: Vec<u32> = oracle_raw
            .split(',')
            .map(|s| s.trim().parse().expect("정답지 쪽수"))
            .collect();
        let baseline: u32 = baseline_raw.trim().parse().expect("기준 rhwp 쪽수");
        out.insert(rel.to_string(), Row { oracle, baseline });
    }
    assert!(!out.is_empty(), "기준선이 비어 있다: {BASELINE_PATH}");
    out
}

/// 문서 하나의 쪽수와 모아찍기 여부. 로드 실패는 이 게이트의 관심사가 아니다
/// (파싱 회귀는 기존 스위트가 잡는다).
fn measure(path: &Path) -> Option<(u32, bool)> {
    let bytes = std::fs::read(path).ok()?;
    let doc = DocumentCore::from_bytes(&bytes).ok()?;
    let nup = print_method_implies_nup(doc.document().doc_info.print_method);
    if path.ends_with("samples/basic/KTX-003.hwp") {
        verify_ktx_single_page_content(&doc);
    }
    if path.ends_with("samples/hwpx/hancom-hwp/issue_157.hwp") {
        verify_shareholder_forms_content(&doc);
    }
    Some((doc.page_count(), nup))
}

/// 독립 Print의 소집공고·참석장과 마지막 날짜·주의 문구가 실제 두 쪽에
/// 각각 한 번 남아 있어야 한다. 내용 손실로 쪽수만 맞추는 통과를 막는다.
fn verify_shareholder_forms_content(doc: &DocumentCore) {
    assert_eq!(doc.page_count(), 2, "독립 Print의 두 주주총회 서식");
    let mut owners = BTreeMap::<String, Vec<u32>>::new();
    fn collect(node: &RenderNode, page: u32, owners: &mut BTreeMap<String, Vec<u32>>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let text = node
                .children
                .iter()
                .filter_map(|child| match &child.node_type {
                    RenderNodeType::TextRun(run) => Some(run.display_or_text()),
                    _ => None,
                })
                .collect::<String>();
            owners.entry(text).or_default().push(page);
        }
        for child in &node.children {
            collect(child, page, owners);
        }
    }
    for page in 0..doc.page_count() {
        let tree = doc
            .build_page_render_tree(page)
            .expect("주주총회 서식 실제 쪽");
        collect(&tree.root, page, &mut owners);
    }
    for (text, page) in [
        ("주주총회 소집공고", 0),
        ("주주총회 참석장", 1),
        ("2026년       월       일", 1),
        ("※ 주주총회 참석자는 신분증을 필히 지참하시기 바랍니다.", 1),
    ] {
        assert_eq!(
            owners.get(text).map(Vec::as_slice),
            Some([page].as_slice()),
            "서식 내용의 쪽 소속·누락·중복: {text}"
        );
    }
}

/// 독립 PDF의 지도·시간표·운임표·주의 문구 관계를 기존 동일 원본 검사에서 보호한다.
fn verify_ktx_single_page_content(doc: &DocumentCore) {
    assert_eq!(
        doc.page_count(),
        1,
        "독립 PDF의 전체 내용은 같은 한 쪽에 속한다"
    );
    let tree = doc.build_page_render_tree(0).expect("KTX 원본 첫 쪽");
    fn collect<'a>(node: &'a RenderNode, lines: &mut Vec<(String, &'a RenderNode)>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let text = node
                .children
                .iter()
                .filter_map(|child| match &child.node_type {
                    RenderNodeType::TextRun(run) => Some(run.display_or_text()),
                    _ => None,
                })
                .collect::<String>();
            lines.push((text, node));
        }
        for child in &node.children {
            collect(child, lines);
        }
    }
    let mut lines = Vec::new();
    collect(&tree.root, &mut lines);
    let line = |text: &str| {
        let hits: Vec<_> = lines.iter().filter(|(value, _)| value == text).collect();
        assert_eq!(hits.len(), 1, "KTX 내용의 누락·중복: {text}");
        hits[0].1
    };
    let map = line("KTX 노선도");
    let time = line("KTX 소요시간");
    let fare = line("KTX 운임안내");
    let minimum = line("최저운임");
    let honam = line("호남선 (단위:원)");
    let gyeongbu = line("경부선 (단위:원)");
    let notice = line("※ KTX 소요 시간과 운임은 철도청의 사정에 따라 변동할 수 있습니다.");
    assert!(
        map.bbox.x < time.bbox.x,
        "지도 제목은 시간표의 왼쪽 단에 속한다"
    );
    for pair in [time, fare, minimum, honam, gyeongbu, notice].windows(2) {
        assert!(
            pair[0].bbox.y + pair[0].bbox.height <= pair[1].bbox.y,
            "시간표·운임 제목·최저운임·두 노선표·마지막 주의 문구의 순서와 비겹침을 유지한다"
        );
    }
}

/// 정답지와의 거리. 여러 정답지가 있으면 가장 가까운 것을 쓴다.
fn gap(oracle: &[u32], got: u32) -> u32 {
    oracle
        .iter()
        .map(|&o| o.abs_diff(got))
        .min()
        .expect("정답지 쪽수는 최소 1개")
}

fn partition_rows(
    root: &Path,
    mut rows: Vec<(String, Row)>,
    partitions: usize,
) -> Vec<Vec<(String, Row)>> {
    rows.sort_by_key(|(rel, _)| {
        let size = std::fs::metadata(root.join(rel))
            .map(|m| m.len())
            .unwrap_or(0);
        (std::cmp::Reverse(size), rel.clone())
    });
    let mut buckets: Vec<(u64, Vec<(String, Row)>)> =
        (0..partitions).map(|_| (0, Vec::new())).collect();
    for (rel, row) in rows {
        let size = std::fs::metadata(root.join(&rel))
            .map(|m| m.len())
            .unwrap_or(0);
        let index = buckets
            .iter()
            .enumerate()
            .min_by_key(|(i, (total, rows))| (*total, rows.len(), *i))
            .map(|(i, _)| i)
            .expect("partition bucket");
        buckets[index].0 += size.max(1);
        buckets[index].1.push((rel, row));
    }
    buckets.into_iter().map(|(_, rows)| rows).collect()
}

fn page_counts_do_not_drift_from_hancom_oracle_partition(part: usize) {
    let baseline = load_baseline();
    let root = repo_root();
    let buckets = partition_rows(
        &root,
        baseline
            .iter()
            .filter(|(k, _)| dedicated_page_count_fixture(k).is_none())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        PARTITIONS,
    );
    let selected = buckets
        .into_iter()
        .nth(part)
        .unwrap_or_else(|| panic!("없는 partition: {part}"));
    assert!(!selected.is_empty(), "oracle partition {part} 이 비어 있음");

    let workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
        .min(8);
    let queue = std::sync::Mutex::new(selected.iter());
    let failures = std::sync::Mutex::new(Vec::<String>::new());
    let stats = std::sync::Mutex::new((0usize, 0usize, 0usize)); // 일치 / 기존격차 / 건너뜀

    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let item = { queue.lock().unwrap().next() };
                let Some((rel, row)) = item else { break };
                let started = Instant::now();
                let Some((got, nup)) = measure(&root.join(rel)) else {
                    stats.lock().unwrap().2 += 1;
                    continue;
                };
                let elapsed = started.elapsed();
                if elapsed >= SLOW_SAMPLE_LOG_THRESHOLD {
                    eprintln!(
                        "oracle page-count slow sample partition {part}/{PARTITIONS}: {:.3}s {rel}",
                        elapsed.as_secs_f64()
                    );
                }
                if nup {
                    // 픽스처 생성 단계에서 제외되므로 여기 오면 문서 해석이 바뀐 것이다.
                    failures.lock().unwrap().push(format!(
                        "{rel}: 모아 찍기 선언이 새로 생겼다 — 기준선에서 빼야 한다"
                    ));
                    continue;
                }
                if row.oracle.contains(&got) {
                    stats.lock().unwrap().0 += 1;
                    continue;
                }
                let now = gap(&row.oracle, got);
                let before = gap(&row.oracle, row.baseline);
                if now > before {
                    failures.lock().unwrap().push(format!(
                        "{rel}: 정답지 {:?} 대비 격차가 커졌다 — 기준 {}쪽(차 {}) → 현재 {}쪽(차 {})",
                        row.oracle, row.baseline, before, got, now
                    ));
                } else {
                    stats.lock().unwrap().1 += 1;
                }
            });
        }
    });

    let (matched, known_gap, skipped) = *stats.lock().unwrap();
    eprintln!(
        "정답지 쪽수 대조 partition {part}/{PARTITIONS}: {}개 / 일치 {} / 기존 격차 유지·개선 {} / 건너뜀 {} / 전용 sentinel 제외 {}",
        selected.len(),
        matched,
        known_gap,
        skipped,
        DEDICATED_PAGE_COUNT_FIXTURES.len()
    );

    let failures = failures.into_inner().unwrap();
    assert!(
        failures.is_empty(),
        "한글이 뽑은 쪽수와의 격차가 커졌다.\n\
         쪽수는 v1.0 \"한컴과 같은 조판\"의 1차 지표다(#5585 와 같은 축).\n\
         의도된 변화만 기준선에 반영한다 — 재생성:\n\
         \x20 python tools/oracle_page_count/regenerate.py --rhwp target/release-test/rhwp.exe\n{}",
        failures.join("\n")
    );
}

macro_rules! oracle_partition_tests {
    ($($name:ident => $part:expr),+ $(,)?) => {
        $(
            #[test]
            fn $name() {
                page_counts_do_not_drift_from_hancom_oracle_partition($part);
            }
        )+
    };
}

oracle_partition_tests!(
    page_counts_do_not_drift_from_hancom_oracle_partition_0 => 0,
    page_counts_do_not_drift_from_hancom_oracle_partition_1 => 1,
    page_counts_do_not_drift_from_hancom_oracle_partition_2 => 2,
    page_counts_do_not_drift_from_hancom_oracle_partition_3 => 3,
    page_counts_do_not_drift_from_hancom_oracle_partition_4 => 4,
    page_counts_do_not_drift_from_hancom_oracle_partition_5 => 5,
    page_counts_do_not_drift_from_hancom_oracle_partition_6 => 6,
    page_counts_do_not_drift_from_hancom_oracle_partition_7 => 7,
    page_counts_do_not_drift_from_hancom_oracle_partition_8 => 8,
    page_counts_do_not_drift_from_hancom_oracle_partition_9 => 9,
    page_counts_do_not_drift_from_hancom_oracle_partition_10 => 10,
    page_counts_do_not_drift_from_hancom_oracle_partition_11 => 11,
    page_counts_do_not_drift_from_hancom_oracle_partition_12 => 12,
    page_counts_do_not_drift_from_hancom_oracle_partition_13 => 13,
    page_counts_do_not_drift_from_hancom_oracle_partition_14 => 14,
    page_counts_do_not_drift_from_hancom_oracle_partition_15 => 15,
);
