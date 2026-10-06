# Task M100 #3888 6단계 - 네 archive와 네 default-feature worker로 정규화

- 이슈: [#3888](https://github.com/edwardkim/rhwp/issues/3888)
- 대상 PR: [#3892](https://github.com/edwardkim/rhwp/pull/3892)
- 기준 CI: run `30822674339`, head `3e6088490`

## 관측

기준 CI는 전체 14분 52초에 성공했다. `slow`, `1`~`3`을 만든 builder A는 5분 15초,
`4`~`7`을 만든 builder B는 7분 33초였고, Native Skia는 6분 38초였다. slow worker의 실제
nextest 실행은 2분 42초였지만, 두 builder를 모두 `needs`로 둔 단일 matrix 때문에 B 완료까지
기다린 뒤 시작했다.

worker별 archive download는 약 4초로, 과거 full archive 반복 전송 병목은 이미 해소됐다. 따라서
실행 worker를 네 개로 줄이면서 `4`~`7` archive를 계속 만드는 것은 전송 최적화가 아니라 실행하지
않을 target compile과 artifact upload를 남기는 낭비다.

## 구현 계획

1. planner는 `slow`, `1`, `2`, `3` 네 label만 생성한다. `overflow_cell_baseline`은 `slow`에 단독
   배정하고 나머지 Cargo test target은 `1`~`3`에 빠짐없이 한 번씩 재배정한다.
2. builder는 물리적으로 세 job으로 분리한다. slow builder는 `slow`, regular A는 `1`·`2`, regular B는
   `3`만 생성·upload한다. archive 수와 실행 worker 수는 모두 네 개다.
3. slow worker는 slow builder와 Native Skia만 기다려 조기 실행한다. regular worker는 `1/3`, `2/3`,
   `3/3`이며 각각 archive 하나만 download하고 하나의 nextest Summary를 기록한다.
4. 집계 job은 expected count artifact 4개와 worker run count artifact 4개의 합계를 비교한다. builder와
   worker 결과도 개별적으로 필수 성공으로 확인한다.
5. 재사용 workflow의 worker 입력은 단수 `archive_label`만 받는다. 복수 archive download 입력을 제거해
   네 worker 구조에서 불필요한 artifact를 다시 내려받을 수 없게 한다.

## 수용 기준

- CI가 `test-archive-slow`, `test-archive-1`, `test-archive-2`, `test-archive-3`만 만든다.
- `Default-feature tests`는 slow와 일반 `1/3`~`3/3`, 총 네 worker만 실행한다.
- 각 worker는 자신이 실행하는 archive 하나만 download한다.
- four archive target에 중복·누락이 없고, expected/run count artifact가 각각 정확히 네 개이며 합계가
  일치한다.
- slow worker는 regular builder A/B의 완료를 기다리지 않는다. Native Skia 실패 시 네 worker 모두
  실행되지 않는다.
- 최신 PR CI에서 네 archive의 실제 build/worker 시간과 전체 임계 경로를 기준 14분 52초와 비교한다.

## 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| 실제 `cargo metadata --no-deps` planner | 통과: test target 451개, `slow` 1개와 일반 `1`·`2`·`3` 각 150개, `.args` 파일 4개, target 중복·누락 없음 |
| builder ownership | 통과: `slow`=`slow`, `1`·`2`=`a`, `3`=`b`; `4`~`7` label은 생성되지 않음 |
| 미래 target synthetic metadata | 통과: test-enabled target 1개를 추가한 452개 입력에서 새 `test:future_default_feature_coverage`가 archive `3`에 배정됨 |
| `node --check` planner | 통과 |
| `actionlint` 1.7.12 | 통과: caller, 두 reusable workflow 포함 |
| builder/worker/aggregate shell `bash -n` | 통과 |
| legacy label 정적 검색 | 통과: workflow와 planner에 `4`~`7`, 일반 7-shard, expected count 8개 참조 없음 |
| `git diff --check` | 통과 |

제품 Rust/TypeScript/fixture는 바꾸지 않았다. 전체 nextest 실행, archive별 실제 runnable count와 네 worker
Summary 합계, builder/worker 시간과 전체 임계 경로는 최신 PR CI에서 검증한다.
