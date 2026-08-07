# Task M100 #4031 Stage 1 완료보고서 — 거대 셀 Enter 지연 기준선과 red contract

## 1. 목적

이슈 #4031(pending pagination 중 Enter의 중복 full pagination 제거) Stage 1: 최신
`upstream/devel@9f564bbe`에서 115쪽 거대 셀 문서의 cold Enter와 pending Enter 전 구간을
분해 계측하고, "pending cell Enter = full flush 1 + full pagination 1" 중복 계약을 수치로
고정한다. flow 보드 Task #8(거대 셀 Enter 지연 해소)의 1단계 실측이기도 하다.

## 2. 환경과 방법

- macOS(Darwin 25.1.0), arm64, Rust release-test profile
- fixture: `samples/issue1949_giant_cell_nested_tables_perf.hwp/.hwpx` (115쪽, 2,507문단 거대 셀)
- probe: `tests/issue_4031_enter_latency_probe.rs` (신규, `#[ignore]` local-only)
  - cold: 새 load 뒤 셀 상단(문단 5)·하단(끝-8) 각 3연타 `split_paragraph_in_cell_native`
  - pending: 56회 deferred insert 잔여 상태에서
    시나리오 A = `flush_deferred_pagination()` → split (현행 studio before-navigation 경로),
    시나리오 B = flush 생략(`cancel_deferred_pagination`) → split → 사후 flush 비용 확인
- `RHWP_2424_PROFILE=1`로 pagination subphase·block-table·continuation cursor timer 동시 기록
- `RHWP_4031_REPEATS=3`, timing assertion 없음. page count 정합만 단언

명령:

```bash
CARGO_TARGET_DIR=... cargo test --profile release-test --test issue_4031_enter_latency_probe --no-run
RHWP_2424_PROFILE=1 RHWP_4031_REPEATS=3 cargo test --profile release-test \
  --test issue_4031_enter_latency_probe -- --ignored --nocapture --test-threads=1
```

측정 중 dev 서버 등 배경 부하로 run 2부터 절대값이 약 3배까지 드리프트했다(run 1 첫
값 1130ms는 #2424 Stage A 기준선 1058ms와 정합). 따라서 아래에서 절대값은 run 1을,
구조 결론(패스 수·fragment 수·비율)은 36개 전 표본을 근거로 삼는다.

## 3. cold Enter — 키 1회가 전체 표 재조판을 소유한다

run 1 기준(ms):

| 형식 | 상단 3연타 | 하단 3연타 |
|---|---|---|
| HWP | 1130 / 1124 / 1173 | 1192 / 1530 / 2230 |
| HWPX | 2238 / 2394 / 2416 | 3133 / 3264 / 2871 |

36개 전 표본에서 예외 없이:

1. split 1회 = `paginate_pass` 정확히 1회. `RHWP_2424_PAGINATION_PROFILE`에서
   typeset_ms가 total의 98.5~99.2% (measure 15~44ms, invalidate/normalize/postprocess < 1ms).
2. `RHWP_2424_CONTINUATION_CURSOR fragments=115~116`: 편집 위치가 셀 상단이든 하단이든
   block-table continuation이 **115~116개 fragment 전부를 매번 처음부터 drain**한다.
   상한·하한 어디를 편집해도 비용이 같다 — fragment 증분 재개 부재(보드 Task #8 이론 3)의
   직접 증거다.
3. 셀 편집은 `para_offset = 0`이라 기존 CONVERGENCE(수렴 재사용) 기계가 아예 시도되지
   않는다. 본문 문단 offset 기반이므로 셀 내부 편집에는 적용 불가.

## 4. pending Enter — flush 1 + pagination 1 중복 (red contract)

3회 × HWP/HWPX(ms, 같은 배경 부하 아래 상대 비교):

| run | 형식 | A: flush | A: split | A 합계 | B: skip-flush split | B: 사후 flush |
|---|---|---:|---:|---:|---:|---:|
| 1 | HWP | 3423 | 3365 | 6788 | 3327 | 0.119 |
| 1 | HWPX | 4467 | 4126 | 8593 | 3514 | 0.088 |
| 2 | HWP | 3576 | 3832 | 7409 | 3830 | 0.116 |
| 2 | HWPX | 3710 | 3502 | 7212 | 3477 | 0.160 |
| 3 | HWP | 3531 | 3650 | 7180 | 3838 | 0.123 |
| 3 | HWPX | 3858 | 3796 | 7654 | 3453 | 0.106 |

1. 현행 경로(A)는 **full pagination 2회**로 단일 split의 약 2배다. flush가 계산한 분할 전
   pagination은 split 직후 통째로 폐기된다 — #4031 배경 그대로.
2. flush를 생략한 B는 split의 `paginate_if_needed()` 1회로 수렴하며, 최종 page count가
   A와 **완전 일치**했다(전 회 단언 통과).
3. B의 사후 `flush_deferred_pagination()`은 0.088~0.160ms의 사실상 no-op이다
   (`status=fallback`, dirty 구역 없음). 즉 split의 동기 pagination이 deferred descriptor를
   소비한 뒤에는 잔여 barrier 비용이 없다 — Stage 2 cancellation 계약의 native 불변식이
   이미 성립한다. `cancel_deferred_pagination()`/`cancelDeferredPagination`도 이미 노출돼 있다.

## 5. 코드 경로 확정 (Stage 2 입력)

- direct Enter: `input-handler-keyboard.ts` keydown 상단
  `PAGINATION_BOUNDARY_KEYS.has('Enter')` → `flushDeferredPaginationIfNeeded('before-navigation')`
  → 같은 함수 하단 `case 'Enter'` → (셀이면) `SplitParagraphInCellCommand` →
  `splitParagraphInCell*` → native `paginate_if_needed()` 재차 full pagination.
- flush 지점(547행)과 `case 'Enter'`(1215행)는 **같은 keydown 함수 스코프**다. admission
  판정을 지역 변수로 전달할 수 있어 인스턴스 상태 누수 없이 구현 가능하다.
- `SplitParagraphInCellCommand`는 `consumeTextMutationEffects`를 구현하지 않아 실행 후
  `deferredPaginationPending`이 자동 해소되지 않는다 → 성공한 split이 pagination을
  소유했음을 반영하는 명시적 상태 전이가 필요하다.
- IME 경로: `input-handler-text.ts` `processPendingNav`의 `code === 'Enter'`는 **조합 확정만
  하고 문단 분할이 없다**(169~171행). 구조 명령이 뒤따르지 않으므로 이 flush는 최신
  모델의 유일한 pagination barrier다 — 중복이 아예 없고, 취소하면 오히려 pagination이
  runner 완주까지 표류한다. Stage 2 admission은 direct keydown 경로에만 연다(fail-closed).
  IME 경로의 flush 비용 자체는 fragment 증분 재개(후속)의 몫이다.

## 6. 다음 단계 판정

1. Stage 2(이 브랜치): direct cell Enter 한정 command-aware admission —
   flush 대신 runner/timer 취소 + `wasm.cancelDeferredPagination()`, 성공한 split이
   pagination·cursor geometry invalidation을 소유, 실패 시 기존 full-flush로 fail-closed.
   기대 효과: pending Enter 약 2×→1×(cold Enter와 동급).
2. cold Enter 자체(~1초, wasm은 그 이상)는 §3의 fragment 전량 drain이 지배 항이다.
   해소는 보드 Task #8 본체인 "편집 행이 속한 fragment부터 재개, 상위 fragment 재사용"
   증분 재조판이며 PR #4122(canonical cell unit + 재귀 cursor) 병합 후 착수한다(별도 후속).
3. Stage A probe는 before/after 비교가 끝날 때까지 진단 자산으로 유지한다.

## 7. 검증

| 게이트 | 결과 |
|---|---|
| probe `--no-run` 빌드 | 통과 |
| probe 2 tests × 3 repeats (HWP/HWPX) | 2 passed, 0 failed (236.39s) |
| page count 정합 단언 (A=B, 전 회) | 통과 |
| `cargo fmt --all -- --check` | 커밋 전 확인 |
