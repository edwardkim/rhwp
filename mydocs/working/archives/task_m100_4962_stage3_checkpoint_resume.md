# Task M100 #4962 W3 Stage 3-R — atomic checkpoint·resume

- **Issue**: [#4962](https://github.com/edwardkim/rhwp/issues/4962)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4962.md`](../../plans/archives/task_m100_4962.md)
- **선행 결과**: [`task_m100_4962_stage3_full_pilot_r2.md`](task_m100_4962_stage3_full_pilot_r2.md)
- **검증 source HEAD**: `832e0ff289fe7b5d6d2ff91da5b44d08da2fdbe5`
- **날짜**: 2026-08-21 KST
- **단계 상태**: checkpoint·resume hard gate 완료, private 10k 전건 미착수

## 1. 승인 범위와 결론

Stage 3의 32건 반복 pilot은 count·failure vector·combined decision hash가 두 번 정확히 일치했지만,
예상 17~36시간의 전건 실행을 process나 host 중단 뒤 재개할 방법이 없었다. 승인된 절편에서는 private
10k를 실행하지 않고 다음 복구 경계를 먼저 구현했다.

- 완료 문서마다 append-only NDJSON journal 한 레코드를 기록한다.
- journal을 `fsync`한 뒤에만 작은 `state.json`을 atomic rename으로 commit한다.
- state가 commit한 journal byte offset과 다음 document index만 복구 기준으로 인정한다.
- 실행 identity가 달라지거나 committed 영역이 손상되면 fail-closed한다.
- aggregate contract와 privacy 검사를 통과한 값만 journal에 기록한다.
- journal 총량 16 GiB와 append 뒤 최소 여유 공간 4 GiB를 강제한다.

공개 HWP 2건과 HWPX 1건의 실제 worker 실행을 2건 commit 뒤 강제 중단했다. 재개 결과는 별도
checkpoint에서 처음부터 수행한 무중단 결과와 문서 상태, count, category, join, backend, usage-row 합,
SHA-256 chain에서 정확히 일치했다. 실행시간과 peak RSS는 관찰값이므로 equality 대상에서 제외했다.

따라서 Stage 4의 **중단·재개 hard gate는 통과**했다. 그러나 journal의 문서별 usage row를 최종 corpus
aggregate로 병합하는 finalizer와 full manifest preflight는 아직 없으므로 10k 전건 실행은 시작하지
않았다.

## 2. crash-consistent commit 순서

```text
document worker 완료
  -> aggregate 계약·privacy·format 검증
  -> 다음 summary와 journal record를 메모리에서 계산
  -> journal 총량·남은 디스크 예산 검사
  -> journal.ndjson append + fsync
  -> state.json.next write + fsync
  -> state.json으로 atomic rename
  -> checkpoint directory fsync
```

이 순서에서 state가 가리키지 않는 journal tail은 commit되지 않은 것으로 본다.

| 중단 지점 | 재개 판정 |
| --- | --- |
| worker 완료 전 | 같은 index부터 다시 실행 |
| journal append 중 | state의 committed byte 뒤 tail을 truncate하고 다시 실행 |
| journal fsync 뒤, state commit 전 | 완전한 record여도 uncommitted tail로 truncate하고 다시 실행 |
| state atomic rename 뒤 | 다음 index부터 실행 |
| committed 영역 단축·변조 | 재개 거부 |
| 완료 state 재호출 | worker를 다시 부르지 않고 동일 state 반환 |

state는 journal의 committed byte 수, record 수와 다음 index를 함께 보존한다. 재개할 때 committed 영역을
처음부터 replay해 계산한 summary가 state와 byte-for-byte canonical equality를 이루지 않으면 진행하지
않는다.

## 3. 실행 identity와 drift 거부

checkpoint state와 hash-chain seed는 다음 identity를 고정한다.

| identity | 역할 |
| --- | --- |
| source HEAD | 실행 source 계보 |
| runner SHA-256 | dirty 또는 다른 checkpoint runner 차단 |
| worker SHA-256 | 다시 빌드된 다른 worker 차단 |
| coverage contract SHA-256 | 분류·분모 schema drift 차단 |
| manifest SHA-256·policy version·document count | 입력 집합·순서 drift 차단 |
| checkpoint policy SHA-256 | commit·storage 정책 drift 차단 |
| analysis options SHA-256 | 분석 옵션 drift 차단 |
| isolation limits SHA-256 | timeout·CPU·주소공간·stdout 한도 drift 차단 |

manifest는 `localOnly: true`인 private pilot/full-corpus kind만 허용하고 각 문서의 format과 64자리 BLAKE3를
요구한다. 같은 `format:BLAKE3`가 두 번 나오면 실행 전에 거부한다. journal record에는 source path,
filename과 BLAKE3를 넣지 않고 순서 index와 format, privacy 검사를 통과한 aggregate 또는 비식별 failure,
자원 관찰값만 둔다.

## 4. 누적 summary 의미

checkpoint summary는 복구와 진행 상황을 위한 가산 상태이며 최종 corpus aggregate가 아니다.

- `counts`는 layout·coverage·non-applicable·excluded·truncated 문자와 문단·source run의 가산 합이다.
- `legacyUsageRows`와 `decisionUsageRows`는 문서별 행 수의 합이므로 `usageRowSums`로 분리했다.
- category, join과 backend count는 가산한다.
- elapsed는 합, worker RSS는 최대 관찰값을 보존한다.
- chain은 identity seed에서 시작해 `format:status:aggregate-hash-or-failure`를 문서 순서대로 결합한다.

동일 usage key가 여러 문서에 나타날 수 있으므로 `usageRowSums`를 최종 병합 행 수로 주장하지 않는다.
최종 비식별 aggregate는 journal에 보존된 privacy-checked `legacyUsage`·`decisionUsage`를 key별로 다시
병합하고 coverage contract의 canonical hash를 계산해야 한다. 이것이 다음 절편의 finalizer 책임이다.

## 5. 저장공간 보호

worker 하나의 stdout은 기존 supervisor에서 최대 128 MiB로 제한되지만, 이를 10k에 단순 곱하면 누적
journal의 디스크 고갈을 막지 못한다. checkpoint policy에 다음 hard limit를 추가했다.

| 항목 | 값 |
| --- | ---: |
| journal 최대 크기 | 17,179,869,184 bytes, 16 GiB |
| append 뒤 filesystem 최소 여유 | 4,294,967,296 bytes, 4 GiB |
| 초과 처리 | record append 전 거부 |

현재 host의 관찰 여유 공간은 약 220.7 GB였지만 이는 identity나 완료 조건이 아니다. Stage 4-A에서는
full manifest 문서 수와 pilot/public aggregate 크기로 예상 journal 범위를 계산하고, 실제 실행 직전
filesystem 여유를 다시 preflight해야 한다.

## 6. 검증 결과

### 6.1 focused·회귀

다음 27건이 모두 통과했다.

- checkpoint focused 6건
  - 강제 중단 뒤 exact resume
  - source·worker·manifest·raw manifest·policy·contract·options·limits drift 거부
  - manifest 중복과 aggregate format mismatch 거부
  - journal storage budget 초과를 append 전 거부
  - uncommitted partial tail truncate
  - committed journal corruption 거부
- W3 coverage contract·privacy 10건
- deterministic pilot selector 4건
- Linux process isolation supervisor 7건

검증 명령은 다음과 같다.

```bash
node --test \
  scripts/tests/font_metric_coverage_contract.test.mjs \
  scripts/tests/font_metric_coverage_pilot_selector.test.mjs \
  scripts/tests/font_metric_coverage_supervisor.test.mjs \
  scripts/tests/font_metric_coverage_checkpoint_runner.test.mjs
```

결과는 `tests 27`, `pass 27`, `fail 0`이었다.

### 6.2 공개 실제 worker 강제 중단

검증 source HEAD에서 공개 문서 3건만 사용했다. 문서 식별자는 checkpoint와 이 보고서에 보존하지 않고
형식과 비식별 합계만 기록한다.

| 항목 | 결과 |
| --- | ---: |
| 공개 입력 | HWP 2 / HWPX 1 |
| 강제 중단 위치 | 2건 commit 뒤, 3번째 worker 시작 전 |
| 재개 attempted / success | 3 / 3 |
| layout / coverage 문자 | 1,546 / 1,546 |
| truncated / excluded | 0 / 0 |
| legacy / decision 문서별 row 합 | 26 / 45 |
| 안정 field 비교 | exact |
| 파일 권한 | state·journal 모두 `0600` |
| 문서 path·identity 잔존 검사 | 0 |

재개·무중단 공통 chain은 다음과 같다.

```text
51be1af0f278686c7ed9907a687650b4cf71dafd0d6d388776383b8d33f3f68f
```

관찰시간은 재개 실행 113 ms, 무중단 실행 98 ms였고 peak worker RSS는 각각 23,121,920 bytes와
23,322,624 bytes였다. 표본이 작고 process 관찰 오차가 지배하므로 checkpoint overhead benchmark로
사용하지 않는다. 공개 검증이 만든 두 checkpoint directory는 검증 직후 제거했다.

## 7. 변경 경계

| 파일 | 책임 |
| --- | --- |
| `font_metric_coverage_checkpoint_policy.json` | identity·journal·atomic state·resume·storage·privacy 정책 |
| `font_metric_coverage_checkpoint_runner.mjs` | append/fsync/atomic commit, replay, drift·corruption·storage gate |
| `font_metric_coverage_checkpoint_runner.test.mjs` | 강제 중단·drift·tail·corruption·storage focused 검증 |

구현 경계는 로컬 commit `24f97b42a`, 계보 보강은 `9bf34e8b5`, 저장공간 보강은 `832e0ff28`에 고정했다.
Rust 제품 source, metric DB, fallback, paint, font asset과 private corpus는 변경하지 않았다. 원격 push와
Issue·PR 변경도 수행하지 않았다.

## 8. 종료 판정과 다음 승인 후보

Stage 3-P2가 요구한 atomic checkpoint·resume hard gate는 충족됐다.

- 문서 완료 단위 crash-consistent commit
- committed byte 경계 기반 재개
- 실행 source·runner·worker·contract·manifest·policy·options·limits drift 거부
- 누적 count·failure·chain replay와 state exact 대사
- private identity 비저장과 로컬 권한 고정
- 공개 실제 worker의 강제 중단·재개 exact 검증
- journal 총량과 filesystem reserve의 fail-closed 제한

다음 승인 후보는 **Stage 4-A — journal finalizer와 local-only full manifest·저장공간 preflight**다.

1. journal의 문서 aggregate를 usage key별로 병합해 최종 coverage contract와 canonical hash를 만든다.
2. 공개 fixture에서 무중단 final aggregate와 checkpoint replay final aggregate가 exact인지 검증한다.
3. 기존 10k 입력을 읽기 전용으로 local-only manifest에 고정하고 format·regular file·size·BLAKE3를
   preflight한다.
4. 예상 journal 크기와 실행 직전 filesystem reserve를 보고한다.

이 절편의 결과와 별도 승인이 있기 전에는 10k decision worker 전건 실행, 반복 pass, 원격 push와 PR을
수행하지 않는다.
