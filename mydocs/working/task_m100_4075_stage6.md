# Task M100 #4075 Stage 6 - 비용 slice 보존 순서 보정

- 이슈: [#4075](https://github.com/edwardkim/rhwp/issues/4075)
- 브랜치: `pr/issue-4075-nextest-cost-shards`
- 시작 기준: `7c73ee04b` (ci: 수동 full 실행에서 비용 model 수집)
- 기록일: 2026-08-05 KST
- 상태: 로컬 구현·검증 완료, 원격 CI 대기

## 문제

`Build & Test`가 runtime slice artifact를 받은 뒤 source checkout을 수행했다. checkout의 기본 clean이
download directory를 삭제해 model merge가 `ENOENT`로 실패했다.

## 구현 계획

1. source checkout을 runtime slice download보다 먼저 실행한다.
2. 정적 workflow test에 checkout이 slice download보다 앞선 순서를 고정한다.
3. 같은 PR full CI에서 model cache 생성이 성공하는지 확인하고, 동일 head 재실행으로 regular
   1/2/3/4 mode를 검증한다.

## 성공 기준

- checkout 뒤 slice directory가 유지되어 merge와 cache save가 성공한다.
- cache 적중 재실행에서 slow는 skipped, regular 1/2/3/4는 success다.

## 검증 결과

- 명령: `node --test scripts/tests/nextest_cost_model.test.mjs`
- 결과: 6개 통과.
- 명령: `python3 -m unittest scripts/tests/test_ci_impact_workflow.py`
- 결과: 16개 통과. source checkout이 runtime slice download보다 앞에 있어야 한다는 순서 계약을
  추가했다.
- 명령: `git diff --check`, `actionlint .github/workflows/ci.yml
  .github/workflows/build-nextest-archives.yml .github/workflows/run-nextest-archives.yml
  .github/workflows/cache-generation-sweep.yml`
- 결과: 모두 exit code 0.
