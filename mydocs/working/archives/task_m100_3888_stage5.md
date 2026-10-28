# Task M100 #3888 5단계 - archive 4를 builder B로 재배정

- 이슈: [#3888](https://github.com/edwardkim/rhwp/issues/3888)
- 대상 PR: [#3892](https://github.com/edwardkim/rhwp/pull/3892)
- 기준 head: `5fb9c7bf`

## 관측

4단계 CI run `30821145911`에서 builder B는 `slow`과 `1`~`4` upload를 skip하고 `5`~`7`만
upload한 뒤 5분 45초에 완료됐다. 같은 시점에 builder A는 `slow`과 `1`~`4` archive 생성 중이었다.
A가 한 개 더 많은 일반 archive와 slow archive를 처리하는 구성이 임계 경로가 될 가능성이 확인됐다.

## 구현 계획

1. 일반 archive `4`의 ownership을 A에서 B로 옮긴다. A는 `slow`·`1`~`3`, B는 `4`~`7`을 만든다.
2. planner의 builder label 집합과 workflow matrix, archive 4 upload 조건을 함께 변경한다.
3. 일반 worker의 archive label과 총 8개 worker는 바꾸지 않는다. slow target은 계속 전용 archive다.
4. metadata planner로 target 중복·누락 없음, 일반 archive target 수 차이 1 이하, 새 A/B source bytes를
   확인한다. actionlint·shell 구문·실제 CI에서 builder 완료 시각과 worker 합계를 다시 확인한다.

## 수용 기준

- Builder A는 `slow`·`1`~`3`만 upload하고 `4`~`7`은 skip한다.
- Builder B는 `4`~`7`만 upload하고 `slow`·`1`~`3`은 skip한다.
- slow worker와 일반 7 worker, archive expected count 8개와 worker count 8개 합계 검증을 유지한다.
- 새 PR CI에서 두 builder의 완료 시각이 4단계보다 더 가깝고 전체 default-feature 테스트가 통과한다.

## 로컬 검증 결과

| 검증 | 결과 |
| --- | --- |
| 실제 `cargo metadata --no-deps` planner | 통과: target 450개, 일반 archive별 64~65 target, A/B source bytes 2,160,615 / 2,251,222 |
| ownership 정적 계약 | 통과: A=`slow`·`1`~`3`, B=`4`~`7`, slow target 단독, target 중복·누락 없음 |
| `actionlint .github/workflows/ci.yml` | 통과 |
| builder/worker/aggregate shell `bash -n` | 통과 |
| `git diff --check` | 통과 |

제품 코드와 test fixture는 바꾸지 않았다. 4단계 CI는 관측 근거로 계속 두고, 이 보정 head를 즉시
push한다. 5단계 CI에서 builder 완료 시각, upload `success/skipped` 분포, archive expected count와
worker run count 합계를 확인한다.
