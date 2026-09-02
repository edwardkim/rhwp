//! Bounded ordered NDJSON stream runtime shared by batch queries and commands.

use std::io::Write as _;

use crate::cli::commands::batch_convert::verdict_differs;
use crate::{EXIT_OK, EXIT_RUNTIME};

pub(crate) struct BatchStreamTally {
    pub(crate) emitted: usize,
    pub(crate) failed: usize,
    pub(crate) verify_diff: usize,
    pub(crate) verify_pages_diff: usize,
    /// stdout 소비자가 끊겨(broken pipe 등) 스트림을 끝까지 내지 못했다.
    pub(crate) aborted: bool,
}

impl BatchStreamTally {
    /// [#3626] 종료 코드 집계. 하드 실패(산출물이 아예 없음)가 가장 나쁘므로 기존 규약대로
    /// 1 이 우선한다. 그 아래는 단건 convert 의 우선순위를 그대로 따른다 — 단건도 쪽수
    /// 검사를 IR 검사보다 먼저 해 exit 4 로 끊는다. 검증 판정을 1 로 접지 않는 이유는
    /// 소비자가 재실행 대상(1)과 검토 대상(3/4)을 갈라야 하기 때문이다.
    pub(crate) fn exit_code(&self) -> i32 {
        if self.failed > 0 {
            EXIT_RUNTIME
        } else if self.verify_pages_diff > 0 {
            4
        } else if self.verify_diff > 0 {
            3
        } else {
            EXIT_OK
        }
    }
}

/// [#3238→#3719] 작업 간 병렬 처리 + 한계 재정렬 버퍼(bounded reorder buffer) 스트리밍.
///
/// 배리어 없이 완전 병렬로 돌리되, 완료 레코드는 **입력 순서대로** 즉시 방출한다.
/// 완료-미방출 레코드가 cap 을 넘으면 워커가 대기(역압)해 메모리를 상한한다.
/// 단, 방출 차례(next_emit) 레코드는 cap 과 무관하게 넣을 수 있어야 교착이 없다 —
/// 느린 작업 하나가 버퍼를 채워도, 그 작업이 곧 방출 차례이므로 항상 전진한다.
///
/// [#3719] `run_batch`(stdin 경로 목록)와 `run_batch_fill`(데이터 행)이 이 하나를 쓴다.
/// 작업 단위가 무엇인지는 `make` 가 정하고, 순서 보존·역압·종료 코드 집계 규약은 공유한다.
pub(crate) fn stream_records<F>(
    n: usize,
    threads: usize,
    make: F,
    out: &mut impl std::io::Write,
) -> BatchStreamTally
where
    F: Fn(usize) -> serde_json::Value + Sync,
{
    let cap = threads.saturating_mul(8).max(1);
    let next_claim = std::sync::atomic::AtomicUsize::new(0);
    let abort = std::sync::atomic::AtomicBool::new(false);
    let buf: std::sync::Mutex<std::collections::HashMap<usize, serde_json::Value>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
    let next_emit = std::sync::atomic::AtomicUsize::new(0);
    let space = std::sync::Condvar::new(); // 버퍼에 자리가 났다
    let ready = std::sync::Condvar::new(); // 방출 차례 레코드가 도착했다

    let (failed, emitted, verify_diff, verify_pages_diff) = std::thread::scope(|scope| {
        for _ in 0..threads.min(n) {
            scope.spawn(|| loop {
                if abort.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let idx = next_claim.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if idx >= n {
                    break;
                }
                let record = make(idx);
                let mut guard = buf.lock().expect("batch buf lock");
                while guard.len() >= cap
                    && idx != next_emit.load(std::sync::atomic::Ordering::Relaxed)
                    && !abort.load(std::sync::atomic::Ordering::Relaxed)
                {
                    guard = space.wait(guard).expect("batch buf lock");
                }
                if abort.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                guard.insert(idx, record);
                // 방출자는 하나뿐이므로 notify_one 으로 충분하다.
                ready.notify_one();
            });
        }

        // 방출자(현재 스레드): 입력 순서대로 도착 즉시 방출한다. 도착해 있는 연속
        // 레코드는 한 번의 락으로 일괄 드레인하고 notify 도 배치당 1회만 보낸다 —
        // 레코드당 notify_all 은 대기 워커 전원을 헛깨우는 thundering herd 가 된다
        // (271건 실측에서 방출 버스트 구간 수 초 손실).
        let mut failed = 0usize;
        let mut emitted = 0usize;
        // [#3626] 검증 판정은 실패가 아니다 — 변환·저장은 성공했고 산출물도 있다.
        // 실패 계수와 섞으면 소비자가 "읽을 수 없었다"와 "변환은 됐는데 IR 이 다르다"를
        // 종료 코드로 구분할 수 없다.
        let mut verify_diff = 0usize;
        let mut verify_pages_diff = 0usize;
        let mut drained: Vec<serde_json::Value> = Vec::new();
        'emit: while emitted < n {
            drained.clear();
            {
                let mut guard = buf.lock().expect("batch buf lock");
                while guard.get(&emitted).is_none() {
                    guard = ready.wait(guard).expect("batch buf lock");
                }
                while let Some(record) = guard.remove(&emitted) {
                    emitted += 1;
                    drained.push(record);
                }
                next_emit.store(emitted, std::sync::atomic::Ordering::Relaxed);
            }
            space.notify_all();
            for record in &drained {
                if record.get("error").is_some() {
                    failed += 1;
                } else if verdict_differs(record, "verifyPages") {
                    verify_pages_diff += 1;
                } else if verdict_differs(record, "verify") {
                    verify_diff += 1;
                }
                if let Err(e) = writeln!(out, "{record}") {
                    // 파이프 소비자가 끊은 경우(broken pipe 등): 새 작업 수주를 멈추고
                    // 대기 중인 워커를 전부 깨워 정리한다.
                    eprintln!("오류: stdout 쓰기 실패 - {}", e);
                    abort.store(true, std::sync::atomic::Ordering::Relaxed);
                    space.notify_all();
                    break 'emit;
                }
            }
        }
        (failed, emitted, verify_diff, verify_pages_diff)
    });

    BatchStreamTally {
        emitted,
        failed,
        verify_diff,
        verify_pages_diff,
        aborted: abort.load(std::sync::atomic::Ordering::Relaxed),
    }
}
