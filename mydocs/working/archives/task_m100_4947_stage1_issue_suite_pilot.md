# Task M100 #4947 Stage 1: issue 회귀 테스트 suite pilot

## 목적

Rust 전수 회귀의 테스트 수를 줄이지 않고 integration test 링크 단위를 줄인다. 최신
`upstream/devel@76e407b12`에는 integration test target이 558개 있으며, 이 target들은
`release-test`에서 각각 최적화·링크된다.

Stage 1은 상대경로 include, 공용 support module, native-skia 고정 CI 목록에 의존하지 않는
작은 `issue_*` 테스트 20개를 하나의 명시적 suite로 묶어 구조적 효과와 호환성을 확인한다.

## 변경

- 기존 최상위 test case 20개를 `tests/suites/issue_regression_pilot/` 아래로 이동했다.
- `tests/issue_regression_pilot.rs`가 각 case를 `#[path] mod`로 로드한다.
- case의 테스트 함수와 fixture는 변경하지 않았다.
- 모듈 경계가 유지되므로 helper 함수 이름이 중복돼도 충돌하지 않는다.
- nextest 결과에는 `issue_regression_pilot issue_xxx::test_name` 계보가 남는다.

## 결과

| 항목 | 변경 전 | 변경 후 |
| --- | ---: | ---: |
| integration test target | 558 | 539 |
| pilot case 파일 | 20 | 20 |
| pilot test 함수 | 24 | 24 |
| pilot test 실패 | 0 | 0 |

20개 독립 바이너리가 1개 suite가 되어 링크 target이 19개 줄었다. 테스트 삭제나 skip은
없었다.

## 검증

```text
cargo metadata --no-deps --format-version 1
  integration test target: 539

time cargo nextest run --cargo-profile release-test \
  --target-dir target/pr-review \
  --test issue_regression_pilot --test-threads 8 --no-fail-fast
  compile/link: 1m 12s
  nextest: 24 passed, 0 skipped, 0 failed
  test execution: 0.158s
  total: 1m 13.71s

cargo fmt --all -- --check
  exit 0

cargo clippy --target-dir target/pr-review \
  --test issue_regression_pilot -- -D warnings
  exit 0

git diff --check
  exit 0
```

동일 캐시에서도 실행 시간보다 컴파일·링크 시간이 압도적으로 길었다. 따라서 후속 단계는
테스트 함수 병합이나 실행 병렬도 변경보다 test target 수 축소를 우선한다.

## 개별 실행 계약

pilot에 포함된 case는 더 이상 독립 Cargo test target이 아니다. 기존
`cargo test --test issue_1035_alignment` 대신 suite binary와 nextest test filter를 사용한다.

```bash
cargo nextest run --test issue_regression_pilot \
  -E 'test(/issue_1035_alignment/)'
```

historical review 문서의 당시 실행 명령은 과거 기록이므로 일괄 수정하지 않는다. 활성 CI의
native-skia 고정 binary 목록과 nextest archive 생성은 이번 pilot 대상과 겹치지 않는다.

## 다음 단계

- pilot 전후의 clean/warm compile-link 시간을 동일 환경에서 별도 측정한다.
- 새 case 누락을 막는 결정적 suite manifest 검사를 추가한다.
- 효과가 확인되면 parser, serializer, renderer/layout, CLI/MCP, native-skia 경계로 확대한다.
- CI archive shard와 nextest filter가 binary 이름에 의존하는 곳은 suite 전환과 함께 수정한다.
