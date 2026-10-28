# #6360 Stage 1: 개별 회귀 테스트 시간 기반 suite 및 archive 배분

## 문제

`regression_suite_NNN`은 파생 harness 이름일 뿐 고정된 실행 workload가 아니다. 기존 정책은
target 이름에 과거 총 시간을 연결했으므로, source가 다른 suite로 이동한 뒤에도 오래된 target
시간을 적용했다. 2026-08-29 devel run `33239619089`에서 B/C/D 예상시간은 약 888초였지만
C의 JUnit 누적은 약 2,503초로 확인됐다.

## 설계

- B/C/D JUnit artifact는 target 합계와 `target::module::test` 개별 시간을 함께 보관한다.
- 정책 v2는 개별 시간을 source module 단위로 합산한 `cases`와 원본 `test_cases`를 함께
  보관한다. 정책 갱신은 성공한 동일 devel run의 B/C/D 측정값만 사용한다.
- archive build는 현재 checkout의 source 목록과 `cases`를 사용해 32개 generated suite를
  다시 LPT 배정한다. 측정값이 없는 source는 `#[test]` 또는 `#[case]` 수당 60초의 보수적
  fallback을 쓴다.
- B/C/D 선택은 현재 생성된 `tests/suites/manifest.json`에서 suite별 source 시간 합계를 다시
  계산한다. 과거 `regression_suite_NNN` 이름에 붙은 시간은 v2에서 사용하지 않는다.
- v1 metrics ref는 migration 동안 읽을 수 있지만 suite 이름 기반 시간은 재배정에 쓰지 않는다.
  첫 성공 devel full lane이 v2 측정 artifact를 올리면 이후 CI부터 개별 측정값을 쓴다.
- 기존 green PR의 trusted review-tail post-merge artifact가 v1이면 재사용 판정과 artifact
  provenance는 그대로 인정하되, policy refresh는 v2 정책을 변경하지 않고 성공한다. 따라서
  #6297의 Render Diff merge-bridge 재사용 경로를 막거나 구형 suite 이름 측정을 되살리지 않는다.

## 검증 범위

- Node 계약 테스트는 JUnit 개별 시간 수집, source 합산, 현재 manifest 기반 suite 추정을 확인한다.
- Rust suite manifest 계약 테스트는 개별 source 시간으로 32-suite LPT 배정이 바뀌는지 확인한다.
- workflow 계약 테스트는 archive builder가 metrics policy를 받은 뒤 duration rebalance와
  current manifest selector를 실행하는지 확인한다.

## 한계와 다음 관찰

개별 테스트의 실제 실행시간 자체가 코드 변경으로 급증하는 첫 PR은 과거 측정값만으로 예측할 수
없다. 이 변경은 mutable suite 이름 때문에 발생한 잘못된 귀속을 제거한다. 첫 v2 devel full lane
후 B/C/D 실제시간과 새 policy의 예측값을 비교해 fallback과 장기 테스트 격리 필요성을 재평가한다.
