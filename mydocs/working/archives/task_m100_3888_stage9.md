# Task M100 #3888 9단계 - four archive direct 배정과 builder 균형화

- 이슈: [#3888](https://github.com/edwardkim/rhwp/issues/3888)
- 대상 PR: [#3892](https://github.com/edwardkim/rhwp/pull/3892)
- 기준 CI: run `30826652305`, head `c2966b892`

## 관측

8단계 보정 뒤 four archive CI는 성공했다. `Build & Test`는 expected/run artifact 각각 4개를 받아
`5081 == 5081`을 확인했다. slow builder는 `slow`만 upload하고 `1`·`2`·`3` upload를 skip해 불필요한
archive 전송도 없었다.

그러나 run 전체는 17분 37초로 기준 run `30822674339`의 14분 52초보다 2분 45초 느렸다.

| job | 소요 |
| --- | --- |
| slow builder | 222초 |
| regular B (`3`) builder | 355초 |
| regular A (`1`·`2`) builder | 508초 |
| slow worker | 207초 |
| regular `1/3` worker | 140초 |
| regular `2/3` worker | 286초 |
| regular `3/3` worker | 184초 |

regular A/B의 300/150 target ownership이 build 불균형의 직접 원인이다. 또한 two-level planner는
regular archive target 수는 같아도 runnable count를 `3472`, `658`, `950`으로 편향시켰고, `2/3`에는
HWP5 baseline 58.367초, large baseline 35.033초, security corpus 60초 초과 test가 함께 들어갔다.

## 구현 계획

1. planner는 regular target을 builder group에 먼저 나누지 않고, `1`·`2`·`3` archive group에 직접
   source-size least-loaded 방식으로 배정한다. 세 archive의 target capacity 차이는 최대 1이다.
2. build job은 `slow+2`, `1`, `3` 세 개로 구성한다. slow builder는 Native Skia와 합쳐도 build 단계의
   동시 runner 수를 4개로 유지하면서 archive `2`를 함께 만든다. 나머지 두 builder는 regular archive를
   하나씩만 compile한다.
3. test worker는 slow, `1/3`, `2/3`, `3/3` 네 개를 독립 job으로 선언한다. 각 job은 정확히 하나의
   archive만 download한다. slow와 `2/3`은 같은 slow builder를 기다리며, `1/3`과 `3/3`은 각자 builder만
   기다린다.
4. 집계는 expected/run count artifact 각각 4개와 세 builder·네 worker 결과 확인을 유지한다.

## 수용 기준

- `slow`, `1`, `2`, `3` 네 archive 이외 artifact를 만들거나 worker가 download하지 않는다.
- regular builder의 ownership은 각각 archive 하나이고 target 수는 대략 150개로 균형화된다.
- `slow` builder가 Native Skia 완료 시점까지 끝나면 slow worker는 regular `1`·`3` builder를 기다리지
  않고 시작한다.
- 최신 CI의 builder 최대 시간이 8단계의 508초보다 줄고 expected/run 합계가 일치한다.

## 구현

- `plan_nextest_target_archives.mjs`는 builder group 선배정을 제거하고, source 크기 내림차순으로 정렬한
  regular target을 `1`·`2`·`3` archive의 가장 가벼운 group에 직접 배정한다.
- archive ownership은 `slow -> slow`, `1 -> a`, `2 -> slow`, `3 -> b`다. 따라서 build job은
  `slow+2`, `1`, `3`이며, 세 builder의 regular target ownership은 각각 150개 수준으로 정규화된다.
- CI caller는 matrix worker를 없애고 `test-regular-shard-1`, `-2`, `-3`을 독립 reusable-workflow
  job으로 선언했다. aggregate job은 네 worker 결과와 네 expected/run count artifact를 계속 확인한다.

## 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| 실제 `cargo metadata --no-deps` planner | 총 451 target을 `slow=1`, `1=150`, `2=150`, `3=150`으로 한 번씩 배정했다. regular source 크기는 1,487,111 / 1,467,713 / 1,464,541 bytes이며 capacity spread는 0이다. |
| builder ownership | `slow+2=151` target, `1=150`, `3=150` target으로 확인했다. 기존 A/B의 300/150 소유 구조가 남지 않았다. |
| 신규 target 합성 metadata | `future_default_feature_coverage`를 추가한 452 target metadata에서 archive `3`에 정확히 한 번 배정됐다. 신규 test도 별도 목록 갱신 없이 네 archive 중 하나에 포함된다. |
| planner 문법 | `node --check .github/scripts/plan_nextest_target_archives.mjs` 통과. |
| CI impact contract | `python3 -m unittest scripts/tests/test_ci_impact_workflow.py` 5건 통과, `node --test scripts/tests/ci-impact-classifier.test.cjs` 20건 통과. |
| workflow 정적 검사 | `actionlint` 1.7.12와 reusable build/worker 및 aggregate Bash `bash -n` 검사를 통과했다. |
| diff 검사 | `git diff --check` 통과. |

제품 Rust·TypeScript·fixture는 변경하지 않았다. 실제 nextest archive 생성, 네 worker coverage 합계와
builder/worker 시간은 이 commit의 최신 GitHub Actions에서 확인한다. 수용 기준의 508초 이하 builder
임계 경로는 원격 runner의 cold cache·queue 영향을 포함하므로 로컬 source-size 균형화만으로 성공으로
간주하지 않는다.
