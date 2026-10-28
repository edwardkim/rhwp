# task_m100_4947 stage 8: 소스 단위 테스트 차등 분리

## 목표

제품 소스의 `#[cfg(test)]`를 한 번에 외부로 옮겨 private 접근을 깨뜨리지 않고, 의존성 수준별로
분류해 점진적으로 integration suite에 이동할 수 있는 강제 가능한 기준선을 만든다.

## 배경

- `tests/`의 integration source는 32개 generated suite로 묶였지만 제품 소스의 unit test는 여전히
  lib test binary 하나에 집중되어 있다.
- 모든 `#[cfg(test)]`를 기계적으로 이동하면 private 항목과 `super::` 의존 테스트가 컴파일되지 않는다.
- PR #4953처럼 새 회귀 테스트가 계속 병합되므로 신규 테스트의 위치와 자동 배정 규칙이 필요하다.

## 차등 분류

| 분류 | 판정 기준 | 이동 정책 |
| --- | --- | --- |
| `integration_ready` | `super::`·`crate::` 의존이 없는 test module | 공개 API 기반 `tests/cases/` 이동 우선 |
| `test_support` | `crate::` 의존이 있는 test module | 최소 지원 API를 정리한 별도 단계에서 이동 |
| `white_box` | `super::` 또는 `use super` 의존이 있는 test module | private 불변식으로 소스에 유지 |

분류는 완전한 Rust 의미 분석을 대체하지 않는다. 이동 우선순위를 안정적으로 정하고 신규 소스 테스트
증가를 막는 보수적 계약으로 사용한다.

## 구현

- `scripts/rust-unit-test-tiers.mjs`가 `src/**/*.rs`의 test module과 test support 항목을 수집한다.
- `tests/suites/unit-test-tiers.json`은 기존 모듈별 test 수와 허용된 support 항목의 기준선이다.
- `--check`는 새 모듈·support 항목 및 기존 모듈 test 수 증가를 거부하고 감소는 허용한다.
- 새 공개 회귀 테스트는 `tests/cases/`에 작성하며 integration manifest가 재귀 수집한다.
- 기존 `tests/` 최상위 원본은 대규모 경로 변경 없이 유지하고 이후 단계에서 점진적으로 옮긴다.
- PR #4953에서 추가된 integration source도 최신 `upstream/devel` 동기화 뒤 generated suite에 반영한다.

## 검증 명령

```bash
node --test scripts/tests/rust-unit-test-tiers.test.mjs
node scripts/rust-unit-test-tiers.mjs --check
node --test scripts/tests/rust-test-suite-manifest.test.mjs
node scripts/rust-test-suite-manifest.mjs --check
python3 -m unittest scripts/tests/test_ci_impact_workflow.py
```

## 결과

- 소스 unit test는 private 의존성에 따라 차등 관리되고 무조건적인 외부 이동을 피한다.
- 새 integration test는 파일 수를 늘려도 Cargo test binary 수를 늘리지 않는다.
- 로컬과 CI가 같은 두 manifest 계약을 검사한다.
- 최신 `upstream/devel`의 기준선은 소스 test 4,224개, cfg(test) module 298개이다.
- tier별 test는 `integration_ready` 0개, `test_support` 87개, `white_box` 4,133개이며
  나머지 4개는 독립 cfg(test) support item 안의 test이다.
- PR #4953의 신규 integration source 3개를 자동 배정한 뒤 561개 source, 2,471개 정적 test
  attribute, 32개 suite와 8개 exception으로 정리됐다.
- 실제 `nextest list`는 총 6,551개(ignored 38개, runnable 6,513개)이며 manifest 최소 래칫을
  6,551개로 올렸다.
