# Task M100 #1664 구현 계획서

## 구현 원칙

- #1664는 cache 정책 정리만 수행한다.
- PR CI의 검증 의미, test command, profile, job 구조는 바꾸지 않는다.
- PR은 cache restore-only로 두고, cache save는 `devel` / `main` trusted branch에서만 허용한다.
- `Swatinem/rust-cache` 도입, `Cargo.toml` profile 변경, job 병렬화는 후속 이슈 범위로 분리한다.
- 회귀 가드 162개가 PR마다 모두 실행되는 도메인 제약을 보존한다.
- 이 구현 계획서는 문서 PR #1701에 포함한다. 실제 `.github/workflows/ci.yml` 변경은 후속 코드 PR #1702에서
  별도로 진행하며, #1701이 merge되어도 workflow 변경이 `devel`에 반영된 것은 아니다.

## Stage 1

후속 코드 PR #1702에서 `Build & Test` job의 cargo cache step을 restore/save 분리 구조로 변경한다.

- `.github/workflows/ci.yml`
  - 기존 `actions/cache@v5` 단일 step을 제거한다.
  - `actions/cache/restore@v5` step을 추가한다.
  - restore step에는 기존 path, primary key, restore key를 그대로 사용한다.
  - restore step에 `id`를 부여해 exact cache hit 여부를 후속 save 조건에서 참조할 수 있게 한다.
  - step 이름은 로그에서 restore/save 정책이 구분되도록 정리한다.

## Stage 2

후속 코드 PR #1702에서 trusted branch save 조건을 추가하고 PR save를 차단한다.

- `.github/workflows/ci.yml`
  - test/build/clippy 단계 이후에 `actions/cache/save@v5` step을 추가한다.
  - save step은 push 이벤트의 `refs/heads/devel` 또는 `refs/heads/main`에서만 실행되게 한다.
  - exact cache hit가 아닌 경우에만 save를 시도해 동일 key 저장 충돌과 불필요한 save 로그를 줄인다.
  - save step에는 restore step과 같은 path와 primary key를 사용한다.
  - `pull_request`, tag, `workflow_dispatch`에서는 cache save가 실행되지 않게 한다.

## Stage 3

workflow 문법과 변경 범위를 검증하고, 측정 기준을 stage 보고서에 기록한다.

- 로컬 검증
  - `git diff --check`
  - `actionlint .github/workflows/ci.yml` 사용 가능 시 실행
  - actionlint가 없으면 YAML 파싱 수준의 대체 검증을 수행하고 한계를 기록
- 변경 범위 확인
  - `.github/workflows/ci.yml`
  - `mydocs/orders/20260630.md`
  - `mydocs/plans/task_m100_1664.md`
  - `mydocs/plans/task_m100_1664_impl.md`
  - `mydocs/working/task_m100_1664_stage1.md`
  - `mydocs/working/task_m100_1664_stage2.md`
  - `mydocs/working/task_m100_1664_stage3.md`
  - `mydocs/report/task_m100_1664_report.md`
- PR/CI 측정 항목
  - PR checks 완료 시간 (P50, P90)
  - `CI / Build & Test` job 시간
  - 주요 step 시간: build / lib test / integration test / native-skia
  - cache hit/miss/save 성공 여부
  - cache 크기
  - 실패 시 원인 가시성
  - runner-minutes 변화
  - branch protection / required check 변경 여부
  - 회귀 가드 162개가 PR마다 모두 실행되는지 확인

## 합격 기준

- PR run에서 cargo cache save step이 실행되지 않는다.
- `devel` / `main` push run에서만 cargo cache save step이 실행된다.
- 기존 cargo cache key와 restore key가 유지된다.
- `Build & Test`의 build/test/clippy command가 변경되지 않는다.
- `Build & Test` job 이름과 required check 표면이 변경되지 않는다.
- 통합 테스트 파일 구조, 회귀 가드 이름, golden 자산 구조가 변경되지 않는다.
- workflow 문법 검증과 whitespace 검증이 통과한다.

## 롤백 기준

- restore가 실패하거나 기존 fallback restore보다 명확히 나빠지는 경우
- trusted branch에서도 cache save가 실행되지 않는 경우
- PR에서 cache save가 계속 시도되는 경우
- required check 이름 또는 branch protection 표면이 바뀌는 경우

롤백은 restore/save 분리 step을 기존 `actions/cache@v5` 단일 step으로 되돌리는 방식으로 수행한다.

## 승인 요청 지점

이 구현 계획서 승인 후 Stage 1을 시작한다. Stage 1 완료 시
`mydocs/working/task_m100_1664_stage1.md`에 변경 내용과 검증 결과를 기록하고 다음 단계 진행 승인을 요청한다.
