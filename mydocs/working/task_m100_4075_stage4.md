# Task M100 #4075 Stage 4 - Build & Test 내부 비용 model 발행

- 이슈: [#4075](https://github.com/edwardkim/rhwp/issues/4075)
- 브랜치: `pr/issue-4075-nextest-cost-shards`
- 시작 기준: `1fa9b2c2b` (ci: 비용 model에서 일반 shard 네 개로 배정)
- 기록일: 2026-08-05 KST
- 상태: 로컬 구현·검증 완료, 원격 CI 대기

## 문제

별도 `Publish nextest cost model` job의 복합 job-level 조건은 동일 저장소 PR의 slice artifact가 모두
생성된 뒤에도 skipped되었다. 비용 model 발행은 본질적으로 shard 결과를 집계하는 `Build & Test`의
후속 작업이므로 별도 job 경계를 둘 이유가 없다.

## 구현 계획

1. publisher job을 제거하고, `Build & Test` 마지막에 scope 판정, slice download, model merge, cache
   save와 이전 cache 정리 단계를 옮긴다.
2. full CI를 통과한 동일 저장소 PR과 devel push만 repository ID 비교 후 cache를 갱신한다.
3. 외부 fork PR은 같은 `Build & Test` 안에서 read-only라고 명시하고 성공 종료한다. base/default
   branch model restore와 4-shard 실행은 그대로 허용하며 cache write/delete는 하지 않는다.
4. docs-only fast-pass에는 scope 판정과 cache 변경 단계를 모두 실행하지 않는다.
5. 정적 workflow test와 actionlint를 보강하고, 원격에서 fallback cache 생성 뒤 재실행하여
   model mode의 regular 1/2/3/4 실행을 확인한다.

## 성공 기준

- `Build & Test`가 성공한 동일 저장소 PR에서 nextest cost model cache를 생성한다.
- 외부 fork와 docs-only fast-pass는 cache write 없이 성공한다.
- 동일 head 재실행에서 `slow`가 skipped이고 regular 1/2/3/4가 성공한다.

## 검증 결과

### 1. planner unit test

- 명령: `node --test scripts/tests/nextest_cost_model.test.mjs`
- 결과: 6개 통과. Stage 3의 model/fallback 네 worker 배정 계약이 유지된다.

### 2. workflow 정적 계약

- 명령: `python3 -m unittest scripts/tests/test_ci_impact_workflow.py`
- 결과: 16개 통과. 별도 publisher job 제거, `Build & Test` tail 단계의 repository ID 판정,
  external fork read-only 경로를 확인했다.

### 3. workflow 문법·형식

- 명령: `git diff --check`, `actionlint .github/workflows/ci.yml
  .github/workflows/build-nextest-archives.yml .github/workflows/run-nextest-archives.yml
  .github/workflows/cache-generation-sweep.yml`, `node --check
  .github/scripts/plan_nextest_target_archives.mjs`
- 결과: 모두 exit code 0.
