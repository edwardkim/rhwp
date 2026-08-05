# Task M100 #4075 Stage 1 - Native Skia 병렬화와 nextest 비용 모델 구현

- 이슈: [#4075](https://github.com/edwardkim/rhwp/issues/4075)
- 브랜치: task/4075-ci-cost-aware-shards
- 시작 기준: 955910136
- 기록일: 2026-08-05 KST
- 상태: 완료

## 목표

Native Skia 결과를 aggregate의 필수 판정으로 유지하면서도 archive가 준비된 default-feature shard의
시작을 막지 않는다. 성공한 devel 실행의 nextest suite 시간만 작은 비용 모델로 보존하여 다음
실행의 target 배정에 사용하고, raw 로그 또는 오래된 cache가 누적되지 않게 한다.

## 변경

- run-nextest-archives.yml은 archive 완료 즉시 shard를 시작하고 Native Skia job을 polling하는
  watcher를 실행한다. 실패·취소 등 성공 이외의 완료 상태가 관측되면 nextest process group만
  종료하고 해당 shard를 실패시킨다.
- nextest_cost_model.mjs는 structured nextest suite event를 Cargo target identity와 실행 시간으로
  요약하고, 최근 성공 결과만 EWMA로 병합한다.
- plan_nextest_target_archives.mjs는 모델이 있으면 실행 시간을 우선하는 LPT 배정과 source 크기
  동률 해소를, 없으면 기존 source-size + target 수 계약을 사용한다.
- 비용 모델은 devel의 전체 aggregate 성공 후에만 Actions cache로 갱신한다. raw event JSONL은
  runner 임시 경로에서 처리 후 삭제되고, cache-generation-sweep.yml은 이 self-managed namespace를
  건드리지 않는다.

## 검증 결과

### 1. 비용 모델 및 Native Skia watcher 단위 검증

- 명령: node --test scripts/tests/nextest_cost_model.test.mjs
- 결과: 5개 통과.
  - libtest-json-plus suite event의 target별 시간 합산
  - EWMA 비용 모델 병합과 제거된 target drop
  - 0밀리초 suite 허용
  - 실제 시간 우선 LPT archive 배정
  - Native Skia 성공/실패/대기 상태 판정

### 2. 상위 CI workflow 정적 검증

- 명령: python3 -m unittest scripts/tests/test_ci_impact_workflow.py
- 결과: 13개 통과.
  - 네 shard가 native-skia-tests를 직접 기다리지 않는지
  - 비용 model publisher가 successful devel push로 한정되는지
  - 매일 cache generation sweep이 self-managed 비용 model prefix를 제외하는지 확인했다.

### 3. 형식 및 workflow 검증

- 명령: Node syntax check 5개 script, actionlint 4개 workflow, git diff --check.
- 결과: 모두 exit code 0.

### 4. 실제 Cargo metadata planner 검증

- 명령: cargo metadata --format-version 1 --no-deps 후 planner 실행.
- 결과: rhwp test target 465개를 slow, 1, 2, 3 archive에 중복·누락 없이 배정했다.
  이력이 없는 첫 실행은 source-size-fallback을 선택하고 estimated_run_ms를 null로 기록한다.

## 잔여 원격 검증

이 Stage는 workflow와 planner의 로컬 계약을 고정한다. PR CI에서 archive 완료 뒤 shard 시작 시각,
Native Skia 실패 시 실행 중 shard 종료, devel 성공 뒤 cache model 갱신과 직전 CI 기준의 wall-clock
변화는 다음 원격 검증 단계에서 측정한다.
