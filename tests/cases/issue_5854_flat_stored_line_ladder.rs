//! [#5854] 저장 LINE_SEG 사다리가 **단일 치수 튜플로 굳어 있으면** 그 치수를 믿지 않는다.
//!
//! 종전(devel `65f71270f7`): `samples/hwpx/hwpx-02.hwpx` 는 4쪽이 통째로 비었다.
//! 이 문서의 lineseg 122개는 2pt~15pt 문단이 섞여 있는데도 전부
//! `(vertsize=1000, textheight=1000, baseline=850, spacing=600)` 하나다 — 한글이 실제로
//! 조판해 저장한 값이 아니라 합성값이다. rhwp 가 그 치수를 그대로 쓰면 4pt 문단도
//! 10pt 줄 높이를 갖고, 3쪽이 29px 일찍 차서 빈 문단 두 개가 자기 쪽으로 밀려났다.
//!
//! 한글 정답지 `pdf/hwpx/hwpx-02-2022.pdf` 3쪽의 글리프 좌표를 재면 줄 진행이
//! `최대글자크기 × 줄간격퍼센트 / 100` 하나로 34개 문단 누적 **0.1px** 안에 설명된다.
//! 그래서 이런 문서에서는 저장 치수 대신 그 공식을 쓴다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 사다리가 단일 튜플인 문서 — 이 수정의 대상.
const FLAT: &str = "samples/hwpx/hwpx-02.hwpx";
/// 사다리가 여러 치수인 문서 — 저장 치수가 권위라 건드리면 안 된다(#1116).
const VARIED: &str = "samples/hwp3-sample16-hwp5.hwp";

fn export_text(sample: &str) -> Vec<String> {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("rhwp_issue_5854_{}_{}", std::process::id(), nth));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("출력 디렉토리 생성");
    let done = Command::new(rhwp_bin())
        .current_dir(repo_root())
        .args(["export-text", sample, "-o", dir.to_str().expect("경로")])
        .output()
        .expect("rhwp export-text 실행");
    assert!(
        done.status.success(),
        "export-text 실패: {}",
        String::from_utf8_lossy(&done.stderr)
    );
    let mut pages: Vec<(usize, String)> = std::fs::read_dir(&dir)
        .expect("출력 디렉토리")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .map(|path| {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            // 다중 쪽은 `<stem>_NNN.txt`, 단일 쪽은 접미사가 없다.
            let no = name
                .rsplit('_')
                .next()
                .and_then(|tail| tail.parse::<usize>().ok())
                .unwrap_or(1);
            let body: String = std::fs::read_to_string(&path)
                .expect("쪽 텍스트")
                .split_whitespace()
                .collect();
            (no, body)
        })
        .collect();
    pages.sort_by_key(|(no, _)| *no);
    pages.into_iter().map(|(_, body)| body).collect()
}

#[test]
fn flat_ladder_document_has_no_blank_page() {
    let pages = export_text(FLAT);
    let blank: Vec<usize> = pages
        .iter()
        .enumerate()
        .filter(|(_, body)| body.is_empty())
        .map(|(idx, _)| idx + 1)
        .collect();
    assert!(
        blank.is_empty(),
        "빈 쪽이 있다: {blank:?} (쪽별 글자 수 {:?}) — 합성 사다리의 줄 높이를 그대로 쓴 상태다",
        pages.iter().map(|b| b.chars().count()).collect::<Vec<_>>()
    );
}

#[test]
fn flat_ladder_document_matches_the_oracle_page_split() {
    // 정답지 `pdf/hwpx/hwpx-02-2022.pdf` 5쪽의 쪽별 글자 수. 6쪽(389자)은 정답지가
    // 담지 못한 둘째 구역이라 여기서 고정하지 않는다.
    const ORACLE: [usize; 5] = [529, 474, 217, 89, 128];
    let pages = export_text(FLAT);
    assert!(
        pages.len() >= ORACLE.len(),
        "쪽수가 정답지보다 적다: {}",
        pages.len()
    );
    let got: Vec<usize> = pages
        .iter()
        .take(ORACLE.len())
        .map(|b| b.chars().count())
        .collect();
    assert_eq!(
        got,
        ORACLE.to_vec(),
        "1~5쪽 글자 수가 정답지와 달라졌다 — 쪽 경계가 어긋난 것이다"
    );
}

#[test]
fn varied_ladder_document_keeps_its_stored_metrics() {
    // #1116 이 이 문서의 저장 lineseg spacing 을 권위로 고정한다. 판정이 넓어져
    // 여기까지 발동하면 그 계약이 깨진다 — 쪽수로 그 경계를 지킨다.
    let pages = export_text(VARIED);
    assert_eq!(
        pages.len(),
        64,
        "사다리가 여러 치수인 문서의 쪽수가 바뀌었다 — 판정이 과하게 넓다"
    );
}
