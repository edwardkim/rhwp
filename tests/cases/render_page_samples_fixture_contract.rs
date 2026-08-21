//! `tests/fixtures/render_page_samples.tsv` 계약 — 저장소 안 자산만으로 도는 쪽수 정답지.
//!
//! `tools/render_page_gate.py` 는 원래 `C:/Users/planet/hwpdocs` 기준 픽스처만 갖고 있어
//! 기여자·CI 가 돌릴 수 없었다. 이 픽스처는 저장소에 이미 커밋된 `samples/` 문서와 그 짝인
//! `pdf/` 한글 출력에서 떠 온 것이라 아무 것도 더 받지 않고 게이트를 돌릴 수 있다.
//!
//! 여기서 고정하는 것은 **픽스처의 형태**다 — 값(쪽수)이 맞는지는 게이트가 실제 렌더로
//! 판정한다. 형태가 무너지면 게이트가 조용히 헛돌기 때문에 그것만 막는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_path() -> PathBuf {
    repo_root().join("tests/fixtures/render_page_samples.tsv")
}

struct Row {
    rel: String,
    hangul_pages: i64,
    rhwp_pages_baseline: i64,
    delta: i64,
}

fn read_rows() -> Vec<Row> {
    let path = fixture_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} 를 읽을 수 없다: {e}", path.display()));
    let mut lines = text.lines();
    let header = lines.next().expect("헤더 줄이 있어야 한다");
    assert_eq!(
        header, "rel\thangul_pages\trhwp_pages_baseline\tdelta",
        "열 이름이 render_page_gate.py 가 읽는 것과 같아야 한다"
    );

    let mut rows = Vec::new();
    for (n, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 4, "{}행: 열이 4개여야 한다 — {line:?}", n + 2);
        let num = |i: usize| -> i64 {
            cols[i]
                .parse::<i64>()
                .unwrap_or_else(|e| panic!("{}행 {}열이 정수가 아니다: {e}", n + 2, i + 1))
        };
        rows.push(Row {
            rel: cols[0].to_string(),
            hangul_pages: num(1),
            rhwp_pages_baseline: num(2),
            delta: num(3),
        });
    }
    rows
}

#[test]
fn fixture_has_rows_and_well_formed_columns() {
    let rows = read_rows();
    assert!(
        rows.len() >= 200,
        "저장소 안 정답지 쌍이 200건 이상이어야 한다 — 실측 {}건. \
         짝짓기 규칙이 깨졌거나 pdf/ 자산이 줄었을 수 있다",
        rows.len()
    );
}

#[test]
fn every_document_path_exists_in_the_repository() {
    let root = repo_root();
    let rows = read_rows();
    let missing: Vec<&str> = rows
        .iter()
        .filter(|r| !root.join(&r.rel).exists())
        .map(|r| r.rel.as_str())
        .take(10)
        .collect();
    assert!(
        missing.is_empty(),
        "픽스처가 가리키는 문서가 저장소에 없다(게이트가 조용히 건너뛴다): {missing:?}"
    );
}

#[test]
fn delta_column_matches_the_two_page_counts() {
    for r in read_rows() {
        assert_eq!(
            r.delta,
            r.rhwp_pages_baseline - r.hangul_pages,
            "{}: delta 는 rhwp − 한글 이어야 한다",
            r.rel
        );
    }
}

#[test]
fn page_counts_are_positive() {
    for r in read_rows() {
        assert!(
            r.hangul_pages > 0 && r.rhwp_pages_baseline > 0,
            "{}: 쪽수는 양수여야 한다 (한글 {}, rhwp {})",
            r.rel,
            r.hangul_pages,
            r.rhwp_pages_baseline
        );
    }
}

#[test]
fn oracle_is_never_a_page_range_excerpt() {
    // `pdf/…-p025-p035.pdf` 같은 발췌본을 정답지로 쓰면 델타가 통째로 거짓이 된다
    // (실측: 그 발췌본은 11쪽, 원본은 415쪽 — 델타가 +402 로 튀었다).
    // 생성기가 발췌본을 거르므로, 남은 델타는 모두 실제 쪽수 차이여야 한다.
    let rows = read_rows();
    let worst = rows.iter().map(|r| r.delta.abs()).max().unwrap_or(0);
    assert!(
        worst <= 50,
        "델타 절댓값이 {worst} 이다 — 정답지가 원본과 다른 문서(발췌본 등)일 때 나오는 크기다. \
         짝짓기를 먼저 의심할 것"
    );
}

#[test]
fn the_gate_script_reads_this_fixture_shape() {
    let gate = repo_root().join("tools/render_page_gate.py");
    let src = std::fs::read_to_string(&gate).expect("render_page_gate.py");
    for key in ["rel", "hangul_pages"] {
        assert!(
            src.contains(&format!("r[\"{key}\"]")),
            "게이트가 읽는 열 이름 {key} 가 스크립트에서 사라졌다 — 픽스처 헤더도 같이 고쳐야 한다"
        );
    }
    let _: &Path = gate.as_path();
}
