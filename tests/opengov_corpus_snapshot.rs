//! Task #1564 — opengov 고정 실문서 회귀 말뭉치 스냅샷 게이트.
//!
//! `samples/hwpx/opengov/` 의 동결 실문서를 parse→serialize→reparse 하여 현재
//! status/ir_diff_count 를 산출하고, 골든 스냅샷(`tests/fixtures/opengov_snapshot.tsv`)과
//! 비교한다. **악화**(PASS→IR_DIFF, diff 증가, FAIL)는 회귀로 실패하고, **개선**
//! (IR_DIFF→PASS, diff 감소)도 실패시켜 스냅샷 갱신(승격)을 강제한다.
//!
//! diff=0 강제 baseline(`hwpx_roundtrip_baseline`)과 별개 — 실문서는 대다수 IR_DIFF 라
//! 구조 보존 여부가 아니라 **status 분포의 회귀**를 추적한다.
//! 한글 전용 페이지 붕괴는 `tools/verify_hangul_pages.py`(#1560)로 별도 검증.

use std::collections::BTreeMap;
use std::path::Path;

use rhwp::parser::hwpx::parse_hwpx;
use rhwp::serializer::hwpx::roundtrip::diff_documents;
use rhwp::serializer::hwpx::serialize_hwpx;

const CORPUS_DIR: &str = "samples/hwpx/opengov";
const SNAPSHOT: &str = "tests/fixtures/opengov_snapshot.tsv";

/// (status, ir_diff_count) 를 정렬 가능한 심각도 키로. 큰 값 = 더 나쁨.
fn severity(status: &str, diff: usize) -> (u8, usize) {
    let tier = match status {
        "PASS" => 0,
        "IR_DIFF" => 1,
        "REPARSE_FAIL" => 2,
        "SERIALIZE_FAIL" => 3,
        _ => 4, // PARSE_FAIL/기타
    };
    (tier, if status == "IR_DIFF" { diff } else { 0 })
}

/// 파일 1건의 현재 (status, diff) 산출.
fn measure(path: &Path) -> (String, usize) {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return ("PARSE_FAIL".into(), 0),
    };
    let doc1 = match parse_hwpx(&bytes) {
        Ok(d) => d,
        Err(_) => return ("PARSE_FAIL".into(), 0),
    };
    let out = match serialize_hwpx(&doc1) {
        Ok(o) => o,
        Err(_) => return ("SERIALIZE_FAIL".into(), 0),
    };
    let doc2 = match parse_hwpx(&out) {
        Ok(d) => d,
        Err(_) => return ("REPARSE_FAIL".into(), 0),
    };
    let diff = diff_documents(&doc1, &doc2).differences.len();
    if diff == 0 {
        ("PASS".into(), 0)
    } else {
        ("IR_DIFF".into(), diff)
    }
}

/// 골든 스냅샷 로드: id -> (status, diff).
fn load_snapshot() -> BTreeMap<String, (String, usize)> {
    let text = std::fs::read_to_string(SNAPSHOT).expect("스냅샷 읽기 실패");
    let mut map = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue; // 헤더/빈 줄
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols.len() >= 3, "스냅샷 컬럼 부족: {line}");
        let id = cols[0].trim().to_string();
        let status = cols[1].trim().to_string();
        let diff: usize = cols[2].trim().parse().expect("ir_diff_count 파싱");
        map.insert(id, (status, diff));
    }
    map
}

/// 말뭉치 파일들: id -> 경로.
fn corpus_files() -> BTreeMap<String, std::path::PathBuf> {
    let mut map = BTreeMap::new();
    let dir = Path::new(CORPUS_DIR);
    for entry in std::fs::read_dir(dir).expect("opengov 말뭉치 폴더 읽기 실패") {
        let path = entry.expect("항목").path();
        if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("hwpx"))
        {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            let id = name.split('_').next().unwrap_or("").to_string();
            map.insert(id, path);
        }
    }
    map
}

#[test]
fn opengov_corpus_matches_snapshot() {
    let snapshot = load_snapshot();
    let files = corpus_files();

    let mut regressions = Vec::new();
    let mut improvements = Vec::new();

    for (id, path) in &files {
        let (gstatus, gdiff) = match snapshot.get(id) {
            Some(v) => v.clone(),
            None => {
                regressions.push(format!(
                    "  {id}: 스냅샷에 없음(신규 파일 — 스냅샷 행 추가 필요)"
                ));
                continue;
            }
        };
        let (cstatus, cdiff) = measure(path);
        let cur = severity(&cstatus, cdiff);
        let gold = severity(&gstatus, gdiff);
        if cur > gold {
            regressions.push(format!(
                "  {id}: 악화 {gstatus}/{gdiff} → {cstatus}/{cdiff}"
            ));
        } else if cur < gold {
            improvements.push(format!(
                "  {id}: 개선 {gstatus}/{gdiff} → {cstatus}/{cdiff}"
            ));
        }
    }

    assert!(
        regressions.is_empty(),
        "opengov 말뭉치 회귀 {}건 — 결함 조사 필요:\n{}",
        regressions.len(),
        regressions.join("\n")
    );
    assert!(
        improvements.is_empty(),
        "opengov 말뭉치 개선 {}건 — tests/fixtures/opengov_snapshot.tsv 갱신(승격) 필요:\n{}",
        improvements.len(),
        improvements.join("\n")
    );
}

#[test]
fn opengov_snapshot_and_corpus_consistent() {
    let snapshot = load_snapshot();
    let files = corpus_files();
    for id in snapshot.keys() {
        assert!(
            files.contains_key(id),
            "스냅샷에 있으나 말뭉치 파일 실종: {id}"
        );
    }
    for id in files.keys() {
        assert!(
            snapshot.contains_key(id),
            "말뭉치에 있으나 스냅샷 행 없음: {id} (스냅샷 갱신 필요)"
        );
    }
    assert!(!files.is_empty(), "opengov 말뭉치가 비어 있음");
}
