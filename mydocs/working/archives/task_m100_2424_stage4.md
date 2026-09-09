# Task M100 #2424 Stage D 완료보고서 — pending job, WASM step API, Studio scheduler

## 1. 목적

Stage C의 caller-controlled continuation context를 실제 브라우저 task 사이에 보존한다. 거대 표 셀 입력이
flow 경계를 넘었을 때 약 1.1초의 전체 조판을 한 번에 실행하지 않고, 공개 pagination을 유지한 채
fragment 단위 shadow job을 전진시킨 뒤 마지막 fragment에서만 결과를 원자 commit한다.

## 2. 코어 수명과 commit 계약

`DocumentCore`의 `PendingPaginationJob`은 다음 값을 소유한다.

- 시작 edit의 `DeferredPaginationDescriptor`와 revision
- borrow를 갖지 않는 `ResumableTablePaginationJob`
- 해당 revision에서 다시 측정한 `MeasuredSection`

begin은 최신 descriptor가 `Current`이고 `cellFlowChanged`가 누적된 경우에만 실행한다. 첫 fast path는
한 구역·한 문단·한 개의 마지막 non-TAC RowBreak 표로 제한한다. 지원하지 않는 형상은 `Fallback`을
반환해 기존 full pagination barrier로 복구한다.

각 step은 descriptor와 table structure fingerprint를 다시 검증하고 paragraph/table/measured table/style을
좌표로 다시 빌린다. 완료 전에는 기존 `pagination`, page tree, layer JSON cache를 교체하지 않는다.
마지막 step에서만 다음을 한 번에 수행한다.

1. shadow `TypesetState`를 `PaginationResult`로 마감
2. page-number layout과 master-page 후처리
3. pagination·measurement cache 교체와 dirty flag 정리
4. 최신 descriptor 소비와 render cache 전체 무효화

새 deferred edit가 들어오면 이전 job은 `Stale`로 폐기된다. 저장·다른 이름으로 저장·인쇄·명시 flush는
pending job을 무제한 budget으로 동기 drain하고, 시작 불가/stale/error이면 기존 `paginate()`로 fallback한다.

## 3. WASM과 Studio

WASM 공개 API에 다음 상태 JSON 경로를 추가했다.

- `beginDeferredPagination(fragmentBudget)`
- `stepDeferredPagination(fragmentBudget)`
- `cancelDeferredPagination()`
- `flushDeferredPagination()` — 동기 barrier

상태는 `none | pending | complete | fallback | stale`이며 revision, 처리 fragment 수, 현재 page count를
함께 반환한다.

Studio의 `DeferredPaginationRunner`는 budget 1로 한 macrotask에 step 하나만 실행한다. 새 입력은 예약
task와 이전 core job을 취소한 뒤 최신 revision을 시작한다. complete callback에서만 전체 document change를
발행하고 캐럿을 최신 pagination으로 재조회한다. 구버전 WASM이나 fast-path 비대상 문서는 기존 동기
flush 의미론을 유지한다. 기존 10초 idle flush와 30쪽 자동-flush 제한도 보존한다.

## 4. exactness와 수명 검증

HWP/HWPX #2214 fixture의 56번째 입력 경계에서 budget 1로 실제 begin/step API를 실행했다.

| 항목 | HWP | HWPX |
|---|---:|---:|
| step 호출 | 115 | 115 |
| 처리 fragment | 115 | 115 |
| commit page | 115 | 115 |
| transient 대비 변경 cut | 113 | 113 |

114번째 pending step까지 공개 cut chain은 transient pagination과 byte-for-byte 같은 구조를 유지했고,
115번째 complete에서만 새 chain이 게시됐다. 모든 adjacent fragment는
`end_cut == next.start_cut` 또는 row-boundary 연속성을 통과했다.

추가 검증은 첫 job을 한 step 전진한 뒤 새 입력으로 revision을 교체했다. 이전 step은 `stale`, 최신 begin은
더 큰 revision을 반환했으며, cancel 뒤 동기 flush가 최신 descriptor를 다시 시작해 115쪽 결과를 완성했다.

## 5. release 계측

`issue_2424_profile_resumable_fragment_steps`를 `release-test`로 실행한 1회 결과다.

| 형식 | begin | step 합계 | p50 | p95 | max |
|---|---:|---:|---:|---:|---:|
| HWP | 31.472ms | 1171.937ms | 10.156ms | 10.733ms | 21.662ms |
| HWPX | 32.482ms | 1180.219ms | 10.178ms | 11.002ms | 22.292ms |

총 계산량은 기존 약 1.06~1.11초와 같은 규모지만, steady-state step의 p95가 약 11ms라 브라우저가 입력,
paint, 다른 task를 continuation 사이에 처리할 수 있다. begin의 약 32ms는 정규화·선택 측정·continuation
prelude를 포함하며, 후속 범위 확장 단계에서 별도 chunk 후보로 남긴다.

## 6. 검증 명령

| 명령 | 결과 |
|---|---|
| `cargo test --profile release-test --lib issue2424_resumable_pagination_commits_only_after_final_fragment -- --nocapture` | HWP/HWPX 통과 |
| `cargo test --profile release-test --lib issue2424_new_edit_stales_old_job_and_sync_flush_restarts_latest_revision -- --nocapture` | 통과 |
| `cargo test --profile release-test --lib issue2214_scoped_cache_coherence_preserves_transient_pagination -- --nocapture` | HWP/HWPX exact 통과 |
| resumable release probe | HWP/HWPX 115 steps 통과 |
| `node --test tests/deferred-pagination-runner.test.ts` | 3 passed |
| `node --test tests/input-edit-invalidation.test.ts` | 10 passed |
| `cargo check --lib --target wasm32-unknown-unknown` | 통과 |

## 7. 후속 완료

전체 #2214/#2424 matrix, Studio type/build, save/save-as/print barrier 계약과 실제 HWP/HWPX 브라우저
E2E는 Stage 5에서 통과했다. 최종 판정과 최신 upstream 재검증은
`mydocs/working/task_m100_2424_stage5.md`와 `mydocs/report/task_m100_2424_report.md`에 기록한다.
