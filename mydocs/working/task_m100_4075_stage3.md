# Task M100 #4075 Stage 3 - 비용 model 기반 3-shard 전환과 외부 fork 읽기 허용

- 이슈: [#4075](https://github.com/edwardkim/rhwp/issues/4075)
- 브랜치: `pr/issue-4075-nextest-cost-shards`
- 시작 기준: `f35e4046a` (fix(ci): 동일 저장소 PR 비용 수집 허용)
- 기록일: 2026-08-05 KST
- 상태: 로컬 구현·검증 완료, 원격 CI 대기

## 문제

초기 비용 model이 없을 때에는 `overflow_cell_baseline`을 `slow` 전용 archive로 분리하는 보수적
fallback이 필요하다. 그러나 model이 유효하면 이 target의 실제 실행 시간도 다른 target과 함께 알 수
있으므로, 별도 `slow` worker를 유지하면 네 runner 중 하나가 불균형하게 오래 걸릴 수 있다.

또한 외부 fork PR은 GitHub의 read-only `GITHUB_TOKEN` 제약 때문에 canonical cache를 갱신할 수 없다.
그렇더라도 base branch cache를 복원하여 같은 배정 결과를 사용해야 한다.

## 구현 계획

1. 유효한 nextest 비용 model이 있으면 모든 Cargo test target을 `1`, `2`, `3`, `4` archive에 LPT 방식으로
   배정한다. `slow` archive와 worker는 만들지 않되, runner 수는 네 개로 유지한다.
2. model이 없거나 손상된 경우에는 기존 `slow + 1 + 2 + 3` fallback을 유지한다.
3. archive builder reusable workflow가 `has_slow_archive`를 output으로 공개하고, main workflow는 이 값이
   true일 때만 slow worker와 네 번째 count를 요구한다.
4. 외부 fork PR도 base/default branch cache를 read-only로 복원해 model 기반 3-shard를 사용한다. raw
   runtime slice와 cache 갱신은 write 권한이 있는 devel push 또는 동일 저장소 PR만 수행한다.
5. Node planner test와 workflow 정적 test로 model/fallback 배정, optional slow worker, 외부 fork read-only
   계약을 고정한다.

## 성공 기준

- model이 있는 plan에는 `slow` archive가 없고 모든 target이 1/2/3/4에 정확히 한 번 배정된다.
- model이 없는 plan에는 `slow` archive가 남는다.
- aggregate 검증은 model mode와 fallback 모두 4개 count/artifact를 요구하되, model mode의 네 번째는
  `slow`가 아닌 일반 shard `4`다.
- 외부 fork PR은 publisher 없이 base branch 비용 model을 복원해 shard 배정에 사용할 수 있다.

## 검증 결과

### 1. planner unit test

- 명령: `node --test scripts/tests/nextest_cost_model.test.mjs`
- 결과: 6개 통과. 유효 model에서 `slow` target을 포함한 1/2/3/4 배정과 model 부재 시
  `slow + 1/2/3` fallback을 각각 확인했다.

### 2. workflow 정적 계약

- 명령: `python3 -m unittest scripts/tests/test_ci_impact_workflow.py`
- 결과: 16개 통과. slow/4 상호 배타 worker 결과, 네 archive 집계, 외부 fork read-only restore와
  writable caller만 publisher를 실행하는 계약을 확인했다.

### 3. workflow 문법·형식

- 명령: `git diff --check`, `actionlint .github/workflows/ci.yml
  .github/workflows/build-nextest-archives.yml .github/workflows/run-nextest-archives.yml
  .github/workflows/cache-generation-sweep.yml`, `node --check
  .github/scripts/plan_nextest_target_archives.mjs`
- 결과: 모두 exit code 0.

## 원격 검증 조건

1. devel model을 restore한 PR CI에서 `test-slow-shard`가 skipped이고 regular 1/2/3/4가 모두
   실행되는지 확인한다.
2. model이 없을 때에는 slow와 regular 1/2/3이 실행되고 regular 4가 skipped인지 확인한다.
3. 동일 저장소 PR publisher가 compact model cache를 남기고, 이후 devel push가 external fork도
   read-only로 restore할 수 있는 canonical model을 갱신하는지 확인한다.

## 원격 검증 결과와 다음 보정

- 실행: PR #4076 CI run `31013386357`.
- 결과: fallback mode는 정상이다. `test-slow-shard`와 regular 1/2/3은 성공했고 regular 4는
  skipped였다. runtime cost slice 4개도 upload됐다.
- 문제: `Publish nextest cost model`만 skipped여서 cache가 저장되지 않았다. 동일 저장소 PR 조건은
  worker의 slice 수집에서는 참으로 평가됐으므로, publisher의 복합 job-level `if` 평가 경로를
  별도 단계의 repository ID 비교로 분리한다.
