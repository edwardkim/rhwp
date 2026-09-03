# Task M100 #4962 W3 Stage 3-I — 문서별 계측 process 격리

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../plans/task_m100_4962.md)
- **선행 보안 기준선**: [`task_m100_4962_stage2_security.md`](task_m100_4962_stage2_security.md)
- **기능 기준선**: `7613a1b8e`
- **날짜**: 2026-08-21 KST
- **단계 상태**: Stage 3의 첫 hard gate 완료, private pilot 미착수

## 1. 승인 범위와 결과

이번 절편은 Stage 2-S가 요구한 corpus worker 격리만 구현했다. private corpus는 열거나 worker에
전달하지 않았다. 새 구현은 문서 하나마다 별도 process를 만들고, 해당 process가 hang·OOM·signal·출력
폭주·민감 값 유출을 일으켜도 실패한 문서만 비식별 상태로 닫은 뒤 다음 문서를 계속 처리한다.

격리 hard gate는 통과했다. 다만 이것은 Stage 3 전체 완료가 아니다. 7개 coverage 분류의 공개 fixture,
private 소규모 pilot, 반복 hash와 처리량·전수 예상 시간 측정은 아직 수행하지 않았다.

## 2. 구현 경계

### 2.1 developer-only Rust worker

`tools/diagnostics/font_metric_coverage_worker.rs`는 제품 `rhwp` CLI·WASM·npm API에 새 surface를 추가하지 않는
개발자용 example이다. worker는 다음 순서만 수행한다.

1. 입력을 최대 257 MiB + 1 byte까지 bounded read한다.
2. empty·DRM·unknown format을 parser 진입 전에 분리한다.
3. `DocumentCore::from_bytes()`로 열고 Stage 2 collector를 실행한다.
4. 성공 시 기존 aggregate JSON 하나만 stdout에 쓴다.
5. 실패 시 허용된 상태 하나만 가진 `font-metric-coverage-worker-result`를 쓰며 경로·오류 문자열·stack은
   출력하지 않는다.

257 MiB는 worker가 입력 파일 자체를 무제한 materialize하지 않기 위한 방어선이다. 이를 넘는 문서는
잘린 성공이 아니라 `resource-limit` 문서 전체 실패다. 향후 실제 정상 대형 문서 근거로 값을 바꿀 때도
문서 내용이 아니라 supervisor 정책만 바꿀 수 있다.

### 2.2 Linux supervisor

`scripts/font_metric_coverage_supervisor.mjs`는 `/usr/bin/prlimit`가 없는 환경에서 fail closed한다.
각 문서는 다음 process group으로 실행된다.

```text
parent Node supervisor
  -> prlimit --as=<soft:hard> --cpu=<soft:hard>
       -> one Rust worker
            -> one document
```

kernel은 address-space와 CPU hard limit을 적용한다. 부모는 wall-clock timeout과 stdout byte budget을
적용하며, 초과 시 worker PID 하나가 아니라 독립 process group 전체에 `SIGKILL`을 보낸다. Linux의
`RLIMIT_RSS`는 실질적인 hard memory 제한으로 신뢰할 수 없으므로 사용하지 않았다. 보고되는
`peakRssBytes`는 `/proc/<pid>/status`의 10 ms 간격 관찰치이고, 보안 보장은 `RLIMIT_AS`가 담당한다.

기본값은 다음과 같다.

| 경계 | 기본값 | caller 허용 최대 | 강제 주체 |
| --- | ---: | ---: | --- |
| wall timeout | 90초 | 3,600초 | parent process |
| CPU time | 75초 | 3,600초 | kernel `RLIMIT_CPU` |
| address space | 2 GiB | 8 GiB | kernel `RLIMIT_AS` |
| worker stdout | 128 MiB | 128 MiB | parent process |
| RSS 관찰 주기 | 10 ms | 1,000 ms | parent 관찰 전용 |

worker 환경에는 locale과 `RUST_BACKTRACE=0`만 전달한다. stderr는 폐기하고 stdout은 한도 안에서 JSON
하나만 허용한다. spawn 자체 실패는 document parser 실패로 가장하지 않고 supervisor 설정 오류로
중단한다.

### 2.3 비식별 failure와 batch 복구

supervisor가 외부로 반환하는 문서 실패 봉투에는 다음 값만 있다.

```json
{
  "schemaVersion": 1,
  "kind": "font-metric-coverage-document-result",
  "status": "failed",
  "failure": "resource-limit",
  "metrics": {
    "elapsedMillis": 50,
    "peakRssBytes": 0
  }
}
```

timeout, signal/OOM, stdout 초과, unexpected exit, malformed JSON과 privacy 검사 실패는 모두
`resource-limit`으로 닫는다. 정상 worker가 명시적으로 보낸 `cancelled`·`drm`·`empty`·`encrypted`·
`parser`·`resource-limit`·`unsupported`만 해당 상태로 보존한다. signal 번호, 원시 오류, stderr와 입력
경로는 반환하지 않는다.

batch API는 문서별 결과 배열을 남기지 않는다. 성공 aggregate는 optional callback에 일시적으로
전달하고, 반환 summary에는 전체 성공·실패 count와 최대 관찰 RSS·누적 시간만 남긴다. 실패한 worker
뒤에도 새 process에서 다음 문서를 계속 실행한다.

## 3. machine-readable 보호 계약

`font_metric_coverage_contract.json`의 `resourcePolicy`에 다음 hard gate를 추가하고 validator와 golden
test가 누락을 거부하도록 했다.

- 문서별 별도 process
- 부모 wall timeout
- OS address-space hard limit
- process-group termination
- supervisor stdout byte budget
- 비식별 failure envelope
- worker failure 후 계속 실행

이 계약은 collector 내부의 work·deadline·depth·row·output 예산을 대체하지 않는다. 내부 cooperative
budget이 1차 방어이고 process 격리가 allocator abort·native hang·peak memory를 닫는 2차 방어다.

## 4. 공개 검증

### 4.1 공격형 supervisor test

```bash
node --test scripts/tests/font_metric_coverage_supervisor.test.mjs \
  scripts/tests/font_metric_coverage_contract.test.mjs
```

결과: **17 passed, 0 failed**.

- 실제 `prlimit` process 안에서 finite address-space·CPU limit 확인
- 64 MiB address-space로 Node worker 시작 실패를 유도하고 `resource-limit` 확인
- hang worker wall timeout과 process-group 강제 종료 확인
- worker가 띄운 자식 process도 timeout 뒤 marker를 만들지 못함을 확인
- stdout overflow, unexpected exit와 `/home/...` 민감 path payload를 fail closed
- 알려진 parser 실패는 원시 진단 없이 `parser`로 변환
- 첫 문서 timeout 뒤 두 번째 문서 성공, summary에 두 입력 path가 없음을 확인

### 4.2 실제 Rust worker와 공개 문서

```bash
cargo build --example font_metric_coverage_worker
node scripts/font_metric_coverage_supervisor.mjs \
  --worker target/debug/examples/font_metric_coverage_worker \
  --input <public-fixture>
```

| 공개 fixture | 형식 | 결과 | 문자 | truncation | 관찰 시간 | 관찰 peak RSS |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `samples/eq-01.hwp` | HWP | complete | 648 | 0 | 33 ms | 19,222,528 bytes |
| `samples/pic2.hwpx` | HWPX | complete | 794 | 0 | 32 ms | 22,716,416 bytes |

`/dev/null`과 공개 HWP를 같은 batch에 순서대로 넣은 복구 검증은 attempted 2, success 1, empty 1로
대사됐고 두 번째 문서를 정상 완료했다. 반환 summary에는 입력 path가 없었다. 이 수치는 공개 소형
fixture의 격리 동작 증거일 뿐 private corpus 처리량 추정치로 사용하지 않는다.

추가 검증:

- `cargo check --example font_metric_coverage_worker`: 통과
- `cargo clippy --example font_metric_coverage_worker -- -D warnings`: 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과

## 5. 남은 경계와 다음 승인

격리 supervisor가 성립했으므로 Stage 3의 다음 후보는 먼저 **공개 fixture만으로 7개 coverage 분류와
non-applicable 경계를 고정하는 절편**이다. 이 절편에서도 private corpus를 사용할 이유가 없다.

그 다음에야 기존 local risk ranking과 format·usage aggregate로 재수집 없는 deterministic pilot cohort를
선정하고, 선정 방법과 규모를 보고해 별도 승인을 받아 private pilot을 실행한다. 따라서 현재 시점에는
private corpus 계측, 10k 전수 실행, 원격 push와 PR을 수행하지 않는다.
