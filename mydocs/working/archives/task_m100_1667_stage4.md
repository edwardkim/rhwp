# Task M100 #1667 Stage 4 보고서

## 단계 목표

Build & Test `Dirty rhwp` / target cache 실효성 분석의 1차 구현 후보로, cargo 명령과 step 이름은 유지한 채
기본 feature test step을 `Build` 직후로 재배치했다.

## 변경 파일

- `.github/workflows/ci.yml`
- `mydocs/plans/task_m100_1667_v3.md`
- `mydocs/plans/task_m100_1667_impl_v3.md`
- `mydocs/working/task_m100_1667_stage3.md`
- `mydocs/working/task_m100_1667_stage4.md`
- `mydocs/orders/20260703.md`

## 구현 내용

기존 주요 순서:

1. `Build`
2. `Check WASM target`
3. `Install native Skia runtime packages`
4. `Native Skia tests`
5. `Run lib tests`
6. `Run integration tests`

변경 후 주요 순서:

1. `Build`
2. `Run integration tests`
3. `Run lib tests`
4. `Check WASM target`
5. `Install native Skia runtime packages`
6. `Native Skia tests`

## 변경하지 않은 것

- `Build` 명령은 유지했다.
- `Run integration tests` 명령은 유지했다.
- `Run lib tests` 명령은 유지했다.
- `Native Skia tests` 명령은 유지했다.
- cache restore/save action, key, path는 유지했다.
- branch protection / required check는 변경하지 않았다.
- `tests/**`, `tests/golden_svg/**`와 회귀 가드 파일은 변경하지 않았다.

## 의도

`Build` 직후 기본 feature의 `Run integration tests`를 먼저 실행해, restored `target`과 방금 생성된
`release-test` 산출물이 integration test에서 재사용될 수 있는지 확인한다.

`Run lib tests`는 test harness 산출물을 만들기 때문에 compile이 남을 가능성이 높고, `Native Skia tests`는
`native-skia skia` feature set이 달라 별도 compile이 정상 비용이다. 따라서 두 step은 integration test
관측 뒤로 배치했다.

## 검증

로컬에서 수행할 수 있는 검증은 다음으로 제한한다.

- YAML diff 확인
- step 이름과 cargo 명령 유지 여부 확인
- `git diff --check`

실제 효과 판단은 PR CI after 관측에서 수행한다.

## PR CI 관측 포인트

PR CI 완료 후 다음을 기록한다.

- PR checks 완료 시간
- `CI / Build & Test` job 시간
- `Build` step 시간과 `Dirty rhwp` 여부
- `Run integration tests` step 시간, `Dirty rhwp` 여부, test binary 수
- `Run lib tests` step 시간과 test harness compile 여부
- `Native Skia tests` step 시간과 feature compile 여부
- cache exact hit / save skipped 여부
- cache size
- cache reservation/read-only/save failure 경고 여부
- branch protection / required check 변경 없음
- 회귀 가드 1:1 추적성 보존 여부

## 판단 기준

성공으로 볼 수 있는 경우:

- `Run integration tests` 시간이 before 표본보다 의미 있게 줄어든다.
- `Run integration tests`에서 `Dirty rhwp`가 사라지거나 원인이 줄어든다.
- 전체 Build & Test 시간이 악화되지 않는다.
- test binary 수와 issue 계열 회귀 가드 수가 유지된다.

효과 없음 또는 중단으로 볼 수 있는 경우:

- `Run integration tests`의 `Dirty rhwp`가 그대로 반복된다.
- 전체 Build & Test 시간이 악화된다.
- 실패 위치 가시성이 나빠진다.
- 회귀 가드 추적성이 깨진다.

## 다음 단계

1. 코드 PR을 생성한다.
2. PR CI 완료 후 after 관측을 수행한다.
3. 효과가 있으면 #1667 measurement 문서에 반영한다.
4. 효과가 작으면 cache action 교체 대신 "target cache는 dependency reuse에는 유효하나 local crate reuse에는
   한계가 있다"는 결론으로 정리한다.
