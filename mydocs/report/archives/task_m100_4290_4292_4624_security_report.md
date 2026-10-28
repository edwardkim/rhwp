# Task m100_4290_4292_4624 결과 - 입력 경계와 WMF 초기화 안전성

## 완료 범위

- #4290: HWP5 polygon/curve point-count를 남은 좌표 payload 바이트 수로 제한했다.
- #4292: numbering start value `0`에서 unsigned underflow가 나지 않도록 `saturating_sub`를 적용했다.
- #4624: WMF symbol/codepage table의 `AtomicBool + static mut` 수동 lazy initialization을 immutable `OnceLock`으로 교체했다.

## 구현 결과

| 이슈 | 변경 | 회귀 보장 |
| --- | --- | --- |
| #4290 | 양수 count를 `remaining / 8`로 clamp | `i32::MAX` count와 좌표 한 쌍이 실제 한 점만 읽는지 확인 |
| #4292 | `(start - 1) + counter`를 `start.saturating_sub(1) + counter`로 교체 | start `0`, counter `1`이 `1.`을 생성하는지 확인 |
| #4624 | 두 전역 table을 `OnceLock<BTreeMap<...>>`로 교체 | 16개 native thread가 production lookup을 동시에 수행해도 HANGUL `949`, GREEK `1253` 매핑을 유지하는지 확인 |

## 이슈 선별 및 제외

- #4649는 upstream commit `7789998a5`에서 이미 해결됐다.
- #4291은 최근 HWPX container depth guard로 반영됐으며 남은 #4730의 generic recursive call graph는 별도 설계 과제다.
- #4739, #4709, #4668, #4669, #4618은 fidelity, font, serializer 또는 fallback 계약 검증이 필요해 이번 안전성 PR에서 제외했다.

## 검증 결과

- focused tests: #4290, #4292, #4624 각각 통과.
- 전체 명령: `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`.
  - 5,935개 중 5,934개 통과, 37 skipped, 7 slow.
  - 유일한 실패 `issue_2833_hml_adapter_row_sizes::inflated_row_count_does_not_slow_down_parsing`는 0.573초의 병렬 실행 시간 예산 실패였다.
  - 같은 `target/pr-review`에서 단독 재실행은 0.079초, 1 passed로 통과했다. 변경 경로와 무관한 일시적 성능 측정 변동으로 판정했다.
- `cargo fmt --check`, `cargo clippy --target-dir target/pr-review --all-targets -- -D warnings`, `git diff --check` 통과.

## 후속

- generic parser recursion의 shared depth 계약은 #4730에서 별도로 다룬다.
- 이번 PR은 input hardening과 native initialization soundness만 포함하며 format fidelity 기준이나 공개 API를 바꾸지 않는다.
