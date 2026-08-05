# Task M100 #4075 Stage 5 - 수동 full CI 비용 model 발행

- 이슈: [#4075](https://github.com/edwardkim/rhwp/issues/4075)
- 브랜치: `pr/issue-4075-nextest-cost-shards`
- 시작 기준: `ed621d73b` (ci: 비용 model 발행을 집계 단계에 통합)
- 기록일: 2026-08-05 KST
- 상태: 로컬 구현·검증 완료, 원격 CI 대기

## 문제

PR synchronize가 자동 CI를 만들지 않은 상태에서 `workflow_dispatch`로 full CI를 시작했다. 그러나
현재 runtime slice 수집과 cache 발행 조건은 devel push와 동일 저장소 `pull_request`만 허용하여
수동 full CI는 model을 생성하지 못한다.

## 구현 계획

1. repository write 권한이 필요한 `workflow_dispatch`를 trusted model 수집·발행 경로에 추가한다.
2. `Build & Test` 내부 scope 판정은 workflow dispatch에서 cache 발행을 허용하고, pull request에서는
   계속 repository ID가 같은 경우만 허용한다.
3. 외부 fork의 `pull_request`는 비용 model을 read-only로 복원하고 cache write/delete를 하지 않는
   기존 경계를 유지한다.
4. workflow 정적 test와 actionlint를 실행한 뒤 같은 branch ref의 수동 full CI로 cache 생성과
   1/2/3/4 재실행을 검증한다.

## 성공 기준

- `workflow_dispatch` full CI가 runtime cost slice와 cache를 생성한다.
- 동일 ref 재실행에서 model을 restore해 slow가 skipped, regular 1/2/3/4가 모두 실행된다.
- 외부 fork pull request는 read-only 조건을 유지한다.

## 검증 결과

### 1. remote 실행 경로 확인

- 관측: Stage 4 commit push 뒤에는 PR CI가 자동 생성되지 않고 `pull_request_target` 기반
  stale-run 정리 workflow만 생성됐다. 같은 ref의 `workflow_dispatch` full CI는 정상 queue됐다.
- 보정: 수동 full CI도 repository write 권한자가 실행하는 경로이므로 runtime slice 수집과
  `Build & Test` cache 발행 scope에 포함했다.

### 2. 로컬 검증

- 명령: `node --test scripts/tests/nextest_cost_model.test.mjs`
- 결과: 6개 통과.
- 명령: `python3 -m unittest scripts/tests/test_ci_impact_workflow.py`
- 결과: 16개 통과. 수동 full CI 수집·발행 조건과 외부 fork read-only 계약을 포함한다.
- 명령: `git diff --check`, `actionlint .github/workflows/ci.yml
  .github/workflows/build-nextest-archives.yml .github/workflows/run-nextest-archives.yml
  .github/workflows/cache-generation-sweep.yml`
- 결과: 모두 exit code 0.
