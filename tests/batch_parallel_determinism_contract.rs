//! 배치 병렬 처리량 축 — `--threads` 축에 대한 **결정론·실패 격리** 계약 회귀 테스트.
//!
//! `batch` 는 이미 병렬이다: 경계 있는 워커 풀 + 입력 순서 재정렬 버퍼(cap = `threads*8`) +
//! 역압 + 파일별 `catch_unwind` 실패 격리 (`batch_stream_records`). 이 파일은 그 병렬성이
//! 스레드 수와 무관하게 지켜야 할 관측 가능한 계약을 회귀로 고정한다.
//!
//! - 결정론: 같은 입력이면 `--threads` 값과 무관하게 stdout 이 바이트 단위로 동일하다 — 워커가 순서 밖으로 끝나도 방출은 입력 순서(저장소 철학 = 결정론).
//! - 실패 격리: 읽을 수 없는 파일은 그 입력 위치의 실패 레코드가 되고 배치를 중단시키지 않는다 — 병렬 실행에서도 입력 N = 성공 + 실패, 부분 실패 exit 1.
//! - 퇴화 입력: 빈 목록·단건·전건 실패에서 병렬 경로가 패닉·교착 없이 계약대로 끝난다.
//!
//! 기존 `batch_axes_contract.rs` 는 기본 스레드 수·입력 3건으로 순서를 보긴 하지만,
//! `--threads` 를 고정해 서로 다른 스레드 수의 출력이 **동일한지**는 검증하지 않았고
//! 재정렬 버퍼 용량을 넘겨 역압 경로를 태우지도 않았다. 이 파일이 그 공백을 메운다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// info 축은 가장 싸고 모든 HWP/HWPX 에서 견고해 병렬 계약 픽스처에 적합하다.
/// 여섯 개의 **서로 다른** 정상 문서 — 레코드가 줄마다 달라 재정렬이 관측 가능하다.
const GOOD: [&str; 6] = [
    "samples/hwp3-sample.hwp",
    "samples/table-001.hwp",
    "samples/field-01.hwp",
    "samples/test-image.hwpx",
    "samples/추진일정.hwpx",
    "samples/table-complex.hwp",
];

/// 저장소 안에 존재하지 않는 경로 — 읽기 실패로 실패 레코드가 되어야 한다.
const MISSING: &str = "samples/__batch_parallel_no_such_file__.hwp";

/// 실패를 끼워 넣는 입력 위치(0-based). 재정렬 버퍼 안팎에 골고루 둔다.
const BAD_POSITIONS: [usize; 3] = [7, 18, 27];
/// 총 입력 줄 수. `--threads 3` 의 cap(=24)을 넘겨 역압 경로를 태운다.
const LINES: usize = 30;

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// 두 계약 테스트가 공유하는 입력. 정상 문서를 돌려 쓰되 지정 위치에 실패를 끼운다.
fn build_input() -> String {
    let mut lines = Vec::with_capacity(LINES);
    let mut g = 0usize;
    for i in 0..LINES {
        if BAD_POSITIONS.contains(&i) {
            lines.push(manifest(MISSING).to_string_lossy().into_owned());
        } else {
            lines.push(
                manifest(GOOD[g % GOOD.len()])
                    .to_string_lossy()
                    .into_owned(),
            );
            g += 1;
        }
    }
    let mut body = lines.join("\n");
    body.push('\n');
    body
}

/// stdin 을 자식이 읽기 전에 종료하는 경로의 BrokenPipe 는 정상이므로 무시한다
/// (`batch_axes_contract.rs` 와 같은 규약).
fn write_stdin_ignoring_early_exit(child: &mut std::process::Child, body: &str) {
    use std::io::ErrorKind;
    if let Err(err) = child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(body.as_bytes())
    {
        assert_eq!(
            err.kind(),
            ErrorKind::BrokenPipe,
            "stdin 쓰기 실패: {err:?}"
        );
    }
}

fn run(threads: &str, body: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(["batch", "info", "--json", "--threads", threads])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rhwp 실행 실패");
    write_stdin_ignoring_early_exit(&mut child, body);
    child.wait_with_output().expect("rhwp 종료 대기 실패")
}

fn records(out: &Output) -> Vec<serde_json::Value> {
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("NDJSON 아님 ({e}): {l}")))
        .collect()
}

/// 계약 1: `--threads` 값이 달라도 stdout 이 바이트 단위로 동일하다(결정론).
/// 직렬(=1)을 기준으로 역압 경로(3)와 완전 병렬(8)을 비교한다.
#[test]
fn batch_output_is_byte_identical_across_thread_counts() {
    let body = build_input();
    let serial = run("1", &body);
    assert_eq!(
        serial.status.code(),
        Some(1),
        "부분 실패이므로 exit 1 이어야 한다\nstderr:\n{}",
        String::from_utf8_lossy(&serial.stderr)
    );
    // 1 은 정의상 입력 순서. 3 은 cap(24)<30 이라 역압, 8 은 완전 병렬.
    for threads in ["3", "8"] {
        let parallel = run(threads, &body);
        assert_eq!(
            parallel.stdout, serial.stdout,
            "--threads {threads} 의 stdout 이 --threads 1 과 바이트 단위로 다르다 (결정론 위반)"
        );
        assert_eq!(
            parallel.status.code(),
            serial.status.code(),
            "--threads {threads} 의 종료 코드가 --threads 1 과 다르다"
        );
    }
}

/// 계약 2: 병렬 실행에서도 실패는 **그 입력 위치에만** 나타나고 배치를 중단시키지 않는다.
/// 레코드가 입력 순서로 나오므로 위치별 성공/실패가 그대로 관측된다.
#[test]
fn batch_failure_isolation_holds_under_parallel() {
    let body = build_input();
    let out = run("4", &body);
    assert_eq!(
        out.status.code(),
        Some(1),
        "부분 실패 → exit 1\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let recs = records(&out);
    // 입력 N = 성공 + 실패: 누락 없이 전건이 레코드가 된다.
    assert_eq!(recs.len(), LINES, "레코드 수가 입력 줄 수와 다르다");
    for (i, rec) in recs.iter().enumerate() {
        let is_err = rec.get("error").is_some();
        assert_eq!(
            is_err,
            BAD_POSITIONS.contains(&i),
            "위치 {i} 의 실패 여부가 계약과 다르다: {rec}"
        );
        if is_err {
            assert_eq!(rec["exitClass"], "runtime", "{rec}");
        }
    }
    let failed = recs.iter().filter(|r| r.get("error").is_some()).count();
    assert_eq!(failed, BAD_POSITIONS.len(), "실패 수가 주입 수와 다르다");
}

/// 계약 3: 병렬 경로가 빈 목록·단건·전건 실패에서 패닉·교착 없이 계약대로 끝난다.
#[test]
fn batch_parallel_handles_degenerate_inputs() {
    // 빈 목록 → 레코드 0, exit 0.
    let empty = run("8", "");
    assert_eq!(empty.status.code(), Some(0), "빈 목록은 exit 0");
    assert!(records(&empty).is_empty(), "빈 목록은 레코드 0");

    // 단건 성공 → 레코드 1, 실패 없음, exit 0.
    let one = run("8", &format!("{}\n", manifest(GOOD[0]).to_string_lossy()));
    assert_eq!(one.status.code(), Some(0), "단건 성공은 exit 0");
    let recs = records(&one);
    assert_eq!(recs.len(), 1, "단건은 레코드 1");
    assert!(
        recs[0].get("error").is_none(),
        "정상 문서인데 실패: {}",
        recs[0]
    );

    // 전건 실패 → 레코드 3 전부 실패, exit 1 (격리가 스트림을 끝까지 유지).
    let all_bad = format!("{m}\n{m}\n{m}\n", m = manifest(MISSING).to_string_lossy());
    let bad = run("8", &all_bad);
    assert_eq!(bad.status.code(), Some(1), "전건 실패는 exit 1");
    let recs = records(&bad);
    assert_eq!(recs.len(), 3, "전건 실패도 전건이 레코드");
    assert!(
        recs.iter().all(|r| r.get("error").is_some()),
        "전건이 실패 레코드여야 한다"
    );
}
